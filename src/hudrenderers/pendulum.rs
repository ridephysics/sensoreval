use crate::kalman::ukf::Functions;
use crate::*;
use eom::traits::Scheme;
use eom::traits::TimeEvolution;
use ndarray::array;
use ndarray::azip;
use ndarray::s;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
/// initial conditions for UKF
pub struct Initial {
    /// angular position, unit: rad
    position: f64,
    /// angular velocity, unit: rad/s
    velocity: f64,
    /// pendulum radius, unit: meters
    radius: f64,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    /// standard deviation of the measurements, used for matrix R
    pub stdev: config::SensorStdev,
    /// initial conditions, used for vector x
    pub initial: Initial,
    /// unit: rad
    #[serde(default)]
    pub orientation_offset: f64,
    /// unit: meters
    #[serde(default)]
    pub radius_override: Option<f64>,
    #[serde(default)]
    pub enable_rts_smoother: bool,
}

pub(crate) struct Pendulum {
    cfg: Config,
    est: Vec<ndarray::Array1<f64>>,
}

pub struct StateFunctions<'a> {
    cfg: &'a Config,
}

impl<'a> StateFunctions<'a> {
    pub fn new(cfg: &'a Config) -> Self {
        Self { cfg }
    }
}

impl<'a> kalman::ukf::Functions for StateFunctions<'a> {
    type Elem = f64;

    fn fx<S>(
        &self,
        x: &ndarray::ArrayBase<S, ndarray::Ix1>,
        dt: Self::Elem,
    ) -> ndarray::Array1<Self::Elem>
    where
        S: ndarray::Data<Elem = Self::Elem>,
    {
        let pa = x[0];
        let va = x[1];
        let r = match self.cfg.radius_override {
            Some(v) => v,
            None => x[2],
        };

        let mut teo =
            eom::explicit::RK4::new(crate::simulator::pendulum::EomFns::from_radius(r), dt);
        let mut ic = array![pa, va];
        let next = teo.iterate(&mut ic);

        array![math::normalize_angle(next[0]), next[1], r]
    }

    #[allow(non_snake_case)]
    fn x_mean<Ss, Swm>(
        &self,
        sigmas: &ndarray::ArrayBase<Ss, ndarray::Ix2>,
        Wm: &ndarray::ArrayBase<Swm, ndarray::Ix1>,
    ) -> ndarray::Array1<Self::Elem>
    where
        Ss: ndarray::Data<Elem = Self::Elem>,
        Swm: ndarray::Data<Elem = Self::Elem>,
    {
        let mut ret = Wm.dot(sigmas);

        let mut pa_sin = 0.0;
        let mut pa_cos = 0.0;

        azip!((sp in sigmas.genrows(), w in Wm) {
            assert!(sp[0] >= -std::f64::consts::PI && sp[0] <= std::f64::consts::PI);

            pa_sin += sp[0].sin() * w;
            pa_cos += sp[0].cos() * w;
        });

        ret[0] = pa_sin.atan2(pa_cos);

        ret
    }

    fn x_residual<Sa, Sb>(
        &self,
        a: &ndarray::ArrayBase<Sa, ndarray::Ix1>,
        b: &ndarray::ArrayBase<Sb, ndarray::Ix1>,
    ) -> ndarray::Array1<Self::Elem>
    where
        Sa: ndarray::Data<Elem = Self::Elem>,
        Sb: ndarray::Data<Elem = Self::Elem>,
    {
        let mut res = a - b;
        res[0] = math::normalize_angle(res[0]);
        res
    }

    fn hx<S>(&self, x: &ndarray::ArrayBase<S, ndarray::Ix1>) -> ndarray::Array1<Self::Elem>
    where
        S: ndarray::Data<Elem = Self::Elem>,
    {
        let pa = x[0];
        let va = x[1];
        let r = x[2];
        let ac = va.powi(2) * r;

        array![
            0.0,
            0.0,
            ac + math::GRAVITY * (pa + self.cfg.orientation_offset).cos(),
            va,
            0.0,
            0.0
        ]
    }

    #[allow(non_snake_case)]
    fn z_mean<Ss, Swm>(
        &self,
        sigmas: &ndarray::ArrayBase<Ss, ndarray::Ix2>,
        Wm: &ndarray::ArrayBase<Swm, ndarray::Ix1>,
    ) -> ndarray::Array1<Self::Elem>
    where
        Ss: ndarray::Data<Elem = Self::Elem>,
        Swm: ndarray::Data<Elem = Self::Elem>,
    {
        Wm.dot(sigmas)
    }

    fn z_residual<Sa, Sb>(
        &self,
        a: &ndarray::ArrayBase<Sa, ndarray::Ix1>,
        b: &ndarray::ArrayBase<Sb, ndarray::Ix1>,
    ) -> ndarray::Array1<Self::Elem>
    where
        Sa: ndarray::Data<Elem = Self::Elem>,
        Sb: ndarray::Data<Elem = Self::Elem>,
    {
        a - b
    }
}

impl<'a> kalman::sigma_points::Functions for StateFunctions<'a> {
    type Elem = f64;

    fn subtract<Sa, Sb>(
        &self,
        a: &ndarray::ArrayBase<Sa, ndarray::Ix1>,
        b: &ndarray::ArrayBase<Sb, ndarray::Ix1>,
    ) -> ndarray::Array1<Self::Elem>
    where
        Sa: ndarray::Data<Elem = Self::Elem>,
        Sb: ndarray::Data<Elem = Self::Elem>,
    {
        self.x_residual(a, b)
    }
}

impl Pendulum {
    pub fn new(_ctx: &render::HudContext, cfg: &Config) -> Self {
        Self {
            cfg: (*cfg).clone(),
            est: Vec::new(),
        }
    }

    #[inline]
    fn get_actual(data: &'_ Data) -> Option<&'_ simulator::pendulum::Actual> {
        if let data::ActualData::Pendulum(p) = data.actual.as_ref()?.as_ref() {
            return Some(p);
        }

        None
    }
}

impl render::HudRenderer for Pendulum {
    #[allow(non_snake_case)]
    fn data_changed(&mut self, ctx: &render::HudContext) {
        let samples = unwrap_opt_or!(ctx.get_dataset(), return);
        let fns = StateFunctions::new(&self.cfg);
        let points_fn = kalman::sigma_points::MerweScaledSigmaPoints::new(3, 0.1, 2.0, 0.0, &fns);
        let mut ukf = kalman::ukf::UKF::new(3, 6, &points_fn, &fns);

        ukf.x = array![
            self.cfg.initial.position,
            self.cfg.initial.velocity,
            self.cfg.initial.radius
        ];
        ukf.P = ndarray::Array::eye(3) * 0.0001;
        ukf.R = ndarray::Array2::from_diag(&array![
            self.cfg.stdev.accel.x.powi(2),
            self.cfg.stdev.accel.y.powi(2),
            self.cfg.stdev.accel.z.powi(2),
            self.cfg.stdev.gyro.x.powi(2),
            self.cfg.stdev.gyro.y.powi(2),
            self.cfg.stdev.gyro.z.powi(2),
        ]);

        self.est.clear();

        let mut t_prev = match samples.get(0) {
            Some(v) => v.time,
            None => 0,
        };
        let mut Ps = Vec::new();
        let mut Qs = Vec::new();
        let mut dts = Vec::new();
        for sample in samples {
            let z = array![
                sample.accel[0],
                sample.accel[1],
                sample.accel[2],
                sample.gyro[0],
                sample.gyro[1],
                sample.gyro[2]
            ];
            let dt = (sample.time - t_prev) as f64 / 1_000_000.0f64;

            ukf.Q
                .slice_mut(s![0..2, 0..2])
                .assign(&kalman::discretization::Q_discrete_white_noise(2, dt, 0.1).unwrap());
            ukf.Q[[2, 2]] = 0.0001;

            ukf.predict(dt);
            ukf.update(&z);

            self.est.push(ukf.x.clone());

            if self.cfg.enable_rts_smoother {
                Ps.push(ukf.P.clone());
                Qs.push(ukf.Q.clone());
                dts.push(dt);
            }

            t_prev = sample.time;
        }

        if self.cfg.enable_rts_smoother {
            let (xss, _) = ukf.rts_smoother(&self.est, &Ps, Some(&Qs), &dts);
            self.est = xss;
        }

        let mut radius_sum = 0.0;
        for x in &self.est {
            radius_sum += x[2];
        }

        println!("average radius: {}", radius_sum / self.est.len() as f64);
    }

    fn render(&self, _ctx: &render::HudContext, _cr: &cairo::Context) -> Result<(), Error> {
        Ok(())
    }

    fn plot(&self, ctx: &render::HudContext) -> Result<(), Error> {
        let samples = ctx.get_dataset().ok_or(Error::NoDataSet)?;

        let mut plot = Plot::new(
            "\
            t = np.array(load_data())\n\
            z = np.array(load_data())\n\
            est = np.array(load_data())\n\
            has_actual = load_data()\n\
            if has_actual:\n\
                \tactual = np.array(load_data())\n\
            \n\
            fig, plots = plt.subplots(5, sharex=True)\n\
            \n\
            plots[0].set_title('p_a', x=-0.15, y=0.5)\n\
            if has_actual:\n\
                \tplots[0].plot(t, np.degrees(actual[:, 0]), ca)\n\
            plots[0].plot(t, np.degrees(est[:, 0]), ce)\n\
            \n\
            plots[1].set_title('v_a', x=-0.15, y=0.5)\n\
            plots[1].plot(t, np.degrees(z[:, 3]), cz)\n\
            if has_actual:\n\
                \tplots[1].plot(t, np.degrees(actual[:, 1]), ca)\n\
            plots[1].plot(t, np.degrees(est[:, 1]), ce)\n\
            \n\
            plots[2].set_title('v_t', x=-0.15, y=0.5)\n\
            if has_actual:\n\
                \tplots[2].plot(t, actual[:, 2], ca)\n\
            plots[2].plot(t, est[:, 1] * est[:, 2], ce)\n\
            \n\
            plots[3].set_title('a', x=-0.15, y=0.5)\n\
            plots[3].plot(t, z[:, 2], cz)\n\
            if has_actual:\n\
                \tplots[3].plot(t, actual[:, 3], ca)\n\
            plots[3].plot(t, est[:, 3], ce)\n\
            \n\
            plots[4].set_title('r', x=-0.15, y=0.5)\n\
            plots[4].plot(t, (est[:, 2]), ce)\n\
            \n\
            fig.tight_layout()\n\
            plt.show()\n\
            ",
        )?;

        plot.write(&DataSerializer::new(&samples, |_i, v| v.time_seconds()))?;

        plot.write(&DataSerializer::new(&samples, |_i, v| {
            vec![
                v.accel[0], v.accel[1], v.accel[2], v.gyro[0], v.gyro[1], v.gyro[2],
            ]
        }))?;

        plot.write(&DataSerializer::new(&self.est, |_i, v| {
            let mut ret = v.to_vec();

            ret.push(
                ret[1].powi(2) * ret[2]
                    + math::GRAVITY * (ret[0] + self.cfg.orientation_offset).cos(),
            );

            ret
        }))?;

        let has_actual = match samples.first() {
            Some(sample) => Self::get_actual(&sample).is_some(),
            None => false,
        };
        plot.write(&has_actual)?;

        if has_actual {
            plot.write(&DataSerializer::new(&samples, |_i, v| {
                let actual = Self::get_actual(v).unwrap();
                vec![
                    actual.p_ang,
                    actual.v_ang,
                    actual.v_tan,
                    actual.ac + math::GRAVITY * (actual.p_ang + self.cfg.orientation_offset).cos(),
                ]
            }))?;
        }

        plot.wait()
    }
}

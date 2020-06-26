use crate::kalman::ukf::Functions;
use crate::render::HudRenderer;
use crate::*;
use eom::traits::Scheme;
use eom::traits::TimeEvolution;
use ndarray::array;
use ndarray::azip;
use ndarray::s;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    /// standard deviation of the measurements, used for matrix R
    pub stdev: config::SensorStdev,

    /// initial conditions, used for vector x
    pub initial: Vec<f64>,

    /// initial conditions, used for matrix P
    pub initial_cov: Vec<f64>,

    #[serde(default)]
    pub enable_rts_smoother: bool,

    pub active_row: usize,
}

pub struct StateFunctions;

impl Default for StateFunctions {
    fn default() -> Self {
        Self
    }
}

impl<'a> kalman::ukf::Functions for StateFunctions {
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
        let aa = x[2];
        let r = 14.6;

        let mut teo = eom::explicit::RK4::new(
            crate::simulator::pendulum::EomFns::for_est(r, 0.2743, aa),
            dt,
        );
        let mut ic = array![pa, va];
        let next = teo.iterate(&mut ic);

        array![math::normalize_angle(next[0]), next[1], aa]
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

        let mut pa_sum = math::SinCosSum::default();

        azip!((sp in sigmas.genrows(), w in Wm) {
            assert!(sp[0] >= -std::f64::consts::PI && sp[0] <= std::f64::consts::PI);
            pa_sum.add(sp[0], *w);
        });

        ret[0] = pa_sum.avg();

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

    fn x_add<Sa, Sb>(
        &self,
        a: &ndarray::ArrayBase<Sa, ndarray::Ix1>,
        b: &ndarray::ArrayBase<Sb, ndarray::Ix1>,
    ) -> ndarray::Array1<Self::Elem>
    where
        Sa: ndarray::Data<Elem = Self::Elem>,
        Sb: ndarray::Data<Elem = Self::Elem>,
    {
        let mut res = a + b;
        res[0] = math::normalize_angle(res[0]);
        res
    }

    fn hx<S>(&self, x: &ndarray::ArrayBase<S, ndarray::Ix1>) -> ndarray::Array1<Self::Elem>
    where
        S: ndarray::Data<Elem = Self::Elem>,
    {
        let pa = x[0];
        let va = x[1];
        let r = 14.6;
        let ac = va.powi(2) * r;

        let accel = nalgebra::Vector3::new(0.0, 0.0, ac + math::GRAVITY * (pa).cos());
        let gyro = nalgebra::Vector3::new(va, 0.0, 0.0);

        array![accel[0], accel[1], accel[2], gyro[0], gyro[1], gyro[2]]
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

impl<'a> kalman::sigma_points::Functions for StateFunctions {
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

pub(crate) struct Pendulum {
    cfg: Config,
    est: Vec<ndarray::Array1<f64>>,
    ys: Vec<ndarray::Array1<f64>>,
    font: pango::FontDescription,
    svg_speed: librsvg::SvgHandle,
    svg_height: librsvg::SvgHandle,
    svg_weight: librsvg::SvgHandle,
}

impl Pendulum {
    pub fn new(ctx: &render::HudContext, cfg: &Config) -> Self {
        let mut o = Self {
            cfg: (*cfg).clone(),
            est: Vec::new(),
            ys: Vec::new(),
            font: pango::FontDescription::new(),
            svg_speed: render::utils::bytes_to_svghandle(include_bytes!(
                "../../assets/icons/speed-24px.svg"
            )),
            svg_height: render::utils::bytes_to_svghandle(include_bytes!(
                "../../assets/icons/height-24px.svg"
            )),
            svg_weight: render::utils::bytes_to_svghandle(include_bytes!(
                "../../assets/icons/weight-24px.svg"
            )),
        };

        o.scale_changed(ctx);

        o
    }
}

impl render::HudRenderer for Pendulum {
    fn scale_changed(&mut self, ctx: &render::HudContext) {
        self.font.set_family("Archivo Black");
        self.font
            .set_absolute_size(ctx.sp2px(100.0) * f64::from(pango::SCALE));

        self.svg_speed
            .set_stylesheet(
                "\
            path:nth-child(2) {\
                fill: white;\
                stroke: black;\
                stroke-width: 0.5;\
            }\
        ",
            )
            .unwrap();

        self.svg_height
            .set_stylesheet(
                "\
            polygon {\
                fill: white;\
                stroke: black;\
                stroke-width: 0.5;\
            }\
        ",
            )
            .unwrap();

        self.svg_weight
            .set_stylesheet(
                "\
            path:nth-child(2) {\
                fill: white;\
                stroke: black;\
                stroke-width: 0.5;\
            }\
        ",
            )
            .unwrap();
    }

    #[allow(non_snake_case)]
    fn data_changed(&mut self, ctx: &render::HudContext) {
        let samples = unwrap_opt_or!(ctx.get_dataset(), return);
        let fns = StateFunctions::default();
        let points_fn = kalman::sigma_points::MerweScaledSigmaPoints::new(3, 0.1, 2.0, -2.0, &fns);
        let mut ukf = kalman::ukf::UKF::new(3, 6, &points_fn, &fns);

        ukf.x = ndarray::Array::from(self.cfg.initial.clone());
        ukf.P = ndarray::Array::from_diag(&ndarray::Array::from(self.cfg.initial_cov.clone()));
        ukf.R = ndarray::Array2::from_diag(&array![
            self.cfg.stdev.accel.x.powi(2),
            self.cfg.stdev.accel.y.powi(2),
            self.cfg.stdev.accel.z.powi(2),
            self.cfg.stdev.gyro.x.powi(2),
            self.cfg.stdev.gyro.y.powi(2),
            self.cfg.stdev.gyro.z.powi(2),
        ]);

        self.est.clear();
        self.ys.clear();

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
                .assign(&kalman::discretization::Q_discrete_white_noise(2, dt, 1.0e-12).unwrap());
            ukf.Q[[2, 2]] = 0.0;
            //ukf.Q[[2, 2]] = 1.0e-6;

            ukf.predict(dt);
            ukf.update(&z);

            self.est.push(ukf.x.clone());
            self.ys.push(ukf.y.clone());

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

        // stats
        println!("x = {} P = \n{}", ukf.x, ukf.P);
    }

    fn render(&self, _ctx: &render::HudContext, _cr: &cairo::Context) -> Result<(), Error> {
        Ok(())
    }

    fn orientation(
        &self,
        _ctx: &render::HudContext,
    ) -> Result<nalgebra::UnitQuaternion<f64>, Error> {
        Ok(nalgebra::UnitQuaternion::identity())
    }

    fn plot(&self, ctx: &render::HudContext, plot: &mut Plot) -> Result<(), Error> {
        let samples = ctx.get_dataset().ok_or(Error::NoDataSet)?;
        let x: Vec<f64> = samples.iter().map(|s| s.time_seconds()).collect();
        let fns = StateFunctions::default();
        let has_actual = match samples.first() {
            Some(sample) => sample.actual.is_some(),
            None => false,
        };

        plot.add_measurements(&samples, &x)?;

        let mut add_row = |rowname: &str,
                           linename: &str,
                           color: &str,
                           id: usize,
                           y: &[f64]|
         -> Result<(), Error> {
            let mut t = Plot::default_line();
            t.x(&x).y(&y).name(linename);
            t.line().color(color);

            plot.add_trace_to_rowname(&mut t, Plot::axisid_to_rowname(rowname, id))?;

            Ok(())
        };

        for i in 0..3 {
            let y: Vec<f64> = self.est.iter().map(|x| fns.hx(&x)[i]).collect();
            add_row("acc", "estimation", COLOR_E, i, &y)?;

            if has_actual {
                let y: Vec<f64> = samples
                    .iter()
                    .map(|s| fns.hx(s.actual.as_ref().unwrap())[i])
                    .collect();
                add_row("acc", "actual", COLOR_A, i, &y)?;
            }
        }

        for i in 0..3 {
            let y: Vec<f64> = self.est.iter().map(|x| fns.hx(&x)[3 + i]).collect();
            add_row("gyr", "estimation", COLOR_E, i, &y)?;

            if has_actual {
                let y: Vec<f64> = samples
                    .iter()
                    .map(|s| fns.hx(s.actual.as_ref().unwrap())[3 + i])
                    .collect();
                add_row("gyr", "actual", COLOR_A, i, &y)?;
            }
        }

        let xnames = ["p", "v", "r", "oo", "re", "rn", "ru"];
        for i in 0..self.est[0].len() {
            let rowid = plot.ensure_row(
                xnames
                    .get(i)
                    .map_or(format!("x{}", i), |s| format!("x{}-{}", i, s)),
            )?;

            let mut t = Plot::default_line();
            t.x(&x);

            let y: Vec<f64> = self.est.iter().map(|x| x[i]).collect();
            t.y(&y).name("estimation").line().color(COLOR_E);
            plot.add_trace_to_rowid(&mut t, rowid)?;

            if has_actual {
                let y: Vec<f64> = samples
                    .iter()
                    .map(|s| s.actual.as_ref().unwrap()[i])
                    .collect();
                t.y(&y).name("actual").line().color(COLOR_A);
                plot.add_trace_to_rowid(&mut t, rowid)?;
            }
        }

        plot.add_row(Some("y"))?;
        for i in 0..self.ys[0].len() {
            let y: Vec<f64> = self.ys.iter().map(|y| y[i]).collect();
            let mut t = Plot::default_line();
            t.x(&x).y(&y).name(format!("y{}", i));
            plot.add_trace(&mut t)?;
        }

        Ok(())
    }

    fn serialize_forweb(
        &self,
        _ctx: &render::HudContext,
        _cfg: &config::Config,
        _path: &std::path::Path,
    ) -> Result<(), Error> {
        Ok(())
    }
}

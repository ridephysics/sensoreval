use crate::config;
use crate::render;
use crate::render::HudRenderer;
use crate::Data;
use crate::Error;
use crate::PlotUtils;
use bincode::config::Options;
use kalman::ukf::Functions;
use ndarray::array;
use ndarray::azip;
use ndarray::s;
use sensoreval_graphics::utils::CairoEx;
use sensoreval_graphics::utils::ToUtilFont;
use sensoreval_psim::Model;
use sensoreval_psim::ToImuSample;
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

impl StateFunctions {
    fn x_normalize(mut x: ndarray::Array1<f64>) -> ndarray::Array1<f64> {
        x[0] = math::normalize_angle(x[0]);
        x[3] = math::normalize_angle(x[3]);
        x[4] = math::normalize_angle(x[4]);
        x[5] = math::normalize_angle(x[5]);
        x[6] = math::normalize_angle(x[6]);
        x
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
        let r = x[2];
        let sensor_pos = x[3];
        let rot_east = x[4];
        let rot_north = x[5];
        let rot_up = x[6];

        let mut next = array![pa, va];
        let params = sensoreval_psim::models::PendulumParams {
            radius: r,
            sensor_pos,
            motor: None,
        };
        let mut model = sensoreval_psim::models::Pendulum::new(params, dt);
        model.step(&mut next);

        array![
            math::normalize_angle(next[0]),
            next[1],
            r,
            math::normalize_angle(sensor_pos),
            math::normalize_angle(rot_east),
            math::normalize_angle(rot_north),
            math::normalize_angle(rot_up),
        ]
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
        let mut o_sum = math::SinCosSum::default();
        let mut re_sum = math::SinCosSum::default();
        let mut rn_sum = math::SinCosSum::default();
        let mut ru_sum = math::SinCosSum::default();

        azip!((sp in sigmas.genrows(), w in Wm) {
            assert!(sp[0] >= -std::f64::consts::PI && sp[0] <= std::f64::consts::PI);
            assert!(sp[3] >= -std::f64::consts::PI && sp[3] <= std::f64::consts::PI);
            assert!(sp[4] >= -std::f64::consts::PI && sp[4] <= std::f64::consts::PI);
            assert!(sp[5] >= -std::f64::consts::PI && sp[5] <= std::f64::consts::PI);
            assert!(sp[6] >= -std::f64::consts::PI && sp[6] <= std::f64::consts::PI);

            pa_sum.add(sp[0], *w);
            o_sum.add(sp[3], *w);
            re_sum.add(sp[4], *w);
            rn_sum.add(sp[5], *w);
            ru_sum.add(sp[6], *w);
        });

        ret[0] = pa_sum.avg();
        ret[3] = o_sum.avg();
        ret[4] = re_sum.avg();
        ret[5] = rn_sum.avg();
        ret[6] = ru_sum.avg();

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
        Self::x_normalize(a - b)
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
        Self::x_normalize(a + b)
    }

    fn hx<S>(&self, x: &ndarray::ArrayBase<S, ndarray::Ix1>) -> ndarray::Array1<Self::Elem>
    where
        S: ndarray::Data<Elem = Self::Elem>,
    {
        let modelstate = x.slice(ndarray::s![0..2]);
        let r = x[2];
        let sensor_pos = x[3];
        let rot = x.slice(ndarray::s![4..7]);
        let rot = rot.as_slice().unwrap();

        let params = sensoreval_psim::models::PendulumParams {
            radius: r,
            sensor_pos,
            motor: None,
        };
        let model = sensoreval_psim::models::Pendulum::new(params, 0.1);

        let mut z = ndarray::Array::zeros(6);
        model.to_accel(&modelstate, &mut z.slice_mut(ndarray::s![0..3]));
        model.to_gyro(&modelstate, &mut z.slice_mut(ndarray::s![3..6]));
        sensoreval_psim::utils::rotate_imudata(rot, &mut z.slice_mut(ndarray::s![0..3]));
        sensoreval_psim::utils::rotate_imudata(rot, &mut z.slice_mut(ndarray::s![3..6]));
        z
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
            font: pango::FontDescription::new(),
            svg_speed: sensoreval_graphics::utils::bytes_to_svghandle(
                sensoreval_graphics::ICON_SPEED,
            ),
            svg_height: sensoreval_graphics::utils::bytes_to_svghandle(
                sensoreval_graphics::ICON_HEIGHT,
            ),
            svg_weight: sensoreval_graphics::utils::bytes_to_svghandle(
                sensoreval_graphics::ICON_WEIGHT,
            ),
        };

        o.scale_changed(ctx);

        o
    }

    fn est_acceleration(x: &ndarray::Array1<f64>) -> f64 {
        x[1].powi(2) * x[2] + math::GRAVITY * (x[0] + x[3]).cos()
    }

    fn est_human_angle(x: &ndarray::Array1<f64>) -> f64 {
        x[0] + x[3]
    }

    fn est_velocity(x: &ndarray::Array1<f64>) -> f64 {
        x[1] * x[2]
    }

    fn est_altitude(x: &ndarray::Array1<f64>) -> f64 {
        x[2] - (x[0] + x[3]).cos() * x[2]
    }

    fn est(&self, actual_ts: u64, dataset: &[Data], dataid: usize) -> ndarray::Array1<f64> {
        let sample = &dataset[dataid];
        let est_sampletime = &self.est[dataid];

        let est_now = if actual_ts > sample.time {
            let fns = StateFunctions::default();
            Some(fns.fx(
                est_sampletime,
                (actual_ts - sample.time) as f64 / 1_000_000.0f64,
            ))
        } else {
            None
        };

        if let Some(est_now) = est_now {
            est_now
        } else {
            est_sampletime.clone()
        }
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
        let points_fn = kalman::sigma_points::MerweScaledSigmaPoints::new(7, 0.1, 2.0, -4.0, &fns);
        let mut ukf = kalman::ukf::UKF::new(7, 6, &points_fn, &fns);

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
                .assign(&kalman::discretization::Q_discrete_white_noise(2, dt, 0.001).unwrap());
            ukf.Q[[2, 2]] = 0.0;
            ukf.Q[[3, 3]] = 0.0;
            ukf.Q
                .slice_mut(s![4..7, 4..7])
                .assign(&kalman::discretization::Q_discrete_white_noise(3, dt, 0.001).unwrap());

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

        // stats
        println!("x = {:.8} P = \n{:.8}", ukf.x, ukf.P);
        let mut avg = ndarray::Array::zeros(ukf.x.dim());
        let mut min = ndarray::Array::from_elem(ukf.x.dim(), std::f64::MAX);
        let mut max = ndarray::Array::from_elem(ukf.x.dim(), std::f64::MIN);
        let mut max_ang = 0.0f64;
        let mut max_vel = 0.0f64;
        let mut max_acc = 0.0f64;
        for x in &self.est {
            avg += x;
            azip!((dst in &mut min, &x in x) *dst = dst.min(x));
            azip!((dst in &mut max, &x in x) *dst = dst.max(x));

            max_ang = max_ang.max(Self::est_human_angle(x).abs());
            max_vel = max_vel.max(Self::est_velocity(x).abs());
            max_acc = max_acc.max(Self::est_acceleration(x).abs());
        }
        avg /= self.est.len() as f64;

        println!("avg x: {:.8}", avg);
        println!("min x: {:.8}", min);
        println!("max x: {:.8}", max);
        println!("max h_ang: {}", max_ang);
        println!("max h_acc: {}", max_acc);
        println!("max vel: {}", max_vel);
    }

    fn render(&self, ctx: &render::HudContext, cr: &cairo::Context) -> Result<(), Error> {
        let dataid = unwrap_opt_or!(ctx.current_data_id(), return Err(Error::SampleNotFound));
        let dataset = ctx.get_dataset().unwrap();
        let est = self.est(ctx.actual_ts, dataset, dataid);

        let mut utilfont = self.font.utilfont();
        utilfont.line_width = ctx.dp2px(3.0);

        // swingboat
        let ssz = cr.surface_sz_user();
        let ppm = 30.0;
        cr.move_to(ssz.0 - ctx.dp2px(16.0 * ppm), ssz.1 - ctx.dp2px(16.5 * ppm));

        cr.save();
        cr.scale(ctx.dpi / 160.0, ctx.dpi / 160.0);
        cr.scale(ppm, ppm);
        sensoreval_graphics::pendulum_nessy::draw(cr, est[0], self.cfg.active_row);
        cr.restore();

        let dataslice = &dataset[0..dataid];
        let estslice = &self.est[0..dataid];

        let mut graph_at = sensoreval_graphics::utils::GraphAndText::new(&utilfont);
        graph_at.graph.width = ctx.dp2px(200.0);
        graph_at.graph.height = ctx.dp2px(100.0);
        graph_at.graph.dt = 10_000_000;
        graph_at.graph.line_width = ctx.dp2px(6.0);
        graph_at.graph.border_width = ctx.dp2px(3.0);
        graph_at.graph_x = ctx.dp2px(500.0);

        // acceleration
        cr.move_to(ctx.dp2px(10.0), ctx.dp2px(10.0));
        graph_at.icon = Some(&self.svg_weight);
        graph_at.graph.maxval = 3.0;
        graph_at.graph.redval = 5.0;
        graph_at.unit = "g";
        graph_at.precision = 1;
        graph_at.draw(
            cr,
            &mut dataslice.iter().rev().map(|data| data.time),
            &mut estslice
                .iter()
                .rev()
                .map(|data| Self::est_acceleration(&data) / math::GRAVITY),
        );

        // velocity
        cr.move_to(ctx.dp2px(10.0), ctx.dp2px(20.0) + graph_at.graph.height);
        graph_at.icon = Some(&self.svg_speed);
        graph_at.graph.maxval = 50.0;
        graph_at.graph.redval = 100.0;
        graph_at.unit = "km/h";
        graph_at.precision = 0;
        graph_at.draw(
            cr,
            &mut dataslice.iter().rev().map(|data| data.time),
            &mut estslice
                .iter()
                .rev()
                .map(|data| Self::est_velocity(&data).abs() * 3.6),
        );

        // altitude
        cr.move_to(
            ctx.dp2px(10.0),
            ctx.dp2px(40.0) + graph_at.graph.height * 2.0,
        );
        graph_at.icon = Some(&self.svg_height);
        graph_at.graph.maxval = 15.0;
        graph_at.graph.redval = 70.0;
        graph_at.unit = "m";
        graph_at.precision = 1;
        graph_at.draw(
            cr,
            &mut dataslice.iter().rev().map(|data| data.time),
            &mut estslice.iter().rev().map(|data| Self::est_altitude(&data)),
        );

        Ok(())
    }

    fn orientation(
        &self,
        ctx: &render::HudContext,
    ) -> Result<nalgebra::UnitQuaternion<f64>, Error> {
        let dataid = unwrap_opt_or!(ctx.current_data_id(), return Err(Error::SampleNotFound));
        let dataset = ctx.get_dataset().unwrap();
        let est = self.est(ctx.actual_ts, dataset, dataid);

        let axis = nalgebra::Unit::new_normalize(nalgebra::Vector3::new(1.0, 0.0, 0.0));
        Ok(nalgebra::UnitQuaternion::from_axis_angle(
            &axis,
            Self::est_human_angle(&est),
        ))
    }

    fn plot(
        &self,
        ctx: &render::HudContext,
        plot: &mut sensoreval_utils::Plot,
    ) -> Result<(), Error> {
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
            let mut t = sensoreval_utils::Plot::default_line();
            t.x(&x).y(&y).name(linename);
            t.line().color(color);

            plot.add_trace_to_rowname(
                &mut t,
                sensoreval_utils::Plot::axisid_to_rowname(rowname, id),
            )?;

            Ok(())
        };

        for i in 0..3 {
            let y: Vec<f64> = self.est.iter().map(|x| fns.hx(&x)[i]).collect();
            add_row("acc", "estimation", sensoreval_utils::COLOR_E, i, &y)?;

            if has_actual {
                let y: Vec<f64> = samples
                    .iter()
                    .map(|s| fns.hx(s.actual.as_ref().unwrap())[i])
                    .collect();
                add_row("acc", "actual", sensoreval_utils::COLOR_A, i, &y)?;
            }
        }

        for i in 0..3 {
            let y: Vec<f64> = self.est.iter().map(|x| fns.hx(&x)[3 + i]).collect();
            add_row("gyr", "estimation", sensoreval_utils::COLOR_E, i, &y)?;

            if has_actual {
                let y: Vec<f64> = samples
                    .iter()
                    .map(|s| fns.hx(s.actual.as_ref().unwrap())[3 + i])
                    .collect();
                add_row("gyr", "actual", sensoreval_utils::COLOR_A, i, &y)?;
            }
        }

        let xnames = ["p", "v", "r", "oo", "re", "rn", "ru"];
        for i in 0..self.est[0].len() {
            let rowid = plot.ensure_row(
                xnames
                    .get(i)
                    .map_or(format!("x{}", i), |s| format!("x{}-{}", i, s)),
            )?;

            let mut t = sensoreval_utils::Plot::default_line();
            t.x(&x);

            let y: Vec<f64> = self.est.iter().map(|x| x[i]).collect();
            t.y(&y)
                .name("estimation")
                .line()
                .color(sensoreval_utils::COLOR_E);
            plot.add_trace_to_rowid(&mut t, rowid)?;

            if has_actual {
                let y: Vec<f64> = samples
                    .iter()
                    .map(|s| s.actual.as_ref().unwrap()[i])
                    .collect();
                t.y(&y)
                    .name("actual")
                    .line()
                    .color(sensoreval_utils::COLOR_A);
                plot.add_trace_to_rowid(&mut t, rowid)?;
            }
        }

        Ok(())
    }

    fn serialize_forweb(
        &self,
        ctx: &render::HudContext,
        cfg: &config::Config,
        path: &std::path::Path,
    ) -> Result<(), Error> {
        const TIMESTEP: u64 = 15000;
        let dataset = ctx.get_dataset().unwrap();
        let out_est = path.join("est.bin");
        let mut file = std::fs::File::create(&out_est)?;

        bincode::options()
            .with_big_endian()
            .serialize_into(&mut file, &(self.cfg.initial[2] as f32))?;

        bincode::options()
            .with_big_endian()
            .serialize_into(&mut file, &(self.cfg.initial[3] as f32))?;

        bincode::options()
            .with_big_endian()
            .serialize_into(&mut file, &TIMESTEP)?;

        let mut us = cfg.video.startoff * 1000;
        while let Some(dataid) = crate::id_for_time(&dataset, 0, us) {
            let est = self.est(us, &dataset, dataid);
            bincode::options().with_big_endian().serialize_into(
                &mut file,
                &[half::f16::from_f64(est[0]), half::f16::from_f64(est[1])],
            )?;

            us += TIMESTEP;
        }

        Ok(())
    }
}

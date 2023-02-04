use crate::config;
use crate::render;
use crate::render::HudRenderer;
use crate::Data;
use crate::Error;
use crate::PlotUtils;
use bincode::config::Options;
use kalman::ukf::Fx;
use kalman::ukf::Hx;
use kalman::Filter;
use kalman::Normalize;
use kalman::SetDt;
use ndarray::array;
use ndarray::azip;
use ndarray::s;
use sensoreval_graphics::utils::CairoEx;
use sensoreval_graphics::utils::ToUtilFont;
use sensoreval_psim::models::pendulum::State as PendulumState;
use sensoreval_psim::models::pendulum::StateArgs as PendulumStateArgs;
use sensoreval_psim::Model;
use sensoreval_psim::ToImuSample;
use sensoreval_utils::macros::*;
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

#[derive(Default)]
struct XFunctions;

#[derive(State, KalmanMath, UKFMath)]
#[state(fnstruct = "XFunctions")]
enum X {
    #[state(angle)]
    Theta,
    ThetaD,
    Radius,
    #[state(angle)]
    SensorPos,
    #[state(angle)]
    RotE,
    #[state(angle)]
    RotN,
    #[state(angle)]
    RotU,
}

#[derive(Default)]
struct ZFunctions;

#[derive(State, KalmanMath, UKFMath)]
#[state(fnstruct = "ZFunctions")]
#[allow(dead_code)]
enum Z {
    AccelE,
    AccelN,
    AccelU,
    GyroE,
    GyroN,
    GyroU,
}

struct FxArgs {
    dt: f64,
}

impl FxArgs {
    pub fn new(dt: f64) -> Self {
        Self { dt }
    }
}

impl kalman::SetDt<f64> for FxArgs {
    fn set_dt(&mut self, dt: &f64) {
        self.dt = *dt;
    }
}

impl kalman::ukf::ApplyDt<f64> for FxArgs {
    #[allow(non_snake_case)]
    fn apply_dt(&self, Q: &mut ndarray::Array2<f64>) {
        Q.slice_mut(s![0..2, 0..2])
            .assign(&kalman::discretization::Q_discrete_white_noise(2, self.dt, 0.001).unwrap());
        Q[[2, 2]] = 0.0;
        Q[[3, 3]] = 0.0;
        Q.slice_mut(s![4..7, 4..7])
            .assign(&kalman::discretization::Q_discrete_white_noise(3, self.dt, 0.001).unwrap());
    }
}

impl kalman::ukf::Fx<FxArgs> for XFunctions {
    type Elem = f64;

    fn fx<S>(
        &self,
        x: &ndarray::ArrayBase<S, ndarray::Ix1>,
        args: &FxArgs,
    ) -> ndarray::Array1<Self::Elem>
    where
        S: ndarray::Data<Elem = Self::Elem>,
    {
        let params = sensoreval_psim::models::PendulumParams {
            radius: x[X::Radius],
            sensor_pos: x[X::SensorPos],
            motor: None,
        };
        let mut model = sensoreval_psim::models::Pendulum::new(params, args.dt);

        let mut psim_state = ndarray::Array1::from(PendulumStateArgs {
            theta: x[X::Theta],
            theta_d: x[X::ThetaD],
        });
        model.step(&mut psim_state);

        self.normalize(ndarray::Array1::from(XArgs {
            theta: psim_state[PendulumState::Theta],
            theta_d: psim_state[PendulumState::ThetaD],
            radius: x[X::Radius],
            sensor_pos: x[X::SensorPos],
            rot_e: x[X::RotE],
            rot_n: x[X::RotN],
            rot_u: x[X::RotU],
        }))
    }
}

impl kalman::ukf::Hx for XFunctions {
    type Elem = f64;

    fn hx<S>(&self, x: &ndarray::ArrayBase<S, ndarray::Ix1>) -> ndarray::Array1<Self::Elem>
    where
        S: ndarray::Data<Elem = Self::Elem>,
    {
        let rot = x.slice(ndarray::s![4..7]);
        let rot = rot.as_slice().unwrap();

        let params = sensoreval_psim::models::PendulumParams {
            radius: x[X::Radius],
            sensor_pos: x[X::SensorPos],
            motor: None,
        };
        let model = sensoreval_psim::models::Pendulum::new(params, 0.1);

        let psim_state = ndarray::Array1::from(PendulumStateArgs {
            theta: x[X::Theta],
            theta_d: x[X::ThetaD],
        });

        let mut accel = ndarray::Array::zeros(3);
        model.to_accel(&psim_state, &mut accel);
        sensoreval_psim::utils::rotate_imudata(rot, &mut accel);

        let mut gyro = ndarray::Array::zeros(3);
        model.to_gyro(&psim_state, &mut gyro);
        sensoreval_psim::utils::rotate_imudata(rot, &mut gyro);

        ndarray::Array1::from(ZArgs {
            accel_e: accel[0],
            accel_n: accel[1],
            accel_u: accel[2],
            gyro_e: gyro[0],
            gyro_n: gyro[1],
            gyro_u: gyro[2],
        })
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
        x[X::ThetaD].powi(2) * x[X::Radius] + math::GRAVITY * (x[X::Theta] + x[X::SensorPos]).cos()
    }

    fn est_human_angle(x: &ndarray::Array1<f64>) -> f64 {
        x[X::Theta] + x[X::SensorPos]
    }

    fn est_velocity(x: &ndarray::Array1<f64>) -> f64 {
        x[X::ThetaD] * x[X::Radius]
    }

    fn est_altitude(x: &ndarray::Array1<f64>) -> f64 {
        x[X::Radius] - (x[X::Theta] + x[X::SensorPos]).cos() * x[X::Radius]
    }

    fn est(&self, actual_ts: u64, dataset: &[Data], dataid: usize) -> ndarray::Array1<f64> {
        let sample = &dataset[dataid];
        let est_sampletime = &self.est[dataid];

        let est_now = if actual_ts > sample.time {
            let dt = (actual_ts - sample.time) as f64 / 1_000_000.0f64;
            let fns = XFunctions::default();
            let fxargs = FxArgs::new(dt);
            Some(fns.fx(est_sampletime, &fxargs))
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
        let points_fn = kalman::sigma_points::MerweScaledSigmaPoints::new(
            7,
            0.1,
            2.0,
            -4.0,
            XFunctions::default(),
        );
        let mut ukf = kalman::ukf::Ukf::new(
            7,
            6,
            &points_fn,
            XFunctions::default(),
            FxArgs::new(0.1),
            ZFunctions::default(),
        );

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
            let z = ndarray::Array1::from(ZArgs {
                accel_e: sample.accel[0],
                accel_n: sample.accel[1],
                accel_u: sample.accel[2],
                gyro_e: sample.gyro[0],
                gyro_n: sample.gyro[1],
                gyro_u: sample.gyro[2],
            });
            let dt = (sample.time - t_prev) as f64 / 1_000_000.0f64;

            ukf.set_dt(&dt);
            ukf.predict().unwrap();
            ukf.update(&z).unwrap();

            self.est.push(ukf.x.clone());

            if self.cfg.enable_rts_smoother {
                Ps.push(ukf.P.clone());
                Qs.push(ukf.Q.clone());
                dts.push(dt);
            }

            t_prev = sample.time;
        }

        if self.cfg.enable_rts_smoother {
            let (xss, _) = ukf.rts_smoother(&self.est, &Ps, Some(&Qs), &dts).unwrap();
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

        println!("avg x: {avg:.8}");
        println!("min x: {min:.8}");
        println!("max x: {max:.8}");
        println!("max h_ang: {max_ang}");
        println!("max h_acc: {max_acc}");
        println!("max vel: {max_vel}");
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

        cr.save().unwrap();
        cr.scale(ctx.dpi / 160.0, ctx.dpi / 160.0);
        cr.scale(ppm, ppm);
        sensoreval_graphics::pendulum_nessy::draw(cr, est[0], self.cfg.active_row);
        cr.restore().unwrap();

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
                .map(|data| Self::est_acceleration(data) / math::GRAVITY),
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
                .map(|data| Self::est_velocity(data).abs() * 3.6),
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
            &mut estslice.iter().rev().map(Self::est_altitude),
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
        let fns = XFunctions::default();
        let has_actual = match samples.first() {
            Some(sample) => sample.actual.is_some(),
            None => false,
        };

        plot.add_measurements(samples, &x)?;

        let mut add_row = |rowname: &str,
                           linename: &str,
                           color: &str,
                           id: usize,
                           y: &[f64]|
         -> Result<(), Error> {
            let mut t = sensoreval_utils::Plot::default_line();
            t.x(&x).y(y).name(linename);
            t.line().color(color);

            plot.add_trace_to_rowname(
                &mut t,
                sensoreval_utils::Plot::axisid_to_rowname(rowname, id),
            )?;

            Ok(())
        };

        for i in 0..3 {
            let y: Vec<f64> = self.est.iter().map(|x| fns.hx(x)[i]).collect();
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
            let y: Vec<f64> = self.est.iter().map(|x| fns.hx(x)[3 + i]).collect();
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
                    .map_or(format!("x{i}"), |s| format!("x{i}-{s}")),
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
        while let Some(dataid) = crate::id_for_time(dataset, 0, us) {
            let est = self.est(us, dataset, dataid);
            bincode::options().with_big_endian().serialize_into(
                &mut file,
                &[
                    half::f16::from_f64(est[X::Theta]),
                    half::f16::from_f64(est[X::ThetaD]),
                ],
            )?;

            us += TIMESTEP;
        }

        Ok(())
    }
}

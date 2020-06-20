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
        let orientation_offset = x[3];
        let rot_east = x[4];
        let rot_north = x[5];
        let rot_up = x[6];

        let mut teo =
            eom::explicit::RK4::new(crate::simulator::pendulum::EomFns::from_radius(r), dt);
        let mut ic = array![pa, va];
        let next = teo.iterate(&mut ic);

        array![
            math::normalize_angle(next[0]),
            next[1],
            r,
            math::normalize_angle(orientation_offset),
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
        let pa = x[0];
        let va = x[1];
        let r = x[2];
        let orientation_offset = x[3];
        let rot_east = x[4];
        let rot_north = x[5];
        let rot_up = x[6];
        let ac = va.powi(2) * r;

        let accel = nalgebra::Vector3::new(
            0.0,
            0.0,
            ac + math::GRAVITY * (pa + orientation_offset).cos(),
        );
        let gyro = nalgebra::Vector3::new(va, 0.0, 0.0);

        let axis_east = nalgebra::Unit::new_normalize(nalgebra::Vector3::new(1.0, 0.0, 0.0));
        let q = nalgebra::UnitQuaternion::from_axis_angle(&axis_east, rot_east);
        let accel = q * accel;
        let gyro = q * gyro;

        let axis_north = nalgebra::Unit::new_normalize(nalgebra::Vector3::new(0.0, 1.0, 0.0));
        let q = nalgebra::UnitQuaternion::from_axis_angle(&axis_north, rot_north);
        let accel = q * accel;
        let gyro = q * gyro;

        let axis_up = nalgebra::Unit::new_normalize(nalgebra::Vector3::new(0.0, 0.0, 1.0));
        let q = nalgebra::UnitQuaternion::from_axis_angle(&axis_up, rot_up);
        let accel = q * accel;
        let gyro = q * gyro;

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

    /// draws to pointy end of a boat
    fn swingboat_head_line(
        cr: &cairo::Context,
        radius0: f64,
        angle0: f64,
        radius1: f64,
        angle1: f64,
    ) {
        let thickness = (radius0 - radius1).abs();
        let (angle_inner, angle_outer) = if radius1 <= radius0 {
            (angle0, angle1)
        } else {
            (angle1, angle0)
        };
        let mirror = if angle_outer <= angle_inner {
            1.0
        } else {
            -1.0
        };
        let (x0, y0) = render::utils::circle_coords(radius0, angle0);
        let (x1, y1) = render::utils::circle_coords(radius1, angle1);

        // tip
        let (tip_x, tip_y) = render::utils::circle_coords(
            thickness,
            angle_outer + (std::f64::consts::FRAC_PI_2 * mirror),
        );

        // bottom
        let (bottom_x, bottom_y) = render::utils::circle_coords(
            thickness / 2.0,
            angle_inner - (std::f64::consts::FRAC_PI_2 * mirror),
        );

        if radius1 <= radius0 {
            cr.curve_to(x0 + bottom_x, y0 + bottom_y, x1 + tip_x, y1 + tip_y, x1, y1);
        } else {
            cr.curve_to(x0 + tip_x, y0 + tip_y, x1 + bottom_x, y1 + bottom_y, x1, y1);
        }
    }

    fn draw_swingboat(
        &self,
        ctx: &render::HudContext,
        cr: &cairo::Context,
        ppm: f64,
        gondola_rotation: f64,
    ) {
        let frame_radius: f64 = 16.0;
        let frame_angle: f64 = (68.0f64).to_radians();
        let frame_thickness: f64 = 0.5;
        let frame_color: u32 = 0xffff_ffe5;
        let frame_top_color: u32 = 0x0000_00e5;
        let gondola_radius: f64 = 15.0;
        let gondola_angle: f64 = (30.0f64).to_radians();
        let gondola_frame_thickness: f64 = 0.4;
        let gondola_thickness: f64 = 1.8;
        let gondola_color: u32 = 0xff8f_00e5;
        let gondola_num_sections: usize = 5;
        let active_row_color: u32 = 0x64dd_17e5;
        let section_divider_color: u32 = 0x0000_00e5;
        let section_divider_width: f64 = 0.2;
        let section_dark_color: u32 = 0x0000_0033;
        let border_width: f64 = 0.1;
        let border_color: u32 = 0x0000_00ff;

        let (cx, cy) = cr.get_current_point();

        cr.save();
        cr.translate(cx, cy);
        cr.scale(ctx.dpi / 160.0, ctx.dpi / 160.0);
        cr.scale(ppm, ppm);

        cr.save();
        cr.rotate(gondola_rotation);

        // gondola-frame
        render::utils::set_source_rgba_u32(cr, frame_color);
        cr.set_line_width(gondola_frame_thickness);
        cr.move_to(0., 0.);
        // we use twice the radius because we want to cut the line horizontally
        // which is done by clipping it
        render::utils::stroke_arc_sides(
            cr,
            gondola_radius,
            std::f64::consts::PI / 2.0,
            gondola_angle / 2.0
                + math::tri_solve_sas(
                    frame_thickness / 2.0,
                    gondola_radius,
                    std::f64::consts::FRAC_PI_2,
                )
                .0,
            border_width,
            border_color,
        );

        // gondola
        {
            // the boat itself
            cr.save();

            let angle_middle = std::f64::consts::FRAC_PI_2;
            let angle_left = angle_middle + gondola_angle / 2.0;
            let angle_right = angle_middle - gondola_angle / 2.0;
            let section_width_ang = gondola_angle / gondola_num_sections as f64;
            let angle_head_right = angle_right - section_width_ang;
            let angle_head_left = angle_left + section_width_ang;
            let radius_ship_outer = gondola_radius + gondola_thickness / 2.0;
            let radius_ship_inner = gondola_radius - gondola_thickness / 2.0;
            let gondola_line_width = gondola_thickness / 6.0;

            cr.set_operator(cairo::Operator::Source);
            cr.set_line_join(cairo::LineJoin::Round);

            let (x0, y0) = render::utils::circle_coords(radius_ship_outer, angle_right);
            cr.move_to(x0, y0);

            Self::swingboat_head_line(
                cr,
                radius_ship_outer,
                angle_right,
                radius_ship_inner,
                angle_head_right,
            );
            cr.arc(
                0.0,
                0.0,
                radius_ship_inner,
                angle_head_right,
                angle_head_left,
            );
            Self::swingboat_head_line(
                cr,
                radius_ship_inner,
                angle_head_left,
                radius_ship_outer,
                angle_left,
            );
            cr.arc_negative(0.0, 0.0, radius_ship_outer, angle_left, angle_right);
            cr.close_path();

            cr.set_line_width(gondola_line_width + border_width);
            render::utils::set_source_rgba_u32(cr, border_color);
            cr.stroke_preserve();

            render::utils::set_source_rgba_u32(cr, gondola_color);
            cr.set_line_width(gondola_line_width);
            cr.stroke_preserve();

            cr.fill();
            cr.restore();

            // sections
            cr.save();
            render::utils::set_source_rgba_u32(cr, section_dark_color);
            cr.set_line_width(gondola_thickness + gondola_line_width);

            // left
            cr.arc(
                0.0,
                0.0,
                gondola_radius,
                angle_left - section_width_ang,
                angle_left,
            );
            cr.stroke();

            // middle
            cr.arc(
                0.0,
                0.0,
                gondola_radius,
                angle_middle - section_width_ang / 2.0,
                angle_middle + section_width_ang / 2.0,
            );
            cr.stroke();

            // right
            cr.arc(
                0.0,
                0.0,
                gondola_radius,
                angle_right,
                angle_right + section_width_ang,
            );
            cr.stroke();

            // active row
            let active_row_left = angle_left - self.cfg.active_row as f64 * section_width_ang / 2.0;
            cr.set_operator(cairo::Operator::Source);
            render::utils::set_source_rgba_u32(cr, active_row_color);
            cr.arc_negative(
                0.0,
                0.0,
                gondola_radius,
                active_row_left,
                active_row_left - section_width_ang / 2.0,
            );
            cr.stroke();

            cr.restore();

            // section dividers
            let radius_divider_inner = radius_ship_inner - gondola_line_width / 2.0;
            let radius_divider_outer = radius_ship_outer + gondola_line_width / 2.0;

            cr.save();
            cr.set_line_width(section_divider_width);

            for i in 0..gondola_num_sections {
                let angle = angle_left - section_width_ang / 2.0 - i as f64 * section_width_ang;

                render::utils::move_to_circle(cr, radius_divider_inner, angle);
                render::utils::line_to_circle(cr, radius_divider_outer, angle);

                render::utils::set_source_rgba_u32(cr, section_divider_color);
                cr.set_operator(cairo::Operator::Source);
                cr.stroke();
            }

            cr.restore();
        }

        cr.restore();

        // frame
        cr.save();
        render::utils::clip_bottom(cr, frame_radius);
        render::utils::set_source_rgba_u32(cr, frame_color);
        cr.set_line_width(frame_thickness);
        cr.move_to(0., 0.);
        render::utils::stroke_arc_sides(
            cr,
            frame_radius * 2.0,
            std::f64::consts::PI / 2.0,
            frame_angle / 2.0,
            border_width,
            border_color,
        );
        cr.restore();

        // top
        cr.set_operator(cairo::Operator::Source);
        render::utils::set_source_rgba_u32(cr, frame_color);
        cr.set_line_width(0.2);
        cr.arc(0., 0., 1.0, 0., 2.0 * std::f64::consts::PI);
        cr.fill_preserve();
        render::utils::set_source_rgba_u32(cr, frame_top_color);
        cr.stroke();

        cr.restore();
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

        let mut utilfont = render::utils::Font::new(&self.font);
        utilfont.line_width = ctx.dp2px(3.0);

        // swingboat
        let ssz = render::utils::surface_sz_user(cr);
        let ppm = 30.0;
        cr.move_to(ssz.0 - ctx.dp2px(16.0 * ppm), ssz.1 - ctx.dp2px(16.5 * ppm));
        self.draw_swingboat(ctx, cr, ppm, est[0]);

        let dataslice = &dataset[0..dataid];
        let estslice = &self.est[0..dataid];

        let mut graph_at = render::utils::GraphAndText::new(&utilfont);
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

        bincode::config()
            .big_endian()
            .serialize_into(&mut file, &(self.cfg.initial[2] as f32))?;

        bincode::config()
            .big_endian()
            .serialize_into(&mut file, &(self.cfg.initial[3] as f32))?;

        bincode::config()
            .big_endian()
            .serialize_into(&mut file, &TIMESTEP)?;

        let mut us = cfg.video.startoff * 1000;
        while let Some(dataid) = id_for_time(&dataset, 0, us) {
            let est = self.est(us, &dataset, dataid);
            bincode::config().big_endian().serialize_into(
                &mut file,
                &[half::f16::from_f64(est[0]), half::f16::from_f64(est[1])],
            )?;

            us += TIMESTEP;
        }

        Ok(())
    }
}

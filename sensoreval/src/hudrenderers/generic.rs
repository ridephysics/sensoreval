use crate::render::HudRenderer;
use crate::*;
use ndarray_linalg::norm::Norm;

pub(crate) struct Generic {
    font: pango::FontDescription,
}

impl Generic {
    pub fn new(ctx: &render::HudContext) -> Self {
        let mut o = Self {
            font: pango::FontDescription::new(),
        };

        o.scale_changed(ctx);

        o
    }
}

impl render::HudRenderer for Generic {
    fn scale_changed(&mut self, ctx: &render::HudContext) {
        self.font.set_family("Archivo Black");
        self.font
            .set_absolute_size(ctx.sp2px(100.0) * f64::from(pango::SCALE));
    }

    fn data_changed(&mut self, _ctx: &render::HudContext) {}

    fn render(&self, ctx: &render::HudContext, cr: &cairo::Context) -> Result<(), Error> {
        let dataid = unwrap_opt_or!(ctx.current_data_id(), return Err(Error::SampleNotFound));
        let dataset = ctx.get_dataset().unwrap();

        let mut utilfont = sensoreval_graphics::utils::Font::new(&self.font);
        utilfont.line_width = ctx.dp2px(3.0);

        let dataslice = &dataset[0..dataid];

        let mut graph_at = sensoreval_graphics::utils::GraphAndText::new(&utilfont);
        graph_at.graph.width = ctx.dp2px(200.0);
        graph_at.graph.height = ctx.dp2px(100.0);
        graph_at.graph.dt = 10_000_000;
        graph_at.graph.line_width = ctx.dp2px(6.0);
        graph_at.graph.border_width = ctx.dp2px(3.0);
        graph_at.graph_x = ctx.dp2px(500.0);

        // acceleration
        cr.move_to(ctx.dp2px(10.0), ctx.dp2px(10.0));
        graph_at.icon = None;
        graph_at.graph.maxval = 3.0;
        graph_at.graph.redval = 5.0;
        graph_at.unit = "g";
        graph_at.precision = 1;
        graph_at.draw(
            cr,
            &mut dataslice.iter().rev().map(|data| data.time),
            &mut dataslice
                .iter()
                .rev()
                .map(|data| data.accel.norm_l2() / math::GRAVITY),
        );

        // gyroscope
        cr.move_to(ctx.dp2px(10.0), ctx.dp2px(20.0) + graph_at.graph.height);
        graph_at.icon = None;
        graph_at.graph.maxval = 50.0;
        graph_at.graph.redval = 100.0;
        graph_at.unit = "rad/s";
        graph_at.precision = 0;
        graph_at.draw(
            cr,
            &mut dataslice.iter().rev().map(|data| data.time),
            &mut dataslice.iter().rev().map(|data| data.gyro.norm_l2()),
        );

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

        plot.add_measurements(&samples, &x)?;

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

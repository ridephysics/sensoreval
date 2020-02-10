pub mod utils;

use crate::*;

enum DataSrc {
    None,
    Data(Data),
    Array { id: usize },
}

pub struct HudContext<'b> {
    pub dataset: Option<&'b Vec<Data>>,
    src: DataSrc,
    pub dpi: f64,
    pub spi: f64,
}

impl<'b> HudContext<'b> {
    pub fn current_data(&self) -> Option<&Data> {
        match &self.src {
            DataSrc::None => None,
            DataSrc::Data(data) => Some(&data),
            DataSrc::Array { id } => match self.dataset {
                None => None,
                Some(arr) => Some(&arr[*id]),
            },
        }
    }

    pub fn current_data_id(&self) -> Option<usize> {
        match &self.src {
            DataSrc::None => None,
            DataSrc::Data(_) => None,
            DataSrc::Array { id } => match self.dataset {
                None => None,
                Some(_) => Some(*id),
            },
        }
    }

    #[inline]
    pub fn dp2px(&self, dp: f64) -> f64 {
        dp * (self.dpi / 160.0)
    }

    #[inline]
    pub fn sp2px(&self, sp: f64) -> f64 {
        sp * (self.spi / 160.0)
    }
}

pub struct Context<'a, 'b> {
    cfg: &'a config::Config,
    hudrenderer: Option<Box<dyn HudRenderer>>,
    hudctx: HudContext<'b>,
}

pub trait HudRenderer {
    fn data_changed(&mut self, ctx: &render::HudContext);
    fn render(&self, ctx: &render::HudContext, cr: &cairo::Context) -> Result<(), Error>;
    fn plot(&self, ctx: &render::HudContext) -> Result<(), Error>;
}

fn renderer_from_ctx(ctx: &Context) -> Option<Box<dyn HudRenderer>> {
    let renderer = match &ctx.cfg.hud.renderer {
        config::HudRenderer::Pendulum(cfg) => {
            hudrenderers::pendulum::Pendulum::new(&ctx.hudctx, cfg)
        }
        _ => return None,
    };

    Some(Box::new(renderer))
}

impl<'a, 'b> Context<'a, 'b> {
    pub fn new(cfg: &'a config::Config, dataset: Option<&'b Vec<Data>>) -> Self {
        let mut ctx = Self {
            cfg,
            hudrenderer: None,
            hudctx: HudContext {
                dataset,
                dpi: 141.21,
                spi: 141.21,
                src: DataSrc::None,
            },
        };

        ctx.hudrenderer = renderer_from_ctx(&ctx);

        ctx
    }

    pub fn set_ts(&mut self, us: u64) -> Result<(), Error> {
        if self.hudctx.dataset.is_none() {
            return Err(Error::NoDataSet);
        }

        match id_for_time(self.hudctx.dataset.unwrap(), 0, us) {
            Some(id) => {
                self.hudctx.src = DataSrc::Array { id };
                Ok(())
            }
            None => Err(Error::SampleNotFound),
        }
    }

    pub fn set_data(&mut self, data: Data) {
        self.hudctx.src = DataSrc::Data(data);

        if let Some(renderer) = &mut self.hudrenderer {
            renderer.data_changed(&self.hudctx);
        }
    }

    pub fn current_data(&self) -> Option<&Data> {
        self.hudctx.current_data()
    }

    pub fn current_data_id(&self) -> Option<usize> {
        self.hudctx.current_data_id()
    }

    pub fn render(&self, cr: &cairo::Context) -> Result<(), Error> {
        // clear
        cr.save();
        cr.set_source_rgba(0., 0., 0., 0.);
        cr.set_operator(cairo::Operator::Source);
        cr.paint();
        cr.restore();

        if let Some(renderer) = &self.hudrenderer {
            cr.save();
            let hudret = renderer.render(&self.hudctx, cr);
            cr.restore();
            hudret?;
        }

        Ok(())
    }

    pub fn plot(&self) -> Result<(), Error> {
        if let Some(renderer) = &self.hudrenderer {
            renderer.plot(&self.hudctx)
        } else {
            Err(Error::NoHudRenderer)
        }
    }

    #[inline]
    pub fn dp2px(&self, dp: f64) -> f64 {
        self.hudctx.dp2px(dp)
    }

    #[inline]
    pub fn sp2px(&self, sp: f64) -> f64 {
        self.hudctx.sp2px(sp)
    }
}

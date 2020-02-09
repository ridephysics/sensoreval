pub mod utils;

use crate::*;

enum DataSrc {
    None,
    Data(Data),
    Array { id: usize },
}

pub struct Context<'a, 'b> {
    pub cfg: &'a config::Config,
    pub dataset: Option<&'b Vec<Data>>,
    pub dpi: f64,
    pub spi: f64,
    src: DataSrc,
    hudhandler: Option<Box<dyn HudRenderer>>,
}

pub trait HudRenderer {
    fn render(&self, ctx: &render::Context, cr: &cairo::Context) -> Result<(), Error>;
}

fn handler_from_ctx(ctx: &Context) -> Option<Box<dyn HudRenderer>> {
    let handler = match &ctx.cfg.hud.renderer {
        config::HudRenderer::Pendulum(cfg) => hudrenderers::pendulum::Pendulum::new(ctx, cfg),
        _ => return None,
    };

    Some(Box::new(handler))
}

impl<'a, 'b> Context<'a, 'b> {
    pub fn new(cfg: &'a config::Config, dataset: Option<&'b Vec<Data>>) -> Self {
        let mut ctx = Self {
            cfg,
            dataset,
            dpi: 141.21,
            spi: 141.21,
            src: DataSrc::None,
            hudhandler: None,
        };

        ctx.hudhandler = handler_from_ctx(&ctx);

        ctx
    }

    pub fn set_ts(&mut self, us: u64) -> Result<(), Error> {
        if self.dataset.is_none() {
            return Err(Error::NoDataSet);
        }

        match id_for_time(self.dataset.unwrap(), 0, us) {
            Some(id) => {
                self.src = DataSrc::Array { id };
                Ok(())
            }
            None => Err(Error::SampleNotFound),
        }
    }

    pub fn set_data(&mut self, data: Data) {
        self.src = DataSrc::Data(data);
    }

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

    pub fn render(&self, cr: &cairo::Context) -> Result<(), Error> {
        // clear
        cr.save();
        cr.set_source_rgba(0., 0., 0., 0.);
        cr.set_operator(cairo::Operator::Source);
        cr.paint();
        cr.restore();

        if let Some(handler) = &self.hudhandler {
            cr.save();
            let hudret = handler.render(self, cr);
            cr.restore();
            hudret?;
        }

        Ok(())
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

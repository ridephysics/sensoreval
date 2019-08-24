use crate::error::*;

enum DataSrc {
    None,
    Data(crate::data::Data),
    Array { us: u64, id: usize },
}

pub struct Context<'a, 'b> {
    pub cfg: &'a crate::config::Config,
    pub dataarr: Option<&'b Vec<crate::data::Data>>,
    pub dpi: f64,
    pub spi: f64,
    src: DataSrc,
    hudhandler: Option<Box<dyn HudHandler>>,
}

pub trait HudHandler {
    fn render(&self, ctx: &crate::render::Context, cr: &cairo::Context) -> Result<(), Error>;
}

fn handler_from_ctx(ctx: &Context) -> Option<Box<dyn HudHandler>> {
    let handler = match ctx.cfg.hud.mode {
        crate::config::HudMode::SwingBoat => crate::hudhandlers::swingboat::SwingBoat::new(ctx),
        _ => return None,
    };

    return Some(Box::new(handler));
}

impl<'a, 'b> Context<'a, 'b> {
    pub fn new(
        cfg: &'a crate::config::Config,
        dataarr: Option<&'b Vec<crate::data::Data>>,
    ) -> Self {
        let mut ctx = Self {
            cfg: cfg,
            dataarr: dataarr,
            dpi: 141.21,
            spi: 141.21,
            src: DataSrc::None,
            hudhandler: None,
        };

        ctx.hudhandler = handler_from_ctx(&ctx);

        return ctx;
    }

    pub fn set_ts(&mut self, us: u64) -> Result<(), Error> {
        if self.dataarr.is_none() {
            return Err(Error::from(ErrorRepr::NoDataArr));
        }

        match crate::data::id_for_time(self.dataarr.unwrap(), 0, us) {
            Some(id) => {
                self.src = DataSrc::Array { us: us, id: id };
                return Ok(());
            }
            None => {
                return Err(Error::from(ErrorRepr::SampleNotFound));
            }
        }
    }

    pub fn set_data(&mut self, data: crate::data::Data) {
        self.src = DataSrc::Data(data);
    }

    pub fn current_data(&self) -> Option<&crate::data::Data> {
        match &self.src {
            DataSrc::None => None,
            DataSrc::Data(data) => Some(&data),
            DataSrc::Array { id, us: _ } => match self.dataarr {
                None => None,
                Some(arr) => Some(&arr[*id]),
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

        match &self.hudhandler {
            Some(handler) => {
                cr.save();
                let hudret = handler.render(self, cr);
                cr.restore();
                hudret?;
            }
            None => {}
        }

        return Ok(());
    }
}

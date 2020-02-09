use crate::*;

pub(crate) struct SwingBoat {}

impl SwingBoat {
    pub fn new(_ctx: &render::Context) -> Self {
        Self {}
    }
}

impl render::HudRenderer for SwingBoat {
    fn render(&self, _ctx: &render::Context, _cr: &cairo::Context) -> Result<(), Error> {
        return Ok(());
    }
}

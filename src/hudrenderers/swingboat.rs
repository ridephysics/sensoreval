use crate::*;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct Config {}

impl Default for Config {
    fn default() -> Self {
        Self {}
    }
}

pub(crate) struct SwingBoat {
    cfg: Config,
}

impl SwingBoat {
    pub fn new(_ctx: &render::Context, cfg: &Config) -> Self {
        Self {
            cfg: (*cfg).clone(),
        }
    }
}

impl render::HudRenderer for SwingBoat {
    fn render(&self, _ctx: &render::Context, _cr: &cairo::Context) -> Result<(), Error> {
        Ok(())
    }
}

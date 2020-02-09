use crate::*;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct Config {}

impl Default for Config {
    fn default() -> Self {
        Self {}
    }
}

pub(crate) struct Pendulum {
    cfg: Config,
}

impl Pendulum {
    pub fn new(_ctx: &render::Context, cfg: &Config) -> Self {
        Self {
            cfg: (*cfg).clone(),
        }
    }
}

impl render::HudRenderer for Pendulum {
    fn render(&self, _ctx: &render::Context, _cr: &cairo::Context) -> Result<(), Error> {
        Ok(())
    }

    fn plot(&self, ctx: &render::Context) -> Result<(), Error> {
        Ok(())
    }
}

use crate::*;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Config {
    radius: f64,
}

pub fn generate(cfg: &Config) -> Result<Vec<crate::Data>, Error> {
    Ok(Vec::new())
}

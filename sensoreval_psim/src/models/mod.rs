pub mod booster;
pub use booster::Booster;
pub use booster::Params as BoosterParams;

pub mod double_pendulum;
pub use double_pendulum::DoublePendulum;

pub mod pendulum;
pub use pendulum::Params as PendulumParams;
pub use pendulum::Pendulum;

use crate::DrawState;
use crate::Model;
use crate::ToImuSample;

#[derive(serde::Deserialize, Debug)]
#[serde(tag = "type")]
pub enum Params {
    #[serde(rename = "booster")]
    Booster(booster::Params),
    #[serde(rename = "pendulum")]
    Pendulum(pendulum::Params),
}

impl Params {
    pub fn to_model_enum(&self, dt: f64) -> ModelEnum {
        match self {
            Self::Booster(p) => Booster::new(p.clone(), dt).into(),
            Self::Pendulum(p) => Pendulum::new(p.clone(), dt).into(),
        }
    }
}

#[enum_dispatch::enum_dispatch(DrawState)]
#[enum_dispatch::enum_dispatch(Model)]
#[enum_dispatch::enum_dispatch(ToImuSample)]
#[derive(Clone)]
pub enum ModelEnum {
    Booster,
    Pendulum,
}

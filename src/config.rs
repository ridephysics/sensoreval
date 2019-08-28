use crate::*;

use nalgebra::geometry::UnitQuaternion;
use serde_derive::Deserialize;
use std::io::Read;

#[derive(Deserialize, Debug)]
pub struct Video {
    #[serde(default)]
    pub startoff: u64,
    #[serde(default)]
    pub endoff: u64,
}

impl Default for Video {
    fn default() -> Self {
        Self {
            startoff: 0,
            endoff: 0,
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct Data {
    #[serde(default)]
    pub startoff: u64,
    #[serde(default = "UnitQuaternion::identity")]
    pub imu_orientation: UnitQuaternion<f64>,
    #[serde(default)]
    pub pressure_coeff: f64,
}

impl Default for Data {
    fn default() -> Self {
        Self {
            startoff: 0,
            imu_orientation: UnitQuaternion::identity(),
            pressure_coeff: 0.,
        }
    }
}

#[derive(Deserialize, Debug)]
pub enum OrientationMode {
    Normal,
}

impl Default for OrientationMode {
    fn default() -> Self {
        Self::Normal
    }
}

#[derive(Deserialize, Debug)]
pub struct Orientation {
    #[serde(default)]
    pub mode: OrientationMode,
}

impl Default for Orientation {
    fn default() -> Self {
        Self {
            mode: OrientationMode::Normal,
        }
    }
}

#[derive(Deserialize, Debug)]
pub enum HudMode {
    Generic,
    SwingBoat,
}

impl Default for HudMode {
    fn default() -> Self {
        Self::Generic
    }
}

#[derive(Deserialize, Debug)]
pub struct Hud {
    #[serde(default)]
    pub mode: HudMode,
    #[serde(default)]
    pub altitude_ground: f64,

    #[serde(default)]
    pub swingboat: SwingBoat,
}

impl Default for Hud {
    fn default() -> Self {
        Self {
            mode: HudMode::Generic,
            altitude_ground: 0.,
            swingboat: SwingBoat::default(),
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct Config {
    #[serde(default)]
    pub video: Video,
    pub data: Data,
    #[serde(default)]
    pub orientation: Orientation,
    #[serde(default)]
    pub hud: Hud,
}

#[derive(Deserialize, Debug)]
pub struct SwingBoat {}

impl Default for SwingBoat {
    fn default() -> Self {
        Self {}
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            video: Video::default(),
            data: Data::default(),
            orientation: Orientation::default(),
            hud: Hud::default(),
        }
    }
}

pub fn load(filename: std::string::String) -> Result<Config, Error> {
    let mut file = std::fs::File::open(&filename)?;
    let mut buffer = String::new();

    file.read_to_string(&mut buffer)?;
    drop(file);

    let mut cfg: Config = toml::from_str(&buffer)?;

    {
        let q = cfg.data.imu_orientation.as_mut_unchecked();

        // we loaded a wxyz quat, even though we need a xyzw quat, fix that
        let w = q[0];
        let x = q[1];
        let y = q[2];
        let z = q[3];
        q[0] = x;
        q[1] = y;
        q[2] = z;
        q[3] = w;
    }

    // we deserialized a normal quat into a unit-quat, fix that
    cfg.data.imu_orientation.renormalize();

    return Ok(cfg);
}

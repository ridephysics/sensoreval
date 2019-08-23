use crate::error::*;

use serde_derive::Deserialize;
use std::io::Read;

#[derive(Deserialize, Debug)]
pub struct Video {
    startoff: u64,
    endoff: u64,
}

impl Default for Video {
    fn default() -> Video {
        Video {
            startoff: 0,
            endoff: 0,
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct Data {
    startoff: u64,
    imu_orientation: [f64; 4],
}

impl Default for Data {
    fn default() -> Data {
        Data {
            startoff: 0,
            imu_orientation: [1., 0., 0., 0.],
        }
    }
}

#[derive(Deserialize, Debug)]
pub enum OrientationMode {
    Normal,
}

#[derive(Deserialize, Debug)]
pub struct Orientation {
    mode: OrientationMode,
}

impl Default for Orientation {
    fn default() -> Orientation {
        Orientation {
            mode: OrientationMode::Normal,
        }
    }
}

#[derive(Deserialize, Debug)]
pub enum HudMode {
    Generic,
    SwingBoat,
}

#[derive(Deserialize, Debug)]
pub struct Hud {
    mode: HudMode,
    altitude_ground: f64,

    swingboat: SwingBoat,
}

impl Default for Hud {
    fn default() -> Hud {
        Hud {
            mode: HudMode::Generic,
            altitude_ground: 0.,
            swingboat: SwingBoat::default(),
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct Config {
    video: Video,
    data: Data,
    orientation: Orientation,
    hud: Hud,
}

#[derive(Deserialize, Debug)]
pub struct SwingBoat {}

impl Default for SwingBoat {
    fn default() -> SwingBoat {
        SwingBoat {}
    }
}

impl Default for Config {
    fn default() -> Config {
        Config {
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

    return match toml::from_str(&buffer) {
        Err(e) => Err(Error::from(e)),
        Ok(v) => Ok(v),
    };
}

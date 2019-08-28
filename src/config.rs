use crate::*;

use nalgebra::geometry::UnitQuaternion;
use serde::Deserialize;
use serde_derive::Deserialize;
use std::io::Read;

#[derive(Deserialize, Debug)]
pub struct Video {
    #[serde(default)]
    pub startoff: u64,
    #[serde(default)]
    pub endoff: u64,
    #[serde(default)]
    pub filename: Option<String>,
}

impl Default for Video {
    fn default() -> Self {
        Self {
            startoff: 0,
            endoff: 0,
            filename: None,
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
    pub filename: String,
    pub format: String,
    #[serde(default)]
    pub mag_cal: Option<String>,
    #[serde(default)]
    pub bias_ag: Option<String>,
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

fn path2abs(dir: &std::path::Path, relpath: &String) -> String {
    String::from(dir.join(std::path::Path::new(&relpath)).to_str().unwrap())
}

pub fn load<P: AsRef<std::path::Path>>(filename: P) -> Result<Config, Error> {
    let mut file = std::fs::File::open(filename.as_ref())?;
    let mut buffer = String::new();
    let cfgdir = std::path::Path::new(filename.as_ref())
        .parent()
        .expect("can't get parent dir of config");

    file.read_to_string(&mut buffer)?;
    drop(file);

    let mut parser = toml::de::Deserializer::new(&buffer);
    let value = toml::Value::deserialize(&mut parser)?;
    let mut has_unsupported: bool = false;
    let mut cfg: Config = serde_ignored::deserialize(value, |path| {
        println!("unsupported config: {:?}", path.to_string());
        has_unsupported = true;
    })?;
    if has_unsupported {
        return Err(Error::from(ErrorRepr::UnsupportedConfigs));
    }

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

    // make all paths absolute
    cfg.data.filename = path2abs(&cfgdir, &cfg.data.filename);
    match cfg.video.filename {
        Some(v) => {
            cfg.video.filename = Some(path2abs(&cfgdir, &v));
        }
        None => (),
    }
    match cfg.data.mag_cal {
        Some(v) => {
            cfg.data.mag_cal = Some(path2abs(&cfgdir, &v));
        }
        None => (),
    }
    match cfg.data.bias_ag {
        Some(v) => {
            cfg.data.bias_ag = Some(path2abs(&cfgdir, &v));
        }
        None => (),
    }

    return Ok(cfg);
}

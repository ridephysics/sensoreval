use crate::*;

use serde::Deserialize;
use std::io::Read;

#[derive(Deserialize, Debug)]
pub struct Video {
    #[serde(default)]
    pub startoff: u64,
    #[serde(default)]
    pub endoff: Option<u64>,
    #[serde(default)]
    pub filename: Option<String>,
}

impl Default for Video {
    fn default() -> Self {
        Self {
            startoff: 0,
            endoff: None,
            filename: None,
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct AxisMap(Vec<isize>);

impl AxisMap {
    #[inline(always)]
    pub fn copy_single<A, T>(&self, dst: &mut A, src: &[T], dstidx: usize)
    where
        A: std::ops::IndexMut<usize, Output = T>,
        T: Copy + std::ops::Neg<Output = T>,
    {
        let mut srcidx = self.0[dstidx].abs() as usize;
        assert!(srcidx != 0);
        srcidx -= 1;

        let mut tmp = src[srcidx];
        if self.0[dstidx] < 0 {
            tmp = -tmp;
        }
        dst[dstidx] = tmp;
    }

    #[inline(always)]
    pub fn copy<A, T>(&self, dst: &mut A, src: &[T])
    where
        A: std::ops::IndexMut<usize, Output = T>,
        T: Copy + std::ops::Neg<Output = T>,
    {
        for i in 0..self.0.len() {
            self.copy_single(dst, src, i);
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct AxisMaps {
    pub accel: AxisMap,
    pub gyro: AxisMap,
    pub mag: AxisMap,
}

impl Default for AxisMaps {
    fn default() -> Self {
        Self {
            accel: AxisMap(vec![1, 2, 3]),
            gyro: AxisMap(vec![1, 2, 3]),
            mag: AxisMap(vec![1, 2, 3]),
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct Data {
    #[serde(default)]
    pub video_off: i64,
    #[serde(default)]
    pub axismaps: AxisMaps,
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
#[serde(tag = "type")]
pub enum HudRenderer {
    #[serde(rename = "generic")]
    Generic,
    #[serde(rename = "swingboat")]
    SwingBoat(hudrenderers::swingboat::Config),
}

impl Default for HudRenderer {
    fn default() -> Self {
        Self::Generic
    }
}

#[derive(Deserialize, Debug)]
pub struct Hud {
    #[serde(default)]
    pub renderer: HudRenderer,
    #[serde(default)]
    pub altitude_ground: f64,
}

impl Default for Hud {
    fn default() -> Self {
        Self {
            renderer: HudRenderer::Generic,
            altitude_ground: 0.,
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

fn path2abs(dir: &std::path::Path, relpath: &str) -> String {
    String::from(dir.join(std::path::Path::new(&relpath)).to_str().unwrap())
}

#[allow(clippy::many_single_char_names)]
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
        return Err(Error::UnsupportedConfigs);
    }

    // make all paths absolute
    cfg.data.filename = path2abs(&cfgdir, &cfg.data.filename);
    if let Some(v) = cfg.video.filename {
        cfg.video.filename = Some(path2abs(&cfgdir, &v));
    }
    if let Some(v) = cfg.data.mag_cal {
        cfg.data.mag_cal = Some(path2abs(&cfgdir, &v));
    }
    if let Some(v) = cfg.data.bias_ag {
        cfg.data.bias_ag = Some(path2abs(&cfgdir, &v));
    }

    Ok(cfg)
}

#[cfg(test)]
mod test {
    use super::*;
    use ndarray::array;

    #[test]
    fn axismap() {
        let mut dst = ndarray::Array::zeros(3);
        let src: [isize; 3] = [10, 20, 30];

        let map = AxisMap(vec![1, 2, 3]);
        map.copy(&mut dst, &src);
        assert_eq!(dst, array![10, 20, 30]);

        let map = AxisMap(vec![1, 3, 2]);
        map.copy(&mut dst, &src);
        assert_eq!(dst, array![10, 30, 20]);

        let map = AxisMap(vec![1, -2, 3]);
        map.copy(&mut dst, &src);
        assert_eq!(dst, array![10, -20, 30]);

        let map = AxisMap(vec![1, 3, -2]);
        map.copy(&mut dst, &src);
        assert_eq!(dst, array![10, 30, -20]);
    }
}

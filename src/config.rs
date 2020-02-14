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

impl Default for AxisMap {
    fn default() -> Self {
        Self(vec![1, 2, 3])
    }
}

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
pub struct SensorData {
    #[serde(default)]
    pub video_off: i64,
    #[serde(default)]
    pub axismap: AxisMap,
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
#[serde(tag = "type")]
pub enum DataSource {
    #[serde(rename = "sensordata")]
    SensorData(SensorData),
    #[serde(rename = "sim_pendulum")]
    SimPendulum(simulator::pendulum::Config),
}

#[derive(Deserialize, Debug, Default)]
pub struct NoiseXYZ {
    #[serde(default)]
    pub x: Option<std::ops::Range<f64>>,
    #[serde(default)]
    pub y: Option<std::ops::Range<f64>>,
    #[serde(default)]
    pub z: Option<std::ops::Range<f64>>,
}

#[derive(Deserialize, Debug, Default)]
pub struct DataNoise {
    #[serde(default)]
    pub accel: NoiseXYZ,
    #[serde(default)]
    pub gyro: NoiseXYZ,
    #[serde(default)]
    pub mag: NoiseXYZ,
}

#[derive(Deserialize, Debug)]
pub struct Data {
    pub source: DataSource,
    #[serde(default)]
    pub noise: DataNoise,
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
    #[serde(rename = "pendulum")]
    Pendulum(hudrenderers::pendulum::Config),
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

impl Config {
    fn add_noise<S, R>(arr: &mut ndarray::ArrayBase<S, ndarray::Ix1>, cfg: &NoiseXYZ, rng: &mut R)
    where
        S: ndarray::DataMut<Elem = f64>,
        R: rand::Rng,
    {
        if let Some(n) = &cfg.x {
            arr[0] += rng.gen_range(n.start, n.end);
        }

        if let Some(n) = &cfg.y {
            arr[1] += rng.gen_range(n.start, n.end);
        }

        if let Some(n) = &cfg.z {
            arr[2] += rng.gen_range(n.start, n.end);
        }
    }

    pub fn load_data(&self) -> Result<Vec<crate::Data>, Error> {
        let mut ret = match &self.data.source {
            DataSource::SensorData(_) => datareader::read_all_samples_cfg(self),
            DataSource::SimPendulum(cfg) => simulator::pendulum::generate(cfg),
        };

        if let Ok(samples) = &mut ret {
            let mut rng = rand::thread_rng();

            for sample in samples {
                Self::add_noise(&mut sample.accel, &self.data.noise.accel, &mut rng);
                Self::add_noise(&mut sample.gyro, &self.data.noise.gyro, &mut rng);
                Self::add_noise(&mut sample.mag, &self.data.noise.mag, &mut rng);
            }
        }

        ret
    }
}

fn path2abs(dir: &std::path::Path, relpath: &str) -> String {
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
        return Err(Error::UnsupportedConfigs);
    }

    // make all paths absolute

    if let DataSource::SensorData(sd) = &mut cfg.data.source {
        sd.filename = path2abs(&cfgdir, &sd.filename);
        if let Some(v) = &sd.mag_cal {
            sd.mag_cal = Some(path2abs(&cfgdir, &v));
        }
        if let Some(v) = &sd.bias_ag {
            sd.bias_ag = Some(path2abs(&cfgdir, &v));
        }
    }

    if let Some(v) = cfg.video.filename {
        cfg.video.filename = Some(path2abs(&cfgdir, &v));
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

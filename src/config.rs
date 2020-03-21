use crate::*;

use serde::Deserialize;
use std::io::Read;

/// video source information
#[derive(Deserialize, Debug)]
pub struct Video {
    /// start offset in milli seconds
    #[serde(default)]
    pub startoff: u64,
    /// end offset in milli seconds
    #[serde(default)]
    pub endoff: Option<u64>,
    /// relative path to video file
    #[serde(default)]
    pub filename: Option<String>,
    /// relative path to blur mask
    #[serde(default)]
    pub blurmask: Option<String>,
}

impl Default for Video {
    fn default() -> Self {
        Self {
            startoff: 0,
            endoff: None,
            filename: None,
            blurmask: None,
        }
    }
}

/// map sensor axes. index: destination, value: source + 1, can be negative
#[derive(Deserialize, Debug)]
pub struct AxisMap(Vec<isize>);

impl Default for AxisMap {
    fn default() -> Self {
        Self(vec![1, 2, 3])
    }
}

impl AxisMap {
    /// copy one axis
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

    /// copy all axes
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

/// sensordata data source
#[derive(Deserialize, Debug)]
pub struct SensorData {
    /// time offset relative to the start of the video (ignoring it's startoff), unit: micro seconds
    #[serde(default)]
    pub video_off: i64,
    /// axismap for accel, gyro and mag. They're not separate because
    /// they're expected to be aligned to each other already
    #[serde(default)]
    pub axismap: AxisMap,
    /// barometer pressure coefficient used for smoothing the data
    #[serde(default)]
    pub pressure_coeff: f64,
    /// relative path to the IMU data. this will be passed to usfs_reader
    pub filename: String,
    /// IMU data format. this will be passed to usfs_reader
    pub format: String,
    /// relative path to the magnetometer calibration file, this will be passed to usfs_reader
    #[serde(default)]
    pub mag_cal: Option<String>,
    /// relative path to the accel/gyro bias file, this will be passed to usfs_reader
    #[serde(default)]
    pub bias_ag: Option<String>,
    /// relative path to calibration info
    #[serde(default)]
    pub calibration: Option<String>,
}

/// data source type and information
#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
pub enum DataSource {
    /// use actual sensor data
    #[serde(rename = "sensordata")]
    SensorData(SensorData),
    /// use the pendulum simulator
    #[serde(rename = "sim_pendulum")]
    SimPendulum(simulator::pendulum::Config),
}

/// noise for X, Y and Z
#[derive(Deserialize, Debug, Default)]
pub struct NoiseXYZ {
    /// range passed to [gen_range](../../rand/trait.Rng.html#method.gen_range)
    #[serde(default)]
    pub x: Option<std::ops::Range<f64>>,
    /// range passed to [gen_range](../../rand/trait.Rng.html#method.gen_range)
    #[serde(default)]
    pub y: Option<std::ops::Range<f64>>,
    /// range passed to [gen_range](../../rand/trait.Rng.html#method.gen_range)
    #[serde(default)]
    pub z: Option<std::ops::Range<f64>>,
}

/// noise for all sensor types
#[derive(Deserialize, Debug, Default)]
pub struct DataNoise {
    /// accelerometer noise, unit: same as [Config.accel](../struct.Data.html#structfield.accel)
    #[serde(default)]
    pub accel: NoiseXYZ,
    /// gyroscope noise, unit: same as [Config.gyro](../struct.Data.html#structfield.gyro)
    #[serde(default)]
    pub gyro: NoiseXYZ,
    /// magnetometer noise, unit: same as [Config.mag](../struct.Data.html#structfield.mag)
    #[serde(default)]
    pub mag: NoiseXYZ,
}

/// data configuration
#[derive(Deserialize, Debug)]
pub struct Data {
    /// data source type and information
    pub source: DataSource,
    /// optionally add noise to the data using thread_rng
    #[serde(default)]
    pub noise: DataNoise,
}

/// renderer type for the HUD and the data plot
#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
pub enum HudRenderer {
    /// generic renderer which doesn't do anything
    #[serde(rename = "generic")]
    Generic,
    /// pendulum renderer
    #[serde(rename = "pendulum")]
    Pendulum(hudrenderers::pendulum::Config),
}

impl Default for HudRenderer {
    fn default() -> Self {
        Self::Generic
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

/// HUD config
#[derive(Deserialize, Debug)]
pub struct Hud {
    /// renderer type and information
    #[serde(default)]
    pub renderer: HudRenderer,
    /// mannequin orientation mode
    #[serde(default)]
    pub orientation_mode: OrientationMode,
}

impl Default for Hud {
    fn default() -> Self {
        Self {
            renderer: HudRenderer::Generic,
            orientation_mode: OrientationMode::default(),
        }
    }
}

/// global configuration
#[derive(Deserialize, Debug)]
pub struct Config {
    /// video config
    #[serde(default)]
    pub video: Video,
    /// data config
    pub data: Data,
    // HUD config
    #[serde(default)]
    pub hud: Hud,
}

/// standard deviation for one sensor's XYZ axes
#[derive(Deserialize, Debug, Clone)]
pub struct SensorStdevXYZ {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

/// standard deviation for all sensors
#[derive(Deserialize, Debug, Clone)]
pub struct SensorStdev {
    /// unit: same as [Config.accel](../struct.Data.html#structfield.accel)
    pub accel: SensorStdevXYZ,
    /// unit: same as [Config.gyro](../struct.Data.html#structfield.gyro)
    pub gyro: SensorStdevXYZ,
    /// unit: same as [Config.mag](../struct.Data.html#structfield.mag)
    pub mag: SensorStdevXYZ,
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

    /// load data from configured source
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

/// load config file
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
        if let Some(v) = &sd.calibration {
            sd.calibration = Some(path2abs(&cfgdir, &v));
        }
    }

    if let Some(v) = cfg.video.filename {
        cfg.video.filename = Some(path2abs(&cfgdir, &v));
    }
    if let Some(v) = cfg.video.blurmask {
        cfg.video.blurmask = Some(path2abs(&cfgdir, &v));
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

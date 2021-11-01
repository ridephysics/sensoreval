use anyhow::anyhow;
use anyhow::Context as _;

/// sensor calibration info
#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct Calibration {
    pub gyro_offs: ndarray::Array1<f64>,
    pub accel_offs: ndarray::Array1<f64>,
    pub accel_t: ndarray::Array2<f64>,
}

impl Calibration {
    pub fn new(
        gyro_offs: ndarray::Array1<f64>,
        accel_offs: ndarray::Array1<f64>,
        accel_t: ndarray::Array2<f64>,
    ) -> Self {
        Self {
            gyro_offs,
            accel_offs,
            accel_t,
        }
    }

    pub fn load<P: AsRef<std::path::Path>>(path: P) -> Result<Self, anyhow::Error> {
        let mut file = std::fs::File::open(path)?;
        Ok(bincode::deserialize_from(&mut file)?)
    }
}

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    path: std::path::PathBuf,
}

pub fn apply(config: &Config, dataset: &mut crate::Dataset) -> anyhow::Result<()> {
    let calibration = Calibration::load(&config.path).context("can't load calibration file")?;

    let mut accel_all = dataset
        .get_kind_mut("accel")
        .ok_or_else(|| anyhow!("can't find accel"))?
        .view_mut()
        .into_dimensionality::<ndarray::Ix2>()
        .context("unsupported accel dimension")?;
    for mut accel in accel_all.lanes_mut(ndarray::Axis(1)) {
        accel.assign(&calibration.accel_t.dot(&(&accel - &calibration.accel_offs)));
    }

    let mut gyro_all = dataset
        .get_kind_mut("gyro")
        .ok_or_else(|| anyhow!("can't find gyro"))?
        .view_mut()
        .into_dimensionality::<ndarray::Ix2>()
        .context("unsupported gyro dimension")?;
    for mut gyro in gyro_all.lanes_mut(ndarray::Axis(1)) {
        gyro -= &calibration.gyro_offs;
    }

    Ok(())
}

/// processed data sample
#[derive(Debug)]
pub struct Data {
    /// timestamp for accel, gyro and mag. unit: micro seconds
    pub time: u64,
    /// timestamp for temperature and pressure, unit: micro seconds
    pub time_baro: u64,

    /// accelerometer sample, unit: m/s^2
    pub accel: ndarray::Array1<f64>,
    /// gyroscope sample, unit: rad/s
    pub gyro: ndarray::Array1<f64>,
    /// magnetometer sample, unit: uT
    pub mag: ndarray::Array1<f64>,

    /// barometer temperature, unit: degrees celsius
    pub temperature: f64,
    /// barometer pressure, unit: hPa
    pub pressure: f64,

    /// optional actual state data, e.g. from the simulator that generated the sample
    pub actual: Option<ndarray::Array1<f64>>,
}

impl Default for Data {
    fn default() -> Self {
        Self {
            time: 0,
            time_baro: 0,
            accel: ndarray::Array::zeros(3),
            gyro: ndarray::Array::zeros(3),
            mag: ndarray::Array::zeros(3),
            temperature: 0.,
            pressure: 0.,
            actual: None,
        }
    }
}

impl std::ops::AddAssign<&Self> for Data {
    fn add_assign(&mut self, other: &Self) {
        self.accel += &other.accel;
        self.gyro += &other.gyro;
        self.mag += &other.mag;
        self.temperature += other.temperature;
        self.pressure += other.pressure;
    }
}

impl std::ops::Div<usize> for Data {
    type Output = Self;

    fn div(mut self, rhs: usize) -> Self::Output {
        self.accel /= rhs as f64;
        self.gyro /= rhs as f64;
        self.mag /= rhs as f64;
        self.temperature /= rhs as f64;
        self.pressure /= rhs as f64;

        self
    }
}

impl Data {
    fn pressure_altitude_feet(&self) -> f64 {
        145_366.45 * (1.0 - (self.pressure / 1013.25).powf(0.190_284))
    }

    pub fn pressure_altitude(&self) -> f64 {
        self.pressure_altitude_feet() * 0.3048
    }

    pub fn time_seconds(&self) -> f64 {
        (self.time as f64) / 1_000_000.0
    }
}

/// return dataset array index at or after the given time
pub fn id_for_time(dataset: &[Data], startid: usize, us: u64) -> Option<usize> {
    if startid >= dataset.len() {
        return None;
    }

    for (i, sample) in dataset[startid..].iter().enumerate() {
        match sample.time.cmp(&us) {
            std::cmp::Ordering::Equal => return Some(i),
            std::cmp::Ordering::Greater => {
                if i == 0 {
                    return Some(0);
                } else {
                    return Some(i - 1);
                }
            }
            std::cmp::Ordering::Less => (),
        }
    }

    None
}

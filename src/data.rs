use crate::*;

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

    for (i, sample) in (&dataset[startid..]).iter().enumerate() {
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

pub fn downscale(lores: &mut Vec<Data>, dataset: &[Data], timeframe: u64) -> Result<(), Error> {
    let data_last = unwrap_opt_or!(dataset.last(), return Err(Error::SampleNotFound));
    let lores_len = data_last.time / timeframe;
    let mut j = 0;

    for i in 0..(lores_len as usize) {
        let mut nsamples: usize = 0;
        let mut data_lores = Data::default();

        data_lores.time = (i as u64) * timeframe + timeframe / 2;

        // sum up all data
        while let Some(data) = dataset.get(j) {
            if data.time >= (i as u64) * timeframe {
                break;
            }
            j += 1;

            for k in 0..3 {
                data_lores.accel[k] += data.accel[k];
                data_lores.gyro[k] += data.gyro[k];
                data_lores.mag[k] += data.mag[k];
            }

            data_lores.temperature += data.temperature;
            data_lores.pressure += data.pressure;

            nsamples += 1;
        }

        if nsamples > 0 {
            // calculate the mean values
            for k in 0..3 {
                data_lores.accel[k] /= nsamples as f64;
                data_lores.gyro[k] /= nsamples as f64;
                data_lores.mag[k] /= nsamples as f64;
            }

            data_lores.temperature /= nsamples as f64;
            data_lores.pressure /= nsamples as f64;
        } else if i > 0 {
            // use the previous sample
            let prev = dataset.get(i - 1).unwrap();

            for k in 0..3 {
                data_lores.accel[k] = prev.accel[k];
                data_lores.gyro[k] = prev.gyro[k];
                data_lores.mag[k] = prev.mag[k];
            }

            data_lores.temperature = prev.temperature;
            data_lores.pressure = prev.pressure;
        } else {
            // use the first sample
            let first = dataset.get(i).unwrap();

            for k in 0..3 {
                data_lores.accel[k] = first.accel[k];
                data_lores.gyro[k] = first.gyro[k];
                data_lores.mag[k] = first.mag[k];
            }

            data_lores.temperature = first.temperature;
            data_lores.pressure = first.pressure;
        }

        lores.push(data_lores);
    }

    Ok(())
}

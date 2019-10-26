use crate::*;

use nalgebra::base::Vector3;
use nalgebra::geometry::UnitQuaternion;
use serde::ser::{Serialize, SerializeSeq, Serializer};

#[derive(Debug)]
pub struct Data {
    // unit: microseconds
    pub time: u64,
    // unit: microseconds
    pub time_baro: u64,

    // unit: g
    pub accel: Vector3<f64>,
    // unit: dps
    pub gyro: Vector3<f64>,
    // unit: uT
    pub mag: Vector3<f64>,

    // format: ENU
    pub quat: UnitQuaternion<f64>,

    // unit: degrees celsius
    pub temperature: f64,
    // unit: hPa
    pub pressure: f64,
}

impl Default for Data {
    fn default() -> Self {
        Self {
            time: 0,
            time_baro: 0,
            accel: Vector3::new(0., 0., 0.),
            gyro: Vector3::new(0., 0., 0.),
            mag: Vector3::new(0., 0., 0.),
            quat: UnitQuaternion::identity(),
            temperature: 0.,
            pressure: 0.,
        }
    }
}

impl Data {
    fn pressure_altitude_feet(&self) -> f64 {
        return 145366.45 * (1.0 - (self.pressure / 1013.25).powf(0.190284));
    }

    pub fn pressure_altitude(&self) -> f64 {
        return self.pressure_altitude_feet() * 0.3048;
    }

    pub fn time_seconds(&self) -> f64 {
        return (self.time as f64) / 1000000.0;
    }
}

pub struct DataIterator<I, F> {
    iter: I,
    f: F,
}

impl<'a, D: 'a, I: Iterator<Item = &'a D>, R, F: Fn(&D) -> R> Iterator for DataIterator<I, F> {
    type Item = R;

    fn next(&mut self) -> Option<Self::Item> {
        match self.iter.next() {
            Some(data) => Some((self.f)(&data)),
            _ => None,
        }
    }
}

impl<'a, D: 'a, I: Iterator<Item = &'a D>, R, F: Fn(&D) -> R> DataIterator<I, F> {
    pub fn new(iter: I, f: F) -> Self {
        Self { iter: iter, f: f }
    }
}

pub struct DataSerializer<'a, D, F> {
    list: &'a Vec<D>,
    f: F,
}

impl<'a, D, ST: Serialize, F: Fn(usize, &D) -> ST> Serialize for DataSerializer<'a, D, F> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.list.len()))?;
        let mut i: usize = 0;
        for data in self.list {
            seq.serialize_element(&(self.f)(i, &data))?;
            i += 1;
        }
        seq.end()
    }
}

impl<'a, D, ST: Serialize, F: Fn(usize, &D) -> ST> DataSerializer<'a, D, F> {
    pub fn new(list: &'a Vec<D>, f: F) -> Self {
        Self { list: list, f: f }
    }
}

pub fn id_for_time(dataset: &Vec<Data>, startid: usize, us: u64) -> Option<usize> {
    if startid >= dataset.len() {
        return None;
    }

    for i in startid..dataset.len() {
        let sample = &dataset[i];
        if sample.time >= us {
            return Some(i);
        }
    }

    return None;
}

pub fn downscale(lores: &mut Vec<Data>, dataset: &Vec<Data>, timeframe: u64) -> Result<(), Error> {
    let data_last = unwrap_opt_or!(
        dataset.last(),
        return Err(Error::from(ErrorRepr::SampleNotFound))
    );
    let lores_len = data_last.time / timeframe;
    let mut j = 0;

    for i in 0..(lores_len as usize) {
        let mut nsamples: usize = 0;
        let mut data_lores = Data::default();
        let mut quat: Option<&UnitQuaternion<f64>> = None;
        let startid = j;

        data_lores.time = (i as u64) * timeframe + timeframe / 2;

        // sum up all data
        loop {
            let data = unwrap_opt_or!(dataset.get(j), break);
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

            // find the quat closest to our time
            if quat.is_none() && data.time >= data_lores.time {
                if j > 0
                    && data_lores.time - dataset.get(j - 1).unwrap().time
                        < data.time - data_lores.time
                {
                    quat = Some(&dataset.get(j - 1).unwrap().quat);
                } else {
                    quat = Some(&data.quat);
                }
            }

            nsamples += 1;
        }

        // copy the quat we found
        if quat.is_none() {
            quat = Some(&dataset.get(startid).unwrap().quat);
        }
        data_lores.quat = quat.unwrap().clone();

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

    return Ok(());
}

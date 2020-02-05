use crate::*;

use nalgebra::base::Vector3;
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
            temperature: 0.,
            pressure: 0.,
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
        Self { iter, f }
    }
}

pub struct DataSerializer<'a, D, F> {
    list: &'a [D],
    f: F,
}

impl<'a, D, ST, F> Serialize for DataSerializer<'a, D, F>
where
    ST: 'a + Serialize,
    F: Fn(usize, &'a D) -> ST,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.list.len()))?;

        for (i, data) in self.list.iter().enumerate() {
            seq.serialize_element(&(self.f)(i, &data))?;
        }

        seq.end()
    }
}

impl<'a, D, ST, F> DataSerializer<'a, D, F>
where
    ST: 'a + Serialize,
    F: Fn(usize, &'a D) -> ST,
{
    pub fn new(list: &'a [D], f: F) -> Self {
        Self { list, f }
    }
}

pub fn id_for_time(dataset: &[Data], startid: usize, us: u64) -> Option<usize> {
    if startid >= dataset.len() {
        return None;
    }

    for (i, sample) in (&dataset[startid..]).iter().enumerate() {
        if sample.time >= us {
            return Some(i);
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

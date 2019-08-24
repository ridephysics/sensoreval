use crate::*;

use nalgebra::geometry::{Quaternion, UnitQuaternion};
use std::mem;

fn read_primitive<S: std::io::Read, R: for<'a> serde::de::Deserialize<'a>>(
    source: &mut S,
) -> Result<R, Error> {
    let mut buffer = vec![0; mem::size_of::<R>()];
    source.read_exact(&mut buffer)?;

    return Ok(bincode::deserialize(&buffer)?);
}

fn filter_eof<R>(r: Result<R, Error>) -> Result<R, Error> {
    match &r {
        Err(e) => match &e.repr {
            ErrorRepr::Io(eio) => match eio.kind() {
                std::io::ErrorKind::UnexpectedEof => Err(Error::from(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "EOF within sample",
                ))),
                _ => r,
            },
            _ => r,
        },
        Ok(_) => r,
    }
}

pub fn read_sample<S: std::io::Read>(source: &mut S, cfg: &config::Config) -> Result<Data, Error> {
    let mut data = Data::default();
    let imu_orientation_inv = cfg.data.imu_orientation.inverse();

    // this is the only one where we let through EOF
    // so the the caller can see the diff between a proper end
    // and an end in the middle of the data
    data.time = read_primitive(source)?;

    for i in 0..data.accel.len() {
        data.accel[i] = filter_eof(read_primitive(source))?;
    }
    data.accel = imu_orientation_inv * data.accel;

    for i in 0..data.gyro.len() {
        data.gyro[i] = filter_eof(read_primitive(source))?;
    }
    data.gyro = imu_orientation_inv * data.gyro;

    for i in 0..data.mag.len() {
        data.mag[i] = filter_eof(read_primitive(source))?;
    }
    data.mag = imu_orientation_inv * data.mag;

    {
        let _: u64 = filter_eof(read_primitive(source))?;
    }

    data.temperature = filter_eof(read_primitive(source))?;
    data.pressure = filter_eof(read_primitive(source))?;

    {
        let w: f64 = filter_eof(read_primitive(source))?;
        let x: f64 = filter_eof(read_primitive(source))?;
        let y: f64 = filter_eof(read_primitive(source))?;
        let z: f64 = filter_eof(read_primitive(source))?;

        let q = Quaternion::new(w, x, y, z);
        data.quat = UnitQuaternion::from_quaternion(q) * cfg.data.imu_orientation;
    }

    return Ok(data);
}

pub fn read_all_samples<S: std::io::Read>(
    source: &mut S,
    cfg: &config::Config,
) -> Result<Vec<Data>, Error> {
    let mut samples: Vec<Data> = Vec::new();

    loop {
        let mut sample = match read_sample(source, cfg) {
            Err(e) => match &e.repr {
                ErrorRepr::Io(eio) => match eio.kind() {
                    std::io::ErrorKind::UnexpectedEof => break,
                    _ => return Err(e),
                },
                _ => return Err(e),
            },
            Ok(v) => v,
        };

        // apply pressure coefficient
        if cfg.data.pressure_coeff > 0. {
            match samples.last() {
                Some(prev) => {
                    sample.pressure = (prev.pressure * (cfg.data.pressure_coeff - 1.0)
                        + sample.pressure)
                        / cfg.data.pressure_coeff;
                }
                None => {}
            }
        }

        samples.push(sample);
    }

    return Ok(samples);
}

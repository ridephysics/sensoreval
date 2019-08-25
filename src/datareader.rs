use crate::*;

use nalgebra::geometry::{Quaternion, UnitQuaternion};
use serde_derive::Deserialize;
use std::mem;

// https://github.com/rust-lang/rust/issues/27060
// in theory, this struct should never need alignment anyway
// because it's a sequence of 8-byte primitives
//#[repr(C, packed)]
#[derive(Deserialize)]
struct RawData {
    time_imu: u64,
    accel: [f64; 3],
    gyro: [f64; 3],
    mag: [f64; 3],
    time_compass: f64,
    temperature: f64,
    pressure: f64,
    quat: [f64; 4],
}

pub struct Context {
    buf: [u8; mem::size_of::<RawData>()],
    bufpos: usize,
}

impl Context {
    pub fn new() -> Self {
        assert_eq!(mem::size_of::<RawData>(), mem::size_of::<[u64; 17]>());

        Self {
            buf: [0; mem::size_of::<RawData>()],
            bufpos: 0,
        }
    }

    pub fn read_sample<S: std::io::Read>(
        &mut self,
        source: &mut S,
        cfg: &config::Config,
    ) -> Result<Data, Error> {
        loop {
            // read data
            let buflen = self.buf.len();
            let ret = source.read(&mut self.buf[self.bufpos..buflen]);
            let nbytes = match ret {
                Ok(0) => match self.bufpos {
                    0 => return Err(Error::from(ErrorRepr::EOF)),
                    _ => {
                        return Err(Error::new_io(
                            std::io::ErrorKind::UnexpectedEof,
                            "EOF within sample",
                        ))
                    }
                },
                Err(e) => match &e.kind() {
                    std::io::ErrorKind::Interrupted => continue,
                    _ => return Err(Error::from(e)),
                },
                Ok(v) => v,
            };

            // try again if we don't have enough yet
            self.bufpos += nbytes;
            assert!(self.bufpos <= buflen);
            if self.bufpos != buflen {
                continue;
            }
            self.bufpos = 0;

            // parse data
            let rawdata: RawData = bincode::deserialize(&self.buf)?;

            // turn rawdata into data
            let mut data = Data::default();
            let imu_orientation_inv = cfg.data.imu_orientation.inverse();

            data.time = rawdata.time_imu;
            data.temperature = rawdata.temperature;
            data.pressure = rawdata.pressure;

            for i in 0..3 {
                data.accel[i] = rawdata.accel[i];
                data.gyro[i] = rawdata.gyro[i];
                data.mag[i] = rawdata.mag[i];
            }
            data.accel = imu_orientation_inv * data.accel;
            data.gyro = imu_orientation_inv * data.gyro;
            data.mag = imu_orientation_inv * data.mag;

            {
                let w: f64 = rawdata.quat[0];
                let x: f64 = rawdata.quat[1];
                let y: f64 = rawdata.quat[2];
                let z: f64 = rawdata.quat[3];

                let q = Quaternion::new(w, x, y, z);
                data.quat = UnitQuaternion::from_quaternion(q) * cfg.data.imu_orientation;
            }

            return Ok(data);
        }
    }
}

pub fn read_all_samples<S: std::io::Read>(
    source: &mut S,
    cfg: &config::Config,
) -> Result<Vec<Data>, Error> {
    let mut samples: Vec<Data> = Vec::new();
    let mut readctx = Context::new();

    loop {
        let mut sample = match readctx.read_sample(source, cfg) {
            Err(e) => match &e.repr {
                ErrorRepr::EOF => break,
                ErrorRepr::Io(eio) => match eio.kind() {
                    std::io::ErrorKind::WouldBlock => {
                        // this could cause some cpu load but it's the best we can do here
                        continue;
                    }
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
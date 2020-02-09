use crate::*;

use serde::Deserialize;
use std::convert::TryInto;
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
    time_baro: u64,
    temperature: f64,
    pressure: f64,
    _quat: [f64; 4],
}

pub struct Context {
    buf: [u8; mem::size_of::<RawData>()],
    bufpos: usize,
    pressure_prev: Option<f64>,
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

impl Context {
    pub fn new() -> Self {
        assert_eq!(mem::size_of::<RawData>(), mem::size_of::<[u64; 17]>());

        Self {
            buf: [0; mem::size_of::<RawData>()],
            bufpos: 0,
            pressure_prev: None,
        }
    }

    pub fn read_sample<S: std::io::Read>(
        &mut self,
        source: &mut S,
        cfg: &config::SensorData,
    ) -> Result<Data, Error> {
        loop {
            // read data
            let buflen = self.buf.len();
            let ret = source.read(&mut self.buf[self.bufpos..buflen]);
            let nbytes = match ret {
                Ok(0) => match self.bufpos {
                    0 => return Err(Error::EOF),
                    _ => {
                        return Err(Error::from(std::io::Error::new(
                            std::io::ErrorKind::UnexpectedEof,
                            "EOF within sample",
                        )))
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

            data.time = rawdata.time_imu;
            data.time_baro = rawdata.time_baro;
            data.temperature = rawdata.temperature;
            data.pressure = rawdata.pressure;

            // copy axis data using mappping
            cfg.axismap.copy(&mut data.accel, &rawdata.accel);
            cfg.axismap.copy(&mut data.gyro, &rawdata.gyro);
            cfg.axismap.copy(&mut data.mag, &rawdata.mag);

            // apply pressure coefficient
            if cfg.pressure_coeff > 0. {
                if let Some(pressure_prev) = self.pressure_prev {
                    data.pressure = (pressure_prev * (cfg.pressure_coeff - 1.0) + data.pressure)
                        / cfg.pressure_coeff;
                }
            }

            // g -> m/s^2
            for a in &mut data.accel {
                *a *= math::GRAVITY;
            }

            // dps -> rad/s
            for g in &mut data.gyro {
                *g = (*g).to_radians();
            }

            self.pressure_prev = Some(data.pressure);
            return Ok(data);
        }
    }
}

fn time_imu2video(cfg: &config::SensorData, us: u64) -> Option<u64> {
    match cfg.video_off {
        x if x > 0 => {
            let off: u64 = x.try_into().unwrap();
            Some(us.checked_add(off).unwrap())
        }
        x if x < 0 => {
            let off: u64 = (-x).try_into().unwrap();
            match us.checked_sub(off) {
                Some(v) => Some(v),
                // just skip samples which came before T0
                None => None,
            }
        }
        _ => Some(us),
    }
}

pub fn read_all_samples_input<S: std::io::Read>(
    source: &mut S,
    cfg: &config::Config,
) -> Result<Vec<Data>, Error> {
    let datacfg = if let config::DataSource::SensorData(sd) = &cfg.data.source {
        sd
    } else {
        return Err(Error::UnsupportedDatatype);
    };

    let mut samples: Vec<Data> = Vec::new();
    let mut readctx = Context::new();

    loop {
        let sample = match readctx.read_sample(source, datacfg) {
            Err(e) => match &e {
                Error::EOF => break,
                Error::Io(eio) => match eio.kind() {
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

        if let Some(prev) = samples.last() {
            if prev.time > sample.time {
                println!("data jumped back in time");
                return Err(Error::SampleNotFound);
            }
        }

        samples.push(sample);
    }

    samples.drain_filter_stable(|sample| {
        let time = match time_imu2video(datacfg, sample.time) {
            Some(v) => v,
            None => return true,
        };
        let time_baro = match time_imu2video(datacfg, sample.time_baro) {
            Some(v) => v,
            None => return true,
        };

        // skip samples before the start of the video
        if time < cfg.video.startoff * 1000 {
            return true;
        }

        // skip samples after the end of the video
        if let Some(endoff) = cfg.video.endoff {
            if time > endoff * 1000 {
                return true;
            }
        }

        sample.time = time;
        sample.time_baro = time_baro;

        false
    });

    Ok(samples)
}

fn run_usfs_reader(cfg: &config::SensorData) -> std::process::Child {
    let mut args: Vec<&str> = Vec::new();

    args.push("--infmt");
    args.push(&cfg.format);
    args.push("--outfmt");
    args.push("processed");

    if let Some(v) = &cfg.mag_cal {
        args.push("--cal_mag");
        args.push(&v);
    }

    if let Some(v) = &cfg.bias_ag {
        args.push("--bias_ag");
        args.push(&v);
    }
    args.push(&cfg.filename);

    std::process::Command::new("usfs_reader")
        .args(args)
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("usfs_reader failed")
}

pub fn read_all_samples_cfg(cfg: &config::Config) -> Result<Vec<Data>, Error> {
    let datacfg = if let config::DataSource::SensorData(sd) = &cfg.data.source {
        sd
    } else {
        return Err(Error::UnsupportedDatatype);
    };

    let mut child_reader = run_usfs_reader(datacfg);

    let res = read_all_samples_input(&mut child_reader.stdout.take().unwrap(), cfg);
    assert!(child_reader
        .wait()
        .expect("can't wait for usfs_reader")
        .success());
    res
}

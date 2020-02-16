use crate::*;
use eom::traits::Scheme;
use ndarray::array;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Config {
    /// unit: meters
    radius: f64,
    /// unit: seconds
    dt: f64,
    /// unit: seconds
    duration: f64,
    /// unit: rad
    initial_angle: f64,
    /// unit: rad
    #[serde(default)]
    orientation_offset: f64,
}

#[derive(Debug)]
pub struct Actual {
    /// angular position, unit: rad
    pub p_ang: f64,
    /// angular velocity, unit: rad/s
    pub v_ang: f64,
    /// tangential velocity, unit: m/s
    pub v_tan: f64,
    /// centripedal acceleration, unit: m/s^2
    pub ac: f64,
}

#[derive(Clone)]
pub struct EomFns {
    /// unit: m/s^2
    radius: f64,
}

impl EomFns {
    pub fn new(cfg: &Config) -> Self {
        Self { radius: cfg.radius }
    }

    pub fn from_radius(radius: f64) -> Self {
        Self { radius: radius }
    }
}

impl eom::traits::ModelSpec for EomFns {
    type Scalar = f64;
    type Dim = ndarray::Ix1;

    fn model_size(&self) -> usize {
        2
    }
}

impl eom::traits::Explicit for EomFns {
    fn rhs<'a, S>(
        &mut self,
        v: &'a mut ndarray::ArrayBase<S, ndarray::Ix1>,
    ) -> &'a mut ndarray::ArrayBase<S, ndarray::Ix1>
    where
        S: ndarray::DataMut<Elem = f64>,
    {
        let theta = v[0];
        let x = v[1];
        v[0] = x;
        v[1] = -(math::GRAVITY / self.radius) * theta.sin();
        v
    }
}

fn build_sample<S>(cfg: &Config, t: f64, data: &ndarray::ArrayBase<S, ndarray::Ix1>) -> Data
where
    S: ndarray::Data<Elem = f64>,
{
    let t_us = (t * 1_000_000.0) as u64;
    let p_ang = data[0];
    let v_ang = data[1];
    let v_tan = v_ang * cfg.radius;
    let ac = v_ang.powi(2) * cfg.radius;

    let actual = Actual {
        p_ang,
        v_ang,
        v_tan,
        ac,
    };

    let mut sample = Data::default();
    sample.time = t_us;
    sample.time_baro = t_us;
    sample.accel = array![
        0.0,
        0.0,
        ac + math::GRAVITY * (p_ang + cfg.orientation_offset).cos()
    ];
    sample.gyro = array![v_ang, 0.0, 0.0];
    sample.actual = Some(Box::new(data::ActualData::Pendulum(actual)));

    sample
}

pub fn generate(cfg: &Config) -> Result<Vec<Data>, Error> {
    let mut ret = Vec::new();

    let mut teo = eom::explicit::RK4::new(EomFns::new(cfg), cfg.dt);
    let ts = eom::adaptor::time_series(ndarray::arr1(&[cfg.initial_angle, 0.0]), &mut teo);
    let nsamples = (cfg.duration / cfg.dt) as usize;

    ret.push(build_sample(cfg, 0.0, &array![cfg.initial_angle, 0.0]));
    for (_t, v) in ts.take(nsamples).enumerate() {
        let t = _t as f64 * cfg.dt;
        ret.push(build_sample(cfg, t, &v));
    }

    Ok(ret)
}

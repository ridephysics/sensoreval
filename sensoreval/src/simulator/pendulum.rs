use crate::*;
use eom::traits::Scheme;
use eom::traits::TimeEvolution;
use ndarray::array;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Config {
    /// unit: seconds
    start_off: f64,
    /// unit: meters
    radius: f64,
    /// unit: rad
    ship_arc_half_angle: f64,
    /// unit: seconds
    dt: f64,
    /// unit: seconds
    duration: f64,

    /// unit: rad
    orientation_offset: f64,
    /// unit: rad
    rot: Vec<f64>,

    initial: Vec<f64>,
    control_input: Vec<Vec<f64>>,
}

#[derive(Clone)]
pub struct EomFns<'c> {
    radius: f64,
    ship_arc_half_angle: f64,
    control_input: Option<&'c Vec<f64>>,
}

impl<'c> EomFns<'c> {
    pub fn new(cfg: &Config) -> Self {
        Self {
            radius: cfg.radius,
            control_input: None,
            ship_arc_half_angle: cfg.ship_arc_half_angle,
        }
    }

    pub fn from_radius(radius: f64) -> Self {
        Self {
            radius,
            control_input: None,
            ship_arc_half_angle: 0.0,
        }
    }

    pub fn set_control_input(&mut self, control_input: Option<&'c Vec<f64>>) {
        self.control_input = control_input;
    }
}

impl<'c> eom::traits::ModelSpec for EomFns<'c> {
    type Scalar = f64;
    type Dim = ndarray::Ix1;

    fn model_size(&self) -> usize {
        2
    }
}

impl<'c> eom::traits::Explicit for EomFns<'c> {
    fn rhs<'a, S>(
        &mut self,
        v: &'a mut ndarray::ArrayBase<S, ndarray::Ix1>,
    ) -> &'a mut ndarray::ArrayBase<S, ndarray::Ix1>
    where
        S: ndarray::DataMut<Elem = f64>,
    {
        let theta = v[0];
        let x = v[1];
        let mut motor = 0.0;

        if let Some(control_input) = self.control_input {
            if theta.abs() <= self.ship_arc_half_angle {
                motor = control_input[1];

                // accelerate into the direction of movement
                if x < 0.0 {
                    motor = -motor;
                };
            }
        }

        v[0] = x;
        v[1] = (-math::GRAVITY * theta.sin() + motor) / self.radius;
        v
    }
}

fn build_sample<S>(cfg: &Config, t_us: u64, data: &ndarray::ArrayBase<S, ndarray::Ix1>) -> Data
where
    S: ndarray::Data<Elem = f64>,
{
    let pa = math::normalize_angle(data[0]);
    let va = data[1];
    let r = cfg.radius;
    let oo = cfg.orientation_offset;
    let re = cfg.rot[0];
    let rn = cfg.rot[1];
    let ru = cfg.rot[2];

    let ac = va.powi(2) * r;

    let accel = nalgebra::Vector3::new(0.0, 0.0, ac + math::GRAVITY * (pa + oo).cos());
    let gyro = nalgebra::Vector3::new(va, 0.0, 0.0);

    let axis_east = nalgebra::Unit::new_normalize(nalgebra::Vector3::new(1.0, 0.0, 0.0));
    let q = nalgebra::UnitQuaternion::from_axis_angle(&axis_east, re);
    let accel = q * accel;
    let gyro = q * gyro;

    let axis_north = nalgebra::Unit::new_normalize(nalgebra::Vector3::new(0.0, 1.0, 0.0));
    let q = nalgebra::UnitQuaternion::from_axis_angle(&axis_north, rn);
    let accel = q * accel;
    let gyro = q * gyro;

    let axis_up = nalgebra::Unit::new_normalize(nalgebra::Vector3::new(0.0, 0.0, 1.0));
    let q = nalgebra::UnitQuaternion::from_axis_angle(&axis_up, ru);
    let accel = q * accel;
    let gyro = q * gyro;

    let mut sample = Data::default();
    sample.time = t_us;
    sample.time_baro = t_us;
    sample.accel = array![accel[0], accel[1], accel[2]];
    sample.gyro = array![gyro[0], gyro[1], gyro[2]];
    sample.actual = Some(array![pa, va, r, oo, re, rn, ru]);

    sample
}

fn next_control_input_time(cfg: &Config, id: usize) -> Option<f64> {
    if id >= cfg.control_input.len() {
        None
    } else {
        Some(cfg.control_input[id][0])
    }
}

pub fn generate(cfg: &Config) -> Result<Vec<Data>, Error> {
    let nsamples = (cfg.duration / cfg.dt) as usize;
    let mut ret = Vec::new();
    let mut teo = eom::explicit::RK4::new(EomFns::new(cfg), cfg.dt);

    let mut x = ndarray::Array::from(cfg.initial.clone());
    let mut ciid = 0;
    let mut next_input_time = next_control_input_time(cfg, ciid);

    for id in 0..nsamples {
        let t = id as f64 * cfg.dt + cfg.start_off;
        let t_us = (t * 1_000_000.0) as u64;

        ret.push(build_sample(cfg, t_us, &x));

        if let Some(nit) = next_input_time {
            if t + cfg.dt >= nit {
                teo.core_mut()
                    .set_control_input(Some(&cfg.control_input[ciid]));

                ciid += 1;
                next_input_time = next_control_input_time(cfg, ciid);
            }
        }

        teo.iterate(&mut x);
    }

    Ok(ret)
}

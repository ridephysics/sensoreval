use eom::traits::Scheme;
use eom::traits::TimeEvolution;
use eom::traits::TimeStep;
use std::convert::TryInto;

#[derive(Clone, serde::Deserialize, Debug)]
pub struct GroundMotor {
    /// defines how long half the ship is. This is used to determine at which
    /// angle the motor starts contacting the ship. unit: rad
    pub ship_arc_half_angle: f64,
}

#[derive(Clone, serde::Deserialize, Debug)]
#[serde(tag = "type")]
pub enum Motor {
    #[serde(rename = "ground")]
    GroundMotor(GroundMotor),
}

#[derive(Clone, serde::Deserialize, Debug)]
pub struct Params {
    pub radius: f64,
    #[serde(default)]
    pub motor: Option<Motor>,

    /// position of the sensor relative to the center of mass
    #[serde(default)]
    pub sensor_pos: f64,
}

#[derive(Clone, Debug)]
struct ParamsInternal {
    params: Params,
    ci: Option<[f64; 1]>,
}

impl eom::traits::ModelSpec for ParamsInternal {
    type Scalar = f64;
    type Dim = ndarray::Ix1;

    fn model_size(&self) -> usize {
        2
    }
}

impl eom::traits::Explicit for ParamsInternal {
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
        v[1] = (-math::GRAVITY * theta.sin()) / self.params.radius;

        if let (Some(ci), Some(motor)) = (&self.ci, &self.params.motor) {
            match motor {
                Motor::GroundMotor(m) => {
                    if theta.abs() <= m.ship_arc_half_angle {
                        // accelerate into the direction of movement
                        let motor = if x.is_sign_negative() { -ci[0] } else { ci[0] };
                        v[1] += motor / self.params.radius;
                    }
                }
            }
        }

        v
    }
}

pub struct Pendulum {
    eom: eom::explicit::RK4<ParamsInternal>,
}

impl Pendulum {
    pub fn new(params: Params, dt: f64) -> Self {
        Self {
            eom: eom::explicit::RK4::new(ParamsInternal { params, ci: None }, dt),
        }
    }
}

impl crate::Model for Pendulum {
    impl_model_inner!(eom);

    fn set_control_input(&mut self, ci: Option<&[f64]>) {
        self.eom.core_mut().ci = ci.map(|x| x.try_into().unwrap());
    }
}

impl crate::ToImuSample for Pendulum {
    fn to_accel<Sa, Sb>(
        &self,
        state: &ndarray::ArrayBase<Sa, ndarray::Ix1>,
        accel: &mut ndarray::ArrayBase<Sb, ndarray::Ix1>,
    ) where
        Sa: ndarray::Data<Elem = f64>,
        Sb: ndarray::DataMut<Elem = f64>,
    {
        let params = &self.eom.core().params;
        let ac = state[1].powi(2) * params.radius;

        accel.assign(&ndarray::array![
            0.0,
            0.0,
            ac + math::GRAVITY * (state[0] + params.sensor_pos).cos()
        ]);
    }

    fn to_gyro<Sa, Sb>(
        &self,
        state: &ndarray::ArrayBase<Sa, ndarray::Ix1>,
        gyro: &mut ndarray::ArrayBase<Sb, ndarray::Ix1>,
    ) where
        Sa: ndarray::Data<Elem = f64>,
        Sb: ndarray::DataMut<Elem = f64>,
    {
        gyro.assign(&ndarray::array![state[1], 0.0, 0.0]);
    }
}

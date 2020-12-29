use eom::traits::Scheme;
use eom::traits::TimeEvolution;
use eom::traits::TimeStep;

#[derive(sensoreval_psim_macros::State)]
pub enum State {
    ThetaB,
    ThetaBD,
    Theta0,
    Theta0D,
}

#[derive(Clone, serde::Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Object {
    /// radius in meters
    pub r: f64,
    /// angle `theta` in radians
    pub t: f64,
    /// mass in kg
    pub m: f64,
}

impl std::str::FromStr for Object {
    type Err = std::num::ParseFloatError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let o: Vec<&str> = s.split(',').collect();

        let r_fromstr = o[0].parse::<f64>()?;
        let t_fromstr = o[1].parse::<f64>()?;
        let m_fromstr = o[2].parse::<f64>()?;

        Ok(Self {
            r: r_fromstr,
            t: t_fromstr,
            m: m_fromstr,
        })
    }
}

#[derive(Clone, serde::Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Params {
    /// list of masses connected to the booster
    pub objects: Vec<Object>,
    /// booster radius
    pub rb: f64,
    /// sensor angle relative to theta0
    pub thetas: f64,
    /// sensor radius
    pub rs: f64,
    /// friction between booster and gondola
    pub friction: Option<f64>,
}

impl Params {
    pub fn theta0dd<S>(&self, state: &ndarray::ArrayBase<S, ndarray::Ix1>) -> f64
    where
        S: ndarray::Data<Elem = f64>,
    {
        let thetab = state[State::ThetaB];
        let thetabd = state[State::ThetaBD];
        let theta0 = state[State::Theta0];
        let theta0d = state[State::Theta0D];

        // single-mass optimization which attempts to reduce inaccuracies
        let mut theta0dd = if self.objects.len() == 1 {
            let thetac = self.objects[0].t;
            let rc = self.objects[0].r;

            (self.rb * thetabd.powi(2) * (thetab - theta0 - thetac).sin()
                - math::GRAVITY * (theta0 + thetac).sin())
                / rc
        } else {
            (self.rb
                * thetabd.powi(2)
                * self
                    .objects
                    .iter()
                    .map(|o| o.m * o.r * (thetab - theta0 - o.t).sin())
                    .sum::<f64>()
                - math::GRAVITY
                    * self
                        .objects
                        .iter()
                        .map(|o| o.m * o.r * (theta0 + o.t).sin())
                        .sum::<f64>())
                / self.objects.iter().map(|o| o.m * o.r.powi(2)).sum::<f64>()
        };

        if let Some(friction) = self.friction {
            theta0dd -= friction * (theta0d - thetabd);
        }

        theta0dd
    }
}

impl eom::traits::ModelSpec for Params {
    type Scalar = f64;
    type Dim = ndarray::Ix1;

    fn model_size(&self) -> usize {
        State::len()
    }
}

impl eom::traits::Explicit for Params {
    fn rhs<'a, S>(
        &mut self,
        v: &'a mut ndarray::ArrayBase<S, ndarray::Ix1>,
    ) -> &'a mut ndarray::ArrayBase<S, ndarray::Ix1>
    where
        S: ndarray::DataMut<Elem = f64>,
    {
        let thetabd = v[State::ThetaBD];
        let theta0d = v[State::Theta0D];
        let theta0dd = self.theta0dd(v);

        v[State::ThetaB] = thetabd;
        v[State::ThetaBD] = 0.0;

        v[State::Theta0] = theta0d;
        v[State::Theta0D] = theta0dd;

        v
    }
}

#[derive(Clone)]
pub struct Booster {
    eom: eom::explicit::RK4<Params>,
}

impl Booster {
    pub fn new(params: Params, dt: f64) -> Self {
        Self {
            eom: eom::explicit::RK4::new(params, dt),
        }
    }

    pub fn params(&self) -> &Params {
        self.eom.core()
    }

    /// returns the center of mass relative to theta0
    pub fn thetac(&self) -> f64 {
        let params = self.eom.core();
        let mc: f64 = params.objects.iter().map(|o| o.m).sum();
        let xc = params
            .objects
            .iter()
            .map(|o| o.m * math::rt2x(o.r, o.t))
            .sum::<f64>()
            / mc;
        let yc = params
            .objects
            .iter()
            .map(|o| o.m * math::rt2y(o.r, o.t))
            .sum::<f64>()
            / mc;

        math::xy2t(xc, yc)
    }

    /// returns a single-mass radius that behaves the same as the current
    /// (multi-mass) booster
    pub fn rc(&self) -> f64 {
        let params = self.eom.core();

        let tmp = params
            .objects
            .iter()
            .map(|o| o.m * o.r.powi(2))
            .sum::<f64>();
        let tmp1 = params
            .objects
            .iter()
            .map(|o| o.m * math::rt2x(o.r, o.t))
            .sum::<f64>()
            .powi(2);
        let tmp2 = params
            .objects
            .iter()
            .map(|o| o.m * math::rt2y(o.r, o.t))
            .sum::<f64>()
            .powi(2);

        tmp / (tmp1 + tmp2).sqrt()
    }
}

impl crate::Model for Booster {
    impl_model_inner!(eom);

    fn normalize<S>(&self, x: &mut ndarray::ArrayBase<S, ndarray::Ix1>)
    where
        S: ndarray::DataMut<Elem = f64>,
    {
        x[State::ThetaB] = math::normalize_angle(x[State::ThetaB]);
        x[State::Theta0] = math::normalize_angle(x[State::Theta0]);
    }
}

impl crate::ToImuSample for Booster {
    fn to_accel<Sa, Sb>(
        &self,
        state: &ndarray::ArrayBase<Sa, ndarray::Ix1>,
        accel: &mut ndarray::ArrayBase<Sb, ndarray::Ix1>,
    ) where
        Sa: ndarray::Data<Elem = f64>,
        Sb: ndarray::DataMut<Elem = f64>,
    {
        let params = self.eom.core();
        let thetab = state[State::ThetaB];
        let thetabd = state[State::ThetaBD];
        let theta0 = state[State::Theta0];
        let theta0d = state[State::Theta0D];
        let theta0dd = params.theta0dd(state);

        accel.assign(&ndarray::array![
            0.0,
            -params.rb * thetabd.powi(2) * (thetab - theta0).sin()
                - params.rs * theta0d.powi(2) * params.thetas.sin()
                + params.rs * theta0dd * params.thetas.cos()
                + math::GRAVITY * theta0.sin(),
            params.rb * thetabd.powi(2) * (thetab - theta0).cos()
                + params.rs * theta0d.powi(2) * params.thetas.cos()
                + params.rs * theta0dd * params.thetas.sin()
                + math::GRAVITY * theta0.cos()
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
        let theta0d = state[State::Theta0D];
        gyro.assign(&ndarray::array![theta0d, 0.0, 0.0]);
    }

    fn to_height<S>(&self, state: &ndarray::ArrayBase<S, ndarray::Ix1>) -> f64
    where
        S: ndarray::Data<Elem = f64>,
    {
        let params = self.eom.core();
        let thetab = state[State::ThetaB];
        let theta0 = state[State::Theta0];

        -params.rb * thetab.cos() - params.rs * (theta0 + params.thetas).cos()
            + params.rb
            + params.rs
    }
}

impl crate::DrawState for Booster {
    fn draw_state<S>(&self, cr: &cairo::Context, state: &ndarray::ArrayBase<S, ndarray::Ix1>)
    where
        S: ndarray::DataMut<Elem = f64>,
    {
        sensoreval_graphics::booster_2d::draw(cr, state[State::Theta0], 0.0, state[State::ThetaB]);
    }
}

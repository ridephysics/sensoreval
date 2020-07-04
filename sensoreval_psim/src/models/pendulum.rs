use eom::traits::Scheme;
use eom::traits::TimeEvolution;
use eom::traits::TimeStep;

#[derive(Clone)]
struct Params {
    radius: f64,
}

impl eom::traits::ModelSpec for Params {
    type Scalar = f64;
    type Dim = ndarray::Ix1;

    fn model_size(&self) -> usize {
        2
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
        let theta = v[0];
        let x = v[1];

        v[0] = x;
        v[1] = (-math::GRAVITY * theta.sin()) / self.radius;
        v
    }
}

pub struct Pendulum {
    eom: eom::explicit::RK4<Params>,
}

impl Pendulum {
    pub fn new(radius: f64, dt: f64) -> Self {
        Self {
            eom: eom::explicit::RK4::new(Params { radius }, dt),
        }
    }
}

impl_model!(Pendulum, eom);

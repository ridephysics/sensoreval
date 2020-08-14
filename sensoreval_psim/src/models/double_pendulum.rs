use eom::traits::Scheme;
use eom::traits::TimeEvolution;
use eom::traits::TimeStep;
use ndarray_linalg::solve::Inverse;

#[derive(Clone)]
struct Params {
    m1: f64,
    m2: f64,
    l1: f64,
    l2: f64,
}

impl eom::traits::ModelSpec for Params {
    type Scalar = f64;
    type Dim = ndarray::Ix1;

    fn model_size(&self) -> usize {
        4
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
        let a1d = v[0];
        let a2d = v[1];
        let a1 = v[2];
        let a2 = v[3];

        let m11 = (self.m1 + self.m2) * self.l1;
        let m12 = self.m2 * self.l2 * (a1 - a2).cos();
        let m21 = self.l1 * (a1 - a2).cos();
        let m22 = self.l2;
        let m = ndarray::array![[m11, m12], [m21, m22]];

        let f1 = -self.m2 * self.l2 * a2d * a2d * (a1 - a2).sin()
            - (self.m1 + self.m2) * math::GRAVITY * a1.sin();
        let f2 = self.l1 * a1d * a1d * (a1 - a2).sin() - math::GRAVITY * a2.sin();
        let f = ndarray::array![f1, f2];

        let accel = m.inv().unwrap().dot(&f);

        v[0] = accel[0];
        v[1] = accel[1];
        v[2] = a1d;
        v[3] = a2d;

        v
    }
}

pub struct DoublePendulum {
    eom: eom::explicit::RK4<Params>,
}

impl DoublePendulum {
    pub fn new(m1: f64, m2: f64, l1: f64, l2: f64, dt: f64) -> Self {
        Self {
            eom: eom::explicit::RK4::new(Params { m1, m2, l1, l2 }, dt),
        }
    }
}

impl_model!(DoublePendulum, eom);

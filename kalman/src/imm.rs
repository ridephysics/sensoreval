#![allow(non_snake_case)]
use std::ops::DivAssign;

#[derive(Debug)]
pub struct IMM<'a, FNS, A, FR, F: ?Sized, MU, M, Sz> {
    fns: FNS,

    /// List of N filters. filters[i] is the ith Kalman filter in the IMM
    /// estimator
    filters: &'a mut [FR],

    /// mode probability: mu[i] is the probability that filter i is the correct
    /// one
    mu: &'a mut MU,

    /// Markov chain transition matrix. M[i,j] is the probability of switching
    /// from filter j to filter i
    M: &'a M,

    /// Current state estimate. Any call to update() or predict() updates this
    /// variable
    x: ndarray::Array1<A>,

    /// Current state covariance matrix. Any call to update() or predict()
    /// updates this variable
    P: ndarray::Array2<A>,

    /// Likelihood of each individual filter's last measurement
    likelihood: ndarray::Array1<A>,

    /// Mixing probabilitity - omega[i, j] is the probabilility of mixing the
    /// state of filter i into filter j. Perhaps more understandably, it weights
    /// the states of each filter by:
    ///     x_j = sum(omega[i,j] * x_i)
    /// with a similar weighting for P_j
    omega: ndarray::Array2<A>,

    /// Total probability, after interaction, that the target is in state j.
    /// We use it as the # normalization constant.
    cbar: ndarray::Array1<A>,

    pd_Sz: std::marker::PhantomData<Sz>,
    pd_F: std::marker::PhantomData<F>,
}

impl<'a, FNS, A, FR, F, MU, M, Sz, T> crate::SetDt<T> for IMM<'a, FNS, A, FR, F, MU, M, Sz>
where
    FR: AsMut<F>,
    F: crate::SetDt<T> + ?Sized,
{
    fn set_dt(&mut self, dt: &T) {
        for filter in &mut *self.filters {
            filter.as_mut().set_dt(dt);
        }
    }
}

impl<'a, FNS, A, FR, F, Smu, SM, Sz> crate::Filter
    for IMM<
        'a,
        FNS,
        A,
        FR,
        F,
        ndarray::ArrayBase<Smu, ndarray::Ix1>,
        ndarray::ArrayBase<SM, ndarray::Ix2>,
        Sz,
    >
where
    FNS: crate::Add<A> + crate::Subtract<A>,
    FR: std::borrow::Borrow<F> + std::borrow::BorrowMut<F>,
    F: crate::Filter<Elem = A, Meas = ndarray::ArrayBase<Sz, ndarray::Ix1>> + ?Sized,
    Smu: ndarray::DataMut<Elem = A>,
    SM: ndarray::Data<Elem = A>,
    A: num_traits::float::Float
        + ndarray::ScalarOperand
        + std::ops::DivAssign
        + std::ops::AddAssign
        + std::convert::From<f32>
        + std::fmt::Display,
    Sz: ndarray::Data<Elem = A>,
{
    type Elem = A;
    type Meas = ndarray::ArrayBase<Sz, ndarray::Ix1>;

    fn predict(&mut self) -> Result<(), crate::Error> {
        let N = self.filters.len();

        // compute mixed initial conditions
        let mut xs = Vec::with_capacity(N);
        let mut Ps = Vec::with_capacity(N);
        for w in self.omega.t().genrows() {
            let mut x = ndarray::Array1::<A>::zeros(self.x.dim());
            ndarray::azip!((kf in &*self.filters, &wj in &w) {
                x = self.fns.add(&x, &(kf.borrow().x() * wj));
            });

            let mut P = ndarray::Array2::<A>::zeros(self.P.dim());
            ndarray::azip!((kf in &*self.filters, &wj in &w) {
                let kf = kf.borrow();

                let y = self.fns.subtract(kf.x(), &x);
                P += &((math::outer_product(&y, &y) + kf.P()) * wj);
            });

            xs.push(x);
            Ps.push(P);
        }

        // compute each filter's prior using the mixed initial conditions
        for (i, f) in self.filters.iter_mut().enumerate() {
            let f = f.borrow_mut();

            // propagate using the mixed state estimate and covariance
            f.x_mut().assign(&xs[i]);
            f.P_mut().assign(&Ps[i]);
            f.predict()?;
        }

        self.compute_state_estimate();

        Ok(())
    }

    fn update(&mut self, z: &ndarray::ArrayBase<Sz, ndarray::Ix1>) -> Result<(), crate::Error> {
        // run update on each filter, and save the likelihood
        for (i, f) in self.filters.iter_mut().enumerate() {
            let f = f.borrow_mut();

            f.update(z)?;
            self.likelihood[i] = f.likelihood()?;
            println!("[{}] LH={}", i, self.likelihood[i]);
        }

        // update mode probabilities from total probability * likelihood
        self.mu.assign(&(&self.cbar * &self.likelihood));
        // normalize
        self.mu.div_assign(self.mu.sum());

        self.compute_mixing_probabilities();
        self.compute_state_estimate();

        Ok(())
    }

    fn likelihood(&self) -> Result<A, crate::Error> {
        Err(crate::Error::InvalidArgument)
    }

    fn x(&self) -> &ndarray::Array1<A> {
        &self.x
    }

    fn P(&self) -> &ndarray::Array2<A> {
        &self.P
    }

    fn x_mut(&mut self) -> &mut ndarray::Array1<A> {
        &mut self.x
    }

    fn P_mut(&mut self) -> &mut ndarray::Array2<A> {
        &mut self.P
    }
}

impl<'a, FNS, A, FR, F, Smu, SM, Sz>
    IMM<
        'a,
        FNS,
        A,
        FR,
        F,
        ndarray::ArrayBase<Smu, ndarray::Ix1>,
        ndarray::ArrayBase<SM, ndarray::Ix2>,
        Sz,
    >
where
    FNS: crate::Add<A> + crate::Subtract<A>,
    FR: std::borrow::Borrow<F> + std::borrow::BorrowMut<F>,
    F: crate::Filter<Elem = A, Meas = ndarray::ArrayBase<Sz, ndarray::Ix1>> + ?Sized,
    Smu: ndarray::DataMut<Elem = A>,
    SM: ndarray::Data<Elem = A>,
    A: num_traits::float::Float
        + ndarray::ScalarOperand
        + std::ops::AddAssign
        + std::convert::From<f32>,
    Sz: ndarray::Data<Elem = A>,
{
    pub fn new(
        filters: &'a mut [FR],
        mu: &'a mut ndarray::ArrayBase<Smu, ndarray::Ix1>,
        M: &'a ndarray::ArrayBase<SM, ndarray::Ix2>,
        fns: FNS,
    ) -> Result<Self, crate::Error> {
        if filters.len() < 2 {
            return Err(crate::Error::NotEnoughFilters);
        }

        let x_dim = filters[0].borrow().x().dim();
        for f in &*filters {
            if f.borrow().x().dim() != x_dim {
                return Err(crate::Error::DifferentFilterShapes);
            }
        }

        let N = filters.len();
        let P_dim = filters[0].borrow().P().dim();
        let mut o = Self {
            fns,
            filters,
            mu,
            M,
            x: ndarray::Array::zeros(x_dim),
            P: ndarray::Array::zeros(P_dim),
            likelihood: ndarray::Array::zeros(N),
            omega: ndarray::Array::zeros((N, N)),
            cbar: ndarray::Array::zeros(N),
            pd_Sz: std::marker::PhantomData,
            pd_F: std::marker::PhantomData,
        };
        o.compute_mixing_probabilities();

        // initialize imm state estimate based on current filters
        o.compute_state_estimate();

        Ok(o)
    }

    /// Computes the IMM's mixed state estimate from each filter using
    /// the the mode probability self.mu to weight the estimates
    fn compute_state_estimate(&mut self) {
        let mut x = ndarray::Array::zeros(self.x.dim());
        ndarray::azip!((f in &*self.filters, mu in &*self.mu) {
            x = self.fns.add(&x, &(f.borrow().x() * *mu));
        });
        self.x = x;

        let mut P = ndarray::Array::zeros(self.P.dim());
        ndarray::azip!((f in &*self.filters, mu in &*self.mu) {
            let f = f.borrow();

            let y = self.fns.subtract(f.x(), &self.x);
            P += &((math::outer_product(&y, &y) + f.P()) * *mu);
        });
        self.P = P;
    }

    /// Compute the mixing probability for each filter
    fn compute_mixing_probabilities(&mut self) {
        self.cbar = self.mu.dot(self.M);
        let N = self.filters.len();
        for i in 0..N {
            for j in 0..N {
                self.omega[(i, j)] = (self.M[(i, j)] * self.mu[i]) / self.cbar[j];
            }
        }
    }

    pub fn filters(&mut self) -> &[FR] {
        self.filters
    }

    pub fn filters_mut(&mut self) -> &mut [FR] {
        self.filters
    }

    pub fn mu(&self) -> &ndarray::ArrayBase<Smu, ndarray::Ix1> {
        self.mu
    }
}

#[cfg(test)]
mod test {
    use crate::Filter;
    use rand::Rng;

    /// simulate a moving target
    fn turning_target(N: usize, turn_start: usize) -> ndarray::Array3<f64> {
        let dt = 1.0f64;
        let phi_sim = ndarray::array![
            [1.0, dt, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, dt],
            [0.0, 0.0, 0.0, 1.0]
        ];

        let gam = ndarray::array![
            [dt.powi(2) / 2.0, 0.0],
            [dt, 0.0],
            [0.0, dt.powi(2) / 2.0],
            [0.0, dt]
        ];

        let mut x = ndarray::array![[2000.0, 0.0, 10000.0, -15.0]]
            .t()
            .to_owned();

        let mut simxs = ndarray::Array3::zeros((N, x.dim().0, x.dim().1));

        for i in 0..N {
            x = phi_sim.dot(&x);
            if i >= turn_start {
                x += &gam.dot(&ndarray::array![[0.075, 0.075]].t());
            }
            simxs.index_axis_mut(ndarray::Axis(0), i).assign(&x);
        }

        simxs
    }

    /// this is the sample from Roger Labbes book
    #[test]
    fn imm() {
        let mut rng = rand::thread_rng();

        let N = 600;
        let dt = 1.0f64;
        let imm_track = turning_target(N, 400);

        // create noisy measurements
        let mut zs = ndarray::Array2::<f64>::zeros((N, 2));
        let r = 1.0;
        for i in 0..N {
            zs[(i, 0)] = imm_track[(i, 0, 0)] + rng.gen::<f64>() * r;
            zs[(i, 1)] = imm_track[(i, 2, 0)] + rng.gen::<f64>() * r;
        }

        let mut ca = super::super::kalman::Kalman::<f64, _>::new(6, 2).unwrap();
        let dt2 = dt.powi(2) / 2.0f64;
        let F = ndarray::array![[1.0, dt, dt2], [0.0, 1.0, dt], [0.0, 0.0, 1.0]];

        ca.F = ndarray::Array::zeros((F.dim().0 * 2, F.dim().1 * 2));
        ca.F.slice_mut(ndarray::s![0..F.dim().0, 0..F.dim().1])
            .assign(&F);
        ca.F.slice_mut(ndarray::s![F.dim().0.., F.dim().1..])
            .assign(&F);

        ca.x = ndarray::array![2000.0f64, 0.0, 0.0, 10000.0, -15.0, 0.0]
            .t()
            .to_owned();
        ca.P *= 1.0e-12;
        ca.R *= r.powi(2);

        let q = ndarray::array![
            [0.05, 0.125, 1.0 / 6.0],
            [0.125, 1.0 / 3.0, 0.5],
            [1.0 / 6.0, 0.5, 1.0]
        ] * 1.0e-3;
        ca.Q = ndarray::Array::zeros((q.dim().0 * 2, q.dim().1 * 2));
        ca.Q.slice_mut(ndarray::s![0..q.dim().0, 0..q.dim().1])
            .assign(&q);
        ca.Q.slice_mut(ndarray::s![q.dim().0.., q.dim().1..])
            .assign(&q);

        ca.H = ndarray::array![
            [1.0, 0.0, 0.0, 0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0, 1.0, 0.0, 0.0]
        ];

        // create identical filter, but with no process error
        let mut cano = ca.clone();
        cano.Q *= 0.0;

        let mut filters = vec![ca, cano];
        let M = ndarray::array![[0.97, 0.03], [0.03, 0.97]];
        let mut mu = ndarray::array![0.5, 0.5];
        let fns = super::super::sigma_points::tests::LinFns::default();
        let mut bank = super::IMM::new(&mut filters, &mut mu, &M, fns).unwrap();

        let mut xs = Vec::with_capacity(zs.len());
        let mut probs = Vec::with_capacity(zs.len());
        for z in zs.genrows().into_iter() {
            bank.predict().unwrap();
            bank.update(&z).unwrap();

            xs.push(bank.x().clone());
            probs.push(bank.mu().clone());

            /*
            eprintln!(
                "x={} actual={}",
                bank.x(),
                imm_track.index_axis(ndarray::Axis(0), i)
            );*/
        }
    }
}

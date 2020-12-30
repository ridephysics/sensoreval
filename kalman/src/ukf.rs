#![allow(non_snake_case)]

use crate::Error;
use ndarray::azip;
use ndarray_linalg::solve::Inverse;

type RTSResult<A> = (Vec<ndarray::Array1<A>>, Vec<ndarray::Array2<A>>);

pub trait Functions {
    type Elem;

    fn fx<S>(
        &self,
        x: &ndarray::ArrayBase<S, ndarray::Ix1>,
        dt: Self::Elem,
    ) -> ndarray::Array1<Self::Elem>
    where
        S: ndarray::Data<Elem = Self::Elem>;

    fn x_mean<Ss, Swm>(
        &self,
        sigmas: &ndarray::ArrayBase<Ss, ndarray::Ix2>,
        Wm: &ndarray::ArrayBase<Swm, ndarray::Ix1>,
    ) -> ndarray::Array1<Self::Elem>
    where
        Ss: ndarray::Data<Elem = Self::Elem>,
        Swm: ndarray::Data<Elem = Self::Elem>;

    fn x_residual<Sa, Sb>(
        &self,
        a: &ndarray::ArrayBase<Sa, ndarray::Ix1>,
        b: &ndarray::ArrayBase<Sb, ndarray::Ix1>,
    ) -> ndarray::Array1<Self::Elem>
    where
        Sa: ndarray::Data<Elem = Self::Elem>,
        Sb: ndarray::Data<Elem = Self::Elem>;

    fn x_add<Sa, Sb>(
        &self,
        a: &ndarray::ArrayBase<Sa, ndarray::Ix1>,
        b: &ndarray::ArrayBase<Sb, ndarray::Ix1>,
    ) -> ndarray::Array1<Self::Elem>
    where
        Sa: ndarray::Data<Elem = Self::Elem>,
        Sb: ndarray::Data<Elem = Self::Elem>;

    fn hx<S>(&self, x: &ndarray::ArrayBase<S, ndarray::Ix1>) -> ndarray::Array1<Self::Elem>
    where
        S: ndarray::Data<Elem = Self::Elem>;

    fn z_mean<Ss, Swm>(
        &self,
        sigmas: &ndarray::ArrayBase<Ss, ndarray::Ix2>,
        Wm: &ndarray::ArrayBase<Swm, ndarray::Ix1>,
    ) -> ndarray::Array1<Self::Elem>
    where
        Ss: ndarray::Data<Elem = Self::Elem>,
        Swm: ndarray::Data<Elem = Self::Elem>;

    fn z_residual<Sa, Sb>(
        &self,
        a: &ndarray::ArrayBase<Sa, ndarray::Ix1>,
        b: &ndarray::ArrayBase<Sb, ndarray::Ix1>,
    ) -> ndarray::Array1<Self::Elem>
    where
        Sa: ndarray::Data<Elem = Self::Elem>,
        Sb: ndarray::Data<Elem = Self::Elem>;
}

pub struct UKF<'a, FP, FNS, A> {
    fns: &'a FNS,

    // state
    pub x: ndarray::Array1<A>,
    pub P: ndarray::Array2<A>,
    pub Q: ndarray::Array2<A>,

    // observation
    z: ndarray::Array1<A>,
    pub R: ndarray::Array2<A>,

    // sigma points
    points_fn: &'a FP,
    pub Wc: ndarray::Array1<A>,
    pub Wm: ndarray::Array1<A>,

    // predict
    sigmas_f: ndarray::Array2<A>,

    // update
    sigmas_h: ndarray::Array2<A>,
    pub y: ndarray::Array1<A>,
    pub S: ndarray::Array2<A>,
}

impl<'a, FP, FNS, A> UKF<'a, FP, FNS, A>
where
    FP: crate::sigma_points::SigmaPoints<Elem = A>,
    FNS: Functions<Elem = A>,
    A: Copy
        + num_traits::float::FloatConst
        + num_traits::float::Float
        + ndarray::ScalarOperand
        + ndarray_linalg::types::Lapack
        + std::convert::From<f32>,
    <A as ndarray_linalg::Scalar>::Real: std::convert::From<f32>,
{
    pub fn new(dim_x: usize, dim_z: usize, points_fn: &'a FP, fns: &'a FNS) -> Self {
        Self {
            fns,

            x: ndarray::Array::zeros(dim_x),
            P: ndarray::Array::ones((dim_x, dim_x)),
            Q: ndarray::Array::eye(dim_x),

            z: ndarray::Array::zeros(dim_z),
            R: ndarray::Array::eye(dim_z),

            points_fn,
            Wc: points_fn.weights_covariance(),
            Wm: points_fn.weights_mean(),

            sigmas_f: ndarray::Array::zeros((points_fn.num_sigmas(), dim_x)),
            sigmas_h: ndarray::Array::zeros((points_fn.num_sigmas(), dim_z)),
            y: ndarray::Array::zeros(dim_z),
            S: ndarray::Array::zeros((dim_z, dim_z)),
        }
    }

    fn cross_variance<Sx, Sz, Sf, Sh>(
        &self,
        x: &ndarray::ArrayBase<Sx, ndarray::Ix1>,
        z: &ndarray::ArrayBase<Sz, ndarray::Ix1>,
        sigmas_f: &ndarray::ArrayBase<Sf, ndarray::Ix2>,
        sigmas_h: &ndarray::ArrayBase<Sh, ndarray::Ix2>,
    ) -> ndarray::Array2<A>
    where
        Sx: ndarray::Data<Elem = A>,
        Sz: ndarray::Data<Elem = A>,
        Sf: ndarray::Data<Elem = A>,
        Sh: ndarray::Data<Elem = A>,
    {
        let mut Pxz = ndarray::Array2::<A>::zeros((self.x.dim(), self.z.dim()));
        azip!((&Wci in &self.Wc, sfi in sigmas_f.genrows(), shi in sigmas_h.genrows()) {
            let dx = self.fns.x_residual(&sfi, x);
            let dz = self.fns.z_residual(&shi, z);
            Pxz += &(math::outer_product(&dx, &dz) * Wci);
        });
        Pxz
    }

    pub fn predict(&mut self, dt: A) -> Result<(), crate::Error> {
        // calculate sigma points for given mean and covariance
        let sigmas = self.points_fn.sigma_points(&self.x, &self.P)?;

        for i in 0..sigmas.nrows() {
            self.sigmas_f
                .index_axis_mut(ndarray::Axis(0), i)
                .assign(&self.fns.fx(&sigmas.index_axis(ndarray::Axis(0), i), dt));
        }

        // and pass sigmas through the unscented transform to compute prior
        let (x_prior, P_prior) = crate::unscented_transform(
            &self.sigmas_f,
            &self.Wm,
            &self.Wc,
            &self.Q,
            |sigmas, mean| self.fns.x_mean(sigmas, mean),
            |a, b| self.fns.x_residual(a, b),
        );

        self.x = x_prior;
        self.P = P_prior;

        // update sigma points to reflect the new variance of the points
        self.sigmas_f = self.points_fn.sigma_points(&self.x, &self.P)?;

        Ok(())
    }

    pub fn update<Sz>(
        &mut self,
        z: &ndarray::ArrayBase<Sz, ndarray::Ix1>,
    ) -> Result<(), crate::Error>
    where
        Sz: ndarray::Data<Elem = A>,
    {
        // transform sigma points into measurement space
        for i in 0..self.sigmas_f.nrows() {
            self.sigmas_h
                .index_axis_mut(ndarray::Axis(0), i)
                .assign(&self.fns.hx(&self.sigmas_f.index_axis(ndarray::Axis(0), i)));
        }

        // mean and covariance of prediction passed through UT
        let (zp, S) = crate::unscented_transform(
            &self.sigmas_h,
            &self.Wm,
            &self.Wc,
            &self.R,
            |sigmas, mean| self.fns.z_mean(sigmas, mean),
            |a, b| self.fns.z_residual(a, b),
        );

        // residual of z
        let y = self.fns.z_residual(&z, &zp);

        // compute cross variance of the state and the measurements
        let Pxz = self.cross_variance(&self.x, &zp, &self.sigmas_f, &self.sigmas_h);

        // Kalman gain
        let K = Pxz.dot(&S.inv()?);

        // new state estimate
        self.x = &self.x + &K.dot(&y);
        self.P = &self.P - &K.dot(&S.dot(&K.t()));

        // provide internal results
        self.y = y;
        self.S = S;

        Ok(())
    }

    /// log-likelihood of the last measurement
    pub fn log_likelihood(&self) -> Result<A, Error> {
        let mean = ndarray::Array1::zeros(self.y.len());
        Ok(math::multivariate::logpdf(&self.y, &mean, &self.S, true)?)
    }

    /// Computed from the log-likelihood. The log-likelihood can be very
    /// small,  meaning a large negative value such as -28000. Taking the
    /// exp() of that results in 0.0, which can break typical algorithms
    /// which multiply by this value, so by default we always return a
    /// number >= `A::min_positive_value`
    pub fn likelihood(&self) -> Result<A, Error> {
        let ll = self.log_likelihood()?;
        let mut l = ll.exp();
        if l.is_zero() {
            l = A::min_positive_value();
        }

        Ok(l)
    }

    pub fn rts_smoother<Sx>(
        &mut self,
        xs: &[ndarray::ArrayBase<Sx, ndarray::Ix1>],
        Ps: &[ndarray::ArrayBase<Sx, ndarray::Ix2>],
        Qs: Option<&[ndarray::Array2<A>]>,
        dts: &[A],
    ) -> Result<RTSResult<A>, crate::Error>
    where
        Sx: ndarray::Data<Elem = A>,
    {
        assert_eq!(xs.len(), Ps.len());
        let mut xss = Vec::with_capacity(xs.len());
        let mut Pss = Vec::with_capacity(Ps.len());

        if xs.is_empty() {
            return Ok((xss, Pss));
        }

        xss.push(xs.last().unwrap().to_owned());
        Pss.push(Ps.last().unwrap().to_owned());

        for k in 0..xs.len() - 1 {
            let k = xs.len() - 2 - k;
            let x = &xs[k];
            let P = &Ps[k];
            let Q = match Qs {
                Some(Qs) => &Qs[k],
                None => &self.Q,
            };
            let dt = dts[k];

            // create sigma points from state estimate
            let sigmas = self.points_fn.sigma_points(x, P)?;

            // pass sigmas through state function
            for i in 0..sigmas.nrows() {
                self.sigmas_f
                    .index_axis_mut(ndarray::Axis(0), i)
                    .assign(&self.fns.fx(&sigmas.index_axis(ndarray::Axis(0), i), dt));
            }

            let (xb, Pb) = crate::unscented_transform(
                &self.sigmas_f,
                &self.Wm,
                &self.Wc,
                Q,
                |sigmas, mean| self.fns.x_mean(sigmas, mean),
                |a, b| self.fns.x_residual(a, b),
            );

            // compute cross variance
            let mut Pxb = ndarray::Array2::<A>::zeros((self.x.dim(), self.x.dim()));
            azip!((&Wci in &self.Wc, sfi in self.sigmas_f.genrows(), si in sigmas.genrows()) {
                let y = self.fns.x_residual(&sfi, &xb);
                let z = self.fns.x_residual(&si, x);
                Pxb += &(math::outer_product(&z, &y) * Wci);
            });

            // compute gain
            let K = Pxb.dot(&Pb.inv()?);

            // residual
            let residual = self.fns.x_residual(&xss.last().unwrap(), &xb);

            // update the smoothed estimates
            xss.push(self.fns.x_add(&x, &K.dot(&residual)));
            Pss.push(P + &K.dot(&(Pss.last().unwrap() - &Pb)).dot(&K.t()));
        }

        xss.reverse();
        Pss.reverse();

        Ok((xss, Pss))
    }
}

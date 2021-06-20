#![allow(non_snake_case)]

use crate::Error;
use crate::Filter;
use crate::SetDt;
use ndarray::azip;
use ndarray_linalg::solve::Inverse;

type RTSResult<A> = (Vec<ndarray::Array1<A>>, Vec<ndarray::Array2<A>>);

pub trait Mean<A> {
    fn mean<Ss, Swm>(
        &self,
        sigmas: &ndarray::ArrayBase<Ss, ndarray::Ix2>,
        Wm: &ndarray::ArrayBase<Swm, ndarray::Ix1>,
    ) -> ndarray::Array1<A>
    where
        Ss: ndarray::Data<Elem = A>,
        Swm: ndarray::Data<Elem = A>;
}

pub trait Fx<A> {
    type Elem;

    fn fx<S>(
        &self,
        x: &ndarray::ArrayBase<S, ndarray::Ix1>,
        args: &A,
    ) -> ndarray::Array1<Self::Elem>
    where
        S: ndarray::Data<Elem = Self::Elem>;
}

pub trait ApplyDt<A> {
    fn apply_dt(&self, Q: &mut ndarray::Array2<A>);
}

pub trait Hx {
    type Elem;

    fn hx<S>(&self, x: &ndarray::ArrayBase<S, ndarray::Ix1>) -> ndarray::Array1<Self::Elem>
    where
        S: ndarray::Data<Elem = Self::Elem>;
}

#[derive(Clone, Debug)]
pub struct UKF<'a, FP, FNSX, ARGSFX, FNSZ, A, Sz> {
    fns_x: FNSX,
    args_fx: ARGSFX,
    fns_z: FNSZ,

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

    pd_Sz: std::marker::PhantomData<Sz>,
}

impl<'a, FP, FNSX, ARGSFX, FNSZ, A, Sz> SetDt<A> for UKF<'a, FP, FNSX, ARGSFX, FNSZ, A, Sz>
where
    ARGSFX: SetDt<A> + ApplyDt<A>,
{
    fn set_dt(&mut self, dt: &A) {
        self.args_fx.set_dt(dt);
        self.args_fx.apply_dt(&mut self.Q);
    }
}

impl<'a, FP, FNSX, ARGSFX, FNSZ, A, Sz> Filter for UKF<'a, FP, FNSX, ARGSFX, FNSZ, A, Sz>
where
    FP: crate::sigma_points::SigmaPoints<Elem = A>,
    FNSX: Fx<ARGSFX, Elem = A> + Hx<Elem = A> + Mean<A> + crate::Add<A> + crate::Subtract<A>,
    FNSZ: Mean<A> + crate::Subtract<A>,
    A: Copy
        + num_traits::float::FloatConst
        + num_traits::float::Float
        + ndarray::ScalarOperand
        + ndarray_linalg::types::Lapack
        + std::convert::From<f32>,
    <A as ndarray_linalg::Scalar>::Real: std::convert::From<f32>,
    Sz: ndarray::Data<Elem = A>,
{
    type Elem = A;
    type Meas = ndarray::ArrayBase<Sz, ndarray::Ix1>;

    fn predict(&mut self) -> Result<(), crate::Error> {
        // calculate sigma points for given mean and covariance
        let sigmas = self.points_fn.sigma_points(&self.x, &self.P)?;

        for i in 0..sigmas.nrows() {
            self.sigmas_f.index_axis_mut(ndarray::Axis(0), i).assign(
                &self
                    .fns_x
                    .fx(&sigmas.index_axis(ndarray::Axis(0), i), &self.args_fx),
            );
        }

        // and pass sigmas through the unscented transform to compute prior
        let (x_prior, P_prior) = crate::unscented_transform(
            &self.sigmas_f,
            &self.Wm,
            &self.Wc,
            &self.Q,
            |sigmas, mean| self.fns_x.mean(sigmas, mean),
            |a, b| self.fns_x.subtract(a, b),
        );

        self.x = x_prior;
        self.P = P_prior;

        // update sigma points to reflect the new variance of the points
        self.sigmas_f = self.points_fn.sigma_points(&self.x, &self.P)?;

        Ok(())
    }

    fn update(&mut self, z: &ndarray::ArrayBase<Sz, ndarray::Ix1>) -> Result<(), crate::Error> {
        // transform sigma points into measurement space
        for i in 0..self.sigmas_f.nrows() {
            self.sigmas_h.index_axis_mut(ndarray::Axis(0), i).assign(
                &self
                    .fns_x
                    .hx(&self.sigmas_f.index_axis(ndarray::Axis(0), i)),
            );
        }

        // mean and covariance of prediction passed through UT
        let (zp, S) = crate::unscented_transform(
            &self.sigmas_h,
            &self.Wm,
            &self.Wc,
            &self.R,
            |sigmas, mean| self.fns_z.mean(sigmas, mean),
            |a, b| self.fns_z.subtract(a, b),
        );

        // residual of z
        let y = self.fns_z.subtract(&z, &zp);

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

    /// Computed from the log-likelihood. The log-likelihood can be very
    /// small,  meaning a large negative value such as -28000. Taking the
    /// exp() of that results in 0.0, which can break typical algorithms
    /// which multiply by this value, so by default we always return a
    /// number >= `A::min_positive_value`
    fn likelihood(&self) -> Result<A, Error> {
        let ll = self.log_likelihood()?;
        let mut l = num_traits::Float::exp(ll);
        if l.is_zero() {
            l = A::min_positive_value();
        }

        Ok(l)
    }

    fn x(&self) -> &ndarray::Array1<A> {
        &self.x
    }

    fn x_mut(&mut self) -> &mut ndarray::Array1<A> {
        &mut self.x
    }

    fn P(&self) -> &ndarray::Array2<A> {
        &self.P
    }

    fn P_mut(&mut self) -> &mut ndarray::Array2<A> {
        &mut self.P
    }
}

impl<'a, FP, FNSX, ARGSFX, FNSZ, A, Sz> crate::Q for UKF<'a, FP, FNSX, ARGSFX, FNSZ, A, Sz> {
    type Elem = A;

    fn Q(&self) -> &ndarray::Array2<A> {
        &self.Q
    }

    fn Q_mut(&mut self) -> &mut ndarray::Array2<A> {
        &mut self.Q
    }
}

impl<'a, FP, FNSX, ARGSFX, FNSZ, A, Sz> crate::R for UKF<'a, FP, FNSX, ARGSFX, FNSZ, A, Sz> {
    type Elem = A;

    fn R(&self) -> &ndarray::Array2<A> {
        &self.R
    }

    fn R_mut(&mut self) -> &mut ndarray::Array2<A> {
        &mut self.R
    }
}

impl<'a, FP, FNSX, ARGSFX, FNSZ, A, Sz> UKF<'a, FP, FNSX, ARGSFX, FNSZ, A, Sz>
where
    FP: crate::sigma_points::SigmaPoints<Elem = A>,
    FNSX: Fx<ARGSFX, Elem = A> + Mean<A> + crate::Add<A> + crate::Subtract<A>,
    FNSZ: Mean<A> + crate::Subtract<A>,
    A: Copy
        + num_traits::float::FloatConst
        + num_traits::float::Float
        + ndarray::ScalarOperand
        + ndarray_linalg::types::Lapack
        + std::convert::From<f32>,
    <A as ndarray_linalg::Scalar>::Real: std::convert::From<f32>,
{
    pub fn new(
        dim_x: usize,
        dim_z: usize,
        points_fn: &'a FP,
        fns_x: FNSX,
        args_fx: ARGSFX,
        fns_z: FNSZ,
    ) -> Self {
        Self {
            fns_x,
            args_fx,
            fns_z,

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

            pd_Sz: std::marker::PhantomData,
        }
    }

    fn cross_variance<Sx, Szl, Sf, Sh>(
        &self,
        x: &ndarray::ArrayBase<Sx, ndarray::Ix1>,
        z: &ndarray::ArrayBase<Szl, ndarray::Ix1>,
        sigmas_f: &ndarray::ArrayBase<Sf, ndarray::Ix2>,
        sigmas_h: &ndarray::ArrayBase<Sh, ndarray::Ix2>,
    ) -> ndarray::Array2<A>
    where
        Sx: ndarray::Data<Elem = A>,
        Szl: ndarray::Data<Elem = A>,
        Sf: ndarray::Data<Elem = A>,
        Sh: ndarray::Data<Elem = A>,
    {
        let mut Pxz = ndarray::Array2::<A>::zeros((self.x.dim(), self.z.dim()));
        azip!((&Wci in &self.Wc, sfi in sigmas_f.genrows(), shi in sigmas_h.genrows()) {
            let dx = self.fns_x.subtract(&sfi, x);
            let dz = self.fns_z.subtract(&shi, z);
            Pxz += &(math::outer_product(&dx, &dz) * Wci);
        });
        Pxz
    }

    /// log-likelihood of the last measurement
    pub fn log_likelihood(&self) -> Result<A, Error> {
        let mean = ndarray::Array1::zeros(self.y.len());
        Ok(math::multivariate::logpdf(&self.y, &mean, &self.S, true)?)
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
        ARGSFX: crate::SetDt<A>,
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
            self.args_fx.set_dt(&dt);

            // create sigma points from state estimate
            let sigmas = self.points_fn.sigma_points(x, P)?;

            // pass sigmas through state function
            for i in 0..sigmas.nrows() {
                self.sigmas_f.index_axis_mut(ndarray::Axis(0), i).assign(
                    &self
                        .fns_x
                        .fx(&sigmas.index_axis(ndarray::Axis(0), i), &self.args_fx),
                );
            }

            let (xb, Pb) = crate::unscented_transform(
                &self.sigmas_f,
                &self.Wm,
                &self.Wc,
                Q,
                |sigmas, mean| self.fns_x.mean(sigmas, mean),
                |a, b| self.fns_x.subtract(a, b),
            );

            // compute cross variance
            let mut Pxb = ndarray::Array2::<A>::zeros((self.x.dim(), self.x.dim()));
            azip!((&Wci in &self.Wc, sfi in self.sigmas_f.genrows(), si in sigmas.genrows()) {
                let y = self.fns_x.subtract(&sfi, &xb);
                let z = self.fns_x.subtract(&si, x);
                Pxb += &(math::outer_product(&z, &y) * Wci);
            });

            // compute gain
            let K = Pxb.dot(&Pb.inv()?);

            // residual
            let residual = self.fns_x.subtract(&xss.last().unwrap(), &xb);

            // update the smoothed estimates
            xss.push(self.fns_x.add(&x, &K.dot(&residual)));
            Pss.push(P + &K.dot(&(Pss.last().unwrap() - &Pb)).dot(&K.t()));
        }

        xss.reverse();
        Pss.reverse();

        Ok((xss, Pss))
    }
}

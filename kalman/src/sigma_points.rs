#![allow(non_snake_case)]

use ndarray_linalg::cholesky::Cholesky;
use std::ops::Neg;

pub trait SigmaPoints {
    type Elem;

    fn num_sigmas(&self) -> usize;

    fn sigma_points<S>(
        &self,
        x: &ndarray::ArrayBase<S, ndarray::Ix1>,
        P: &ndarray::ArrayBase<S, ndarray::Ix2>,
    ) -> Result<ndarray::Array2<Self::Elem>, crate::Error>
    where
        S: ndarray::Data<Elem = Self::Elem>;

    fn weights_covariance(&self) -> ndarray::Array1<Self::Elem>;
    fn weights_mean(&self) -> ndarray::Array1<Self::Elem>;
}

#[derive(Clone, Debug)]
pub struct MerweScaledSigmaPoints<FNS> {
    fns: FNS,

    n: usize,
    alpha: f64,
    beta: f64,
    kappa: f64,
}

impl<FNS> MerweScaledSigmaPoints<FNS> {
    /// n: number of dimensions
    /// alpha: between 0 and 1, a larger value spreads the sigma points further from the mean
    /// beta: 2 is a good choice for gaussian problems
    /// kappa: 3 - n
    pub fn new(n: usize, alpha: f64, beta: f64, kappa: f64, fns: FNS) -> Self {
        Self {
            fns,
            n,
            alpha,
            beta,
            kappa,
        }
    }

    fn lambda(&self) -> f64 {
        let nf = self.n as f64;
        self.alpha.powf(2.0) * (nf + self.kappa) - nf
    }

    fn c(&self, lambda: f64) -> f64 {
        let nf = self.n as f64;
        0.5 / (nf + lambda)
    }
}

impl<FNS> SigmaPoints for MerweScaledSigmaPoints<FNS>
where
    FNS: crate::Subtract<f64>,
{
    type Elem = f64;

    fn num_sigmas(&self) -> usize {
        2 * self.n + 1
    }

    fn sigma_points<S>(
        &self,
        x: &ndarray::ArrayBase<S, ndarray::Ix1>,
        P: &ndarray::ArrayBase<S, ndarray::Ix2>,
    ) -> Result<ndarray::Array2<Self::Elem>, crate::Error>
    where
        S: ndarray::Data<Elem = Self::Elem>,
    {
        assert_eq!(x.dim(), self.n);
        assert_eq!(P.dim(), (self.n, self.n));

        let lambda = self.lambda();
        let U = (((self.n as Self::Elem) + lambda) * P).cholesky(ndarray_linalg::UPLO::Upper)?;

        let mut sigmas = ndarray::Array2::<Self::Elem>::zeros((2 * self.n + 1, self.n));
        sigmas.index_axis_mut(ndarray::Axis(0), 0).assign(&x);

        for k in 0..self.n {
            let Uk = U.index_axis(ndarray::Axis(0), k);
            sigmas
                .index_axis_mut(ndarray::Axis(0), k + 1)
                .assign(&self.fns.subtract(&Uk, &x.neg()));
            sigmas
                .index_axis_mut(ndarray::Axis(0), self.n + k + 1)
                .assign(&self.fns.subtract(&Uk.neg().view(), &x.neg()));
        }

        Ok(sigmas)
    }

    fn weights_covariance(&self) -> ndarray::Array1<Self::Elem> {
        let lambda = self.lambda();
        let mut wc = ndarray::Array1::<Self::Elem>::from_elem(2 * self.n + 1, self.c(lambda));
        wc[0] =
            lambda / ((self.n as Self::Elem) + lambda) + (1.0 - self.alpha.powf(2.0) + self.beta);
        wc
    }

    fn weights_mean(&self) -> ndarray::Array1<Self::Elem> {
        let lambda = self.lambda();
        let mut wm = ndarray::Array1::<Self::Elem>::from_elem(2 * self.n + 1, self.c(lambda));
        wm[0] = lambda / ((self.n as Self::Elem) + lambda);
        wm
    }
}

#[derive(Clone, Debug)]
pub struct JulierSigmaPoints<FNS> {
    fns: FNS,

    n: usize,
    kappa: f64,
}

impl<FNS> JulierSigmaPoints<FNS> {
    pub fn new(n: usize, kappa: f64, fns: FNS) -> Self {
        Self { fns, n, kappa }
    }
}

impl<FNS> SigmaPoints for JulierSigmaPoints<FNS>
where
    FNS: crate::Subtract<f64>,
{
    type Elem = f64;

    fn num_sigmas(&self) -> usize {
        2 * self.n + 1
    }

    fn sigma_points<S>(
        &self,
        x: &ndarray::ArrayBase<S, ndarray::Ix1>,
        P: &ndarray::ArrayBase<S, ndarray::Ix2>,
    ) -> Result<ndarray::Array2<Self::Elem>, crate::Error>
    where
        S: ndarray::Data<Elem = Self::Elem>,
    {
        assert_eq!(x.dim(), self.n);
        assert_eq!(P.dim(), (self.n, self.n));

        let U =
            (((self.n as Self::Elem) + self.kappa) * P).cholesky(ndarray_linalg::UPLO::Upper)?;

        let mut sigmas = ndarray::Array2::<Self::Elem>::zeros((2 * self.n + 1, self.n));
        sigmas.index_axis_mut(ndarray::Axis(0), 0).assign(&x);

        for k in 0..self.n {
            let Uk = U.index_axis(ndarray::Axis(0), k);
            sigmas
                .index_axis_mut(ndarray::Axis(0), k + 1)
                .assign(&self.fns.subtract(&Uk, &x.neg()));
            sigmas
                .index_axis_mut(ndarray::Axis(0), self.n + k + 1)
                .assign(&self.fns.subtract(&Uk.neg().view(), &x.neg()));
        }

        Ok(sigmas)
    }

    fn weights_covariance(&self) -> ndarray::Array1<Self::Elem> {
        self.weights_mean()
    }

    fn weights_mean(&self) -> ndarray::Array1<Self::Elem> {
        let npk = self.n as Self::Elem + self.kappa;

        let mut wm = ndarray::Array1::<Self::Elem>::from_elem(2 * self.n + 1, 0.5 / npk);
        wm[0] = self.kappa / npk;
        wm
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use ndarray::array;

    #[derive(Default)]
    pub struct LinFns;

    impl<A: num_traits::Float> crate::Add<A> for LinFns {
        fn add<Sa, Sb>(
            &self,
            a: &ndarray::ArrayBase<Sa, ndarray::Ix1>,
            b: &ndarray::ArrayBase<Sb, ndarray::Ix1>,
        ) -> ndarray::Array1<A>
        where
            Sa: ndarray::Data<Elem = A>,
            Sb: ndarray::Data<Elem = A>,
        {
            a + b
        }
    }

    impl<A: num_traits::Float> crate::Subtract<A> for LinFns {
        fn subtract<Sa, Sb>(
            &self,
            a: &ndarray::ArrayBase<Sa, ndarray::Ix1>,
            b: &ndarray::ArrayBase<Sb, ndarray::Ix1>,
        ) -> ndarray::Array1<A>
        where
            Sa: ndarray::Data<Elem = A>,
            Sb: ndarray::Data<Elem = A>,
        {
            a - b
        }
    }

    #[test]
    fn merwe() {
        let fns = LinFns::default();
        let points = MerweScaledSigmaPoints::new(2, 0.1, 2.0, 1.0, fns);
        let sigmas = points
            .sigma_points(&array![0.0, 0.0], &array![[1.0, 0.1], [0.1, 1.0]])
            .unwrap();
        let wc = points.weights_covariance();
        let wm = points.weights_mean();

        testlib::assert_arr2_eq(
            &sigmas,
            &array![
                [0.0, 0.0],
                [0.17320508, 0.01732051],
                [0.0, 0.17233688],
                [-0.17320508, -0.01732051],
                [0.0, -0.17233688],
            ],
        );

        testlib::assert_arr1_eq(
            &wc,
            &array![
                -62.67666667,
                16.66666667,
                16.66666667,
                16.66666667,
                16.66666667
            ],
        );

        testlib::assert_arr1_eq(
            &wm,
            &array![
                -65.66666667,
                16.66666667,
                16.66666667,
                16.66666667,
                16.66666667
            ],
        );
    }

    #[test]
    fn julier() {
        let fns = LinFns::default();
        let points = JulierSigmaPoints::new(2, 1.0, fns);
        let sigmas = points
            .sigma_points(&array![0.0, 0.0], &array![[1.0, 0.1], [0.1, 1.0]])
            .unwrap();
        let wc = points.weights_covariance();
        let wm = points.weights_mean();

        testlib::assert_arr2_eq(
            &sigmas,
            &array![
                [0., 0.],
                [1.73205081, 0.17320508],
                [0., 1.72336879],
                [-1.73205081, -0.17320508],
                [0., -1.72336879]
            ],
        );

        testlib::assert_arr1_eq(
            &wc,
            &array![0.33333333, 0.16666667, 0.16666667, 0.16666667, 0.16666667],
        );

        testlib::assert_arr1_eq(
            &wm,
            &array![0.33333333, 0.16666667, 0.16666667, 0.16666667, 0.16666667],
        );
    }
}

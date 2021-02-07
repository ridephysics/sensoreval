#![allow(non_snake_case)]

use ndarray_linalg::solve::Inverse;

#[derive(Clone, Debug)]
pub struct Kalman<A, Sz> {
    dim_x: usize,
    dim_z: usize,
    pub x: ndarray::Array1<A>,
    pub P: ndarray::Array2<A>,
    pub Q: ndarray::Array2<A>,
    pub F: ndarray::Array2<A>,
    pub H: ndarray::Array2<A>,
    pub R: ndarray::Array2<A>,
    pub y: ndarray::Array1<A>,
    pub S: ndarray::Array2<A>,

    pd_Sz: std::marker::PhantomData<Sz>,
}

impl<A, Sz> crate::Filter for Kalman<A, Sz>
where
    A: num_traits::float::Float
        + num_traits::float::FloatConst
        + ndarray::ScalarOperand
        + ndarray_linalg::types::Lapack
        + std::ops::DivAssign
        + std::ops::AddAssign
        + std::convert::From<f32>
        + std::fmt::Display,
    <A as ndarray_linalg::Scalar>::Real: std::convert::From<f32>,
    Sz: ndarray::Data<Elem = A>,
{
    type Elem = A;
    type Meas = ndarray::ArrayBase<Sz, ndarray::Ix1>;

    fn predict(&mut self) -> Result<(), crate::Error> {
        self.x = self.F.dot(&self.x);
        self.P = self.F.dot(&self.P).dot(&self.F.t()) + &self.Q;
        Ok(())
    }

    fn update(&mut self, z: &ndarray::ArrayBase<Sz, ndarray::Ix1>) -> Result<(), crate::Error> {
        self.y = z - &self.H.dot(&self.x);
        let PHT = self.P.dot(&self.H.t());
        self.S = self.H.dot(&PHT) + &self.R;
        let SI = self.S.inv()?;
        let K = PHT.dot(&SI);
        self.x = &self.x + &K.dot(&self.y);

        let Ix = ndarray::Array2::<A>::eye(self.dim_x);
        let I_KH = Ix - K.dot(&self.H);
        self.P = &I_KH.dot(&self.P).dot(&I_KH.t()) + &K.dot(&self.R).dot(&K.t());
        Ok(())
    }

    fn likelihood(&self) -> Result<A, crate::Error> {
        let ll = self.log_likelihood()?;
        let mut l = ll.exp();
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

impl<A, Sz> Kalman<A, Sz>
where
    A: num_traits::float::Float
        + num_traits::float::FloatConst
        + ndarray::ScalarOperand
        + ndarray_linalg::types::Lapack
        + std::ops::AddAssign
        + std::convert::From<f32>,
    <A as ndarray_linalg::Scalar>::Real: std::convert::From<f32>,
{
    pub fn new(dim_x: usize, dim_z: usize) -> Result<Self, crate::Error> {
        Ok(Self {
            dim_x,
            dim_z,
            x: ndarray::Array1::zeros(dim_x),
            P: ndarray::Array2::eye(dim_x),
            Q: ndarray::Array2::eye(dim_x),
            F: ndarray::Array2::eye(dim_x),
            H: ndarray::Array2::zeros((dim_z, dim_x)),
            R: ndarray::Array2::eye(dim_z),
            y: ndarray::Array1::zeros(dim_z),
            S: ndarray::Array2::zeros((dim_z, dim_z)),
            pd_Sz: std::marker::PhantomData,
        })
    }

    pub fn log_likelihood(&self) -> Result<A, crate::Error> {
        let mean = ndarray::Array1::zeros(self.y.len());
        Ok(math::multivariate::logpdf(&self.y, &mean, &self.S, true)?)
    }
}

pub mod discretization;
pub mod imm;
pub mod kalman;
pub mod sigma_points;
pub mod ukf;

mod error;
pub use error::Error;

mod unscented_transform;
pub use unscented_transform::*;

#[cfg(test)]
extern crate lapack_src;

#[allow(non_snake_case)]
pub trait Filter<A, Z> {
    fn predict(&mut self) -> Result<(), crate::Error>;
    fn update(&mut self, z: &Z) -> Result<(), crate::Error>;

    fn x(&self) -> &ndarray::Array1<A>;
    fn x_mut(&mut self) -> &mut ndarray::Array1<A>;

    fn P(&self) -> &ndarray::Array2<A>;
    fn P_mut(&mut self) -> &mut ndarray::Array2<A>;

    fn likelihood(&self) -> Result<A, crate::Error>;
}

pub trait SetDt<T> {
    fn set_dt(&mut self, dt: &T);
}

pub trait ApplyDt<T, F> {
    fn apply_dt(dt: &T, filter: &mut F);
}

pub trait Subtract<A> {
    fn subtract<Sa, Sb>(
        &self,
        a: &ndarray::ArrayBase<Sa, ndarray::Ix1>,
        b: &ndarray::ArrayBase<Sb, ndarray::Ix1>,
    ) -> ndarray::Array1<A>
    where
        Sa: ndarray::Data<Elem = A>,
        Sb: ndarray::Data<Elem = A>;
}

pub trait Add<A> {
    fn add<Sa, Sb>(
        &self,
        a: &ndarray::ArrayBase<Sa, ndarray::Ix1>,
        b: &ndarray::ArrayBase<Sb, ndarray::Ix1>,
    ) -> ndarray::Array1<A>
    where
        Sa: ndarray::Data<Elem = A>,
        Sb: ndarray::Data<Elem = A>;
}

pub trait Normalize<A> {
    fn normalize(&self, x: ndarray::Array1<A>) -> ndarray::Array1<A>;
}

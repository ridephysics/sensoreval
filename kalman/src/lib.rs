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

pub mod discretization;
pub mod sigma_points;
pub mod ukf;

mod error;
pub use error::Error;

mod unscented_transform;
pub use unscented_transform::*;

#[cfg(test)]
extern crate lapack_src;

#[allow(non_snake_case)]
pub trait Filter<A> {
    fn predict(&mut self, dt: A) -> Result<(), crate::Error>;
    fn update<Sz>(&mut self, z: &ndarray::ArrayBase<Sz, ndarray::Ix1>) -> Result<(), crate::Error>
    where
        Sz: ndarray::Data<Elem = A>;
    fn likelihood(&self) -> Result<A, crate::Error>;
    fn x(&self) -> &ndarray::Array1<A>;
    fn P(&self) -> &ndarray::Array2<A>;
    fn x_mut(&mut self) -> &mut ndarray::Array1<A>;
    fn P_mut(&mut self) -> &mut ndarray::Array2<A>;
}

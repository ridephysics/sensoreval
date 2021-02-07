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
pub trait Filter {
    type Elem;
    type Meas;

    fn predict(&mut self) -> Result<(), crate::Error>;
    fn update(&mut self, z: &Self::Meas) -> Result<(), crate::Error>;

    fn x(&self) -> &ndarray::Array1<Self::Elem>;
    fn x_mut(&mut self) -> &mut ndarray::Array1<Self::Elem>;

    fn P(&self) -> &ndarray::Array2<Self::Elem>;
    fn P_mut(&mut self) -> &mut ndarray::Array2<Self::Elem>;

    fn likelihood(&self) -> Result<Self::Elem, crate::Error>;
}

#[allow(non_snake_case)]
pub trait Q {
    type Elem;

    fn Q(&self) -> &ndarray::Array2<Self::Elem>;
    fn Q_mut(&mut self) -> &mut ndarray::Array2<Self::Elem>;
}

#[allow(non_snake_case)]
pub trait R {
    type Elem;

    fn R(&self) -> &ndarray::Array2<Self::Elem>;
    fn R_mut(&mut self) -> &mut ndarray::Array2<Self::Elem>;
}

pub trait SetDt<T> {
    fn set_dt(&mut self, dt: &T);
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

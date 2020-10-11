pub trait Model {
    fn step<S>(&mut self, x: &mut ndarray::ArrayBase<S, ndarray::Ix1>)
    where
        S: ndarray::DataMut<Elem = f64>;
    fn set_dt(&mut self, dt: f64);
    fn dt(&self) -> f64;

    fn set_control_input(&mut self, _ci: Option<&[f64]>) {}
}

pub trait ToImuSample {
    fn to_imusample<S>(
        &self,
        state: &ndarray::ArrayBase<S, ndarray::Ix1>,
        accel: &mut ndarray::ArrayBase<S, ndarray::Ix1>,
        gyro: &mut ndarray::ArrayBase<S, ndarray::Ix1>,
    ) where
        S: ndarray::DataMut<Elem = f64>;
}

macro_rules! impl_model_inner {
    ($field:ident) => {
        fn step<S>(&mut self, x: &mut ndarray::ArrayBase<S, ndarray::Ix1>)
        where
            S: ndarray::DataMut<Elem = f64>,
        {
            self.$field.iterate(x);
        }

        fn set_dt(&mut self, dt: f64) {
            self.$field.set_dt(dt);
        }

        fn dt(&self) -> f64 {
            self.$field.get_dt()
        }
    };
}

macro_rules! impl_model {
    ($type:ty, $field:ident) => {
        impl crate::Model for $type {
            impl_model_inner!($field);
        }
    };
}

pub mod models;
pub mod utils;

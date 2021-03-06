#[enum_dispatch::enum_dispatch]
pub trait Model {
    fn step<S>(&mut self, x: &mut ndarray::ArrayBase<S, ndarray::Ix1>)
    where
        S: ndarray::DataMut<Elem = f64>;
    fn normalize<S>(&self, _x: &mut ndarray::ArrayBase<S, ndarray::Ix1>)
    where
        S: ndarray::DataMut<Elem = f64>,
    {
    }
    fn set_dt(&mut self, dt: f64);
    fn dt(&self) -> f64;

    fn set_control_input(&mut self, _ci: Option<&[f64]>) {}
}

#[enum_dispatch::enum_dispatch]
pub trait ToImuSample {
    fn to_accel<Sa, Sb>(
        &self,
        state: &ndarray::ArrayBase<Sa, ndarray::Ix1>,
        accel: &mut ndarray::ArrayBase<Sb, ndarray::Ix1>,
    ) where
        Sa: ndarray::Data<Elem = f64>,
        Sb: ndarray::DataMut<Elem = f64>;

    fn to_gyro<Sa, Sb>(
        &self,
        state: &ndarray::ArrayBase<Sa, ndarray::Ix1>,
        gyro: &mut ndarray::ArrayBase<Sb, ndarray::Ix1>,
    ) where
        Sa: ndarray::Data<Elem = f64>,
        Sb: ndarray::DataMut<Elem = f64>;

    /// returns the height in meters
    fn to_height<S>(&self, _state: &ndarray::ArrayBase<S, ndarray::Ix1>) -> f64
    where
        S: ndarray::Data<Elem = f64>,
    {
        0.0
    }
}

#[enum_dispatch::enum_dispatch]
pub trait DrawState {
    fn draw_state<S>(&self, cr: &cairo::Context, state: &ndarray::ArrayBase<S, ndarray::Ix1>)
    where
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
pub mod run;
pub mod utils;

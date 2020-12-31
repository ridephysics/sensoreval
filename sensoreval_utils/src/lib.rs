mod error;
pub use error::Error;

mod drain_filter;
pub use drain_filter::DrainFilterTrait;

mod intoitermap;
pub use intoitermap::IntoIteratorMap;

mod plot;
pub use plot::Plot;
pub use plot::COLOR_A;
pub use plot::COLOR_E;
pub use plot::COLOR_M;

mod python;
pub use python::Python;

mod timedarray;
pub use timedarray::TimedArray;

pub trait StateUtils {
    fn len() -> usize;
    fn id(&self) -> usize;
}
pub trait AssignState<A> {
    fn assign_state(&mut self, args: A);
}
pub use sensoreval_macros as macros;

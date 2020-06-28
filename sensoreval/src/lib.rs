#[macro_export]
macro_rules! unwrap_opt_or {
    ($opt:expr, $default:expr) => {
        match $opt {
            Some(x) => x,
            None => $default,
        }
    };
}

// this forces them to get linked into the binaries
extern crate blas_src;
extern crate lapack_src;

use plotly_types as plotly;

pub mod config;
#[macro_use]
mod data;
pub mod datareader;
mod error;
mod hudrenderers;
mod plot;
pub mod render;
mod simulator;

mod python;
pub use python::*;

mod drain_filter;
use drain_filter::*;

mod intoitermap;
pub use intoitermap::IntoIteratorMap;

pub use data::*;
pub use error::*;
pub use plot::*;

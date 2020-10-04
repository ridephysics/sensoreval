#[macro_export]
macro_rules! unwrap_opt_or {
    ($opt:expr, $default:expr) => {
        match $opt {
            Some(x) => x,
            None => $default,
        }
    };
}

pub mod config;
pub mod datareader;
mod hudrenderers;
pub mod render;

mod data;
pub use data::id_for_time;
pub use data::Data;

mod error;
pub use error::Error;

mod plotutils;
pub use plotutils::PlotUtils;

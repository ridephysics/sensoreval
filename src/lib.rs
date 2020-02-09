macro_rules! unwrap_res_or {
    ($opt:expr, $default:expr) => {
        match $opt {
            Err(e) => {
                println!("{:?}", e);
                $default
            }
            Ok(v) => v,
        }
    };
}

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
#[macro_use]
mod data;
mod capi;
mod datareader;
mod error;
mod hudrenderers;
mod math;
mod plot;
pub mod render;
mod simulator;

mod drain_filter;
use drain_filter::*;

pub use data::*;
pub use error::*;
pub use plot::*;

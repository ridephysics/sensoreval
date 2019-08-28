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
pub mod capi;
pub mod datareader;
mod error;
pub(crate) mod hudhandlers;
mod plot;
pub mod render;

pub use data::*;
pub use error::*;
pub use plot::*;

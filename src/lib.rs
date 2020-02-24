#![allow(dead_code)]

/*mod bindings {
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]

    include!(concat!(env!("OUT_DIR"), "/bindings_render_native.rs"));
}*/

macro_rules! unwrap_res_or {
    ($opt:expr, $default:expr) => {
        match $opt {
            Err(_) => $default,
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

macro_rules! num2t {
    ($type:ty, $num:expr) => {
        <$type>::from($num).ok_or(Error::FloatConversion)?
    };
}

// this forces them to get linked into the binaries
extern crate blas_src;
extern crate lapack_src;

pub mod config;
#[macro_use]
mod data;
mod capi;
mod datareader;
mod error;
pub mod ffmpeg;
mod hudrenderers;
mod kalman;
mod math;
mod plot;
pub mod render;
mod simulator;

#[cfg(test)]
mod test;

mod drain_filter;
use drain_filter::*;

pub use data::*;
pub use error::*;
pub use plot::*;

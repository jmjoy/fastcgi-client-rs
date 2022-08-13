#![warn(rust_2018_idioms)]
#![warn(clippy::dbg_macro, clippy::print_stdout)]
#![doc = include_str!("../README.md")]

pub mod client;
pub mod conn;
mod error;
mod meta;
pub mod params;
pub mod request;
pub mod response;

pub use crate::{client::Client, error::*, params::Params, request::Request, response::Response};

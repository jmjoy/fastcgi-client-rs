//! Fastcgi client implemented for Rust.

mod client;
mod error;
mod id;
mod meta;
mod params;

pub use crate::client::{Client, ClientBuilder};
pub use crate::error::{ClientError, ClientResult};
pub use crate::meta::Address;
pub use crate::params::Params;

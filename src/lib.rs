// Copyright 2022 jmjoy
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![warn(rust_2018_idioms)]
#![warn(clippy::dbg_macro, clippy::print_stdout, missing_docs)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]

#[cfg(not(any(feature = "runtime-tokio", feature = "runtime-smol")))]
compile_error!("Enable at least one runtime feature: `runtime-tokio` or `runtime-smol`.");

pub mod client;
pub mod conn;
mod error;
pub mod io;
mod meta;
pub mod params;
pub mod request;
pub mod response;

/// Re Export StreamExt for .next support
pub use futures_util::stream::StreamExt;

pub use crate::{client::Client, error::*, params::Params, request::Request, response::Response};

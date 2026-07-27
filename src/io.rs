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

//! Runtime-neutral async I/O facade used by the FastCGI client.
//!
//! The client is built on the `futures_io` traits, so any stream implementing
//! them works without enabling a cargo feature. The Tokio re-exports below are
//! only a convenience bridge for Tokio's own I/O traits.

pub use std::io::{Error, ErrorKind, Result};

pub use futures_io::{AsyncRead, AsyncWrite};
pub use futures_util::io::{AsyncReadExt, AsyncWriteExt, Cursor, Empty, empty};

#[cfg(feature = "tokio")]
pub use tokio_util::compat::{
    Compat as TokioCompat, TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt,
};

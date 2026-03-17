//! Runtime-neutral async I/O facade used by the FastCGI client.

pub use std::io::{Error, ErrorKind, Result};

pub use futures_io::{AsyncRead, AsyncWrite};
pub use futures_util::io::{empty, AsyncReadExt, AsyncWriteExt, Cursor, Empty};

#[cfg(feature = "runtime-tokio")]
pub use tokio_util::compat::{
    Compat as TokioCompat, TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt,
};

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

//! Connection mode markers for [`Client`](crate::Client).
//!
//! A [`Client`](crate::Client) is parameterized by a connection mode, which is
//! a compile time only marker: it selects the value of the `FCGI_KEEP_CONN`
//! flag sent in the `FCGI_BEGIN_REQUEST` record, and it decides which request
//! methods are available.
//!
//! This crate never creates, closes or otherwise manages sockets; the stream is
//! owned by the caller. The mode therefore only describes what the client tells
//! the FastCGI server about connection reuse, and how the client type behaves
//! afterwards:
//!
//! - [`ShortConn`]: the server is free to close the connection once the
//!   response is complete, so the client can be used for a single request and
//!   its request methods consume `self`.
//! - [`KeepAlive`]: the server is asked to keep the connection open, so the
//!   client stays usable and its request methods take `&mut self`, sending
//!   requests one after another.

mod sealed {
    /// Prevents [`Mode`](super::Mode) from being implemented outside of this
    /// crate.
    pub trait Sealed {}
}

/// Marker trait implemented by the connection modes of
/// [`Client`](crate::Client).
///
/// This trait is sealed: [`ShortConn`] and [`KeepAlive`] are its only
/// implementors and no other type can implement it, which keeps the set of
/// connection modes closed and allows it to gain new members without breaking
/// downstream code.
///
/// ```compile_fail
/// use fastcgi_client::conn::Mode;
///
/// enum MyMode {}
///
/// // error: the trait bound `MyMode: Sealed` is not satisfied
/// impl Mode for MyMode {
///     const KEEP_CONN: bool = true;
/// }
/// ```
pub trait Mode: sealed::Sealed {
    /// Value of the `FCGI_KEEP_CONN` flag sent in the `FCGI_BEGIN_REQUEST`
    /// record.
    ///
    /// When `false`, the FastCGI server closes the connection after responding.
    const KEEP_CONN: bool;
}

/// Short connection mode.
///
/// The `FCGI_KEEP_CONN` flag is not set, so the FastCGI server closes the
/// connection after it finished responding. Consequently the request methods of
/// [`Client<S, ShortConn>`](crate::Client) consume the client, and a new stream
/// is needed for the next request.
///
/// This type is a compile time marker and cannot be instantiated:
///
/// ```compile_fail
/// use fastcgi_client::conn::ShortConn;
///
/// // error: expected value, found enum `ShortConn`
/// let _mode = ShortConn;
/// ```
///
/// A short connection client is consumed by its request methods, so it cannot
/// be reused:
///
/// ```compile_fail
/// # async fn example() {
/// use fastcgi_client::{io, Client, Params, Request};
///
/// let stream = smol::net::TcpStream::connect(("127.0.0.1", 9000))
///     .await
///     .unwrap();
/// let client = Client::new(stream);
///
/// let _ = client
///     .execute_once(Request::new(Params::default(), io::empty()))
///     .await;
///
/// // error[E0382]: use of moved value: `client`
/// let _ = client
///     .execute_once(Request::new(Params::default(), io::empty()))
///     .await;
/// # }
/// ```
pub enum ShortConn {}

impl sealed::Sealed for ShortConn {}

impl Mode for ShortConn {
    const KEEP_CONN: bool = false;
}

/// Keep alive connection mode.
///
/// The `FCGI_KEEP_CONN` flag is set, asking the FastCGI server to keep the
/// connection open after responding. The request methods of
/// [`Client<S, KeepAlive>`](crate::Client) therefore take `&mut self` and can
/// be called repeatedly, sending one request at a time over the same stream.
///
/// This type is a compile time marker and cannot be instantiated:
///
/// ```compile_fail
/// use fastcgi_client::conn::KeepAlive;
///
/// // error: expected value, found enum `KeepAlive`
/// let _mode = KeepAlive {};
/// ```
pub enum KeepAlive {}

impl sealed::Sealed for KeepAlive {}

impl Mode for KeepAlive {
    const KEEP_CONN: bool = true;
}

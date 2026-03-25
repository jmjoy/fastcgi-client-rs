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

//! FastCGI request structure and builders.
//!
//! This module provides the `Request` struct that encapsulates
//! the parameters and stdin data for a FastCGI request.

#[cfg(feature = "http")]
use std::{borrow::Cow, collections::HashMap};

use crate::{Params, io::AsyncRead};

#[cfg(feature = "http")]
use crate::{HttpConversionError, HttpConversionResult};

#[cfg(feature = "runtime-tokio")]
use crate::io::{TokioAsyncReadCompatExt, TokioCompat};

/// FastCGI request containing parameters and stdin data.
///
/// This structure represents a complete FastCGI request with all necessary
/// parameters and an optional stdin stream for request body data.
pub struct Request<'a, I: AsyncRead + Unpin> {
    pub(crate) params: Params<'a>,
    pub(crate) stdin: I,
}

impl<'a, I: AsyncRead + Unpin> Request<'a, I> {
    /// Creates a new FastCGI request with the given parameters and stdin.
    ///
    /// # Arguments
    ///
    /// * `params` - The FastCGI parameters
    /// * `stdin` - The stdin stream for request body data
    pub fn new(params: Params<'a>, stdin: I) -> Self {
        Self { params, stdin }
    }

    /// Returns a reference to the request parameters.
    pub fn params(&self) -> &Params<'a> {
        &self.params
    }

    /// Returns a mutable reference to the request parameters.
    pub fn params_mut(&mut self) -> &mut Params<'a> {
        &mut self.params
    }

    /// Returns a reference to the stdin stream.
    pub fn stdin(&self) -> &I {
        &self.stdin
    }

    /// Returns a mutable reference to the stdin stream.
    pub fn stdin_mut(&mut self) -> &mut I {
        &mut self.stdin
    }

    /// Converts a FastCGI request into an `http::Request` without buffering the
    /// body.
    #[cfg(feature = "http")]
    pub fn try_into_http(self) -> HttpConversionResult<::http::Request<I>> {
        self.try_into()
    }
}

#[cfg(feature = "http")]
impl<I> Request<'static, I>
where
    I: AsyncRead + Unpin,
{
    /// Builds a FastCGI request from an `http::Request`, merging
    /// caller-provided FastCGI extras such as `SCRIPT_FILENAME`.
    pub fn try_from_http_with<'a>(
        request: ::http::Request<I>, extras: Params<'a>,
    ) -> HttpConversionResult<Self> {
        let (parts, body) = request.into_parts();
        let mut params: Params<'static> = parts.try_into()?;
        for (name, value) in HashMap::<Cow<'a, str>, Cow<'a, str>>::from(extras) {
            params.insert(
                Cow::Owned(name.into_owned()),
                Cow::Owned(value.into_owned()),
            );
        }
        Ok(Request::new(params, body))
    }

    /// Builds a FastCGI request from an `http::Request` using only
    /// HTTP-representable metadata.
    pub fn try_from_http(request: ::http::Request<I>) -> HttpConversionResult<Self> {
        Self::try_from_http_with(request, Params::default())
    }
}

#[cfg(feature = "http")]
impl<'a, I> TryFrom<Request<'a, I>> for ::http::Request<I>
where
    I: AsyncRead + Unpin,
{
    type Error = HttpConversionError;

    fn try_from(request: Request<'a, I>) -> Result<Self, Self::Error> {
        let (parts, body) = ((&request.params).try_into()?, request.stdin);
        Ok(::http::Request::from_parts(parts, body))
    }
}

#[cfg(feature = "runtime-tokio")]
impl<'a, I> Request<'a, TokioCompat<I>>
where
    I: tokio::io::AsyncRead + Unpin,
{
    /// Creates a new FastCGI request from a Tokio reader.
    pub fn new_tokio(params: Params<'a>, stdin: I) -> Self {
        Self::new(params, stdin.compat())
    }
}

#[cfg(feature = "runtime-smol")]
impl<'a, I> Request<'a, I>
where
    I: AsyncRead + Unpin,
{
    /// Creates a new FastCGI request from a Smol-compatible reader.
    pub fn new_smol(params: Params<'a>, stdin: I) -> Self {
        Self::new(params, stdin)
    }
}

#[cfg(all(test, feature = "http"))]
mod http_tests {
    use crate::{Params, Request, io};

    #[test]
    fn request_from_http_with_extras() {
        let request = ::http::Request::builder()
            .method(::http::Method::POST)
            .uri("/submit?foo=bar")
            .header(::http::header::HOST, "example.com")
            .body(io::Cursor::new(b"body".to_vec()))
            .unwrap();

        let extras = Params::default()
            .script_filename("/srv/www/index.php")
            .script_name("/index.php");
        let request = Request::try_from_http_with(request, extras).unwrap();

        assert_eq!(request.params()["REQUEST_METHOD"], "POST");
        assert_eq!(request.params()["QUERY_STRING"], "foo=bar");
        assert_eq!(request.params()["HTTP_HOST"], "example.com");
        assert_eq!(request.params()["SCRIPT_FILENAME"], "/srv/www/index.php");
    }

    #[test]
    fn request_into_http_preserves_stream_body() {
        let params = Params::default()
            .request_method("GET")
            .request_uri("/index.php");
        let request = Request::new(params, io::empty());

        let http_request = request.try_into_http().unwrap();

        assert_eq!(http_request.method(), ::http::Method::GET);
        assert_eq!(http_request.uri(), "/index.php");
    }
}

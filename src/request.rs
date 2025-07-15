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

use crate::Params;
use tokio::io::AsyncRead;

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
}

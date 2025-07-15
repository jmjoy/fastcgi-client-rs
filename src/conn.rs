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

//! Connection mode definitions for FastCGI clients.
//!
//! This module defines the different connection modes that can be used
//! with the FastCGI client: short connection and keep-alive modes.

/// Trait defining the behavior of different connection modes.
pub trait Mode {
    /// Returns whether this mode supports keep-alive connections.
    fn is_keep_alive() -> bool;
}

/// Short connection mode.
///
/// In this mode, the client establishes a new connection for each request
/// and closes it after receiving the response.
pub struct ShortConn;

impl Mode for ShortConn {
    fn is_keep_alive() -> bool {
        false
    }
}

/// Keep alive connection mode.
///
/// In this mode, the client maintains a persistent connection
/// and can send multiple requests over the same connection.
pub struct KeepAlive {}

impl Mode for KeepAlive {
    fn is_keep_alive() -> bool {
        true
    }
}

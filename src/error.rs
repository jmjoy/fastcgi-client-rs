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

//! Error types and result type aliases for FastCGI operations.
//!
//! This module defines the error types that can occur during FastCGI
//! communication and provides convenient type aliases for results.

use crate::meta::{ProtocolStatus, RequestType};

/// Result type alias for FastCGI client operations.
pub type ClientResult<T> = Result<T, ClientError>;

/// Error types that can occur during FastCGI communication.
#[derive(Debug, thiserror::Error)]
pub enum ClientError {
    /// Wapper of `tokio::io::Error`
    #[error(transparent)]
    Io(#[from] tokio::io::Error),

    /// Usually not happen.
    #[error("Response not found of request id `{id}`")]
    RequestIdNotFound {
        /// The request ID that was not found
        id: u16,
    },

    /// Usually not happen.
    #[error("Response not found of request id `{id}`")]
    ResponseNotFound {
        /// The request ID for which no response was found
        id: u16,
    },

    /// Maybe unimplemented request type received fom response.
    #[error("Response not found of request id `{request_type}`")]
    UnknownRequestType {
        /// The unknown request type received
        request_type: RequestType,
    },

    /// Response not complete, first is protocol status and second is app
    /// status, see fastcgi protocol.
    #[error("This app can't multiplex [CantMpxConn]; AppStatus: {app_status}")]
    EndRequestCantMpxConn {
        /// The application status code
        app_status: u32,
    },

    /// Response not complete, first is protocol status and second is app
    /// status, see fastcgi protocol.
    #[error("New request rejected; too busy [OVERLOADED]; AppStatus: {app_status}")]
    EndRequestOverloaded {
        /// The application status code
        app_status: u32,
    },

    /// Response not complete, first is protocol status and second is app
    /// status, see fastcgi protocol.
    #[error("Role value not known [UnknownRole]; AppStatus: {app_status}")]
    EndRequestUnknownRole {
        /// The application status code
        app_status: u32,
    },
}

impl ClientError {
    /// Creates a new end request error based on the protocol status.
    ///
    /// # Arguments
    ///
    /// * `protocol_status` - The protocol status returned by the FastCGI server
    /// * `app_status` - The application status code
    pub(crate) fn new_end_request_with_protocol_status(
        protocol_status: ProtocolStatus, app_status: u32,
    ) -> Self {
        match protocol_status {
            ProtocolStatus::CantMpxConn => ClientError::EndRequestCantMpxConn { app_status },
            ProtocolStatus::Overloaded => ClientError::EndRequestOverloaded { app_status },
            _ => ClientError::EndRequestUnknownRole { app_status },
        }
    }
}

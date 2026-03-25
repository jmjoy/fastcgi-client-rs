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

/// Result type alias for HTTP/FastCGI conversion operations.
#[cfg(feature = "http")]
pub type HttpConversionResult<T> = Result<T, HttpConversionError>;

/// Error types that can occur during FastCGI communication.
#[derive(Debug, thiserror::Error)]
pub enum ClientError {
    /// Wrapper of `std::io::Error`
    #[error(transparent)]
    Io(#[from] std::io::Error),

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

/// Error types that can occur while converting between FastCGI and HTTP types.
#[cfg(feature = "http")]
#[derive(Debug, thiserror::Error)]
pub enum HttpConversionError {
    /// Required FastCGI param is missing during HTTP conversion.
    #[error("Missing FastCGI param `{name}`")]
    MissingFastcgiParam {
        /// The missing FastCGI param name.
        name: &'static str,
    },

    /// Invalid HTTP method during conversion.
    #[error(transparent)]
    InvalidHttpMethod(#[from] ::http::method::InvalidMethod),

    /// Invalid HTTP URI during conversion.
    #[error(transparent)]
    InvalidHttpUri(#[from] ::http::uri::InvalidUri),

    /// Invalid HTTP header name during conversion.
    #[error(transparent)]
    InvalidHttpHeaderName(#[from] ::http::header::InvalidHeaderName),

    /// Invalid HTTP header value during conversion.
    #[error(transparent)]
    InvalidHttpHeaderValue(#[from] ::http::header::InvalidHeaderValue),

    /// Invalid HTTP status code during conversion.
    #[error(transparent)]
    InvalidHttpStatusCode(#[from] ::http::status::InvalidStatusCode),

    /// Invalid HTTP message constructed by builder.
    #[error(transparent)]
    InvalidHttpMessage(#[from] ::http::Error),

    /// CGI response payload could not be parsed into headers and body.
    #[error("Malformed CGI response: {message}")]
    MalformedHttpResponse {
        /// Human-readable parse failure reason.
        message: &'static str,
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

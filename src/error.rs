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

use crate::meta::{ProtocolStatus, RequestType};

pub type ClientResult<T> = Result<T, ClientError>;

#[derive(Debug, thiserror::Error)]
pub enum ClientError {
    /// Wapper of `tokio::io::Error`
    #[error(transparent)]
    Io(#[from] tokio::io::Error),

    /// Usually not happen.
    #[error("Response not found of request id `{id}`")]
    RequestIdNotFound { id: u16 },

    /// Usually not happen.
    #[error("Response not found of request id `{id}`")]
    ResponseNotFound { id: u16 },

    /// Maybe unimplemented request type received fom response.
    #[error("Response not found of request id `{request_type}`")]
    UnknownRequestType { request_type: RequestType },

    /// Response not complete, first is protocol status and second is app
    /// status, see fastcgi protocol.
    #[error("This app can't multiplex [CantMpxConn]; AppStatus: {app_status}")]
    EndRequestCantMpxConn { app_status: u32 },

    /// Response not complete, first is protocol status and second is app
    /// status, see fastcgi protocol.
    #[error("New request rejected; too busy [OVERLOADED]; AppStatus: {app_status}")]
    EndRequestOverloaded { app_status: u32 },

    /// Response not complete, first is protocol status and second is app
    /// status, see fastcgi protocol.
    #[error("Role value not known [UnknownRole]; AppStatus: {app_status}")]
    EndRequestUnknownRole { app_status: u32 },
}

impl ClientError {
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

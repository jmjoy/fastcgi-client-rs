use crate::meta::{ProtocolStatus, RequestType};
use std::fmt::Debug;
use std::fmt::{self, Display, Formatter};
use std::hint::unreachable_unchecked;
use std::io;

pub type ClientResult<T> = Result<T, ClientError>;

#[derive(Debug)]
pub enum ClientError {
    IoError(io::Error),
    RequestIdNotFound(u16),
    ResponseNotFound(u16),
    UnknownRequestType(RequestType),
    EndRequest(ProtocolStatus, u32),
}

impl Display for ClientError {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        match self {
            ClientError::IoError(e) => Display::fmt(e, f),
            ClientError::RequestIdNotFound(id) => Display::fmt(&format!("Request id `{}` not found.", id), f),
            ClientError::ResponseNotFound(id) => Display::fmt(&format!("Response not found of request id `{}`.", id), f),
            ClientError::UnknownRequestType(r#type) => Display::fmt(&format!("Unknown request type `{:?}`.", r#type), f),
            ClientError::EndRequest(protocol_status, app_status) => match protocol_status {
                ProtocolStatus::CantMpxConn => Display::fmt(&format!("This app can't multiplex [CantMpxConn]; AppStatus: {}", app_status), f),
                ProtocolStatus::Overloaded => Display::fmt(&format!("New request rejected; too busy [OVERLOADED]; AppStatus: {}", app_status), f),
                ProtocolStatus::UnknownRole => Display::fmt(&format!("Role value not known [UnknownRole]; AppStatus: {}", app_status), f),
                _ => unreachable!(),
            },
        }
    }
}

impl std::error::Error for ClientError {}

impl From<io::Error> for ClientError {
    fn from(e: io::Error) -> Self {
        ClientError::IoError(e)
    }
}

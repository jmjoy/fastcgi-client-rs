use crate::meta::RequestType;
use std::fmt::Debug;
use std::fmt::{self, Display, Formatter};
use std::io;

pub type ClientResult<T> = Result<T, ClientError>;

#[derive(Debug)]
pub enum ClientError {
    IoError(io::Error),
    ClientError(String),
    RequestIdNotFound(u16),
    UnknownRequestType(RequestType),
}

impl Display for ClientError {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        match self {
            ClientError::IoError(e) => Display::fmt(e, f),
            ClientError::ClientError(s) => Display::fmt(s, f),
            ClientError::RequestIdNotFound(id) => {
                Display::fmt(&format!("Request id `{}` not found.", id), f)
            }
            ClientError::UnknownRequestType(r#type) => {
                Display::fmt(&format!("Unknown request type `{:?}`.", r#type), f)
            }
        }
    }
}

impl std::error::Error for ClientError {}

impl From<io::Error> for ClientError {
    fn from(e: io::Error) -> Self {
        ClientError::IoError(e)
    }
}

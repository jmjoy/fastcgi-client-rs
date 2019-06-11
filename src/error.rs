use std::io;
use std::fmt::{self, Display, Formatter};

pub(crate) type ClientResult<T> = Result<T, ClientError>;

#[derive(Debug)]
pub(crate) enum ClientError {
    IoError(io::Error),
    ClientError(String),
}

impl Display for ClientError {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        match self {
            ClientError::IoError(e) => e.fmt(f),
            ClientError::ClientError(s) => s.fmt(f),
        }
    }
}

impl std::error::Error for ClientError {}

impl From<io::Error> for ClientError {
    fn from(e: io::Error) -> Self {
        ClientError::IoError(e)
    }
}


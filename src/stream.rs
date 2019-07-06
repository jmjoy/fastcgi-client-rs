use std::fs::File;
use std::io::{self, Read, Write};
use std::net::TcpStream;

pub trait Stream: Sync + Send + Sized + Read + Write {
    fn try_clone(&self) -> Result<Self, io::Error>;
}

impl Stream for TcpStream {
    fn try_clone(&self) -> Result<Self, io::Error> {
        self.try_clone()
    }
}

#[cfg(unix)]
impl Stream for std::os::unix::net::UnixStream {
    fn try_clone(&self) -> Result<Self, io::Error> {
        self.try_clone()
    }
}

impl Stream for File {
    fn try_clone(&self) -> Result<Self, io::Error> {
        self.try_clone()
    }
}

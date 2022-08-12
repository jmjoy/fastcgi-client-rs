use tokio::{
    io::{AsyncRead, AsyncWrite},
    net::{self, TcpStream},
};

/// Connection mode, indicate is keep alive or not.
pub trait Mode {
    fn is_keep_alive() -> bool;
}

/// Short connection mode.
pub struct Short;

impl Mode for Short {
    fn is_keep_alive() -> bool {
        false
    }
}

/// Keep alive connection mode.
pub struct KeepAlive {}

impl Mode for KeepAlive {
    fn is_keep_alive() -> bool {
        true
    }
}

pub trait Split {
    type Read: AsyncRead + Unpin;

    type Write: AsyncWrite + Unpin;

    fn split(self) -> (Self::Read, Self::Write);
}

impl Split for TcpStream {
    type Read = net::tcp::OwnedReadHalf;
    type Write = net::tcp::OwnedWriteHalf;

    fn split(self) -> (Self::Read, Self::Write) {
        self.into_split()
    }
}

#[cfg(unix)]
impl Split for net::UnixStream {
    type Read = net::unix::OwnedReadHalf;
    type Write = net::unix::OwnedWriteHalf;

    fn split(self) -> (Self::Read, Self::Write) {
        self.into_split()
    }
}

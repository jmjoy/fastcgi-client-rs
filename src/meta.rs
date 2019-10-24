use crate::error::{ErrorKind, Result as ClientResult};
use crate::Params;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::cmp::min;
use std::collections::HashMap;

use tokio_io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use std::io;

use std::fmt::{self, Debug, Display};

use std::mem::size_of;
use std::ops::{Deref, DerefMut};

pub(crate) const VERSION_1: u8 = 1;
pub(crate) const MAX_LENGTH: usize = 0xffff;
pub(crate) const HEADER_LEN: usize = size_of::<Header>();

pub(crate) trait AsyncReadWrite: AsyncRead + AsyncWrite {}

impl<T> AsyncReadWrite for T where T: AsyncRead + AsyncWrite {}

#[derive(Debug, Clone)]
#[repr(u8)]
pub enum RequestType {
    BeginRequest = 1,
    AbortRequest = 2,
    EndRequest = 3,
    Params = 4,
    Stdin = 5,
    Stdout = 6,
    Stderr = 7,
    Data = 8,
    GetValues = 9,
    GetValuesResult = 10,
    UnknownType = 11,
}

impl RequestType {
    fn from_u8(u: u8) -> Self {
        match u {
            1 => RequestType::BeginRequest,
            2 => RequestType::AbortRequest,
            3 => RequestType::EndRequest,
            4 => RequestType::Params,
            5 => RequestType::Stdin,
            6 => RequestType::Stdout,
            7 => RequestType::Stderr,
            8 => RequestType::Data,
            9 => RequestType::GetValues,
            10 => RequestType::GetValuesResult,
            _ => RequestType::UnknownType,
        }
    }
}

impl Display for RequestType {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        Display::fmt(&(self.clone() as u8), f)
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Header {
    pub(crate) version: u8,
    pub(crate) r#type: RequestType,
    pub(crate) request_id: u16,
    pub(crate) content_length: u16,
    pub(crate) padding_length: u8,
    pub(crate) reserved: u8,
}

impl Header {
    pub(crate) async fn write_to_stream_batches<F, R, W>(
        r#type: RequestType,
        request_id: u16,
        writer: &mut W,
        content: &mut R,
        before_write: Option<F>,
    ) -> io::Result<()>
    where
        F: Fn(Header) -> Header,
        R: AsyncRead + Unpin,
        W: AsyncWrite + Unpin
    {
        let mut buf: [u8; MAX_LENGTH] = [0; MAX_LENGTH];
        let mut had_writen = false;

        loop {
            let read = content.read(&mut buf).await?;
            if had_writen && (read == 0 || read < MAX_LENGTH) {
                break;
            }

            let buf = &buf[..read];
            let mut header = Self::new(r#type.clone(), request_id, buf);
            if let Some(ref f) = before_write {
                header = f(header);
            }
            header.write_to_stream(writer, buf).await?;

            had_writen = true;
        }
        Ok(())
    }

    fn new(r#type: RequestType, request_id: u16, content: &[u8]) -> Self {
        let content_length = min(content.len(), MAX_LENGTH) as u16;
        Self {
            version: VERSION_1,
            r#type,
            request_id,
            content_length,
            padding_length: (-(content_length as i16) & 7) as u8,
            reserved: 0,
        }
    }

    async fn write_to_stream<T>(self, writer: &mut T, content: &[u8]) -> io::Result<()>
    where T: AsyncWrite + Unpin {
        let mut buf: Vec<u8> = Vec::new();
        buf.push(self.version);
        buf.push(self.r#type as u8);
        buf.write_u16::<BigEndian>(self.request_id)?;
        buf.write_u16::<BigEndian>(self.content_length)?;
        buf.push(self.padding_length);
        buf.push(self.reserved);

        writer.write_all(&buf).await?;
        writer.write_all(content).await?;
        writer.write_all(&vec![0; self.padding_length as usize]).await?;
        Ok(())
    }

    pub(crate) async fn new_from_stream<T>(reader: &mut T) -> io::Result<Self>
    where T: AsyncRead + Unpin {
        let mut buf: [u8; HEADER_LEN] = [0; HEADER_LEN];
        reader.read_exact(&mut buf).await?;

        Ok(Self {
            version: buf[0],
            r#type: RequestType::from_u8(buf[1]),
            request_id: (&buf[2..4]).read_u16::<BigEndian>()?,
            content_length: (&buf[4..6]).read_u16::<BigEndian>()?,
            padding_length: buf[6],
            reserved: buf[7],
        })
    }

    pub(crate) async fn read_content_from_stream<T>(&self, reader: &mut T) -> io::Result<Vec<u8>>
    where T: AsyncRead + Unpin {
        let mut buf = vec![0; self.content_length as usize];
        reader.read_exact(&mut buf).await?;
        let mut padding_buf = vec![0; self.padding_length as usize];
        reader.read_exact(&mut padding_buf).await?;
        Ok(buf)
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(u16)]
#[allow(dead_code)]
pub enum Role {
    Responder = 1,
    Authorizer = 2,
    Filter = 3,
}

#[derive(Debug)]
pub(crate) struct BeginRequest {
    pub(crate) role: Role,
    pub(crate) flags: u8,
    pub(crate) reserved: [u8; 5],
}

impl BeginRequest {
    pub(crate) fn new(role: Role, keep_alive: bool) -> Self {
        Self {
            role,
            flags: keep_alive as u8,
            reserved: [0; 5],
        }
    }

    pub(crate) fn to_content(&self) -> io::Result<Vec<u8>> {
        let mut buf: Vec<u8> = Vec::new();
        buf.write_u16::<BigEndian>(self.role as u16)?;
        buf.push(self.flags);
        buf.extend_from_slice(&self.reserved);
        Ok(buf)
    }
}

pub(crate) struct BeginRequestRec {
    pub(crate) header: Header,
    pub(crate) begin_request: BeginRequest,
    pub(crate) content: Vec<u8>,
}

impl BeginRequestRec {
    pub(crate) fn new(request_id: u16, role: Role, keep_alive: bool) -> io::Result<Self> {
        let begin_request = BeginRequest::new(role, keep_alive);
        let content = begin_request.to_content()?;
        let header = Header::new(RequestType::BeginRequest, request_id, &content);
        Ok(Self {
            header,
            begin_request,
            content,
        })
    }

    pub(crate) async fn write_to_stream<T>(self, writer: &mut T) -> io::Result<()>
        where T: AsyncWrite + Unpin {
        self.header.write_to_stream(writer, &self.content).await
    }
}

impl Debug for BeginRequestRec {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        Debug::fmt(
            &format!("BeginRequestRec {{header: {:?}, begin_request: {:?}}}", self.header, self.begin_request),
            f,
        )
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ParamLength {
    Short(u8),
    Long(u32),
}

impl ParamLength {
    pub fn new(length: usize) -> Self {
        if length < 128 {
            ParamLength::Short(length as u8)
        } else {
            let mut length = length;
            length |= 1 << 31;
            ParamLength::Long(length as u32)
        }
    }

    pub fn content(self) -> io::Result<Vec<u8>> {
        let mut buf: Vec<u8> = Vec::new();
        match self {
            ParamLength::Short(l) => buf.push(l),
            ParamLength::Long(l) => buf.write_u32::<BigEndian>(l)?,
        }
        Ok(buf)
    }
}

#[derive(Debug)]
pub struct ParamPair<'a> {
    name_length: ParamLength,
    value_length: ParamLength,
    name_data: &'a str,
    value_data: &'a str,
}

impl<'a> ParamPair<'a> {
    fn new(name: &'a str, value: &'a str) -> Self {
        let name_length = ParamLength::new(name.len());
        let value_length = ParamLength::new(value.len());
        Self {
            name_length,
            value_length,
            name_data: name,
            value_data: value,
        }
    }

    async fn write_to_stream<T>(&self, mut writer: T) -> io::Result<()>
        where T: AsyncWrite + Unpin {
        writer.write_all(&self.name_length.content()?).await?;
        writer.write_all(&self.value_length.content()?).await?;
        writer.write_all(self.name_data.as_bytes()).await?;
        writer.write_all(self.value_data.as_bytes()).await?;
        Ok(())
    }
}

#[derive(Debug)]
pub(crate) struct ParamPairs<'a>(Vec<ParamPair<'a>>);

impl<'a> ParamPairs<'a> {
    pub(crate) fn new(params: &Params<'a>) -> Self {
        let mut param_pairs = Vec::new();
        for (name, value) in params.iter() {
            let param_pair = ParamPair::new(name, value);
            param_pairs.push(param_pair);
        }

        Self(param_pairs)
    }

    pub(crate) async fn to_content(&self) -> io::Result<Vec<u8>> {
        let mut buf: Vec<u8> = Vec::new();

        for param_pair in self.iter() {
            param_pair.write_to_stream(&mut buf).await?;
        }

        Ok(buf)
    }
}

impl<'a> Deref for ParamPairs<'a> {
    type Target = Vec<ParamPair<'a>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> DerefMut for ParamPairs<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Debug)]
#[repr(u8)]
pub enum ProtocolStatus {
    RequestComplete = 0,
    CantMpxConn = 1,
    Overloaded = 2,
    UnknownRole = 3,
}

impl ProtocolStatus {
    pub fn from_u8(u: u8) -> Self {
        match u {
            0 => ProtocolStatus::RequestComplete,
            1 => ProtocolStatus::CantMpxConn,
            2 => ProtocolStatus::Overloaded,
            _ => ProtocolStatus::UnknownRole,
        }
    }

    pub(crate) fn convert_to_client_result(self, app_status: u32) -> ClientResult<()> {
        match self {
            ProtocolStatus::RequestComplete => Ok(()),
            _ => Err(ErrorKind::new_end_request_with_protocol_status(self, app_status).into()),
        }
    }
}

#[derive(Debug)]
pub struct EndRequest {
    pub(crate) app_status: u32,
    pub(crate) protocol_status: ProtocolStatus,
    reserved: [u8; 3],
}

#[derive(Debug)]
pub(crate) struct EndRequestRec {
    header: Header,
    pub(crate) end_request: EndRequest,
}

impl EndRequestRec {
    pub(crate) async fn from_header<T>(header: &Header, reader: &mut T) -> io::Result<Self>
        where T: AsyncRead + Unpin {
        let header = header.clone();
        let mut content = &*header.read_content_from_stream(reader).await?;
        let app_status = content.read_u32::<BigEndian>()?;
        let protocol_status = ProtocolStatus::from_u8(content.read_u8()?);
        let mut reserved: [u8; 3] = [0; 3];
        content.read_exact(&mut reserved).await?;

        Ok(Self {
            header,
            end_request: EndRequest {
                app_status,
                protocol_status,
                reserved,
            },
        })
    }
}

/// Fastcgi server address.
#[derive(Debug)]
pub enum Address<'a> {
    /// Tcp connection info.
    /// - `0` host.
    /// - `1` port.
    Tcp(&'a str, u16),
    #[cfg(unix)]
    /// Unix socket info.
    /// - `0` path of unix-sock file.
    UnixSock(&'a str),
}

pub(crate) type OutputMap = HashMap<u16, Output>;

/// Output of fastcgi request, contains STDOUT and STDERR.
#[derive(Default)]
pub struct Output {
    stdout: Option<Vec<u8>>,
    stderr: Option<Vec<u8>>,
}

impl Output {
    pub(crate) fn set_stdout(&mut self, stdout: Vec<u8>) {
        match self.stdout {
            Some(ref mut buf) => buf.extend(stdout.iter()),
            None => self.stdout = Some(stdout),
        }
    }

    pub(crate) fn set_stderr(&mut self, stderr: Vec<u8>) {
        match self.stderr {
            Some(ref mut buf) => buf.extend(stderr.iter()),
            None => self.stderr = Some(stderr),
        }
    }

    pub fn get_stdout(&self) -> Option<Vec<u8>> {
        self.stdout.clone()
    }

    pub fn get_stderr(&self) -> Option<Vec<u8>> {
        self.stderr.clone()
    }
}

impl Debug for Output {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        Debug::fmt(r#"Output { stdout: "...", stderr: "..." }"#, f)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_header_len() {
        assert_eq!(HEADER_LEN, 8);
    }
}

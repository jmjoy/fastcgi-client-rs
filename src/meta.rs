use std::mem::size_of;
use std::io::{self, Read, Write};
use std::collections::HashMap;
use byteorder::{WriteBytesExt, BigEndian};
use std::fs::hard_link;
use std::cmp::min;
use std::fmt::{self, Debug};
use std::convert::TryInto;

pub(crate) const VERSION_1: u8 = 1;
pub(crate) const MAX_LENGTH: usize = 0xffff;
pub(crate) const HEADER_LEN: usize = size_of::<Header>();

pub(crate) trait ReadWrite: Read + Write {}

impl<T> ReadWrite for T where T: Read + Write {}

#[derive(Debug)]
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
}

#[derive(Debug)]
pub(crate) struct Header {
    pub(crate)   version: u8,
    pub(crate)   r#type: RequestType,
    pub(crate)   request_id: u16,
    pub(crate)   content_length: u16,
    pub(crate)   padding_length: u8,
    pub(crate)   reserved: u8,
}

impl Header {
    fn write_to_stream_batches(r#type: RequestType, request_id: u16, writer: &mut Write, content: &mut Read) -> io::Result<()> {
        let mut buf: [u8; MAX_LENGTH] = [0; MAX_LENGTH];
        let readed = content.read(&mut buf)?;

        let buf = &buf[..readed];
        let header = Self::new(r#type, request_id, buf);
        header.write_to_stream(writer, buf)
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

    fn write_to_stream(self, writer: &mut Write, content: &[u8]) -> io::Result<()> {
        let mut buf: Vec<u8> = Vec::new();
        buf.push(self.version);
        buf.push(self.r#type as u8);
        buf.write_u16::<BigEndian>(self.request_id)?;
        buf.write_u16::<BigEndian>(self.content_length)?;
        buf.push(self.padding_length);
        buf.push(self.reserved);

        writer.write_all(&buf)?;
        writer.write_all(content)?;
        writer.write_all(&vec![0; self.padding_length as usize]);
        Ok(())
    }
}

#[derive(Debug)]
struct Record<'a> {
    header: Header,
    content: &'a [u8],
}

impl<'a> Record<'a> {}

#[derive(Debug, Clone, Copy)]
#[repr(u16)]
pub enum Role {
    Responder = 1,
    Authorizer = 2,
    Filter = 3,
}

#[derive(Debug)]
pub(crate) struct BeginRequest {
    pub(crate)   role: Role,
    pub(crate)   flags: u8,
    pub(crate)   reserved: [u8; 5],
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

    pub(crate) fn write_to_stream(self, writer: &mut Write) -> io::Result<()> {
        self.header.write_to_stream(writer, &self.content)
    }
}

impl Debug for BeginRequestRec {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        Debug::fmt(&format!("BeginRequestRec {{header: {:?}, begin_request: {:?}}}", self.header, self.begin_request), f)
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

#[derive(Debug)]
pub struct EndRequest {
    app_status: u32,
    protocol_status: ProtocolStatus,
    reserved: [u8; 3],
}

struct EndRequestRec {
    header: Header,
    end_request: EndRequest,
}

#[derive(Debug)]
pub enum Address<'a> {
    Tcp(&'a str, u16),
    UnixSock(&'a str),
}

#[derive(Debug)]
struct Response {
    version: u8,
    typ: u8,
    request_id: u16,
    content_length: u16,
    padding_length: u8,
    reserved: u8,
    content: Vec<u8>,
}

pub(crate) type OutputMap = HashMap<u16, Output>;

pub struct Output {
    stdout: Box<Read>,
    stderr: Box<Read>,
}

impl Debug for Output {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        Debug::fmt(r#"Output {{ stdout: "", output: "" }}"#, f)
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


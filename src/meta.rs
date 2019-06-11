use std::mem::size_of;
use std::io::{self, Read, Write};
use std::collections::HashMap;
use byteorder::BigEndian;
use std::fs::hard_link;

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
    pub fn new(r#type: RequestType, request_id: u16, content: &[u8]) -> Self {
        Self {
            version: VERSION_1,
            r#type,
            request_id,
            content_length:
        }
    }

    pub fn write_to_stream(self, writer: &mut Write, content: &[u8]) -> io::Result<()> {
        let mut buf: Vec<u8> = Vec::new();
        buf.push(self.version);
        buf.push(self.r#type as u8);
        buf.write_u16::<BigEndian>(self.request_id)?;
        buf.write_u16::<BigEndian>(self.content_length)?;
        buf.push(self.padding_length);
        buf.push(self.reserved);

        writer.write_all(&buf)?;
        writer.write_all(content)?;
        writer.write_all(&[0; self.padding_length]);
        Ok(())
    }
}

#[derive(Debug)]
pub struct Record<'a> {
    header: Header,
    content: &'a [u8],
}

impl Record {
}

#[derive(Debug)]
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

impl Into<Vec<u8>> for BeginRequest {
    fn into(self) -> Vec<u8> {
        let mut buf: Vec<u8> = Vec::new();
        buf.write_u16::<BigEndian>(self.role as u16)?;
        buf.push(self.flags);
        buf.extend_from_slice(&self.reserved);
        buf
    }
}

#[derive(Debug)]
pub(crate) struct BeginRequestRec {
    pub(crate) header: Header,
    pub(crate) begin_request: BeginRequest,
}

impl BeginRequestRec {
    pub(crate) fn write_to_stream(self, writer: &mut Write) -> io::Result<()> {
        self.header.write_to_stream(writer, self.begin_request.into())
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

#[allow(dead_code)]
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_header_len() {
        assert_eq!(HEADER_LEN, 8);
    }
}

use std::mem::size_of;
use std::io::{Read, Write};

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
pub struct Header {
    version: u8,
    r#type: RequestType,
    request_id: u16,
    content_length: u16,
    padding_length: u8,
    reserved: u8,
}

#[derive(Debug)]
pub struct Record<'a> {
    header: Header,
    content_data: &'a [u8],
    padding_data: &'a [u8],
}

#[derive(Debug)]
#[repr(u16)]
pub enum Role {
    Responder = 1,
    Authorizer = 2,
    Filter = 3,
}

#[derive(Debug)]
pub struct BeginRequest {
    role: Role,
    flags: u8,
    reserved: [u8; 5],
}

#[derive(Debug)]
struct BeginRequestRec {
    header: Header,
    begin_request: BeginRequest,
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_header_len() {
        assert_eq!(HEADER_LEN, 8);
    }
}

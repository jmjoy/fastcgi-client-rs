use crate::Params;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::cmp::min;
use std::collections::HashMap;
use std::convert::TryInto;
use std::fmt::{self, Debug};
use std::fs::hard_link;
use std::io::{self, Read, Write};
use std::mem::size_of;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;

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

#[derive(Debug)]
pub(crate) struct Header {
    pub(crate) version: u8,
    pub(crate) r#type: RequestType,
    pub(crate) request_id: u16,
    pub(crate) content_length: u16,
    pub(crate) padding_length: u8,
    pub(crate) reserved: u8,
}

impl Header {
    pub(crate) fn write_to_stream_batches<F>(
        r#type: RequestType,
        request_id: u16,
        writer: &mut Write,
        content: &mut Read,
        before_write: Option<F>,
    ) -> io::Result<()>
    where
        F: Fn(Header) -> Header,
    {
        let mut buf: [u8; MAX_LENGTH] = [0; MAX_LENGTH];
        let read = content.read(&mut buf)?;

        let buf = &buf[..read];
        let mut header = Self::new(r#type, request_id, buf);
        if let Some(before_write) = before_write {
            header = before_write(header);
        }
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

    pub(crate) fn new_from_stream(reader: &mut Read) -> io::Result<Self> {
        let mut buf: [u8; HEADER_LEN] = [0; HEADER_LEN];
        reader.read_exact(&mut buf)?;

        Ok(Self {
            version: buf[0],
            r#type: RequestType::from_u8(buf[1]),
            request_id: (&buf[2..4]).read_u16::<BigEndian>()?,
            content_length: (&buf[4..6]).read_u16::<BigEndian>()?,
            padding_length: buf[6],
            reserved: buf[7],
        })
    }

    pub(crate) fn read_content_from_stream(self, reader: &mut Read) -> io::Result<Vec<u8>> {
        let mut buf = vec![0; self.content_length as usize];
        reader.read_exact(&mut buf)?;
        let mut padding_buf = vec![0; self.padding_length as usize];
        reader.read_exact(&mut padding_buf)?;
        Ok(buf)
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(u16)]
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

    pub(crate) fn write_to_stream(self, writer: &mut Write) -> io::Result<()> {
        self.header.write_to_stream(writer, &self.content)
    }
}

impl Debug for BeginRequestRec {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        Debug::fmt(
            &format!(
                "BeginRequestRec {{header: {:?}, begin_request: {:?}}}",
                self.header, self.begin_request
            ),
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

    fn write_to_stream(&self, writer: &mut Write) -> io::Result<()> {
        writer.write_all(&self.name_length.content()?)?;
        writer.write_all(&self.value_length.content()?)?;
        writer.write_all(self.name_data.as_bytes())?;
        writer.write_all(self.value_data.as_bytes())?;
        Ok(())
    }
}

pub struct ParamsRec<'a> {
    pub(crate) header: Header,
    pub(crate) param_pairs: Vec<ParamPair<'a>>,
    pub(crate) content: Vec<u8>,
}

impl<'a> ParamsRec<'a> {
    pub fn new(request_id: u16, params: &Params<'a>) -> io::Result<Self> {
        let mut buf: Vec<u8> = Vec::new();
        let mut param_pairs = Vec::new();
        for (name, value) in params.iter() {
            let param_pair = ParamPair::new(name, value);
            param_pair.write_to_stream(&mut buf);
            param_pairs.push(param_pair);
        }

        let header = Header::new(RequestType::Params, request_id, &buf);

        Ok(Self {
            header,
            param_pairs,
            content: buf,
        })
    }

    pub(crate) fn write_to_stream(self, writer: &mut Write) -> io::Result<()> {
        self.header.write_to_stream(writer, &self.content)
    }
}

impl<'a> Debug for ParamsRec<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        Debug::fmt(
            &format!(
                "ParamsRec {{header: {:?}, param_pairs: {:?}}}",
                self.header, self.param_pairs
            ),
            f,
        )
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

#[derive(Default)]
pub struct Output {
    stdout: Option<Vec<u8>>,
    stderr: Option<Vec<u8>>,
}

impl Output {
    pub(crate) fn set_stdout(&mut self, stdout: Vec<u8>) {
        self.stdout = Some(stdout);
    }

    pub(crate) fn set_stderr(&mut self, stderr: Vec<u8>) {
        self.stderr = Some(stderr);
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
        Debug::fmt(&format!(r#"Output {{ stdout: "...", stderr: "..." }}"#), f)
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

const VERSION_1: u8 = 1;
const MAX_LENGTH: usize = 0xffff;
const KEEP_CONN: u8 = 1;
pub const HEADER_LEN: usize = size_of::<Header>();

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
struct Record<'a> {
    header: Header,
    content_data: &'a [u8],
    padding_data: &'a [u8],
}

#[derive(Debug)]
struct BeginRequest {
    role: u16,
    flags: u8,
    reserved: [u8; 5],
}

#[derive(Debug)]
#[allow(dead_code)]
struct BeginRequestRec {
    header: Header,
    begin_request: BeginRequest,
}

#[derive(Debug)]
struct EndRequest {
    app_status: u32,
    protocol_status: u8,
    reserved: [u8; 3],
}

#[allow(dead_code)]
pub struct EndRequestRec {
    header: Header,
    end_request: EndRequest,
}

trait ReadWrite: Read + Write {}

impl<T> ReadWrite for T where T: Read + Write {}




#[derive(Debug)]
pub enum Address<'a> {
    Tcp(&'a str, u16),
    UnixSock(&'a str),
}

#[derive(Debug)]
pub struct Response {
    version: u8,
    typ: u8,
    request_id: u16,
    content_length: u16,
    padding_length: u8,
    reserved: u8,
    content: Vec<u8>,
}


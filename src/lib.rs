use byteorder::{BigEndian, WriteBytesExt, ReadBytesExt};
use std::cmp::min;
use std::collections::HashMap;
use std::io::{self, ErrorKind, Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::{Duration, SystemTime};
use std::rc::Rc;
use std::cell::RefCell;

const VERSION_1: u8 = 1;

const BEGIN_REQUEST: u8 = 1;
const ABORT_REQUEST: u8 = 2;
const END_REQUEST: u8 = 3;
const PARAMS: u8 = 4;
const STDIN: u8 = 5;
const STDOUT: u8 = 6;
const STDERR: u8 = 7;
const DATA: u8 = 8;
const GET_VALUES: u8 = 9;
const GET_VALUES_RESULT: u8 = 10;
const UNKNOWN_TYPE: u8 = 11;
const MAXTYPE: u8 = UNKNOWN_TYPE;

const RESPONDER: u8 = 1;
const AUTHORIZER: u8 = 2;
const FILTER: u8 = 3;

const REQUEST_COMPLETE: u8 = 0;
const CANT_MPX_CONN: u8 = 1;
const OVERLOADED: u8 = 2;
const UNKNOWN_ROLE: u8 = 3;

const MAX_CONNS: &'static str = "MAX_CONNS";
const MAX_REQS: &'static str = "MAX_REQS";
const MPXS_CONNS: &'static str = "MPXS_CONNS";

const HEADER_LEN: usize = 8;

pub trait ReadWrite: Read + Write {}

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

pub struct ClientBuilder<'a> {
    address: Address<'a>,
    connect_timeout: Option<Duration>,
    read_timeout: Option<Duration>,
    write_timeout: Option<Duration>,
    keep_alive: bool,
}

impl<'a> ClientBuilder<'a> {
    pub fn new(address: Address<'a>) -> Self {
        Self {
            address,
            connect_timeout: Some(Duration::from_secs(30)),
            read_timeout: Some(Duration::from_secs(30)),
            write_timeout: Some(Duration::from_secs(30)),
            keep_alive: false,
        }
    }

    pub fn set_connect_timeout(mut self, connect_timeout: Option<Duration>) -> Self {
        self.connect_timeout = connect_timeout;
        self
    }

    pub fn set_read_timeout(mut self, read_timeout: Option<Duration>) -> Self {
        self.read_timeout = read_timeout;
        self
    }

    pub fn set_write_timeout(mut self, write_timeout: Option<Duration>) -> Self {
        self.write_timeout = write_timeout;
        self
    }

    pub fn set_read_write_timeout(self, timeout: Option<Duration>) -> Self {
        self.set_read_timeout(timeout).set_write_timeout(timeout)
    }

    pub fn set_keep_alive(mut self, keep_alive: bool) -> Self {
        self.keep_alive = keep_alive;
        self
    }

    pub fn build(self) -> Result<Client<'a>, failure::Error> {
        let stream = match self.address {
            Address::Tcp(host, port) => match self.connect_timeout {
                Some(connect_timeout) => {
                    let addr = (host, port).to_socket_addrs()?.next()
                        .unwrap();
//                        .ok_or_else(|| {
//                            failure::Error::new(
//                                ErrorKind::NotFound,
//                                "This should not happen, but if it happen, \
//                                 it means that your address is incorrect.",
//                            )
//                        }
//                        )
//                    ?;
                    TcpStream::connect_timeout(&addr, connect_timeout)?
                }
                None => TcpStream::connect((host, port))?,
            },
            Address::UnixSock(_path) => unimplemented!(),
        };

        Ok(Client {
            builder: self,
            stream: Box::new(stream),
            response_buf: Vec::new(),
        })
    }
}

#[derive(Default, Debug)]
pub struct Params<'a> {
    pub gateway_interface: &'a str,
    pub request_method: &'a str,
    pub script_filename: &'a str,
    pub script_name: &'a str,
    pub query_string: &'a str,
    pub request_uri: &'a str,
    pub document_uri: &'a str,
    pub server_software: &'a str,
    pub remote_addr: &'a str,
    pub remote_port: &'a str,
    pub server_addr: &'a str,
    pub server_port: &'a str,
    pub server_name: &'a str,
    pub server_protocol: &'a str,
    pub content_type: &'a str,
    pub content_length: &'a str,
}

impl<'a> Params<'a> {
    pub fn with(
        request_method: &'a str,
        script_name: &'a str,
        query_string: &'a str,
        request_uri: &'a str,
        document_uri: &'a str,

        remote_addr: &'a str,
        remote_port: &'a str,
        server_addr: &'a str,
        server_port: &'a str,
        server_name: &'a str,
        content_type: &'a str,
        content_length: &'a str,
    ) -> Self {
        let mut params: Params = Default::default();

        params.request_method = request_method;
        params.script_name = script_name;
        params.query_string = query_string;
        params.request_uri = request_uri;
        params.document_uri = document_uri;
        params.remote_addr = remote_addr;
        params.remote_port = remote_port;
        params.server_addr = server_addr;
        params.server_port = server_port;
        params.server_name = server_name;
        params.content_type = content_type;
        params.content_length = content_length;

        params.gateway_interface = "FastCGI/1.0";
        params.server_software = "rust/fastcgi-client";
        params.server_protocol = "HTTP/1.1";

        params
    }
}

impl<'a> Into<HashMap<&'a str, &'a str>> for Params<'a> {
    fn into(self) -> HashMap<&'a str, &'a str> {
        let mut map = HashMap::new();
        map.insert("GATEWAY_INTERFACE", self.gateway_interface);
        map.insert("REQUEST_METHOD", self.request_method);
        map.insert("SCRIPT_FILENAME", self.script_name);
        map.insert("SCRIPT_NAME", self.script_name);
        map.insert("QUERY_STRING", self.query_string);
        map.insert("REQUEST_URI", self.request_uri);
        map.insert("DOCUMENT_URI", self.document_uri);
        map.insert("SERVER_SOFTWARE", self.server_software);
        map.insert("REMOTE_ADDR", self.remote_addr);
        map.insert("REMOTE_PORT", self.remote_port);
        map.insert("SERVER_ADDR", self.server_addr);
        map.insert("SERVER_PORT", self.server_port);
        map.insert("SERVER_NAME", self.server_name);
        map.insert("SERVER_PROTOCOL", self.server_protocol);
        map.insert("CONTENT_TYPE", self.content_type);
        map.insert("CONTENT_LENGTH", self.content_length);
        map
    }
}

pub struct Client<'a> {
    builder: ClientBuilder<'a>,
    stream: Box<ReadWrite>,
    response_buf: Vec<u8>,
}

impl<'a> Client<'a> {
    pub fn request(mut self, params: Params<'a>, input: &mut Read) -> Result<Vec<u8>, failure::Error> {
        let id = self.do_request(params, input)?;
        self.do_response(id)?;
        Ok(self.response_buf)
    }

    fn do_request(&mut self, params: Params<'a>, input: &mut Read) -> Result<u16, failure::Error> {
        let id = Self::generate_request_id();
        dbg!(id);
        let keep_alive = self.builder.keep_alive as u8;
        let mut request_buf = Self::build_packet(
            BEGIN_REQUEST,
            &vec![0, RESPONDER, keep_alive, 0, 0, 0, 0, 0],
            id,
        )?;
        dbg!(&request_buf);
        let mut params_buf: Vec<u8> = Vec::new();
        let params: HashMap<&'a str, &'a str> = params.into();
        for (k, v) in params {
            params_buf.write_all(&Self::build_nv_pair(k, v)?);
        }
        if params_buf.len() > 0 {
            request_buf.write_all(&params_buf)?;
        }

        let mut input_buf: Vec<u8> = Vec::new();
        io::copy(input, &mut input_buf)?;
        if input_buf.len() > 0 {
            request_buf.write_all(&Self::build_packet(STDIN, &input_buf, id)?)?;
        }
        request_buf.write_all(&Self::build_packet(STDIN, &vec![], id)?)?;

        dbg!(&request_buf);

        self.stream.write_all(&request_buf)?;


        Ok(id)
    }

    fn do_response(&mut self, request_id: u16) -> Result<(), failure::Error> {
        loop {
            let response = match self.read_packet() {
                Ok(response) => response,
                Err(e) => {
//                    if e.kind() == ErrorKind::UnexpectedEof {
//                        break;
//                    }
                    return Err(e);
                },
            };

            match response.typ {
                STDOUT => {
                    self.response_buf.write_all(&response.content)?;
                }
                STDERR => {
                    unreachable!();
//                    return Err(failure::Error::new(ErrorKind::InvalidData, "Response type is STDERR."));
                }
                END_REQUEST => if response.request_id != request_id {
                    break;
                }
                _ => {
                    unreachable!();
//                    return Err(failure::Error::new(ErrorKind::InvalidData, "Response type unknown."));
                }
            }
        }

        Ok(())
//        match self.response_buf[4] {
//            CANT_MPX_CONN => Err(failure::Error::new(ErrorKind::Other, "This app can't multiplex [CANT_MPX_CONN]")),
//            OVERLOADED => Err(failure::Error::new(ErrorKind::Other, "New request rejected; too busy [OVERLOADED]")),
//            UNKNOWN_ROLE => Err(failure::Error::new(ErrorKind::Other, "Role value not known [UNKNOWN_ROLE]")),
//            REQUEST_COMPLETE=> Ok(()),
//            _ => Err(failure::Error::new(ErrorKind::InvalidData, "Unexpected value of content[4]"))
//        }
    }

    fn generate_request_id() -> u16 {
        match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
            Ok(duration) => (duration.as_secs() % 65535) as u16 + 1,
            Err(_) => 1,
        }
    }

    fn build_packet(typ: u8, content: &[u8], request_id: u16) -> Result<Vec<u8>, failure::Error> {
        let len = content.len();
        // TODO Now just limit 2^16 lengths content, I will optimize it later version.
        let len = min(len, 65535) as u16;

        let mut buf: Vec<u8> = Vec::new();
        buf.push(VERSION_1);
        buf.push(typ);
        buf.write_u16::<BigEndian>(request_id)?;
        buf.write_u16::<BigEndian>(len)?;
        buf.push(0);
        buf.push(0);
        buf.write_all(&content[..len as usize])?;
        Ok(buf)
    }

    fn build_nv_pair<'b>(name: &'b str, value: &'b str) -> Result<Vec<u8>, failure::Error> {
        let mut buf = Vec::new();

        let mut n_len = name.len() as u32;
        let mut v_len = value.len() as u32;

        if n_len < 128 {
            buf.write_u8(n_len as u8)?;
        } else {
            n_len |= 1 << 31;
            buf.write_u32::<BigEndian>(n_len)?;
        }

        if v_len < 128 {
            buf.write_u8(v_len as u8)?;
        } else {
            v_len |= 1 << 31;
            buf.write_u32::<BigEndian>(v_len)?;
        }

        buf.write_all(name.as_bytes())?;
        buf.write_all(value.as_bytes())?;

        Ok(buf)
    }

    fn read_packet(&mut self) -> Result<Response, failure::Error> {
        let mut buf: [u8; HEADER_LEN] = [0; HEADER_LEN];
        self.stream.read_exact(&mut buf)?;
        let mut response = self.decode_packet_header(&buf)?;

        if response.content_length > 0 {
            let mut buf: Vec<u8> = vec![0; response.content_length as usize];
            self.stream.read_exact(&mut buf)?;
            response.content.write_all(&mut buf)?;
        }
        if response.padding_length > 0 {
            let mut buf: Vec<u8> = vec![0; response.padding_length as usize];
            self.stream.read_exact(&mut buf)?;
        }

        Ok(response)
    }

    fn decode_packet_header(&mut self, buf: &[u8; HEADER_LEN]) -> Result<Response, failure::Error> {
        let mut response = Response {
            version: buf[0],
            typ: buf[1],
            request_id: (&buf[2..4]).read_u16::<BigEndian>()?,
            content_length: (&buf[4..6]).read_u16::<BigEndian>()?,
            padding_length: buf[6],
            reserved: buf[7],
            content: Vec::new(),
        };

        Ok(response)
    }

}

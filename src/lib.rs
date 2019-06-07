use std::collections::HashMap;
use std::io::{self, ErrorKind, Read};
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::time::Duration;

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

const HEADER_LEN: u8 = 8;

pub enum Address<'a> {
    Tcp(&'a str, u16),
    UnixSock(&'a str),
}

pub struct ClientBuilder<'a> {
    address: Address<'a>,
    connect_timeout: Option<Duration>,
    read_timeout: Option<Duration>,
    write_timeout: Option<Duration>,
}

impl<'a> ClientBuilder<'a> {
    pub fn new(address: Address<'a>) -> Self {
        Self {
            address,
            connect_timeout: Some(Duration::from_secs(30)),
            read_timeout: Some(Duration::from_secs(30)),
            write_timeout: Some(Duration::from_secs(30)),
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

    pub fn build(self) -> Result<Client<'a>, io::Error> {
        let stream = match self.address {
            Address::Tcp(host, port) => match self.connect_timeout {
                Some(connect_timeout) => {
                    let addr = (host, port).to_socket_addrs()?.next().ok_or_else(|| {
                        io::Error::new(
                            ErrorKind::NotFound,
                            "This should not happen, but if it happen, \
                             it means that your address is incorrect.",
                        )
                    })?;
                    TcpStream::connect_timeout(&addr, connect_timeout)?
                }
                None => TcpStream::connect((host, port))?,
            },
            Address::UnixSock(_path) => unimplemented!(),
        };
        Ok(Client {
            builder: self,
            stream: Box::new(stream),
        })
    }
}

pub struct Params {
    //    gateway_interface: &'a str,
//    request_method: &'a str,
//    script_filename: &'a str,
//    script_name: &'a str,
//    query_string: &'a str,
//    request_uri: &'a str,
//    document_uri: &'a str,
//    server_software: &'a str,
//    remote_addr: &'a str,
//    remote_port: &'a str,
//    server_addr: &'a str,
//    server_port: &'a str,
//    server_name: &'a str,
//    server_protocol: &'a str,
//    content_type: &'a str,
//    content_length: &'a str,
}

impl Params {
    pub fn create_params_map<'a>(
        request_method: &'a str,
        script_name: &'a str,
    ) -> HashMap<&'a str, &'a str> {
        let map = HashMap::new();
        map

        //        "GATEWAY_INTERFACE" => "FastCGI/1.0",
        //        "REQUEST_METHOD"    => request_method,
        //        "SCRIPT_FILENAME"   => script_name,
        //        "SCRIPT_NAME"       => $req,
        //        "QUERY_STRING"      => $url["query"],
        //        "REQUEST_URI"       => $uri,
        //        "DOCUMENT_URI"      => $req,
        //        "SERVER_SOFTWARE"   => "php/fcgiclient",
        //        "REMOTE_ADDR"       => "127.0.0.1",
        //        "REMOTE_PORT"       => "9985",
        //        "SERVER_ADDR"       => "127.0.0.1",
        //        "SERVER_PORT"       => "80",
        //        "SERVER_NAME"       => php_uname("n"),
        //        "SERVER_PROTOCOL"   => "HTTP/1.1",
        //        "CONTENT_TYPE"      => "",
        //        "CONTENT_LENGTH"    => 0
    }
}

pub struct Client<'a> {
    builder: ClientBuilder<'a>,
    stream: Box<Read>,
}

impl<'a> Client<'a> {
    pub fn request() {}
}

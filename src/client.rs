use std::time::Duration;
use std::io::{self, ErrorKind, Result, Read, Write};
use crate::{ClientResult, ClientError};
use byteorder::BigEndian;
use std::net::TcpStream;
use crate::meta::{VERSION_1, Address, OutputMap, Output, BeginRequestRec, Header, BeginRequest, RequestType, ReadWrite, Role,
    ParamsRec
};
use std::collections::HashMap;
use crate::id::RequestIdGenerator;
use log::{debug, info};
use crate::params::Params;
use std::net::ToSocketAddrs as _;

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

    pub fn build(self) -> io::Result<Client<'a>> {
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
            outputs: HashMap::new(),
        })
    }
}


pub struct Client<'a> {
    builder: ClientBuilder<'a>,
    stream: Box<ReadWrite>,
    outputs: OutputMap,
}

impl<'a> Client<'a> {
    pub fn do_request(&mut self, params: &Params<'a>, body: &mut Read) -> ClientResult<&mut Output> {
        let id = RequestIdGenerator.generate();
        self.handle_request(id, params, body)?;
        self.handle_response(id)?;
        Ok(self.outputs.get_mut(&id).ok_or(ClientError::RequestIdNotFound(id))?)
    }

    fn handle_request(&mut self, id: u16, params: &Params<'a>, body: &mut Read) -> ClientResult<()> {
        info!("[id = {}] Start handle request.", id);

        let begin_request_rec = BeginRequestRec::new(id, Role::Responder, self.builder.keep_alive)?;
        info!("[id = {}] Send to stream: {:?}.", id, &begin_request_rec);
        begin_request_rec.write_to_stream(&mut self.stream)?;

        let params_rec = ParamsRec::new(id, params)?;
        info!("[id = {}] Send to stream: {:?}.", id, &params_rec);
        params_rec.write_to_stream(&mut self.stream)?;

        Header::write_to_stream_batches(RequestType::Stdin, id, &mut self.stream, body, Some(|header| {
            info!("[id = {}] Send to stream for Stdin: {:?}.", id, &header);
            header
        }))?;

//        let mut params_buf: Vec<u8> = Vec::new();
//        let params: HashMap<&'a str, &'a str> = params.into();
//        for (k, v) in params {
//            params_buf.write_all(&Self::build_nv_pair(k, v)?)?;
//        }
//        if params_buf.len() > 0 {
//            let buf = &Self::build_packet(TYPE_PARAMS, &params_buf, id)?;
//            request_buf.write_all(buf)?;
//            info!("[id = {}] Sended PARAMS: {:?}", id, buf);
//        }
//
//        let mut input_buf: Vec<u8> = Vec::new();
//        io::copy(body, &mut input_buf)?;
//        if input_buf.len() > 0 {
//            let buf = &Self::build_packet(TYPE_STDIN, &input_buf, id)?;
//            request_buf.write_all(buf)?;
//            info!("[id = {}] Sended STDIN: {:?}", id, buf);
//        }
//        request_buf.write_all(&Self::build_packet(TYPE_STDIN, &vec![], id)?)?;
//
//        self.stream.write_all(&request_buf)?;
//
//        debug!("临时");

        Ok(())
    }

    fn handle_response(&mut self, request_id: u16) -> ClientResult<()> {
        loop {
            break;
//            let response = match self.read_packet() {
//                Ok(response) => response,
//                Err(e) => {
//                    //                    if e.kind() == ErrorKind::UnexpectedEof {
//                    //                        break;
//                    //                    }
//                    return Err(e.into());
//                }
//            };

//            info!("[id = {}] Read response packet: {:?}", request_id, response);
//
//            match response.typ {
//                TYPE_STDOUT => {
//                    self.response_buf.write(&response.content)?;
//                    info!("[id = {}] Write to response buffer: {:?}", request_id, &self.response_buf);
//                }
//                TYPE_STDERR => {
//                    self.response_buf.write(&response.content)?;
//                    info!("[id = {}] Write to response buffer (HAS STDERROR): {:?}", request_id, &self.response_buf);
////                    return Err(io::Error::new(
////                        ErrorKind::InvalidData,
////                        "Response type is STDERR.",
////                    ));
//                }
//                TYPE_END_REQUEST => {
//                    if response.request_id == request_id {
//                        info!("[id = {}] End of request", request_id);
//                        break;
//                    }
//                }
//                _ => {
//                    return Err(io::Error::new(ErrorKind::InvalidData, "Response type unknown.").into());
//                }
//            }
        }
//
//        info!("[id = {}] Finish response, buf: {:?}", request_id, &self.response_buf);

            Ok(())

//        match self.response_buf[4] {
//            STATUS_CANT_MPX_CONN => Err(io::Error::new(ErrorKind::Other, "This app can't multiplex [CantMpxConn]")),
//            STATUS_OVERLOADED => Err(io::Error::new(ErrorKind::Other, "New request rejected; too busy [OVERLOADED]")),
//            STATUS_UNKNOWN_ROLE => Err(io::Error::new(ErrorKind::Other, "Role value not known [UnknownRole]")),
//            STATUS_REQUEST_COMPLETE=> Ok(()),
//            _ => Err(io::Error::new(ErrorKind::InvalidData, "Unexpected value of content[4]"))
//        }
        }

//    fn build_packet(typ: u8, content: &[u8], request_id: u16) -> Result<Vec<u8>, ClientError> {
//        let len = content.len();
//        // TODO Now just limit 2^16 lengths content, I will optimize it later version.
//        let len = min(len, 65535) as u16;
//
//        let mut buf: Vec<u8> = Vec::new();
//        buf.push(VERSION_1);
//        buf.push(typ);
//        buf.write_u16::<BigEndian>(request_id)?;
//        buf.write_u16::<BigEndian>(len)?;
//        buf.push(0);
//        buf.push(0);
//        buf.write_all(&content[..len as usize])?;
//        Ok(buf)
//    }
//
//    fn build_nv_pair<'b>(name: &'b str, value: &'b str) -> Result<Vec<u8>, ClientError> {
//        let mut buf = Vec::new();
//
//        let mut n_len = name.len() as u32;
//        let mut v_len = value.len() as u32;
//
//        if n_len < 128 {
//            buf.write_u8(n_len as u8)?;
//        } else {
//            n_len |= 1 << 31;
//            buf.write_u32::<BigEndian>(n_len)?;
//        }
//
//        if v_len < 128 {
//            buf.write_u8(v_len as u8)?;
//        } else {
//            v_len |= 1 << 31;
//            buf.write_u32::<BigEndian>(v_len)?;
//        }
//
//        buf.write_all(name.as_bytes())?;
//        buf.write_all(value.as_bytes())?;
//
//        Ok(buf)
//    }
//
//    fn read_packet(&mut self) -> io::Result<Response, io::Error> {
//        let mut buf: [u8; HEADER_LEN] = [0; HEADER_LEN];
//        self.stream.read_exact(&mut buf)?;
//        let mut response = self.decode_packet_header(&buf)?;
//
//        if response.content_length > 0 {
//            let mut buf: Vec<u8> = vec![0; response.content_length as usize];
//            self.stream.read_exact(&mut buf)?;
//            response.content.write_all(&mut buf)?;
//        }
//        if response.padding_length > 0 {
//            let mut buf: Vec<u8> = vec![0; response.padding_length as usize];
//            self.stream.read_exact(&mut buf)?;
//        }
//
//        Ok(response)
//    }
//
//    fn decode_packet_header(&mut self, buf: &[u8; HEADER_LEN]) -> io::Result<Response> {
//        let mut response = Response {
//            version: buf[0],
//            typ: buf[1],
//            request_id: (&buf[2..4]).read_u16::<BigEndian>()?,
//            content_length: (&buf[4..6]).read_u16::<BigEndian>()?,
//            padding_length: buf[6],
//            reserved: buf[7],
//            content: Vec::new(),
//        };
//
//        Ok(response)
//    }
    }

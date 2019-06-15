use crate::id::RequestIdGenerator;
use crate::meta::{Address, BeginRequestRec, EndRequestRec, Header, Output, OutputMap, ParamPairs, ReadWrite, RequestType, Role};
use crate::params::Params;
use crate::{ClientError, ClientResult};

use log::info;
use std::collections::HashMap;
use std::io::{self, ErrorKind, Read};
use std::net::TcpStream;
use std::net::ToSocketAddrs as _;

use std::time::Duration;

#[cfg(unix)]
use std::os::unix::net::UnixStream;

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
        let stream: Box<ReadWrite> = match self.address {
            Address::Tcp(host, port) => {
                let stream = match self.connect_timeout {
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
                };
                stream.set_read_timeout(self.read_timeout)?;
                stream.set_write_timeout(self.write_timeout)?;
                Box::new(stream)
            }
            Address::UnixSock(path) => {
                if cfg!(unix) {
                    let stream = UnixStream::connect(path)?;
                    Box::new(stream)
                } else {
                    panic!("Unix socket not support for your operate system.")
                }
            }
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
        Ok(self.outputs.get_mut(&id).ok_or_else(|| ClientError::RequestIdNotFound(id))?)
    }

    fn handle_request(&mut self, id: u16, params: &Params<'a>, body: &mut Read) -> ClientResult<()> {
        info!("[id = {}] Start handle request.", id);

        let begin_request_rec = BeginRequestRec::new(id, Role::Responder, self.builder.keep_alive)?;
        info!("[id = {}] Send to stream: {:?}.", id, &begin_request_rec);
        begin_request_rec.write_to_stream(&mut self.stream)?;

        dbg!(params);
        let param_pairs = ParamPairs::new(params);
        info!("[id = {}] Params will be sent: {:?}.", id, &param_pairs);

        Header::write_to_stream_batches(
            RequestType::Params,
            id,
            &mut self.stream,
            &mut &param_pairs.to_content()?[..],
            Some(|header| {
                info!("[id = {}] Send to stream for Params: {:?}.", id, &header);
                header
            }),
        )?;

        Header::write_to_stream_batches(
            RequestType::Stdin,
            id,
            &mut self.stream,
            body,
            Some(|header| {
                info!("[id = {}] Send to stream for Stdin: {:?}.", id, &header);
                header
            }),
        )?;

        Ok(())
    }

    fn handle_response(&mut self, id: u16) -> ClientResult<()> {
        self.init_output(id);

        let global_end_request_rec: Option<EndRequestRec>;

        loop {
            let header = Header::new_from_stream(&mut self.stream)?;
            info!("[id = {}] Receive from stream: {:?}.", id, &header);

            if header.request_id != id {
                return Err(ClientError::ResponseNotFound(id));
            }

            match header.r#type {
                RequestType::Stdout => {
                    let content = header.read_content_from_stream(&mut self.stream)?;
                    self.get_output_mut(id)?.set_stdout(content)
                }
                RequestType::Stderr => {
                    let content = header.read_content_from_stream(&mut self.stream)?;
                    self.get_output_mut(id)?.set_stderr(content)
                }
                RequestType::EndRequest => {
                    let end_request_rec = EndRequestRec::from_header(&header, &mut self.stream)?;
                    info!("[id = {}] Receive from stream: {:?}.", id, &end_request_rec);
                    global_end_request_rec = Some(end_request_rec);
                    break;
                }
                r#type => return Err(ClientError::UnknownRequestType(r#type)),
            }
        }

        match global_end_request_rec {
            Some(end_request_rec) => end_request_rec
                .end_request
                .protocol_status
                .convert_to_client_result(end_request_rec.end_request.app_status),
            None => unreachable!(),
        }
    }

    fn init_output(&mut self, id: u16) {
        self.outputs.insert(id, Default::default());
    }

    fn get_output_mut(&mut self, id: u16) -> ClientResult<&mut Output> {
        self.outputs.get_mut(&id).ok_or_else(|| ClientError::RequestIdNotFound(id))
    }
}

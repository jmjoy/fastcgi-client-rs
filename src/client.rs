use crate::id::RequestIdGenerator;
use crate::meta::{Address, BeginRequestRec, EndRequestRec, Header, Output, OutputMap, ParamPairs, RequestType, Role};
use crate::params::Params;
use crate::{ClientError, ClientResult};
use crate::Stream;

use log::info;
use std::collections::HashMap;
use std::io::{self, BufReader, BufWriter, ErrorKind, Read, Write};
use std::net::TcpStream;
use std::net::ToSocketAddrs as _;

use std::sync::Arc;
use std::time::Duration;

/// Client for handling communication between fastcgi server.
pub struct Client<R, W>
where
    R: Read + Send + Sync,
    W: Write + Send + Sync,
{
    keep_alive: bool,
    read_stream: Box<R>,
    write_stream: Box<W>,
    outputs: OutputMap,
}

impl<R: Stream, W: Stream> Client<R, W> {
//    /// Construct a `Client` Object with stream (such as `std::net::TcpStream` or `std::os::unix::net::UnixStream`,
//    /// with buffed read/write for stream.
//    pub fn new(stream: &impl Stream, keep_alive: bool) -> io::Result<Self> {
//        Self::new_with_is_buffed(stream, keep_alive, true)
//    }

    /// Construct a `Client` Object with stream (such as `std::net::TcpStream` or `std::os::unix::net::UnixStream`,
    /// - `is_buffed` whether to buffed read/write for stream.
    pub fn new_with_is_buffed<S: Stream>(stream: &S, keep_alive: bool, is_buffed: bool) -> io::Result<Self> {
        let outputs = HashMap::new();

//        if is_buffed {
//            Self {
//                keep_alive,
//                read_stream: BufReader::new(stream.try_clone()?),
//                write_stream: BufWriter::new(stream.try_clone()?),
//                outputs,
//            }
//        } else {
        Ok(Self {
            keep_alive,
            read_stream: Box::new(stream.try_clone()?),
            write_stream: Box::new(stream.try_clone()?),
            outputs,
        })
//        }
    }
}

impl<R, W> Client<R, W>
where
    R: Read + Send + Sync,
    W: Write + Send + Sync,
{
    /// Send request and receive response from fastcgi server.
    /// - `params` fastcgi params.
    /// - `body` always the http post or put body.
    ///
    /// return the output of fastcgi stdout and stderr.
    pub fn do_request<'a>(&mut self, params: &Params<'a>, body: &mut Read) -> ClientResult<&mut Output> {
        let id = RequestIdGenerator.generate();
        self.handle_request(id, params, body)?;
        self.handle_response(id)?;
        Ok(self.outputs.get_mut(&id).ok_or_else(|| ClientError::RequestIdNotFound(id))?)
    }

    fn handle_request<'a>(&mut self, id: u16, params: &Params<'a>, body: &mut Read) -> ClientResult<()> {
        let write_stream = &mut self.write_stream;

        info!("[id = {}] Start handle request.", id);

        let begin_request_rec = BeginRequestRec::new(id, Role::Responder, self.keep_alive)?;
        info!("[id = {}] Send to stream: {:?}.", id, &begin_request_rec);
        begin_request_rec.write_to_stream(write_stream)?;

        let param_pairs = ParamPairs::new(params);
        info!("[id = {}] Params will be sent: {:?}.", id, &param_pairs);

        Header::write_to_stream_batches(
            RequestType::Params,
            id,
            write_stream,
            &mut &param_pairs.to_content()?[..],
            Some(|header| {
                info!("[id = {}] Send to stream for Params: {:?}.", id, &header);
                header
            }),
        )?;

        Header::write_to_stream_batches(
            RequestType::Params,
            id,
            write_stream,
            &mut io::empty(),
            Some(|header| {
                info!("[id = {}] Send to stream for Params: {:?}.", id, &header);
                header
            }),
        )?;

        Header::write_to_stream_batches(
            RequestType::Stdin,
            id,
            write_stream,
            body,
            Some(|header| {
                info!("[id = {}] Send to stream for Stdin: {:?}.", id, &header);
                header
            }),
        )?;

        Header::write_to_stream_batches(
            RequestType::Stdin,
            id,
            write_stream,
            &mut io::empty(),
            Some(|header| {
                info!("[id = {}] Send to stream for Stdin: {:?}.", id, &header);
                header
            }),
        )?;

        write_stream.flush()?;

        Ok(())
    }

    fn handle_response(&mut self, id: u16) -> ClientResult<()> {
        self.init_output(id);

        let global_end_request_rec: Option<EndRequestRec>;

        loop {
            let read_stream = &mut self.read_stream;
            let header = Header::new_from_stream(read_stream)?;
            info!("[id = {}] Receive from stream: {:?}.", id, &header);

            if header.request_id != id {
                return Err(ClientError::ResponseNotFound(id));
            }

            match header.r#type {
                RequestType::Stdout => {
                    let content = header.read_content_from_stream(read_stream)?;
                    self.get_output_mut(id)?.set_stdout(content)
                }
                RequestType::Stderr => {
                    let content = header.read_content_from_stream(read_stream)?;
                    self.get_output_mut(id)?.set_stderr(content)
                }
                RequestType::EndRequest => {
                    let end_request_rec = EndRequestRec::from_header(&header, read_stream)?;
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

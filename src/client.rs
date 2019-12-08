use crate::id::RequestIdGenerator;
use crate::meta::{BeginRequestRec, EndRequestRec, Header, Output, OutputMap, ParamPairs, RequestType, Role};
use crate::params::Params;
use crate::{ErrorKind, Result as ClientResult};
use bufstream::BufStream;

use log::debug;
use std::collections::HashMap;
use std::io::{self, Read, Write};

#[cfg(feature = "futures")]
use futures::io::{AsyncRead, AsyncWrite, AsyncWriteExt};

/// Client for handling communication between fastcgi server.
pub struct Client<S: Read + Write + Send + Sync> {
    keep_alive: bool,
    stream: Box<S>,
    outputs: OutputMap,
}

impl<S: Read + Write + Send + Sync> Client<BufStream<S>> {
    /// Construct a `Client` Object with stream (such as `std::net::TcpStream` or `std::os::unix::net::UnixStream`,
    /// with buffered read/write for stream.
    pub fn new(stream: S, keep_alive: bool) -> Self {
        Self {
            keep_alive,
            stream: Box::new(BufStream::new(stream)),
            outputs: HashMap::new(),
        }
    }
}

impl<S: Read + Write + Send + Sync> Client<S> {
    /// Construct a `Client` Object with stream (such as `std::net::TcpStream` or `std::os::unix::net::UnixStream`,
    /// without buffered read/write for stream.
    pub fn new_without_buffered(stream: S, keep_alive: bool) -> Self {
        Self {
            keep_alive,
            stream: Box::new(stream),
            outputs: HashMap::new(),
        }
    }

    /// Send request and receive response from fastcgi server.
    /// - `params` fastcgi params.
    /// - `body` always the http post or put body.
    ///
    /// return the output of fastcgi stdout and stderr.
    pub fn do_request<'a>(&mut self, params: &Params<'a>, body: &mut dyn Read) -> ClientResult<&mut Output> {
        let id = RequestIdGenerator.generate();
        self.handle_request(id, params, body)?;
        self.handle_response(id)?;
        Ok(self.outputs.get_mut(&id).ok_or_else(|| ErrorKind::RequestIdNotFound(id))?)
    }

    fn handle_request<'a>(&mut self, id: u16, params: &Params<'a>, body: &mut dyn Read) -> ClientResult<()> {
        let write_stream = &mut self.stream;

        debug!("[id = {}] Start handle request.", id);

        let begin_request_rec = BeginRequestRec::new(id, Role::Responder, self.keep_alive)?;
        debug!("[id = {}] Send to stream: {:?}.", id, &begin_request_rec);
        begin_request_rec.write_to_stream(write_stream)?;

        let param_pairs = ParamPairs::new(params);
        debug!("[id = {}] Params will be sent: {:?}.", id, &param_pairs);

        Header::write_to_stream_batches(
            RequestType::Params,
            id,
            write_stream,
            &mut &param_pairs.to_content()?[..],
            Some(|header| {
                debug!("[id = {}] Send to stream for Params: {:?}.", id, &header);
                header
            }),
        )?;

        Header::write_to_stream_batches(
            RequestType::Params,
            id,
            write_stream,
            &mut io::empty(),
            Some(|header| {
                debug!("[id = {}] Send to stream for Params: {:?}.", id, &header);
                header
            }),
        )?;

        Header::write_to_stream_batches(
            RequestType::Stdin,
            id,
            write_stream,
            body,
            Some(|header| {
                debug!("[id = {}] Send to stream for Stdin: {:?}.", id, &header);
                header
            }),
        )?;

        Header::write_to_stream_batches(
            RequestType::Stdin,
            id,
            write_stream,
            &mut io::empty(),
            Some(|header| {
                debug!("[id = {}] Send to stream for Stdin: {:?}.", id, &header);
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
            let read_stream = &mut self.stream;
            let header = Header::new_from_stream(read_stream)?;
            debug!("[id = {}] Receive from stream: {:?}.", id, &header);

            if header.request_id != id {
                return Err(ErrorKind::ResponseNotFound(id).into());
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
                    debug!("[id = {}] Receive from stream: {:?}.", id, &end_request_rec);
                    global_end_request_rec = Some(end_request_rec);
                    break;
                }
                r#type => return Err(ErrorKind::UnknownRequestType(r#type).into()),
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
        self.outputs.get_mut(&id).ok_or_else(|| ErrorKind::RequestIdNotFound(id).into())
    }
}

#[cfg(feature = "futures")]
#[cfg_attr(docsrs, doc(cfg(feature = "futures")))]
/// Async client for handling communication between fastcgi server.
pub struct AsyncClient<S: AsyncRead + AsyncWrite + Send + Sync + Unpin> {
    keep_alive: bool,
    stream: Box<S>,
    outputs: OutputMap,
}

#[cfg(feature = "futures")]
impl<S: AsyncRead + AsyncWrite + Send + Sync + Unpin> AsyncClient<S> {
    /// Construct a `AsyncClient` Object with stream (such as `async_std::net::TcpStream` or `async_std::os::unix::net::UnixStream`,
    /// with buffered read/write for stream.
    pub fn new(stream: S, keep_alive: bool) -> Self {
        Self {
            keep_alive,
            stream: Box::new(stream),
            outputs: HashMap::new(),
        }
    }

    /// Send request and receive response from fastcgi server.
    /// - `params` fastcgi params.
    /// - `body` always the http post or put body.
    ///
    /// return the output of fastcgi stdout and stderr.
    pub async fn do_request<'a>(&mut self, params: &Params<'a>, body: &mut (dyn AsyncRead + Unpin)) -> ClientResult<&mut Output> {
        let id = RequestIdGenerator.generate();
        self.handle_request(id, params, body).await?;
        self.handle_response(id).await?;
        Ok(self.outputs.get_mut(&id).ok_or_else(|| ErrorKind::RequestIdNotFound(id))?)
    }

    async fn handle_request<'a>(&mut self, id: u16, params: &Params<'a>, body: &mut (dyn AsyncRead + Unpin)) -> ClientResult<()> {
        let write_stream = &mut self.stream;

        debug!("[id = {}] Start handle request.", id);

        let begin_request_rec = BeginRequestRec::new(id, Role::Responder, self.keep_alive)?;
        debug!("[id = {}] Send to stream: {:?}.", id, &begin_request_rec);
        begin_request_rec.async_write_to_stream(write_stream).await?;

        let param_pairs = ParamPairs::new(params);
        debug!("[id = {}] Params will be sent: {:?}.", id, &param_pairs);

        Header::async_write_to_stream_batches(
            RequestType::Params,
            id,
            write_stream,
            &mut &param_pairs.to_content()?[..],
            Some(|header| {
                debug!("[id = {}] Send to stream for Params: {:?}.", id, &header);
                header
            }),
        )
        .await?;

        Header::async_write_to_stream_batches(
            RequestType::Params,
            id,
            write_stream,
            &mut futures::io::empty(),
            Some(|header| {
                debug!("[id = {}] Send to stream for Params: {:?}.", id, &header);
                header
            }),
        )
        .await?;

        Header::async_write_to_stream_batches(
            RequestType::Stdin,
            id,
            write_stream,
            body,
            Some(|header| {
                debug!("[id = {}] Send to stream for Stdin: {:?}.", id, &header);
                header
            }),
        )
        .await?;

        Header::async_write_to_stream_batches(
            RequestType::Stdin,
            id,
            write_stream,
            &mut futures::io::empty(),
            Some(|header| {
                debug!("[id = {}] Send to stream for Stdin: {:?}.", id, &header);
                header
            }),
        )
        .await?;

        write_stream.flush().await?;

        Ok(())
    }

    async fn handle_response(&mut self, id: u16) -> ClientResult<()> {
        self.init_output(id);

        let global_end_request_rec: Option<EndRequestRec>;

        loop {
            let read_stream = &mut self.stream;
            let header = Header::new_from_async_stream(read_stream).await?;
            debug!("[id = {}] Receive from stream: {:?}.", id, &header);

            if header.request_id != id {
                return Err(ErrorKind::ResponseNotFound(id).into());
            }

            match header.r#type {
                RequestType::Stdout => {
                    let content = header.read_content_from_async_stream(read_stream).await?;
                    self.get_output_mut(id)?.set_stdout(content)
                }
                RequestType::Stderr => {
                    let content = header.read_content_from_async_stream(read_stream).await?;
                    self.get_output_mut(id)?.set_stderr(content)
                }
                RequestType::EndRequest => {
                    let end_request_rec = EndRequestRec::from_async_header(&header, read_stream).await?;
                    debug!("[id = {}] Receive from stream: {:?}.", id, &end_request_rec);
                    global_end_request_rec = Some(end_request_rec);
                    break;
                }
                r#type => return Err(ErrorKind::UnknownRequestType(r#type).into()),
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
        self.outputs.get_mut(&id).ok_or_else(|| ErrorKind::RequestIdNotFound(id).into())
    }
}

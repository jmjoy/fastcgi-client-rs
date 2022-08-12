use crate::{
    conn::{KeepAlive, Mode, Short},
    id::{AllocRequestId, FixRequestIdAllocator, PooledRequestIdAllocator},
    meta::{BeginRequestRec, EndRequestRec, Header, ParamPairs, RequestType, Role},
    params::Params,
    request::Request,
    response::ResponseMap,
    ClientError, ClientResult, Response,
};
use std::{collections::HashMap, marker::PhantomData};
use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt};
use tracing::debug;

/// Async client for handling communication between fastcgi server.
pub struct Client<S, M, A> {
    stream: S,
    id_allocator: A,
    outputs: ResponseMap,
    _mode: PhantomData<M>,
}

impl<S: AsyncRead + AsyncWrite + Unpin> Client<S, Short, FixRequestIdAllocator> {
    /// Construct a `Client` Object with stream, such as `tokio::net::TcpStream`
    /// or `tokio::net::UnixStream`, under short connection mode.
    pub fn new(stream: S) -> Self {
        Self {
            stream,
            id_allocator: FixRequestIdAllocator,
            outputs: HashMap::new(),
            _mode: PhantomData,
        }
    }

    /// Send request and receive response from fastcgi server, under short
    /// connection mode.
    pub async fn execute_once<I: AsyncRead + Unpin>(
        mut self, request: Request<'_, I>,
    ) -> ClientResult<Response> {
        self.do_execute(request).await
    }
}

impl<S: AsyncRead + AsyncWrite + Unpin> Client<S, KeepAlive, PooledRequestIdAllocator> {
    /// Construct a `Client` Object with stream, such as `tokio::net::TcpStream`
    /// or `tokio::net::UnixStream`, under keep alive connection mode.
    pub fn new_keep_alive(stream: S) -> Self {
        Self {
            stream,
            id_allocator: PooledRequestIdAllocator::default(),
            outputs: HashMap::new(),
            _mode: PhantomData,
        }
    }

    /// Send request and receive response from fastcgi server, under keep alive
    /// connection mode.
    pub async fn execute<I: AsyncRead + Unpin>(
        &mut self, request: Request<'_, I>,
    ) -> ClientResult<Response> {
        self.do_execute(request).await
    }
}

impl<S: AsyncRead + AsyncWrite + Unpin, M: Mode, A: AllocRequestId> Client<S, M, A> {
    async fn do_execute<I: AsyncRead + Unpin>(
        &mut self, request: Request<'_, I>,
    ) -> ClientResult<Response> {
        let id = self.id_allocator.alloc()?;
        let result = self.inner_execute(request, id).await;
        self.id_allocator.release(id);
        result
    }

    async fn inner_execute<I: AsyncRead + Unpin>(
        &mut self, mut request: Request<'_, I>, id: u16,
    ) -> ClientResult<Response> {
        self.handle_request(id, &request.params, &mut request.stdin)
            .await?;
        self.handle_response(id).await?;
        self.outputs
            .get(&id)
            .cloned()
            .ok_or(ClientError::RequestIdNotFound { id })
    }

    async fn handle_request<'a, I: AsyncRead + Unpin>(
        &mut self, id: u16, params: &Params<'a>, body: &mut I,
    ) -> ClientResult<()> {
        let write_stream = &mut self.stream;

        debug!(id, "Start handle request");

        let begin_request_rec =
            BeginRequestRec::new(id, Role::Responder, <M>::is_keep_alive()).await?;
        debug!(id, ?begin_request_rec, "Send to stream.");
        begin_request_rec.write_to_stream(write_stream).await?;

        let param_pairs = ParamPairs::new(params);
        debug!(id, ?param_pairs, "Params will be sent.");

        Header::write_to_stream_batches(
            RequestType::Params,
            id,
            write_stream,
            &mut &param_pairs.to_content().await?[..],
            Some(|header| {
                debug!(id, ?header, "Send to stream for Params.");
                header
            }),
        )
        .await?;

        Header::write_to_stream_batches(
            RequestType::Params,
            id,
            write_stream,
            &mut tokio::io::empty(),
            Some(|header| {
                debug!(id, ?header, "Send to stream for Params.");
                header
            }),
        )
        .await?;

        Header::write_to_stream_batches(
            RequestType::Stdin,
            id,
            write_stream,
            body,
            Some(|header| {
                debug!(id, ?header, "Send to stream for Stdin.");
                header
            }),
        )
        .await?;

        Header::write_to_stream_batches(
            RequestType::Stdin,
            id,
            write_stream,
            &mut tokio::io::empty(),
            Some(|header| {
                debug!(id, ?header, "Send to stream for Stdin.");
                header
            }),
        )
        .await?;

        write_stream.flush().await?;

        Ok(())
    }

    async fn handle_response(&mut self, id: u16) -> ClientResult<()> {
        self.init_output(id);

        let global_end_request_rec = loop {
            let read_stream = &mut self.stream;
            let header = Header::new_from_stream(read_stream).await?;
            debug!(id, ?header, "Receive from stream.");

            if header.request_id != id {
                return Err(ClientError::ResponseNotFound { id });
            }

            match header.r#type {
                RequestType::Stdout => {
                    let content = header.read_content_from_stream(read_stream).await?;
                    self.get_output_mut(id)?.set_stdout(content)
                }
                RequestType::Stderr => {
                    let content = header.read_content_from_stream(read_stream).await?;
                    self.get_output_mut(id)?.set_stderr(content)
                }
                RequestType::EndRequest => {
                    let end_request_rec = EndRequestRec::from_header(&header, read_stream).await?;
                    debug!(id, ?end_request_rec, "Receive from stream.");
                    break Some(end_request_rec);
                }
                r#type => {
                    return Err(ClientError::UnknownRequestType {
                        request_type: r#type,
                    })
                }
            }
        };

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

    fn get_output_mut(&mut self, id: u16) -> ClientResult<&mut Response> {
        self.outputs
            .get_mut(&id)
            .ok_or(ClientError::RequestIdNotFound { id })
    }
}

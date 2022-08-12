use crate::{
    conn::{KeepAlive, Mode, Short, Split},
    id::{AllocRequestId, FixRequestIdAllocator, PooledRequestIdAllocator},
    meta::{BeginRequestRec, EndRequestRec, Header, ParamPairs, RequestType, Role},
    params::Params,
    request::Request,
    response::ResponseMap,
    ClientError, ClientResult, Response,
};
use std::{collections::HashMap, future::Future, marker::PhantomData};
use tokio::{
    io::{AsyncRead, AsyncWrite, AsyncWriteExt},
    sync::Mutex,
};
use tracing::debug;

/// Async client for handling communication between fastcgi server.
pub struct Client<R, W, M, A> {
    read: Mutex<Option<R>>,
    write: Mutex<W>,
    id_allocator: Mutex<A>,
    outputs: ResponseMap,
    _mode: PhantomData<M>,
}

impl<R: AsyncRead + Unpin, W: AsyncWrite + Unpin> Client<R, W, Short, FixRequestIdAllocator> {
    /// Construct a `Client` Object with stream, such as `tokio::net::TcpStream`
    /// or `tokio::net::UnixStream`, under short connection mode.
    pub fn new<S>(stream: S) -> Self
    where
        S: Split<Read = R, Write = W>,
    {
        let (read, write) = stream.split();
        Self {
            read: Mutex::new(Some(read)),
            write: Mutex::new(write),
            id_allocator: Mutex::new(FixRequestIdAllocator),
            outputs: HashMap::new(),
            _mode: PhantomData,
        }
    }

    /// Send request and receive response from fastcgi server, under short
    /// connection mode.
    pub async fn execute_once<I: AsyncRead + Unpin>(
        mut self, mut request: Request<'_, I>,
    ) -> ClientResult<Response> {
        let id = self.id_allocator.get_mut().alloc()?;
        self.handle_request_short(id, &request.params, &mut request.stdin)
            .await?;
        let result = Self::handle_response(self.read.get_mut().as_mut().unwrap(), Some(id)).await;
        self.id_allocator.get_mut().release(id);
        result
    }

    async fn handle_request_short<'a, I: AsyncRead + Unpin>(
        &mut self, id: u16, params: &Params<'a>, body: &mut I,
    ) -> ClientResult<()> {
        let write = self.write.get_mut();
        Self::handle_request_start(write, id).await?;
        Self::handle_request_params(write, id, params).await?;
        Self::handle_request_body(write, id, body).await?;
        Self::handle_request_flush(write).await?;
        Ok(())
    }
}

impl<R: AsyncRead + Unpin, W: AsyncWrite + Unpin>
    Client<R, W, KeepAlive, PooledRequestIdAllocator>
{
    /// Construct a `Client` Object with stream, such as `tokio::net::TcpStream`
    /// or `tokio::net::UnixStream`, under keep alive connection mode.
    pub fn new_keep_alive<S>(stream: S) -> Self
    where
        S: Split<Read = R, Write = W>,
    {
        let (read, write) = stream.split();
        Self {
            read: Mutex::new(Some(read)),
            write: Mutex::new(write),
            id_allocator: Mutex::new(PooledRequestIdAllocator::default()),
            outputs: HashMap::new(),
            _mode: PhantomData,
        }
    }

    /// Send request and receive response from fastcgi server, under keep alive
    /// connection mode.
    pub async fn execute<I: AsyncRead + Unpin>(
        &self, request: Request<'_, I>,
    ) -> ClientResult<Response> {
        let id = self.id_allocator.lock().await.alloc()?;
        let result = self.inner_execute(request, id).await;
        self.id_allocator.lock().await.release(id);
        result
    }
}

impl<R: AsyncRead + Unpin, W: AsyncWrite + Unpin, M: Mode, A: AllocRequestId> Client<R, W, M, A> {
    // async fn inner_execute<I: AsyncRead + Unpin>(
    //     &mut self, mut request: Request<'_, I>, id: u16,
    // ) -> ClientResult<Response> {
    //     self.handle_request(id, &request.params, &mut request.stdin)
    //         .await?;
    //     self.handle_response(id).await?;
    //     self.outputs
    //         .get(&id)
    //         .cloned()
    //         .ok_or(ClientError::RequestIdNotFound { id })
    // }

    async fn handle_request_start(write_stream: &mut W, id: u16) -> ClientResult<()> {
        debug!(id, "Start handle request");

        let begin_request_rec =
            BeginRequestRec::new(id, Role::Responder, <M>::is_keep_alive()).await?;

        debug!(id, ?begin_request_rec, "Send to stream.");

        begin_request_rec.write_to_stream(write_stream).await?;

        Ok(())
    }

    async fn handle_request_params<'a>(
        write_stream: &mut W, id: u16, params: &Params<'a>,
    ) -> ClientResult<()> {
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

        Ok(())
    }

    async fn handle_request_body<I: AsyncRead + Unpin>(
        write_stream: &mut W, id: u16, body: &mut I,
    ) -> ClientResult<()> {
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

        Ok(())
    }

    async fn handle_request_flush(write_stream: &mut W) -> ClientResult<()> {
        write_stream.flush().await?;

        Ok(())
    }

    async fn handle_response(read_stream: &mut R, id: Option<u16>) -> ClientResult<Response> {
        let mut response = Response::default();

        loop {
            let header = Header::new_from_stream(read_stream).await?;
            debug!(id, ?header, "Receive from stream.");

            if let Some(id) = id {
                if header.request_id != id {
                    return Err(ClientError::ResponseNotFound { id });
                }
            }

            match header.r#type {
                RequestType::Stdout => {
                    let content = header.read_content_from_stream(read_stream).await?;
                    response.set_stdout(content);
                }
                RequestType::Stderr => {
                    let content = header.read_content_from_stream(read_stream).await?;
                    response.set_stderr(content);
                }
                RequestType::EndRequest => {
                    let end_request_rec = EndRequestRec::from_header(&header, read_stream).await?;
                    debug!(id, ?end_request_rec, "Receive from stream.");

                    end_request_rec
                        .end_request
                        .protocol_status
                        .convert_to_client_result(end_request_rec.end_request.app_status)?;

                    break Ok(response);
                }
                r#type => {
                    return Err(ClientError::UnknownRequestType {
                        request_type: r#type,
                    })
                }
            }
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

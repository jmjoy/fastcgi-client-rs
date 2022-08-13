use crate::{
    conn::{KeepAlive, Mode, ShortConn},
    meta::{BeginRequestRec, EndRequestRec, Header, ParamPairs, RequestType, Role},
    params::Params,
    request::Request,
    ClientError, ClientResult, Response,
};
use std::marker::PhantomData;
use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt};
use tracing::debug;

/// I refer to nignx fastcgi implementation, found the request id is always 1.
///
/// <https://github.com/nginx/nginx/blob/f7ea8c76b55f730daa3b63f5511feb564b44d901/src/http/modules/ngx_http_fastcgi_module.c>
const REQUEST_ID: u16 = 1;

/// Async client for handling communication between fastcgi server.
pub struct Client<S, M> {
    stream: S,
    _mode: PhantomData<M>,
}

impl<S: AsyncRead + AsyncWrite + Unpin> Client<S, ShortConn> {
    /// Construct a `Client` Object with stream, such as `tokio::net::TcpStream`
    /// or `tokio::net::UnixStream`, under short connection mode.
    pub fn new(stream: S) -> Self {
        Self {
            stream,
            _mode: PhantomData,
        }
    }

    /// Send request and receive response from fastcgi server, under short
    /// connection mode.
    pub async fn execute_once<I: AsyncRead + Unpin>(
        mut self, request: Request<'_, I>,
    ) -> ClientResult<Response> {
        self.inner_execute(request).await
    }
}

impl<S: AsyncRead + AsyncWrite + Unpin> Client<S, KeepAlive> {
    /// Construct a `Client` Object with stream, such as `tokio::net::TcpStream`
    /// or `tokio::net::UnixStream`, under keep alive connection mode.
    pub fn new_keep_alive(stream: S) -> Self {
        Self {
            stream,
            _mode: PhantomData,
        }
    }

    /// Send request and receive response from fastcgi server, under keep alive
    /// connection mode.
    pub async fn execute<I: AsyncRead + Unpin>(
        &mut self, request: Request<'_, I>,
    ) -> ClientResult<Response> {
        self.inner_execute(request).await
    }
}

impl<S: AsyncRead + AsyncWrite + Unpin, M: Mode> Client<S, M> {
    async fn inner_execute<I: AsyncRead + Unpin>(
        &mut self, mut request: Request<'_, I>,
    ) -> ClientResult<Response> {
        Self::handle_request(
            &mut self.stream,
            REQUEST_ID,
            &request.params,
            &mut request.stdin,
        )
        .await?;
        Self::handle_response(&mut self.stream, REQUEST_ID).await
    }

    async fn handle_request<'a, I: AsyncRead + Unpin>(
        stream: &mut S, id: u16, params: &Params<'a>, body: &mut I,
    ) -> ClientResult<()> {
        Self::handle_request_start(stream, id).await?;
        Self::handle_request_params(stream, id, params).await?;
        Self::handle_request_body(stream, id, body).await?;
        Self::handle_request_flush(stream).await?;
        Ok(())
    }

    async fn handle_request_start(stream: &mut S, id: u16) -> ClientResult<()> {
        debug!(id, "Start handle request");

        let begin_request_rec =
            BeginRequestRec::new(id, Role::Responder, <M>::is_keep_alive()).await?;

        debug!(id, ?begin_request_rec, "Send to stream.");

        begin_request_rec.write_to_stream(stream).await?;

        Ok(())
    }

    async fn handle_request_params<'a>(
        stream: &mut S, id: u16, params: &Params<'a>,
    ) -> ClientResult<()> {
        let param_pairs = ParamPairs::new(params);
        debug!(id, ?param_pairs, "Params will be sent.");

        Header::write_to_stream_batches(
            RequestType::Params,
            id,
            stream,
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
            stream,
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
        stream: &mut S, id: u16, body: &mut I,
    ) -> ClientResult<()> {
        Header::write_to_stream_batches(
            RequestType::Stdin,
            id,
            stream,
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
            stream,
            &mut tokio::io::empty(),
            Some(|header| {
                debug!(id, ?header, "Send to stream for Stdin.");
                header
            }),
        )
        .await?;

        Ok(())
    }

    async fn handle_request_flush(stream: &mut S) -> ClientResult<()> {
        stream.flush().await?;

        Ok(())
    }

    async fn handle_response(stream: &mut S, id: u16) -> ClientResult<Response> {
        let mut response = Response::default();

        loop {
            let header = Header::new_from_stream(stream).await?;
            if header.request_id != id {
                return Err(ClientError::ResponseNotFound { id });
            }
            debug!(id, ?header, "Receive from stream.");

            match header.r#type {
                RequestType::Stdout => {
                    let content = header.read_content_from_stream(stream).await?;
                    response.set_stdout(content);
                }
                RequestType::Stderr => {
                    let content = header.read_content_from_stream(stream).await?;
                    response.set_stderr(content);
                }
                RequestType::EndRequest => {
                    let end_request_rec = EndRequestRec::from_header(&header, stream).await?;
                    debug!(id, ?end_request_rec, "Receive from stream.");

                    end_request_rec
                        .end_request
                        .protocol_status
                        .convert_to_client_result(end_request_rec.end_request.app_status)?;

                    return Ok(response);
                }
                r#type => {
                    return Err(ClientError::UnknownRequestType {
                        request_type: r#type,
                    })
                }
            }
        }
    }
}

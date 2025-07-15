// Copyright 2022 jmjoy
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! FastCGI client implementation for async communication with FastCGI servers.
//!
//! This module provides the main `Client` struct that handles communication
//! with FastCGI servers in both short connection and keep-alive modes.
//! The client can execute requests and receive responses or response streams.

use crate::{
    conn::{KeepAlive, Mode, ShortConn},
    meta::{BeginRequestRec, EndRequestRec, Header, ParamPairs, RequestType, Role},
    params::Params,
    request::Request,
    response::ResponseStream,
    ClientError, ClientResult, Response,
};
use std::marker::PhantomData;
use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt};
use tracing::debug;

/// I refer to nginx fastcgi implementation, found the request id is always 1.
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

    /// Send request and receive response stream from fastcgi server, under
    /// short connection mode.
    ///
    /// # Examples
    ///
    /// ```
    /// use fastcgi_client::{response::Content, Client, Params, Request, StreamExt};
    /// use tokio::{io, net::TcpStream};
    ///
    /// async fn stream() {
    ///     let stream = TcpStream::connect(("127.0.0.1", 9000)).await.unwrap();
    ///     let client = Client::new(stream);
    ///     let mut stream = client
    ///         .execute_once_stream(Request::new(Params::default(), &mut io::empty()))
    ///         .await
    ///         .unwrap();
    ///
    ///     while let Some(content) = stream.next().await {
    ///         let content = content.unwrap();
    ///
    ///         match content {
    ///             Content::Stdout(out) => todo!(),
    ///             Content::Stderr(out) => todo!(),
    ///         }
    ///     }
    /// }
    /// ```
    pub async fn execute_once_stream<I: AsyncRead + Unpin>(
        mut self, request: Request<'_, I>,
    ) -> ClientResult<ResponseStream<S>> {
        Self::handle_request(&mut self.stream, REQUEST_ID, request.params, request.stdin).await?;
        Ok(ResponseStream::new(self.stream, REQUEST_ID))
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

    /// Send request and receive response stream from fastcgi server, under
    /// keep alive connection mode.
    ///
    /// # Examples
    ///
    /// ```
    /// use fastcgi_client::{response::Content, Client, Params, Request, StreamExt};
    /// use tokio::{io, net::TcpStream};
    ///
    /// async fn stream() {
    ///     let stream = TcpStream::connect(("127.0.0.1", 9000)).await.unwrap();
    ///     let mut client = Client::new_keep_alive(stream);
    ///
    ///     for _ in (0..3) {
    ///         let mut stream = client
    ///             .execute_stream(Request::new(Params::default(), &mut io::empty()))
    ///             .await
    ///             .unwrap();
    ///
    ///         while let Some(content) = stream.next().await {
    ///             let content = content.unwrap();
    ///
    ///             match content {
    ///                 Content::Stdout(out) => todo!(),
    ///                 Content::Stderr(out) => todo!(),
    ///             }
    ///         }
    ///     }
    /// }
    /// ```
    pub async fn execute_stream<I: AsyncRead + Unpin>(
        &mut self, request: Request<'_, I>,
    ) -> ClientResult<ResponseStream<&mut S>> {
        Self::handle_request(&mut self.stream, REQUEST_ID, request.params, request.stdin).await?;
        Ok(ResponseStream::new(&mut self.stream, REQUEST_ID))
    }
}

impl<S: AsyncRead + AsyncWrite + Unpin, M: Mode> Client<S, M> {
    /// Internal method to execute a request and return a complete response.
    ///
    /// # Arguments
    ///
    /// * `request` - The request to execute
    async fn inner_execute<I: AsyncRead + Unpin>(
        &mut self, request: Request<'_, I>,
    ) -> ClientResult<Response> {
        Self::handle_request(&mut self.stream, REQUEST_ID, request.params, request.stdin).await?;
        Self::handle_response(&mut self.stream, REQUEST_ID).await
    }

    /// Handles the complete request process.
    ///
    /// # Arguments
    ///
    /// * `stream` - The stream to write to
    /// * `id` - The request ID
    /// * `params` - The request parameters
    /// * `body` - The request body stream
    async fn handle_request<'a, I: AsyncRead + Unpin>(
        stream: &mut S, id: u16, params: Params<'a>, mut body: I,
    ) -> ClientResult<()> {
        Self::handle_request_start(stream, id).await?;
        Self::handle_request_params(stream, id, params).await?;
        Self::handle_request_body(stream, id, &mut body).await?;
        Self::handle_request_flush(stream).await?;
        Ok(())
    }

    /// Handles the start of a request by sending the begin request record.
    ///
    /// # Arguments
    ///
    /// * `stream` - The stream to write to
    /// * `id` - The request ID
    async fn handle_request_start(stream: &mut S, id: u16) -> ClientResult<()> {
        debug!(id, "Start handle request");

        let begin_request_rec =
            BeginRequestRec::new(id, Role::Responder, <M>::is_keep_alive()).await?;

        debug!(id, ?begin_request_rec, "Send to stream.");

        begin_request_rec.write_to_stream(stream).await?;

        Ok(())
    }

    /// Handles sending request parameters to the stream.
    ///
    /// # Arguments
    ///
    /// * `stream` - The stream to write to
    /// * `id` - The request ID
    /// * `params` - The request parameters
    async fn handle_request_params<'a>(
        stream: &mut S, id: u16, params: Params<'a>,
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

    /// Handles sending the request body to the stream.
    ///
    /// # Arguments
    ///
    /// * `stream` - The stream to write to
    /// * `id` - The request ID
    /// * `body` - The request body stream
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

    /// Flushes the stream to ensure all data is sent.
    ///
    /// # Arguments
    ///
    /// * `stream` - The stream to flush
    async fn handle_request_flush(stream: &mut S) -> ClientResult<()> {
        stream.flush().await?;

        Ok(())
    }

    /// Handles reading and processing the response from the stream.
    ///
    /// # Arguments
    ///
    /// * `stream` - The stream to read from
    /// * `id` - The request ID to match
    async fn handle_response(stream: &mut S, id: u16) -> ClientResult<Response> {
        let mut response = Response::default();

        let mut stderr = Vec::new();
        let mut stdout = Vec::new();

        loop {
            let header = Header::new_from_stream(stream).await?;
            if header.request_id != id {
                return Err(ClientError::ResponseNotFound { id });
            }
            debug!(id, ?header, "Receive from stream.");

            match header.r#type {
                RequestType::Stdout => {
                    stdout.extend(header.read_content_from_stream(stream).await?);
                }
                RequestType::Stderr => {
                    stderr.extend(header.read_content_from_stream(stream).await?);
                }
                RequestType::EndRequest => {
                    let end_request_rec = EndRequestRec::from_header(&header, stream).await?;
                    debug!(id, ?end_request_rec, "Receive from stream.");

                    end_request_rec
                        .end_request
                        .protocol_status
                        .convert_to_client_result(end_request_rec.end_request.app_status)?;

                    response.stdout = if stdout.is_empty() {
                        None
                    } else {
                        Some(stdout)
                    };
                    response.stderr = if stderr.is_empty() {
                        None
                    } else {
                        Some(stderr)
                    };

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

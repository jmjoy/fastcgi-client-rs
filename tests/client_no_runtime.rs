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

//! Runtime agnostic tests.
//!
//! These tests drive the client with `futures_executor::block_on` over an
//! in-memory stream, proving the crate needs neither an async runtime nor any
//! cargo feature to work. They also cover the FastCGI record encoding without
//! requiring a live php-fpm.

use fastcgi_client::{
    Client, Params, Request, StreamExt,
    io::{self, AsyncRead, AsyncWrite},
    response::Content,
};
use futures_executor::block_on;
use std::{
    io::Result as IoResult,
    pin::Pin,
    task::{Context, Poll},
};

const VERSION_1: u8 = 1;
const HEADER_LEN: usize = 8;

const TYPE_BEGIN_REQUEST: u8 = 1;
const TYPE_END_REQUEST: u8 = 3;
const TYPE_PARAMS: u8 = 4;
const TYPE_STDIN: u8 = 5;
const TYPE_STDOUT: u8 = 6;

const REQUEST_ID: u16 = 1;

const BODY: &[u8] = b"Content-type: text/html; charset=UTF-8\r\n\r\nhello";

/// In-memory duplex stream implementing the `futures_io` traits only.
///
/// Reads replay a pre-encoded FastCGI response, writes are collected so the
/// generated request records can be asserted on.
struct MockStream {
    to_read: Vec<u8>,
    read_pos: usize,
    written: Vec<u8>,
}

impl MockStream {
    fn new(to_read: Vec<u8>) -> Self {
        Self {
            to_read,
            read_pos: 0,
            written: Vec::new(),
        }
    }
}

impl AsyncRead for MockStream {
    fn poll_read(
        mut self: Pin<&mut Self>, _cx: &mut Context<'_>, buf: &mut [u8],
    ) -> Poll<IoResult<usize>> {
        let remaining = self.to_read.len() - self.read_pos;
        let len = remaining.min(buf.len());
        let start = self.read_pos;
        buf[..len].copy_from_slice(&self.to_read[start..start + len]);
        self.read_pos += len;
        Poll::Ready(Ok(len))
    }
}

impl AsyncWrite for MockStream {
    fn poll_write(
        mut self: Pin<&mut Self>, _cx: &mut Context<'_>, buf: &[u8],
    ) -> Poll<IoResult<usize>> {
        self.written.extend_from_slice(buf);
        Poll::Ready(Ok(buf.len()))
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<IoResult<()>> {
        Poll::Ready(Ok(()))
    }

    fn poll_close(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<IoResult<()>> {
        Poll::Ready(Ok(()))
    }
}

/// Encodes one FastCGI record, without padding.
fn record(r#type: u8, request_id: u16, content: &[u8]) -> Vec<u8> {
    let mut buf = Vec::with_capacity(HEADER_LEN + content.len());
    buf.push(VERSION_1);
    buf.push(r#type);
    buf.extend_from_slice(&request_id.to_be_bytes());
    buf.extend_from_slice(&(content.len() as u16).to_be_bytes());
    buf.push(0); // padding length
    buf.push(0); // reserved
    buf.extend_from_slice(content);
    buf
}

/// A successful response: one STDOUT record followed by END_REQUEST.
fn response_bytes() -> Vec<u8> {
    let mut end_request = Vec::with_capacity(8);
    end_request.extend_from_slice(&0u32.to_be_bytes()); // app status
    end_request.push(0); // protocol status: REQUEST_COMPLETE
    end_request.extend_from_slice(&[0; 3]); // reserved

    let mut buf = record(TYPE_STDOUT, REQUEST_ID, BODY);
    buf.extend(record(TYPE_END_REQUEST, REQUEST_ID, &end_request));
    buf
}

/// A parsed FastCGI record from the written byte stream.
#[derive(Debug)]
struct ParsedRecord {
    version: u8,
    r#type: u8,
    request_id: u16,
    content: Vec<u8>,
}

/// Splits the bytes written by the client back into records.
fn parse_records(mut buf: &[u8]) -> Vec<ParsedRecord> {
    let mut records = Vec::new();
    while !buf.is_empty() {
        assert!(
            buf.len() >= HEADER_LEN,
            "truncated header, {} bytes left",
            buf.len()
        );
        let content_length = u16::from_be_bytes([buf[4], buf[5]]) as usize;
        let padding_length = buf[6] as usize;
        let record = ParsedRecord {
            version: buf[0],
            r#type: buf[1],
            request_id: u16::from_be_bytes([buf[2], buf[3]]),
            content: buf[HEADER_LEN..HEADER_LEN + content_length].to_vec(),
        };
        buf = &buf[HEADER_LEN + content_length + padding_length..];
        records.push(record);
    }
    records
}

fn params() -> Params<'static> {
    Params::default()
        .request_method("GET")
        .script_name("/index.php")
}

/// Asserts the request record sequence produced for a single request.
fn assert_request_records(written: &[u8], keep_alive: bool) {
    let records = parse_records(written);

    for record in &records {
        assert_eq!(record.version, VERSION_1);
        assert_eq!(record.request_id, REQUEST_ID);
    }

    let types: Vec<u8> = records.iter().map(|record| record.r#type).collect();
    assert_eq!(
        types,
        vec![
            TYPE_BEGIN_REQUEST,
            TYPE_PARAMS,
            TYPE_PARAMS,
            TYPE_STDIN,
            TYPE_STDIN,
        ],
        "unexpected record sequence"
    );

    // BEGIN_REQUEST: role = Responder(1), flags bit 0 = keep alive.
    let begin = &records[0].content;
    assert_eq!(begin.len(), 8);
    assert_eq!(u16::from_be_bytes([begin[0], begin[1]]), 1);
    assert_eq!(begin[2], keep_alive as u8);

    // PARAMS: first record carries the pairs, second one terminates the stream.
    let params = &records[1].content;
    assert!(!params.is_empty());
    assert!(
        params.windows(14).any(|window| window == b"REQUEST_METHOD"),
        "params should contain REQUEST_METHOD"
    );
    assert!(records[2].content.is_empty(), "params must be terminated");

    // STDIN: first record carries the body, second one terminates the stream.
    assert_eq!(records[3].content, b"hello=world");
    assert!(records[4].content.is_empty(), "stdin must be terminated");
}

#[test]
fn execute_once_encodes_expected_records() {
    let mut stream = MockStream::new(response_bytes());

    let output = block_on(async {
        let client = Client::new(&mut stream);
        let request = Request::new(params(), io::Cursor::new(b"hello=world".to_vec()));
        client.execute_once(request).await.unwrap()
    });

    assert_eq!(output.stdout.as_deref(), Some(BODY));
    assert_eq!(output.stderr, None);
    assert_request_records(&stream.written, false);
}

#[test]
fn execute_keep_alive_without_runtime() {
    let mut stream = MockStream::new(response_bytes());

    let output = block_on(async {
        let mut client = Client::new_keep_alive(&mut stream);
        let request = Request::new(params(), io::Cursor::new(b"hello=world".to_vec()));
        client.execute(request).await.unwrap()
    });

    assert_eq!(output.stdout.as_deref(), Some(BODY));
    assert_eq!(output.stderr, None);
    assert_request_records(&stream.written, true);
}

#[test]
fn execute_once_stream_without_runtime() {
    let stream = MockStream::new(response_bytes());

    let stdout = block_on(async {
        let client = Client::new(stream);
        let request = Request::new(params(), io::empty());
        let mut stream = client.execute_once_stream(request).await.unwrap();

        let mut stdout = Vec::new();
        while let Some(content) = stream.next().await {
            match content.unwrap() {
                Content::Stdout(out) => stdout.extend_from_slice(&out),
                Content::Stderr(out) => panic!("unexpected stderr: {out:?}"),
            }
        }
        stdout
    });

    assert_eq!(stdout, BODY);
}

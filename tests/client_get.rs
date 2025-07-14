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

use fastcgi_client::{conn::ShortConn, request::Request, response::Content, Client, Params};
use std::env::current_dir;
use tokio::{
    io::{self, AsyncRead, AsyncWrite},
    net::TcpStream,
};

use futures::stream::StreamExt;

mod common;

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test() {
    common::setup();

    let stream = TcpStream::connect(("127.0.0.1", 9000)).await.unwrap();
    test_client(Client::new(stream)).await;
}

async fn test_client<S: AsyncRead + AsyncWrite + Unpin>(client: Client<S, ShortConn>) {
    let document_root = current_dir().unwrap().join("tests").join("php");
    let document_root = document_root.to_str().unwrap();
    let script_name = current_dir()
        .unwrap()
        .join("tests")
        .join("php")
        .join("index.php");
    let script_name = script_name.to_str().unwrap();

    let params = Params::default()
        .request_method("GET")
        .document_root(document_root)
        .script_name("/index.php")
        .script_filename(script_name)
        .request_uri("/index.php")
        .document_uri("/index.php")
        .remote_addr("127.0.0.1")
        .remote_port(12345)
        .server_addr("127.0.0.1")
        .server_port(80)
        .server_name("jmjoy-pc")
        .content_type("")
        .content_length(0);

    let output = client
        .execute_once(Request::new(params, &mut io::empty()))
        .await
        .unwrap();

    assert_eq!(
        String::from_utf8(output.stdout.unwrap_or(Default::default())).unwrap(),
        "X-Powered-By: PHP/7.1.30\r\nContent-type: text/html; charset=UTF-8\r\n\r\nhello"
    );
    assert_eq!(output.stderr, None);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_stream() {
    common::setup();

    let stream = TcpStream::connect(("127.0.0.1", 9000)).await.unwrap();
    test_client_stream(Client::new(stream)).await;
}

async fn test_client_stream<S: AsyncRead + AsyncWrite + Unpin>(client: Client<S, ShortConn>) {
    let document_root = current_dir().unwrap().join("tests").join("php");
    let document_root = document_root.to_str().unwrap();
    let script_name = current_dir()
        .unwrap()
        .join("tests")
        .join("php")
        .join("index.php");
    let script_name = script_name.to_str().unwrap();

    let params = Params::default()
        .request_method("GET")
        .document_root(document_root)
        .script_name("/index.php")
        .script_filename(script_name)
        .request_uri("/index.php")
        .document_uri("/index.php")
        .remote_addr("127.0.0.1")
        .remote_port(12345)
        .server_addr("127.0.0.1")
        .server_port(80)
        .server_name("jmjoy-pc")
        .content_type("")
        .content_length(0);

    let mut stream = client
        .execute_once_stream(Request::new(params, &mut io::empty()))
        .await
        .unwrap();

    let mut stdout = Vec::<u8>::new();
    while let Some(content) = stream.next().await {
        let content = content.unwrap();
        match content {
            Content::Stdout(out) => {
                stdout.extend_from_slice(&out);
            }
            Content::Stderr(_) => {
                panic!("stderr should not happened");
            }
        }
    }

    assert_eq!(
        String::from_utf8(stdout).unwrap(),
        "X-Powered-By: PHP/7.1.30\r\nContent-type: text/html; charset=UTF-8\r\n\r\nhello"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_big_response_stream() {
    common::setup();

    let stream = TcpStream::connect(("127.0.0.1", 9000)).await.unwrap();
    test_client_big_response_stream(Client::new(stream)).await;
}

async fn test_client_big_response_stream<S: AsyncRead + AsyncWrite + Unpin>(
    client: Client<S, ShortConn>,
) {
    let document_root = current_dir().unwrap().join("tests").join("php");
    let document_root = document_root.to_str().unwrap();
    let script_name = current_dir()
        .unwrap()
        .join("tests")
        .join("php")
        .join("big-response.php");
    let script_name = script_name.to_str().unwrap();

    let params = Params::default()
        .request_method("GET")
        .document_root(document_root)
        .script_name("/big-response.php")
        .script_filename(script_name)
        .request_uri("/big-response.php")
        .document_uri("/big-response.php")
        .remote_addr("127.0.0.1")
        .remote_port(12345)
        .server_addr("127.0.0.1")
        .server_port(80)
        .server_name("jmjoy-pc")
        .content_type("")
        .content_length(0);

    let mut stream = client
        .execute_once_stream(Request::new(params, &mut io::empty()))
        .await
        .unwrap();

    let mut stdout = Vec::<u8>::new();
    while let Some(content) = stream.next().await {
        let content = content.unwrap();
        match content {
            Content::Stdout(out) => {
                stdout.extend_from_slice(&out);
            }
            Content::Stderr(_) => {
                panic!("stderr should not happened");
            }
        }
    }

    assert_eq!(
        String::from_utf8(stdout).unwrap(),
        format!(
            "X-Powered-By: PHP/7.1.30\r\nContent-type: text/html; charset=UTF-8\r\n\r\n{}",
            ".".repeat(10000)
        )
    );
}

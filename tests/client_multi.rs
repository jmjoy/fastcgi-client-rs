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

use fastcgi_client::{request::Request, response::Content, Client, Params};
use std::{env::current_dir, io::Cursor};
use tokio::net::TcpStream;

use futures::stream::StreamExt;

mod common;

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn multi() {
    common::setup();

    let tasks = (0..3).map(|_| tokio::spawn(single())).collect::<Vec<_>>();
    for task in tasks {
        task.await.unwrap();
    }
}

async fn single() {
    let stream = TcpStream::connect(("127.0.0.1", 9000)).await.unwrap();
    let mut client = Client::new_keep_alive(stream);

    let document_root = current_dir().unwrap().join("tests").join("php");
    let document_root = document_root.to_str().unwrap();
    let script_name = current_dir()
        .unwrap()
        .join("tests")
        .join("php")
        .join("post.php");
    let script_name = script_name.to_str().unwrap();

    let body = b"p1=3&p2=4";

    let params = Params::default()
        .request_method("POST")
        .document_root(document_root)
        .script_name("/post.php")
        .script_filename(script_name)
        .request_uri("/post.php?g1=1&g2=2")
        .query_string("g1=1&g2=2")
        .document_uri("/post.php")
        .remote_addr("127.0.0.1")
        .remote_port(12345)
        .server_addr("127.0.0.1")
        .server_port(80)
        .server_name("jmjoy-pc")
        .content_type("application/x-www-form-urlencoded")
        .content_length(body.len());

    for _ in 0..3 {
        let output = client
            .execute(Request::new(params.clone(), Cursor::new(body)))
            .await
            .unwrap();

        let stdout = String::from_utf8(output.stdout.unwrap_or(Default::default())).unwrap();
        assert!(stdout.contains("Content-type: text/html; charset=UTF-8"));
        assert!(stdout.contains("\r\n\r\n"));
        assert!(stdout.contains("1234"));

        let stderr = String::from_utf8(output.stderr.unwrap_or(Default::default())).unwrap();
        assert!(stderr.contains("PHP message: PHP Fatal error:  Uncaught Exception: TEST"));
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn multi_stream() {
    common::setup();

    let tasks = (0..3)
        .map(|_| tokio::spawn(single_stream()))
        .collect::<Vec<_>>();
    for task in tasks {
        task.await.unwrap();
    }
}

async fn single_stream() {
    let stream = TcpStream::connect(("127.0.0.1", 9000)).await.unwrap();
    let mut client = Client::new_keep_alive(stream);

    let document_root = current_dir().unwrap().join("tests").join("php");
    let document_root = document_root.to_str().unwrap();
    let script_name = current_dir()
        .unwrap()
        .join("tests")
        .join("php")
        .join("post.php");
    let script_name = script_name.to_str().unwrap();

    let body = b"p1=3&p2=4";

    let params = Params::default()
        .request_method("POST")
        .document_root(document_root)
        .script_name("/post.php")
        .script_filename(script_name)
        .request_uri("/post.php?g1=1&g2=2")
        .query_string("g1=1&g2=2")
        .document_uri("/post.php")
        .remote_addr("127.0.0.1")
        .remote_port(12345)
        .server_addr("127.0.0.1")
        .server_port(80)
        .server_name("jmjoy-pc")
        .content_type("application/x-www-form-urlencoded")
        .content_length(body.len());

    for _ in 0..3 {
        let mut stream = client
            .execute_stream(Request::new(params.clone(), Cursor::new(body)))
            .await
            .unwrap();

        let mut stdout = Vec::<u8>::new();
        let mut stderr = Vec::<u8>::new();

        while let Some(content) = stream.next().await {
            let content = content.unwrap();
            match content {
                Content::Stdout(out) => {
                    stdout.extend_from_slice(&out);
                }
                Content::Stderr(err) => {
                    stderr.extend_from_slice(&err);
                }
            }
        }

        assert!(String::from_utf8(stdout).unwrap().starts_with(
            "X-Powered-By: PHP/7.1.30\r\nContent-type: text/html; charset=UTF-8\r\n\r\n1234<br \
             />\n<b>Fatal error</b>:  Uncaught Exception: TEST in"
        ));
        assert!(String::from_utf8(stderr)
            .unwrap()
            .starts_with("PHP message: PHP Fatal error:  Uncaught Exception: TEST in"));
    }
}

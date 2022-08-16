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

use fastcgi_client::{request::Request, Client, Params};
use std::{env::current_dir, time::Duration};
use tokio::{net::TcpStream, time::timeout};

mod common;

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn post_big_body() {
    common::setup();

    let stream = TcpStream::connect(("127.0.0.1", 9000)).await.unwrap();
    let mut client = Client::new_keep_alive(stream);

    let document_root = current_dir().unwrap().join("tests").join("php");
    let document_root = document_root.to_str().unwrap();
    let script_name = current_dir()
        .unwrap()
        .join("tests")
        .join("php")
        .join("body-size.php");
    let script_name = script_name.to_str().unwrap();

    let body = [0u8; 131072];
    let len = format!("{}", body.len());

    let params = Params::default()
        .set_request_method("POST")
        .set_document_root(document_root)
        .set_script_name("/body-size.php")
        .set_script_filename(script_name)
        .set_request_uri("/body-size.php")
        .set_query_string("")
        .set_document_uri("/body-size.php")
        .set_remote_addr("127.0.0.1")
        .set_remote_port("12345")
        .set_server_addr("127.0.0.1")
        .set_server_port("80")
        .set_server_name("jmjoy-pc")
        .set_content_type("text/plain")
        .set_content_length(&len);

    let output = timeout(
        Duration::from_secs(3),
        client.execute(Request::new(params.clone(), &mut &body[..])),
    )
    .await
    .unwrap()
    .unwrap();

    let stdout = String::from_utf8(output.get_stdout().unwrap_or(Default::default())).unwrap();
    assert!(stdout.contains("Content-type: text/html; charset=UTF-8"));
    assert!(stdout.contains("\r\n\r\n"));
    assert!(stdout.contains("131072"));
}

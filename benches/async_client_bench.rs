#![feature(test)]

extern crate test;

use fastcgi_client::{conn::KeepAlive, request::Request, Client, Params};
use std::env::current_dir;
use test::Bencher;
use tokio::{
    io::{self, AsyncRead, AsyncWrite},
    net::TcpStream,
};

mod common;

async fn test_client<S: AsyncRead + AsyncWrite + Unpin>(client: &mut Client<S, KeepAlive>) {
    let document_root = current_dir().unwrap().join("tests").join("php");
    let document_root = document_root.to_str().unwrap();
    let script_name = current_dir()
        .unwrap()
        .join("tests")
        .join("php")
        .join("index.php");
    let script_name = script_name.to_str().unwrap();

    let params = Params::default()
        .set_request_method("GET")
        .set_document_root(document_root)
        .set_script_name("/index.php")
        .set_script_filename(script_name)
        .set_request_uri("/index.php")
        .set_document_uri("/index.php")
        .set_remote_addr("127.0.0.1")
        .set_remote_port("12345")
        .set_server_addr("127.0.0.1")
        .set_server_port("80")
        .set_server_name("jmjoy-pc")
        .set_content_type("")
        .set_content_length("0");

    let output = client
        .execute(Request::new(params, &mut io::empty()))
        .await
        .unwrap();

    let stdout = String::from_utf8(output.get_stdout().unwrap_or(Default::default())).unwrap();
    assert!(stdout.contains("Content-type: text/html; charset=UTF-8"));
    assert!(stdout.contains("\r\n\r\n"));
    assert!(stdout.contains("hello"));
    assert_eq!(output.get_stderr(), None);
}

#[bench]
fn bench_execute(b: &mut Bencher) {
    common::setup();

    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(6)
        .enable_all()
        .build()
        .unwrap();

    let mut client = rt.block_on(async {
        let stream = TcpStream::connect(("127.0.0.1", 9000)).await.unwrap();
        Client::new_keep_alive(stream)
    });

    b.iter(|| {
        rt.block_on(async {
            test_client(&mut client).await;
        });
    });
}

use fastcgi_client::{request::Request, Client, Params};
use std::env::current_dir;
use tokio::net::TcpStream;
use tokio::time::timeout;
use std::time::Duration;

mod common;

#[tokio::test]
async fn test() {
    common::setup();

    let stream = TcpStream::connect(("127.0.0.1", 9000)).await.unwrap();
    let mut client = Client::new(stream, true);

    let document_root = current_dir().unwrap().join("tests").join("php");
    let document_root = document_root.to_str().unwrap();
    let script_name = current_dir()
        .unwrap()
        .join("tests")
        .join("php")
        .join("post.php");
    let script_name = script_name.to_str().unwrap();

    let body = b"p1=3&p2=4";
    let len = format!("{}", body.len());

    let params = Params::default()
        .set_request_method("POST")
        .set_document_root(document_root)
        .set_script_name("/post.php")
        .set_script_filename(script_name)
        .set_request_uri("/post.php?g1=1&g2=2")
        .set_query_string("g1=1&g2=2")
        .set_document_uri("/post.php")
        .set_remote_addr("127.0.0.1")
        .set_remote_port("12345")
        .set_server_addr("127.0.0.1")
        .set_server_port("80")
        .set_server_name("jmjoy-pc")
        .set_content_type("application/x-www-form-urlencoded")
        .set_content_length(&len);

    let _ = client
        .execute(Request::new(params.clone(), &mut &body[..]))
        .await
        .unwrap();
    let output = client
        .execute(Request::new(params, &mut &body[..]))
        .await
        .unwrap();

    let stdout = String::from_utf8(output.get_stdout().unwrap_or(Default::default())).unwrap();
    assert!(stdout.contains("Content-type: text/html; charset=UTF-8"));
    assert!(stdout.contains("\r\n\r\n"));
    assert!(stdout.contains("1234"));

    let stderr = String::from_utf8(output.get_stderr().unwrap_or(Default::default())).unwrap();
    let stderr = dbg!(stderr);
    assert!(stderr.contains("PHP message: PHP Fatal error:  Uncaught Exception: TEST"));
}

#[tokio::test]
async fn post_big_body() {
    common::setup();

    let stream = TcpStream::connect(("127.0.0.1", 9000)).await.unwrap();
    let mut client = Client::new(stream, true);

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

    let output = timeout(Duration::from_secs(3), client
        .execute(Request::new(params.clone(), &mut &body[..])))
        .await
        .unwrap()
        .unwrap();

    let stdout = String::from_utf8(output.get_stdout().unwrap_or(Default::default())).unwrap();
    assert!(stdout.contains("Content-type: text/html; charset=UTF-8"));
    assert!(stdout.contains("\r\n\r\n"));
    assert!(stdout.contains("131072"));
}

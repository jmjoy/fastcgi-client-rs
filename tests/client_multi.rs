use fastcgi_client::{request::Request, Client, Params};
use std::{env::current_dir, io::Cursor};
use tokio::net::TcpStream;

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

    for _ in 0..3 {
        let output = client
            .execute(Request::new(params.clone(), Cursor::new(body)))
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
}

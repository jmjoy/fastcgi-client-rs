use fastcgi_client::{Address, ClientBuilder, Params};
use std::env::current_dir;
use std::io;

mod common;

#[test]
fn test() {
    common::setup();

    let mut client = ClientBuilder::new(Address::Tcp("127.0.0.1", 9000)).build().unwrap();
    //    let mut client = ClientBuilder::new(Address::UnixSock("/run/php/php7.1-fpm.sock")).build().unwrap();

    let document_root = current_dir().unwrap().join("tests").join("php");
    let document_root = document_root.to_str().unwrap();
    let script_name = current_dir().unwrap().join("tests").join("php").join("index.php");
    let script_name = script_name.to_str().unwrap();

    let params = Params::with_predefine()
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
    let output = client.do_request(&params, &mut io::empty()).unwrap();

    let stdout = String::from_utf8(output.get_stdout().unwrap_or(Default::default())).unwrap();
    assert!(stdout.contains("Content-type: text/html; charset=UTF-8"));
    assert!(stdout.contains("\r\n\r\n"));
    assert!(stdout.contains("hello"));
    assert_eq!(output.get_stderr(), None);
}

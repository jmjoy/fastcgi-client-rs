use fastcgi_client::{Address, ClientBuilder, Params};
use std::env::current_dir;

use std::io;

#[test]
fn test_client() {
    env_logger::init();

    //    let mut client = ClientBuilder::new(Address::Tcp("127.0.0.1", 9000)).build().unwrap();
    let mut client = ClientBuilder::new(Address::UnixSock("/run/php/php7.1-fpm.sock")).build().unwrap();

    let script_name = current_dir().unwrap().join("tests").join("php").join("index.php");

    let script_name = script_name.to_str().unwrap();

    let params = Params::with_predefine()
        .set_request_method("GET")
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
    dbg!(&output);
    dbg!(String::from_utf8(output.get_stdout().unwrap_or(Default::default())));
    dbg!(String::from_utf8(output.get_stderr().unwrap_or(Default::default())));
}

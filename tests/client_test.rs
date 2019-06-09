use fastcgi_client::{Address, ClientBuilder, Params};
use std::io;
use std::error::Error;

#[test]
fn test_client() {
    let response = ClientBuilder::new(Address::Tcp("127.0.0.1", 9000))
        .build()
        .unwrap()
        .request(Params::with(
            "GET",
            "/home/jmjoy/workspace/rust/fastcgi-client-rs/tests/php/index.php",
            "",
            "/index.php", "/index.php",
            "127.0.0.1",
            "12345",
            "127.0.0.1",
            "80",
            "jmjoy-PC",
            "",
            "0",
        ), &mut io::empty());

    dbg!(&response);

    response.map_err(|e| {
        dbg!(e.source());
    }).map(|x| {
        dbg!(x);
    });
}

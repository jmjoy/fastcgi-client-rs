use fastcgi_client::{Address, ClientBuilder, Params};
use std::io;

#[test]
fn test_client() {
    ClientBuilder::new(Address::Tcp("127.0.0.1", 9000))
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
    assert_eq!("hello", "hello");
}

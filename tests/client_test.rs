use fastcgi_client::{Address, ClientBuilder, Params};
use std::io;

#[test]
fn test_client() {
    ClientBuilder::new(Address::Tcp("127.0.0.1", 9000))
        .build()
        .unwrap()
        .request(Params::with(
            "GET", "", "", "", "", "", "", "", "", "", "", "",
        ), &mut io::empty());
    assert_eq!("hello", "hello");
}

# fastcgi-client-rs

[![Rust](https://github.com/jmjoy/fastcgi-client-rs/actions/workflows/rust.yml/badge.svg)](https://github.com/jmjoy/fastcgi-client-rs/actions/workflows/rust.yml)
[![Crate](https://img.shields.io/crates/v/fastcgi-client.svg)](https://crates.io/crates/fastcgi-client)
[![API](https://docs.rs/fastcgi-client/badge.svg)](https://docs.rs/fastcgi-client)

Fastcgi client implemented for Rust, power by [tokio](https://crates.io/crates/tokio).

## Installation

Add dependencies to your `Cargo.toml` by `cargo add`:

```shell
cargo add tokio --features full
cargo add fastcgi-client
```

## Examples

Short connection mode:

```rust, no_run
use fastcgi_client::{Client, Params, Request};
use std::env;
use tokio::{io, net::TcpStream};

#[tokio::main]
async fn main() {
    let script_filename = env::current_dir()
        .unwrap()
        .join("tests")
        .join("php")
        .join("index.php");
    let script_filename = script_filename.to_str().unwrap();
    let script_name = "/index.php";

    // Connect to php-fpm default listening address.
    let stream = TcpStream::connect(("127.0.0.1", 9000)).await.unwrap();
    let mut client = Client::new(stream);

    // Fastcgi params, please reference to nginx-php-fpm config.
    let params = Params::default()
        .request_method("GET")
        .script_name(script_name)
        .script_filename(script_filename)
        .request_uri(script_name)
        .document_uri(script_name)
        .remote_addr("127.0.0.1")
        .remote_port(12345)
        .server_addr("127.0.0.1")
        .server_port(80)
        .server_name("jmjoy-pc")
        .content_type("")
        .content_length(0);

    // Fetch fastcgi server(php-fpm) response.
    let output = client.execute_once(Request::new(params, &mut io::empty())).await.unwrap();

    // "Content-type: text/html; charset=UTF-8\r\n\r\nhello"
    let stdout = String::from_utf8(output.stdout.unwrap()).unwrap();

    assert!(stdout.contains("Content-type: text/html; charset=UTF-8"));
    assert!(stdout.contains("hello"));
    assert_eq!(output.stderr, None);
}
```

Keep alive mode:

```rust, no_run
use fastcgi_client::{Client, Params, Request};
use std::env;
use tokio::{io, net::TcpStream};

#[tokio::main]
async fn main() {
    // Connect to php-fpm default listening address.
    let stream = TcpStream::connect(("127.0.0.1", 9000)).await.unwrap();
    let mut client = Client::new_keep_alive(stream);

    // Fastcgi params, please reference to nginx-php-fpm config.
    let params = Params::default();

    for _ in (0..3) {
        // Fetch fastcgi server(php-fpm) response.
        let output = client.execute(Request::new(params.clone(), &mut io::empty())).await.unwrap();

        // "Content-type: text/html; charset=UTF-8\r\n\r\nhello"
        let stdout = String::from_utf8(output.stdout.unwrap()).unwrap();

        assert!(stdout.contains("Content-type: text/html; charset=UTF-8"));
        assert!(stdout.contains("hello"));
        assert_eq!(output.stderr, None);
    }
}
```

## License

[Apache-2.0](https://github.com/jmjoy/fastcgi-client-rs/blob/master/LICENSE).

# fastcgi-client-rs

[![Rust](https://github.com/jmjoy/fastcgi-client-rs/actions/workflows/rust.yml/badge.svg)](https://github.com/jmjoy/fastcgi-client-rs/actions/workflows/rust.yml)
[![Crate](https://img.shields.io/crates/v/fastcgi-client.svg)](https://crates.io/crates/fastcgi-client)
[![API](https://docs.rs/fastcgi-client/badge.svg)](https://docs.rs/fastcgi-client)

Fastcgi client implemented for Rust with optional runtime support for
[tokio](https://crates.io/crates/tokio) and [smol](https://crates.io/crates/smol).

## Installation

Choose one or both runtime features explicitly:

Tokio:

```shell
cargo add fastcgi-client --features runtime-tokio
cargo add tokio --features full
```

Smol:

```shell
cargo add fastcgi-client --features runtime-smol
cargo add smol
```

Both runtimes:

```shell
cargo add fastcgi-client --features runtime-tokio,runtime-smol
cargo add tokio --features full
cargo add smol
```

## Examples

Tokio short connection mode:

```rust, no_run
# #[cfg(feature = "runtime-tokio")]
# async fn example() {
use fastcgi_client::{io, Client, Params, Request};
use std::env;
use tokio::net::TcpStream;

    let script_filename = env::current_dir()
        .unwrap()
        .join("tests")
        .join("php")
        .join("index.php");
    let script_filename = script_filename.to_str().unwrap();
    let script_name = "/index.php";

    // Connect to php-fpm default listening address.
    let stream = TcpStream::connect(("127.0.0.1", 9000)).await.unwrap();
    let client = Client::new_tokio(stream);

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
    let output = client.execute_once(Request::new(params, io::empty())).await.unwrap();

    // "Content-type: text/html; charset=UTF-8\r\n\r\nhello"
    let stdout = String::from_utf8(output.stdout.unwrap()).unwrap();

    assert!(stdout.contains("Content-type: text/html; charset=UTF-8"));
    assert!(stdout.contains("hello"));
    assert_eq!(output.stderr, None);
}
# #[cfg(not(feature = "runtime-tokio"))]
# fn example() {}
```

Tokio keep alive mode:

```rust, no_run
# #[cfg(feature = "runtime-tokio")]
# async fn example() {
use fastcgi_client::{io, Client, Params, Request};
use tokio::net::TcpStream;

    // Connect to php-fpm default listening address.
    let stream = TcpStream::connect(("127.0.0.1", 9000)).await.unwrap();
    let mut client = Client::new_keep_alive_tokio(stream);

    // Fastcgi params, please reference to nginx-php-fpm config.
    let params = Params::default();

    for _ in (0..3) {
        // Fetch fastcgi server(php-fpm) response.
        let output = client.execute(Request::new(params.clone(), io::empty())).await.unwrap();

        // "Content-type: text/html; charset=UTF-8\r\n\r\nhello"
        let stdout = String::from_utf8(output.stdout.unwrap()).unwrap();

        assert!(stdout.contains("Content-type: text/html; charset=UTF-8"));
        assert!(stdout.contains("hello"));
        assert_eq!(output.stderr, None);
    }
}
# #[cfg(not(feature = "runtime-tokio"))]
# fn example() {}
```

Smol short connection mode:

```rust, no_run
# #[cfg(feature = "runtime-smol")]
# async fn example() {
use fastcgi_client::{io, Client, Params, Request};
use std::env;
use smol::net::TcpStream;

    let script_filename = env::current_dir()
        .unwrap()
        .join("tests")
        .join("php")
        .join("index.php");
    let script_filename = script_filename.to_str().unwrap();
    let script_name = "/index.php";

    let stream = TcpStream::connect(("127.0.0.1", 9000)).await.unwrap();
    let client = Client::new_smol(stream);

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

    let output = client.execute_once(Request::new(params, io::empty())).await.unwrap();

    let stdout = String::from_utf8(output.stdout.unwrap()).unwrap();

    assert!(stdout.contains("Content-type: text/html; charset=UTF-8"));
    assert!(stdout.contains("hello"));
    assert_eq!(output.stderr, None);
}
# #[cfg(not(feature = "runtime-smol"))]
# fn example() {}
```

Smol keep alive mode:

```rust, no_run
# #[cfg(feature = "runtime-smol")]
# async fn example() {
use fastcgi_client::{io, Client, Params, Request};
use smol::net::TcpStream;

    let stream = TcpStream::connect(("127.0.0.1", 9000)).await.unwrap();
    let mut client = Client::new_keep_alive_smol(stream);

    let params = Params::default();

    for _ in (0..3) {
        let output = client.execute(Request::new(params.clone(), io::empty())).await.unwrap();

        let stdout = String::from_utf8(output.stdout.unwrap()).unwrap();

        assert!(stdout.contains("Content-type: text/html; charset=UTF-8"));
        assert!(stdout.contains("hello"));
        assert_eq!(output.stderr, None);
    }
}
# #[cfg(not(feature = "runtime-smol"))]
# fn example() {}
```

## Optional HTTP conversions

Enable the `http` feature if you want to convert between this crate's FastCGI
types and the `http` crate.

```toml
fastcgi-client = { version = "0.10", features = ["http", "runtime-tokio"] }
```

The conversion boundary is intentionally split in two:

- `Request<'a, I>` can convert into `http::Request<I>` without buffering the body.
- `http::Request<I>` can convert back into FastCGI metadata, but CGI-only params
    such as `SCRIPT_FILENAME` must be supplied explicitly through extra `Params`.
- `Response` can be parsed into `http::Response<Vec<u8>>`; `stderr` remains
    available only on the original FastCGI response.

## License

[Apache-2.0](https://github.com/jmjoy/fastcgi-client-rs/blob/master/LICENSE).

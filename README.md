# fastcgi-client-rs

[![Rust](https://github.com/jmjoy/fastcgi-client-rs/actions/workflows/rust.yml/badge.svg)](https://github.com/jmjoy/fastcgi-client-rs/actions/workflows/rust.yml)
[![Crate](https://img.shields.io/crates/v/fastcgi-client.svg)](https://crates.io/crates/fastcgi-client)
[![API](https://docs.rs/fastcgi-client/badge.svg)](https://docs.rs/fastcgi-client)

Runtime-agnostic Fastcgi client implemented for Rust.

The client is built on the [futures-io](https://crates.io/crates/futures-io)
`AsyncRead` / `AsyncWrite` traits and contains no task spawning, timers or
socket creation, so it works with any async runtime, and even with no runtime at
all.

## Installation

No cargo feature is required:

```shell
cargo add fastcgi-client
```

Any stream implementing the `futures-io` traits works out of the box, for
example [smol](https://crates.io/crates/smol) or
[async-net](https://crates.io/crates/async-net):

```shell
cargo add smol
```

Tokio defines its own I/O traits, so its streams need a compatibility wrapper.
Enable the optional `tokio` feature to get convenience constructors that do the
wrapping for you:

```shell
cargo add fastcgi-client --features tokio
cargo add tokio --features full
```

Without the `tokio` feature you can still use Tokio by bridging the stream
yourself with [tokio-util](https://crates.io/crates/tokio-util):

```rust, no_run
use fastcgi_client::Client;
use tokio::net::TcpStream;
use tokio_util::compat::TokioAsyncReadCompatExt;

#[tokio::main]
async fn main() {
    let stream = TcpStream::connect(("127.0.0.1", 9000)).await.unwrap();
    let _client = Client::new(stream.compat());
}
```

## Examples

Short connection mode:

```rust, no_run
use fastcgi_client::{io, Client, Params, Request};
use std::env;

fn main() {
    smol::block_on(async {
        let script_filename = env::current_dir()
            .unwrap()
            .join("tests")
            .join("php")
            .join("index.php");
        let script_filename = script_filename.to_str().unwrap();
        let script_name = "/index.php";

        // Connect to php-fpm default listening address.
        let stream = smol::net::TcpStream::connect(("127.0.0.1", 9000))
            .await
            .unwrap();
        let client = Client::new(stream);

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
        let output = client
            .execute_once(Request::new(params, io::empty()))
            .await
            .unwrap();

        // "Content-type: text/html; charset=UTF-8\r\n\r\nhello"
        let stdout = String::from_utf8(output.stdout.unwrap()).unwrap();

        assert!(stdout.contains("Content-type: text/html; charset=UTF-8"));
        assert!(stdout.contains("hello"));
        assert_eq!(output.stderr, None);
    });
}
```

Keep alive mode:

```rust, no_run
use fastcgi_client::{io, Client, Params, Request};

fn main() {
    smol::block_on(async {
        // Connect to php-fpm default listening address.
        let stream = smol::net::TcpStream::connect(("127.0.0.1", 9000))
            .await
            .unwrap();
        let mut client = Client::new_keep_alive(stream);

        // Fastcgi params, please reference to nginx-php-fpm config.
        let params = Params::default();

        for _ in 0..3 {
            // Fetch fastcgi server(php-fpm) response.
            let output = client
                .execute(Request::new(params.clone(), io::empty()))
                .await
                .unwrap();

            // "Content-type: text/html; charset=UTF-8\r\n\r\nhello"
            let stdout = String::from_utf8(output.stdout.unwrap()).unwrap();

            assert!(stdout.contains("Content-type: text/html; charset=UTF-8"));
            assert!(stdout.contains("hello"));
            assert_eq!(output.stderr, None);
        }
    });
}
```

With the `tokio` feature, `Client::new_tokio` and `Client::new_keep_alive_tokio`
accept Tokio streams directly:

```rust, ignore
use fastcgi_client::{io, Client, Params, Request};
use tokio::net::TcpStream;

#[tokio::main]
async fn main() {
    // Short connection mode.
    let stream = TcpStream::connect(("127.0.0.1", 9000)).await.unwrap();
    let client = Client::new_tokio(stream);
    let output = client
        .execute_once(Request::new(Params::default(), io::empty()))
        .await
        .unwrap();
    assert!(String::from_utf8(output.stdout.unwrap()).unwrap().contains("hello"));

    // Keep alive mode.
    let stream = TcpStream::connect(("127.0.0.1", 9000)).await.unwrap();
    let mut client = Client::new_keep_alive_tokio(stream);
    for _ in 0..3 {
        let output = client
            .execute(Request::new(Params::default(), io::empty()))
            .await
            .unwrap();
        assert!(String::from_utf8(output.stdout.unwrap()).unwrap().contains("hello"));
    }
}
```

Runnable versions of every flow, including response streaming and an Axum
HTTP-to-FastCGI proxy, live under
[`examples/`](https://github.com/jmjoy/fastcgi-client-rs/blob/master/examples/README.md).

## Optional HTTP conversions

Enable the `http` feature if you want to convert between this crate's FastCGI
types and the `http` crate.

```toml
fastcgi-client = { version = "0.12", features = ["http"] }
```

The conversion boundary is intentionally split in two:

- `Request<'a, I>` can convert into `http::Request<I>` without buffering the body.
- `http::Request<I>` can convert back into FastCGI metadata, but CGI-only params
    such as `SCRIPT_FILENAME` must be supplied explicitly through extra `Params`.
- `Response` can be parsed into `http::Response<Vec<u8>>`; `stderr` remains
    available only on the original FastCGI response.

## License

[Apache-2.0](https://github.com/jmjoy/fastcgi-client-rs/blob/master/LICENSE).

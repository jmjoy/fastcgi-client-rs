# fastcgi-client-rs

[![Build Status](https://travis-ci.org/jmjoy/fastcgi-client-rs.svg?branch=master)](https://travis-ci.org/jmjoy/fastcgi-client-rs)
[![Crate](https://img.shields.io/crates/v/fastcgi-client.svg)](https://crates.io/crates/fastcgi-client)
[![API](https://docs.rs/fastcgi-client/badge.svg)](https://docs.rs/fastcgi-client)

![fastcgi-client-rs](https://raw.githubusercontent.com/jmjoy/fastcgi-client-rs/master/fastcgi-client-rs.png)

Async Fastcgi client implemented for Rust.

**Notice: This crate is not productive yet, please do not use in production.**

## Example

```rust
use fastcgi_client::{Client, Params};
use std::{env, io};
use tokio::net::TcpStream;

let script_filename = env::current_dir()
    .unwrap()
    .join("tests")
    .join("php")
    .join("index.php");
let script_filename = script_filename.to_str().unwrap();
let script_name = "/index.php";

// Connect to php-fpm default listening address.
let addr: SocketAddr = "127.0.0.1:9000".parse().unwrap();
let stream = TcpStream::connect(&addr).await.unwrap();
let mut client = Client::new(stream, false);

// Fastcgi params, please reference to nginx-php-fpm config.
let params = Params::with_predefine()
    .set_request_method("GET")
    .set_script_name(script_name)
    .set_script_filename(script_filename)
    .set_request_uri(script_name)
    .set_document_uri(script_name)
    .set_remote_addr("127.0.0.1")
    .set_remote_port("12345")
    .set_server_addr("127.0.0.1")
    .set_server_port("80")
    .set_server_name("jmjoy-pc")
    .set_content_type("")
    .set_content_length("0");

// Fetch fastcgi server(php-fpm) response.
let output = client.do_request(&params, &mut io::empty()).await.unwrap();

// "Content-type: text/html; charset=UTF-8\r\n\r\nhello"
let stdout = String::from_utf8(output.get_stdout().unwrap()).unwrap();

assert!(stdout.contains("Content-type: text/html; charset=UTF-8"));
assert!(stdout.contains("hello"));
assert_eq!(output.get_stderr(), None);
```

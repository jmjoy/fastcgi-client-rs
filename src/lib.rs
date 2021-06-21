#![warn(rust_2018_idioms, clippy::dbg_macro, clippy::print_stdout)]

//! Fastcgi client implemented for Rust, power by [tokio](https://crates.io/crates/tokio).
//!
//! ## Installation
//!
//! Add this to your Cargo.toml:
//!
//! ```toml
//! [dependencies]
//! fastcgi-client = "0.7"
//! tokio = { version = "1", features = ["full"] }
//! ```
//!
//! ## Examples
//!
//! ```
//! use fastcgi_client::{Client, Params, Request};
//! use std::env;
//! use tokio::{io, task};
//! use tokio::net::TcpStream;
//!
//! #[tokio::main]
//! async fn main() {
//!     let script_filename = env::current_dir()
//!         .unwrap()
//!         .join("tests")
//!         .join("php")
//!         .join("index.php");
//!     let script_filename = script_filename.to_str().unwrap();
//!     let script_name = "/index.php";
//!
//!     // Connect to php-fpm default listening address.
//!     let stream = TcpStream::connect(("127.0.0.1", 9000)).await.unwrap();
//!     let mut client = Client::new(stream, false);
//!
//!     // Fastcgi params, please reference to nginx-php-fpm config.
//!     let params = Params::default()
//!         .set_request_method("GET")
//!         .set_script_name(script_name)
//!         .set_script_filename(script_filename)
//!         .set_request_uri(script_name)
//!         .set_document_uri(script_name)
//!         .set_remote_addr("127.0.0.1")
//!         .set_remote_port("12345")
//!         .set_server_addr("127.0.0.1")
//!         .set_server_port("80")
//!         .set_server_name("jmjoy-pc")
//!         .set_content_type("")
//!         .set_content_length("0");
//!
//!     // Fetch fastcgi server(php-fpm) response.
//!     let output = client.execute(Request::new(params, &mut io::empty())).await.unwrap();
//!
//!     // "Content-type: text/html; charset=UTF-8\r\n\r\nhello"
//!     let stdout = String::from_utf8(output.get_stdout().unwrap()).unwrap();
//!
//!     assert!(stdout.contains("Content-type: text/html; charset=UTF-8"));
//!     assert!(stdout.contains("hello"));
//!     assert_eq!(output.get_stderr(), None);
//! }
//! ```
//!
//! ## License
//! [MIT](https://github.com/jmjoy/fastcgi-client-rs/blob/master/LICENSE).
//!

pub mod client;
mod error;
mod id;
mod meta;
pub mod params;
pub mod request;
pub mod response;

pub use crate::{client::Client, error::*, params::Params, request::Request, response::Response};

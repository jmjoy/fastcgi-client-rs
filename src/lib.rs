//! Fastcgi client implemented for Rust.
//!
//! ![fastcgi-client-rs](https://raw.githubusercontent.com/jmjoy/fastcgi-client-rs/master/fastcgi-client-rs.png)
//!
//! ### Example
//! ```
//! use fastcgi_client::{Client, Params};
//! use std::{env, io};
//! use std::net::TcpStream;
//!
//! let script_filename = env::current_dir()
//!     .unwrap()
//!     .join("tests")
//!     .join("php")
//!     .join("index.php");
//! let script_filename = script_filename.to_str().unwrap();
//! let script_name = "/index.php";
//!
//! // Connect to php-fpm default listening address.
//! let stream = TcpStream::connect(("127.0.0.1", 9000)).unwrap();
//! let mut client = Client::new(stream, false);
//!
//! // Fastcgi params, please reference to nginx-php-fpm config.
//! let params = Params::with_predefine()
//!     .set_request_method("GET")
//!     .set_script_name(script_name)
//!     .set_script_filename(script_filename)
//!     .set_request_uri(script_name)
//!     .set_document_uri(script_name)
//!     .set_remote_addr("127.0.0.1")
//!     .set_remote_port("12345")
//!     .set_server_addr("127.0.0.1")
//!     .set_server_port("80")
//!     .set_server_name("jmjoy-pc")
//!     .set_content_type("")
//!     .set_content_length("0");
//!
//! // Fetch fastcgi server(php-fpm) response.
//! let output = client.do_request(&params, &mut io::empty()).unwrap();
//!
//! // "Content-type: text/html; charset=UTF-8\r\n\r\nhello"
//! let stdout = String::from_utf8(output.get_stdout().unwrap()).unwrap();
//!
//! assert!(stdout.contains("Content-type: text/html; charset=UTF-8"));
//! assert!(stdout.contains("hello"));
//! assert_eq!(output.get_stderr(), None);
//! ```
//!

mod client;
mod error;
mod id;
mod meta;
mod params;
mod empty;

pub use crate::client::Client;
pub use crate::error::*;
pub use crate::meta::{Address, Output};
pub use crate::params::Params;
pub use crate::empty::*;

/// Version of this crate.
pub const VERSION: &'static str = env!("LIB_BUILD_VERSION");

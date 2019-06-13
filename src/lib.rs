use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use log::{debug, info};
use std::cmp::min;
use std::collections::HashMap;
use std::fmt::{self, Display, Formatter};
use std::io::{self, ErrorKind, Read, Write};
use std::mem::size_of;
use std::net::{TcpStream, ToSocketAddrs};
use std::sync::atomic::AtomicU16;
use std::time::Duration;

//
//const TYPE_BEGIN_REQUEST: u8 = 1;
//const TYPE_ABORT_REQUEST: u8 = 2;
//const TYPE_END_REQUEST: u8 = 3;
//const TYPE_PARAMS: u8 = 4;
//const TYPE_STDIN: u8 = 5;
//const TYPE_STDOUT: u8 = 6;
//const TYPE_STDERR: u8 = 7;
//const TYPE_DATA: u8 = 8;
//const TYPE_GET_VALUES: u8 = 9;
//const TYPE_GET_VALUES_RESULT: u8 = 10;
//const TYPE_UNKNOWN_TYPE: u8 = 11;
//const TYPE_MAX_TYPE: u8 = TYPE_UNKNOWN_TYPE;
//
//const ROLE_RESPONDER: u8 = 1;
//const ROLE_AUTHORIZER: u8 = 2;
//const ROLE_FILTER: u8 = 3;
//
//const STATUS_REQUEST_COMPLETE: u8 = 0;
//const STATUS_CANT_MPX_CONN: u8 = 1;
//const STATUS_OVERLOADED: u8 = 2;
//const STATUS_UNKNOWN_ROLE: u8 = 3;
//
//

mod client;
mod error;
mod id;
mod meta;
mod params;

pub use crate::client::{Client, ClientBuilder};
pub use crate::error::{ClientError, ClientResult};
pub use crate::meta::Address;
pub use crate::params::Params;

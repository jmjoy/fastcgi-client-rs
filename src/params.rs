// Copyright 2022 jmjoy
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::{
    borrow::Cow,
    collections::HashMap,
    ops::{Deref, DerefMut},
};

/// Fastcgi params, please reference to nginx-php-fpm fastcgi_params.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Params<'a>(HashMap<Cow<'a, str>, Cow<'a, str>>);

impl<'a> Params<'a> {
    #[inline]
    pub fn custom<K: Into<Cow<'a, str>>, S: Into<Cow<'a, str>>>(mut self, key: K, value: S) -> Self {
        self.insert(k.into(), v.into());
        self
    }

    #[inline]
    pub fn gateway_interface<S: Into<Cow<'a, str>>>(mut self, gateway_interface: S) -> Self {
        self.insert("GATEWAY_INTERFACE".into(), gateway_interface.into());
        self
    }

    #[inline]
    pub fn server_software<S: Into<Cow<'a, str>>>(mut self, server_software: S) -> Self {
        self.insert("SERVER_SOFTWARE".into(), server_software.into());
        self
    }

    #[inline]
    pub fn server_protocol<S: Into<Cow<'a, str>>>(mut self, server_protocol: S) -> Self {
        self.insert("SERVER_PROTOCOL".into(), server_protocol.into());
        self
    }

    #[inline]
    pub fn request_method<S: Into<Cow<'a, str>>>(mut self, request_method: S) -> Self {
        self.insert("REQUEST_METHOD".into(), request_method.into());
        self
    }

    #[inline]
    pub fn script_filename<S: Into<Cow<'a, str>>>(mut self, script_filename: S) -> Self {
        self.insert("SCRIPT_FILENAME".into(), script_filename.into());
        self
    }

    #[inline]
    pub fn script_name<S: Into<Cow<'a, str>>>(mut self, script_name: S) -> Self {
        self.insert("SCRIPT_NAME".into(), script_name.into());
        self
    }

    #[inline]
    pub fn query_string<S: Into<Cow<'a, str>>>(mut self, query_string: S) -> Self {
        self.insert("QUERY_STRING".into(), query_string.into());
        self
    }

    #[inline]
    pub fn request_uri<S: Into<Cow<'a, str>>>(mut self, request_uri: S) -> Self {
        self.insert("REQUEST_URI".into(), request_uri.into());
        self
    }

    #[inline]
    pub fn document_root<S: Into<Cow<'a, str>>>(mut self, document_root: S) -> Self {
        self.insert("DOCUMENT_ROOT".into(), document_root.into());
        self
    }

    #[inline]
    pub fn document_uri<S: Into<Cow<'a, str>>>(mut self, document_uri: S) -> Self {
        self.insert("DOCUMENT_URI".into(), document_uri.into());
        self
    }

    #[inline]
    pub fn remote_addr<S: Into<Cow<'a, str>>>(mut self, remote_addr: S) -> Self {
        self.insert("REMOTE_ADDR".into(), remote_addr.into());
        self
    }

    #[inline]
    pub fn remote_port(mut self, remote_port: u16) -> Self {
        self.insert("REMOTE_PORT".into(), remote_port.to_string().into());
        self
    }

    #[inline]
    pub fn server_addr<S: Into<Cow<'a, str>>>(mut self, server_addr: S) -> Self {
        self.insert("SERVER_ADDR".into(), server_addr.into());
        self
    }

    #[inline]
    pub fn server_port(mut self, server_port: u16) -> Self {
        self.insert("SERVER_PORT".into(), server_port.to_string().into());
        self
    }

    #[inline]
    pub fn server_name<S: Into<Cow<'a, str>>>(mut self, server_name: S) -> Self {
        self.insert("SERVER_NAME".into(), server_name.into());
        self
    }

    #[inline]
    pub fn content_type<S: Into<Cow<'a, str>>>(mut self, content_type: S) -> Self {
        self.insert("CONTENT_TYPE".into(), content_type.into());
        self
    }

    #[inline]
    pub fn content_length(mut self, content_length: usize) -> Self {
        self.insert("CONTENT_LENGTH".into(), content_length.to_string().into());
        self
    }
}

impl<'a> Default for Params<'a> {
    fn default() -> Self {
        Params(HashMap::new())
            .gateway_interface("FastCGI/1.0")
            .server_software("fastcgi-client-rs")
            .server_protocol("HTTP/1.1")
    }
}

impl<'a> Deref for Params<'a> {
    type Target = HashMap<Cow<'a, str>, Cow<'a, str>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> DerefMut for Params<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'a> From<Params<'a>> for HashMap<Cow<'a, str>, Cow<'a, str>> {
    fn from(params: Params<'a>) -> Self {
        params.0
    }
}

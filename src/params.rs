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
    collections::HashMap,
    ops::{Deref, DerefMut},
};

/// Fastcgi params, please reference to nginx-php-fpm fastcgi_params.
#[derive(Debug, Clone)]
pub struct Params<'a>(HashMap<&'a str, &'a str>);

impl<'a> Params<'a> {
    pub fn set_gateway_interface(mut self, gateway_interface: &'a str) -> Self {
        self.insert("GATEWAY_INTERFACE", gateway_interface);
        self
    }

    pub fn set_server_software(mut self, server_software: &'a str) -> Self {
        self.insert("SERVER_SOFTWARE", server_software);
        self
    }

    pub fn set_server_protocol(mut self, server_protocol: &'a str) -> Self {
        self.insert("SERVER_PROTOCOL", server_protocol);
        self
    }

    pub fn set_request_method(mut self, request_method: &'a str) -> Self {
        self.insert("REQUEST_METHOD", request_method);
        self
    }

    pub fn set_script_filename(mut self, script_filename: &'a str) -> Self {
        self.insert("SCRIPT_FILENAME", script_filename);
        self
    }

    pub fn set_script_name(mut self, script_name: &'a str) -> Self {
        self.insert("SCRIPT_NAME", script_name);
        self
    }

    pub fn set_query_string(mut self, query_string: &'a str) -> Self {
        self.insert("QUERY_STRING", query_string);
        self
    }

    pub fn set_request_uri(mut self, request_uri: &'a str) -> Self {
        self.insert("REQUEST_URI", request_uri);
        self
    }

    pub fn set_document_root(mut self, document_root: &'a str) -> Self {
        self.insert("DOCUMENT_ROOT", document_root);
        self
    }

    pub fn set_document_uri(mut self, document_uri: &'a str) -> Self {
        self.insert("DOCUMENT_URI", document_uri);
        self
    }

    pub fn set_remote_addr(mut self, remote_addr: &'a str) -> Self {
        self.insert("REMOTE_ADDR", remote_addr);
        self
    }

    pub fn set_remote_port(mut self, remote_port: &'a str) -> Self {
        self.insert("REMOTE_PORT", remote_port);
        self
    }

    pub fn set_server_addr(mut self, server_addr: &'a str) -> Self {
        self.insert("SERVER_ADDR", server_addr);
        self
    }

    pub fn set_server_port(mut self, server_port: &'a str) -> Self {
        self.insert("SERVER_PORT", server_port);
        self
    }

    pub fn set_server_name(mut self, server_name: &'a str) -> Self {
        self.insert("SERVER_NAME", server_name);
        self
    }

    pub fn set_content_type(mut self, content_type: &'a str) -> Self {
        self.insert("CONTENT_TYPE", content_type);
        self
    }

    pub fn set_content_length(mut self, content_length: &'a str) -> Self {
        self.insert("CONTENT_LENGTH", content_length);
        self
    }
}

impl<'a> Default for Params<'a> {
    fn default() -> Self {
        Params(HashMap::new())
            .set_gateway_interface("FastCGI/1.0")
            .set_server_software("fastcgi-client-rs")
            .set_server_protocol("HTTP/1.1")
    }
}

impl<'a> Deref for Params<'a> {
    type Target = HashMap<&'a str, &'a str>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> DerefMut for Params<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

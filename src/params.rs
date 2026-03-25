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

//! FastCGI parameters builder and container.
//!
//! This module provides the `Params` struct which acts as a builder
//! for FastCGI parameters that are sent to the FastCGI server.
//! It includes convenient methods for setting common CGI parameters.

use std::{
    borrow::Cow,
    collections::HashMap,
    ops::{Deref, DerefMut},
};

#[cfg(feature = "http")]
use std::str::FromStr;

#[cfg(feature = "http")]
use crate::{HttpConversionError, HttpConversionResult};

/// Fastcgi params, please reference to nginx-php-fpm fastcgi_params.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Params<'a>(HashMap<Cow<'a, str>, Cow<'a, str>>);

impl<'a> Params<'a> {
    /// Sets a custom parameter with the given key and value.
    ///
    /// # Arguments
    ///
    /// * `key` - The parameter name
    /// * `value` - The parameter value
    #[inline]
    pub fn custom<K: Into<Cow<'a, str>>, S: Into<Cow<'a, str>>>(
        mut self, key: K, value: S,
    ) -> Self {
        self.insert(key.into(), value.into());
        self
    }

    /// Sets the GATEWAY_INTERFACE parameter.
    ///
    /// # Arguments
    ///
    /// * `gateway_interface` - The gateway interface version (e.g., "CGI/1.1")
    #[inline]
    pub fn gateway_interface<S: Into<Cow<'a, str>>>(mut self, gateway_interface: S) -> Self {
        self.insert("GATEWAY_INTERFACE".into(), gateway_interface.into());
        self
    }

    /// Sets the SERVER_SOFTWARE parameter.
    ///
    /// # Arguments
    ///
    /// * `server_software` - The server software name and version
    #[inline]
    pub fn server_software<S: Into<Cow<'a, str>>>(mut self, server_software: S) -> Self {
        self.insert("SERVER_SOFTWARE".into(), server_software.into());
        self
    }

    /// Sets the SERVER_PROTOCOL parameter.
    ///
    /// # Arguments
    ///
    /// * `server_protocol` - The server protocol version (e.g., "HTTP/1.1")
    #[inline]
    pub fn server_protocol<S: Into<Cow<'a, str>>>(mut self, server_protocol: S) -> Self {
        self.insert("SERVER_PROTOCOL".into(), server_protocol.into());
        self
    }

    /// Sets the REQUEST_METHOD parameter.
    ///
    /// # Arguments
    ///
    /// * `request_method` - The HTTP request method (e.g., "GET", "POST")
    #[inline]
    pub fn request_method<S: Into<Cow<'a, str>>>(mut self, request_method: S) -> Self {
        self.insert("REQUEST_METHOD".into(), request_method.into());
        self
    }

    /// Sets the SCRIPT_FILENAME parameter.
    ///
    /// # Arguments
    ///
    /// * `script_filename` - The full path to the script file
    #[inline]
    pub fn script_filename<S: Into<Cow<'a, str>>>(mut self, script_filename: S) -> Self {
        self.insert("SCRIPT_FILENAME".into(), script_filename.into());
        self
    }

    /// Sets the SCRIPT_NAME parameter.
    ///
    /// # Arguments
    ///
    /// * `script_name` - The URI part that identifies the script
    #[inline]
    pub fn script_name<S: Into<Cow<'a, str>>>(mut self, script_name: S) -> Self {
        self.insert("SCRIPT_NAME".into(), script_name.into());
        self
    }

    /// Sets the QUERY_STRING parameter.
    ///
    /// # Arguments
    ///
    /// * `query_string` - The query string part of the URL
    #[inline]
    pub fn query_string<S: Into<Cow<'a, str>>>(mut self, query_string: S) -> Self {
        self.insert("QUERY_STRING".into(), query_string.into());
        self
    }

    /// Sets the REQUEST_URI parameter.
    ///
    /// # Arguments
    ///
    /// * `request_uri` - The full request URI
    #[inline]
    pub fn request_uri<S: Into<Cow<'a, str>>>(mut self, request_uri: S) -> Self {
        self.insert("REQUEST_URI".into(), request_uri.into());
        self
    }

    /// Sets the DOCUMENT_ROOT parameter.
    ///
    /// # Arguments
    ///
    /// * `document_root` - The document root directory path
    #[inline]
    pub fn document_root<S: Into<Cow<'a, str>>>(mut self, document_root: S) -> Self {
        self.insert("DOCUMENT_ROOT".into(), document_root.into());
        self
    }

    /// Sets the DOCUMENT_URI parameter.
    ///
    /// # Arguments
    ///
    /// * `document_uri` - The document URI
    #[inline]
    pub fn document_uri<S: Into<Cow<'a, str>>>(mut self, document_uri: S) -> Self {
        self.insert("DOCUMENT_URI".into(), document_uri.into());
        self
    }

    /// Sets the REMOTE_ADDR parameter.
    ///
    /// # Arguments
    ///
    /// * `remote_addr` - The remote client IP address
    #[inline]
    pub fn remote_addr<S: Into<Cow<'a, str>>>(mut self, remote_addr: S) -> Self {
        self.insert("REMOTE_ADDR".into(), remote_addr.into());
        self
    }

    /// Sets the REMOTE_PORT parameter.
    ///
    /// # Arguments
    ///
    /// * `remote_port` - The remote client port number
    #[inline]
    pub fn remote_port(mut self, remote_port: u16) -> Self {
        self.insert("REMOTE_PORT".into(), remote_port.to_string().into());
        self
    }

    /// Sets the SERVER_ADDR parameter.
    ///
    /// # Arguments
    ///
    /// * `server_addr` - The server IP address
    #[inline]
    pub fn server_addr<S: Into<Cow<'a, str>>>(mut self, server_addr: S) -> Self {
        self.insert("SERVER_ADDR".into(), server_addr.into());
        self
    }

    /// Sets the SERVER_PORT parameter.
    ///
    /// # Arguments
    ///
    /// * `server_port` - The server port number
    #[inline]
    pub fn server_port(mut self, server_port: u16) -> Self {
        self.insert("SERVER_PORT".into(), server_port.to_string().into());
        self
    }

    /// Sets the SERVER_NAME parameter.
    ///
    /// # Arguments
    ///
    /// * `server_name` - The server name or hostname
    #[inline]
    pub fn server_name<S: Into<Cow<'a, str>>>(mut self, server_name: S) -> Self {
        self.insert("SERVER_NAME".into(), server_name.into());
        self
    }

    /// Sets the CONTENT_TYPE parameter.
    ///
    /// # Arguments
    ///
    /// * `content_type` - The content type of the request body
    #[inline]
    pub fn content_type<S: Into<Cow<'a, str>>>(mut self, content_type: S) -> Self {
        self.insert("CONTENT_TYPE".into(), content_type.into());
        self
    }

    /// Sets the CONTENT_LENGTH parameter.
    ///
    /// # Arguments
    ///
    /// * `content_length` - The length of the request body in bytes
    #[inline]
    pub fn content_length(mut self, content_length: usize) -> Self {
        self.insert("CONTENT_LENGTH".into(), content_length.to_string().into());
        self
    }
}

impl Default for Params<'_> {
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

impl DerefMut for Params<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'a> From<Params<'a>> for HashMap<Cow<'a, str>, Cow<'a, str>> {
    fn from(params: Params<'a>) -> Self {
        params.0
    }
}

#[cfg(feature = "http")]
impl<'a> TryFrom<&Params<'a>> for ::http::request::Parts {
    type Error = HttpConversionError;

    fn try_from(params: &Params<'a>) -> Result<Self, Self::Error> {
        let method =
            ::http::Method::from_bytes(required_param(params, "REQUEST_METHOD")?.as_bytes())?;
        let version = params
            .get("SERVER_PROTOCOL")
            .map(|protocol| parse_server_protocol(protocol))
            .transpose()?
            .unwrap_or(::http::Version::HTTP_11);
        let uri = build_request_uri(params)?;

        let mut builder = ::http::Request::builder()
            .method(method)
            .uri(uri)
            .version(version);
        let headers = builder
            .headers_mut()
            .expect("request builder should provide headers");
        for (name, value) in params_to_headers(params)? {
            headers.append(name, value);
        }
        let (parts, _) = builder.body(())?.into_parts();
        Ok(parts)
    }
}

#[cfg(feature = "http")]
impl<'a> TryFrom<Params<'a>> for ::http::request::Parts {
    type Error = HttpConversionError;

    fn try_from(params: Params<'a>) -> Result<Self, Self::Error> {
        (&params).try_into()
    }
}

#[cfg(feature = "http")]
impl TryFrom<&::http::request::Parts> for Params<'static> {
    type Error = HttpConversionError;

    fn try_from(parts: &::http::request::Parts) -> Result<Self, Self::Error> {
        let mut params = Params::default().request_method(parts.method.as_str().to_owned());

        if let Some(path_and_query) = parts.uri.path_and_query() {
            params = params
                .request_uri(path_and_query.as_str().to_owned())
                .document_uri(parts.uri.path().to_owned());
        } else {
            params = params.request_uri("/").document_uri("/");
        }

        if let Some(query) = parts.uri.query() {
            params = params.query_string(query.to_owned());
        }

        params = params.server_protocol(version_to_server_protocol(parts.version));

        if let Some(authority) = parts.uri.authority() {
            params = params.custom("HTTP_HOST", authority.as_str().to_owned());
        }

        for (name, value) in &parts.headers {
            let param_name = header_name_to_param_name(name.as_str());
            let header_value =
                value
                    .to_str()
                    .map_err(|_| HttpConversionError::MalformedHttpResponse {
                        message: "HTTP header value is not valid ASCII/UTF-8",
                    })?;
            params = params.custom(param_name, header_value.to_owned());
        }

        Ok(params)
    }
}

#[cfg(feature = "http")]
impl TryFrom<::http::request::Parts> for Params<'static> {
    type Error = HttpConversionError;

    fn try_from(parts: ::http::request::Parts) -> Result<Self, Self::Error> {
        (&parts).try_into()
    }
}

#[cfg(feature = "http")]
fn build_request_uri(params: &Params<'_>) -> HttpConversionResult<::http::Uri> {
    let request_uri = params
        .get("REQUEST_URI")
        .map(|value| value.as_ref())
        .unwrap_or("/");
    let query = params
        .get("QUERY_STRING")
        .map(|value| value.as_ref())
        .unwrap_or("");
    let uri = if query.is_empty() || request_uri.contains('?') {
        request_uri.to_owned()
    } else {
        format!("{request_uri}?{query}")
    };
    Ok(::http::Uri::from_str(&uri)?)
}

#[cfg(feature = "http")]
fn parse_server_protocol(protocol: &str) -> HttpConversionResult<::http::Version> {
    match protocol {
        "HTTP/0.9" => Ok(::http::Version::HTTP_09),
        "HTTP/1.0" => Ok(::http::Version::HTTP_10),
        "HTTP/1.1" => Ok(::http::Version::HTTP_11),
        "HTTP/2" | "HTTP/2.0" => Ok(::http::Version::HTTP_2),
        "HTTP/3" | "HTTP/3.0" => Ok(::http::Version::HTTP_3),
        _ => Err(HttpConversionError::MalformedHttpResponse {
            message: "unsupported SERVER_PROTOCOL value",
        }),
    }
}

#[cfg(feature = "http")]
fn version_to_server_protocol(version: ::http::Version) -> &'static str {
    match version {
        ::http::Version::HTTP_09 => "HTTP/0.9",
        ::http::Version::HTTP_10 => "HTTP/1.0",
        ::http::Version::HTTP_11 => "HTTP/1.1",
        ::http::Version::HTTP_2 => "HTTP/2.0",
        ::http::Version::HTTP_3 => "HTTP/3.0",
        _ => "HTTP/1.1",
    }
}

#[cfg(feature = "http")]
fn required_param<'a>(params: &'a Params<'_>, name: &'static str) -> HttpConversionResult<&'a str> {
    params
        .get(name)
        .map(|value| value.as_ref())
        .ok_or(HttpConversionError::MissingFastcgiParam { name })
}

#[cfg(feature = "http")]
fn params_to_headers(
    params: &Params<'_>,
) -> HttpConversionResult<Vec<(::http::header::HeaderName, ::http::header::HeaderValue)>> {
    let mut headers = Vec::new();

    for (name, value) in params.iter() {
        let header_name = match name.as_ref() {
            "CONTENT_TYPE" => Some(::http::header::CONTENT_TYPE),
            "CONTENT_LENGTH" => Some(::http::header::CONTENT_LENGTH),
            _ => name
                .strip_prefix("HTTP_")
                .map(|header_name| {
                    ::http::header::HeaderName::from_bytes(
                        param_name_to_header_name(header_name).as_bytes(),
                    )
                })
                .transpose()?,
        };

        if let Some(header_name) = header_name {
            headers.push((
                header_name,
                ::http::header::HeaderValue::from_str(value.as_ref())?,
            ));
        }
    }

    Ok(headers)
}

#[cfg(feature = "http")]
fn param_name_to_header_name(name: &str) -> String {
    name.chars()
        .map(|ch| match ch {
            '_' => '-',
            _ => ch.to_ascii_lowercase(),
        })
        .collect()
}

#[cfg(feature = "http")]
fn header_name_to_param_name(name: &str) -> String {
    match name {
        "content-type" => "CONTENT_TYPE".to_owned(),
        "content-length" => "CONTENT_LENGTH".to_owned(),
        _ => format!(
            "HTTP_{}",
            name.chars()
                .map(|ch| match ch {
                    '-' => '_',
                    _ => ch.to_ascii_uppercase(),
                })
                .collect::<String>()
        ),
    }
}

#[cfg(all(test, feature = "http"))]
mod http_tests {
    use super::Params;

    #[test]
    fn params_into_http_parts() {
        let params = Params::default()
            .request_method("POST")
            .request_uri("/index.php")
            .query_string("foo=bar")
            .content_type("application/json")
            .content_length(3)
            .custom("HTTP_HOST", "example.com")
            .custom("HTTP_X_TRACE_ID", "abc");

        let parts: ::http::request::Parts = (&params).try_into().unwrap();

        assert_eq!(parts.method, ::http::Method::POST);
        assert_eq!(parts.uri, "/index.php?foo=bar");
        assert_eq!(parts.headers[::http::header::HOST], "example.com");
        assert_eq!(parts.headers["x-trace-id"], "abc");
        assert_eq!(
            parts.headers[::http::header::CONTENT_TYPE],
            "application/json"
        );
    }

    #[test]
    fn http_parts_into_params() {
        let request = ::http::Request::builder()
            .method(::http::Method::POST)
            .uri("/submit?foo=bar")
            .header(::http::header::HOST, "example.com")
            .header(::http::header::CONTENT_TYPE, "text/plain")
            .body(())
            .unwrap();
        let (parts, _) = request.into_parts();

        let params = Params::try_from(parts).unwrap();

        assert_eq!(params["REQUEST_METHOD"], "POST");
        assert_eq!(params["REQUEST_URI"], "/submit?foo=bar");
        assert_eq!(params["DOCUMENT_URI"], "/submit");
        assert_eq!(params["QUERY_STRING"], "foo=bar");
        assert_eq!(params["HTTP_HOST"], "example.com");
        assert_eq!(params["CONTENT_TYPE"], "text/plain");
    }
}

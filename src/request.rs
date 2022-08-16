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

use crate::Params;
use tokio::io::AsyncRead;

/// fastcgi request.
pub struct Request<'a, I: AsyncRead + Unpin> {
    pub(crate) params: Params<'a>,
    pub(crate) stdin: I,
}

impl<'a, I: AsyncRead + Unpin> Request<'a, I> {
    pub fn new(params: Params<'a>, stdin: I) -> Self {
        Self { params, stdin }
    }

    // pub fn from_http_request(http_request: http::Request<I>) -> Self {
    //     // TODO fill logic
    //     Self {
    //         params: Default::default(),
    //         stdin: http_request.into_body(),
    //     }
    // }

    pub fn params(&self) -> &Params<'a> {
        &self.params
    }

    pub fn params_mut(&mut self) -> &mut Params<'a> {
        &mut self.params
    }

    pub fn stdin(&self) -> &I {
        &self.stdin
    }

    pub fn stdin_mut(&mut self) -> &mut I {
        &mut self.stdin
    }
}

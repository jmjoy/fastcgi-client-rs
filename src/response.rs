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

use std::{fmt, fmt::Debug, str};

/// Output of fastcgi request, contains STDOUT and STDERR.
#[derive(Default, Clone)]
#[non_exhaustive]
pub struct Response {
    pub stdout: Option<Vec<u8>>,
    pub stderr: Option<Vec<u8>>,
}

impl Debug for Response {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        f.debug_struct("Response")
            .field("stdout", &self.stdout.as_deref().map(str::from_utf8))
            .field("stderr", &self.stderr.as_deref().map(str::from_utf8))
            .finish()
    }
}

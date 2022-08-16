/// Connection mode, indicate is keep alive or not.
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

pub trait Mode {
    fn is_keep_alive() -> bool;
}

/// Short connection mode.
pub struct ShortConn;

impl Mode for ShortConn {
    fn is_keep_alive() -> bool {
        false
    }
}

/// Keep alive connection mode.
pub struct KeepAlive {}

impl Mode for KeepAlive {
    fn is_keep_alive() -> bool {
        true
    }
}

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

use std::sync::Once;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

static START: Once = Once::new();

/// Setup function that is only run once, even if called multiple times.
pub fn setup() {
    START.call_once(|| {
        let subscriber = FmtSubscriber::builder()
            .with_max_level(Level::DEBUG)
            .finish();

        tracing::subscriber::set_global_default(subscriber)
            .expect("setting default subscriber failed");
    });
}

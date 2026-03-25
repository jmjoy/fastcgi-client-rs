// Copyright 2026 jmjoy
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use fastcgi_client::{Client, Params, Request, io};
use smol::net::TcpStream;
use std::{error::Error, path::PathBuf};

fn main() -> Result<(), Box<dyn Error>> {
    smol::block_on(async {
        let document_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("php");
        let script_filename = document_root.join("index.php");
        let script_name = "/index.php";

        let stream = TcpStream::connect(("127.0.0.1", 9000)).await?;
        let mut client = Client::new_keep_alive_smol(stream);

        let params = Params::default()
            .request_method("GET")
            .document_root(document_root.to_string_lossy().into_owned())
            .script_name(script_name)
            .script_filename(script_filename.to_string_lossy().into_owned())
            .request_uri(script_name)
            .document_uri(script_name)
            .remote_addr("127.0.0.1")
            .remote_port(12345)
            .server_addr("127.0.0.1")
            .server_port(80)
            .server_name("localhost")
            .content_type("")
            .content_length(0);

        for request_number in 1..=3 {
            let output = client
                .execute(Request::new(params.clone(), io::empty()))
                .await?;

            println!(
                "response #{request_number}:\n{}",
                String::from_utf8_lossy(&output.stdout.unwrap_or_default())
            );

            if let Some(stderr) = output.stderr {
                eprintln!(
                    "stderr #{request_number}:\n{}",
                    String::from_utf8_lossy(&stderr)
                );
            }
        }

        Ok(())
    })
}

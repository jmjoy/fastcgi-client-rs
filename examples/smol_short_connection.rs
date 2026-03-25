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
        let client = Client::new_smol(stream);

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

        let output = client
            .execute_once(Request::new(params, io::empty()))
            .await?;

        println!(
            "stdout:\n{}",
            String::from_utf8_lossy(&output.stdout.unwrap_or_default())
        );

        if let Some(stderr) = output.stderr {
            eprintln!("stderr:\n{}", String::from_utf8_lossy(&stderr));
        }

        Ok(())
    })
}

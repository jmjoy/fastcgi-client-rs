use fastcgi_client::{Client, Params, Request, StreamExt, io, response::Content};
use std::{error::Error, path::PathBuf};
use tokio::net::TcpStream;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    let document_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("php");
    let script_filename = document_root.join("big-response.php");
    let script_name = "/big-response.php";

    let stream = TcpStream::connect(("127.0.0.1", 9000)).await?;
    let client = Client::new_tokio(stream);

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

    let mut stream = client
        .execute_once_stream(Request::new(params, io::empty()))
        .await?;

    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let mut stdout_chunks = 0usize;
    let mut stderr_chunks = 0usize;

    while let Some(content) = stream.next().await {
        match content? {
            Content::Stdout(out) => {
                stdout_chunks += 1;
                println!("stdout chunk #{stdout_chunks}: {} bytes", out.len());
                stdout.extend_from_slice(&out);
            }
            Content::Stderr(err) => {
                stderr_chunks += 1;
                eprintln!("stderr chunk #{stderr_chunks}: {} bytes", err.len());
                stderr.extend_from_slice(&err);
            }
        }
    }

    println!("received {stdout_chunks} stdout chunks and {stderr_chunks} stderr chunks");
    println!("total stdout bytes: {}", stdout.len());
    println!(
        "stdout preview:\n{}",
        String::from_utf8_lossy(&stdout[..stdout.len().min(200)])
    );

    if !stderr.is_empty() {
        eprintln!("stderr:\n{}", String::from_utf8_lossy(&stderr));
    }

    Ok(())
}

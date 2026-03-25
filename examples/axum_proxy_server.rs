use axum::{
    Router,
    body::{Body, to_bytes},
    extract::{Request as AxumRequest, State},
    http::StatusCode,
    response::{IntoResponse, Response as AxumResponse},
    routing::any,
};
use fastcgi_client::{Client, Params, Request, Response, io};
use std::{
    error::Error,
    net::SocketAddr,
    path::{Component, Path, PathBuf},
};
use tokio::net::{TcpListener, TcpStream};

#[derive(Clone)]
struct AppState {
    document_root: PathBuf,
    fastcgi_addr: SocketAddr,
    server_addr: SocketAddr,
}

#[derive(Clone)]
struct ScriptTarget {
    filename: PathBuf,
    script_name: String,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    let document_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("php")
        .canonicalize()?;
    let fastcgi_addr = SocketAddr::from(([127, 0, 0, 1], 9000));
    let server_addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let state = AppState {
        document_root,
        fastcgi_addr,
        server_addr,
    };

    let app = Router::new()
        .route("/", any(proxy_request))
        .route("/{*path}", any(proxy_request))
        .with_state(state);

    let listener = TcpListener::bind(server_addr).await?;
    println!("axum proxy listening on http://{server_addr}");
    println!("forwarding requests to fastcgi://{}", fastcgi_addr);
    axum::serve(listener, app).await?;
    Ok(())
}

async fn proxy_request(
    State(state): State<AppState>, request: AxumRequest,
) -> Result<AxumResponse, AxumResponse> {
    let (parts, body) = request.into_parts();
    let script = resolve_script(&state.document_root, parts.uri.path())?;
    let request_body = to_bytes(body, usize::MAX)
        .await
        .map_err(|error| bad_request(format!("failed to read request body: {error}")))?;

    let content_type = parts
        .headers
        .get(axum::http::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("")
        .to_owned();

    let request = build_fastcgi_request(parts, request_body.to_vec(), &state, &script)
        .map_err(|error| bad_gateway(format!("failed to build fastcgi request: {error}")))?;

    let stream = TcpStream::connect(state.fastcgi_addr)
        .await
        .map_err(|error| bad_gateway(format!("failed to connect to php-fpm: {error}")))?;
    let client = Client::new_tokio(stream);
    let response = client
        .execute_once(request)
        .await
        .map_err(|error| bad_gateway(format!("fastcgi request failed: {error}")))?;

    if let Some(stderr) = response.stderr.as_ref().filter(|stderr| !stderr.is_empty()) {
        eprintln!("fastcgi stderr:\n{}", String::from_utf8_lossy(stderr));
    }

    response_to_axum(response, &content_type)
}

fn build_fastcgi_request(
    parts: axum::http::request::Parts, request_body: Vec<u8>, state: &AppState,
    script: &ScriptTarget,
) -> Result<Request<'static, io::Cursor<Vec<u8>>>, fastcgi_client::HttpConversionError> {
    let content_type = parts
        .headers
        .get(axum::http::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("")
        .to_owned();
    let content_length = request_body.len();
    let http_request = axum::http::Request::from_parts(parts, io::Cursor::new(request_body));
    let extras = Params::default()
        .document_root(state.document_root.to_string_lossy().into_owned())
        .script_filename(script.filename.to_string_lossy().into_owned())
        .script_name(script.script_name.clone())
        .document_uri(script.script_name.clone())
        .remote_addr("127.0.0.1")
        .remote_port(0)
        .server_addr(state.server_addr.ip().to_string())
        .server_port(state.server_addr.port())
        .server_name("localhost")
        .content_type(content_type)
        .content_length(content_length);

    Request::try_from_http_with(http_request, extras)
}

fn resolve_script(document_root: &Path, request_path: &str) -> Result<ScriptTarget, AxumResponse> {
    let relative_path = normalized_script_path(request_path)
        .ok_or_else(|| not_found("requested path is not a valid php script"))?;
    let filename = document_root.join(&relative_path);
    let canonical = filename
        .canonicalize()
        .map_err(|_| not_found("requested script does not exist"))?;

    if !canonical.starts_with(document_root) || !canonical.is_file() {
        return Err(not_found("requested script does not exist"));
    }

    Ok(ScriptTarget {
        filename: canonical,
        script_name: format!("/{relative_path}"),
    })
}

fn normalized_script_path(request_path: &str) -> Option<String> {
    let trimmed = request_path.trim_start_matches('/');
    let candidate = if trimmed.is_empty() {
        "index.php"
    } else {
        trimmed
    };
    let path = Path::new(candidate);
    let mut clean = PathBuf::new();

    for component in path.components() {
        match component {
            Component::Normal(segment) => clean.push(segment),
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => return None,
        }
    }

    if clean.as_os_str().is_empty() || clean.extension()?.to_str()? != "php" {
        return None;
    }

    Some(clean.to_string_lossy().replace('\\', "/"))
}

fn response_to_axum(
    response: Response, fallback_content_type: &str,
) -> Result<AxumResponse, AxumResponse> {
    let http_response: axum::http::Response<Vec<u8>> = response
        .try_into()
        .map_err(|error| bad_gateway(format!("failed to convert fastcgi response: {error}")))?;
    let (mut parts, body) = http_response.into_parts();

    if !fallback_content_type.is_empty()
        && !parts.headers.contains_key(axum::http::header::CONTENT_TYPE)
    {
        let header_value = axum::http::HeaderValue::from_str(fallback_content_type)
            .map_err(|error| internal_error(format!("invalid content-type header: {error}")))?;
        parts
            .headers
            .insert(axum::http::header::CONTENT_TYPE, header_value);
    }

    Ok(axum::http::Response::from_parts(parts, Body::from(body)))
}

fn bad_request(message: String) -> AxumResponse {
    (StatusCode::BAD_REQUEST, message).into_response()
}

fn not_found(message: &str) -> AxumResponse {
    (StatusCode::NOT_FOUND, message.to_owned()).into_response()
}

fn bad_gateway(message: String) -> AxumResponse {
    (StatusCode::BAD_GATEWAY, message).into_response()
}

fn internal_error(message: String) -> AxumResponse {
    (StatusCode::INTERNAL_SERVER_ERROR, message).into_response()
}

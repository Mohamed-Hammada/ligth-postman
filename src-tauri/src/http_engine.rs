//! Rust HTTP engine (README §18). Zero DB dependency — `execution.rs` is the glue that
//! loads a request, resolves its variables, calls `execute` here, then persists the result.
//!
//! Large-response handling (README's "Large Response Handling" requirement): the body is
//! consumed chunk-by-chunk from the stream and buffered only up to `max_inline_bytes`. Past
//! that cap we stop growing the in-memory buffer and spill every further chunk straight to
//! disk — the engine never holds more than `max_inline_bytes` of a response body in RAM,
//! regardless of how large the real response is.

use std::path::PathBuf;
use std::time::{Duration, Instant};

use futures_util::StreamExt;
use reqwest::{Client, Method};
use tokio::io::AsyncWriteExt;

use crate::error::AppError;
use crate::models::HeaderEntry;

pub struct HttpRequestSpec {
    pub method: String,
    pub url: String,
    /// Only `enabled` headers are sent.
    pub headers: Vec<HeaderEntry>,
    pub body: Option<String>,
    pub timeout_ms: u64,
}

pub enum BodyCapture {
    Inline(Vec<u8>),
    Spilled {
        path: PathBuf,
        // Redundant with `HttpResult::body_size` in production use, but kept on the variant
        // itself so tests (and any future disk-body consumer) don't need the outer struct.
        #[allow(dead_code)]
        size: u64,
    },
}

pub struct HttpResult {
    pub status: u16,
    pub status_text: String,
    pub headers: Vec<HeaderEntry>,
    pub duration_ms: u64,
    pub body_size: u64,
    pub body: BodyCapture,
}

/// `spill_dir` is where overflow bodies get written (the caller controls this so files land
/// under the app's data directory, not a system temp dir nobody cleans up).
pub async fn execute(
    client: &Client,
    spec: HttpRequestSpec,
    max_inline_bytes: usize,
    spill_dir: &std::path::Path,
) -> Result<HttpResult, AppError> {
    let method = Method::from_bytes(spec.method.as_bytes())
        .map_err(|_| AppError::Validation(format!("invalid HTTP method '{}'", spec.method)))?;

    let mut builder = client
        .request(method, &spec.url)
        .timeout(Duration::from_millis(spec.timeout_ms));

    for header in &spec.headers {
        if header.enabled {
            builder = builder.header(&header.key, &header.value);
        }
    }
    if let Some(body) = spec.body {
        builder = builder.body(body);
    }

    let started = Instant::now();
    let response = builder.send().await.map_err(|err| AppError::Network(describe(&err)))?;

    let status = response.status();
    let status_text = status.canonical_reason().unwrap_or("").to_string();
    let headers = response
        .headers()
        .iter()
        .map(|(k, v)| HeaderEntry {
            key: k.to_string(),
            value: v.to_str().unwrap_or("").to_string(),
            enabled: true,
        })
        .collect();

    let (body, body_size) = capture_body(response, max_inline_bytes, spill_dir).await?;

    Ok(HttpResult {
        status: status.as_u16(),
        status_text,
        headers,
        duration_ms: started.elapsed().as_millis() as u64,
        body_size,
        body,
    })
}

async fn capture_body(
    response: reqwest::Response,
    max_inline_bytes: usize,
    spill_dir: &std::path::Path,
) -> Result<(BodyCapture, u64), AppError> {
    let mut stream = response.bytes_stream();
    let mut buffer: Vec<u8> = Vec::new();
    let mut spill: Option<(tokio::fs::File, PathBuf)> = None;
    let mut total_size: u64 = 0;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|err| AppError::Network(describe(&err)))?;
        total_size += chunk.len() as u64;

        if let Some((file, _)) = spill.as_mut() {
            file.write_all(&chunk).await.map_err(|err| AppError::Storage(err.to_string()))?;
            continue;
        }

        if buffer.len() + chunk.len() > max_inline_bytes {
            tokio::fs::create_dir_all(spill_dir)
                .await
                .map_err(|err| AppError::Storage(err.to_string()))?;
            let path = spill_dir.join(format!("{}.bin", uuid::Uuid::new_v4()));
            let mut file = tokio::fs::File::create(&path)
                .await
                .map_err(|err| AppError::Storage(err.to_string()))?;
            file.write_all(&buffer).await.map_err(|err| AppError::Storage(err.to_string()))?;
            file.write_all(&chunk).await.map_err(|err| AppError::Storage(err.to_string()))?;
            spill = Some((file, path));
            buffer.clear();
        } else {
            buffer.extend_from_slice(&chunk);
        }
    }

    let capture = match spill {
        Some((_, path)) => BodyCapture::Spilled { path, size: total_size },
        None => BodyCapture::Inline(buffer),
    };
    Ok((capture, total_size))
}

fn describe(err: &reqwest::Error) -> String {
    if err.is_timeout() {
        "request timed out".to_string()
    } else if err.is_connect() {
        format!("connection failed: {err}")
    } else {
        err.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpListener;

    fn spawn_mock_server(response: Vec<u8>) -> u16 {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind mock server");
        let port = listener.local_addr().unwrap().port();
        std::thread::spawn(move || {
            if let Ok((mut stream, _)) = listener.accept() {
                let _ = stream.set_nodelay(true);
                // Drain until the end of headers rather than trusting a single read() to
                // return the whole request in one syscall — under a loaded test binary
                // (many tests hammering real sockets concurrently) a request can arrive in
                // more than one TCP segment.
                let mut received = Vec::new();
                let mut buf = [0u8; 1024];
                loop {
                    match stream.read(&mut buf) {
                        Ok(0) => break,
                        Ok(n) => {
                            received.extend_from_slice(&buf[..n]);
                            if received.windows(4).any(|w| w == b"\r\n\r\n") {
                                break;
                            }
                        }
                        Err(_) => break,
                    }
                }
                let _ = stream.write_all(&response);
                let _ = stream.flush();
            }
        });
        port
    }

    fn spec(url: String) -> HttpRequestSpec {
        HttpRequestSpec { method: "GET".into(), url, headers: vec![], body: None, timeout_ms: 5000 }
    }

    #[tokio::test]
    async fn executes_get_request_and_captures_status_headers_body() {
        let port = spawn_mock_server(
            b"HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: 5\r\nConnection: close\r\n\r\nhello"
                .to_vec(),
        );
        let client = Client::new();
        let tmp = std::env::temp_dir().join("postman-client-test-inline");

        let result = execute(&client, spec(format!("http://127.0.0.1:{port}/")), 1024 * 1024, &tmp)
            .await
            .unwrap();

        assert_eq!(result.status, 200);
        assert_eq!(result.body_size, 5);
        match result.body {
            BodyCapture::Inline(bytes) => assert_eq!(bytes, b"hello"),
            BodyCapture::Spilled { .. } => panic!("expected an inline body"),
        }
    }

    #[tokio::test]
    async fn large_body_spills_to_disk_instead_of_growing_unbounded_in_memory() {
        let payload = vec![b'x'; 4096];
        let mut response = format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            payload.len()
        )
        .into_bytes();
        response.extend_from_slice(&payload);
        let port = spawn_mock_server(response);

        let client = Client::new();
        let tmp = std::env::temp_dir().join("postman-client-test-spill");

        // Cap far smaller than the body forces a spill.
        let result = execute(&client, spec(format!("http://127.0.0.1:{port}/")), 256, &tmp)
            .await
            .unwrap();

        assert_eq!(result.body_size, 4096);
        match result.body {
            BodyCapture::Spilled { path, size } => {
                assert_eq!(size, 4096);
                let contents = std::fs::read(&path).expect("spilled file must exist");
                assert_eq!(contents.len(), 4096);
                let _ = std::fs::remove_file(&path);
            }
            BodyCapture::Inline(_) => panic!("expected a spilled body"),
        }
    }

    #[tokio::test]
    async fn unresolvable_host_is_reported_as_network_error_not_a_panic() {
        let client = Client::new();
        let tmp = std::env::temp_dir().join("postman-client-test-network-error");
        let result = execute(
            &client,
            spec("http://127.0.0.1:1/".into()), // nothing listens on port 1
            1024,
            &tmp,
        )
        .await;
        assert!(matches!(result, Err(AppError::Network(_))));
    }
}

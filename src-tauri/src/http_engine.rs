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
use crate::models::{HeaderEntry, RequestSettings, ResponseCookie};

pub struct HttpRequestSpec {
    pub method: String,
    pub url: String,
    /// Only `enabled` headers are sent.
    pub headers: Vec<HeaderEntry>,
    pub body: Option<String>,
    pub timeout_ms: u64,
    pub settings: Option<RequestSettings>,
}

pub fn build_configured_client(settings: &RequestSettings) -> Result<Client, AppError> {
    let mut builder = reqwest::Client::builder();

    if settings.verify_ssl == Some(false) {
        builder = builder.danger_accept_invalid_certs(true);
    }

    if settings.follow_redirects == Some(false) {
        builder = builder.redirect(reqwest::redirect::Policy::none());
    } else if let Some(max) = settings.max_redirects {
        if max > 0 {
            builder = builder.redirect(reqwest::redirect::Policy::limited(max as usize));
        }
    }

    if let Some(proxy_str) = &settings.proxy_url {
        let trimmed = proxy_str.trim();
        if !trimmed.is_empty() {
            let proxy = reqwest::Proxy::all(trimmed)
                .map_err(|e| AppError::Validation(format!("invalid proxy URL '{trimmed}': {e}")))?;
            builder = builder.proxy(proxy);
        }
    }

    if let Some(ver) = &settings.http_version {
        if ver == "HTTP/1.1" {
            builder = builder.http1_only();
        }
    }

    builder
        .build()
        .map_err(|err| AppError::Network(format!("failed to configure HTTP client: {err}")))
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
    #[allow(dead_code)]
    pub content_type: Option<String>,
    #[allow(dead_code)]
    pub cookies: Vec<ResponseCookie>,
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

    let custom_client = if let Some(settings) = &spec.settings {
        if settings.verify_ssl == Some(false)
            || settings.follow_redirects == Some(false)
            || (settings.max_redirects.is_some() && settings.max_redirects != Some(10))
            || settings.proxy_url.is_some()
            || settings.http_version.is_some()
        {
            Some(build_configured_client(settings)?)
        } else {
            None
        }
    } else {
        None
    };

    let active_client = custom_client.as_ref().unwrap_or(client);

    let mut builder = active_client
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
    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let headers: Vec<HeaderEntry> = response
        .headers()
        .iter()
        .map(|(k, v)| HeaderEntry {
            key: k.to_string(),
            value: v.to_str().unwrap_or("").to_string(),
            enabled: true,
            description: None,
        })
        .collect();

    let mut cookies = Vec::new();
    for val in response.headers().get_all(reqwest::header::SET_COOKIE) {
        if let Ok(cookie_str) = val.to_str() {
            if let Some(cookie) = parse_cookie_header(cookie_str) {
                cookies.push(cookie);
            }
        }
    }

    let (body, body_size) = capture_body(response, max_inline_bytes, spill_dir).await?;

    Ok(HttpResult {
        status: status.as_u16(),
        status_text,
        headers,
        content_type,
        cookies,
        duration_ms: started.elapsed().as_millis() as u64,
        body_size,
        body,
    })
}

pub fn parse_cookie_header(header: &str) -> Option<ResponseCookie> {
    let mut parts = header.split(';').map(|p| p.trim());
    let first = parts.next()?;
    let (name, value) = match first.find('=') {
        Some(idx) => (first[..idx].trim().to_string(), first[idx + 1..].trim().to_string()),
        None => (first.to_string(), String::new()),
    };

    let mut domain = None;
    let mut path = None;
    let mut expires = None;
    let mut http_only = false;
    let mut secure = false;
    let mut same_site = None;

    for part in parts {
        let (k, v) = match part.find('=') {
            Some(idx) => (part[..idx].trim(), part[idx + 1..].trim()),
            None => (part.trim(), ""),
        };
        if k.eq_ignore_ascii_case("domain") {
            domain = Some(v.to_string());
        } else if k.eq_ignore_ascii_case("path") {
            path = Some(v.to_string());
        } else if k.eq_ignore_ascii_case("expires") {
            expires = Some(v.to_string());
        } else if k.eq_ignore_ascii_case("httponly") {
            http_only = true;
        } else if k.eq_ignore_ascii_case("secure") {
            secure = true;
        } else if k.eq_ignore_ascii_case("samesite") {
            same_site = Some(v.to_string());
        }
    }

    Some(ResponseCookie {
        name,
        value,
        domain,
        path,
        expires,
        http_only,
        secure,
        same_site,
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
        Some((mut file, path)) => {
            file.flush().await.map_err(|err| AppError::Storage(err.to_string()))?;
            drop(file);
            BodyCapture::Spilled { path, size: total_size }
        }
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
        HttpRequestSpec { method: "GET".into(), url, headers: vec![], body: None, timeout_ms: 5000, settings: None }
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

    #[test]
    fn build_configured_client_applies_settings_cleanly() {
        let settings = RequestSettings {
            timeout_ms: Some(3000),
            follow_redirects: Some(false),
            max_redirects: Some(3),
            verify_ssl: Some(false),
            proxy_url: None,
            http_version: Some("HTTP/1.1".into()),
        };
        let client = build_configured_client(&settings);
        assert!(client.is_ok(), "configured client should build successfully");

        let bad_proxy = RequestSettings {
            proxy_url: Some("invalid proxy URI %%".into()),
            ..Default::default()
        };
        let err_client = build_configured_client(&bad_proxy);
        assert!(matches!(err_client, Err(AppError::Validation(_))));
    }
}

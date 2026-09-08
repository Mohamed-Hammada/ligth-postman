use base64::Engine;

use crate::error::AppError;
use crate::models::{Auth, FormDataPart, HeaderEntry, QueryParam, RequestBody, RequestSettings};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ParsedCurlRequest {
    pub method: String,
    pub url: String,
    pub headers: Vec<HeaderEntry>,
    pub query_params: Vec<QueryParam>,
    pub auth: Auth,
    pub body: Option<String>,
    #[serde(default)]
    pub settings: RequestSettings,
}

pub fn parse_curl(command: &str) -> Result<ParsedCurlRequest, AppError> {
    let tokens = tokenize_command(command);
    if tokens.is_empty() {
        return Err(AppError::Validation("empty curl command".into()));
    }

    let mut method = None;
    let mut url_candidate = None;
    let mut headers = Vec::new();
    let mut body_parts = Vec::new();
    let mut form_parts = Vec::new();
    let mut auth = Auth::None;
    let mut settings = RequestSettings::default();

    let mut iter = tokens.into_iter();
    if let Some(first) = iter.next() {
        let first_lower = first.to_lowercase();
        if first_lower != "curl" && first_lower != "curl.exe" {
            process_token(
                &first,
                &mut iter,
                &mut method,
                &mut url_candidate,
                &mut headers,
                &mut body_parts,
                &mut form_parts,
                &mut auth,
                &mut settings,
            )?;
        }
    }

    while let Some(token) = iter.next() {
        process_token(
            &token,
            &mut iter,
            &mut method,
            &mut url_candidate,
            &mut headers,
            &mut body_parts,
            &mut form_parts,
            &mut auth,
            &mut settings,
        )?;
    }

    let raw_url = url_candidate.ok_or_else(|| AppError::Validation("no URL found in curl command".into()))?;

    let inferred_method = match method {
        Some(m) => m.to_uppercase(),
        None => {
            if !body_parts.is_empty() || !form_parts.is_empty() {
                "POST".to_string()
            } else {
                "GET".to_string()
            }
        }
    };

    // `-F` takes priority over `-d`/`--data` if a command somehow mixes them (not valid real
    // curl usage, but favoring the multipart interpretation matches what `-F`'s presence signals
    // more strongly than a body-shape guess would). Serialized the same tagged-JSON shape
    // `RequestBody::FormData` already uses everywhere else, so the frontend's existing
    // form-data editor picks it up with no special-casing for curl-imported requests.
    let body = if !form_parts.is_empty() {
        serde_json::to_string(&RequestBody::FormData { items: form_parts })
            .map_err(|e| AppError::Validation(format!("failed to encode imported form-data body: {e}")))
            .map(Some)?
    } else if body_parts.is_empty() {
        None
    } else {
        Some(body_parts.join("&"))
    };

    let (clean_url, query_params) = extract_query_params(&raw_url);

    // If Authorization header exists and no basic auth set, extract Bearer or Basic if possible
    if matches!(auth, Auth::None) {
        if let Some(pos) = headers.iter().position(|h| h.key.eq_ignore_ascii_case("authorization")) {
            let auth_val = headers[pos].value.trim();
            if let Some(token) = auth_val.strip_prefix("Bearer ") {
                auth = Auth::Bearer { token: token.trim().to_string() };
                headers.remove(pos);
            } else if let Some(basic_enc) = auth_val.strip_prefix("Basic ") {
                if let Ok(decoded) = base64::engine::general_purpose::STANDARD.decode(basic_enc.trim()) {
                    if let Ok(userpass) = String::from_utf8(decoded) {
                        if let Some((user, pass)) = userpass.split_once(':') {
                            auth = Auth::Basic { username: user.to_string(), password: pass.to_string() };
                            headers.remove(pos);
                        }
                    }
                }
            }
        }
    }

    Ok(ParsedCurlRequest {
        method: inferred_method,
        url: clean_url,
        headers,
        query_params,
        auth,
        body,
        settings,
    })
}

fn tokenize_command(cmd: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut in_single_quote = false;
    let mut in_double_quote = false;
    let mut chars = cmd.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '\\' | '`' | '^' if chars.peek() == Some(&'\n') => {
                chars.next();
            }
            '\\' | '`' | '^' if chars.peek() == Some(&'\r') => {
                chars.next();
                if chars.peek() == Some(&'\n') {
                    chars.next();
                }
            }
            '\\' if in_double_quote => {
                if let Some(&next_c) = chars.peek() {
                    if next_c == '"' || next_c == '\\' {
                        current.push(next_c);
                        chars.next();
                    } else {
                        current.push('\\');
                    }
                } else {
                    current.push('\\');
                }
            }
            '\'' if !in_double_quote => {
                in_single_quote = !in_single_quote;
            }
            '"' if !in_single_quote => {
                in_double_quote = !in_double_quote;
            }
            c if c.is_whitespace() && !in_single_quote && !in_double_quote => {
                if !current.is_empty() {
                    tokens.push(std::mem::take(&mut current));
                }
            }
            c => {
                current.push(c);
            }
        }
    }

    if !current.is_empty() {
        tokens.push(current);
    }

    tokens
}

#[allow(clippy::too_many_arguments)]
fn process_token(
    token: &str,
    iter: &mut std::vec::IntoIter<String>,
    method: &mut Option<String>,
    url_candidate: &mut Option<String>,
    headers: &mut Vec<HeaderEntry>,
    body_parts: &mut Vec<String>,
    form_parts: &mut Vec<FormDataPart>,
    auth: &mut Auth,
    settings: &mut RequestSettings,
) -> Result<(), AppError> {
    if token == "-X" || token == "--request" {
        if let Some(m) = iter.next() {
            *method = Some(m);
        }
    } else if let Some(m) = token.strip_prefix("--request=") {
        *method = Some(m.to_string());
    } else if token == "-H" || token == "--header" {
        if let Some(h) = iter.next() {
            parse_header_token(&h, headers);
        }
    } else if let Some(h) = token.strip_prefix("--header=") {
        parse_header_token(h, headers);
    } else if token == "-d" || token == "--data" || token == "--data-raw" || token == "--data-ascii" || token == "--data-binary" || token == "--data-urlencode" {
        if let Some(d) = iter.next() {
            body_parts.push(d);
        }
    } else if let Some(d) = token.strip_prefix("--data=").or_else(|| token.strip_prefix("--data-raw=")) {
        body_parts.push(d.to_string());
    } else if token == "-F" || token == "--form" {
        if let Some(f) = iter.next() {
            parse_form_token(&f, form_parts);
        }
    } else if let Some(f) = token.strip_prefix("--form=") {
        parse_form_token(f, form_parts);
    } else if token == "-b" || token == "--cookie" {
        if let Some(c) = iter.next() {
            parse_cookie_token(&c, headers);
        }
    } else if let Some(c) = token.strip_prefix("--cookie=") {
        parse_cookie_token(c, headers);
    } else if token == "-A" || token == "--user-agent" {
        if let Some(a) = iter.next() {
            headers.push(HeaderEntry { key: "User-Agent".to_string(), value: a, enabled: true, description: None });
        }
    } else if let Some(a) = token.strip_prefix("--user-agent=") {
        headers.push(HeaderEntry { key: "User-Agent".to_string(), value: a.to_string(), enabled: true, description: None });
    } else if token == "-k" || token == "--insecure" {
        settings.verify_ssl = Some(false);
    } else if token == "-x" || token == "--proxy" {
        if let Some(p) = iter.next() {
            settings.proxy_url = Some(p);
        }
    } else if let Some(p) = token.strip_prefix("--proxy=") {
        settings.proxy_url = Some(p.to_string());
    } else if token == "-u" || token == "--user" {
        if let Some(u) = iter.next() {
            parse_auth_token(&u, auth);
        }
    } else if let Some(u) = token.strip_prefix("--user=") {
        parse_auth_token(u, auth);
    } else if token == "--url" {
        if let Some(u) = iter.next() {
            *url_candidate = Some(u);
        }
    } else if let Some(u) = token.strip_prefix("--url=") {
        *url_candidate = Some(u.to_string());
    } else if !token.starts_with('-') && url_candidate.is_none() {
        *url_candidate = Some(token.to_string());
    }
    Ok(())
}

fn parse_header_token(h: &str, headers: &mut Vec<HeaderEntry>) {
    if let Some((k, v)) = h.split_once(':') {
        headers.push(HeaderEntry {
            key: k.trim().to_string(),
            value: v.trim().to_string(),
            enabled: true,
            description: None,
        });
    }
}

/// `-F key=value` for a plain text field, `-F key=@/path/to/file` for a file field (curl's
/// optional trailing `;type=...`/`;filename=...` modifiers are dropped — this app's form-data
/// model has no field for an explicit part content-type).
fn parse_form_token(f: &str, form_parts: &mut Vec<FormDataPart>) {
    let Some((key, raw_value)) = f.split_once('=') else {
        return;
    };
    if let Some(file_ref) = raw_value.strip_prefix('@') {
        let file_path = file_ref.split(';').next().unwrap_or(file_ref);
        form_parts.push(FormDataPart {
            key: key.trim().to_string(),
            value: String::new(),
            enabled: true,
            description: None,
            is_file: true,
            file_path: Some(file_path.trim().to_string()),
        });
    } else {
        form_parts.push(FormDataPart {
            key: key.trim().to_string(),
            value: raw_value.trim().to_string(),
            enabled: true,
            description: None,
            is_file: false,
            file_path: None,
        });
    }
}

/// `-b "name1=value1; name2=value2"` becomes a literal `Cookie` header — the same thing curl
/// itself sends. `-b @file` (read cookies from a Netscape cookie-jar file) can't be resolved
/// while just parsing a command string, so that form is silently skipped rather than emitting a
/// broken header with the literal "@file" text as its value.
fn parse_cookie_token(c: &str, headers: &mut Vec<HeaderEntry>) {
    let trimmed = c.trim();
    if trimmed.starts_with('@') {
        return;
    }
    headers.push(HeaderEntry {
        key: "Cookie".to_string(),
        value: trimmed.to_string(),
        enabled: true,
        description: None,
    });
}

fn parse_auth_token(u: &str, auth: &mut Auth) {
    if let Some((user, pass)) = u.split_once(':') {
        *auth = Auth::Basic {
            username: user.to_string(),
            password: pass.to_string(),
        };
    } else {
        *auth = Auth::Basic {
            username: u.to_string(),
            password: String::new(),
        };
    }
}

fn extract_query_params(raw_url: &str) -> (String, Vec<QueryParam>) {
    if let Ok(url) = reqwest::Url::parse(raw_url) {
        let mut query_params = Vec::new();
        for (k, v) in url.query_pairs() {
            query_params.push(QueryParam {
                key: k.into_owned(),
                value: v.into_owned(),
                enabled: true,
                description: None,
            });
        }
        let mut clean = url.clone();
        clean.set_query(None);
        (clean.to_string(), query_params)
    } else if let Some((base, query)) = raw_url.split_once('?') {
        let mut params = Vec::new();
        for part in query.split('&') {
            if part.is_empty() {
                continue;
            }
            let (k, v) = match part.split_once('=') {
                Some((k, v)) => (k.to_string(), v.to_string()),
                None => (part.to_string(), String::new()),
            };
            params.push(QueryParam {
                key: k,
                value: v,
                enabled: true,
                description: None,
            });
        }
        (base.to_string(), params)
    } else {
        (raw_url.to_string(), Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_simple_get() {
        let cmd = "curl https://api.example.com/items";
        let parsed = parse_curl(cmd).unwrap();
        assert_eq!(parsed.method, "GET");
        assert_eq!(parsed.url, "https://api.example.com/items");
        assert!(parsed.headers.is_empty());
        assert!(parsed.body.is_none());
    }

    #[test]
    fn parses_post_with_headers_and_body() {
        let cmd = r#"curl -X POST 'https://api.example.com/users' \
  -H 'Content-Type: application/json' \
  -H 'Authorization: Bearer secret123' \
  --data-raw '{"name":"Alice"}'"#;

        let parsed = parse_curl(cmd).unwrap();
        assert_eq!(parsed.method, "POST");
        assert_eq!(parsed.url, "https://api.example.com/users");
        assert_eq!(parsed.headers.len(), 1);
        assert_eq!(parsed.headers[0].key, "Content-Type");
        assert_eq!(parsed.headers[0].value, "application/json");
        assert_eq!(parsed.auth, Auth::Bearer { token: "secret123".into() });
        assert_eq!(parsed.body.as_deref(), Some(r#"{"name":"Alice"}"#));
    }

    #[test]
    fn parses_query_parameters_into_separate_list() {
        let cmd = "curl 'https://api.example.com/search?q=rust&limit=10'";
        let parsed = parse_curl(cmd).unwrap();
        assert_eq!(parsed.url, "https://api.example.com/search");
        assert_eq!(parsed.query_params.len(), 2);
        assert_eq!(parsed.query_params[0].key, "q");
        assert_eq!(parsed.query_params[0].value, "rust");
        assert_eq!(parsed.query_params[1].key, "limit");
        assert_eq!(parsed.query_params[1].value, "10");
    }

    #[test]
    fn parses_powershell_and_cmd_curl_syntax() {
        let ps_cmd = "curl.exe -X PUT 'https://example.com/item' `\n  -H 'X-Test: true' `\n  -d 'hello'";
        let parsed = parse_curl(ps_cmd).unwrap();
        assert_eq!(parsed.method, "PUT");
        assert_eq!(parsed.headers[0].key, "X-Test");
        assert_eq!(parsed.body.as_deref(), Some("hello"));

        let cmd_cmd = "curl.exe -X DELETE ^\n  \"https://example.com/item/1\"";
        let parsed_cmd = parse_curl(cmd_cmd).unwrap();
        assert_eq!(parsed_cmd.method, "DELETE");
        assert_eq!(parsed_cmd.url, "https://example.com/item/1");
    }

    #[test]
    fn parses_basic_auth() {
        let cmd = "curl -u myuser:mypass https://example.com";
        let parsed = parse_curl(cmd).unwrap();
        assert_eq!(parsed.auth, Auth::Basic { username: "myuser".into(), password: "mypass".into() });
    }

    #[test]
    fn parses_multipart_form_fields_and_files() {
        let cmd = r#"curl -F 'name=Alice' -F 'avatar=@/tmp/pic.png;type=image/png' https://api.example.com/upload"#;
        let parsed = parse_curl(cmd).unwrap();
        assert_eq!(parsed.method, "POST", "presence of -F should infer POST just like -d does");

        let body: RequestBody = serde_json::from_str(parsed.body.as_deref().unwrap()).unwrap();
        match body {
            RequestBody::FormData { items } => {
                assert_eq!(items.len(), 2);
                assert_eq!(items[0].key, "name");
                assert_eq!(items[0].value, "Alice");
                assert!(!items[0].is_file);
                assert_eq!(items[1].key, "avatar");
                assert!(items[1].is_file);
                // The ";type=image/png" modifier must not leak into the stored file path.
                assert_eq!(items[1].file_path.as_deref(), Some("/tmp/pic.png"));
            }
            other => panic!("expected RequestBody::FormData, got {other:?}"),
        }
    }

    #[test]
    fn parses_cookie_user_agent_insecure_and_proxy_flags() {
        let cmd = r#"curl -b 'session=abc123; theme=dark' -A 'MyClient/1.0' -k -x 'http://proxy.local:8080' https://api.example.com"#;
        let parsed = parse_curl(cmd).unwrap();

        let cookie = parsed.headers.iter().find(|h| h.key == "Cookie").expect("Cookie header");
        assert_eq!(cookie.value, "session=abc123; theme=dark");

        let ua = parsed.headers.iter().find(|h| h.key == "User-Agent").expect("User-Agent header");
        assert_eq!(ua.value, "MyClient/1.0");

        assert_eq!(parsed.settings.verify_ssl, Some(false));
        assert_eq!(parsed.settings.proxy_url.as_deref(), Some("http://proxy.local:8080"));
    }

    #[test]
    fn cookie_jar_file_reference_is_skipped_rather_than_sent_as_a_literal_header() {
        let cmd = "curl -b @cookies.txt https://api.example.com";
        let parsed = parse_curl(cmd).unwrap();
        assert!(parsed.headers.iter().all(|h| h.key != "Cookie"));
    }
}

use base64::Engine;

use crate::error::AppError;
use crate::models::{Auth, HeaderEntry, QueryParam};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ParsedCurlRequest {
    pub method: String,
    pub url: String,
    pub headers: Vec<HeaderEntry>,
    pub query_params: Vec<QueryParam>,
    pub auth: Auth,
    pub body: Option<String>,
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
    let mut auth = Auth::None;

    let mut iter = tokens.into_iter();
    if let Some(first) = iter.next() {
        let first_lower = first.to_lowercase();
        if first_lower != "curl" && first_lower != "curl.exe" {
            process_token(&first, &mut iter, &mut method, &mut url_candidate, &mut headers, &mut body_parts, &mut auth)?;
        }
    }

    while let Some(token) = iter.next() {
        process_token(&token, &mut iter, &mut method, &mut url_candidate, &mut headers, &mut body_parts, &mut auth)?;
    }

    let raw_url = url_candidate.ok_or_else(|| AppError::Validation("no URL found in curl command".into()))?;

    let inferred_method = match method {
        Some(m) => m.to_uppercase(),
        None => {
            if !body_parts.is_empty() {
                "POST".to_string()
            } else {
                "GET".to_string()
            }
        }
    };

    let body = if body_parts.is_empty() {
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

fn process_token(
    token: &str,
    iter: &mut std::vec::IntoIter<String>,
    method: &mut Option<String>,
    url_candidate: &mut Option<String>,
    headers: &mut Vec<HeaderEntry>,
    body_parts: &mut Vec<String>,
    auth: &mut Auth,
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
}

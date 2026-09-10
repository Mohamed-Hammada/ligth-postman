//! Code snippet generation (task-pack LP-0601/0602/0605/0606).
//!
//! Every generator consumes a `CanonicalRequest` — never the UI, never raw request text —
//! so a snippet can never show something different from what actually gets sent (LP-0601's
//! "generate from CanonicalRequest, never from UI text").
//!
//! Only one target exists today (Bash `curl`); PowerShell/CMD/other languages (LP-0603,
//! 0604, 0607) are not implemented — add them as siblings to `generate_curl_bash` and a new
//! `SnippetTarget` variant, not by touching `SnippetMode`/`generate_snippet`.

use crate::canonical_request;
use crate::error::AppError;
use crate::models::RequestFull;
use crate::resolver::ScopeChain;

/// LP-0606 secret-safe modes. `Masked` (show real values except secrets) is NOT implemented —
/// it would need the resolver to track which output bytes came from a secret variable, which
/// `resolver::resolve_template` doesn't do today. Rather than fake it, only the two modes that
/// are fully honest are offered: never resolve anything, or resolve everything explicitly.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SnippetMode {
    /// Default. Variables stay as literal `{{name}}` — a secret can never leak into a
    /// snippet unless the user explicitly switches to `Resolved`.
    Placeholder,
    /// Explicit opt-in: every `{{var}}` (including secrets) is resolved to its real value,
    /// exactly as it would be sent over the wire.
    Resolved,
}

/// Target shell or tool syntax for the generated snippet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SnippetTarget {
    Bash,
    PowerShell,
    WindowsCmd,
    PythonRequests,
    JavaScriptFetch,
}

impl Default for SnippetTarget {
    fn default() -> Self {
        SnippetTarget::Bash
    }
}

pub fn generate_snippet(
    request: &RequestFull,
    chain: &ScopeChain,
    mode: SnippetMode,
    target: SnippetTarget,
) -> Result<String, AppError> {
    let canonical = match mode {
        SnippetMode::Resolved => canonical_request::build(request, chain)?,
        // An empty chain resolves nothing — every {{var}} is left exactly as stored. This
        // reuses the real resolver/canonical-request path instead of a separate "don't
        // resolve" code path, so placeholder mode can't silently drift from resolved mode.
        SnippetMode::Placeholder => canonical_request::build(request, &ScopeChain::default())?,
    };
    match target {
        SnippetTarget::Bash => Ok(generate_curl_bash(&canonical)),
        SnippetTarget::PowerShell => Ok(generate_curl_powershell(&canonical)),
        SnippetTarget::WindowsCmd => Ok(generate_curl_cmd(&canonical)),
        SnippetTarget::PythonRequests => Ok(generate_python(&canonical)),
        SnippetTarget::JavaScriptFetch => Ok(generate_javascript(&canonical)),
    }
}

/// `-F key=value` for text parts, `-F key=@path;filename=name` for files — curl generates its
/// own multipart boundary from these flags, so the snippet never has to (and never could)
/// spell one out itself.
fn multipart_curl_flags(parts: &[crate::models::ResolvedFormPart], quote: impl Fn(&str) -> String) -> Vec<String> {
    parts
        .iter()
        .map(|p| match p {
            crate::models::ResolvedFormPart::Text { key, value } => {
                format!("-F {}", quote(&format!("{key}={value}")))
            }
            crate::models::ResolvedFormPart::File { key, file_name, file_path } => {
                format!("-F {}", quote(&format!("{key}=@{file_path};filename={file_name}")))
            }
        })
        .collect()
}

pub fn generate_curl_bash(canonical: &canonical_request::CanonicalRequest) -> String {
    let mut parts = vec![format!("curl -X {}", canonical.method), shell_single_quote(&canonical.url)];
    for header in &canonical.headers {
        parts.push(format!(
            "-H {}",
            shell_single_quote(&format!("{}: {}", header.key, header.value))
        ));
    }
    if let Some(multipart) = &canonical.multipart {
        parts.extend(multipart_curl_flags(multipart, shell_single_quote));
    } else if let Some(path) = &canonical.body_file_path {
        parts.push(format!("--data-binary @{}", shell_single_quote(path)));
    } else if let Some(body) = &canonical.body {
        parts.push(format!("--data-raw {}", shell_single_quote(body)));
    }
    parts.join(" \\\n  ")
}

pub fn generate_curl_powershell(canonical: &canonical_request::CanonicalRequest) -> String {
    let mut parts = vec![
        format!("curl.exe -X {}", canonical.method),
        powershell_single_quote(&canonical.url),
    ];
    for header in &canonical.headers {
        parts.push(format!(
            "-H {}",
            powershell_single_quote(&format!("{}: {}", header.key, header.value))
        ));
    }
    if let Some(multipart) = &canonical.multipart {
        parts.extend(multipart_curl_flags(multipart, powershell_single_quote));
    } else if let Some(path) = &canonical.body_file_path {
        parts.push(format!("--data-binary @{}", powershell_single_quote(path)));
    } else if let Some(body) = &canonical.body {
        parts.push(format!("--data-raw {}", powershell_single_quote(body)));
    }
    parts.join(" `\n  ")
}

pub fn generate_curl_cmd(canonical: &canonical_request::CanonicalRequest) -> String {
    let mut parts = vec![
        format!("curl.exe -X {}", canonical.method),
        cmd_double_quote(&canonical.url),
    ];
    for header in &canonical.headers {
        parts.push(format!(
            "-H {}",
            cmd_double_quote(&format!("{}: {}", header.key, header.value))
        ));
    }
    if let Some(multipart) = &canonical.multipart {
        parts.extend(multipart_curl_flags(multipart, cmd_double_quote));
    } else if let Some(path) = &canonical.body_file_path {
        parts.push(format!("--data-binary @{}", cmd_double_quote(path)));
    } else if let Some(body) = &canonical.body {
        parts.push(format!("--data-raw {}", cmd_double_quote(body)));
    }
    parts.join(" ^\n  ")
}

pub fn generate_python(canonical: &canonical_request::CanonicalRequest) -> String {
    let mut lines = vec![
        "import requests".to_string(),
        "".to_string(),
        format!("url = \"{}\"", canonical.url),
    ];
    if !canonical.headers.is_empty() {
        lines.push("headers = {".to_string());
        for h in &canonical.headers {
            lines.push(format!("    \"{}\": \"{}\",", h.key, h.value.replace('\\', "\\\\").replace('"', "\\\"")));
        }
        lines.push("}".to_string());
    } else {
        lines.push("headers = {}".to_string());
    }

    if let Some(multipart) = &canonical.multipart {
        let mut has_files = false;
        lines.push("files = {".to_string());
        for part in multipart {
            match part {
                crate::models::ResolvedFormPart::Text { key, value } => {
                    lines.push(format!("    \"{}\": (None, \"{}\"),", key, value.replace('\\', "\\\\").replace('"', "\\\"")));
                }
                crate::models::ResolvedFormPart::File { key, file_name, file_path } => {
                    has_files = true;
                    lines.push(format!(
                        "    \"{}\": (\"{}\", open(\"{}\", \"rb\")),",
                        key, file_name, file_path.replace('\\', "\\\\")
                    ));
                }
            }
        }
        lines.push("}".to_string());
        if !has_files {
            lines.push("# note: no file fields — 'files=' still triggers multipart/form-data encoding".to_string());
        }
        lines.push(format!(
            "response = requests.request(\"{}\", url, headers=headers, files=files)",
            canonical.method
        ));
    } else if let Some(path) = &canonical.body_file_path {
        lines.push(format!("data = open(\"{}\", \"rb\")", path.replace('\\', "\\\\")));
        lines.push(format!("response = requests.request(\"{}\", url, headers=headers, data=data)", canonical.method));
    } else if let Some(body) = &canonical.body {
        lines.push(format!("data = \"\"\"{}\"\"\"", body));
        lines.push(format!("response = requests.request(\"{}\", url, headers=headers, data=data)", canonical.method));
    } else {
        lines.push(format!("response = requests.request(\"{}\", url, headers=headers)", canonical.method));
    }
    lines.push("print(response.status_code)".to_string());
    lines.push("print(response.text)".to_string());
    lines.join("\n")
}

pub fn generate_javascript(canonical: &canonical_request::CanonicalRequest) -> String {
    let mut lines = vec![format!("const url = \"{}\";", canonical.url)];

    if let Some(multipart) = &canonical.multipart {
        // FormData sets its own multipart/form-data + boundary Content-Type when passed as
        // fetch's body — same reasoning as the Rust engine, just via the browser/Node API
        // instead of reqwest. File fields need an actual File/Blob, which a code snippet can't
        // synthesize from a path string, so those are left as a clear TODO rather than faked.
        lines.push("const form = new FormData();".to_string());
        for part in multipart {
            match part {
                crate::models::ResolvedFormPart::Text { key, value } => {
                    lines.push(format!(
                        "form.append(\"{}\", \"{}\");",
                        key,
                        value.replace('\\', "\\\\").replace('"', "\\\"")
                    ));
                }
                crate::models::ResolvedFormPart::File { key, file_name, .. } => {
                    lines.push(format!(
                        "form.append(\"{key}\", /* TODO: File/Blob for \"{file_name}\" */ undefined, \"{file_name}\");"
                    ));
                }
            }
        }
        lines.push("const options = {".to_string());
        lines.push(format!("  method: \"{}\",", canonical.method));
        if !canonical.headers.is_empty() {
            lines.push("  headers: {".to_string());
            for h in &canonical.headers {
                lines.push(format!("    \"{}\": \"{}\",", h.key, h.value.replace('\\', "\\\\").replace('"', "\\\"")));
            }
            lines.push("  },".to_string());
        }
        lines.push("  body: form,".to_string());
        lines.push("};".to_string());
    } else {
        lines.push("const options = {".to_string());
        lines.push(format!("  method: \"{}\",", canonical.method));
        if !canonical.headers.is_empty() {
            lines.push("  headers: {".to_string());
            for h in &canonical.headers {
                lines.push(format!("    \"{}\": \"{}\",", h.key, h.value.replace('\\', "\\\\").replace('"', "\\\"")));
            }
            lines.push("  },".to_string());
        }
        if let Some(path) = &canonical.body_file_path {
            lines.push(format!("  body: /* TODO: read file bytes for \"{path}\" (e.g. fs.readFileSync in Node) */ undefined,"));
        } else if let Some(body) = &canonical.body {
            lines.push(format!("  body: JSON.stringify({}),", body));
        }
        lines.push("};".to_string());
    }

    lines.push("".to_string());
    lines.push("const response = await fetch(url, options);".to_string());
    lines.push("const result = await response.text();".to_string());
    lines.push("console.log(result);".to_string());
    lines.join("\n")
}

/// POSIX single-quoting: wrap in `'...'`, and turn any literal `'` into `'\''` (close quote,
/// escaped literal quote, reopen quote) — the standard, minimal-surprise way to make an
/// arbitrary string safe inside single quotes in `sh`/`bash`.
fn shell_single_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

/// PowerShell single-quoting: wrap in `'...'`, and turn any literal `'` into `''` (doubled single quote).
fn powershell_single_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

/// Windows CMD double-quoting: wrap in `"..."`, escape double quotes with `\"`.
fn cmd_double_quote(value: &str) -> String {
    format!("\"{}\"", value.replace('"', "\\\""))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Auth, HeaderEntry, QueryParam};
    use std::collections::HashMap;

    fn request() -> RequestFull {
        RequestFull {
            id: "r1".into(),
            project_id: "p1".into(),
            folder_id: None,
            name: "Get user".into(),
            method: "GET".into(),
            url: "https://api.example.com/users/{{userId}}".into(),
            headers: vec![HeaderEntry { key: "Accept".into(), value: "application/json".into(), enabled: true, description: None }],
            query_params: vec![],
            auth: Auth::Bearer { token: "{{token}}".into() },
            body: None,
            description: None,
            settings: Default::default(),
            pre_request_script: None,
            post_request_script: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }

    #[test]
    fn shell_single_quote_escapes_embedded_quotes() {
        assert_eq!(shell_single_quote("it's fine"), r#"'it'\''s fine'"#);
        assert_eq!(shell_single_quote("no quotes"), "'no quotes'");
    }

    #[test]
    fn powershell_single_quote_escapes_embedded_quotes() {
        assert_eq!(powershell_single_quote("it's fine"), "'it''s fine'");
        assert_eq!(powershell_single_quote("no quotes"), "'no quotes'");
    }

    #[test]
    fn cmd_double_quote_escapes_embedded_quotes() {
        assert_eq!(cmd_double_quote(r#"{"key":"val"}"#), r#""{\"key\":\"val\"}""#);
    }

    #[test]
    fn placeholder_mode_never_resolves_variables_including_secrets() {
        let snippet = generate_snippet(&request(), &ScopeChain::default(), SnippetMode::Placeholder, SnippetTarget::Bash).unwrap();
        assert!(snippet.contains("{{userId}}"));
        assert!(snippet.contains("Bearer {{token}}"));
        assert!(!snippet.contains("Bearer sk-"));
    }

    #[test]
    fn resolved_mode_substitutes_every_variable() {
        let vars: HashMap<String, String> =
            [("userId".to_string(), "42".to_string()), ("token".to_string(), "sk-live-123".to_string())].into();
        let chain = ScopeChain { global: Some(&vars), ..Default::default() };
        let snippet = generate_snippet(&request(), &chain, SnippetMode::Resolved, SnippetTarget::Bash).unwrap();
        assert!(snippet.contains("https://api.example.com/users/42"));
        assert!(snippet.contains("Bearer sk-live-123"));
        assert!(!snippet.contains("{{"));
    }

    #[test]
    fn generated_snippet_has_expected_curl_structure() {
        let vars: HashMap<String, String> =
            [("userId".to_string(), "42".to_string()), ("token".to_string(), "abc".to_string())].into();
        let chain = ScopeChain { global: Some(&vars), ..Default::default() };
        let snippet = generate_snippet(&request(), &chain, SnippetMode::Resolved, SnippetTarget::Bash).unwrap();
        assert_eq!(
            snippet,
            "curl -X GET \\\n  'https://api.example.com/users/42' \\\n  -H 'Accept: application/json' \\\n  -H 'Authorization: Bearer abc'"
        );
    }

    #[test]
    fn powershell_generator_uses_backticks_and_escapes() {
        let vars: HashMap<String, String> =
            [("userId".to_string(), "42".to_string()), ("token".to_string(), "abc".to_string())].into();
        let chain = ScopeChain { global: Some(&vars), ..Default::default() };
        let mut req = request();
        req.method = "POST".into();
        req.body = Some(r#"{"note":"Ada's test"}"#.into());

        let snippet = generate_snippet(&req, &chain, SnippetMode::Resolved, SnippetTarget::PowerShell).unwrap();
        assert!(snippet.starts_with("curl.exe -X POST"));
        assert!(snippet.contains(" `\n  "));
        assert!(snippet.contains("--data-raw '{\"note\":\"Ada''s test\"}'"));
    }

    #[test]
    fn cmd_generator_uses_carets_and_double_quotes() {
        let vars: HashMap<String, String> =
            [("userId".to_string(), "42".to_string()), ("token".to_string(), "abc".to_string())].into();
        let chain = ScopeChain { global: Some(&vars), ..Default::default() };
        let mut req = request();
        req.method = "POST".into();
        req.body = Some(r#"{"name":"Ada"}"#.into());

        let snippet = generate_snippet(&req, &chain, SnippetMode::Resolved, SnippetTarget::WindowsCmd).unwrap();
        assert!(snippet.starts_with("curl.exe -X POST"));
        assert!(snippet.contains(" ^\n  "));
        assert!(snippet.contains(r#"--data-raw "{\"name\":\"Ada\"}""#));
    }

    #[test]
    fn body_is_included_with_data_raw_and_query_params_are_appended() {
        let mut req = request();
        req.method = "POST".into();
        req.body = Some(r#"{"name":"Ada"}"#.into());
        req.query_params.push(QueryParam { key: "verbose".into(), value: "true".into(), enabled: true, description: None });

        let snippet = generate_snippet(&req, &ScopeChain::default(), SnippetMode::Placeholder, SnippetTarget::Bash).unwrap();
        assert!(snippet.contains("curl -X POST"));
        assert!(snippet.contains("verbose=true"));
        assert!(snippet.ends_with(r#"--data-raw '{"name":"Ada"}'"#));
    }

    #[test]
    fn multipart_form_data_generates_dash_f_flags_not_a_fake_data_raw_body() {
        let mut req = request();
        req.method = "POST".into();
        req.body = Some(
            r#"{"type":"form_data","items":[
                {"key":"name","value":"ada","enabled":true,"is_file":false},
                {"key":"avatar","value":"","enabled":true,"is_file":true,"file_path":"/tmp/pic.png"}
            ]}"#
                .into(),
        );

        let bash = generate_snippet(&req, &ScopeChain::default(), SnippetMode::Placeholder, SnippetTarget::Bash).unwrap();
        assert!(bash.contains("-F 'name=ada'"));
        assert!(bash.contains("-F 'avatar=@/tmp/pic.png;filename=pic.png'"));
        assert!(!bash.contains("--data-raw"), "must not fall back to a flattened data-raw body");

        let python =
            generate_snippet(&req, &ScopeChain::default(), SnippetMode::Placeholder, SnippetTarget::PythonRequests).unwrap();
        assert!(python.contains("files = {"));
        assert!(python.contains("open(\"/tmp/pic.png\", \"rb\")"));

        let js =
            generate_snippet(&req, &ScopeChain::default(), SnippetMode::Placeholder, SnippetTarget::JavaScriptFetch).unwrap();
        assert!(js.contains("new FormData()"));
        assert!(js.contains(r#"form.append("name", "ada")"#));
        assert!(js.contains("body: form"));
    }

    #[test]
    fn python_generator_produces_valid_structure() {
        let vars: HashMap<String, String> =
            [("userId".to_string(), "42".to_string()), ("token".to_string(), "abc".to_string())].into();
        let chain = ScopeChain { global: Some(&vars), ..Default::default() };
        let mut req = request();
        req.method = "POST".into();
        req.body = Some(r#"{"test": true}"#.into());

        let snippet = generate_snippet(&req, &chain, SnippetMode::Resolved, SnippetTarget::PythonRequests).unwrap();
        assert!(snippet.contains("import requests"));
        assert!(snippet.contains("url = \"https://api.example.com/users/42\""));
        assert!(snippet.contains("\"Authorization\": \"Bearer abc\""));
        assert!(snippet.contains("requests.request(\"POST\", url"));
    }

    #[test]
    fn javascript_generator_produces_fetch_syntax() {
        let vars: HashMap<String, String> =
            [("userId".to_string(), "42".to_string()), ("token".to_string(), "abc".to_string())].into();
        let chain = ScopeChain { global: Some(&vars), ..Default::default() };
        let mut req = request();
        req.method = "GET".into();

        let snippet = generate_snippet(&req, &chain, SnippetMode::Resolved, SnippetTarget::JavaScriptFetch).unwrap();
        assert!(snippet.contains("const url = \"https://api.example.com/users/42\";"));
        assert!(snippet.contains("method: \"GET\""));
        assert!(snippet.contains("await fetch(url, options)"));
    }
}

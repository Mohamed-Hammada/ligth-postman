//! Source Project Analyzer & Framework Route Discovery (LP-0810 - LP-0821).
//!
//! Provides read-only scanning of local backend source repositories to detect
//! frameworks (Spring, Express, NestJS, FastAPI, Flask, Django, Go, Laravel, ASP.NET)
//! and discover API routes, schemas, and OpenAPI specifications.

use std::fs;
use std::path::{Path, PathBuf};

use regex::Regex;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

use crate::error::AppError;

/// Discovered API route from source code or OpenAPI specs (LP-0814, LP-0815, LP-0821).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredEndpoint {
    pub name: String,
    pub method: String,
    pub path: String,
    pub description: Option<String>,
    pub source_file: String,
    pub line_number: Option<usize>,
    pub auth_hint: Option<String>,
    pub framework: String,
}

/// Analysis report of a local backend codebase (LP-0812, LP-0816, LP-0820).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceProjectReport {
    pub directory: String,
    pub project_type: String,
    pub frameworks: Vec<String>,
    pub has_openapi: bool,
    pub openapi_path: Option<String>,
    pub endpoints: Vec<DiscoveredEndpoint>,
    pub scanned_files_count: usize,
    pub warnings: Vec<String>,
}

const MAX_SEARCH_DEPTH: usize = 4;
const MAX_SCANNED_FILES: usize = 120;
const MAX_FILE_SIZE_BYTES: u64 = 65_536; // 64 KB

const IGNORED_DIRS: &[&str] = &[
    "node_modules",
    ".git",
    ".svn",
    ".hg",
    "target",
    "vendor",
    "bin",
    "obj",
    "dist",
    "build",
    "out",
    ".next",
    ".nuxt",
    "__pycache__",
    ".venv",
    "env",
    "venv",
    ".idea",
    ".vscode",
];

/// Scans a local source project in strict read-only mode (LP-0811, LP-0820).
pub fn scan_source_project(directory_path: &str) -> Result<SourceProjectReport, AppError> {
    let dir = Path::new(directory_path);
    if !dir.exists() || !dir.is_dir() {
        return Err(AppError::Validation(format!(
            "Directory does not exist or is not a directory: {directory_path}"
        )));
    }

    let mut scanned_files_count = 0;
    let mut warnings = Vec::new();
    let mut frameworks = Vec::new();
    let mut project_type = "Unknown".to_string();
    let mut has_openapi = false;
    let mut openapi_path = None;
    let mut endpoints = Vec::new();

    // 1. Detect project type from root manifests (LP-0812)
    if dir.join("package.json").is_file() {
        project_type = "Node.js / TypeScript".to_string();
        if let Ok(content) = fs::read_to_string(dir.join("package.json")) {
            if content.contains("express") {
                frameworks.push("Express".to_string());
            }
            if content.contains("@nestjs") {
                frameworks.push("NestJS".to_string());
            }
            if content.contains("fastify") {
                frameworks.push("Fastify".to_string());
            }
            if content.contains("koa") {
                frameworks.push("Koa".to_string());
            }
        }
    } else if dir.join("pom.xml").is_file() || dir.join("build.gradle").is_file() || dir.join("build.gradle.kts").is_file() {
        project_type = "Java / Kotlin".to_string();
        frameworks.push("Spring Boot".to_string());
    } else if dir.join("requirements.txt").is_file() || dir.join("pyproject.toml").is_file() || dir.join("Pipfile").is_file() {
        project_type = "Python".to_string();
        let py_text = fs::read_to_string(dir.join("requirements.txt"))
            .unwrap_or_else(|_| fs::read_to_string(dir.join("pyproject.toml")).unwrap_or_default());
        if py_text.contains("fastapi") {
            frameworks.push("FastAPI".to_string());
        }
        if py_text.contains("flask") {
            frameworks.push("Flask".to_string());
        }
        if py_text.contains("django") {
            frameworks.push("Django".to_string());
        }
    } else if dir.join("go.mod").is_file() {
        project_type = "Go".to_string();
        if let Ok(go_mod) = fs::read_to_string(dir.join("go.mod")) {
            if go_mod.contains("gin-gonic") {
                frameworks.push("Gin".to_string());
            }
            if go_mod.contains("go-chi") {
                frameworks.push("Chi".to_string());
            }
            if go_mod.contains("labstack/echo") {
                frameworks.push("Echo".to_string());
            }
        }
    } else if dir.join("composer.json").is_file() {
        project_type = "PHP".to_string();
        if let Ok(comp) = fs::read_to_string(dir.join("composer.json")) {
            if comp.contains("laravel/framework") {
                frameworks.push("Laravel".to_string());
            }
            if comp.contains("symfony") {
                frameworks.push("Symfony".to_string());
            }
        }
    }

    // 2. Check for OpenAPI / Swagger specifications first (LP-0813)
    let candidate_openapi_files = [
        "openapi.json",
        "openapi.yaml",
        "openapi.yml",
        "swagger.json",
        "swagger.yaml",
        "swagger.yml",
        "docs/openapi.json",
        "docs/openapi.yaml",
        "docs/swagger.json",
        "api/openapi.json",
        "api/openapi.yaml",
    ];

    for candidate in candidate_openapi_files {
        let p = dir.join(candidate);
        if p.is_file() {
            has_openapi = true;
            openapi_path = Some(candidate.to_string());
            if let Ok(content) = fs::read_to_string(&p) {
                if candidate.ends_with(".json") {
                    if let Ok(parsed) = parse_openapi_json(&content, candidate) {
                        endpoints.extend(parsed);
                    }
                } else {
                    // Basic YAML route extraction
                    let parsed = parse_openapi_yaml_simple(&content, candidate);
                    endpoints.extend(parsed);
                }
            }
            break;
        }
    }

    // 3. Framework route discovery from source code files (LP-0814, LP-0815, LP-0820)
    let mut collected_files = Vec::new();
    collect_source_files(dir, 0, &mut collected_files, &mut scanned_files_count);

    let route_regexes = get_route_regexes();

    for file_path in collected_files {
        let relative_path = file_path
            .strip_prefix(dir)
            .unwrap_or(&file_path)
            .to_string_lossy()
            .replace('\\', "/");

        let Ok(metadata) = fs::metadata(&file_path) else {
            continue;
        };

        if metadata.len() > MAX_FILE_SIZE_BYTES {
            continue;
        }

        let Ok(content) = fs::read_to_string(&file_path) else {
            continue;
        };

        for (framework, regex, method_idx, path_idx) in &route_regexes {
            for line_idx in 0..content.lines().count() {
                let Some(line) = content.lines().nth(line_idx) else {
                    continue;
                };

                for cap in regex.captures_iter(line) {
                    let raw_method = cap.get(*method_idx).map(|m| m.as_str()).unwrap_or("GET");
                    let method = normalize_method(raw_method);
                    let path = cap.get(*path_idx).map(|p| p.as_str()).unwrap_or("/");

                    if !path.starts_with('/') && !path.starts_with("http") && !path.is_empty() {
                        continue;
                    }

                    let clean_path = if path.starts_with('/') {
                        path.to_string()
                    } else {
                        format!("/{path}")
                    };

                    let auth_hint = if line.contains("auth") || line.contains("jwt") || line.contains("bearer") || line.contains("Authorize") {
                        Some("Authentication Required".to_string())
                    } else {
                        None
                    };

                    let name = format!("{method} {clean_path}");

                    // Deduplicate
                    if !endpoints.iter().any(|e| e.method == method && e.path == clean_path) {
                        endpoints.push(DiscoveredEndpoint {
                            name,
                            method,
                            path: clean_path,
                            description: Some(format!("Discovered from {framework} route in {relative_path}")),
                            source_file: relative_path.clone(),
                            line_number: Some(line_idx + 1),
                            auth_hint,
                            framework: framework.to_string(),
                        });
                    }
                }
            }
        }
    }

    if scanned_files_count >= MAX_SCANNED_FILES {
        warnings.push("Maximum file scan limit reached (500 files). Some files may have been skipped.".to_string());
    }

    if frameworks.is_empty() && !endpoints.is_empty() {
        frameworks.push("Generic HTTP".to_string());
    }

    Ok(SourceProjectReport {
        directory: directory_path.to_string(),
        project_type,
        frameworks,
        has_openapi,
        openapi_path,
        endpoints,
        scanned_files_count,
        warnings,
    })
}

fn collect_source_files(
    current_dir: &Path,
    depth: usize,
    collected: &mut Vec<PathBuf>,
    scanned_count: &mut usize,
) {
    if depth > MAX_SEARCH_DEPTH || *scanned_count >= MAX_SCANNED_FILES {
        return;
    }

    let Ok(entries) = fs::read_dir(current_dir) else {
        return;
    };

    for entry in entries.flatten() {
        if *scanned_count >= MAX_SCANNED_FILES {
            break;
        }

        let path = entry.path();
        let file_name = entry.file_name().to_string_lossy().to_string();

        if path.is_dir() {
            if !IGNORED_DIRS.contains(&file_name.as_str()) && !file_name.starts_with('.') {
                collect_source_files(&path, depth + 1, collected, scanned_count);
            }
        } else if path.is_file() {
            *scanned_count += 1;
            if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                let ext_lower = ext.to_lowercase();
                if ["js", "ts", "jsx", "tsx", "py", "java", "go", "php", "cs", "rb"].contains(&ext_lower.as_str()) {
                    collected.push(path);
                }
            }
        }
    }
}

fn get_route_regexes() -> Vec<(&'static str, Regex, usize, usize)> {
    vec![
        // Express / NestJS: app.get('/users') or router.post("/users")
        (
            "Express",
            Regex::new(r#"(?:app|router)\s*\.\s*(get|post|put|delete|patch|options|head)\s*\(\s*['"]([^'"]+)['"]"#).unwrap(),
            1,
            2,
        ),
        // FastAPI / Flask: @app.get("/users") or @router.post("/items")
        (
            "FastAPI / Python",
            Regex::new(r#"@(?:app|router)\s*\.\s*(get|post|put|delete|patch)\s*\(\s*['"]([^'"]+)['"]"#).unwrap(),
            1,
            2,
        ),
        // Spring Boot: @GetMapping("/users") or @PostMapping(value = "/save")
        (
            "Spring Boot",
            Regex::new(r#"@(Get|Post|Put|Delete|Patch)Mapping\s*\(\s*(?:value\s*=\s*)?['"]([^'"]+)['"]"#).unwrap(),
            1,
            2,
        ),
        // Go: r.GET("/users", ...) or e.POST("/items", ...)
        (
            "Go Web",
            Regex::new(r#"\.\s*(GET|POST|PUT|DELETE|PATCH)\s*\(\s*['"]([^'"]+)['"]"#).unwrap(),
            1,
            2,
        ),
        // Laravel: Route::get('/users', ...)
        (
            "Laravel",
            Regex::new(r#"Route\s*::\s*(get|post|put|delete|patch)\s*\(\s*['"]([^'"]+)['"]"#).unwrap(),
            1,
            2,
        ),
        // ASP.NET Core: [HttpGet("api/users")]
        (
            "ASP.NET Core",
            Regex::new(r#"\[Http(Get|Post|Put|Delete|Patch)\s*\(\s*["']([^"']+)["']\s*\)\]"#).unwrap(),
            1,
            2,
        ),
    ]
}

fn normalize_method(method: &str) -> String {
    let m = method.to_uppercase();
    match m.as_str() {
        "GET" | "POST" | "PUT" | "DELETE" | "PATCH" | "HEAD" | "OPTIONS" => m,
        _ => "GET".to_string(),
    }
}

/// Parses an OpenAPI v2 / v3 JSON document into DiscoveredEndpoints (LP-0813).
fn parse_openapi_json(content: &str, file_name: &str) -> Result<Vec<DiscoveredEndpoint>, AppError> {
    let root: serde_json::Value = serde_json::from_str(content)
        .map_err(|e| AppError::Validation(format!("Failed to parse OpenAPI JSON: {e}")))?;

    let mut endpoints = Vec::new();
    let Some(paths) = root.get("paths").and_then(|p| p.as_object()) else {
        return Ok(endpoints);
    };

    for (path, item) in paths {
        let Some(methods_obj) = item.as_object() else {
            continue;
        };

        for (method_str, details) in methods_obj {
            let m = method_str.to_uppercase();
            if !["GET", "POST", "PUT", "DELETE", "PATCH", "HEAD", "OPTIONS"].contains(&m.as_str()) {
                continue;
            }

            let summary = details
                .get("summary")
                .and_then(|s| s.as_str())
                .or_else(|| details.get("operationId").and_then(|s| s.as_str()))
                .unwrap_or(path.as_str());

            let description = details
                .get("description")
                .and_then(|d| d.as_str())
                .map(|s| s.to_string());

            let has_security = details.get("security").is_some() || root.get("security").is_some();
            let auth_hint = if has_security {
                Some("OpenAPI Security Defined".to_string())
            } else {
                None
            };

            endpoints.push(DiscoveredEndpoint {
                name: format!("{m} {summary}"),
                method: m,
                path: path.clone(),
                description,
                source_file: file_name.to_string(),
                line_number: None,
                auth_hint,
                framework: "OpenAPI Specification".to_string(),
            });
        }
    }

    Ok(endpoints)
}

/// Simple regex-based fallback for OpenAPI YAML paths (LP-0813).
fn parse_openapi_yaml_simple(content: &str, file_name: &str) -> Vec<DiscoveredEndpoint> {
    let mut endpoints = Vec::new();
    let mut current_path: Option<String> = None;

    let path_re = Regex::new(r#"^\s{2}(/[^:\s]+):\s*$"#).unwrap();
    let method_re = Regex::new(r#"^\s{4}(get|post|put|delete|patch|head|options):\s*$"#).unwrap();

    for (line_no, line) in content.lines().enumerate() {
        if let Some(cap) = path_re.captures(line) {
            current_path = Some(cap[1].to_string());
        } else if let Some(cap) = method_re.captures(line) {
            if let Some(ref path) = current_path {
                let method = cap[1].to_uppercase();
                endpoints.push(DiscoveredEndpoint {
                    name: format!("{method} {path}"),
                    method,
                    path: path.clone(),
                    description: Some("Extracted from OpenAPI YAML".to_string()),
                    source_file: file_name.to_string(),
                    line_number: Some(line_no + 1),
                    auth_hint: None,
                    framework: "OpenAPI Specification".to_string(),
                });
            }
        }
    }

    endpoints
}

// Database helper functions for Project Source Associations (LP-0810)
pub fn get_project_source_association(
    conn: &Connection,
    project_id: &str,
) -> Result<Option<String>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT source_directory FROM project_source_associations WHERE project_id = ?1",
    )?;
    let mut rows = stmt.query(params![project_id])?;
    if let Some(row) = rows.next()? {
        Ok(Some(row.get(0)?))
    } else {
        Ok(None)
    }
}

pub fn set_project_source_association(
    conn: &Connection,
    project_id: &str,
    source_directory: &str,
    framework: Option<&str>,
) -> Result<(), AppError> {
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO project_source_associations (project_id, source_directory, framework, detected_at)
         VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT(project_id) DO UPDATE SET
            source_directory = excluded.source_directory,
            framework = excluded.framework,
            detected_at = excluded.detected_at",
        params![project_id, source_directory, framework, now],
    )?;
    Ok(())
}

pub fn remove_project_source_association(
    conn: &Connection,
    project_id: &str,
) -> Result<(), AppError> {
    conn.execute(
        "DELETE FROM project_source_associations WHERE project_id = ?1",
        params![project_id],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;

    #[test]
    fn parses_openapi_json_successfully() {
        let json = r#"{
            "openapi": "3.0.0",
            "paths": {
                "/users": {
                    "get": {
                        "summary": "List users",
                        "security": [{ "bearerAuth": [] }]
                    },
                    "post": {
                        "summary": "Create user"
                    }
                },
                "/users/{id}": {
                    "delete": {
                        "summary": "Delete user"
                    }
                }
            }
        }"#;

        let endpoints = parse_openapi_json(json, "openapi.json").unwrap();
        assert_eq!(endpoints.len(), 3);
        assert!(endpoints.iter().any(|e| e.method == "GET" && e.path == "/users" && e.auth_hint.is_some()));
        assert!(endpoints.iter().any(|e| e.method == "POST" && e.path == "/users"));
        assert!(endpoints.iter().any(|e| e.method == "DELETE" && e.path == "/users/{id}"));
    }

    #[test]
    fn extracts_express_and_fastapi_routes() {
        let dir_path = std::env::temp_dir().join(format!("test_src_{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir_path).unwrap();

        // Write express route file
        let express_file = dir_path.join("routes.js");
        let mut f = File::create(&express_file).unwrap();
        writeln!(f, "router.get('/api/orders', authMiddleware, handler);").unwrap();
        writeln!(f, "app.post('/api/orders', handler);").unwrap();

        // Write fastapi route file
        let py_file = dir_path.join("main.py");
        let mut f2 = File::create(&py_file).unwrap();
        writeln!(f2, "@app.get('/items')\ndef get_items(): pass").unwrap();
        writeln!(f2, "@router.delete('/items/{{id}}')\ndef del_item(): pass").unwrap();

        let report = scan_source_project(&dir_path.to_string_lossy()).unwrap();
        assert!(report.endpoints.iter().any(|e| e.method == "GET" && e.path == "/api/orders"));
        assert!(report.endpoints.iter().any(|e| e.method == "POST" && e.path == "/api/orders"));
        assert!(report.endpoints.iter().any(|e| e.method == "GET" && e.path == "/items"));
        assert!(report.endpoints.iter().any(|e| e.method == "DELETE" && e.path == "/items/{id}"));

        let _ = fs::remove_dir_all(&dir_path);
    }
}

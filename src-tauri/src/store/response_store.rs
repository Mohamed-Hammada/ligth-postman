use chrono::Utc;
use rusqlite::{params, Connection};
use uuid::Uuid;

use crate::error::AppError;
use crate::http_engine::BodyCapture;
use crate::models::{
    HeaderEntry, ResponseBodyPayload, ResponseCookie, ResponseMeta, ResponseSummary,
};

/// How much of a disk-backed body `get_response_body` will actually read back over the
/// Tauri IPC boundary — independent of how much was captured to disk in the first place.
const BODY_READ_CAP_BYTES: usize = 2 * 1024 * 1024;

pub struct NewResponseInput {
    pub request_id: String,
    pub status: u16,
    pub status_text: String,
    pub headers: Vec<HeaderEntry>,
    pub duration_ms: u64,
    pub body_size: u64,
    pub body: BodyCapture,
}

fn extract_metadata(headers: &[HeaderEntry]) -> (Option<String>, Vec<ResponseCookie>) {
    let content_type = headers
        .iter()
        .find(|h| h.key.eq_ignore_ascii_case("content-type"))
        .map(|h| h.value.clone());

    let cookies = headers
        .iter()
        .filter(|h| h.key.eq_ignore_ascii_case("set-cookie"))
        .filter_map(|h| crate::http_engine::parse_cookie_header(&h.value))
        .collect();

    (content_type, cookies)
}

pub fn create_response(conn: &Connection, input: NewResponseInput) -> Result<ResponseMeta, AppError> {
    let id = Uuid::new_v4().to_string();
    let created_at = Utc::now();
    let (storage, inline_blob, disk_path) = match input.body {
        BodyCapture::Inline(bytes) => ("inline", Some(bytes), None),
        BodyCapture::Spilled { path, .. } => {
            ("disk", None, Some(path.to_string_lossy().to_string()))
        }
    };
    let headers_json = serde_json::to_string(&input.headers)
        .map_err(|err| AppError::Validation(format!("invalid headers: {err}")))?;

    conn.execute(
        "INSERT INTO responses (id, request_id, status, status_text, headers, duration_ms, body_size, body_storage, body_inline, body_path, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        params![
            id,
            input.request_id,
            input.status,
            input.status_text,
            headers_json,
            input.duration_ms,
            input.body_size,
            storage,
            inline_blob,
            disk_path,
            created_at.to_rfc3339()
        ],
    )?;

    let (content_type, cookies) = extract_metadata(&input.headers);

    Ok(ResponseMeta {
        id,
        request_id: input.request_id,
        status: input.status,
        status_text: input.status_text,
        headers: input.headers,
        content_type,
        cookies,
        duration_ms: input.duration_ms,
        body_size: input.body_size,
        created_at,
    })
}

/// History list for a request — status/size/duration only, never the body.
pub fn list_summaries(conn: &Connection, request_id: &str) -> Result<Vec<ResponseSummary>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT id, request_id, status, status_text, duration_ms, body_size, created_at
         FROM responses WHERE request_id = ?1 ORDER BY created_at DESC",
    )?;
    let rows = stmt.query_map(params![request_id], |row| {
        let created_at: String = row.get(6)?;
        Ok(ResponseSummary {
            id: row.get(0)?,
            request_id: row.get(1)?,
            status: row.get(2)?,
            status_text: row.get(3)?,
            duration_ms: row.get(4)?,
            body_size: row.get(5)?,
            created_at: created_at.parse().unwrap_or_else(|_| Utc::now()),
        })
    })?;
    rows.collect::<Result<Vec<_>, _>>().map_err(AppError::from)
}

pub fn get_response(conn: &Connection, id: &str) -> Result<ResponseMeta, AppError> {
    conn.query_row(
        "SELECT id, request_id, status, status_text, headers, duration_ms, body_size, created_at
         FROM responses WHERE id = ?1",
        params![id],
        |row| {
            let headers_json: String = row.get(4)?;
            let created_at: String = row.get(7)?;
            let headers: Vec<HeaderEntry> = serde_json::from_str(&headers_json).unwrap_or_default();
            let (content_type, cookies) = extract_metadata(&headers);
            Ok(ResponseMeta {
                id: row.get(0)?,
                request_id: row.get(1)?,
                status: row.get(2)?,
                status_text: row.get(3)?,
                headers,
                content_type,
                cookies,
                duration_ms: row.get(5)?,
                body_size: row.get(6)?,
                created_at: created_at.parse().unwrap_or_else(|_| Utc::now()),
            })
        },
    )
    .map_err(|err| match err {
        rusqlite::Error::QueryReturnedNoRows => AppError::NotFound(format!("response {id} not found")),
        other => AppError::from(other),
    })
}

/// Reads the body back, capped at `BODY_READ_CAP_BYTES` regardless of storage — a disk-backed
/// body can be far larger than we'll ever hand over the Tauri IPC channel in one call.
pub fn get_response_body(conn: &Connection, id: &str) -> Result<ResponseBodyPayload, AppError> {
    let (storage, inline, path): (String, Option<Vec<u8>>, Option<String>) = conn
        .query_row(
            "SELECT body_storage, body_inline, body_path FROM responses WHERE id = ?1",
            params![id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .map_err(|err| match err {
            rusqlite::Error::QueryReturnedNoRows => AppError::NotFound(format!("response {id} not found")),
            other => AppError::from(other),
        })?;

    match storage.as_str() {
        "inline" => {
            let bytes = inline.unwrap_or_default();
            let truncated = bytes.len() > BODY_READ_CAP_BYTES;
            let slice = &bytes[..bytes.len().min(BODY_READ_CAP_BYTES)];
            Ok(ResponseBodyPayload { text: String::from_utf8_lossy(slice).into_owned(), truncated })
        }
        "disk" => {
            let path = path.ok_or_else(|| AppError::Storage("disk-backed response missing a path".into()))?;
            let bytes = std::fs::read(&path).map_err(|err| AppError::Storage(err.to_string()))?;
            let truncated = bytes.len() > BODY_READ_CAP_BYTES;
            let slice = &bytes[..bytes.len().min(BODY_READ_CAP_BYTES)];
            Ok(ResponseBodyPayload { text: String::from_utf8_lossy(slice).into_owned(), truncated })
        }
        other => Err(AppError::Storage(format!("unknown body_storage '{other}'"))),
    }
}

pub fn delete_response(conn: &Connection, id: &str) -> Result<(), AppError> {
    delete_disk_file_if_any(conn, id)?;
    let affected = conn.execute("DELETE FROM responses WHERE id = ?1", params![id])?;
    if affected == 0 {
        return Err(AppError::NotFound(format!("response {id} not found")));
    }
    Ok(())
}

fn delete_disk_file_if_any(conn: &Connection, id: &str) -> Result<(), AppError> {
    let path: Option<String> = conn
        .query_row("SELECT body_path FROM responses WHERE id = ?1", params![id], |row| row.get(0))
        .unwrap_or(None);
    if let Some(path) = path {
        let _ = std::fs::remove_file(path); // best-effort; a missing file is not an error here
    }
    Ok(())
}

/// Must run *before* deleting the request/project row — once the row is gone (cascaded),
/// there is nothing left to tell us which files belonged to it.
pub fn delete_disk_files_for_request(conn: &Connection, request_id: &str) -> Result<(), AppError> {
    let mut stmt = conn.prepare(
        "SELECT body_path FROM responses WHERE request_id = ?1 AND body_storage = 'disk'",
    )?;
    let paths = stmt.query_map(params![request_id], |row| row.get::<_, String>(0))?;
    for path in paths {
        let _ = std::fs::remove_file(path?);
    }
    Ok(())
}

pub fn delete_disk_files_for_project(conn: &Connection, project_id: &str) -> Result<(), AppError> {
    let mut stmt = conn.prepare(
        "SELECT r.body_path FROM responses r
         JOIN requests q ON q.id = r.request_id
         WHERE q.project_id = ?1 AND r.body_storage = 'disk'",
    )?;
    let paths = stmt.query_map(params![project_id], |row| row.get::<_, String>(0))?;
    for path in paths {
        let _ = std::fs::remove_file(path?);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use crate::models::{NewProjectInput, NewRequestInput};
    use crate::store::{project_store, request_store};

    fn seed_request(conn: &Connection) -> (String, String) {
        let project = project_store::create_project(conn, NewProjectInput { name: "Demo".into() }).unwrap();
        let request = request_store::create_request(
            conn,
            NewRequestInput {
                project_id: project.id.clone(),
                name: "Get".into(),
                method: "GET".into(),
                url: "https://api.example.com".into(),
                headers: vec![],
                query_params: vec![],
                auth: crate::models::Auth::None,
                body: None,
                description: None,
                ..Default::default()
            },
        )
        .unwrap();
        (project.id, request.id)
    }

    #[test]
    fn create_list_get_round_trip_for_inline_body() {
        let conn = db::open_in_memory().unwrap();
        let (_project_id, request_id) = seed_request(&conn);

        let meta = create_response(
            &conn,
            NewResponseInput {
                request_id: request_id.clone(),
                status: 200,
                status_text: "OK".into(),
                headers: vec![HeaderEntry { key: "Content-Type".into(), value: "text/plain".into(), enabled: true, description: None }],
                duration_ms: 42,
                body_size: 5,
                body: BodyCapture::Inline(b"hello".to_vec()),
            },
        )
        .unwrap();

        let summaries = list_summaries(&conn, &request_id).unwrap();
        assert_eq!(summaries.len(), 1);
        assert_eq!(summaries[0].id, meta.id);

        let fetched = get_response(&conn, &meta.id).unwrap();
        assert_eq!(fetched.headers.len(), 1);

        let body = get_response_body(&conn, &meta.id).unwrap();
        assert_eq!(body.text, "hello");
        assert!(!body.truncated);
    }

    #[test]
    fn get_response_body_reads_disk_backed_file() {
        let conn = db::open_in_memory().unwrap();
        let (_project_id, request_id) = seed_request(&conn);

        let dir = std::env::temp_dir().join("postman-client-test-response-store");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join(format!("{}.bin", Uuid::new_v4()));
        std::fs::write(&path, b"disk body contents").unwrap();

        let meta = create_response(
            &conn,
            NewResponseInput {
                request_id,
                status: 200,
                status_text: "OK".into(),
                headers: vec![],
                duration_ms: 10,
                body_size: 19,
                body: BodyCapture::Spilled { path: path.clone(), size: 19 },
            },
        )
        .unwrap();

        let body = get_response_body(&conn, &meta.id).unwrap();
        assert_eq!(body.text, "disk body contents");

        delete_response(&conn, &meta.id).unwrap();
        assert!(!path.exists(), "delete_response must remove the backing file");
    }

    #[test]
    fn delete_disk_files_for_request_removes_files_before_cascade() {
        let conn = db::open_in_memory().unwrap();
        let (_project_id, request_id) = seed_request(&conn);

        let dir = std::env::temp_dir().join("postman-client-test-response-store-cascade");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join(format!("{}.bin", Uuid::new_v4()));
        std::fs::write(&path, b"will be orphaned without explicit cleanup").unwrap();

        create_response(
            &conn,
            NewResponseInput {
                request_id: request_id.clone(),
                status: 200,
                status_text: "OK".into(),
                headers: vec![],
                duration_ms: 10,
                body_size: 42,
                body: BodyCapture::Spilled { path: path.clone(), size: 42 },
            },
        )
        .unwrap();

        delete_disk_files_for_request(&conn, &request_id).unwrap();
        assert!(!path.exists());
    }
}

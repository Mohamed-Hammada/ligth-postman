use chrono::Utc;
use rusqlite::{params, Connection};
use uuid::Uuid;

use crate::error::AppError;
use crate::models::{HeaderEntry, NewRequestInput, RequestFull, RequestSummary, VALID_METHODS};

pub fn create_request(conn: &Connection, input: NewRequestInput) -> Result<RequestFull, AppError> {
    let name = input.name.trim();
    if name.is_empty() {
        return Err(AppError::Validation("request name must not be empty".into()));
    }
    let url = input.url.trim();
    if url.is_empty() {
        return Err(AppError::Validation("request url must not be empty".into()));
    }
    let method = input.method.trim().to_uppercase();
    if !VALID_METHODS.contains(&method.as_str()) {
        return Err(AppError::Validation(format!(
            "unsupported method '{method}'"
        )));
    }

    // Foreign key ON DELETE CASCADE + PRAGMA foreign_keys=ON turns a bad project_id
    // into a clean rusqlite constraint error rather than an orphaned row.
    let headers_json = serde_json::to_string(&input.headers)
        .map_err(|err| AppError::Validation(format!("invalid headers: {err}")))?;

    let request = RequestFull {
        id: Uuid::new_v4().to_string(),
        project_id: input.project_id,
        name: name.to_string(),
        method,
        url: url.to_string(),
        headers: input.headers,
        body: input.body,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    conn.execute(
        "INSERT INTO requests (id, project_id, name, method, url, headers, body, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            request.id,
            request.project_id,
            request.name,
            request.method,
            request.url,
            headers_json,
            request.body,
            request.created_at.to_rfc3339(),
            request.updated_at.to_rfc3339()
        ],
    )
    .map_err(|err| match err {
        rusqlite::Error::SqliteFailure(e, _) if e.code == rusqlite::ErrorCode::ConstraintViolation => {
            AppError::Validation(format!("project {} does not exist", request.project_id))
        }
        other => AppError::from(other),
    })?;

    Ok(request)
}

/// Never selects `headers`/`body` — list views must stay cheap even with 10k+ requests (README §20).
pub fn list_requests(conn: &Connection, project_id: &str) -> Result<Vec<RequestSummary>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT id, project_id, name, method, url, updated_at
         FROM requests WHERE project_id = ?1 ORDER BY updated_at DESC",
    )?;
    let rows = stmt.query_map(params![project_id], |row| {
        let updated_at: String = row.get(5)?;
        Ok(RequestSummary {
            id: row.get(0)?,
            project_id: row.get(1)?,
            name: row.get(2)?,
            method: row.get(3)?,
            url: row.get(4)?,
            updated_at: updated_at.parse().unwrap_or_else(|_| Utc::now()),
        })
    })?;
    rows.collect::<Result<Vec<_>, _>>().map_err(AppError::from)
}

pub fn get_request(conn: &Connection, id: &str) -> Result<RequestFull, AppError> {
    conn.query_row(
        "SELECT id, project_id, name, method, url, headers, body, created_at, updated_at
         FROM requests WHERE id = ?1",
        params![id],
        |row| {
            let headers_json: String = row.get(5)?;
            let created_at: String = row.get(7)?;
            let updated_at: String = row.get(8)?;
            Ok(RequestFull {
                id: row.get(0)?,
                project_id: row.get(1)?,
                name: row.get(2)?,
                method: row.get(3)?,
                url: row.get(4)?,
                headers: serde_json::from_str::<Vec<HeaderEntry>>(&headers_json)
                    .unwrap_or_default(),
                body: row.get(6)?,
                created_at: created_at.parse().unwrap_or_else(|_| Utc::now()),
                updated_at: updated_at.parse().unwrap_or_else(|_| Utc::now()),
            })
        },
    )
    .map_err(|err| match err {
        rusqlite::Error::QueryReturnedNoRows => {
            AppError::NotFound(format!("request {id} not found"))
        }
        other => AppError::from(other),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use crate::store::project_store;
    use crate::models::NewProjectInput;

    fn seed_project(conn: &Connection) -> String {
        project_store::create_project(conn, NewProjectInput { name: "Demo".into() })
            .unwrap()
            .id
    }

    #[test]
    fn create_rejects_unknown_method() {
        let conn = db::open_in_memory().unwrap();
        let project_id = seed_project(&conn);
        let result = create_request(
            &conn,
            NewRequestInput {
                project_id,
                name: "Get users".into(),
                method: "FETCH".into(),
                url: "https://api.example.com/users".into(),
                headers: vec![],
                body: None,
            },
        );
        assert!(matches!(result, Err(AppError::Validation(_))));
    }

    #[test]
    fn create_rejects_missing_project() {
        let conn = db::open_in_memory().unwrap();
        let result = create_request(
            &conn,
            NewRequestInput {
                project_id: "ghost-project".into(),
                name: "Get users".into(),
                method: "GET".into(),
                url: "https://api.example.com/users".into(),
                headers: vec![],
                body: None,
            },
        );
        assert!(matches!(result, Err(AppError::Validation(_))));
    }

    #[test]
    fn list_is_lightweight_and_get_is_full() {
        let conn = db::open_in_memory().unwrap();
        let project_id = seed_project(&conn);
        let created = create_request(
            &conn,
            NewRequestInput {
                project_id: project_id.clone(),
                name: "Get users".into(),
                method: "get".into(), // lowercase input must normalize
                url: "https://api.example.com/users".into(),
                headers: vec![HeaderEntry {
                    key: "Authorization".into(),
                    value: "Bearer token".into(),
                    enabled: true,
                }],
                body: Some("{}".into()),
            },
        )
        .unwrap();
        assert_eq!(created.method, "GET");

        let summaries = list_requests(&conn, &project_id).unwrap();
        assert_eq!(summaries.len(), 1);
        assert_eq!(summaries[0].id, created.id);

        let full = get_request(&conn, &created.id).unwrap();
        assert_eq!(full.headers.len(), 1);
        assert_eq!(full.body, Some("{}".into()));
    }

    #[test]
    fn get_missing_request_returns_not_found() {
        let conn = db::open_in_memory().unwrap();
        let result = get_request(&conn, "does-not-exist");
        assert!(matches!(result, Err(AppError::NotFound(_))));
    }
}

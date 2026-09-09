use chrono::Utc;
use rusqlite::{params, Connection};
use uuid::Uuid;

use crate::error::AppError;
use crate::models::{
    HeaderEntry, NewRequestInput, QueryParam, RequestFull, RequestSummary, UpdateRequestInput,
    VALID_METHODS,
};

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
    let query_params_json = serde_json::to_string(&input.query_params)
        .map_err(|err| AppError::Validation(format!("invalid query params: {err}")))?;

    let request = RequestFull {
        id: Uuid::new_v4().to_string(),
        project_id: input.project_id,
        name: name.to_string(),
        method,
        url: url.to_string(),
        headers: input.headers,
        query_params: input.query_params,
        body: input.body,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    conn.execute(
        "INSERT INTO requests (id, project_id, name, method, url, headers, query_params, body, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![
            request.id,
            request.project_id,
            request.name,
            request.method,
            request.url,
            headers_json,
            query_params_json,
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
        "SELECT id, project_id, name, method, url, headers, query_params, body, created_at, updated_at
         FROM requests WHERE id = ?1",
        params![id],
        |row| {
            let headers_json: String = row.get(5)?;
            let query_params_json: String = row.get(6)?;
            let created_at: String = row.get(8)?;
            let updated_at: String = row.get(9)?;
            Ok(RequestFull {
                id: row.get(0)?,
                project_id: row.get(1)?,
                name: row.get(2)?,
                method: row.get(3)?,
                url: row.get(4)?,
                headers: serde_json::from_str::<Vec<HeaderEntry>>(&headers_json)
                    .unwrap_or_default(),
                query_params: serde_json::from_str::<Vec<QueryParam>>(&query_params_json)
                    .unwrap_or_default(),
                body: row.get(7)?,
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

/// Merges only the fields the caller actually provided onto the stored row, then writes
/// the result back in one transaction. Unset fields must survive untouched.
pub fn update_request(conn: &Connection, input: UpdateRequestInput) -> Result<RequestFull, AppError> {
    let existing = get_request(conn, &input.id)?;

    let name = match input.name {
        Some(name) => {
            let trimmed = name.trim();
            if trimmed.is_empty() {
                return Err(AppError::Validation("request name must not be empty".into()));
            }
            trimmed.to_string()
        }
        None => existing.name,
    };

    let method = match input.method {
        Some(method) => {
            let upper = method.trim().to_uppercase();
            if !VALID_METHODS.contains(&upper.as_str()) {
                return Err(AppError::Validation(format!("unsupported method '{upper}'")));
            }
            upper
        }
        None => existing.method,
    };

    let url = match input.url {
        Some(url) => {
            let trimmed = url.trim();
            if trimmed.is_empty() {
                return Err(AppError::Validation("request url must not be empty".into()));
            }
            trimmed.to_string()
        }
        None => existing.url,
    };

    let headers = input.headers.unwrap_or(existing.headers);
    let query_params = input.query_params.unwrap_or(existing.query_params);
    let body = if input.clear_body {
        None
    } else {
        input.body.or(existing.body)
    };

    let headers_json = serde_json::to_string(&headers)
        .map_err(|err| AppError::Validation(format!("invalid headers: {err}")))?;
    let query_params_json = serde_json::to_string(&query_params)
        .map_err(|err| AppError::Validation(format!("invalid query params: {err}")))?;
    let updated_at = Utc::now();

    let tx = conn.unchecked_transaction()?;
    tx.execute(
        "UPDATE requests SET name = ?1, method = ?2, url = ?3, headers = ?4, query_params = ?5, body = ?6, updated_at = ?7
         WHERE id = ?8",
        params![name, method, url, headers_json, query_params_json, body, updated_at.to_rfc3339(), existing.id],
    )?;
    tx.commit()?;

    Ok(RequestFull {
        id: existing.id,
        project_id: existing.project_id,
        name,
        method,
        url,
        headers,
        query_params,
        body,
        created_at: existing.created_at,
        updated_at,
    })
}

pub fn delete_request(conn: &Connection, id: &str) -> Result<(), AppError> {
    // Must run before the DELETE below: once the row (and its responses, via cascade) is
    // gone there is no way to know which on-disk bodies belonged to it.
    crate::store::response_store::delete_disk_files_for_request(conn, id)?;
    let affected = conn.execute("DELETE FROM requests WHERE id = ?1", params![id])?;
    if affected == 0 {
        return Err(AppError::NotFound(format!("request {id} not found")));
    }
    Ok(())
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
                query_params: vec![],
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
                query_params: vec![],
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
                query_params: vec![],
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
        assert_eq!(full.query_params.len(), 0);
    }

    #[test]
    fn get_missing_request_returns_not_found() {
        let conn = db::open_in_memory().unwrap();
        let result = get_request(&conn, "does-not-exist");
        assert!(matches!(result, Err(AppError::NotFound(_))));
    }

    fn seed_request(conn: &Connection, project_id: &str) -> RequestFull {
        create_request(
            conn,
            NewRequestInput {
                project_id: project_id.into(),
                name: "Get users".into(),
                method: "GET".into(),
                url: "https://api.example.com/users".into(),
                headers: vec![HeaderEntry { key: "X-A".into(), value: "1".into(), enabled: true }],
                query_params: vec![QueryParam { key: "page".into(), value: "1".into(), enabled: true, description: None }],
                body: Some("{}".into()),
            },
        )
        .unwrap()
    }

    #[test]
    fn update_with_only_name_preserves_url_headers_and_body() {
        let conn = db::open_in_memory().unwrap();
        let project_id = seed_project(&conn);
        let created = seed_request(&conn, &project_id);

        let updated = update_request(
            &conn,
            UpdateRequestInput {
                id: created.id.clone(),
                name: Some("Get all users".into()),
                method: None,
                url: None,
                headers: None,
                query_params: None,
                body: None,
                clear_body: false,
            },
        )
        .unwrap();

        assert_eq!(updated.name, "Get all users");
        assert_eq!(updated.url, created.url);
        assert_eq!(updated.headers.len(), 1);
        assert_eq!(updated.query_params.len(), 1);
        assert_eq!(updated.body, Some("{}".into()));
        assert_eq!(updated.created_at, created.created_at);
    }

    #[test]
    fn update_can_explicitly_clear_body() {
        let conn = db::open_in_memory().unwrap();
        let project_id = seed_project(&conn);
        let created = seed_request(&conn, &project_id);

        let updated = update_request(
            &conn,
            UpdateRequestInput {
                id: created.id,
                name: None,
                method: None,
                url: None,
                headers: None,
                query_params: None,
                body: None,
                clear_body: true,
            },
        )
        .unwrap();

        assert_eq!(updated.body, None);
    }

    #[test]
    fn update_rejects_invalid_method_without_mutating_row() {
        let conn = db::open_in_memory().unwrap();
        let project_id = seed_project(&conn);
        let created = seed_request(&conn, &project_id);

        let result = update_request(
            &conn,
            UpdateRequestInput {
                id: created.id.clone(),
                name: None,
                method: Some("FETCH".into()),
                url: None,
                headers: None,
                query_params: None,
                body: None,
                clear_body: false,
            },
        );
        assert!(matches!(result, Err(AppError::Validation(_))));

        let unchanged = get_request(&conn, &created.id).unwrap();
        assert_eq!(unchanged.method, "GET");
    }

    #[test]
    fn update_missing_request_returns_not_found() {
        let conn = db::open_in_memory().unwrap();
        let result = update_request(
            &conn,
            UpdateRequestInput {
                id: "ghost".into(),
                name: Some("New name".into()),
                method: None,
                url: None,
                headers: None,
                query_params: None,
                body: None,
                clear_body: false,
            },
        );
        assert!(matches!(result, Err(AppError::NotFound(_))));
    }

    #[test]
    fn delete_removes_request_and_is_idempotent_error_on_second_call() {
        let conn = db::open_in_memory().unwrap();
        let project_id = seed_project(&conn);
        let created = seed_request(&conn, &project_id);

        delete_request(&conn, &created.id).unwrap();
        assert!(matches!(get_request(&conn, &created.id), Err(AppError::NotFound(_))));
        assert!(matches!(delete_request(&conn, &created.id), Err(AppError::NotFound(_))));
    }
}

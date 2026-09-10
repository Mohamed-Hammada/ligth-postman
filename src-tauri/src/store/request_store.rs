use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};
use uuid::Uuid;

use crate::error::AppError;
use crate::models::{
    Auth, Cookie, HeaderEntry, NewCookieInput, NewRequestInput, NewSampleResponseInput,
    QueryParam, RequestFull, RequestSettings, RequestSummary, SampleResponse,
    UpdateRequestInput, VALID_METHODS,
};

pub fn create_request(conn: &Connection, input: NewRequestInput) -> Result<RequestFull, AppError> {
    let name = input.name.trim();
    if name.is_empty() {
        return Err(AppError::Validation("request name must not be empty".into()));
    }
    // Empty is allowed here — a request is a draft-in-progress until it's actually sent, and
    // send time already produces a clear error for an unparseable/empty URL there.
    let url = input.url.trim();
    let method = input.method.trim().to_uppercase();
    if !VALID_METHODS.contains(&method.as_str()) {
        return Err(AppError::Validation(format!(
            "unsupported method '{method}'"
        )));
    }

    let headers_json = serde_json::to_string(&input.headers)
        .map_err(|err| AppError::Validation(format!("invalid headers: {err}")))?;
    let query_params_json = serde_json::to_string(&input.query_params)
        .map_err(|err| AppError::Validation(format!("invalid query params: {err}")))?;
    let auth_json = serde_json::to_string(&input.auth)
        .map_err(|err| AppError::Validation(format!("invalid auth: {err}")))?;
    let settings_json = input
        .settings
        .as_ref()
        .map(|s| serde_json::to_string(s))
        .transpose()
        .map_err(|err| AppError::Validation(format!("invalid settings: {err}")))?;

    let request = RequestFull {
        id: Uuid::new_v4().to_string(),
        project_id: input.project_id,
        folder_id: input.folder_id,
        name: name.to_string(),
        method,
        url: url.to_string(),
        headers: input.headers,
        query_params: input.query_params,
        auth: input.auth,
        body: input.body,
        description: input.description,
        settings: input.settings,
        pre_request_script: input.pre_request_script,
        post_request_script: input.post_request_script,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    conn.execute(
        "INSERT INTO requests (id, project_id, folder_id, name, method, url, headers, query_params, auth, body, description, settings, pre_request_script, post_request_script, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)",
        params![
            request.id,
            request.project_id,
            request.folder_id,
            request.name,
            request.method,
            request.url,
            headers_json,
            query_params_json,
            auth_json,
            request.body,
            request.description,
            settings_json,
            request.pre_request_script,
            request.post_request_script,
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
        "SELECT id, project_id, folder_id, name, method, url, updated_at
         FROM requests WHERE project_id = ?1 ORDER BY updated_at DESC",
    )?;
    let rows = stmt.query_map(params![project_id], |row| {
        let updated_at: String = row.get(6)?;
        Ok(RequestSummary {
            id: row.get(0)?,
            project_id: row.get(1)?,
            folder_id: row.get(2)?,
            name: row.get(3)?,
            method: row.get(4)?,
            url: row.get(5)?,
            updated_at: updated_at.parse().unwrap_or_else(|_| Utc::now()),
        })
    })?;
    rows.collect::<Result<Vec<_>, _>>().map_err(AppError::from)
}

pub fn get_request(conn: &Connection, id: &str) -> Result<RequestFull, AppError> {
    conn.query_row(
        "SELECT id, project_id, folder_id, name, method, url, headers, query_params, auth, body, description, settings, pre_request_script, post_request_script, created_at, updated_at
         FROM requests WHERE id = ?1",
        params![id],
        |row| {
            let headers_json: String = row.get(6)?;
            let query_params_json: String = row.get(7)?;
            let auth_json: String = row.get(8)?;
            let settings_json: Option<String> = row.get(11)?;
            let created_at: String = row.get(14)?;
            let updated_at: String = row.get(15)?;
            Ok(RequestFull {
                id: row.get(0)?,
                project_id: row.get(1)?,
                folder_id: row.get(2)?,
                name: row.get(3)?,
                method: row.get(4)?,
                url: row.get(5)?,
                headers: serde_json::from_str::<Vec<HeaderEntry>>(&headers_json)
                    .unwrap_or_default(),
                query_params: serde_json::from_str::<Vec<QueryParam>>(&query_params_json)
                    .unwrap_or_default(),
                auth: serde_json::from_str::<Auth>(&auth_json).unwrap_or(Auth::None),
                body: row.get(9)?,
                description: row.get(10)?,
                settings: settings_json.as_deref().and_then(|s| serde_json::from_str::<RequestSettings>(s).ok()),
                pre_request_script: row.get(12)?,
                post_request_script: row.get(13)?,
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

    // Empty is allowed here too, for the same reason as create_request — see comment there.
    let url = match input.url {
        Some(url) => url.trim().to_string(),
        None => existing.url,
    };

    let headers = input.headers.unwrap_or(existing.headers);
    let query_params = input.query_params.unwrap_or(existing.query_params);
    let auth = input.auth.unwrap_or(existing.auth);
    let body = if input.clear_body {
        None
    } else {
        input.body.or(existing.body)
    };
    let description = if input.clear_description {
        None
    } else {
        input.description.or(existing.description)
    };
    let settings = if input.clear_settings {
        None
    } else {
        input.settings.or(existing.settings)
    };
    let pre_request_script = if input.clear_pre_request_script {
        None
    } else {
        input.pre_request_script.or(existing.pre_request_script)
    };
    let post_request_script = if input.clear_post_request_script {
        None
    } else {
        input.post_request_script.or(existing.post_request_script)
    };
    let folder_id = if input.clear_folder_id {
        None
    } else {
        input.folder_id.or(existing.folder_id)
    };

    let headers_json = serde_json::to_string(&headers)
        .map_err(|err| AppError::Validation(format!("invalid headers: {err}")))?;
    let query_params_json = serde_json::to_string(&query_params)
        .map_err(|err| AppError::Validation(format!("invalid query params: {err}")))?;
    let auth_json = serde_json::to_string(&auth)
        .map_err(|err| AppError::Validation(format!("invalid auth: {err}")))?;
    let settings_json = settings
        .as_ref()
        .map(|s| serde_json::to_string(s))
        .transpose()
        .map_err(|err| AppError::Validation(format!("invalid settings: {err}")))?;
    let updated_at = Utc::now();

    let tx = conn.unchecked_transaction()?;
    tx.execute(
        "UPDATE requests SET name = ?1, method = ?2, url = ?3, headers = ?4, query_params = ?5, auth = ?6, body = ?7, description = ?8, settings = ?9, pre_request_script = ?10, post_request_script = ?11, folder_id = ?12, updated_at = ?13
         WHERE id = ?14",
        params![
            name,
            method,
            url,
            headers_json,
            query_params_json,
            auth_json,
            body,
            description,
            settings_json,
            pre_request_script,
            post_request_script,
            folder_id,
            updated_at.to_rfc3339(),
            existing.id
        ],
    )?;
    tx.commit()?;

    Ok(RequestFull {
        id: existing.id,
        project_id: existing.project_id,
        folder_id,
        name,
        method,
        url,
        headers,
        query_params,
        auth,
        body,
        description,
        settings,
        pre_request_script,
        post_request_script,
        created_at: existing.created_at,
        updated_at,
    })
}

pub fn delete_request(conn: &Connection, id: &str) -> Result<(), AppError> {
    crate::store::response_store::delete_disk_files_for_request(conn, id)?;
    let affected = conn.execute("DELETE FROM requests WHERE id = ?1", params![id])?;
    if affected == 0 {
        return Err(AppError::NotFound(format!("request {id} not found")));
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Sample Responses CRUD (LP-0117)
// ---------------------------------------------------------------------------

pub fn create_sample_response(
    conn: &Connection,
    input: NewSampleResponseInput,
) -> Result<SampleResponse, AppError> {
    let name = input.name.trim();
    if name.is_empty() {
        return Err(AppError::Validation("sample response name must not be empty".into()));
    }
    let headers_json = serde_json::to_string(&input.headers)
        .map_err(|err| AppError::Validation(format!("invalid headers: {err}")))?;

    let sample = SampleResponse {
        id: Uuid::new_v4().to_string(),
        request_id: input.request_id,
        name: name.to_string(),
        status: input.status,
        status_text: input.status_text,
        headers: input.headers,
        body: input.body,
        content_type: input.content_type,
        created_at: Utc::now(),
    };

    conn.execute(
        "INSERT INTO sample_responses (id, request_id, name, status, status_text, headers, body, content_type, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            sample.id,
            sample.request_id,
            sample.name,
            sample.status,
            sample.status_text,
            headers_json,
            sample.body,
            sample.content_type,
            sample.created_at.to_rfc3339()
        ],
    )?;

    Ok(sample)
}

pub fn list_sample_responses(
    conn: &Connection,
    request_id: &str,
) -> Result<Vec<SampleResponse>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT id, request_id, name, status, status_text, headers, body, content_type, created_at
         FROM sample_responses WHERE request_id = ?1 ORDER BY created_at ASC",
    )?;
    let rows = stmt.query_map(params![request_id], |row| {
        let headers_json: String = row.get(5)?;
        let created_at: String = row.get(8)?;
        Ok(SampleResponse {
            id: row.get(0)?,
            request_id: row.get(1)?,
            name: row.get(2)?,
            status: row.get(3)?,
            status_text: row.get(4)?,
            headers: serde_json::from_str::<Vec<HeaderEntry>>(&headers_json).unwrap_or_default(),
            body: row.get(6)?,
            content_type: row.get(7)?,
            created_at: created_at.parse().unwrap_or_else(|_| Utc::now()),
        })
    })?;
    rows.collect::<Result<Vec<_>, _>>().map_err(AppError::from)
}

pub fn delete_sample_response(conn: &Connection, id: &str) -> Result<(), AppError> {
    let affected = conn.execute("DELETE FROM sample_responses WHERE id = ?1", params![id])?;
    if affected == 0 {
        return Err(AppError::NotFound(format!("sample response {id} not found")));
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Cookies CRUD (LP-0113)
// ---------------------------------------------------------------------------

pub fn create_cookie(conn: &Connection, input: NewCookieInput) -> Result<Cookie, AppError> {
    let name = input.name.trim();
    if name.is_empty() {
        return Err(AppError::Validation("cookie name must not be empty".into()));
    }
    let domain = input.domain.trim();
    if domain.is_empty() {
        return Err(AppError::Validation("cookie domain must not be empty".into()));
    }

    let cookie = Cookie {
        id: Uuid::new_v4().to_string(),
        project_id: input.project_id,
        domain: domain.to_string(),
        path: if input.path.is_empty() { "/".to_string() } else { input.path },
        name: name.to_string(),
        value: input.value,
        expires: input.expires,
        secure: input.secure,
        http_only: input.http_only,
        same_site: input.same_site,
        created_at: Utc::now(),
    };

    let expires_str = cookie.expires.map(|e| e.to_rfc3339());

    conn.execute(
        "INSERT INTO cookies (id, project_id, domain, path, name, value, expires, secure, http_only, same_site, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        params![
            cookie.id,
            cookie.project_id,
            cookie.domain,
            cookie.path,
            cookie.name,
            cookie.value,
            expires_str,
            if cookie.secure { 1 } else { 0 },
            if cookie.http_only { 1 } else { 0 },
            cookie.same_site,
            cookie.created_at.to_rfc3339()
        ],
    )?;

    Ok(cookie)
}

pub fn upsert_cookie(conn: &Connection, input: NewCookieInput) -> Result<Cookie, AppError> {
    let domain = input.domain.trim();
    let name = input.name.trim();
    if domain.is_empty() || name.is_empty() {
        return Err(AppError::Validation("cookie domain and name must not be empty".into()));
    }
    let path = if input.path.trim().is_empty() { "/" } else { input.path.trim() };

    let existing_id: Option<String> = conn
        .query_row(
            "SELECT id FROM cookies WHERE project_id = ?1 AND domain = ?2 AND path = ?3 AND name = ?4",
            params![input.project_id, domain, path, name],
            |row| row.get(0),
        )
        .optional()?;

    let expires_str = input.expires.map(|e| e.to_rfc3339());

    if let Some(id) = existing_id {
        conn.execute(
            "UPDATE cookies SET value = ?1, expires = ?2, secure = ?3, http_only = ?4, same_site = ?5 WHERE id = ?6",
            params![
                input.value,
                expires_str,
                if input.secure { 1 } else { 0 },
                if input.http_only { 1 } else { 0 },
                input.same_site,
                id
            ],
        )?;
        Ok(Cookie {
            id,
            project_id: input.project_id,
            domain: domain.to_string(),
            path: path.to_string(),
            name: name.to_string(),
            value: input.value,
            expires: input.expires,
            secure: input.secure,
            http_only: input.http_only,
            same_site: input.same_site,
            created_at: Utc::now(),
        })
    } else {
        create_cookie(conn, input)
    }
}

pub fn list_cookies_for_project(
    conn: &Connection,
    project_id: &str,
) -> Result<Vec<Cookie>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT id, project_id, domain, path, name, value, expires, secure, http_only, same_site, created_at
         FROM cookies WHERE project_id = ?1 ORDER BY domain, path, name ASC",
    )?;
    let rows = stmt.query_map(params![project_id], |row| {
        let expires_str: Option<String> = row.get(6)?;
        let secure_int: i64 = row.get(7)?;
        let http_only_int: i64 = row.get(8)?;
        let created_at: String = row.get(10)?;
        Ok(Cookie {
            id: row.get(0)?,
            project_id: row.get(1)?,
            domain: row.get(2)?,
            path: row.get(3)?,
            name: row.get(4)?,
            value: row.get(5)?,
            expires: expires_str.and_then(|s| s.parse().ok()),
            secure: secure_int != 0,
            http_only: http_only_int != 0,
            same_site: row.get(9)?,
            created_at: created_at.parse().unwrap_or_else(|_| Utc::now()),
        })
    })?;
    rows.collect::<Result<Vec<_>, _>>().map_err(AppError::from)
}

pub fn delete_cookie(conn: &Connection, id: &str) -> Result<(), AppError> {
    let affected = conn.execute("DELETE FROM cookies WHERE id = ?1", params![id])?;
    if affected == 0 {
        return Err(AppError::NotFound(format!("cookie {id} not found")));
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
                auth: Auth::None,
                body: None,
                description: None,
                ..Default::default()
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
                auth: Auth::None,
                body: None,
                description: None,
                ..Default::default()
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
                    description: Some("Auth header".into()),
                }],
                query_params: vec![],
                auth: Auth::None,
                body: Some("{}".into()),
                description: Some("Fetch all active users".into()),
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(created.method, "GET");
        assert_eq!(created.description, Some("Fetch all active users".into()));

        let summaries = list_requests(&conn, &project_id).unwrap();
        assert_eq!(summaries.len(), 1);
        assert_eq!(summaries[0].id, created.id);

        let full = get_request(&conn, &created.id).unwrap();
        assert_eq!(full.headers.len(), 1);
        assert_eq!(full.headers[0].description, Some("Auth header".into()));
        assert_eq!(full.body, Some("{}".into()));
        assert_eq!(full.description, Some("Fetch all active users".into()));
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
                headers: vec![HeaderEntry { key: "X-A".into(), value: "1".into(), enabled: true, description: None }],
                query_params: vec![QueryParam { key: "page".into(), value: "1".into(), enabled: true, description: None }],
                auth: Auth::Bearer { token: "seed-token".into() },
                body: Some("{}".into()),
                description: Some("Original description".into()),
                ..Default::default()
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
                auth: None,
                body: None,
                clear_body: false,
                description: None,
                clear_description: false,
                ..Default::default()
            },
        )
        .unwrap();

        assert_eq!(updated.name, "Get all users");
        assert_eq!(updated.url, created.url);
        assert_eq!(updated.headers.len(), 1);
        assert_eq!(updated.query_params.len(), 1);
        assert!(matches!(updated.auth, Auth::Bearer { ref token } if token == "seed-token"));
        assert_eq!(updated.body, Some("{}".into()));
        assert_eq!(updated.description, Some("Original description".into()));
        assert_eq!(updated.created_at, created.created_at);
    }

    #[test]
    fn update_can_change_auth_type() {
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
                auth: Some(Auth::Basic { username: "alice".into(), password: "{{pw}}".into() }),
                body: None,
                clear_body: false,
                description: None,
                clear_description: false,
                ..Default::default()
            },
        )
        .unwrap();

        match updated.auth {
            Auth::Basic { username, password } => {
                assert_eq!(username, "alice");
                assert_eq!(password, "{{pw}}");
            }
            other => panic!("expected Basic auth, got {other:?}"),
        }
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
                auth: None,
                body: None,
                clear_body: true,
                description: None,
                clear_description: false,
                ..Default::default()
            },
        )
        .unwrap();

        assert_eq!(updated.body, None);
    }

    #[test]
    fn update_can_update_and_clear_description() {
        let conn = db::open_in_memory().unwrap();
        let project_id = seed_project(&conn);
        let created = seed_request(&conn, &project_id);

        let updated = update_request(
            &conn,
            UpdateRequestInput {
                id: created.id.clone(),
                name: None,
                method: None,
                url: None,
                headers: None,
                query_params: None,
                auth: None,
                body: None,
                clear_body: false,
                description: Some("Updated docs".into()),
                clear_description: false,
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(updated.description, Some("Updated docs".into()));

        let cleared = update_request(
            &conn,
            UpdateRequestInput {
                id: created.id.clone(),
                name: None,
                method: None,
                url: None,
                headers: None,
                query_params: None,
                auth: None,
                body: None,
                clear_body: false,
                description: None,
                clear_description: true,
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(cleared.description, None);
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
                auth: None,
                body: None,
                clear_body: false,
                description: None,
                clear_description: false,
                ..Default::default()
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
                auth: None,
                body: None,
                clear_body: false,
                description: None,
                clear_description: false,
                ..Default::default()
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

    #[test]
    fn update_can_update_and_clear_settings_and_scripts() {
        let conn = db::open_in_memory().unwrap();
        let project_id = seed_project(&conn);
        let created = seed_request(&conn, &project_id);

        assert!(created.settings.is_none());
        assert!(created.pre_request_script.is_none());
        assert!(created.post_request_script.is_none());

        let settings = RequestSettings {
            timeout_ms: Some(5000),
            follow_redirects: Some(true),
            max_redirects: Some(5),
            verify_ssl: Some(false),
            proxy_url: Some("http://localhost:8080".into()),
            http_version: Some("http1.1".into()),
        };

        let updated = update_request(
            &conn,
            UpdateRequestInput {
                id: created.id.clone(),
                name: None,
                method: None,
                url: None,
                headers: None,
                query_params: None,
                auth: None,
                body: None,
                clear_body: false,
                description: None,
                clear_description: false,
                settings: Some(settings.clone()),
                clear_settings: false,
                pre_request_script: Some("console.log('before');".into()),
                clear_pre_request_script: false,
                post_request_script: Some("console.log('after');".into()),
                clear_post_request_script: false,
                ..Default::default()
            },
        )
        .unwrap();

        assert_eq!(updated.settings, Some(settings));
        assert_eq!(updated.pre_request_script.as_deref(), Some("console.log('before');"));
        assert_eq!(updated.post_request_script.as_deref(), Some("console.log('after');"));

        // Clear settings and scripts
        let cleared = update_request(
            &conn,
            UpdateRequestInput {
                id: created.id.clone(),
                name: None,
                method: None,
                url: None,
                headers: None,
                query_params: None,
                auth: None,
                body: None,
                clear_body: false,
                description: None,
                clear_description: false,
                settings: None,
                clear_settings: true,
                pre_request_script: None,
                clear_pre_request_script: true,
                post_request_script: None,
                clear_post_request_script: true,
                ..Default::default()
            },
        )
        .unwrap();

        assert!(cleared.settings.is_none());
        assert!(cleared.pre_request_script.is_none());
        assert!(cleared.post_request_script.is_none());
    }

    #[test]
    fn sample_responses_crud_and_cascade_delete() {
        let conn = db::open_in_memory().unwrap();
        let project_id = seed_project(&conn);
        let request = seed_request(&conn, &project_id);

        let sample = create_sample_response(
            &conn,
            NewSampleResponseInput {
                request_id: request.id.clone(),
                name: "200 Success Sample".into(),
                status: 200,
                status_text: "OK".into(),
                headers: vec![HeaderEntry {
                    key: "Content-Type".into(),
                    value: "application/json".into(),
                    enabled: true,
                    description: None,
                }],
                body: Some(r#"{"success": true}"#.into()),
                content_type: Some("application/json".into()),
            },
        )
        .unwrap();

        assert_eq!(sample.name, "200 Success Sample");
        assert_eq!(sample.status, 200);

        let list = list_sample_responses(&conn, &request.id).unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].id, sample.id);

        delete_sample_response(&conn, &sample.id).unwrap();
        let empty = list_sample_responses(&conn, &request.id).unwrap();
        assert_eq!(empty.len(), 0);
    }

    #[test]
    fn cookies_crud() {
        let conn = db::open_in_memory().unwrap();
        let project_id = seed_project(&conn);

        let cookie = create_cookie(
            &conn,
            NewCookieInput {
                project_id: project_id.clone(),
                domain: "example.com".into(),
                path: "/api".into(),
                name: "session_id".into(),
                value: "abc123xyz".into(),
                expires: None,
                secure: true,
                http_only: true,
                same_site: Some("Strict".into()),
            },
        )
        .unwrap();

        assert_eq!(cookie.name, "session_id");
        assert_eq!(cookie.value, "abc123xyz");
        assert!(cookie.secure);
        assert!(cookie.http_only);

        let list = list_cookies_for_project(&conn, &project_id).unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].id, cookie.id);

        delete_cookie(&conn, &cookie.id).unwrap();
        let empty = list_cookies_for_project(&conn, &project_id).unwrap();
        assert_eq!(empty.len(), 0);
    }
}

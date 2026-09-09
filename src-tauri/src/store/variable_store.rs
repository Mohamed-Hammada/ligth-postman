use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};
use uuid::Uuid;

use crate::error::AppError;
use crate::models::{NewVariableInput, UpdateVariableInput, Variable, VariableScope};
use crate::resolver::VarMap;

fn parse_scope(raw: &str) -> VariableScope {
    match raw {
        "environment" => VariableScope::Environment,
        "request" => VariableScope::Request,
        _ => VariableScope::Global,
    }
}

/// Every scope must carry exactly its own reference and none of the others — a `global`
/// variable can't also claim to belong to a `request`, etc.
fn validate_scope_refs(input: &NewVariableInput) -> Result<(), AppError> {
    let (has_project, has_env, has_request) = (
        input.project_id.is_some(),
        input.environment_id.is_some(),
        input.request_id.is_some(),
    );
    let ok = match input.scope {
        VariableScope::Global => has_project && !has_env && !has_request,
        VariableScope::Environment => has_env && !has_project && !has_request,
        VariableScope::Request => has_request && !has_project && !has_env,
    };
    if !ok {
        return Err(AppError::Validation(format!(
            "scope '{}' requires exactly its matching reference (project_id/environment_id/request_id)",
            input.scope.as_str()
        )));
    }
    Ok(())
}

pub fn create_variable(conn: &Connection, input: NewVariableInput) -> Result<Variable, AppError> {
    validate_scope_refs(&input)?;

    let key = input.key.trim();
    if key.is_empty() {
        return Err(AppError::Validation("variable key must not be empty".into()));
    }

    if variable_exists(conn, input.scope, &input.project_id, &input.environment_id, &input.request_id, key)? {
        return Err(AppError::Validation(format!(
            "variable '{key}' already exists in this scope"
        )));
    }

    let variable = Variable {
        id: Uuid::new_v4().to_string(),
        scope: input.scope,
        project_id: input.project_id,
        environment_id: input.environment_id,
        request_id: input.request_id,
        key: key.to_string(),
        value: input.value,
        enabled: input.enabled,
        is_secret: input.is_secret,
        description: input.description,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    conn.execute(
        "INSERT INTO variables
            (id, scope, project_id, environment_id, request_id, key, value, enabled, is_secret, description, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
        params![
            variable.id,
            variable.scope.as_str(),
            variable.project_id,
            variable.environment_id,
            variable.request_id,
            variable.key,
            variable.value,
            variable.enabled,
            variable.is_secret,
            variable.description,
            variable.created_at.to_rfc3339(),
            variable.updated_at.to_rfc3339()
        ],
    )
    .map_err(|err| match err {
        rusqlite::Error::SqliteFailure(e, _) if e.code == rusqlite::ErrorCode::ConstraintViolation => {
            AppError::Validation("the referenced project/environment/request does not exist".into())
        }
        other => AppError::from(other),
    })?;

    Ok(variable)
}

fn variable_exists(
    conn: &Connection,
    scope: VariableScope,
    project_id: &Option<String>,
    environment_id: &Option<String>,
    request_id: &Option<String>,
    key: &str,
) -> Result<bool, AppError> {
    let existing: Option<i64> = conn
        .query_row(
            "SELECT 1 FROM variables
             WHERE scope = ?1
               AND project_id IS ?2 AND environment_id IS ?3 AND request_id IS ?4
               AND key = ?5
             LIMIT 1",
            params![scope.as_str(), project_id, environment_id, request_id, key],
            |row| row.get(0),
        )
        .optional()?;
    Ok(existing.is_some())
}

/// Lists variables for exactly one concrete scope instance (e.g. one environment's rows).
/// Includes disabled rows — the UI needs to show and toggle them; resolution excludes them.
pub fn list_variables_for_scope(
    conn: &Connection,
    scope: VariableScope,
    scope_ref: &str,
) -> Result<Vec<Variable>, AppError> {
    let column = match scope {
        VariableScope::Global => "project_id",
        VariableScope::Environment => "environment_id",
        VariableScope::Request => "request_id",
    };
    let sql = format!(
        "SELECT id, scope, project_id, environment_id, request_id, key, value, enabled, is_secret, description, created_at, updated_at
         FROM variables WHERE scope = ?1 AND {column} = ?2 ORDER BY key ASC"
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(params![scope.as_str(), scope_ref], row_to_variable)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(AppError::from)
}

pub fn get_variable(conn: &Connection, id: &str) -> Result<Variable, AppError> {
    conn.query_row(
        "SELECT id, scope, project_id, environment_id, request_id, key, value, enabled, is_secret, description, created_at, updated_at
         FROM variables WHERE id = ?1",
        params![id],
        row_to_variable,
    )
    .map_err(|err| match err {
        rusqlite::Error::QueryReturnedNoRows => AppError::NotFound(format!("variable {id} not found")),
        other => AppError::from(other),
    })
}

pub fn update_variable(conn: &Connection, input: UpdateVariableInput) -> Result<Variable, AppError> {
    let existing = get_variable(conn, &input.id)?;

    let key = match input.key {
        Some(key) => {
            let trimmed = key.trim();
            if trimmed.is_empty() {
                return Err(AppError::Validation("variable key must not be empty".into()));
            }
            trimmed.to_string()
        }
        None => existing.key,
    };
    let value = input.value.unwrap_or(existing.value);
    let enabled = input.enabled.unwrap_or(existing.enabled);
    let is_secret = input.is_secret.unwrap_or(existing.is_secret);
    let description = if input.clear_description {
        None
    } else {
        input.description.or(existing.description)
    };
    let updated_at = Utc::now();

    let tx = conn.unchecked_transaction()?;
    tx.execute(
        "UPDATE variables SET key = ?1, value = ?2, enabled = ?3, is_secret = ?4, description = ?5, updated_at = ?6
         WHERE id = ?7",
        params![key, value, enabled, is_secret, description, updated_at.to_rfc3339(), existing.id],
    )?;
    tx.commit()?;

    Ok(Variable {
        id: existing.id,
        scope: existing.scope,
        project_id: existing.project_id,
        environment_id: existing.environment_id,
        request_id: existing.request_id,
        key,
        value,
        enabled,
        is_secret,
        description,
        created_at: existing.created_at,
        updated_at,
    })
}

pub fn delete_variable(conn: &Connection, id: &str) -> Result<(), AppError> {
    let affected = conn.execute("DELETE FROM variables WHERE id = ?1", params![id])?;
    if affected == 0 {
        return Err(AppError::NotFound(format!("variable {id} not found")));
    }
    Ok(())
}

/// Loads the enabled variables for each real (DB-backed) scope tier as plain maps, ready to
/// hand to `resolver::ScopeChain`. `environment_id`/`request_id` are optional because a
/// request can be resolved with no environment selected, or outside any request context.
pub fn load_scope_maps(
    conn: &Connection,
    project_id: &str,
    environment_id: Option<&str>,
    request_id: Option<&str>,
) -> Result<(VarMap, VarMap, VarMap), AppError> {
    let global = load_enabled_map(conn, "project_id", project_id)?;
    let environment = match environment_id {
        Some(id) => load_enabled_map(conn, "environment_id", id)?,
        None => VarMap::new(),
    };
    let request = match request_id {
        Some(id) => load_enabled_map(conn, "request_id", id)?,
        None => VarMap::new(),
    };
    Ok((global, environment, request))
}

fn load_enabled_map(conn: &Connection, column: &str, scope_ref: &str) -> Result<VarMap, AppError> {
    let sql = format!("SELECT key, value FROM variables WHERE {column} = ?1 AND enabled = 1");
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(params![scope_ref], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    })?;
    rows.collect::<Result<VarMap, _>>().map_err(AppError::from)
}

fn row_to_variable(row: &rusqlite::Row) -> rusqlite::Result<Variable> {
    let scope_raw: String = row.get(1)?;
    let created_at: String = row.get(10)?;
    let updated_at: String = row.get(11)?;
    Ok(Variable {
        id: row.get(0)?,
        scope: parse_scope(&scope_raw),
        project_id: row.get(2)?,
        environment_id: row.get(3)?,
        request_id: row.get(4)?,
        key: row.get(5)?,
        value: row.get(6)?,
        enabled: row.get(7)?,
        is_secret: row.get(8)?,
        description: row.get(9)?,
        created_at: created_at.parse().unwrap_or_else(|_| Utc::now()),
        updated_at: updated_at.parse().unwrap_or_else(|_| Utc::now()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use crate::models::{NewProjectInput, NewRequestInput};
    use crate::store::{project_store, request_store};

    fn seed_project(conn: &Connection) -> String {
        project_store::create_project(conn, NewProjectInput { name: "Demo".into() }).unwrap().id
    }

    fn base_input(scope: VariableScope, key: &str) -> NewVariableInput {
        NewVariableInput {
            scope,
            project_id: None,
            environment_id: None,
            request_id: None,
            key: key.into(),
            value: "value".into(),
            enabled: true,
            is_secret: false,
            description: None,
        }
    }

    #[test]
    fn create_rejects_mismatched_scope_reference() {
        let conn = db::open_in_memory().unwrap();
        let project_id = seed_project(&conn);
        let mut input = base_input(VariableScope::Global, "baseUrl");
        input.request_id = Some("some-request".into()); // wrong ref for Global
        input.project_id = Some(project_id);
        let result = create_variable(&conn, input);
        assert!(matches!(result, Err(AppError::Validation(_))));
    }

    #[test]
    fn create_rejects_duplicate_key_in_same_scope() {
        let conn = db::open_in_memory().unwrap();
        let project_id = seed_project(&conn);
        let mut input = base_input(VariableScope::Global, "baseUrl");
        input.project_id = Some(project_id.clone());
        create_variable(&conn, input.clone()).unwrap();

        let result = create_variable(&conn, input);
        assert!(matches!(result, Err(AppError::Validation(_))));
    }

    #[test]
    fn same_key_is_allowed_across_different_scopes() {
        let conn = db::open_in_memory().unwrap();
        let project_id = seed_project(&conn);
        let request = request_store::create_request(
            &conn,
            NewRequestInput {
                project_id: project_id.clone(),
                name: "Get users".into(),
                method: "GET".into(),
                url: "https://api.example.com/users".into(),
                headers: vec![],
                query_params: vec![],
                body: None,
            },
        )
        .unwrap();

        let mut global = base_input(VariableScope::Global, "token");
        global.project_id = Some(project_id);
        create_variable(&conn, global).unwrap();

        let mut req_scoped = base_input(VariableScope::Request, "token");
        req_scoped.request_id = Some(request.id);
        // Same key, different scope must not collide.
        create_variable(&conn, req_scoped).unwrap();
    }

    #[test]
    fn update_preserves_unset_fields_and_bumps_updated_at() {
        let conn = db::open_in_memory().unwrap();
        let project_id = seed_project(&conn);
        let mut input = base_input(VariableScope::Global, "baseUrl");
        input.project_id = Some(project_id);
        let created = create_variable(&conn, input).unwrap();

        let updated = update_variable(
            &conn,
            UpdateVariableInput {
                id: created.id.clone(),
                key: None,
                value: Some("https://new.example.com".into()),
                enabled: None,
                is_secret: None,
                description: None,
                clear_description: false,
            },
        )
        .unwrap();

        assert_eq!(updated.key, "baseUrl");
        assert_eq!(updated.value, "https://new.example.com");
        assert_eq!(updated.enabled, true);
    }

    #[test]
    fn delete_missing_variable_returns_not_found() {
        let conn = db::open_in_memory().unwrap();
        assert!(matches!(delete_variable(&conn, "ghost"), Err(AppError::NotFound(_))));
    }

    #[test]
    fn secret_values_survive_storage_round_trip_for_internal_use() {
        // Internal `Variable` must keep the real value — only the frontend-facing
        // `VariableView` masks it (see models::VariableView).
        let conn = db::open_in_memory().unwrap();
        let project_id = seed_project(&conn);
        let mut input = base_input(VariableScope::Global, "apiKey");
        input.project_id = Some(project_id);
        input.is_secret = true;
        input.value = "sk-real-secret".into();
        let created = create_variable(&conn, input).unwrap();

        let fetched = get_variable(&conn, &created.id).unwrap();
        assert_eq!(fetched.value, "sk-real-secret");
        assert!(fetched.is_secret);
    }

    #[test]
    fn load_scope_maps_only_includes_enabled_rows() {
        let conn = db::open_in_memory().unwrap();
        let project_id = seed_project(&conn);

        let mut enabled = base_input(VariableScope::Global, "enabledVar");
        enabled.project_id = Some(project_id.clone());
        create_variable(&conn, enabled).unwrap();

        let mut disabled = base_input(VariableScope::Global, "disabledVar");
        disabled.project_id = Some(project_id.clone());
        disabled.enabled = false;
        create_variable(&conn, disabled).unwrap();

        let (global, _env, _req) = load_scope_maps(&conn, &project_id, None, None).unwrap();
        assert!(global.contains_key("enabledVar"));
        assert!(!global.contains_key("disabledVar"));
    }
}

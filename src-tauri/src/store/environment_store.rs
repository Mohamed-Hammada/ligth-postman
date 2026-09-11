use chrono::Utc;
use rusqlite::{params, Connection};
use uuid::Uuid;

use crate::error::AppError;
use crate::models::{Environment, EnvironmentWithProject, NewEnvironmentInput, UpdateEnvironmentInput};

pub fn create_environment(
    conn: &Connection,
    input: NewEnvironmentInput,
) -> Result<Environment, AppError> {
    let name = input.name.trim();
    if name.is_empty() {
        return Err(AppError::Validation("environment name must not be empty".into()));
    }

    let environment = Environment {
        id: Uuid::new_v4().to_string(),
        project_id: input.project_id,
        name: name.to_string(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    conn.execute(
        "INSERT INTO environments (id, project_id, name, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            environment.id,
            environment.project_id,
            environment.name,
            environment.created_at.to_rfc3339(),
            environment.updated_at.to_rfc3339()
        ],
    )
    .map_err(map_constraint_error)?;

    Ok(environment)
}

pub fn list_environments(conn: &Connection, project_id: &str) -> Result<Vec<Environment>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT id, project_id, name, created_at, updated_at FROM environments
         WHERE project_id = ?1 ORDER BY name ASC",
    )?;
    let rows = stmt.query_map(params![project_id], row_to_environment)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(AppError::from)
}

/// Every environment in every project — environment selection is global (the picker in the
/// topbar and the Environments screen lets you pick any of these regardless of which project
/// is currently open), so the UI needs the full set, not just the current project's own rows.
pub fn list_all_environments(conn: &Connection) -> Result<Vec<EnvironmentWithProject>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT e.id, e.project_id, p.name, e.name, e.created_at, e.updated_at
         FROM environments e
         JOIN projects p ON p.id = e.project_id
         ORDER BY p.name ASC, e.name ASC",
    )?;
    let rows = stmt.query_map([], |row| {
        let created_at: String = row.get(4)?;
        let updated_at: String = row.get(5)?;
        Ok(EnvironmentWithProject {
            id: row.get(0)?,
            project_id: row.get(1)?,
            project_name: row.get(2)?,
            name: row.get(3)?,
            created_at: created_at.parse().unwrap_or_else(|_| Utc::now()),
            updated_at: updated_at.parse().unwrap_or_else(|_| Utc::now()),
        })
    })?;
    rows.collect::<Result<Vec<_>, _>>().map_err(AppError::from)
}

pub fn get_environment(conn: &Connection, id: &str) -> Result<Environment, AppError> {
    conn.query_row(
        "SELECT id, project_id, name, created_at, updated_at FROM environments WHERE id = ?1",
        params![id],
        row_to_environment,
    )
    .map_err(|err| match err {
        rusqlite::Error::QueryReturnedNoRows => {
            AppError::NotFound(format!("environment {id} not found"))
        }
        other => AppError::from(other),
    })
}

pub fn update_environment(
    conn: &Connection,
    input: UpdateEnvironmentInput,
) -> Result<Environment, AppError> {
    let existing = get_environment(conn, &input.id)?;

    let name = match input.name {
        Some(name) => {
            let trimmed = name.trim();
            if trimmed.is_empty() {
                return Err(AppError::Validation("environment name must not be empty".into()));
            }
            trimmed.to_string()
        }
        None => existing.name,
    };

    let updated_at = Utc::now();
    let tx = conn.unchecked_transaction()?;
    tx.execute(
        "UPDATE environments SET name = ?1, updated_at = ?2 WHERE id = ?3",
        params![name, updated_at.to_rfc3339(), existing.id],
    )
    .map_err(map_constraint_error)?;
    tx.commit()?;

    Ok(Environment {
        id: existing.id,
        project_id: existing.project_id,
        name,
        created_at: existing.created_at,
        updated_at,
    })
}

/// Deletes the environment; its variables cascade automatically via the
/// `variables.environment_id` foreign key (README §41 shared-vs-local design note applies
/// once we add secure storage — see PROJECT_MAP).
pub fn delete_environment(conn: &Connection, id: &str) -> Result<(), AppError> {
    // No DB-level FK action on default_environment_id (see db.rs migration 12) — clear it here
    // instead, so a project never points at an environment that no longer exists.
    conn.execute(
        "UPDATE projects SET default_environment_id = NULL WHERE default_environment_id = ?1",
        params![id],
    )?;
    let affected = conn.execute("DELETE FROM environments WHERE id = ?1", params![id])?;
    if affected == 0 {
        return Err(AppError::NotFound(format!("environment {id} not found")));
    }
    Ok(())
}

fn map_constraint_error(err: rusqlite::Error) -> AppError {
    match err {
        rusqlite::Error::SqliteFailure(e, _) if e.code == rusqlite::ErrorCode::ConstraintViolation => {
            AppError::Validation(
                "an environment with that name already exists in this project, or the project does not exist".into(),
            )
        }
        other => AppError::from(other),
    }
}

fn row_to_environment(row: &rusqlite::Row) -> rusqlite::Result<Environment> {
    let created_at: String = row.get(3)?;
    let updated_at: String = row.get(4)?;
    Ok(Environment {
        id: row.get(0)?,
        project_id: row.get(1)?,
        name: row.get(2)?,
        created_at: created_at.parse().unwrap_or_else(|_| Utc::now()),
        updated_at: updated_at.parse().unwrap_or_else(|_| Utc::now()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use crate::models::NewProjectInput;
    use crate::store::project_store;

    fn seed_project(conn: &Connection) -> String {
        project_store::create_project(conn, NewProjectInput { name: "Demo".into(), workspace_id: "default".into() }).unwrap().id
    }

    #[test]
    fn create_rejects_blank_name() {
        let conn = db::open_in_memory().unwrap();
        let project_id = seed_project(&conn);
        let result = create_environment(&conn, NewEnvironmentInput { project_id, name: "  ".into() });
        assert!(matches!(result, Err(AppError::Validation(_))));
    }

    #[test]
    fn create_rejects_duplicate_name_within_project() {
        let conn = db::open_in_memory().unwrap();
        let project_id = seed_project(&conn);
        create_environment(&conn, NewEnvironmentInput { project_id: project_id.clone(), name: "Staging".into() }).unwrap();

        let result = create_environment(&conn, NewEnvironmentInput { project_id, name: "Staging".into() });
        assert!(matches!(result, Err(AppError::Validation(_))));
    }

    #[test]
    fn create_rejects_missing_project() {
        let conn = db::open_in_memory().unwrap();
        let result = create_environment(&conn, NewEnvironmentInput { project_id: "ghost".into(), name: "Staging".into() });
        assert!(matches!(result, Err(AppError::Validation(_))));
    }

    #[test]
    fn list_orders_by_name_and_get_round_trips() {
        let conn = db::open_in_memory().unwrap();
        let project_id = seed_project(&conn);
        create_environment(&conn, NewEnvironmentInput { project_id: project_id.clone(), name: "Staging".into() }).unwrap();
        let dev = create_environment(&conn, NewEnvironmentInput { project_id, name: "Development".into() }).unwrap();

        let listed = list_environments(&conn, &dev.project_id).unwrap();
        assert_eq!(listed.len(), 2);
        assert_eq!(listed[0].name, "Development");

        let fetched = get_environment(&conn, &dev.id).unwrap();
        assert_eq!(fetched.name, "Development");
    }

    #[test]
    fn list_all_environments_spans_every_project_with_project_name_attached() {
        let conn = db::open_in_memory().unwrap();
        let project_a = project_store::create_project(&conn, NewProjectInput { name: "Alpha".into(), workspace_id: "default".into() }).unwrap();
        let project_b = project_store::create_project(&conn, NewProjectInput { name: "Beta".into(), workspace_id: "default".into() }).unwrap();
        create_environment(&conn, NewEnvironmentInput { project_id: project_a.id.clone(), name: "Prod".into() }).unwrap();
        create_environment(&conn, NewEnvironmentInput { project_id: project_b.id.clone(), name: "Dev".into() }).unwrap();

        let all = list_all_environments(&conn).unwrap();
        assert_eq!(all.len(), 2);
        assert!(all.iter().any(|e| e.name == "Prod" && e.project_name == "Alpha"));
        assert!(all.iter().any(|e| e.name == "Dev" && e.project_name == "Beta"));
    }

    #[test]
    fn delete_cascades_its_variables() {
        use crate::models::{NewVariableInput, VariableScope};
        use crate::store::variable_store;

        let conn = db::open_in_memory().unwrap();
        let project_id = seed_project(&conn);
        let env = create_environment(&conn, NewEnvironmentInput { project_id, name: "Staging".into() }).unwrap();

        variable_store::create_variable(
            &conn,
            NewVariableInput {
                scope: VariableScope::Environment,
                project_id: None,
                environment_id: Some(env.id.clone()),
                request_id: None,
                key: "baseUrl".into(),
                value: "https://staging.example.com".into(),
                enabled: true,
                is_secret: false,
                is_local: false,
                description: None,
            },
        )
        .unwrap();

        delete_environment(&conn, &env.id).unwrap();

        let remaining: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM variables WHERE environment_id = ?1",
                params![env.id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(remaining, 0);
    }
}

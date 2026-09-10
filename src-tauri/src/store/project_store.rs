use chrono::Utc;
use rusqlite::{params, Connection};
use uuid::Uuid;

use crate::error::AppError;
use crate::models::{NewProjectInput, Project, UpdateProjectInput};

pub fn create_project(conn: &Connection, input: NewProjectInput) -> Result<Project, AppError> {
    let name = input.name.trim();
    if name.is_empty() {
        return Err(AppError::Validation("project name must not be empty".into()));
    }

    let project = Project {
        id: Uuid::new_v4().to_string(),
        name: name.to_string(),
        default_environment_id: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    conn.execute(
        "INSERT INTO projects (id, name, created_at, updated_at) VALUES (?1, ?2, ?3, ?4)",
        params![
            project.id,
            project.name,
            project.created_at.to_rfc3339(),
            project.updated_at.to_rfc3339()
        ],
    )?;

    Ok(project)
}

/// Metadata only — never joins in requests/collections (README §5 project loading).
pub fn list_projects(conn: &Connection) -> Result<Vec<Project>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT id, name, default_environment_id, created_at, updated_at FROM projects ORDER BY updated_at DESC",
    )?;
    let rows = stmt.query_map([], row_to_project)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(AppError::from)
}

/// Request count per project, in one query — for the Launcher screen's per-project metadata.
/// A `GROUP BY` rather than one `COUNT(*)` per project, so listing N projects costs one query
/// regardless of N. Projects with zero requests are simply absent from the map.
pub fn request_counts_by_project(
    conn: &Connection,
) -> Result<std::collections::HashMap<String, usize>, AppError> {
    let mut stmt = conn.prepare("SELECT project_id, COUNT(*) FROM requests GROUP BY project_id")?;
    let rows = stmt.query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)? as usize)))?;
    rows.collect::<Result<std::collections::HashMap<_, _>, _>>().map_err(AppError::from)
}

pub fn get_project(conn: &Connection, id: &str) -> Result<Project, AppError> {
    conn.query_row(
        "SELECT id, name, default_environment_id, created_at, updated_at FROM projects WHERE id = ?1",
        params![id],
        row_to_project,
    )
    .map_err(|err| match err {
        rusqlite::Error::QueryReturnedNoRows => {
            AppError::NotFound(format!("project {id} not found"))
        }
        other => AppError::from(other),
    })
}

/// Only overwrites fields that were actually provided; everything else is preserved as-is.
pub fn update_project(conn: &Connection, input: UpdateProjectInput) -> Result<Project, AppError> {
    let existing = get_project(conn, &input.id)?;

    let name = match input.name {
        Some(name) => {
            let trimmed = name.trim();
            if trimmed.is_empty() {
                return Err(AppError::Validation("project name must not be empty".into()));
            }
            trimmed.to_string()
        }
        None => existing.name,
    };

    let default_environment_id = if input.clear_default_environment_id {
        None
    } else {
        input.default_environment_id.or(existing.default_environment_id)
    };

    let updated_at = Utc::now();

    let tx = conn.unchecked_transaction()?;
    tx.execute(
        "UPDATE projects SET name = ?1, default_environment_id = ?2, updated_at = ?3 WHERE id = ?4",
        params![name, default_environment_id, updated_at.to_rfc3339(), existing.id],
    )?;
    tx.commit()?;

    Ok(Project {
        id: existing.id,
        name,
        default_environment_id,
        created_at: existing.created_at,
        updated_at,
    })
}

/// Deletes the project and, via `ON DELETE CASCADE`, every request that belongs to it.
/// This is a deliberate choice (README §28): a project's requests have no meaning without
/// their parent project, unlike e.g. a future "shared environment" which must not cascade.
pub fn delete_project(conn: &Connection, id: &str) -> Result<(), AppError> {
    // Same reasoning as request_store::delete_request: gather disk-backed response bodies
    // before the cascading DELETE removes the rows that point to them.
    crate::store::response_store::delete_disk_files_for_project(conn, id)?;
    let affected = conn.execute("DELETE FROM projects WHERE id = ?1", params![id])?;
    if affected == 0 {
        return Err(AppError::NotFound(format!("project {id} not found")));
    }
    Ok(())
}

fn row_to_project(row: &rusqlite::Row) -> rusqlite::Result<Project> {
    let created_at: String = row.get(3)?;
    let updated_at: String = row.get(4)?;
    Ok(Project {
        id: row.get(0)?,
        name: row.get(1)?,
        default_environment_id: row.get(2)?,
        created_at: created_at
            .parse()
            .unwrap_or_else(|_| Utc::now()),
        updated_at: updated_at
            .parse()
            .unwrap_or_else(|_| Utc::now()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;

    #[test]
    fn create_rejects_empty_name() {
        let conn = db::open_in_memory().unwrap();
        let result = create_project(&conn, NewProjectInput { name: "   ".into() });
        assert!(matches!(result, Err(AppError::Validation(_))));
    }

    #[test]
    fn create_then_list_then_get_round_trips() {
        let conn = db::open_in_memory().unwrap();
        let created = create_project(&conn, NewProjectInput { name: "Payments API".into() }).unwrap();

        let listed = list_projects(&conn).unwrap();
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].id, created.id);

        let fetched = get_project(&conn, &created.id).unwrap();
        assert_eq!(fetched.name, "Payments API");
    }

    #[test]
    fn get_missing_project_returns_not_found() {
        let conn = db::open_in_memory().unwrap();
        let result = get_project(&conn, "does-not-exist");
        assert!(matches!(result, Err(AppError::NotFound(_))));
    }

    #[test]
    fn update_with_no_fields_preserves_existing_values() {
        let conn = db::open_in_memory().unwrap();
        let created = create_project(&conn, NewProjectInput { name: "Original".into() }).unwrap();

        let updated = update_project(&conn, UpdateProjectInput { id: created.id.clone(), name: None, ..Default::default() }).unwrap();

        assert_eq!(updated.name, "Original");
        assert_eq!(updated.created_at, created.created_at);
        assert!(updated.updated_at >= created.updated_at);
    }

    #[test]
    fn update_rejects_blank_name() {
        let conn = db::open_in_memory().unwrap();
        let created = create_project(&conn, NewProjectInput { name: "Original".into() }).unwrap();

        let result = update_project(
            &conn,
            UpdateProjectInput { id: created.id, name: Some("   ".into()), ..Default::default() },
        );
        assert!(matches!(result, Err(AppError::Validation(_))));
    }

    #[test]
    fn update_missing_project_returns_not_found() {
        let conn = db::open_in_memory().unwrap();
        let result = update_project(
            &conn,
            UpdateProjectInput { id: "ghost".into(), name: Some("New name".into()), ..Default::default() },
        );
        assert!(matches!(result, Err(AppError::NotFound(_))));
    }

    #[test]
    fn default_environment_id_can_be_set_then_cleared() {
        let conn = db::open_in_memory().unwrap();
        let project = create_project(&conn, NewProjectInput { name: "Has Envs".into() }).unwrap();
        assert_eq!(project.default_environment_id, None);
        let env = crate::store::environment_store::create_environment(
            &conn,
            crate::models::NewEnvironmentInput { project_id: project.id.clone(), name: "Staging".into() },
        )
        .unwrap();

        let with_default = update_project(
            &conn,
            UpdateProjectInput { id: project.id.clone(), default_environment_id: Some(env.id.clone()), ..Default::default() },
        )
        .unwrap();
        assert_eq!(with_default.default_environment_id, Some(env.id.clone()));

        // A plain re-fetch must see the same persisted value, not just the in-memory return value.
        let refetched = get_project(&conn, &project.id).unwrap();
        assert_eq!(refetched.default_environment_id, Some(env.id.clone()));

        let cleared = update_project(
            &conn,
            UpdateProjectInput { id: project.id, clear_default_environment_id: true, ..Default::default() },
        )
        .unwrap();
        assert_eq!(cleared.default_environment_id, None);
    }

    #[test]
    fn deleting_the_default_environment_clears_the_projects_reference_to_it() {
        let conn = db::open_in_memory().unwrap();
        let project = create_project(&conn, NewProjectInput { name: "Has Envs".into() }).unwrap();
        let env = crate::store::environment_store::create_environment(
            &conn,
            crate::models::NewEnvironmentInput { project_id: project.id.clone(), name: "Staging".into() },
        )
        .unwrap();
        update_project(
            &conn,
            UpdateProjectInput { id: project.id.clone(), default_environment_id: Some(env.id.clone()), ..Default::default() },
        )
        .unwrap();

        crate::store::environment_store::delete_environment(&conn, &env.id).unwrap();

        let refetched = get_project(&conn, &project.id).unwrap();
        assert_eq!(refetched.default_environment_id, None, "dangling reference to a deleted environment must be cleared");
    }

    #[test]
    fn delete_missing_project_returns_not_found() {
        let conn = db::open_in_memory().unwrap();
        let result = delete_project(&conn, "ghost");
        assert!(matches!(result, Err(AppError::NotFound(_))));
    }

    #[test]
    fn request_counts_by_project_groups_correctly_and_omits_empty_projects() {
        let conn = db::open_in_memory().unwrap();
        let busy = create_project(&conn, NewProjectInput { name: "Busy".into() }).unwrap();
        let empty = create_project(&conn, NewProjectInput { name: "Empty".into() }).unwrap();
        let _ = empty; // exists in the DB but gets no requests — must be absent from the map

        for name in ["Get users", "Create user"] {
            crate::store::request_store::create_request(
                &conn,
                crate::models::NewRequestInput {
                    project_id: busy.id.clone(),
                    name: name.into(),
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
        }

        let counts = request_counts_by_project(&conn).unwrap();
        assert_eq!(counts.get(&busy.id), Some(&2));
        assert_eq!(counts.get(&empty.id), None, "projects with zero requests are absent, not zero");
    }

    #[test]
    fn delete_project_cascades_to_its_requests() {
        let conn = db::open_in_memory().unwrap();
        let project = create_project(&conn, NewProjectInput { name: "Doomed".into() }).unwrap();
        crate::store::request_store::create_request(
            &conn,
            crate::models::NewRequestInput {
                project_id: project.id.clone(),
                name: "Get users".into(),
                method: "GET".into(),
                url: "https://api.example.com/users".into(),
                headers: vec![],
                query_params: vec![],
                auth: crate::models::Auth::None,
                body: None,
                description: None,
                ..Default::default()
            },
        )
        .unwrap();

        delete_project(&conn, &project.id).unwrap();

        let remaining = conn
            .query_row(
                "SELECT COUNT(*) FROM requests WHERE project_id = ?1",
                params![project.id],
                |row| row.get::<_, i64>(0),
            )
            .unwrap();
        assert_eq!(remaining, 0);
    }
}

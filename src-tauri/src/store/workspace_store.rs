use chrono::Utc;
use rusqlite::{params, Connection};
use uuid::Uuid;

use crate::error::AppError;
use crate::models::{NewWorkspaceInput, UpdateWorkspaceInput, Workspace};

pub fn create_workspace(conn: &Connection, input: NewWorkspaceInput) -> Result<Workspace, AppError> {
    let name = input.name.trim();
    if name.is_empty() {
        return Err(AppError::Validation("workspace name must not be empty".into()));
    }

    let workspace = Workspace {
        id: Uuid::new_v4().to_string(),
        name: name.to_string(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    conn.execute(
        "INSERT INTO workspaces (id, name, created_at, updated_at) VALUES (?1, ?2, ?3, ?4)",
        params![
            workspace.id,
            workspace.name,
            workspace.created_at.to_rfc3339(),
            workspace.updated_at.to_rfc3339()
        ],
    )?;

    Ok(workspace)
}

pub fn list_workspaces(conn: &Connection) -> Result<Vec<Workspace>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT id, name, created_at, updated_at FROM workspaces ORDER BY name ASC",
    )?;
    let rows = stmt.query_map([], row_to_workspace)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(AppError::from)
}

pub fn get_workspace(conn: &Connection, id: &str) -> Result<Workspace, AppError> {
    conn.query_row(
        "SELECT id, name, created_at, updated_at FROM workspaces WHERE id = ?1",
        params![id],
        row_to_workspace,
    )
    .map_err(|err| match err {
        rusqlite::Error::QueryReturnedNoRows => {
            AppError::NotFound(format!("workspace {id} not found"))
        }
        other => AppError::from(other),
    })
}

/// Only overwrites fields that were actually provided, same convention as `project_store::update_project`.
pub fn update_workspace(conn: &Connection, input: UpdateWorkspaceInput) -> Result<Workspace, AppError> {
    let existing = get_workspace(conn, &input.id)?;

    let name = match input.name {
        Some(name) => {
            let trimmed = name.trim();
            if trimmed.is_empty() {
                return Err(AppError::Validation("workspace name must not be empty".into()));
            }
            trimmed.to_string()
        }
        None => existing.name,
    };

    let updated_at = Utc::now();

    conn.execute(
        "UPDATE workspaces SET name = ?1, updated_at = ?2 WHERE id = ?3",
        params![name, updated_at.to_rfc3339(), existing.id],
    )?;

    Ok(Workspace {
        id: existing.id,
        name,
        created_at: existing.created_at,
        updated_at,
    })
}

/// Deletes the workspace and every project inside it — `project_store::delete_project` already
/// cascades requests/environments/folders/etc. per project, so this just loops that over every
/// project_id in scope (no DB-level cascade on the ALTER-added `workspace_id` column, see db.rs
/// migration 15). Refuses to delete the last remaining workspace: the app always needs one.
pub fn delete_workspace(conn: &Connection, id: &str) -> Result<(), AppError> {
    let total: i64 = conn.query_row("SELECT COUNT(*) FROM workspaces", [], |row| row.get(0))?;
    if total <= 1 {
        return Err(AppError::Validation(
            "cannot delete the last workspace".into(),
        ));
    }

    let mut stmt = conn.prepare("SELECT id FROM projects WHERE workspace_id = ?1")?;
    let project_ids: Vec<String> = stmt
        .query_map(params![id], |row| row.get(0))?
        .collect::<Result<Vec<_>, _>>()?;
    drop(stmt);
    for project_id in project_ids {
        crate::store::project_store::delete_project(conn, &project_id)?;
    }

    let affected = conn.execute("DELETE FROM workspaces WHERE id = ?1", params![id])?;
    if affected == 0 {
        return Err(AppError::NotFound(format!("workspace {id} not found")));
    }
    Ok(())
}

fn row_to_workspace(row: &rusqlite::Row) -> rusqlite::Result<Workspace> {
    let created_at: String = row.get(2)?;
    let updated_at: String = row.get(3)?;
    Ok(Workspace {
        id: row.get(0)?,
        name: row.get(1)?,
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

    #[test]
    fn default_workspace_exists_after_migrations() {
        let conn = db::open_in_memory().unwrap();
        let workspaces = list_workspaces(&conn).unwrap();
        assert_eq!(workspaces.len(), 1);
        assert_eq!(workspaces[0].id, "default");
    }

    #[test]
    fn create_then_list_then_get_round_trips() {
        let conn = db::open_in_memory().unwrap();
        let created = create_workspace(&conn, NewWorkspaceInput { name: "Team A".into() }).unwrap();

        let listed = list_workspaces(&conn).unwrap();
        assert_eq!(listed.len(), 2);

        let fetched = get_workspace(&conn, &created.id).unwrap();
        assert_eq!(fetched.name, "Team A");
    }

    #[test]
    fn create_rejects_empty_name() {
        let conn = db::open_in_memory().unwrap();
        let result = create_workspace(&conn, NewWorkspaceInput { name: "   ".into() });
        assert!(matches!(result, Err(AppError::Validation(_))));
    }

    #[test]
    fn delete_refuses_the_last_workspace() {
        let conn = db::open_in_memory().unwrap();
        let result = delete_workspace(&conn, "default");
        assert!(matches!(result, Err(AppError::Validation(_))));
    }

    #[test]
    fn delete_cascades_its_projects() {
        let conn = db::open_in_memory().unwrap();
        let ws = create_workspace(&conn, NewWorkspaceInput { name: "Team A".into() }).unwrap();
        let project = project_store::create_project(
            &conn,
            NewProjectInput { name: "Doomed".into(), workspace_id: ws.id.clone() },
        )
        .unwrap();

        delete_workspace(&conn, &ws.id).unwrap();

        assert!(matches!(
            project_store::get_project(&conn, &project.id),
            Err(AppError::NotFound(_))
        ));
        assert!(matches!(get_workspace(&conn, &ws.id), Err(AppError::NotFound(_))));
    }
}

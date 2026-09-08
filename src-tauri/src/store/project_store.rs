use chrono::Utc;
use rusqlite::{params, Connection};
use uuid::Uuid;

use crate::error::AppError;
use crate::models::{NewProjectInput, Project};

pub fn create_project(conn: &Connection, input: NewProjectInput) -> Result<Project, AppError> {
    let name = input.name.trim();
    if name.is_empty() {
        return Err(AppError::Validation("project name must not be empty".into()));
    }

    let project = Project {
        id: Uuid::new_v4().to_string(),
        name: name.to_string(),
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
        "SELECT id, name, created_at, updated_at FROM projects ORDER BY updated_at DESC",
    )?;
    let rows = stmt.query_map([], row_to_project)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(AppError::from)
}

pub fn get_project(conn: &Connection, id: &str) -> Result<Project, AppError> {
    conn.query_row(
        "SELECT id, name, created_at, updated_at FROM projects WHERE id = ?1",
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

fn row_to_project(row: &rusqlite::Row) -> rusqlite::Result<Project> {
    let created_at: String = row.get(2)?;
    let updated_at: String = row.get(3)?;
    Ok(Project {
        id: row.get(0)?,
        name: row.get(1)?,
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
}

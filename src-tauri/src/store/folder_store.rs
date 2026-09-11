use chrono::Utc;
use rusqlite::{params, Connection};
use uuid::Uuid;

use crate::error::AppError;
use crate::models::{Folder, NewFolderInput, UpdateFolderInput};

pub fn create_folder(conn: &Connection, input: NewFolderInput) -> Result<Folder, AppError> {
    let name = input.name.trim();
    if name.is_empty() {
        return Err(AppError::Validation("folder name must not be empty".into()));
    }

    if let Some(parent_id) = &input.parent_folder_id {
        let parent = get_folder(conn, parent_id)?;
        if parent.project_id != input.project_id {
            return Err(AppError::Validation(
                "parent folder must belong to the same project".into(),
            ));
        }
    }

    let folder = Folder {
        id: Uuid::new_v4().to_string(),
        project_id: input.project_id,
        name: name.to_string(),
        parent_folder_id: input.parent_folder_id,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    conn.execute(
        "INSERT INTO folders (id, project_id, name, parent_folder_id, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            folder.id,
            folder.project_id,
            folder.name,
            folder.parent_folder_id,
            folder.created_at.to_rfc3339(),
            folder.updated_at.to_rfc3339()
        ],
    )
    .map_err(map_constraint_error)?;

    Ok(folder)
}

/// Returns every folder in the project, flat — at any depth. The frontend groups them into a
/// tree by `parent_folder_id`.
pub fn list_folders(conn: &Connection, project_id: &str) -> Result<Vec<Folder>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT id, project_id, name, parent_folder_id, created_at, updated_at FROM folders
         WHERE project_id = ?1 ORDER BY name ASC",
    )?;
    let rows = stmt.query_map(params![project_id], row_to_folder)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(AppError::from)
}

pub fn get_folder(conn: &Connection, id: &str) -> Result<Folder, AppError> {
    conn.query_row(
        "SELECT id, project_id, name, parent_folder_id, created_at, updated_at FROM folders WHERE id = ?1",
        params![id],
        row_to_folder,
    )
    .map_err(|err| match err {
        rusqlite::Error::QueryReturnedNoRows => AppError::NotFound(format!("folder {id} not found")),
        other => AppError::from(other),
    })
}

pub fn update_folder(conn: &Connection, input: UpdateFolderInput) -> Result<Folder, AppError> {
    let existing = get_folder(conn, &input.id)?;

    let name = match input.name {
        Some(name) => {
            let trimmed = name.trim();
            if trimmed.is_empty() {
                return Err(AppError::Validation("folder name must not be empty".into()));
            }
            trimmed.to_string()
        }
        None => existing.name,
    };

    let updated_at = Utc::now();
    let tx = conn.unchecked_transaction()?;
    tx.execute(
        "UPDATE folders SET name = ?1, updated_at = ?2 WHERE id = ?3",
        params![name, updated_at.to_rfc3339(), existing.id],
    )
    .map_err(map_constraint_error)?;
    tx.commit()?;

    Ok(Folder {
        id: existing.id,
        project_id: existing.project_id,
        name,
        parent_folder_id: existing.parent_folder_id,
        created_at: existing.created_at,
        updated_at,
    })
}

/// Deleting a folder ungroups its requests back to the project root, and promotes any child
/// folders up to the deleted folder's own parent — a folder is just an organizational grouping,
/// not a container whose contents are meaningless without it (unlike a project, whose requests
/// really do cascade-delete).
pub fn delete_folder(conn: &Connection, id: &str) -> Result<(), AppError> {
    let existing = get_folder(conn, id)?;

    let tx = conn.unchecked_transaction()?;
    tx.execute("UPDATE requests SET folder_id = NULL WHERE folder_id = ?1", params![id])?;
    tx.execute(
        "UPDATE folders SET parent_folder_id = ?1 WHERE parent_folder_id = ?2",
        params![existing.parent_folder_id, id],
    )?;
    let affected = tx.execute("DELETE FROM folders WHERE id = ?1", params![id])?;
    if affected == 0 {
        return Err(AppError::NotFound(format!("folder {id} not found")));
    }
    tx.commit()?;
    Ok(())
}

fn map_constraint_error(err: rusqlite::Error) -> AppError {
    match err {
        rusqlite::Error::SqliteFailure(e, _) if e.code == rusqlite::ErrorCode::ConstraintViolation => {
            AppError::Validation("the project does not exist".into())
        }
        other => AppError::from(other),
    }
}

fn row_to_folder(row: &rusqlite::Row) -> rusqlite::Result<Folder> {
    let created_at: String = row.get(4)?;
    let updated_at: String = row.get(5)?;
    Ok(Folder {
        id: row.get(0)?,
        project_id: row.get(1)?,
        name: row.get(2)?,
        parent_folder_id: row.get(3)?,
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
        project_store::create_project(conn, NewProjectInput { name: "Demo".into(), workspace_id: "default".into() }).unwrap().id
    }

    #[test]
    fn create_rejects_blank_name() {
        let conn = db::open_in_memory().unwrap();
        let project_id = seed_project(&conn);
        let result = create_folder(&conn, NewFolderInput { project_id, name: "  ".into(), ..Default::default() });
        assert!(matches!(result, Err(AppError::Validation(_))));
    }

    #[test]
    fn create_rejects_missing_project() {
        let conn = db::open_in_memory().unwrap();
        let result = create_folder(&conn, NewFolderInput { project_id: "ghost".into(), name: "Auth".into(), ..Default::default() });
        assert!(matches!(result, Err(AppError::Validation(_))));
    }

    #[test]
    fn list_orders_by_name_and_get_round_trips() {
        let conn = db::open_in_memory().unwrap();
        let project_id = seed_project(&conn);
        create_folder(&conn, NewFolderInput { project_id: project_id.clone(), name: "Zeta".into(), ..Default::default() }).unwrap();
        let alpha = create_folder(&conn, NewFolderInput { project_id, name: "Alpha".into(), ..Default::default() }).unwrap();

        let listed = list_folders(&conn, &alpha.project_id).unwrap();
        assert_eq!(listed.len(), 2);
        assert_eq!(listed[0].name, "Alpha");

        let fetched = get_folder(&conn, &alpha.id).unwrap();
        assert_eq!(fetched.name, "Alpha");
    }

    #[test]
    fn update_renames_folder() {
        let conn = db::open_in_memory().unwrap();
        let project_id = seed_project(&conn);
        let folder = create_folder(&conn, NewFolderInput { project_id, name: "Auth".into(), ..Default::default() }).unwrap();

        let renamed = update_folder(&conn, UpdateFolderInput { id: folder.id, name: Some("Authentication".into()) }).unwrap();
        assert_eq!(renamed.name, "Authentication");
    }

    #[test]
    fn delete_missing_folder_returns_not_found() {
        let conn = db::open_in_memory().unwrap();
        let result = delete_folder(&conn, "ghost");
        assert!(matches!(result, Err(AppError::NotFound(_))));
    }

    #[test]
    fn deleting_a_folder_ungroups_its_requests_instead_of_deleting_them() {
        let conn = db::open_in_memory().unwrap();
        let project_id = seed_project(&conn);
        let folder = create_folder(&conn, NewFolderInput { project_id: project_id.clone(), name: "Auth".into(), ..Default::default() }).unwrap();
        let request = request_store::create_request(
            &conn,
            NewRequestInput {
                project_id,
                folder_id: Some(folder.id.clone()),
                name: "Login".into(),
                method: "POST".into(),
                url: "https://api.example.com/login".into(),
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(request.folder_id, Some(folder.id.clone()));

        delete_folder(&conn, &folder.id).unwrap();

        let refetched = request_store::get_request(&conn, &request.id).unwrap();
        assert_eq!(refetched.folder_id, None, "request must survive its folder's deletion, ungrouped");
    }

    #[test]
    fn create_nests_a_folder_under_its_parent() {
        let conn = db::open_in_memory().unwrap();
        let project_id = seed_project(&conn);
        let parent = create_folder(&conn, NewFolderInput { project_id: project_id.clone(), name: "Auth".into(), ..Default::default() }).unwrap();
        let child = create_folder(
            &conn,
            NewFolderInput { project_id, name: "Tokens".into(), parent_folder_id: Some(parent.id.clone()) },
        )
        .unwrap();

        assert_eq!(child.parent_folder_id, Some(parent.id));
    }

    #[test]
    fn create_rejects_parent_folder_from_a_different_project() {
        let conn = db::open_in_memory().unwrap();
        let project_a = seed_project(&conn);
        let project_b = seed_project(&conn);
        let parent = create_folder(&conn, NewFolderInput { project_id: project_a, name: "Auth".into(), ..Default::default() }).unwrap();

        let result = create_folder(
            &conn,
            NewFolderInput { project_id: project_b, name: "Tokens".into(), parent_folder_id: Some(parent.id) },
        );
        assert!(matches!(result, Err(AppError::Validation(_))));
    }

    #[test]
    fn deleting_a_folder_promotes_its_children_to_its_own_parent() {
        let conn = db::open_in_memory().unwrap();
        let project_id = seed_project(&conn);
        let grandparent = create_folder(&conn, NewFolderInput { project_id: project_id.clone(), name: "Auth".into(), ..Default::default() }).unwrap();
        let parent = create_folder(
            &conn,
            NewFolderInput { project_id: project_id.clone(), name: "Tokens".into(), parent_folder_id: Some(grandparent.id.clone()) },
        )
        .unwrap();
        let child = create_folder(
            &conn,
            NewFolderInput { project_id, name: "Refresh".into(), parent_folder_id: Some(parent.id.clone()) },
        )
        .unwrap();

        delete_folder(&conn, &parent.id).unwrap();

        let refetched = get_folder(&conn, &child.id).unwrap();
        assert_eq!(
            refetched.parent_folder_id,
            Some(grandparent.id),
            "child folder must be promoted to its deleted parent's own parent"
        );
    }
}

use std::path::Path;

use rusqlite::Connection;

use crate::error::AppError;

/// Ordered schema migrations. Never edit an already-shipped entry — append a new one (README §35).
const MIGRATIONS: &[(i64, &str)] = &[
    (
        1,
        "CREATE TABLE projects (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );",
    ),
    (
        2,
        "CREATE TABLE requests (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
            name TEXT NOT NULL,
            method TEXT NOT NULL,
            url TEXT NOT NULL,
            headers TEXT NOT NULL DEFAULT '[]',
            body TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );
        CREATE INDEX idx_requests_project_id ON requests(project_id);",
    ),
    (
        3,
        "CREATE TABLE environments (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
            name TEXT NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            UNIQUE(project_id, name)
        );
        CREATE INDEX idx_environments_project_id ON environments(project_id);

        -- scope is one of: global | environment | collection | folder | request.
        -- collection_id/folder_id are reserved (no backing table yet) so adding those
        -- features later needs a new table + FK, not a variables schema rewrite.
        CREATE TABLE variables (
            id TEXT PRIMARY KEY,
            scope TEXT NOT NULL CHECK(scope IN ('global','environment','collection','folder','request')),
            project_id TEXT REFERENCES projects(id) ON DELETE CASCADE,
            environment_id TEXT REFERENCES environments(id) ON DELETE CASCADE,
            request_id TEXT REFERENCES requests(id) ON DELETE CASCADE,
            collection_id TEXT,
            folder_id TEXT,
            key TEXT NOT NULL,
            value TEXT NOT NULL DEFAULT '',
            enabled INTEGER NOT NULL DEFAULT 1,
            is_secret INTEGER NOT NULL DEFAULT 0,
            description TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );
        CREATE INDEX idx_variables_project_id ON variables(project_id);
        CREATE INDEX idx_variables_environment_id ON variables(environment_id);
        CREATE INDEX idx_variables_request_id ON variables(request_id);",
    ),
    (
        4,
        "-- Metadata is always cheap to list; the body never is, so it's split out
        -- (README §19/§23). body_inline holds small bodies as a BLOB (binary-safe);
        -- body_path holds a path under the app data dir for anything over the cap.
        CREATE TABLE responses (
            id TEXT PRIMARY KEY,
            request_id TEXT NOT NULL REFERENCES requests(id) ON DELETE CASCADE,
            status INTEGER NOT NULL,
            status_text TEXT NOT NULL,
            headers TEXT NOT NULL DEFAULT '[]',
            duration_ms INTEGER NOT NULL,
            body_size INTEGER NOT NULL,
            body_storage TEXT NOT NULL CHECK(body_storage IN ('inline','disk')),
            body_inline BLOB,
            body_path TEXT,
            created_at TEXT NOT NULL
        );
        CREATE INDEX idx_responses_request_id ON responses(request_id);",
    ),
    (
        5,
        "ALTER TABLE requests ADD COLUMN query_params TEXT NOT NULL DEFAULT '[]';",
    ),
    (
        6,
        r#"ALTER TABLE requests ADD COLUMN auth TEXT NOT NULL DEFAULT '{"type":"none"}';"#,
    ),
    (
        7,
        "ALTER TABLE requests ADD COLUMN description TEXT;",
    ),
    (
        8,
        "ALTER TABLE requests ADD COLUMN settings TEXT;
        ALTER TABLE requests ADD COLUMN pre_request_script TEXT;
        ALTER TABLE requests ADD COLUMN post_request_script TEXT;
        CREATE TABLE sample_responses (
            id TEXT PRIMARY KEY,
            request_id TEXT NOT NULL REFERENCES requests(id) ON DELETE CASCADE,
            name TEXT NOT NULL,
            status INTEGER NOT NULL,
            status_text TEXT NOT NULL,
            headers TEXT NOT NULL DEFAULT '[]',
            body TEXT,
            content_type TEXT,
            created_at TEXT NOT NULL
        );
        CREATE INDEX idx_sample_responses_request_id ON sample_responses(request_id);
        CREATE TABLE cookies (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
            domain TEXT NOT NULL,
            path TEXT NOT NULL DEFAULT '/',
            name TEXT NOT NULL,
            value TEXT NOT NULL,
            expires TEXT,
            secure INTEGER NOT NULL DEFAULT 0,
            http_only INTEGER NOT NULL DEFAULT 0,
            same_site TEXT,
            created_at TEXT NOT NULL
        );
        CREATE INDEX idx_cookies_project_id ON cookies(project_id);",
    ),
    (
        9,
        "ALTER TABLE variables ADD COLUMN is_local INTEGER NOT NULL DEFAULT 0;",
    ),
    (
        10,
        "CREATE TABLE project_git_settings (
            project_id TEXT PRIMARY KEY REFERENCES projects(id) ON DELETE CASCADE,
            repo_path TEXT,
            remote_url TEXT,
            branch TEXT NOT NULL DEFAULT 'main',
            auto_sync INTEGER NOT NULL DEFAULT 0,
            github_token TEXT,
            last_sync_at TEXT
        );
        CREATE TABLE sync_queue (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
            operation TEXT NOT NULL,
            payload TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'pending',
            created_at TEXT NOT NULL
        );
        CREATE INDEX idx_sync_queue_project_id ON sync_queue(project_id);",
    ),
    (
        11,
        "CREATE TABLE app_settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );
        CREATE TABLE project_source_associations (
            project_id TEXT PRIMARY KEY REFERENCES projects(id) ON DELETE CASCADE,
            source_directory TEXT NOT NULL,
            framework TEXT,
            detected_at TEXT NOT NULL
        );",
    ),
    (
        12,
        // No DB-level FK action here (ALTER-added foreign keys have inconsistent ON DELETE
        // support across SQLite versions) — delete_environment clears this in application code
        // instead (see environment_store::delete_environment).
        "ALTER TABLE projects ADD COLUMN default_environment_id TEXT;",
    ),
];

pub fn open(path: &Path) -> Result<Connection, AppError> {
    let conn = Connection::open(path)?;
    configure(&conn)?;
    run_migrations(&conn)?;
    Ok(conn)
}

#[cfg(test)]
pub fn open_in_memory() -> Result<Connection, AppError> {
    let conn = Connection::open_in_memory()?;
    configure(&conn)?;
    run_migrations(&conn)?;
    Ok(conn)
}

fn configure(conn: &Connection) -> Result<(), AppError> {
    conn.execute_batch(
        "PRAGMA journal_mode = WAL;
         PRAGMA foreign_keys = ON;
         PRAGMA synchronous = NORMAL;",
    )?;
    Ok(())
}

pub fn run_migrations(conn: &Connection) -> Result<(), AppError> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            version INTEGER PRIMARY KEY,
            applied_at TEXT NOT NULL
        );",
        [],
    )?;

    let current_version: i64 = conn.query_row(
        "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
        [],
        |row| row.get(0),
    )?;

    for (version, sql) in MIGRATIONS {
        if *version <= current_version {
            continue;
        }
        let tx = conn.unchecked_transaction()?;
        tx.execute_batch(sql)?;
        tx.execute(
            "INSERT INTO schema_migrations (version, applied_at) VALUES (?1, ?2)",
            rusqlite::params![version, chrono::Utc::now().to_rfc3339()],
        )?;
        tx.commit()?;
        log::info!("applied migration {version}");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrations_apply_cleanly_and_are_idempotent() {
        let conn = open_in_memory().expect("open in-memory db");

        // Re-running should be a no-op, not an error.
        run_migrations(&conn).expect("second migration run must be idempotent");

        let version: i64 = conn
            .query_row("SELECT MAX(version) FROM schema_migrations", [], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(version, MIGRATIONS.last().unwrap().0);
    }
}

# Custom Order (import order + drag-and-drop) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Persist a manual order for projects, folders, and requests, make a fresh Postman collection import land in that order automatically, and let users drag-and-drop to set their own order.

**Architecture:** Add a `sort_order` column (backfilled, auto-assigned on create) to `projects`/`folders`/`requests`. Three new Tauri commands rewrite `sort_order` for a full sibling group at once, driven by native HTML5 drag-and-drop in the sidebar. A new "Custom order" sort choice (alongside existing Name/Created/Updated) is what makes `sort_order` visible and draggable; nothing changes for a user who never touches it.

**Tech Stack:** Rust (rusqlite/SQLite, Tauri commands), Svelte 5 runes, TypeScript.

**Spec:** `docs/superpowers/specs/2026-09-15-custom-order-design.md`

## Global Constraints

- Never edit a shipped migration in `src-tauri/src/db.rs` — append a new numbered entry (README §35). The next one is `17`.
- `create_project`/`create_folder`/`create_request` always auto-assign `sort_order`; no caller-supplied override (see spec's Importer section for why this is sufficient).
- Existing `ORDER BY` clauses in `list_projects`/`list_folders`/`list_requests` stay as they are — the frontend already re-sorts everything client-side, so backend list order is untouched by this feature.
- "Custom order" is a new, non-default sort choice. Default sort stays "Name" for every existing project/list; nothing changes for a user who doesn't pick it.
- Drag-and-drop is only enabled when the relevant sort field is already "custom" (dragging under another sort would be immediately undone by the next re-sort).

---

## Task 1: Migration 17 — add and backfill `sort_order`

**Files:**
- Modify: `src-tauri/src/db.rs` (append to the `MIGRATIONS` array, after entry `16`)

**Interfaces:**
- Produces: every row in `projects`, `folders`, `requests` has a `sort_order INTEGER NOT NULL DEFAULT 0` column, backfilled so pre-existing rows get a stable per-sibling-group sequential order matching creation order.

- [ ] **Step 1: Write the migration**

Add this entry to `MIGRATIONS` in `src-tauri/src/db.rs`, right after entry `16`:

```rust
    (
        17,
        // Persisted manual order for the sidebar tree (projects, folders, requests) — see
        // docs/superpowers/specs/2026-09-15-custom-order-design.md. Backfilled with a
        // sequential rank per sibling group (by created_at, id as tiebreak) rather than left
        // at the column default, so upgrading users get a stable starting order instead of
        // every row reading 0. Auto-assigned going forward by create_project/create_folder/
        // create_request (COALESCE(MAX(sort_order), -1) + 1 among siblings) — this table
        // itself never enforces sequential/gap-free values, callers do.
        "ALTER TABLE projects ADD COLUMN sort_order INTEGER NOT NULL DEFAULT 0;
        UPDATE projects SET sort_order = (
            SELECT COUNT(*) FROM projects p2
            WHERE p2.workspace_id = projects.workspace_id
              AND (p2.created_at < projects.created_at
                   OR (p2.created_at = projects.created_at AND p2.id < projects.id))
        );
        ALTER TABLE folders ADD COLUMN sort_order INTEGER NOT NULL DEFAULT 0;
        UPDATE folders SET sort_order = (
            SELECT COUNT(*) FROM folders f2
            WHERE f2.project_id = folders.project_id
              AND (f2.parent_folder_id IS folders.parent_folder_id)
              AND (f2.created_at < folders.created_at
                   OR (f2.created_at = folders.created_at AND f2.id < folders.id))
        );
        ALTER TABLE requests ADD COLUMN sort_order INTEGER NOT NULL DEFAULT 0;
        UPDATE requests SET sort_order = (
            SELECT COUNT(*) FROM requests r2
            WHERE r2.project_id = requests.project_id
              AND (r2.folder_id IS requests.folder_id)
              AND (r2.created_at < requests.created_at
                   OR (r2.created_at = requests.created_at AND r2.id < requests.id))
        );",
    ),
```

- [ ] **Step 2: Add a backfill-order test**

Add to the `#[cfg(test)] mod tests` block at the bottom of `src-tauri/src/db.rs`:

```rust
    #[test]
    fn migration_17_backfills_sequential_sort_order_per_sibling_group() {
        let conn = open_in_memory().expect("open in-memory db");

        // Two requests directly under the project root, created in a known order.
        conn.execute(
            "INSERT INTO projects (id, name, workspace_id, created_at, updated_at)
             VALUES ('p1', 'Demo', 'default', '2024-01-01T00:00:00Z', '2024-01-01T00:00:00Z')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO requests (id, project_id, folder_id, name, method, url, created_at, updated_at)
             VALUES ('r1', 'p1', NULL, 'First', 'GET', 'https://a', '2024-01-01T00:00:01Z', '2024-01-01T00:00:01Z')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO requests (id, project_id, folder_id, name, method, url, created_at, updated_at)
             VALUES ('r2', 'p1', NULL, 'Second', 'GET', 'https://b', '2024-01-01T00:00:02Z', '2024-01-01T00:00:02Z')",
            [],
        )
        .unwrap();

        // These inserts ran against a DB already migrated to 17 by open_in_memory() (via
        // run_migrations), so sort_order already exists — re-derive it by hand to prove the
        // backfill formula, since inserting after the fact means the column defaults to 0 for
        // both. Run the exact backfill UPDATE again and check it separates them correctly.
        conn.execute_batch(
            "UPDATE requests SET sort_order = (
                SELECT COUNT(*) FROM requests r2
                WHERE r2.project_id = requests.project_id
                  AND (r2.folder_id IS requests.folder_id)
                  AND (r2.created_at < requests.created_at
                       OR (r2.created_at = requests.created_at AND r2.id < requests.id))
            );",
        )
        .unwrap();

        let order_r1: i64 = conn
            .query_row("SELECT sort_order FROM requests WHERE id = 'r1'", [], |r| r.get(0))
            .unwrap();
        let order_r2: i64 = conn
            .query_row("SELECT sort_order FROM requests WHERE id = 'r2'", [], |r| r.get(0))
            .unwrap();
        assert_eq!(order_r1, 0, "created first, so ranks first");
        assert_eq!(order_r2, 1, "created second, so ranks second");
    }
```

- [ ] **Step 3: Run the tests**

Run: `cargo test --lib db:: --manifest-path src-tauri/Cargo.toml`
Expected: both `migrations_apply_cleanly_and_are_idempotent` and the new test PASS.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/db.rs
git commit -m "feat(db): add sort_order column to projects/folders/requests (migration 17)"
```

---

## Task 2: `sort_order` on projects — struct, auto-assign, reorder

**Files:**
- Modify: `src-tauri/src/models.rs` (`Project` struct)
- Modify: `src-tauri/src/store/project_store.rs` (`create_project`, `list_projects`, `get_project`, `row_to_project`; add `reorder_projects`)
- Modify: `src-tauri/src/commands.rs` (add `reorder_projects` command)
- Modify: `src-tauri/src/lib.rs` (register `commands::reorder_projects`)

**Interfaces:**
- Produces: `Project.sort_order: i64` on every returned project. `project_store::reorder_projects(conn: &Connection, workspace_id: &str, ordered_ids: &[String]) -> Result<(), AppError>`. Tauri command `reorder_projects(state, workspace_id: String, ordered_ids: Vec<String>) -> Result<(), AppError>`.

- [ ] **Step 1: Add `sort_order` to the `Project` struct**

In `src-tauri/src/models.rs`, in `pub struct Project { ... }` (currently ending `pub updated_at: DateTime<Utc>,`), add:

```rust
    /// Manual order within the workspace — see docs/superpowers/specs/2026-09-15-custom-order-design.md.
    /// Only meaningful when the sidebar's project sort is set to "Custom"; every other sort
    /// mode ignores it. Auto-assigned on create; rewritten wholesale by `reorder_projects`.
    pub sort_order: i64,
```

- [ ] **Step 2: Auto-assign `sort_order` in `create_project`, and select it everywhere**

In `src-tauri/src/store/project_store.rs`:

Replace the body of `create_project` (currently builds `Project { id, name, default_environment_id: None, workspace_id, created_at, updated_at }` then does a 5-column `INSERT`) with:

```rust
pub fn create_project(conn: &Connection, input: NewProjectInput) -> Result<Project, AppError> {
    let name = input.name.trim();
    if name.is_empty() {
        return Err(AppError::Validation("project name must not be empty".into()));
    }
    let workspace_id = input.workspace_id.trim();
    if workspace_id.is_empty() {
        return Err(AppError::Validation("workspace_id must not be empty".into()));
    }

    let next_order: i64 = conn.query_row(
        "SELECT COALESCE(MAX(sort_order), -1) + 1 FROM projects WHERE workspace_id = ?1",
        params![workspace_id],
        |row| row.get(0),
    )?;

    let project = Project {
        id: Uuid::new_v4().to_string(),
        name: name.to_string(),
        default_environment_id: None,
        workspace_id: workspace_id.to_string(),
        sort_order: next_order,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    conn.execute(
        "INSERT INTO projects (id, name, workspace_id, sort_order, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            project.id,
            project.name,
            project.workspace_id,
            project.sort_order,
            project.created_at.to_rfc3339(),
            project.updated_at.to_rfc3339()
        ],
    )?;

    Ok(project)
}
```

Update `list_projects` and `get_project`'s `SELECT` to include `sort_order`:

```rust
pub fn list_projects(conn: &Connection, workspace_id: &str) -> Result<Vec<Project>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT id, name, default_environment_id, workspace_id, sort_order, created_at, updated_at
         FROM projects WHERE workspace_id = ?1 ORDER BY updated_at DESC",
    )?;
    let rows = stmt.query_map(params![workspace_id], row_to_project)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(AppError::from)
}
```

```rust
pub fn get_project(conn: &Connection, id: &str) -> Result<Project, AppError> {
    conn.query_row(
        "SELECT id, name, default_environment_id, workspace_id, sort_order, created_at, updated_at
         FROM projects WHERE id = ?1",
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
```

And `row_to_project` (column indices shift because `sort_order` is inserted before `created_at`):

```rust
fn row_to_project(row: &rusqlite::Row) -> rusqlite::Result<Project> {
    let created_at: String = row.get(5)?;
    let updated_at: String = row.get(6)?;
    Ok(Project {
        id: row.get(0)?,
        name: row.get(1)?,
        default_environment_id: row.get(2)?,
        workspace_id: row.get(3)?,
        sort_order: row.get(4)?,
        created_at: created_at
            .parse()
            .unwrap_or_else(|_| Utc::now()),
        updated_at: updated_at
            .parse()
            .unwrap_or_else(|_| Utc::now()),
    })
}
```

`update_project` in the same file builds a `Project { ... }` literal by hand from `existing` (the `get_project` result) plus whatever fields the update actually changed:

```rust
    Ok(Project {
        id: existing.id,
        name,
        default_environment_id,
        workspace_id: existing.workspace_id,
        sort_order: existing.sort_order,
        created_at: existing.created_at,
        updated_at,
    })
```

Add the `sort_order: existing.sort_order,` line to that literal — updates never touch order, only `reorder_projects` does.

- [ ] **Step 3: Add `reorder_projects`**

Append to `src-tauri/src/store/project_store.rs`:

```rust
/// Rewrites `sort_order` for every id in `ordered_ids` to its index in that list (0, 1, 2, ...)
/// — the caller (the sidebar's drag-and-drop) always sends the *complete*, newly-ordered
/// sibling list, not a single moved id, so this never has to reason about insertion points.
/// All-or-nothing: if any id doesn't belong to `workspace_id`, nothing is written.
pub fn reorder_projects(
    conn: &Connection,
    workspace_id: &str,
    ordered_ids: &[String],
) -> Result<(), AppError> {
    let tx = conn.unchecked_transaction()?;
    for (idx, id) in ordered_ids.iter().enumerate() {
        let affected = tx.execute(
            "UPDATE projects SET sort_order = ?1 WHERE id = ?2 AND workspace_id = ?3",
            params![idx as i64, id, workspace_id],
        )?;
        if affected == 0 {
            return Err(AppError::Validation(format!(
                "project {id} not found in workspace {workspace_id}"
            )));
        }
    }
    tx.commit()?;
    Ok(())
}
```

- [ ] **Step 4: Write the store test**

Add to the `#[cfg(test)] mod tests` block in `src-tauri/src/store/project_store.rs` (check the existing tests in that file for the exact helper names/imports already in scope — reuse them rather than re-declaring):

```rust
    #[test]
    fn create_auto_assigns_sequential_sort_order_and_reorder_rewrites_it() {
        let conn = db::open_in_memory().unwrap();
        let a = create_project(&conn, NewProjectInput { name: "A".into(), workspace_id: "default".into() }).unwrap();
        let b = create_project(&conn, NewProjectInput { name: "B".into(), workspace_id: "default".into() }).unwrap();
        let c = create_project(&conn, NewProjectInput { name: "C".into(), workspace_id: "default".into() }).unwrap();
        assert_eq!((a.sort_order, b.sort_order, c.sort_order), (0, 1, 2));

        // Move C to the front.
        reorder_projects(&conn, "default", &[c.id.clone(), a.id.clone(), b.id.clone()]).unwrap();
        assert_eq!(get_project(&conn, &c.id).unwrap().sort_order, 0);
        assert_eq!(get_project(&conn, &a.id).unwrap().sort_order, 1);
        assert_eq!(get_project(&conn, &b.id).unwrap().sort_order, 2);
    }

    #[test]
    fn reorder_rejects_an_id_outside_the_workspace_and_writes_nothing() {
        let conn = db::open_in_memory().unwrap();
        let a = create_project(&conn, NewProjectInput { name: "A".into(), workspace_id: "default".into() }).unwrap();
        let other_ws = crate::store::workspace_store::create_workspace(
            &conn,
            crate::models::NewWorkspaceInput { name: "Other".into() },
        )
        .unwrap();
        let foreign = create_project(&conn, NewProjectInput { name: "Foreign".into(), workspace_id: other_ws.id }).unwrap();

        let result = reorder_projects(&conn, "default", &[a.id.clone(), foreign.id]);
        assert!(matches!(result, Err(AppError::Validation(_))));
        assert_eq!(get_project(&conn, &a.id).unwrap().sort_order, 0, "unchanged — the batch failed before touching it");
    }
```

- [ ] **Step 5: Add the Tauri command**

In `src-tauri/src/commands.rs`, near the existing `list_projects` command:

```rust
#[tauri::command]
pub fn reorder_projects(
    state: State<AppState>,
    workspace_id: String,
    ordered_ids: Vec<String>,
) -> Result<(), AppError> {
    let conn = state.db.lock().expect("db mutex poisoned");
    project_store::reorder_projects(&conn, &workspace_id, &ordered_ids)
}
```

- [ ] **Step 6: Register the command**

In `src-tauri/src/lib.rs`, add `commands::reorder_projects,` right after `commands::list_projects,` in the `invoke_handler` list.

- [ ] **Step 7: Run the tests**

Run: `cargo test --lib store::project_store:: --manifest-path src-tauri/Cargo.toml`
Expected: all PASS, including the two new tests.

Run: `cargo check --manifest-path src-tauri/Cargo.toml`
Expected: no errors (catches any other place in the codebase that builds a `Project` literal and is now missing `sort_order`).

- [ ] **Step 8: Commit**

```bash
git add src-tauri/src/models.rs src-tauri/src/store/project_store.rs src-tauri/src/commands.rs src-tauri/src/lib.rs
git commit -m "feat(projects): persist and reorder project sort_order"
```

---

## Task 3: `sort_order` on folders — struct, auto-assign, reorder

**Files:**
- Modify: `src-tauri/src/models.rs` (`Folder` struct)
- Modify: `src-tauri/src/store/folder_store.rs` (`create_folder`, `list_folders`, `get_folder`, `row_to_folder`, existing tests; add `reorder_folders`)
- Modify: `src-tauri/src/commands.rs` (add `reorder_folders` command)
- Modify: `src-tauri/src/lib.rs` (register `commands::reorder_folders`)

**Interfaces:**
- Consumes: same `AppError` variants and `unchecked_transaction` pattern as Task 2.
- Produces: `Folder.sort_order: i64`. `folder_store::reorder_folders(conn: &Connection, project_id: &str, parent_folder_id: Option<&str>, ordered_ids: &[String]) -> Result<(), AppError>`. Tauri command `reorder_folders(state, project_id: String, parent_folder_id: Option<String>, ordered_ids: Vec<String>) -> Result<(), AppError>`.

- [ ] **Step 1: Add `sort_order` to the `Folder` struct**

In `src-tauri/src/models.rs`, in `pub struct Folder { ... }`, add (mirroring `Project`'s doc comment):

```rust
    /// Manual order among siblings (same project + parent_folder_id) — see
    /// docs/superpowers/specs/2026-09-15-custom-order-design.md. Only meaningful when the
    /// sidebar's request/folder sort is set to "Custom". Auto-assigned on create; rewritten
    /// wholesale by `reorder_folders`.
    pub sort_order: i64,
```

- [ ] **Step 2: Auto-assign in `create_folder`, select it in `list_folders`/`get_folder`/`update_folder`, fix `row_to_folder`**

In `src-tauri/src/store/folder_store.rs`, in `create_folder`, after the existing parent-folder validation block and before building the `Folder` literal, add:

```rust
    let next_order: i64 = conn.query_row(
        "SELECT COALESCE(MAX(sort_order), -1) + 1 FROM folders WHERE project_id = ?1 AND parent_folder_id IS ?2",
        params![input.project_id, input.parent_folder_id],
        |row| row.get(0),
    )?;
```

Add `sort_order: next_order,` to the `Folder { ... }` literal, and update the `INSERT`:

```rust
    conn.execute(
        "INSERT INTO folders (id, project_id, name, parent_folder_id, sort_order, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            folder.id,
            folder.project_id,
            folder.name,
            folder.parent_folder_id,
            folder.sort_order,
            folder.created_at.to_rfc3339(),
            folder.updated_at.to_rfc3339()
        ],
    )
    .map_err(map_constraint_error)?;
```

Update `list_folders` and `get_folder`'s `SELECT`:

```rust
pub fn list_folders(conn: &Connection, project_id: &str) -> Result<Vec<Folder>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT id, project_id, name, parent_folder_id, sort_order, created_at, updated_at FROM folders
         WHERE project_id = ?1 ORDER BY name ASC",
    )?;
    let rows = stmt.query_map(params![project_id], row_to_folder)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(AppError::from)
}

pub fn get_folder(conn: &Connection, id: &str) -> Result<Folder, AppError> {
    conn.query_row(
        "SELECT id, project_id, name, parent_folder_id, sort_order, created_at, updated_at FROM folders WHERE id = ?1",
        params![id],
        row_to_folder,
    )
    .map_err(|err| match err {
        rusqlite::Error::QueryReturnedNoRows => AppError::NotFound(format!("folder {id} not found")),
        other => AppError::from(other),
    })
}
```

Update `row_to_folder` (indices shift — `sort_order` is column 4, so `created_at`/`updated_at` move to 5/6):

```rust
fn row_to_folder(row: &rusqlite::Row) -> rusqlite::Result<Folder> {
    let created_at: String = row.get(5)?;
    let updated_at: String = row.get(6)?;
    Ok(Folder {
        id: row.get(0)?,
        project_id: row.get(1)?,
        name: row.get(2)?,
        parent_folder_id: row.get(3)?,
        sort_order: row.get(4)?,
        created_at: created_at.parse().unwrap_or_else(|_| Utc::now()),
        updated_at: updated_at.parse().unwrap_or_else(|_| Utc::now()),
    })
}
```

`update_folder` builds a `Folder { ... }` literal by hand from `existing`:

```rust
    Ok(Folder {
        id: existing.id,
        project_id: existing.project_id,
        name,
        parent_folder_id: existing.parent_folder_id,
        sort_order: existing.sort_order,
        created_at: existing.created_at,
        updated_at,
    })
```

Add the `sort_order: existing.sort_order,` line — updates never touch order, only `reorder_folders` does.

- [ ] **Step 3: Add `reorder_folders`**

Append to `src-tauri/src/store/folder_store.rs`, before the `fn map_constraint_error` helper:

```rust
/// Same contract as `project_store::reorder_projects`: `ordered_ids` is the complete new
/// order for one sibling group (same project + parent), rewritten to 0..N. `parent_folder_id`
/// uses `IS` (not `=`) so root-level folders (`NULL` parent) reorder correctly too.
pub fn reorder_folders(
    conn: &Connection,
    project_id: &str,
    parent_folder_id: Option<&str>,
    ordered_ids: &[String],
) -> Result<(), AppError> {
    let tx = conn.unchecked_transaction()?;
    for (idx, id) in ordered_ids.iter().enumerate() {
        let affected = tx.execute(
            "UPDATE folders SET sort_order = ?1 WHERE id = ?2 AND project_id = ?3 AND parent_folder_id IS ?4",
            params![idx as i64, id, project_id, parent_folder_id],
        )?;
        if affected == 0 {
            return Err(AppError::Validation(format!(
                "folder {id} not found in that project/parent"
            )));
        }
    }
    tx.commit()?;
    Ok(())
}
```

- [ ] **Step 4: Fix the existing order-dependent test, add new tests**

`list_orders_by_name_and_get_round_trips` in the same file's test module still passes unchanged (it asserts on `.name`, and `list_folders`'s `ORDER BY name ASC` is untouched). Add two new tests right after it:

```rust
    #[test]
    fn create_auto_assigns_sequential_sort_order_per_parent() {
        let conn = db::open_in_memory().unwrap();
        let project_id = seed_project(&conn);
        let root_a = create_folder(&conn, NewFolderInput { project_id: project_id.clone(), name: "Root A".into(), ..Default::default() }).unwrap();
        let root_b = create_folder(&conn, NewFolderInput { project_id: project_id.clone(), name: "Root B".into(), ..Default::default() }).unwrap();
        assert_eq!((root_a.sort_order, root_b.sort_order), (0, 1));

        // A child of root_a starts its own sequence at 0, independent of its parent's siblings.
        let child = create_folder(
            &conn,
            NewFolderInput { project_id, name: "Child".into(), parent_folder_id: Some(root_a.id.clone()) },
        )
        .unwrap();
        assert_eq!(child.sort_order, 0);
    }

    #[test]
    fn reorder_folders_rewrites_order_within_one_parent_scope() {
        let conn = db::open_in_memory().unwrap();
        let project_id = seed_project(&conn);
        let a = create_folder(&conn, NewFolderInput { project_id: project_id.clone(), name: "A".into(), ..Default::default() }).unwrap();
        let b = create_folder(&conn, NewFolderInput { project_id: project_id.clone(), name: "B".into(), ..Default::default() }).unwrap();

        reorder_folders(&conn, &project_id, None, &[b.id.clone(), a.id.clone()]).unwrap();
        assert_eq!(get_folder(&conn, &b.id).unwrap().sort_order, 0);
        assert_eq!(get_folder(&conn, &a.id).unwrap().sort_order, 1);
    }
```

- [ ] **Step 5: Add the Tauri command**

In `src-tauri/src/commands.rs`, near `list_folders`:

```rust
#[tauri::command]
pub fn reorder_folders(
    state: State<AppState>,
    project_id: String,
    parent_folder_id: Option<String>,
    ordered_ids: Vec<String>,
) -> Result<(), AppError> {
    let conn = state.db.lock().expect("db mutex poisoned");
    folder_store::reorder_folders(&conn, &project_id, parent_folder_id.as_deref(), &ordered_ids)
}
```

- [ ] **Step 6: Register the command**

In `src-tauri/src/lib.rs`, add `commands::reorder_folders,` right after `commands::list_folders,`.

- [ ] **Step 7: Run the tests**

Run: `cargo test --lib store::folder_store:: --manifest-path src-tauri/Cargo.toml`
Expected: all PASS (existing tests unaffected, two new ones pass).

Run: `cargo check --manifest-path src-tauri/Cargo.toml`
Expected: no errors.

- [ ] **Step 8: Commit**

```bash
git add src-tauri/src/models.rs src-tauri/src/store/folder_store.rs src-tauri/src/commands.rs src-tauri/src/lib.rs
git commit -m "feat(folders): persist and reorder folder sort_order"
```

---

## Task 4: `sort_order` on requests — struct, auto-assign, reorder

**Files:**
- Modify: `src-tauri/src/models.rs` (`RequestSummary`, `RequestFull` structs)
- Modify: `src-tauri/src/store/request_store.rs` (`create_request`, `list_requests`, `get_request`, `update_request`; add `reorder_requests`)
- Modify: `src-tauri/src/commands.rs` (add `reorder_requests` command)
- Modify: `src-tauri/src/lib.rs` (register `commands::reorder_requests`)

**Interfaces:**
- Produces: `RequestSummary.sort_order: i64`, `RequestFull.sort_order: i64`. `request_store::reorder_requests(conn: &Connection, project_id: &str, folder_id: Option<&str>, ordered_ids: &[String]) -> Result<(), AppError>`. Tauri command `reorder_requests(state, project_id: String, folder_id: Option<String>, ordered_ids: Vec<String>) -> Result<(), AppError>`.

- [ ] **Step 1: Add `sort_order` to both structs**

In `src-tauri/src/models.rs`, add to `RequestSummary` (after `pub folder_id: Option<String>,`, before `pub name: String,` — placement doesn't matter for a named-field struct, but keep it near `folder_id` since both describe "where this request sits"):

```rust
    /// Manual order among siblings (same project + folder_id) — see
    /// docs/superpowers/specs/2026-09-15-custom-order-design.md. Only meaningful when the
    /// sidebar's request sort is set to "Custom". Auto-assigned on create; rewritten wholesale
    /// by `reorder_requests`.
    pub sort_order: i64,
```

Add the identical field (same doc comment) to `RequestFull`.

- [ ] **Step 2: Auto-assign in `create_request`, select it everywhere, fix `RequestFull` construction**

In `src-tauri/src/store/request_store.rs`, in `create_request`, right before the `let request = RequestFull { ... }` block, add:

```rust
    let next_order: i64 = conn.query_row(
        "SELECT COALESCE(MAX(sort_order), -1) + 1 FROM requests WHERE project_id = ?1 AND folder_id IS ?2",
        params![input.project_id, input.folder_id],
        |row| row.get(0),
    )?;
```

Add `sort_order: next_order,` to the `RequestFull { ... }` literal (anywhere in the field list), and update the `INSERT`:

```rust
    conn.execute(
        "INSERT INTO requests (id, project_id, folder_id, name, method, url, headers, query_params, auth, body, description, settings, pre_request_script, post_request_script, sort_order, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)",
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
            request.sort_order,
            request.created_at.to_rfc3339(),
            request.updated_at.to_rfc3339()
        ],
    )
```

Update `list_requests`:

```rust
pub fn list_requests(conn: &Connection, project_id: &str) -> Result<Vec<RequestSummary>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT id, project_id, folder_id, name, method, url, sort_order, created_at, updated_at
         FROM requests WHERE project_id = ?1 ORDER BY updated_at DESC",
    )?;
    let rows = stmt.query_map(params![project_id], |row| {
        let created_at: String = row.get(7)?;
        let updated_at: String = row.get(8)?;
        Ok(RequestSummary {
            id: row.get(0)?,
            project_id: row.get(1)?,
            folder_id: row.get(2)?,
            name: row.get(3)?,
            method: row.get(4)?,
            url: row.get(5)?,
            sort_order: row.get(6)?,
            created_at: created_at.parse().unwrap_or_else(|_| Utc::now()),
            updated_at: updated_at.parse().unwrap_or_else(|_| Utc::now()),
        })
    })?;
    rows.collect::<Result<Vec<_>, _>>().map_err(AppError::from)
}
```

Update `get_request` (insert `sort_order` as column 14, right before `created_at`/`updated_at`, which shift from indices 14/15 to 15/16):

```rust
pub fn get_request(conn: &Connection, id: &str) -> Result<RequestFull, AppError> {
    conn.query_row(
        "SELECT id, project_id, folder_id, name, method, url, headers, query_params, auth, body, description, settings, pre_request_script, post_request_script, sort_order, created_at, updated_at
         FROM requests WHERE id = ?1",
        params![id],
        |row| {
            let headers_json: String = row.get(6)?;
            let query_params_json: String = row.get(7)?;
            let auth_json: String = row.get(8)?;
            let settings_json: Option<String> = row.get(11)?;
            let created_at: String = row.get(15)?;
            let updated_at: String = row.get(16)?;
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
                sort_order: row.get(14)?,
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
```

(This replaces the whole function body — everything after the final `.map_err(...)` block, e.g. any trailing `?` or closing brace pattern, stays exactly as it already is; only the `SELECT` string and the closure above changed.)

`update_request` builds a `RequestFull` from `get_request`'s result (`existing`) plus whatever fields the update actually changed:

```rust
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
        sort_order: existing.sort_order,
        created_at: existing.created_at,
        updated_at,
    })
```

Add the `sort_order: existing.sort_order,` line — updates never touch order, only `reorder_requests` does. Note: `update_request` *can* change `folder_id` (an existing capability, unrelated to this feature) without renumbering `sort_order` for the new folder — a request moved this way keeps its old order value, which may tie with a sibling already at that position in the destination folder. Ties are harmless (stable render order, no crash) and resolve themselves the next time anything in that folder is dragged via `reorder_requests`. This is the same "moving between folders is a separate, already-existing path" boundary the spec's Out of Scope section draws — do not expand this task to renumber on folder-move.

- [ ] **Step 3: Add `reorder_requests`**

Append to `src-tauri/src/store/request_store.rs`:

```rust
/// Same contract as `folder_store::reorder_folders`: `ordered_ids` is the complete new order
/// for one sibling group (same project + folder), rewritten to 0..N. `folder_id` uses `IS`
/// (not `=`) so requests sitting directly under the project root (`NULL` folder) reorder
/// correctly too.
pub fn reorder_requests(
    conn: &Connection,
    project_id: &str,
    folder_id: Option<&str>,
    ordered_ids: &[String],
) -> Result<(), AppError> {
    let tx = conn.unchecked_transaction()?;
    for (idx, id) in ordered_ids.iter().enumerate() {
        let affected = tx.execute(
            "UPDATE requests SET sort_order = ?1 WHERE id = ?2 AND project_id = ?3 AND folder_id IS ?4",
            params![idx as i64, id, project_id, folder_id],
        )?;
        if affected == 0 {
            return Err(AppError::Validation(format!(
                "request {id} not found in that project/folder"
            )));
        }
    }
    tx.commit()?;
    Ok(())
}
```

- [ ] **Step 4: Write the store tests**

Add to `request_store.rs`'s test module:

```rust
    #[test]
    fn create_auto_assigns_sequential_sort_order_per_folder() {
        let conn = db::open_in_memory().unwrap();
        let project_id = seed_project(&conn); // check the file's existing tests for the helper name; if absent, inline project_store::create_project as the other tests in this file already do
        let r1 = create_request(&conn, NewRequestInput { project_id: project_id.clone(), name: "First".into(), method: "GET".into(), url: "https://a".into(), ..Default::default() }).unwrap();
        let r2 = create_request(&conn, NewRequestInput { project_id, name: "Second".into(), method: "GET".into(), url: "https://b".into(), ..Default::default() }).unwrap();
        assert_eq!((r1.sort_order, r2.sort_order), (0, 1));
    }

    #[test]
    fn reorder_requests_rewrites_order_within_one_folder_scope() {
        let conn = db::open_in_memory().unwrap();
        let project_id = seed_project(&conn);
        let a = create_request(&conn, NewRequestInput { project_id: project_id.clone(), name: "A".into(), method: "GET".into(), url: "https://a".into(), ..Default::default() }).unwrap();
        let b = create_request(&conn, NewRequestInput { project_id: project_id.clone(), name: "B".into(), method: "GET".into(), url: "https://b".into(), ..Default::default() }).unwrap();

        reorder_requests(&conn, &project_id, None, &[b.id.clone(), a.id.clone()]).unwrap();
        assert_eq!(get_request(&conn, &b.id).unwrap().sort_order, 0);
        assert_eq!(get_request(&conn, &a.id).unwrap().sort_order, 1);
    }
```

`request_store.rs`'s test module already has a `seed_project(conn: &Connection) -> String` helper (mirrors the one in `folder_store.rs`), so the tests above can use it as-is.

- [ ] **Step 5: Add the Tauri command**

In `src-tauri/src/commands.rs`, near `list_requests`:

```rust
#[tauri::command]
pub fn reorder_requests(
    state: State<AppState>,
    project_id: String,
    folder_id: Option<String>,
    ordered_ids: Vec<String>,
) -> Result<(), AppError> {
    let conn = state.db.lock().expect("db mutex poisoned");
    request_store::reorder_requests(&conn, &project_id, folder_id.as_deref(), &ordered_ids)
}
```

- [ ] **Step 6: Register the command**

In `src-tauri/src/lib.rs`, add `commands::reorder_requests,` right after `commands::list_requests,`.

- [ ] **Step 7: Run the full backend test suite**

Run: `cargo test --lib --manifest-path src-tauri/Cargo.toml`
Expected: all tests PASS (this is the point where every store's tests, the importer tests, and everything else that touches `Project`/`Folder`/`RequestSummary`/`RequestFull` literals gets exercised together — `cargo check` in earlier tasks catches compile errors, this catches behavioral ones).

- [ ] **Step 8: Commit**

```bash
git add src-tauri/src/models.rs src-tauri/src/store/request_store.rs src-tauri/src/commands.rs src-tauri/src/lib.rs
git commit -m "feat(requests): persist and reorder request sort_order"
```

---

## Task 5: Frontend types and API bindings

**Files:**
- Modify: `src/lib/api.ts` (`Project`, `Folder`, `RequestSummary` interfaces; add three `reorder*` bindings)

**Interfaces:**
- Consumes: Tauri commands `reorder_projects`, `reorder_folders`, `reorder_requests` from Tasks 2-4 (param names `workspace_id`/`project_id`/`parent_folder_id`/`folder_id`/`ordered_ids` — Tauri's `invoke` maps camelCase JS keys to these snake_case Rust arg names automatically, matching every other binding already in this file, e.g. `searchRequestsInWorkspace`).
- Produces: `api.reorderProjects(workspaceId, orderedIds)`, `api.reorderFolders(projectId, parentFolderId, orderedIds)`, `api.reorderRequests(projectId, folderId, orderedIds)`, all `Promise<void>`. `sort_order: number` available on every `Project`/`Folder`/`RequestSummary`/`RequestFull` (the last via the existing `extends RequestSummary`).

- [ ] **Step 1: Add `sort_order` to the three interfaces**

In `src/lib/api.ts`:

```ts
export interface Project {
  id: string;
  name: string;
  default_environment_id: string | null;
  workspace_id: string;
  sort_order: number;
  created_at: string;
  updated_at: string;
}
```

```ts
export interface RequestSummary {
  id: string;
  project_id: string;
  folder_id: string | null;
  name: string;
  method: string;
  url: string;
  sort_order: number;
  created_at: string;
  updated_at: string;
}
```

```ts
export interface Folder {
  id: string;
  project_id: string;
  name: string;
  parent_folder_id: string | null;
  sort_order: number;
  created_at: string;
  updated_at: string;
}
```

(`RequestFull extends RequestSummary` already, so it picks up `sort_order` with no separate edit.)

- [ ] **Step 2: Add the three reorder bindings**

Right after `searchRequestsInWorkspace` in the `requests` group (and analogous spots for projects/folders — match wherever `createProject`/`createFolder` already live):

```ts
  reorderProjects: (workspaceId: string, orderedIds: string[]) =>
    invoke<void>("reorder_projects", { workspaceId, orderedIds }),
```

```ts
  reorderFolders: (projectId: string, parentFolderId: string | null, orderedIds: string[]) =>
    invoke<void>("reorder_folders", { projectId, parentFolderId, orderedIds }),
```

```ts
  reorderRequests: (projectId: string, folderId: string | null, orderedIds: string[]) =>
    invoke<void>("reorder_requests", { projectId, folderId, orderedIds }),
```

- [ ] **Step 3: Verify types**

Run: `npx svelte-check` (from `D:\hamada\postman`)
Expected: 0 errors. (There will be errors here if `src-tauri` hasn't been built with the Task 1-4 changes yet in this same working copy — that's expected only if those tasks haven't run; if they have, this must be clean since nothing in the frontend *uses* `sort_order` yet, so nothing can be inconsistent.)

- [ ] **Step 4: Commit**

```bash
git add src/lib/api.ts
git commit -m "feat(api): add sort_order fields and reorder bindings"
```

---

## Task 6: "Custom order" sort option (projects, requests, folders)

**Files:**
- Modify: `src/routes/+page.svelte` (`ProjectSortField`, `RequestSortField` types; `sortProjectList`, `sortRequestList`, `foldersByParentId`; `PROJECT_SORT_FIELDS`, `REQUEST_SORT_FIELDS`)
- Modify: `src/lib/i18n.ts` (`sidebar.sortByCustom` key, en + ar)

**Interfaces:**
- Consumes: `Project.sort_order`, `RequestSummary.sort_order`, `Folder.sort_order` from Task 5.
- Produces: `"custom"` is a valid value of both `ProjectSortField` and `RequestSortField`; sorting by it orders by `sort_order` ascending/descending per the existing `dir` toggle.

- [ ] **Step 1: Widen the sort field types**

In `src/routes/+page.svelte`, change:

```ts
  type ProjectSortField = "name" | "created" | "updated";
  type RequestSortField = "name" | "method" | "created" | "updated";
```

to:

```ts
  type ProjectSortField = "name" | "created" | "updated" | "custom";
  type RequestSortField = "name" | "method" | "created" | "updated" | "custom";
```

- [ ] **Step 2: Add the `custom` branch to both sort functions**

```ts
  function sortProjectList(list: Project[]): Project[] {
    const dir = projectSortDir === "asc" ? 1 : -1;
    return [...list].sort((a, b) => {
      if (projectSortField === "custom") return (a.sort_order - b.sort_order) * dir;
      if (projectSortField === "name") return a.name.localeCompare(b.name) * dir;
      if (projectSortField === "created") return a.created_at.localeCompare(b.created_at) * dir;
      return a.updated_at.localeCompare(b.updated_at) * dir;
    });
  }
  function sortRequestList(list: RequestSummary[]): RequestSummary[] {
    const dir = requestSortDir === "asc" ? 1 : -1;
    return [...list].sort((a, b) => {
      if (requestSortField === "custom") return (a.sort_order - b.sort_order) * dir;
      if (requestSortField === "name") return a.name.localeCompare(b.name) * dir;
      if (requestSortField === "method") return a.method.localeCompare(b.method) * dir;
      if (requestSortField === "created") return a.created_at.localeCompare(b.created_at) * dir;
      return a.updated_at.localeCompare(b.updated_at) * dir;
    });
  }
```

- [ ] **Step 3: Add the `custom` branch to the folder sort (shares `requestSortField`)**

In `foldersByParentId`'s `$derived.by`, change:

```ts
    const sorted = [...folders].sort((a, b) => {
      if (requestSortField === "updated") return a.updated_at.localeCompare(b.updated_at) * dir;
      return a.name.localeCompare(b.name) * dir;
    });
```

to:

```ts
    const sorted = [...folders].sort((a, b) => {
      if (requestSortField === "custom") return (a.sort_order - b.sort_order) * dir;
      if (requestSortField === "updated") return a.updated_at.localeCompare(b.updated_at) * dir;
      return a.name.localeCompare(b.name) * dir;
    });
```

- [ ] **Step 4: Add the new menu entries**

```ts
  const PROJECT_SORT_FIELDS: { field: ProjectSortField; label: string }[] = [
    { field: "name", label: "sidebar.sortByName" },
    { field: "created", label: "sidebar.sortByCreated" },
    { field: "updated", label: "sidebar.sortByUpdated" },
    { field: "custom", label: "sidebar.sortByCustom" },
  ];
  const REQUEST_SORT_FIELDS: { field: RequestSortField; label: string }[] = [
    { field: "name", label: "sidebar.sortByName" },
    { field: "method", label: "sidebar.sortByMethod" },
    { field: "created", label: "sidebar.sortByCreated" },
    { field: "updated", label: "sidebar.sortByUpdated" },
    { field: "custom", label: "sidebar.sortByCustom" },
  ];
```


- [ ] **Step 5: Add the i18n key**

In `src/lib/i18n.ts`, find the `sidebar.sortByUpdated` key in both the `en` and `ar` dicts (grep for it) and add right after each:

English section:
```ts
  "sidebar.sortByCustom": "Custom order",
```

Arabic section:
```ts
  "sidebar.sortByCustom": "ترتيب مخصص",
```

- [ ] **Step 6: Also validate `"custom"` in the persisted-project-sort loader**

In the app-mount `$effect` that reads `lp-project-sort` from `localStorage` (search for `savedProjectSort`), widen the field allow-list:

```ts
        if (parsed.field === "name" || parsed.field === "created" || parsed.field === "updated" || parsed.field === "custom") projectSortField = parsed.field;
```

(Leave the `lp-request-sort` load in that same block alone for now — Task 7 replaces it with a per-project version entirely.)

- [ ] **Step 7: Verify**

Run: `npx svelte-check` (from `D:\hamada\postman`)
Expected: 0 errors, no new warnings.

Run: `npm run build`
Expected: succeeds.

- [ ] **Step 8: Commit**

```bash
git add src/routes/+page.svelte src/lib/i18n.ts
git commit -m "feat(sort): add Custom order option for projects/folders/requests"
```

---

## Task 7: Per-project request/folder sort persistence

**Files:**
- Modify: `src/routes/+page.svelte` (`setRequestSortField`, `toggleRequestSortDir`, the app-mount `$effect`, `selectProject`)

**Interfaces:**
- Consumes: `selectedProjectId: string | null` (existing state).
- Produces: `requestSortField`/`requestSortDir` are still the same two `$state` variables everything already reads (no rename — every consumer from Task 6 keeps working unchanged), but persistence moves from one global `localStorage` key to one key per project, loaded fresh every time `selectProject` runs.

- [ ] **Step 1: Remove the global `lp-request-sort` load at app mount**

In the app-mount `$effect`, delete this block entirely (it's being replaced by a per-project load in `selectProject`):

```ts
      const savedRequestSort = localStorage.getItem("lp-request-sort");
      if (savedRequestSort) {
        const parsed = JSON.parse(savedRequestSort);
        if (parsed.field === "name" || parsed.field === "method" || parsed.field === "created" || parsed.field === "updated") requestSortField = parsed.field;
        if (parsed.dir === "asc" || parsed.dir === "desc") requestSortDir = parsed.dir;
      }
```

- [ ] **Step 2: Load the per-project preference in `selectProject`**

In `selectProject(id)`, right after the existing block that loads `lp-open-tabs-${id}`:

```ts
    try {
      const savedTabs = localStorage.getItem(`lp-open-tabs-${id}`);
      if (savedTabs) openTabs = JSON.parse(savedTabs);
    } catch {}
    requestSortField = "name";
    requestSortDir = "asc";
    try {
      const savedRequestSort = localStorage.getItem(`lp-request-sort-${id}`);
      if (savedRequestSort) {
        const parsed = JSON.parse(savedRequestSort);
        if (parsed.field === "name" || parsed.field === "method" || parsed.field === "created" || parsed.field === "updated" || parsed.field === "custom") requestSortField = parsed.field;
        if (parsed.dir === "asc" || parsed.dir === "desc") requestSortDir = parsed.dir;
      }
    } catch {}
```

(The two reset lines before the `try` matter: without them, opening a project with no saved preference would keep whichever `requestSortField`/`Dir` the *previously open* project happened to be left on, rather than falling back to the "name"/"asc" default.)

- [ ] **Step 3: Persist to the per-project key**

Change `setRequestSortField` and `toggleRequestSortDir`:

```ts
  function setRequestSortField(field: RequestSortField) {
    requestSortField = field;
    if (!selectedProjectId) return;
    try { localStorage.setItem(`lp-request-sort-${selectedProjectId}`, JSON.stringify({ field: requestSortField, dir: requestSortDir })); } catch {}
  }
  function toggleRequestSortDir() {
    requestSortDir = requestSortDir === "asc" ? "desc" : "asc";
    if (!selectedProjectId) return;
    try { localStorage.setItem(`lp-request-sort-${selectedProjectId}`, JSON.stringify({ field: requestSortField, dir: requestSortDir })); } catch {}
  }
```

- [ ] **Step 4: Manual verification**

Run the app (`npm run tauri dev` or the project's existing dev workflow), open project A, set its request sort to "Custom", switch to project B — B's sort must still read "Name" (its own default, untouched). Switch back to A — A must still read "Custom". This is the behavior the per-project key exists for; it isn't covered by `svelte-check`.

- [ ] **Step 5: Verify types and build**

Run: `npx svelte-check`
Expected: 0 errors.

- [ ] **Step 6: Commit**

```bash
git add src/routes/+page.svelte
git commit -m "feat(sort): make request/folder sort preference per-project"
```

---

## Task 8: Auto-switch to Custom order after a fresh collection import

**Files:**
- Modify: `src/routes/+page.svelte` (`importPostmanCollectionAction`)

**Interfaces:**
- Consumes: `report.project_id: string` (existing field on `CollectionImportReport`), `collectionImportTarget: "new" | "current"` (existing state).

- [ ] **Step 1: Set the new project's persisted sort before selecting it**

In `importPostmanCollectionAction`, currently:

```ts
      const report = await api.importPostmanCollection(collectionImportText.trim(), targetId, activeWorkspaceId);
      collectionImportReport = report;
      await loadProjects();
      if (!selectedProjectId || collectionImportTarget === "new") {
        await selectProject(report.project_id);
      } else {
        await selectProject(selectedProjectId);
      }
```

Change to:

```ts
      const report = await api.importPostmanCollection(collectionImportText.trim(), targetId, activeWorkspaceId);
      collectionImportReport = report;
      await loadProjects();
      if (!selectedProjectId || collectionImportTarget === "new") {
        // A fresh import already lands its requests in sort_order matching the source
        // collection's order (create_request auto-assigns sequentially — see
        // docs/superpowers/specs/2026-09-15-custom-order-design.md). Without this, the
        // sidebar's default "Name" sort would immediately re-alphabetize them, hiding that
        // order. Written before selectProject() so its per-project load (Task 7) picks
        // this up instead of falling back to the "name"/"asc" default.
        try {
          localStorage.setItem(
            `lp-request-sort-${report.project_id}`,
            JSON.stringify({ field: "custom", dir: "asc" }),
          );
        } catch {}
        await selectProject(report.project_id);
      } else {
        await selectProject(selectedProjectId);
      }
```

- [ ] **Step 2: Manual verification**

Import any multi-request Postman collection (use the `test-postman-importers` skill's guidance for exercising the import path safely). Confirm: the sidebar's request sort control reads "Custom order" immediately after import, and the request list matches the source collection's top-to-bottom order. Then open a *different*, pre-existing project — its own sort preference must be unaffected.

- [ ] **Step 3: Verify types and build**

Run: `npx svelte-check`
Expected: 0 errors.

- [ ] **Step 4: Commit**

```bash
git add src/routes/+page.svelte
git commit -m "feat(import): default a freshly-imported collection to Custom order"
```

---

## Task 9: Drag-and-drop for the project list

**Files:**
- Modify: `src/routes/+page.svelte` (project list markup — `class="project-list"` block; new script state/functions; CSS)
- Modify: `src/lib/i18n.ts` (a tooltip string for the disabled-drag state)

**Interfaces:**
- Consumes: `filteredProjects: Project[]` (existing `$derived`), `api.reorderProjects` (Task 5), `projectSortField` (Task 6).
- Produces: dragging a project row when `projectSortField === "custom"` reorders `projects` locally and persists via `api.reorderProjects`.

- [ ] **Step 1: Add drag state and handlers**

Near `projectSortField`'s declaration in `src/routes/+page.svelte`, add:

```ts
  // Drag-and-drop reordering — only meaningful (and only enabled in the markup) once
  // projectSortField === "custom"; dragging under Name/Created/Updated would just get
  // silently undone by the next re-sort, so it's disabled rather than working-then-reverting.
  let draggedProjectId = $state<string | null>(null);
  function onProjectDragStart(id: string) {
    if (projectSortField !== "custom") return;
    draggedProjectId = id;
  }
  async function onProjectDrop(targetId: string) {
    const draggedId = draggedProjectId;
    draggedProjectId = null;
    if (!draggedId || draggedId === targetId || projectSortField !== "custom" || !activeWorkspaceId) return;

    const ids = filteredProjects.map((p) => p.id);
    const from = ids.indexOf(draggedId);
    const to = ids.indexOf(targetId);
    if (from === -1 || to === -1) return;
    ids.splice(to, 0, ids.splice(from, 1)[0]);

    const orderById = new Map(ids.map((id, idx) => [id, idx]));
    const previousProjects = projects;
    projects = projects.map((p) => (orderById.has(p.id) ? { ...p, sort_order: orderById.get(p.id)! } : p));

    try {
      await api.reorderProjects(activeWorkspaceId, ids);
    } catch (err) {
      projects = previousProjects;
      errorMessage = describeError(err);
    }
  }
```

- [ ] **Step 2: Wire the markup**

In the `class="project-list"` block, the top-level `{#each filteredProjects as project (project.id)}` renders a `<div class="project-node">` containing `<div class="project-row" ...>`. Add drag attributes to that outer `<div class="project-node">`:

```svelte
        {#each filteredProjects as project (project.id)}
          <div
            class="project-node"
            draggable={projectSortField === "custom"}
            class:drag-active={projectSortField === "custom"}
            ondragstart={() => onProjectDragStart(project.id)}
            ondragover={(e) => { if (projectSortField === "custom") e.preventDefault(); }}
            ondrop={(e) => { e.preventDefault(); onProjectDrop(project.id); }}
            title={projectSortField === "custom" ? undefined : t("sidebar.dragRequiresCustomOrder")}
          >
```

(Keep everything already inside that `<div class="project-node">` unchanged — this only adds attributes to its opening tag.)

- [ ] **Step 3: Add the CSS affordance**

Near `.project-row`'s existing styles, add:

```css
  /* A draggable row gets a grab cursor only once Custom order is active — otherwise it stays
     the default cursor (attempting to drag does nothing, and the title tooltip from the
     markup explains why). */
  .project-node[draggable="true"] {
    cursor: grab;
  }
  .project-node[draggable="true"]:active {
    cursor: grabbing;
  }
```

- [ ] **Step 4: Add the tooltip i18n key**

`src/lib/i18n.ts`, `en` and `ar` sections, near `sidebar.sortByCustom`:

```ts
  "sidebar.dragRequiresCustomOrder": "Switch to Custom order to drag",
```
```ts
  "sidebar.dragRequiresCustomOrder": "بدّل إلى الترتيب المخصص للسحب",
```

- [ ] **Step 5: Manual verification**

Run the app, switch the project sort to "Custom order", drag a project to a new position — it must stay there after a reload (persisted via `reorder_projects`). Switch sort back to "Name" — dragging must have no effect (cursor stays normal, drop does nothing).

- [ ] **Step 6: Verify and build**

Run: `npx svelte-check`
Expected: 0 errors.

Run: `npm run build`
Expected: succeeds.

- [ ] **Step 7: Commit**

```bash
git add src/routes/+page.svelte src/lib/i18n.ts
git commit -m "feat(dnd): drag-and-drop reordering for the project list"
```

---

## Task 10: Drag-and-drop for folders and requests within a project

**Files:**
- Modify: `src/routes/+page.svelte` (folder/request row markup inside the active project's tree — `requestRow` snippet, `folderNode` snippet, the root request list; new script state/functions; CSS)

**Interfaces:**
- Consumes: `rootRequests`, `requestsByFolderId`, `foldersByParentId` (existing `$derived`s from Task 6), `api.reorderFolders`/`api.reorderRequests` (Task 5), `requestSortField` (Task 6/7).
- Produces: dragging a request or folder row when `requestSortField === "custom"` reorders `requests`/`folders` locally (within the same folder/parent) and persists via the matching `reorder_*` call.

- [ ] **Step 1: Add drag state and handlers**

Near the Task 9 handlers (or right after them, same area of the script), add:

```ts
  // Same pattern as the project list's drag-and-drop (Task 9), but scoped per sibling group:
  // a request only reorders among requests in the same folder (or same project root); a
  // folder only reorders among folders under the same parent (or same project root).
  let draggedRequestId = $state<string | null>(null);
  function onRequestDragStart(id: string) {
    if (requestSortField !== "custom") return;
    draggedRequestId = id;
  }
  async function onRequestDrop(targetId: string, folderId: string | null) {
    const draggedId = draggedRequestId;
    draggedRequestId = null;
    if (!draggedId || draggedId === targetId || requestSortField !== "custom" || !selectedProjectId) return;

    const siblings = folderId ? (requestsByFolderId.get(folderId) ?? []) : rootRequests;
    const ids = siblings.map((r) => r.id);
    const from = ids.indexOf(draggedId);
    const to = ids.indexOf(targetId);
    if (from === -1 || to === -1) return;
    ids.splice(to, 0, ids.splice(from, 1)[0]);

    const orderById = new Map(ids.map((id, idx) => [id, idx]));
    const previousRequests = requests;
    requests = requests.map((r) => (orderById.has(r.id) ? { ...r, sort_order: orderById.get(r.id)! } : r));

    try {
      await api.reorderRequests(selectedProjectId, folderId, ids);
    } catch (err) {
      requests = previousRequests;
      errorMessage = describeError(err);
    }
  }

  let draggedFolderId = $state<string | null>(null);
  function onFolderDragStart(id: string) {
    if (requestSortField !== "custom") return;
    draggedFolderId = id;
  }
  async function onFolderDrop(targetId: string, parentFolderId: string | null) {
    const draggedId = draggedFolderId;
    draggedFolderId = null;
    if (!draggedId || draggedId === targetId || requestSortField !== "custom" || !selectedProjectId) return;

    const key = parentFolderId ?? ROOT_FOLDER_KEY;
    const siblings = foldersByParentId.get(key) ?? [];
    const ids = siblings.map((f) => f.id);
    const from = ids.indexOf(draggedId);
    const to = ids.indexOf(targetId);
    if (from === -1 || to === -1) return;
    ids.splice(to, 0, ids.splice(from, 1)[0]);

    const orderById = new Map(ids.map((id, idx) => [id, idx]));
    const previousFolders = folders;
    folders = folders.map((f) => (orderById.has(f.id) ? { ...f, sort_order: orderById.get(f.id)! } : f));

    try {
      await api.reorderFolders(selectedProjectId, parentFolderId, ids);
    } catch (err) {
      folders = previousFolders;
      errorMessage = describeError(err);
    }
  }
```

- [ ] **Step 2: Wire the request row snippet**

`{#snippet requestRow(req: RequestSummary)}` (around line 4665) has root element `<li class="request-item-wrapper">`. Add drag attributes to it, keeping the existing `class` as-is:

```svelte
  {#snippet requestRow(req: RequestSummary)}
    {@const sampleCount = sampleResponsesByRequestId.get(req.id)?.length}
    <li
      class="request-item-wrapper"
      draggable={requestSortField === "custom"}
      ondragstart={() => onRequestDragStart(req.id)}
      ondragover={(e) => { if (requestSortField === "custom") e.preventDefault(); }}
      ondrop={(e) => { e.preventDefault(); onRequestDrop(req.id, req.folder_id); }}
      title={requestSortField === "custom" ? undefined : t("sidebar.dragRequiresCustomOrder")}
    >
```

(Everything from the existing `<div class="request-item" ...>` onward, through the snippet's closing `</li>`, stays exactly as it already is — only the opening `<li>` tag's attributes change.)

- [ ] **Step 3: Wire the folder row**

Find `{#snippet folderNode(project: Project, folder: Folder)}`, which contains `<div class="folder-row">`. Add the same drag attributes to that `<div>`:

```svelte
    <div
      class="folder-row"
      draggable={requestSortField === "custom"}
      ondragstart={() => onFolderDragStart(folder.id)}
      ondragover={(e) => { if (requestSortField === "custom") e.preventDefault(); }}
      ondrop={(e) => { e.preventDefault(); onFolderDrop(folder.id, folder.parent_folder_id); }}
      title={requestSortField === "custom" ? undefined : t("sidebar.dragRequiresCustomOrder")}
    >
```

(`secondaryFolderNode` — the variant used for a project rendered in "secondary" mode — is out of scope for this task; note it in the commit message as a known gap rather than silently skipping it, since it renders folders for a project other than the actively-selected one and the drag handlers above assume `selectedProjectId` is the project being dragged in.)

- [ ] **Step 4: CSS**

Reuse the same cursor rule pattern as Task 9, extended to the two new selectors:

```css
  .request-item-wrapper[draggable="true"],
  .folder-row[draggable="true"] {
    cursor: grab;
  }
  .request-item-wrapper[draggable="true"]:active,
  .folder-row[draggable="true"]:active {
    cursor: grabbing;
  }
```

- [ ] **Step 5: Manual verification**

With a project open and its request sort set to "Custom order": drag a root-level request to a new position (persists after reload); drag a request within a folder (persists, and does not affect root-level order); drag a folder among its siblings (persists). Switch sort back to "Name" — none of these drags should have any effect.

- [ ] **Step 6: Verify and build**

Run: `npx svelte-check`
Expected: 0 errors.

Run: `npm run build`
Expected: succeeds.

- [ ] **Step 7: Commit**

```bash
git add src/routes/+page.svelte
git commit -m "feat(dnd): drag-and-drop reordering for folders and requests"
```

---

## Task 11: Full verification pass

**Files:** none (verification only)

- [ ] **Step 1: Full backend test suite**

Run: `cargo test --lib --manifest-path src-tauri/Cargo.toml`
Expected: all tests PASS.

- [ ] **Step 2: Full frontend typecheck and build**

Run: `npx svelte-check` (from `D:\hamada\postman`)
Expected: 0 errors, no new warnings beyond the pre-existing `.detail h3` unused-selector one.

Run: `npm run build`
Expected: succeeds.

- [ ] **Step 3: Manual end-to-end pass in the running app**

Per the `run` skill / this project's own dev workflow (not the Chrome dev-server proxy, which has no real Tauri backend — see project memory on browser-testing limits): import a real multi-request Postman collection, confirm Custom order is selected and matches source order; drag a project, a folder, and a request each to a new spot and confirm persistence across a restart; switch every sort control back to Name/Created/Updated and confirm dragging is inert there.

- [ ] **Step 4: Report**

Summarize what was verified and any gaps found (e.g. the `secondaryFolderNode` gap noted in Task 10) back to the user — no commit for this task, it's a checkpoint.

# Custom order: Postman import order + manual drag-and-drop

Date: 2026-09-15

## Problem

Today nothing persists a manual/source order:

- `projects`, `folders`, `requests` have no order column. The sidebar always re-sorts
  client-side by Name/Created/Updated (`sortProjectList`/`sortRequestList` in
  `src/routes/+page.svelte`), default "Name" ascending.
- Because of that, importing a Postman collection does *not* preserve the collection's
  original folder/request order — the sidebar immediately re-alphabetizes everything.
- There's no way for a user to manually arrange folders/requests/projects and have that
  stick.

## Goals

1. Importing a Postman collection preserves the collection's original order, and that
   project's sidebar view shows it (not silently re-alphabetized).
2. Users can manually drag-and-drop projects, folders, and requests to set their own
   order, and it persists.
3. Existing users see zero behavior change until they explicitly opt in — "Custom" is a
   new sort choice, not the new default.

## Data model

Add `sort_order INTEGER NOT NULL DEFAULT 0` to `projects`, `folders`, and `requests` via
migration 17 in `src-tauri/src/db.rs` (append-only, per README §35 — never edit a shipped
migration). Backfill existing rows with a sequential order per sibling group, ordered by
`created_at`:

- projects: partitioned by `workspace_id`
- folders: partitioned by `(project_id, parent_folder_id)`
- requests: partitioned by `(project_id, folder_id)`

SQLite's `ROW_NUMBER() OVER (PARTITION BY ... ORDER BY created_at)` (window functions,
available in the bundled SQLite) does this in one `UPDATE ... FROM` statement per table.

## Backend

- `create_project`/`create_folder`/`create_request` (in their respective `store/*.rs`)
  always auto-assign `sort_order = COALESCE(MAX(sort_order), -1) + 1` among siblings
  (same partition key as above) — no caller-supplied override. This alone is what makes
  import order "just work" (see Importer below): siblings created one at a time, in
  order, land in sequential `sort_order` for free.
- `Project`, `Folder`, `RequestSummary` (and `RequestFull`) gain a `sort_order: i64`
  field, returned by every list/get so the frontend can sort by it.
- Three new commands/store functions, one per scope, each takes the sibling group's full
  ordered id list and rewrites `sort_order` to the list index (0, 1, 2, ...) in a single
  transaction, validating every id actually belongs to that scope before writing anything:
  - `reorder_projects(workspace_id, ordered_ids)`
  - `reorder_folders(project_id, parent_folder_id, ordered_ids)`
  - `reorder_requests(project_id, folder_id, ordered_ids)`

## Importer

No importer code changes needed. `postman_compat/importer.rs`'s single `.json`
collection import creates every request sequentially, in source order, all as siblings
of one project (it flattens Postman's nested folders into the request name — it never
calls `folder_store::create_folder` at all, an existing simplification this spec doesn't
touch). Since `create_request`'s auto-assign (above) gives each new row
`MAX(siblings) + 1`, and siblings here are created one at a time in exactly the order
they appear in the source JSON, the resulting `sort_order` already matches collection
order with no explicit wiring. The same reasoning covers
`local_workspace_importer.rs`'s nested `create_folder`/`create_request` calls (order
there reflects directory-walk order rather than a Postman-authored order, which is an
inherent limit of a directory-based import, not something this feature can improve).

After the single-collection import (`importPostmanCollectionAction` in
`+page.svelte`) finishes creating a **new** project (`collectionImportTarget === "new"`),
the frontend sets that project's *persisted* request/folder sort preference (see below)
to `{ field: "custom", dir: "asc" }` before selecting it — so the user lands on the
collection's original order without manually switching the sort dropdown. Scoped to:
this one import path, and only when it created a new project (not "import into current
project", which merges into a project that already has its own order). The
multi-project local workspace importer is intentionally left out of this
auto-switch — it doesn't select a single resulting project the way the collection
import does, and directory order isn't a meaningful "original order" to promote by
default anyway.

## Frontend sort

- Add `"custom"` to `ProjectSortField` and `RequestSortField`
  (`src/routes/+page.svelte`), with a new "Custom order" entry in
  `PROJECT_SORT_FIELDS`/`REQUEST_SORT_FIELDS`. `sortProjectList`/`sortRequestList` add a
  branch: `if (field === "custom") return (a.sort_order - b.sort_order) * dir`.
- Folders already ride on `requestSortField`/`requestSortDir` (confirmed in
  `foldersByParentId`), falling back to name for fields folders don't have — extend that
  fallback list to leave `sort_order` as a real field folders do have, instead of falling
  back to name for "custom".
- **Sort preference becomes per-project**, not global. Today `lp-request-sort` is one
  global localStorage key; change it to `lp-request-sort-${projectId}`, following the
  exact pattern already used for `lp-open-tabs-${projectId}` /
  `lp-selected-request-${projectId}` in `selectProject()`. Falls back to the (still
  global) default of `{ field: "name", dir: "asc" }` for a project with no saved
  preference. `lp-project-sort` (the top-level project list) stays global/workspace-wide
  — it's not part of any single import.

## Drag-and-drop

Native HTML5 DnD (`draggable`, `dragstart`/`dragover`/`drop`) on project rows, folder
rows, and request rows. **Only enabled when that scope's sort field is already
"Custom"** — dragging under Name/Created/Updated would just get silently overridden by
the next re-sort, so the drag handle is disabled (with a tooltip: "Switch to Custom
order to drag") rather than working and then appearing to do nothing.

On drop: reorder the sibling id list locally and update `sort_order` on the affected
items optimistically (so the UI reflects the new order immediately), then call the
matching `reorder_*` command; on failure, revert to the pre-drop order and surface
`errorMessage` the same way other failed mutations already do.

## Testing

- Rust: migration idempotency (existing `migrations_apply_cleanly_and_are_idempotent`
  test still passes), backfill produces sequential per-partition order, `create_*`
  auto-assigns next `sort_order`, each `reorder_*` rewrites correctly and rejects
  ids outside its stated scope.
- Importer: `postman_compat/importer.rs` has no existing unit test suite (collection
  import is verified manually against real Postman exports, per the
  `test-postman-importers` skill) — this feature doesn't add one either, since the
  mechanism it relies on (`create_request`'s auto-assign) is already covered directly by
  the `request_store` tests above. Manual verification: import a real multi-request
  collection and confirm order in the running app.
- Frontend: `svelte-check` clean; manual verification in the running app (per
  `test-postman-importers` skill for import-path changes) since drag-and-drop and
  cross-project sidebar state aren't practically covered by the existing test setup.

## Out of scope

- Dragging a request into a *different* folder (move + reorder) — this spec only
  reorders within the same sibling group. Moving between folders already exists via a
  different UI path (not touched here).
- Reordering nested data that already has its own ordering concept (sample responses,
  environment variables, etc.).

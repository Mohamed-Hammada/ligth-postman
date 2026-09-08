---
name: test-postman-importers
description: Use when changing anything in src-tauri/src/postman_compat/ (importer.rs, local_workspace_importer.rs, exporter.rs, schema.rs) — how to test against real Postman data safely, run cargo without fighting the running desktop app, and reset the local database when a test import needs a clean slate.
---

# Testing the Postman import/export code

This project has **two separate importers** that parse Postman data completely
independently — a bug fix in one does not apply to the other:

- `src-tauri/src/postman_compat/importer.rs` — a single Postman Collection v2.1
  JSON file (`import_collection`). Postman request/response objects, `header: []`
  arrays, `body.mode` + `body.options.raw.language`.
- `src-tauri/src/postman_compat/local_workspace_importer.rs` — Postman's newer
  on-disk "local files" workspace format: a directory tree of
  `collections/<name>/**/*.request.yaml` + `environments/*.environment.yaml`
  (`import_local_workspace`). Different schema, different quirks.

When fixing an import bug, check whether it exists in **both** files before
calling it done — they duplicate a lot of the same categories of logic
(headers, body-type inference, auth) with separate, hand-rolled parsing.

## Known real-world quirks in the local-workspace YAML format

Discovered by running the importer against a real ~3,200-request workspace
(not from the small hand-written test fixtures) — the source of truth for
"does this actually work" is real exported data, not synthetic examples:

- `headers` is sometimes a flat mapping (`Key: value`) and sometimes a
  **sequence** of `{key, value, disabled, description}` objects — Postman
  switches shape the moment one header needs `disabled` or a description
  (there's no way to express that in the flat mapping). Handle both.
- A raw body's `type` field (json/xml/html/javascript) is the only source for
  its Content-Type — it is **not** mirrored into an explicit header. If the
  importer doesn't translate `body.type` into a real `Content-Type` header
  when the file doesn't already have one, the request editor's raw-type
  dropdown falls back to "Text" and formatting/Beautify has nothing to detect.
- Some source files declare `body.type: text` even though the content is
  actually JSON (the original Postman request was authored without ever
  switching the language dropdown). This is a property of the source data,
  not a bug to fix on import — the frontend's Beautify button
  (`prettifyBody()` in `+page.svelte`) content-sniffs as a fallback for
  exactly this case, so don't try to "fix" it by guessing at import time.

## Testing against real data without touching the live app database

Point the importer at a real workspace directory with an in-memory SQLite
connection — zero risk to the user's actual `app.db`:

```rust
#[test]
#[ignore] // throwaway — real absolute host path, not portable; delete when done
fn diagnostic_real_import() {
    let conn = crate::db::open_in_memory().unwrap();
    let report = import_local_workspace(&conn, r"D:\postman\postman", "default").unwrap();
    println!("requests_imported={} warnings={}", report.requests_imported, report.warnings.len());
    // ... assert/print whatever the bug report is about, e.g. iterate
    // request_store::get_request() per request and check headers/body ...
}
```

Run with `cargo test <name> -- --ignored --nocapture` to see `println!` output.
**Delete this test before wrapping up** — it's a diagnostic scratchpad, not a
permanent test (it depends on an absolute path that only exists on this
machine). Write a real, portable test with an inline fixture (see the
existing tests in both files for the pattern: `ScratchDir` for the local
workspace importer, an inline JSON string for the collection importer) to
actually pin the fix.

**Common false positive when scripting a diagnostic:** the stored `body`
string for a structured body (form-data/urlencoded/graphql) is itself tagged
JSON internally (e.g. `{"type":"FormData","items":[...]}`), so a naive
`body.starts_with('{')` check will misidentify it as a raw JSON body. Use
`crate::canonical_request::resolve_body(body, &identity_fn)` and check that
both the multipart and content-type return values are `None` before treating
something as a genuine raw body — that's what the app itself uses to tell the
difference at send time.

## Running cargo while `tauri dev` is running

If the user has the desktop app open via `tauri dev` (very likely — they'll
often ask you to keep it running so they can check a fix live), its debug
binary is locked and a same-directory `cargo check`/`cargo test` will either
block on the file lock or fail outright trying to remove the old exe. Don't
ask the user to close the app — build into an isolated target dir instead:

```bash
CARGO_TARGET_DIR="<scratchpad>/cargo-target" cargo test postman_compat --message-format=short
```

This also means a source change won't require *you* to relaunch anything —
`tauri dev`'s own file watcher rebuilds and restarts the running app
automatically on save (watch its log for `File ... changed. Rebuilding
application...`). Only launch a fresh `tauri dev` yourself if the user
explicitly says nothing is running, or after a database reset (see below) —
and never start a second one in parallel with theirs; check
`tasklist | grep -i postman-client` first.

## Resetting the local app database

The SQLite database lives at
`%APPDATA%\com.hamada.postmanclient\app.db` (+ `-wal`/`-shm`), independent of
the source tree. This is **real user data** (client API projects) unless
they've told you otherwise — never delete it without an explicit, direct
confirmation from the user first, even if the request sounds like "clear
everything." When they do confirm:

1. Stop the running app first (`taskkill //F //PID <postman-client.exe pid> //T`)
   — SQLite doesn't like its file yanked out from under a live connection.
2. Back it up before deleting (copy `app.db*` to `~/Downloads/lightpost-db-backup-<timestamp>/`)
   — cheap insurance, costs nothing, and the user may still want a specific
   project out of it later even after confirming a full wipe.
3. Delete `app.db`, `app.db-wal`, `app.db-shm`.
4. Relaunch (`npx tauri dev` from the project root) — migrations reapply
   automatically against the fresh file (watch the log for `applied
   migration N` lines, 1 through however many exist in `src-tauri/src/db.rs`'s
   `MIGRATIONS`).

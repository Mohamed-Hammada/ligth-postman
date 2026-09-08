# PROJECT_MAP

Live state tracker for the Lightweight API Client (see [`README.md`](./README.md) for the full architecture spec — section numbers below refer to it). Update this file in the same commit as the code that changes its status. Nothing here is aspirational; every "done" line has a passing test or a verified manual run behind it.

---

## SYSTEM_FLOW (target end-to-end journey)

```text
Launch app (fast, low idle RAM)
    → See project list (metadata only)
    → Open/create a project
    → See request list for that project (metadata only)
    → Open a request → hydrate full request on demand
    → Edit request → Save locally (instant, no network)
    → Send request → HTTP engine executes → response viewer
    → Optionally: attach project to a Git repo → Sync (manual/auto)
    → Optionally: import a Postman collection into a project
```

Everything below is scoped against this flow. A feature that doesn't serve a step in it doesn't belong in this codebase yet (Protocol 4).

---

## DONE

| Area | What's implemented | Verified by |
|---|---|---|
| Desktop shell | Tauri 2 + SvelteKit + TS scaffold, SPA mode | `npm run build`, `cargo build` both clean |
| Persistence (§6) | SQLite via `rusqlite` (bundled), WAL + foreign_keys pragmas, versioned migration runner (`src-tauri/src/db.rs`) | `db::tests::migrations_apply_cleanly_and_are_idempotent` |
| Domain model — Project (§5) | `Project`, `NewProjectInput` (`src-tauri/src/models.rs`) | `project_store::tests::*` (3 tests) |
| Domain model — Request (§13, partial) | `RequestFull`/`RequestSummary` split so list views never carry headers/body (§4/§20 lazy loading) | `request_store::tests::*` (4 tests) |
| Store layer | `project_store.rs`, `request_store.rs` — create/list/get, input validation (empty name, unknown HTTP method, missing project FK → typed `Validation` error, not a raw SQLite error) | 8/8 `cargo test --lib` passing |
| Error handling (§34) | `AppError` enum (`Storage`/`NotFound`/`Validation`), serialized to the frontend as `{ kind, message }`, never a raw Rust error string | `describeError()` in `src/lib/api.ts` |
| Tauri command layer | `create_project`, `list_projects`, `get_project`, `create_request`, `list_requests`, `get_request` — all synchronous over a `Mutex<Connection>` | Manual run: app launches, DB opens, migrations 1 & 2 apply (see log below) |
| Save system (§7) | Every command commits directly; no debounce needed yet since there's no continuous-edit surface (single-field creates only) | N/A yet — revisit once the request editor gets live-typing |
| Frontend vertical slice | Sidebar project list + create form; request list per project (lazy) + create form; click-to-hydrate full request detail | `npm run check` — 0 errors; manual flow reasoned through against the command contracts |
| Observability (§30, partial) | `tauri-plugin-log` wired, `log::info!` on project/request creation and DB open — no secrets logged | Manual run log: `opening database at ...`, `applied migration 1`, `applied migration 2` |

**Manual smoke test (2026-09-08):** `cargo run` → app launched as `postman-client.exe`, idle RSS ~29 MB, `app.db`/`app.db-shm`/`app.db-wal` created in `%APPDATA%\com.hamada.postmanclient\`, both migrations applied, clean shutdown on kill.

---

## ORPHANS & PENDING

Nothing here is wired to a command, a store, or a UI surface yet. Grouped by README phase (§39). This list is expected to be long right now — the product is a scaffold plus one thin vertical slice, not a Postman replacement.

### Phase 1 — Core (started, not finished)
- [ ] HTTP engine (§18) — no request is actually ever *sent*. `RequestFull` is stored and displayed only.
- [ ] Response persistence/viewer (§19, §23) — no response model, no history table.
- [ ] Update/delete for projects and requests — store layer only has create/list/get.
- [ ] Tab architecture in the UI (§4) — current UI has no tabs at all yet, just a single detail pane.

### Phase 2 — UI
- [ ] Virtualized lists (§21) — plain `{#each}`, fine at current scale, will need virtualization before it meets the "10k requests" bar.
- [ ] Full request editor (headers table, body editor, auth) — UI only shows method/URL/header count today.

### Phase 3 — Postman Compatibility (§12)
- [ ] Postman JSON parser/importer — not started.
- [ ] Environment/variable import — not started.

### Phase 4 — Git (§8, §11, §28)
- [ ] Local Git repository support — not started.
- [ ] Project file format on disk (§28) — projects only exist as SQLite rows right now, no exported file tree.

### Phase 5 — GitHub (§9, §24)
- [ ] Auth, clone, pull/push, sync status — not started.

### Phase 6 — Automatic Sync (§10)
- [ ] Not started (depends on Phase 4/5).

### Not started at all
- [ ] Environments & Variables (§14, §41) — no table, no resolver.
- [ ] Pre/post-request scripts + sandbox (§15–17).
- [ ] Search & indexing beyond the implicit SQL `ORDER BY` (§20).
- [ ] Background job system (§33).
- [ ] Resource budgets / perf benchmarks (§31).
- [ ] Packaging & CI (§27, Phase 8) — icons exist from scaffold, no GitHub Actions workflow yet.
- [ ] Security hardening beyond input validation — no credential storage, no script sandbox (nothing to sandbox yet), no secret redaction (no secrets yet).

---

## Next concrete step

Phase 1 close-out, in order: (1) request **update** + **delete** (currently create/list/get only — an editor is useless without save-in-place), (2) the Rust HTTP engine (`reqwest` or hyper-based, streaming, so §18/§19 have something to render), (3) a `responses` table so a send actually persists something history can later read. None of these should start until explicitly picked up — this file is the handoff point, not a queue to burn through unattended.

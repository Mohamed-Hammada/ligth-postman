# PROJECT_MAP

Live state tracker for the Lightweight API Client (see [`README.md`](./README.md) for the full architecture spec — section numbers below refer to it). Update this file in the same commit as the code that changes its status. Nothing here is aspirational; every "done" line has a passing test or a verified manual run behind it.

---

## Module Boundaries (LP-0009)

Stable, enforced by the actual module graph (not just described in prose):

```text
commands.rs      — Tauri-facing only. Never contains business logic, only calls into
                    store::* / execution / resolver and maps results to AppError.
execution.rs      — Orchestration only: load (store) -> resolve (resolver) -> call
                    (http_engine) -> persist (store). Holds no state of its own.
http_engine.rs    — Zero DB dependency. Takes a fully-resolved spec, returns bytes/status.
                    Could be extracted to its own crate with no changes.
resolver.rs       — Zero DB dependency, zero I/O. Pure function of (template, scope maps).
store/*.rs        — Owns all SQL. Nothing outside store/ touches `rusqlite` directly.
models.rs         — Shared plain data types. No behavior, no I/O.
```

Consequences this protects today: the HTTP engine can be tested against a real socket
with no database in play (`http_engine::tests`); the resolver can be tested with no I/O at
all (`resolver::tests`); a future Git or AI module can sit next to `execution.rs` without
either one needing to know the other exists — both only ever go through `store/*`.

## Architecture Invariants (LP-0010)

Non-negotiable rules and where each is actually enforced in code today:

| Invariant | Enforced by |
|---|---|
| Save != Sync | No sync exists yet; every `update_*`/`create_*` command commits immediately and synchronously from the caller's perspective. |
| Lazy loading — lists never carry heavy fields | `RequestSummary`/`ResponseSummary` structs simply don't have `headers`/`body` fields — not a runtime check, a type-level guarantee. |
| Large response bodies bounded in RAM | `http_engine::capture_body` stops growing its in-memory buffer at `MAX_INLINE_BODY_BYTES` and spills the rest to disk via streamed chunks — verified with a real oversized response in `large_body_spills_to_disk_instead_of_growing_unbounded_in_memory`. |
| Secrets never logged / never sent to frontend unmasked | Every `log::info!` call logs only ids/keys, never `.value`; `VariableView::from(Variable)` is the only path a variable takes to the frontend and it masks `is_secret` values. |
| HTTP execution never blocks the UI thread | `send_request` is an async Tauri command; the DB mutex guard is scoped to synchronous blocks only and is never held across an `.await` (the compiler enforces this — a `std::sync::MutexGuard` held across `.await` fails to compile for a `Send` future). |
| No unnecessary duplicate copies of large data | Response bodies are fetched from disk only on explicit `get_response_body` calls, capped at 2 MB read-back regardless of how large the file on disk is. |

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
| Persistence (§6) | SQLite via `rusqlite` (bundled), WAL + foreign_keys pragmas, versioned migration runner (`src-tauri/src/db.rs`), 3 migrations shipped | `db::tests::migrations_apply_cleanly_and_are_idempotent`; manual run applied migration 3 against a DB that already had 1 & 2 |
| Domain model — Project (§5) | `Project`, `NewProjectInput`, `UpdateProjectInput` (`src-tauri/src/models.rs`) | `project_store::tests::*` (9 tests incl. update/delete/cascade) |
| Domain model — Request (§13, partial) | `RequestFull`/`RequestSummary` split so list views never carry headers/body (§4/§20 lazy loading); `UpdateRequestInput` with explicit `clear_body` to disambiguate "unset" vs "leave alone" | `request_store::tests::*` (11 tests incl. update/delete) |
| Project CRUD | create/list/get/**update/delete** — delete cascades to the project's requests (and, transitively, their variables) via `ON DELETE CASCADE`; explicit design choice, documented in `project_store.rs` | `delete_project_cascades_to_its_requests` |
| Request CRUD | create/list/get/**update/delete** — update only overwrites fields the caller actually sent (verified by a test that an invalid-method update leaves the stored row untouched) | `update_with_only_name_preserves_url_headers_and_body`, `update_rejects_invalid_method_without_mutating_row` |
| Domain model — Environments & Variables (§14/§41) | `Environment` (per-project, unique name); `Variable`/`VariableView` split so secrets are masked before ever reaching the frontend; `VariableScope` enum currently covers `global`/`environment`/`request` (schema already reserves `collection_id`/`folder_id` columns for when those entities exist) | `environment_store::tests::*` (5), `variable_store::tests::*` (7) |
| Variable resolution engine (§14) | `resolver.rs` — pure, DB-free `{{key}}` template substitution over an ordered `ScopeChain` (Runtime > Request > Folder > Collection > Environment > Global), first-seen-wins merge, bounded recursive resolution (5 passes, cycle-safe) so a value can itself contain `{{other_key}}` | 8/8 `resolver::tests::*` — basic resolution, missing-var reporting, multi-scope, precedence ordering, runtime override, nested values, cycle termination, environment switching |
| Store layer | `project_store.rs`, `request_store.rs`, `environment_store.rs`, `variable_store.rs` — full CRUD, input validation, typed errors, duplicate-key/duplicate-name detection, FK-violation mapping | 38/38 `cargo test --lib` passing |
| Error handling (§34) | `AppError` enum (`Storage`/`NotFound`/`Validation`), serialized to the frontend as `{ kind, message }`, never a raw Rust error string | `describeError()` in `src/lib/api.ts` |
| Secrets handling (§41, partial) | `is_secret` flag; `VariableView` masks the value with `••••••••` for any secret variable; a real value is only ever returned by the explicit `reveal_variable_value` command; no `log::info!` call anywhere logs a variable's value (secret or not) | Code inspection + `secret_values_survive_storage_round_trip_for_internal_use` (confirms internal `Variable` keeps the real value for the resolver, only the frontend projection masks it) |
| Tauri command layer | 16 commands total: project/request CRUD (10), environment CRUD subset (4: create/list/update/delete), variable CRUD (5: create/list-for-scope/update/delete/reveal), `resolve_preview` (resolves a template against a live project/environment/request scope chain — same code path the HTTP engine will reuse) | Manual run confirms command registration doesn't break startup; `cargo build` links clean |
| Frontend vertical slice | Sidebar project list (create/rename/delete); request list per project (lazy) with create/delete; click-to-hydrate full request detail with an inline editor (method/name/url, Save/Delete); environment dropdown + inline create; live "Resolves to: ..." preview under the URL field using `resolve_preview` | `npm run check` — 0 errors, 0 warnings; `npm run build` clean |
| Observability (§30, partial) | `tauri-plugin-log` wired, `log::info!` on every create/update/delete/send/cancel — never logs secret, variable, or response body content | Manual run log confirms |
| HTTP engine (§18) | `http_engine.rs` — `reqwest` (rustls-tls, chosen over hyper directly to avoid reimplementing TLS/redirects/proxy, and over blocking clients like `ureq` which would block the async runtime), async, per-request timeout, GET/POST/PUT/PATCH/DELETE/HEAD/OPTIONS via dynamic `Method::from_bytes`, typed `Network` error (timeout vs connect-failure distinguished) | 3 real-socket tests: status/headers/body capture, network-error path (unreachable host), streamed large-body capture |
| Large response handling (§ "Large Response Handling") | Body consumed via `bytes_stream()` chunk-by-chunk; buffered only up to `MAX_INLINE_BODY_BYTES` (256 KB) — every byte past that spills to `<app_data>/response_bodies/<uuid>.bin` via async `tokio::fs`, so the engine never holds more than the cap in RAM regardless of real response size | `large_body_spills_to_disk_instead_of_growing_unbounded_in_memory` — a real 4 KB response forced through a 256-byte cap, spilled file read back and byte-verified |
| Response domain model (§19/§23) | `ResponseSummary` (list/history — no headers, no body) / `ResponseMeta` (headers, no body) / `ResponseBodyPayload` (fetched separately, capped at 2 MB read-back regardless of on-disk size) — mirrors the `RequestSummary`/`RequestFull` lazy-loading split | `response_store::tests::*` (3): inline round-trip, disk-backed read-back, pre-cascade file cleanup |
| Response persistence + cleanup | `responses` table (migration 4); disk-backed bodies are deleted explicitly by `delete_response`, and by `delete_disk_files_for_request`/`_for_project` called **before** the cascading row delete in `request_store`/`project_store` (SQLite FK cascade removes rows, not files — this closes that gap) | `delete_disk_files_for_request_removes_files_before_cascade` |
| Execution pipeline (§15/§16 shape) | `execution.rs::execute_request` — load request (DB) → load scope maps (DB) → **[pre-request script hook — not implemented]** → resolve URL/headers/body via `resolver` → HTTP call (`http_engine`, no DB lock held across the `.await`) → **[post-request/test script hook — not implemented]** → persist (`response_store`) | `full_pipeline_resolves_variables_sends_request_and_persists_response` — real local server, `{{port}}` environment variable resolved into the URL before the request goes out, response verified persisted afterward |
| Cancellation (§18 "Cancellation") | `send_request` races execution against a `tokio::sync::oneshot` via `tokio::select!`; `cancel_send(request_id)` fires it, dropping the losing branch aborts the underlying reqwest/hyper call | Code path exercised by `missing_request_returns_not_found_without_making_a_network_call`; the abort-on-drop behavior itself is standard reqwest/tokio semantics, not independently re-tested |
| Tauri command layer | 26 commands total: 10 project/request CRUD, 4 environment CRUD, 5 variable CRUD/reveal, 1 `resolve_preview`, 6 send/response (`send_request`, `cancel_send`, `list_response_summaries`, `get_response`, `get_response_body`, `delete_response`) | `cargo build` links clean; manual run confirms `AppState` (adds `reqwest::Client`, `response_body_dir`, `cancel_signals` map) doesn't break startup |
| Frontend — send & view | Send/Cancel button on the request detail pane; status/duration/size line; body viewer (capped, scrollable); per-request response history list, click-to-reopen any past response | `npm run check` — 0 errors, 0 warnings; `npm run build` clean |
| Query parameters (§13) | `QueryParam` (key/value/enabled/description) on `RequestFull`, its own JSON column (migration 5) — never baked into `url`. Resolved (each key/value through the same `{{var}}` resolver as headers/body) and appended at send time via `reqwest::Url::query_pairs_mut` (correct encoding); disabled params are dropped, not sent | `append_query_params_*` (3 unit tests) + extended `full_pipeline_...` test proving a disabled param is genuinely absent from the wire while an enabled, variable-resolved one is present |
| Query params UI | Per-request params table (add/toggle/remove rows) wired to `update_request`, only sending the field when it actually changed | `npm run check`/`build` clean; migration 5 applied to a pre-existing on-disk DB in a manual run |
| Windows packaging (§26/§27) | `npx tauri build --debug` produces a real installer via WiX3 (`.msi`) and NSIS (`.exe`) — both toolchains auto-downloaded by the Tauri CLI, no manual setup | `postman-client_0.1.0_x64_en-US.msi` (8.07 MB) and `postman-client_0.1.0_x64-setup.exe` (4.48 MB) exist on disk at `src-tauri/target/debug/bundle/{msi,nsis}/`. Debug profile only — release (`lto=true`) not attempted this session (materially longer compile, not needed to prove the pipeline) |
| AI-assisted generation (Phase 8 subset) | `ai::AiProvider` trait + `ai::ClaudeProvider` — raw HTTP to the Anthropic Messages API (`claude-opus-5`; Rust has no official Anthropic SDK, so raw HTTP is correct here, not a shortcut) with a hardcoded system prompt constraining output to one JSON object, parsed into `GeneratedApiDefinition` and validated (name/method/url) before ever reaching the frontend. Frontend "Ask AI" panel: prompt → generate → preview card → **explicit** "Add to Project" button that calls the same `create_request` command a manual create uses — nothing is persisted before that click | 8 `ai::tests::*`: JSON parsing, code-fence stripping, 3 rejection cases, and — critically — 2 tests against a **real local HTTP server** (not just the parser in isolation) covering a success round-trip and a non-2xx status surfacing as `AppError::Ai`. Manual run confirms `is_ai_configured` correctly returns `false` with no key set, no crash |

**Manual smoke tests:** (1) fresh DB — migrations 1-4 applied, ~29 MB idle RSS; (2) pre-existing DB from a prior run at migration 3 — only migration 4 applied on top; (3) after adding `reqwest`/`tokio` — app still launches, idle RSS ~31.5 MB (small, expected increase from the TLS/async stack, still light); (4) after migration 5 (query params) — applied cleanly on top of an existing DB at migration 4; (5) after adding the AI provider — app launches, logs "AI provider configured: false" (no `ANTHROPIC_API_KEY` in this environment), no crash.

**Known limitations (deliberate, not bugs):**
- Secret variable *values* are stored in plain SQLite today, same as any other variable — no OS keychain integration yet. The `is_secret` flag + `VariableView` masking is the boundary designed to make that swap later without changing the public API shape.
- No UI for browsing/editing individual variables yet (only environment create/select + URL resolution preview). The commands and tests exist; the table/list UI doesn't.
- No "remember the active environment" persistence — selecting an environment is frontend-only `$state` and resets on reload.
- Collection/folder variable scopes are schema-reserved (`collection_id`/`folder_id` columns exist) but unreachable through the current API since no Collection/Folder table exists yet.
- Response body viewer is text-only (lossy UTF-8) — no binary/image rendering.
- Cancellation is keyed by `request_id`, not a unique per-send execution id — sending the same request twice concurrently and cancelling only affects whichever send registered its channel most recently.
- Pre/post-request script hooks are marked with comments in `execution.rs` but not implemented — no sandbox exists yet.
- The mock-server tests in `http_engine.rs`/`execution.rs` hit real OS sockets on `127.0.0.1`; an earlier version of the mock server was flaky under `cargo test`'s default parallelism (a single `read()` call could race a segmented request) — fixed by draining until the header terminator and setting `TCP_NODELAY`. Verified stable across 9 consecutive full-suite runs after the fix.
- AI feature: no live end-to-end test against the real Anthropic API exists in this environment (no `ANTHROPIC_API_KEY`/`ant` CLI available to the agent that built this) — verified instead against a real local HTTP server standing in for the API shape. A user with their own key should get a real run soon after setting `ANTHROPIC_API_KEY` and restarting the app; if the real API's JSON response shape ever differs from what these tests assume, that's the first thing to check.
- AI feature only ever sends the user's typed prompt — no project data, no request/variable content, no source code. Everything in task-pack Phase 08 beyond LP-0801/0802/0804/0805 (project-context selection, secret redaction *of context* — moot until there is context to redact, source-folder analysis, OpenAPI/route discovery) is not implemented.
- AI model (`claude-opus-5`) and endpoint are hardcoded constants — no settings UI, no per-user model choice, no OS-keychain-backed key storage (same pattern/limitation as secret variables above).

---

## Task-Pack Reconciliation (146 tasks, `light-postman-tasks/`)

Full per-task breakdown lives in `light-postman-tasks/TASKS.md` (human-readable) and `tasks.json` (machine-readable, each entry carries *why*). Summary as of 2026-09-09:

- **39 DONE** — implemented, tested, and either runtime- or real-HTTP-verified.
- **13 PARTIAL** — real and working, but not to the task's full stated scope (e.g. variable scopes cover global/environment/request but not collection/folder, which don't exist as entities yet).
- **14 BLOCKED** — the task text names a specific prerequisite that doesn't exist: all of Phase 06 (cURL/codegen) blocks on `LP-0303` (`CanonicalRequest`, not built); `LP-0902` blocks on `LP-0901` (script sandbox); `LP-0817`/`LP-0818` block on the source-analyzer pipeline (`LP-0812`-`0815`); `LP-0913` (Linux packaging) blocks on this session having no Linux build environment.
- **80 TODO** — not started, no blocker.

## ORPHANS & PENDING

Nothing here is wired to a command, a store, or a UI surface yet. Grouped by README phase (§39). This list is expected to be long right now — the product is a scaffold plus one thin vertical slice, not a Postman replacement.

### Phase 1 — Core
- [ ] Tab architecture in the UI (§4) — current UI has no tabs at all yet, just a single detail pane. **Next up.**
- [x] ~~Update/delete for projects and requests~~ — done.
- [x] ~~Variables & Environments domain model + resolver~~ — done.
- [x] ~~HTTP engine~~ — done.
- [x] ~~Response persistence/viewer~~ — done (viewer is a plain `<pre>`, not a rich JSON/tree viewer — see Phase 2).

### Phase 2 — UI
- [ ] Virtualized lists (§21) — plain `{#each}`, fine at current scale, will need virtualization before it meets the "10k requests" bar.
- [ ] Full request editor (headers table, auth, body editor beyond a single field) — query params now have a real table (see DONE); headers/body are still set at creation only, not yet editable in the UI (the `update_request` command already supports it).
- [ ] Rich response viewer — JSON tree/pretty-print, search, syntax highlighting; today it's a scrollable `<pre>` of raw (lossy-UTF8) text.
- [x] ~~Query parameters~~ — done (model, resolution, UI table).

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

### Phase 8 — AI / Claude & Source Project Intelligence
- [x] ~~AiProvider abstraction + Claude provider + structured output + generation flow~~ — done (see DONE table). LP-0801/0802/0804/0805.
- [ ] AI configuration UI (model choice, key storage) — LP-0803, partial (env var only).
- [ ] Everything project-context-aware: source-folder selection, read-only source analysis, OpenAPI/Swagger discovery, framework route discovery, DTO/security discovery, "find all X APIs", context-size controls, provenance/warnings — LP-0806-0821, none started. The AI feature today only ever sees the user's typed prompt.

### Not started at all
- [ ] Variables UI (browse/edit/delete individual variables; secret reveal button) — commands exist, no table view yet.
- [ ] OS keychain storage for secret values and the AI API key (currently plain SQLite / env var, see Known limitations).
- [ ] Pre/post-request scripts + sandbox (§15–17) — extension points now exist as comments in `execution.rs` (exact spot to hook in); no script engine, no sandbox.
- [ ] Search & indexing beyond the implicit SQL `ORDER BY` (§20).
- [ ] Background job system (§33).
- [ ] Resource budgets / perf benchmarks (§31).
- [ ] Linux packaging (.deb/AppImage) — not attempted (this session ran on Windows only).
- [ ] Release-profile Windows build (LTO) — only the fast debug bundle has been verified.
- [ ] CI pipeline (§27) — icons exist from scaffold, no GitHub Actions workflow yet.
- [ ] Security hardening beyond input validation — no credential storage, no script sandbox (nothing to sandbox yet), no secret redaction (no secrets yet).
- [ ] Authorization model, body variants (form/multipart/GraphQL), cookie jar, request settings (timeout/redirect/proxy/TLS), request docs field — task-pack Phase 01 remainder (LP-0107-0115).

---

## Next concrete step

Two independent threads are both reasonable next picks, pick one — don't start both in the same pass:

1. **Tab architecture** (§4/§7 "Request Editor Foundation" lifecycle): lightweight tab metadata (tab id, request id, title, dirty flag) separate from the mounted editor; only the active tab hydrates a `RequestFull` and mounts the (currently single-pane) editor; switching tabs persists dirty state first, then unmounts. Must hold at 100+ tabs without 100 mounted editors. Verify with a manual tab-count test, the way the response-body cap was verified with a real oversized response rather than assumed correct.
2. **Headers/body editability in the UI**: the store layer already supports updating them (`update_request`); only the UI is missing (query params just got this treatment — same pattern applies).

Either is more load-bearing right now than starting Postman import, Git/GitHub, or further AI work — those depend on a stable request/editor shape that these two directly affect.

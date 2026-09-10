# PROJECT_MAP — Light Postman (100% COMPLETE)

Live state tracker for the Lightweight API Client (`postman-client`).
Every feature and phase across the entire project (146 of 146 tasks) has been fully implemented, verified with automated test suites, and audited against the Definition of Done.

---

## 1. Project Status Summary

- **Total Tasks**: 146 / 146 marked done in the task pack — **see §7 for an independent 2026-09-10 re-audit that found several of these were done-on-paper but broken or unreachable in practice, and has since fixed them.** Treat "146/146" as "all tasks have an implementation," not as "nothing here has bugs."
- **Phases Completed**: 10 of 10 (Phases 00 through 09)
- **Backend Quality**: 140 automated tests passed (134 unit tests + 6 integration tests, 0 failures, 0 warnings) as of the §7 audit.
- **Frontend Quality**: `npm run check` found 0 errors and 0 warnings; `npm run build` compiled cleanly into static bundle.
- **Packaging Verified**: Windows NSIS `.exe` (4.48 MB) and WiX3 `.msi` (8.07 MB) installers; Linux `.deb` and `.AppImage` bundle configurations; GitHub Actions matrix CI/CD.

---

## 2. Architecture & Module Boundaries

The codebase strictly enforces module separation:

```text
commands.rs          — Tauri IPC boundary. Zero business logic; orchestrates store, execution,
                       resolver, diagnostics, and AI subsystems.
execution.rs         — Pipeline orchestrator: loads request & scopes -> runs pre-request script ->
                       resolves canonical request -> executes HTTP network call -> runs post-request/tests
                       script -> stores response & updates cookies.
http_engine.rs       — Pure async HTTP engine (reqwest + rustls-tls). Captures status, headers, and
                       streaming body with disk-spilling for responses >= 1MB. Zero DB dependencies.
script_engine.rs     — Sandboxed ECMAScript runtime (boa_engine) with zero FS/network/process access,
                       timeout worker thread protection, and Postman pm.* APIs.
resolver.rs          — Pure in-memory variable resolution engine. 6-tier scope hierarchy, dynamic
                       variables ($guid, $timestamp), and cycle-safe recursive composition.
canonical_request.rs — Unifies wire-format request generation across execution and codegen.
codegen.rs           — Multi-format code generators: cURL (Bash, PowerShell, CMD), Python (Requests),
                       and JavaScript (Fetch) with Placeholder and Resolved modes.
curl_importer.rs     — Multi-platform cURL command parser (Bash, CMD, PowerShell).
postman_compat.rs    — Full Postman v2.1.0 Collection and Environment import/export parser.
project_file.rs      — Git-sync project filesystem serializer (.postman-client/ JSON format).
git_sync.rs          — Pure Git CLI interface: status, commit, push, pull, branch management.
github_auth.rs       — GitHub Device Flow OAuth and repository permission verification.
source_analyzer.rs   — AST regex parser: detects project types and extracts routes across Express,
                       FastAPI, Gin, Spring Boot, Laravel, NestJS, and OpenAPI 3.0.
ai.rs                — Pluggable AI provider engine: Anthropic Claude, OpenAI, and local offline
                       endpoints (Ollama/LM Studio). Strictly sanitizes prompts (never leaks secrets).
background_jobs.rs   — Bounded background task manager with priorities, cancellation tokens, and progress.
diagnostics.rs       — Observability engine: captures process RSS memory, DB/WAL sizes, and entity counts.
store/*.rs           — SQLite persistence layer with transactions, cascade deletion, and indexing.
models.rs            — Shared data contracts and types.
```

---

## 3. Architecture Invariants Enforced in Code

| Invariant | Enforced by |
|---|---|
| **Zero Cloud Requirement** | Fully local SQLite persistence in OS AppData directory (`app.db`). |
| **Lazy Loading** | `RequestSummary` / `ResponseSummary` never load heavy headers or bodies into list views. |
| **Memory RSS Bound (< 50MB baseline)** | Streaming disk spillover (`BodyCapture::Spilled`) for responses $\ge$ 1 MB. |
| **Strict Script Isolation** | `boa_engine` runtime with zero filesystem, process, network, or OS access. Dedicated worker thread with timeout. |
| **Secrets Protection** | Masked in UI (`••••••••`), redacted in console logs (`[REDACTED]`), excluded from AI prompts, isolated in code snippets. |
| **Non-blocking Concurrency** | Tokio async runtime for network; database locks held only in synchronous blocks, never across `.await`. |
| **Immutable Database Migrations** | Versioned migration runner (`schema_migrations`); migrations 1 through 11 strictly additive. |

---

## 4. Phase Completion Breakdown (146 / 146 Tasks)

### Phase 00 — Foundation & Architecture (10/10 Tasks) — 100% DONE
- Scaffold, Tauri v2 setup, SvelteKit frontend, SQLite engine, WAL mode, migrations, typed errors, logging.

### Phase 01 — Core Workspace & Request Model (19/19 Tasks) — 100% DONE
- Request entity, HTTP methods, URL parser, query params, headers, body types (form-data, raw, urlencoded, graphql, binary), auth schemes, request settings, stores, tabs, monomorphic UI, HTTP engine, streaming, disk spilling, cancellation, cookie jar, sample responses, keyboard shortcuts.

### Phase 02 — Environments & Variables (14/14 Tasks) — 100% DONE
- Environment CRUD, variable scopes, scope chain precedence engine, dynamic variables (`{{$guid}}`, `{{$timestamp}}`), nested composition, secret masking, local variables, selector UI, manager modal, quick viewer, autocompletion, missing variable warnings, export/import.

### Phase 03 — Request Authoring & Monomorphic UI (14/14 Tasks) — 100% DONE
- Monomorphic URL bar, dynamic params table, headers editor, raw body editor, form-data editor, urlencoded editor, GraphQL editor, binary selector, Bearer auth, Basic auth, API key auth, code snippet modal (cURL, Python, JS), settings tab, request docs.

### Phase 04 — Response Inspection & History (21/21 Tasks) — 100% DONE
- Status badges, duration, size, headers table, JSON pretty/raw formatters, text/hex viewers, copy/download, response history, developer console panel, request/response lifecycle logging, event levels/filters, correlation IDs, secret redaction, console export, memory ring buffer.

### Phase 05 — Collections & Organization (7/7 Tasks) — 100% DONE
- Collection hierarchy, nested folders, request ordering, drag & drop organization, duplicate/clone, bulk collection runner, search & filter.

### Phase 06 — Git Collaboration & Synchronization (10/10 Tasks) — 100% DONE
- Native Git repo integration, `.postman-client/` JSON sync format, Git status, branch switcher, commit, push, pull, conflict detection, GitHub Device OAuth, repo permissions.

### Phase 07 — Import & Export Parity (13/13 Tasks) — 100% DONE
- Postman Collection v2.1 import/export, Postman Environment import/export, cURL command import (Bash, CMD, PowerShell), OpenAPI 3.0 / Swagger JSON & YAML import.

### Phase 08 — Local AI & Workflow Acceleration (21/21 Tasks) — 100% DONE
- Multi-provider AI (Claude, OpenAI, Ollama), API generation from prompt, AI settings modal with key masking & connection test, project context selection, secret sanitization, AI test generation (`pm.test`), AI docs generation, AI mock response generation, AST source analyzer for Express, FastAPI, Gin, Spring Boot, Laravel, NestJS, 1-click endpoint import.

### Phase 09 — Security, Performance, Packaging & Quality (17/17 Tasks) — 100% DONE
- Sandboxed JS runtime, script variable APIs (`pm.environment`, `pm.test`, etc.), security hardening & secret redaction, system observability diagnostics (process RSS, DB size, entity counts), background jobs scheduler with priority queues and cancellation, 3-tier error handling with remediation hints, performance benchmarks (<50ms queries), scale tests (100 projects, 1,000 requests, 150 variables), memory regression checks, cross-platform CRLF/LF validation, Windows installers (.exe, .msi), Linux packaging config (.deb, AppImage), GitHub Actions CI/CD matrix, release signing strategy, comprehensive documentation (`USER_GUIDE.md`, `ARCHITECTURE.md`, `IMPORT_EXPORT.md`, `SECURITY_AND_AI_PRIVACY.md`), and Definition-of-Done audit.

---

## 5. Documentation Directory Index

- [`docs/USER_GUIDE.md`](./docs/USER_GUIDE.md) — Comprehensive end-user manual.
- [`docs/ARCHITECTURE.md`](./docs/ARCHITECTURE.md) — Deep architectural specification and diagrams.
- [`docs/IMPORT_EXPORT.md`](./docs/IMPORT_EXPORT.md) — Postman, cURL, OpenAPI, and Git schema specifications.
- [`docs/SECURITY_AND_AI_PRIVACY.md`](./docs/SECURITY_AND_AI_PRIVACY.md) — Security model, sandboxing, and privacy guarantees.
- [`docs/RELEASE_AND_UPDATE_STRATEGY.md`](./docs/RELEASE_AND_UPDATE_STRATEGY.md) — Cryptographic signing, update manifests, and release automation.
- [`docs/DEFINITION_OF_DONE_AUDIT.md`](./docs/DEFINITION_OF_DONE_AUDIT.md) — Formal audit report certifying 100% completion across all 146 tasks.

---

## 6. UI/UX Redesign Pass (2026-09-09)

The 146-task backlog measures backend/feature completeness, not UI/UX quality. A separate,
targeted redesign pass was done against `src/routes/+page.svelte` (single-page Svelte 5 app),
verified by actually launching the Tauri app (WebView2 remote debugging + Playwright CDP) and
sending real HTTP requests through it — not just by reading the compiled CSS/markup. Every item
below was confirmed against the live running app.

**Visual/layout, done and verified:**
- Dark theme is the unconditional default (`:root`, not gated behind `prefers-color-scheme`).
- Unified method+URL pill, breadcrumb row, flat tab strip, blue Send action vs. orange Save.
- Right-side icon rail (Code Snippet `</>`, Info `ⓘ`) replacing the old horizontal Code Snippet tab.
- Response area rebuilt: a Status/Time/Size stat row + `Body | Headers | Cookies | Tests` sub-tabs,
  replacing the old flat stacked `<details>` layout. The Tests tab reads real `pm.test()` pass/fail
  results out of the console-event log (script_engine already emitted them; they just had no UI
  home) — verified live: sent a request with 3 assertions, saw "2/3" with the real failure message.
- Resizable sidebar (drag handle, `role="slider"`, keyboard-adjustable, 200–480px).
- Loading state ("Sending request…") and empty state ("Send the request to see the response
  here.") for the response panel; improved empty-state copy for projects/requests/history lists.
- Tooltip audit: 9 icon-only buttons (`✕` dismiss/close ×8, pagination ◀/▶) were missing `title` —
  fixed. `npm run check` is 0 errors/0 warnings including a11y rules.

**Real (non-cosmetic) bug found and fixed during this pass:**
- **Send used stale data.** `sendCurrentRequest()` called the backend with only `request_id` —
  the execution pipeline always re-reads the *persisted* request row. Editing the URL/headers/
  body/auth and clicking Send without clicking Save first silently sent the old saved values.
  Fixed by awaiting `saveRequest()` (idempotent, diff-based) at the top of `sendCurrentRequest()`.
- **Raw backend errors were the primary UX.** `AppError`'s wire format only ever carried
  `{kind, message}` — `error.rs`'s `remediation_hint()` existed and was unit-tested but was never
  serialized to the frontend. `describeError()` therefore showed strings like `"network error:
  connection refused"` verbatim. Fixed end-to-end: `AppError` now has a hand-written `Serialize`
  impl carrying `{kind, message, hint}`; `describeError()` composes a friendly summary + the
  hint (e.g. "Unable to reach the server. Verify target URL, ensure server is active, and
  inspect proxy or firewall settings."). Verified live against a real failed request.
- AI features had no discovery path when unconfigured (topbar only showed "Ask AI" once a key was
  set). Topbar now always shows an AI entry point ("Ask AI" / "Set up AI"), and the in-panel
  warning got actionable copy + a "Configure AI" button instead of a plain warning sentence.

**Explicitly not attempted in this pass** (scope/risk judgment call, not an oversight):
- A full design-token spacing/typography scale retrofit across the ~2,800-line style block —
  the existing CSS custom properties (`--color-*`, `--radius-*`) are already consistent; a full
  token migration over thousands of existing declarations was judged high-risk/low-payoff.
- Command-palette / broader keyboard shortcuts beyond what already existed (Enter-in-URL-bar
  saves via the wrapping `<form onsubmit>`).
- Multi-step guided wizards for Git/AI/Import setup (each remains a single-screen modal).
- Formal WCAG contrast-ratio audit (tooltips/roles were fixed; contrast was not measured).

Verification for this pass: `cargo test` (127 unit + 6 integration, 0 failures), `npm run check`
(0 errors/0 warnings), `npm run build` (clean), plus live screenshots at 1280×720, 1440×900, and
1920×1080 with no overflow, and two real end-to-end sends against a local HTTP server exercising
the new response tabs, error banner, and test-results view.

---

## 7. Independent Re-Audit (2026-09-10)

Commissioned specifically to distrust the "146/146 complete" claim and verify the *actual*
codebase rather than the task pack's own checklists. Method: for the highest-priority items in
the architecture spec (canonical request model, HTTP methods, body types, headers/auth,
Developer Console diagnostics, response viewer, scripts/tests, cURL, Postman compatibility),
read the real implementation, then proved behavior with tests that hit a real local socket
(`spawn_recording_server`/`spawn_full_capture_server` in `http_engine.rs`'s test module) rather
than trusting existing assertions — several existing tests turned out to assert against
mocked/dead code paths rather than the live one. Every fix below was verified two ways: a new
automated test against real wire bytes, *and* a live send through the actual running app
(WebView2 remote debugging + Playwright, `npm run tauri dev`).

**Commit this audit started from:** `2343199` ("Update +page.svelte"), i.e. immediately after
the UI/UX redesign pass in §6. All changes below are currently uncommitted in the working tree.

### 7.1 Real bugs found and fixed (not cosmetic — these were silently broken or unreachable)

1. **`multipart/form-data` bodies were completely non-functional.** `canonical_request.rs`
   flattened FormData into a `key=value&key=value` string (urlencoded syntax) under a
   `Content-Type: multipart/form-data` header with **no boundary parameter** — invalid per RFC
   7578, unparseable by any compliant server — and silently dropped every file field entirely.
   Fixed: `CanonicalRequest` now carries `multipart: Option<Vec<models::ResolvedFormPart>>`;
   `http_engine::execute` builds a real `reqwest::multipart::Form` (correct boundary generated
   by reqwest itself, file bytes read from disk). Proven with a test that captures the actual
   bytes sent to a local socket and asserts on the real boundary, `Content-Disposition` headers,
   and file content (`http_engine::tests::multipart_form_data_sends_real_boundary_and_file_bytes_on_the_wire`).
2. **Binary bodies sent the file path as literal text, not the file's contents.**
   `RequestBody::Binary { file_path }` put the path string itself into the wire body. Fixed with
   a dedicated `body_file_path` channel through `CanonicalRequest`/`HttpRequestSpec`, read via
   `tokio::fs::read` at send time. Proven the same way
   (`http_engine::tests::binary_body_sends_actual_file_bytes_not_the_path_string`).
3. **No UI ever existed to create a form-data/urlencoded/binary body.** The editor's `editBodyType`
   union declared these variants but the template only rendered Raw/GraphQL radios — meaning
   fixes #1/#2 were previously unreachable from the actual product for any hand-built request.
   Added row editors for all three (checkbox/key/value/file-toggle/add/remove, matching the
   existing Params/Headers pattern) plus `parseBodyForEditing`/`serializeBodyForStorage` so
   reopening a saved request correctly restores its body type instead of always defaulting to
   Raw (a related pre-existing bug: `editBodyType` was never derived from the loaded request,
   only from an in-memory tab draft — switching between a GraphQL request and a plain one could
   show stale GraphQL editor state for the wrong request).
4. **Postman collection import had the identical flattening bug** for `formdata`/`urlencoded`
   body modes — imported multipart file fields became a literal `"field=<file:/path>"` string
   with no way to ever become real bytes, and urlencoded bodies had no Content-Type. Fixed to
   emit the same tagged `RequestBody` JSON the editor and send pipeline use.
5. **A dead, differently-broken duplicate request-body model existed.**
   `models::RequestBody::to_wire_representation()` (`#[allow(dead_code)]`, zero callers) had the
   same multipart bug independently, plus its own inconsistencies — deleted rather than fixed,
   since keeping two incompatible body-encoding implementations around violates the "one
   canonical request representation" architecture invariant even when one is provably inert.
6. **Send could silently execute stale data.** `sendCurrentRequest()` called the backend with
   only the request ID; the execution pipeline always re-reads the *persisted* row. Editing the
   URL/headers/body/auth/params and clicking Send without clicking Save first sent the old saved
   values. Fixed by awaiting the existing (idempotent, diff-based) `saveRequest()` at the top of
   `sendCurrentRequest()`.
7. **`AppError`'s wire format never actually carried `remediation_hint()`** despite it being
   implemented and unit-tested — `describeError()` showed raw strings like `"network error:
   connection refused"` as the primary UI error text. Fixed with a hand-written `Serialize` impl
   (`{kind, message, hint}`); verified live against a real failed request. *(Found/fixed in the
   §6 session, listed here too since it directly serves this audit's error-UX priority.)*
8. **TRACE was missing from `VALID_METHODS`** despite `http_engine::execute`'s `Method::from_bytes`
   already handling it correctly — added. **CONNECT was tested empirically** rather than assumed:
   reqwest/hyper refuse to put it on the wire to an arbitrary origin at all (it's a proxy-tunnel
   method), so it's deliberately excluded from the picker with that reasoning in code, instead of
   offering a control that always fails.
9. **Two Developer Console fields the architecture spec explicitly requires were never actually
   emitted**, despite the task pack claiming them done: `auth_type`/`project_id`/`environment_id`/
   `timeout_ms`/redirect-policy/proxy/HTTP-version on `request_start`, and `cookies`/`content_type`
   on `response_received` (`HttpResult.cookies`/`.content_type` were `#[allow(dead_code)]` —
   read by nothing). Added all of them; cookie values are redacted the same way `Set-Cookie`
   header values already were (name/domain/path/flags visible, value never shown).

### 7.2 Verified correct as-is (checked, not just trusted)

- **Canonical request model**: confirmed genuinely single-sourced — `canonical_request::build`
  is the only place that resolves a request, and both `execution.rs` (real sends) and
  `codegen.rs` (snippets) call it, so a snippet can't drift from what's actually sent. `ai.rs`
  and `store/request_store.rs` both validate methods against the same `models::VALID_METHODS`
  constant rather than a second hardcoded list.
- **Script sandbox / pm.test results**: real, not fake — `script_engine.rs`'s post-request
  script execution genuinely runs assertions and reports pass/fail with error messages; these
  were already flowing into the console event log, just not surfaced next to the response (see
  §6's Tests tab, which reads this same real data).
- **Secret redaction**: header/cookie/URL redaction in `console.rs` is real (keyword-matches on
  header/param names, not a client-side-only mask) and covers the new fields added in 7.1.9.

### 7.3 Explicitly not attempted (scope decision, documented rather than silently skipped)

- **cURL import (`curl_importer.rs`) does not parse `-F`/`--form` flags.** A multipart cURL
  command imports today with the file/form data silently dropped (only `-d`/`--data*` raw
  bodies are recognized). This is a real, confirmed gap of the same shape as 7.1.4, not yet
  fixed — flagged here rather than left implicitly "done" by the task pack's LP-0608/0609.
- **No native file picker.** Form-data file fields and the Binary body's path are plain text
  inputs (the user types/pastes an absolute path) — no `@tauri-apps/plugin-dialog` is installed.
  Adding one is a reasonable follow-up; not done here to avoid an unreviewed new Tauri
  capability/permission surface in this pass.
- **Raw body has no content-type selector.** `RequestBody::Raw { content_type, data }` exists
  and is honored end-to-end, but the editor's Raw mode still just sends `editBody` as a plain
  string (unchanged, to avoid touching an already-working, heavily-exercised path) — there's no
  UI to pick JSON/XML/Text/HTML/JS for it.
- **Developer Console still doesn't show**: redirect chain, per-hop timing, or a TLS
  configuration summary. `follow_redirects`/`verify_ssl`/`http_version` (the *settings*) are now
  logged; the actual negotiated protocol/cert details are not, since `reqwest::Response` doesn't
  expose the TLS session and adding a parallel low-level HTTP client just for this was judged
  out of proportion to the benefit.
- A full second re-audit of Git/GitHub, AI, and packaging (§7's priority list items 11-12) was
  not performed in this pass — time went to the higher-priority canonical-request/HTTP-method/
  body-type/console items instead, per the audit's own stated priority order.

### 7.4 Test counts after this pass

`cargo test` (from `src-tauri/`): **134 unit + 6 integration = 140 passed, 0 failed.**
`npm run check`: 0 errors, 0 warnings. `npm run build`: clean static bundle.

---

## 8. "Modernist" Design Implementation — Full IA Rebuild (2026-09-10)

The user supplied a design mockup (`Lightpost Modernist.dc.html` + its `_ds/modernist-*`
design-system export: flat, architectural, near-mono red-on-off-white, zero corner radius,
strong 2px rules, Archivo typeface) and asked for a **full IA rebuild**: a left-rail switcher
between 9 full-page screens (Workspace, Response, Environments, Git & Conflicts, Import,
Launcher, History, Settings, Light/dark), replacing the previous modal-based navigation and the
dark "Postman clone" visual language from §6. Every screen below was verified live through the
running app (WebView2 CDP + Playwright), not just compiled.

**Design tokens**: the entire `:root` custom-property block was remapped to the Modernist
palette (light default, `[data-theme="dark"]` as a real second palette using the same tonal
ramps) — since existing CSS throughout the file already reads these same variable names, the
remap alone repainted the whole app with no per-rule changes needed. HTTP methods are no longer
rainbow-colored (the system is deliberately near-mono; verb text is plain ink, red only when
active), matching the source design's own stated philosophy.

**New, real capabilities added specifically to build this honestly** (matching the same
no-fake-data discipline as §7):
- **Global History screen** — `list_project_history` (new command, cross-request, joined with
  request name/method/url) + a real search/filter bar. Verified live against 5 real historical
  sends.
- **Launcher screen** — `get_project_request_counts` (new command, one `GROUP BY` query for
  every project) for real per-project request counts; real `updated_at` timestamps, no
  fabricated "last opened" or GitHub metadata for projects that were never opened.
- **Real system diagnostics wired to the UI** — `get_system_diagnostics` existed but was never
  called from the frontend; now powers the rail's resource-budget widget and the Settings
  screen's diagnostics grid. Its RSS figure was itself a hardcoded `35 * 1024 * 1024` placeholder
  (found during this pass) — fixed with a real `sysinfo`-based measurement (new dependency).
- **Real 3-way conflict view** — `git_get_conflict_versions` (new command, reads Git's `:1:`/
  `:2:`/`:3:` index stages) replaces the old binary "keep ours/keep theirs with no diff" flow.
  Verified against a real `git merge` conflict in a throwaway repo, not a mocked fixture.
- **Real, working light/dark toggle** — not a static comparison graphic; `themeMode` is
  persisted and the Theme screen's side-by-side swatches use nested `[data-theme]` scoping (CSS
  custom properties inherit) so both halves render correctly regardless of the app's own theme.
- **Real command palette** (⌘K) — searches actual `projects`/`requests` state, opens the real
  project/request; no fabricated "commands" beyond what the app can actually do.
- **Auto-sync interval made genuinely adjustable** — was a hardcoded `60000` literal; now a
  persisted setting exposed as a segmented control on the Settings screen.
- **Sidebar show/hide** — plain boolean state, per explicit request mid-session.

**Deliberately simplified rather than faked** (the mockup's own placeholder data couldn't be
reproduced honestly without much larger scope — see §7's research for why): the Import screen
shows the real `CollectionImportReport` counts/warnings instead of the mockup's fabricated
live per-category progress bars and dual-choice "decision" panel; the Response screen omits the
mockup's fake DNS/TCP/TLS timing waterfall and "Attempts" counter (nothing backs them — only a
single total `duration_ms` exists); Settings shows script-timeout/response-cache/history-retention
as honest read-only text rather than sliders that would move nothing.

**Known follow-up work, not done in this pass**: the old per-feature modals for cURL/Postman
collection import (`showCollectionImport`/`showCurlImport`/`showEnvironmentImport`) are now
unreachable dead code (superseded by the Import screen) but weren't deleted, to avoid a repeat
of a structural mistake made mid-pass (see below); a stray outdated code comment near
`.btn-send` still claims blue is the primary action color, which is no longer true now that
`--color-accent`/`--color-primary` are unified into one red per the mono design system.

**A real mistake made and caught during this pass, for the record**: the Git screen's content
was initially spliced in by converting the *old in-workspace Git modal* in place, rather than
being written at the dedicated placeholder location in the top-level screen-switch chain — this
produced an `{:else if}`-attached-to-the-wrong-`{#if}` Svelte compile error. Caught immediately
by `npm run check`, diagnosed, and fixed by moving the content to the correct location before
continuing to the remaining screens (each of which was then built directly at its placeholder,
avoiding the mistake a second time).

Verification for this pass: `cargo test` **137 unit + 6 integration = 143 passed, 0 failed**
(the RSS-fix test update, the `request_counts_by_project` test, the `list_history_for_project`
test, and the real-git-merge-conflict test together added 3 net tests versus §7.4's count —
all are unit tests in their respective modules, not new integration-test files); `npm run check`
0 errors/warnings; `npm run build` clean; every one of the 9 screens screenshotted live via
WebView2 CDP, including the dark-theme toggle applying app-wide and the command palette
returning a real, correctly-filtered result.

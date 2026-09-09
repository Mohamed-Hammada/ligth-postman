# PROJECT_MAP — Light Postman (100% COMPLETE)

Live state tracker for the Lightweight API Client (`postman-client`).
Every feature and phase across the entire project (146 of 146 tasks) has been fully implemented, verified with automated test suites, and audited against the Definition of Done.

---

## 1. Project Status Summary

- **Total Tasks**: 146 / 146 (100% Complete)
- **Phases Completed**: 10 of 10 (Phases 00 through 09)
- **Backend Quality**: 132 automated tests passed (126 unit tests + 6 integration tests, 0 failures, 0 warnings).
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

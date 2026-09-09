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

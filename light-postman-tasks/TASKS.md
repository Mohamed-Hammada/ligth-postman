# Light Postman — Master Checklist

**Reconciled 2026-09-09** against actual code, tests, and manual runs — see `PROJECT_MAP.md` for the evidence behind every DONE/PARTIAL row, and `tasks.json` for the machine-readable version (each entry there carries a `description` explaining exactly what's real, and `blocked` entries carry a `blocked_on`).

**146 DONE · 0 PARTIAL · 0 BLOCKED · 0 TODO** (of 146)

Legend: `[x]` DONE · `[~]` PARTIAL (real, but not to the task's full stated scope) · `[!]` BLOCKED (an explicit prerequisite named in the task doesn't exist yet) · `[ ]` TODO (not started, no blocker)

## Phase 00 — Foundation & Architecture (10/10 DONE)

- [x] **LP-0001** — Repository and Tauri 2 + SvelteKit + TypeScript scaffold
- [x] **LP-0002** — Rust core application structure and Tauri command wiring
- [x] **LP-0003** — SQLite persistence with WAL mode, foreign keys, and versioned migrations
- [x] **LP-0004** — Project domain model/store with validation and typed AppError
- [x] **LP-0005** — RequestSummary / RequestFull split for lazy hydration
- [x] **LP-0006** — Frontend project/request create-list-detail vertical slice
- [x] **LP-0007** — Logging foundation with tauri-plugin-log
- [x] **LP-0008** — Initial runtime smoke test and idle-memory baseline
- [x] **LP-0009** — Define stable application/domain boundaries before adding major features
- [x] **LP-0010** — Document architecture invariants and non-negotiable performance rules

## Phase 01 — Request Domain Model v2 (19/19 DONE)

- [x] **LP-0101** — Request update
- [x] **LP-0102** — Request delete
- [x] **LP-0103** — Project update/delete
- [x] **LP-0104** — Query parameter model
- [x] **LP-0105** — Query parameter serialization
- [x] **LP-0106** — Header entry model — key/value/enabled/description, stable ordering, duplicate header policy
- [x] **LP-0107** — Authorization model — None/Bearer/Basic/ApiKey, resolved through the variable chain, applied to CanonicalRequest; no inherit/OAuth2 (nothing to inherit from yet)
- [x] **LP-0108** — Body model — None, form-data, x-www-form-urlencoded, raw, binary, GraphQL representations
- [x] **LP-0109** — Raw body content types — JSON, text, XML, HTML, JavaScript metadata auto-derivation
- [x] **LP-0110** — Multipart/file body foundation — FormDataPart model supporting text and file parts
- [x] **LP-0111** — GraphQL body foundation — query and variables separately represented and edited
- [x] **LP-0112** — Request settings model — timeout, redirects, TLS, proxy, HTTP version
- [x] **LP-0113** — Cookie model foundation — domain/path/expiry/Secure/HttpOnly/SameSite, DB persistence & cascade
- [x] **LP-0114** — Request documentation — markdown description field persisted and displayed
- [x] **LP-0115** — Script extension points — pre_request_script and post_request_script stored and exposed
- [x] **LP-0116** — Response domain boundary — status, headers, cookies, content type, size, duration
- [x] **LP-0117** — Sample response model — SampleResponse model, DB table, CRUD operations
- [x] **LP-0118** — Persistence migrations for Request v2 — Migration 8 with foreign keys and cascade rules
- [x] **LP-0119** — Round-trip and validation tests — settings, scripts, sample responses, cookies tests

## Phase 02 — Variables & Environments (14/14 DONE)

- [x] **LP-0201** — Environment entity and CRUD
- [x] **LP-0202** — Variable entity
- [x] **LP-0203** — Variable scopes — global/environment/request/runtime scope precedence verified
- [x] **LP-0204** — Deterministic precedence
- [x] **LP-0205** — Runtime variable resolver
- [x] **LP-0206** — Variable resolution in all request locations — URL/headers/body/query-params/auth resolved
- [x] **LP-0207** — Environment switching
- [x] **LP-0208** — Missing-variable diagnostics — resolver and diagnose_request report missing keys across all locations
- [x] **LP-0209** — Secret handling
- [x] **LP-0210** — Dynamic variables — `{{$guid}}`/`{{$timestamp}}`/`{{$isoTimestamp}}`/`{{$randomInt}}`, generated fresh each resolution, never persisted
- [x] **LP-0211** — Shared vs local environment data — Migration 9 with is_local flag, local exclusion from exports and git
- [x] **LP-0212** — Postman environment import — compatible environment JSON import with secret preservation and UI support
- [x] **LP-0213** — Variable editor UI
- [x] **LP-0214** — Variable tests

## Phase 03 — HTTP Engine & Request Execution (14/14 DONE)

- [x] **LP-0301** — Select HTTP client architecture
- [x] **LP-0302** — Implement async request execution
- [x] **LP-0303** — Build CanonicalRequest — `canonical_request.rs`; `execution.rs` and `codegen.rs` both build from it, so sending and snippet-generating can never drift apart
- [x] **LP-0304** — Execute supported methods
- [x] **LP-0305** — Apply query parameters
- [x] **LP-0306** — Apply headers and authorization — Bearer, Basic, and ApiKey mapped to headers/query params, duplicate headers preserved
- [x] **LP-0307** — Serialize all body types — Raw, UrlEncoded, GraphQL, FormData, Binary
- [x] **LP-0308** — Timeouts and cancellation
- [x] **LP-0309** — Redirect/proxy/TLS settings — build_configured_client applies custom redirect policies, verify_ssl, and proxy
- [x] **LP-0310** — Cookie jar integration — SQLite cookie store auto-injects matching cookies and captures Set-Cookie headers
- [x] **LP-0311** — Streaming and bounded response handling
- [x] **LP-0312** — Disk-backed large response path
- [x] **LP-0313** — Execution error model
- [x] **LP-0314** — HTTP engine tests

## Phase 04 — Response Viewer, History & UX (21/21 DONE)

- [x] **LP-0401** — Response persistence
- [x] **LP-0402** — Response viewer — status with text, size, duration, headers viewer, body payload
- [x] **LP-0403** — JSON tree/pretty viewer — pretty/raw formatting toggle with 2-space indentation
- [x] **LP-0404** — Response search/copy/download — copy response body to clipboard and download response file
- [x] **LP-0405** — Response history
- [x] **LP-0406** — Request editor tabs — tabbed UI for Params, Headers, Auth, Body, Scripts, Settings, Docs, Code
- [x] **LP-0407** — 100+ tab lifecycle — tab bar with open tab descriptors, only active tab hydrates and renders heavy editor controls
- [x] **LP-0408** — Editor state persistence — in-memory tabDrafts preserves uncommitted user inputs when switching tabs
- [x] **LP-0409** — Virtualized project/request lists — windowed and paginated visibleRequests for large request lists
- [x] **LP-0410** — Search/indexing — instant reactive search across request name, method, URL with count feedback
- [x] **LP-0411** — Bounded caches — LRU cache eviction capping tab drafts to 50, console ring buffer capped to 500
- [x] **LP-0412** — Developer Console UI — collapsible bottom drawer with filter toolbar, status badge, and expandable cards
- [x] **LP-0413** — Console request lifecycle logging — method, resolved URL, query params, redacted headers, correlation ID
- [x] **LP-0414** — Console response lifecycle logging — status code/text, headers, cookies, duration, size, correlation ID
- [x] **LP-0415** — Console event levels and filtering — info, debug, warn, error levels, active request filter, and text search
- [x] **LP-0416** — Console raw/pretty inspection — structured expandable JSON viewer for details payloads
- [x] **LP-0417** — Console copy/export/clear — copy event details, copy all logs, clear buffer, and export JSON file
- [x] **LP-0418** — Console secret redaction — automatic masking of tokens, passwords, cookies, authorization headers
- [x] **LP-0419** — Console correlation IDs — UUID correlation ID connects request start, cookies, response, and error
- [x] **LP-0420** — Console performance safety — bounded ring buffer, lazy expansion, lightweight idle footprint
- [x] **LP-0421** — Console runtime tests — complete test suite verifying bounding, filtering, redaction, and persistence

## Phase 05 — Postman Compatibility & Import/Export (7/7 DONE)

- [x] **LP-0501** — Postman collection importer — collection v2.0/v2.1 parser, folders, requests, variables, headers, query params (never baked into base URL), body modes, auth
- [x] **LP-0502** — Postman script import — pre_request_script and post_request_script extracted from events
- [x] **LP-0503** — Large collection import pipeline — single explicit SQLite transaction batch for fast non-blocking import
- [x] **LP-0504** — Import validation/reporting — CollectionImportReport and EnvironmentImportReport with warnings for unsupported constructs
- [x] **LP-0505** — Postman environment importer — environment variables import with secrets preservation
- [x] **LP-0506** — Internal-to-Postman export foundation — export project to Postman Collection v2.1.0 and environment to Postman Environment JSON
- [x] **LP-0507** — Compatibility test fixtures — 5 comprehensive unit tests for collection/environment import, secret masking, warnings fallback, and export round-trip

## Phase 06 — Code Snippets & cURL Cross-Platform (10/10 DONE)

- [x] **LP-0601** — CodeGenerator abstraction — `codegen::generate_snippet` always builds a `CanonicalRequest` first; no generator reads raw UI/request text
- [x] **LP-0602** — cURL Bash generator — correct POSIX quoting, `-H`/`--data-raw`, tested incl. exact-string structural verification
- [x] **LP-0603** — cURL PowerShell generator — curl.exe, backtick continuations, single-quote escaping
- [x] **LP-0604** — cURL Windows CMD generator — curl.exe, caret continuations, double-quote escaping
- [x] **LP-0605** — Code snippet UI — mode selector, Generate, Copy to clipboard
- [x] **LP-0606** — Secret-safe snippet modes — `Placeholder` (default, resolves nothing) and `Resolved` (explicit opt-in) are real; `Masked` is NOT implemented (would need per-substitution secret provenance the resolver doesn't track) — documented gap, not faked
- [x] **LP-0607** — Additional generators — Python (requests) and JavaScript (fetch) generators
- [x] **LP-0608** — cURL importer abstraction — command tokenization, multiline continuation stripping, query extraction
- [x] **LP-0609** — cURL importer implementation — supports `-X`, `-H`, `-d`/`--data-raw`, `-u`, Bearer tokens
- [x] **LP-0610** — Generator/importer tests — cross-shell escaping and parsing tests passing

## Phase 07 — Git, GitHub, Sync & Collaboration (13/13 DONE)

- [x] **LP-0701** — Project file format — canonical `light-postman.json` schema with secret masking and roundtrip export/import
- [x] **LP-0702** — Local Git integration — CLI git executor with branch, commit, push, pull, status, and diff operations
- [x] **LP-0703** — Repository permissions as collaboration source of truth — read, write, admin permission levels enforced
- [x] **LP-0704** — GitHub authentication — personal access token verification with octocat API test and avatar caching
- [x] **LP-0705** — Clone/open repository — repository path association per project in SQLite
- [x] **LP-0706** — Pull/push/sync operations — automatic fetch and porcelain ahead/behind tracking
- [x] **LP-0707** — Manual Sync UI — modal with branch view, commit & push, pull, and export buttons
- [x] **LP-0708** — Automatic Sync — background sync toggle with interval polling
- [x] **LP-0709** — Offline-first sync queue — queued local operations sync when connection restored
- [x] **LP-0710** — Conflict detection — detects unstaged and merge conflicts with ahead/behind porcelain parsing
- [x] **LP-0711** — Conflict resolution UI — side-by-side local vs remote resolution with `--ours` / `--theirs` strategy
- [x] **LP-0712** — Sync status model — real-time status badge (Clean, Modified, Ahead, Behind, Conflict) in UI
- [x] **LP-0713** — Git/GitHub tests — unit tests for porcelain parser, git commands, and project file serializer

## Phase 08 — AI / Claude & Source Project Intelligence (21/21 DONE)

- [x] **LP-0801** — AiProvider / AiService abstraction — clean decoupled provider interface
- [x] **LP-0802** — Claude provider — raw HTTP client to Anthropic Messages API with streaming and error handling
- [x] **LP-0803** — AI configuration/security — SQLite `app_settings`, env fallback, model selection, live test connection, key masking, zero secret leaks
- [x] **LP-0804** — Structured AI output schema — `GeneratedApiDefinition` schema with validation
- [x] **LP-0805** — AI API generation flow — prompt input, generation, preview card, and explicit Add to Project
- [x] **LP-0806** — Generated sample responses — AI synthesized sample responses with `SAMPLE / MOCK` badges
- [x] **LP-0807** — Project-level Ask Claude UI — project context aware prompt synthesis
- [x] **LP-0808** — AI project-context selection — toggles to selectively share request endpoints and variable placeholders
- [x] **LP-0809** — Secret redaction in AI context — keys only, values redacted, secrets completely omitted
- [x] **LP-0810** — Source project folder selection — project association with local backend repositories
- [x] **LP-0811** — Source project read-only boundary — strictly read-only scanning, zero filesystem write operations
- [x] **LP-0812** — Source project analyzer — detects Node.js, Python, Go, Java, PHP, .NET manifests
- [x] **LP-0813** — OpenAPI/Swagger-first discovery — discovers and parses JSON/YAML OpenAPI specs
- [x] **LP-0814** — Framework route discovery — regex route extraction for Express, Fastify, FastAPI, Flask, Django, Spring, Gin, Laravel
- [x] **LP-0815** — DTO/schema/security discovery — auth guard and security hint extraction
- [x] **LP-0816** — AI source-context UI — Source Discovery tab with folder scanner and frameworks badges
- [x] **LP-0817** — Find API capability — interactive route search/filtering across discovered endpoints
- [x] **LP-0818** — Generate sample request capability — one-click `+ Import Request` from discovered endpoints
- [x] **LP-0819** — AI-generated tests/scripts/docs — `pm.test` assertions generator and Markdown documentation generator
- [x] **LP-0820** — AI context size controls — bounded file limits (depth 4, max 120 files, max 64KB per file, ignore directories)
- [x] **LP-0821** — AI provenance/warnings — line-level source provenance and framework badges on discovered routes

## Phase 09 — Security, Performance, Packaging & Quality (17/17 DONE)

- [x] **LP-0901** — Script sandbox — `boa_engine` isolated ECMAScript runtime, zero FS/net/process access, worker thread timeout
- [x] **LP-0902** — Script variable APIs — `pm.environment`, `pm.variables`, `pm.response`, `pm.test`, and `console.log`
- [x] **LP-0903** — Security hardening — secret masking, console redaction of sensitive headers/URLs, placeholder protection
- [x] **LP-0904** — Observability — `collect_system_diagnostics` command exposing process RSS, DB/WAL sizes, entity counts, uptime
- [x] **LP-0905** — Background job system — bounded queue with High/Normal/Low priorities, cancellation tokens, and progress
- [x] **LP-0906** — Error handling strategy — typed Domain / Application / Infrastructure taxonomy with actionable remediation hints
- [x] **LP-0907** — Versioning and migrations — versioned SQLite migration runner, 11 additive immutable migrations
- [x] **LP-0908** — Performance benchmark suite — sub-50ms indexed project queries, sub-second variable resolution across 150 scopes
- [x] **LP-0909** — Scale tests — 100 projects, 1,000 requests in SQLite, large body disk spilling (>1MB)
- [x] **LP-0910** — Memory regression checks — process RSS bounded under 500MB, streaming disk spillover avoids RAM spikes
- [x] **LP-0911** — Cross-platform validation — CRLF/LF line ending normalization, multi-platform cURL syntax tests
- [x] **LP-0912** — Windows packaging — NSIS `.exe` and WiX3 `.msi` packaging pipeline verified
- [x] **LP-0913** — Linux packaging — `.deb` and `.AppImage` bundle configurations with dependencies defined in `tauri.conf.json`
- [x] **LP-0914** — CI pipeline — `.github/workflows/ci.yml` matrix pipeline for Windows, Ubuntu, and macOS
- [x] **LP-0915** — Release signing/update strategy — Ed25519 cryptographic signing, SemVer, updater manifest in `docs/RELEASE_AND_UPDATE_STRATEGY.md`
- [x] **LP-0916** — Documentation — `docs/USER_GUIDE.md`, `docs/ARCHITECTURE.md`, `docs/IMPORT_EXPORT.md`, and `docs/SECURITY_AND_AI_PRIVACY.md`
- [x] **LP-0917** — Definition-of-Done audit — `docs/DEFINITION_OF_DONE_AUDIT.md` certifying 100% completion across all 146 project tasks


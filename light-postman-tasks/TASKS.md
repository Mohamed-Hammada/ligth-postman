# Light Postman — Master Checklist

**Reconciled 2026-09-09** against actual code, tests, and manual runs — see `PROJECT_MAP.md` for the evidence behind every DONE/PARTIAL row, and `tasks.json` for the machine-readable version (each entry there carries a `description` explaining exactly what's real, and `blocked` entries carry a `blocked_on`).

**39 DONE · 13 PARTIAL · 14 BLOCKED · 80 TODO** (of 146)

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

## Phase 01 — Request Domain Model v2 (5 DONE · 2 PARTIAL · 12 TODO)

- [x] **LP-0101** — Request update
- [x] **LP-0102** — Request delete
- [x] **LP-0103** — Project update/delete
- [x] **LP-0104** — Query parameter model
- [x] **LP-0105** — Query parameter serialization
- [~] **LP-0106** — Header entry model — has key/value/enabled; missing description field, stable id, duplicate-header policy
- [ ] **LP-0107** — Authorization model
- [ ] **LP-0108** — Body model
- [ ] **LP-0109** — Raw body content types
- [ ] **LP-0110** — Multipart/file body foundation
- [ ] **LP-0111** — GraphQL body foundation
- [ ] **LP-0112** — Request settings model
- [ ] **LP-0113** — Cookie model foundation
- [ ] **LP-0114** — Request documentation
- [ ] **LP-0115** — Script extension points
- [~] **LP-0116** — Response domain boundary — status/headers/body/size/duration/timestamps exist; no cookies, no explicit content-type field
- [ ] **LP-0117** — Sample response model
- [ ] **LP-0118** — Persistence migrations for Request v2
- [ ] **LP-0119** — Round-trip and validation tests

## Phase 02 — Variables & Environments (7 DONE · 3 PARTIAL · 4 TODO)

- [x] **LP-0201** — Environment entity and CRUD
- [x] **LP-0202** — Variable entity
- [~] **LP-0203** — Variable scopes — global/environment/request are real; collection/folder are schema-reserved columns only, no backing entity
- [x] **LP-0204** — Deterministic precedence
- [x] **LP-0205** — Runtime variable resolver
- [~] **LP-0206** — Variable resolution in all request locations — URL/headers/body/query-params resolved; no auth model or scripts exist to resolve
- [x] **LP-0207** — Environment switching
- [~] **LP-0208** — Missing-variable diagnostics — resolver reports missing keys; surfaced in the URL preview only, not headers/body/params in the editor
- [x] **LP-0209** — Secret handling
- [ ] **LP-0210** — Dynamic variables
- [ ] **LP-0211** — Shared vs local environment data
- [ ] **LP-0212** — Postman environment import
- [ ] **LP-0213** — Variable editor UI
- [x] **LP-0214** — Variable tests

## Phase 03 — HTTP Engine & Request Execution (9 DONE · 1 PARTIAL · 4 TODO)

- [x] **LP-0301** — Select HTTP client architecture
- [x] **LP-0302** — Implement async request execution
- [ ] **LP-0303** — Build CanonicalRequest — blocks all of Phase 06
- [x] **LP-0304** — Execute supported methods
- [x] **LP-0305** — Apply query parameters
- [~] **LP-0306** — Apply headers and authorization — headers done; no Authorization model, must be typed as a raw header today
- [ ] **LP-0307** — Serialize all body types — raw string only
- [x] **LP-0308** — Timeouts and cancellation
- [ ] **LP-0309** — Redirect/proxy/TLS settings
- [ ] **LP-0310** — Cookie jar integration
- [x] **LP-0311** — Streaming and bounded response handling
- [x] **LP-0312** — Disk-backed large response path
- [x] **LP-0313** — Execution error model
- [x] **LP-0314** — HTTP engine tests

## Phase 04 — Response Viewer, History & UX (2 DONE · 1 PARTIAL · 18 TODO)

- [x] **LP-0401** — Response persistence
- [~] **LP-0402** — Response viewer — status/size/duration/body shown; headers fetched but not rendered; no cookies model
- [ ] **LP-0403** — JSON tree/pretty viewer
- [ ] **LP-0404** — Response search/copy/download
- [x] **LP-0405** — Response history
- [ ] **LP-0406** — Request editor tabs
- [ ] **LP-0407** — 100+ tab lifecycle
- [ ] **LP-0408** — Editor state persistence
- [ ] **LP-0409** — Virtualized project/request lists
- [ ] **LP-0410** — Search/indexing
- [ ] **LP-0411** — Bounded caches
- [ ] **LP-0412** — Developer Console UI
- [ ] **LP-0413** — Console request lifecycle logging
- [ ] **LP-0414** — Console response lifecycle logging
- [ ] **LP-0415** — Console event levels and filtering
- [ ] **LP-0416** — Console raw/pretty inspection
- [ ] **LP-0417** — Console copy/export/clear
- [ ] **LP-0418** — Console secret redaction
- [ ] **LP-0419** — Console correlation IDs
- [ ] **LP-0420** — Console performance safety
- [ ] **LP-0421** — Console runtime tests

## Phase 05 — Postman Compatibility & Import/Export (0/7 — all TODO)

- [ ] **LP-0501** — Postman collection importer
- [ ] **LP-0502** — Postman script import
- [ ] **LP-0503** — Large collection import pipeline
- [ ] **LP-0504** — Import validation/reporting
- [ ] **LP-0505** — Postman environment importer
- [ ] **LP-0506** — Internal-to-Postman export foundation
- [ ] **LP-0507** — Compatibility test fixtures

## Phase 06 — Code Snippets & cURL Cross-Platform (0/10 — all BLOCKED on LP-0303)

Every task here either *is* the CodeGenerator abstraction the task pack says must "generate from CanonicalRequest, never from UI text" (LP-0601), or depends on it. None can meaningfully start until LP-0303 exists.

- [!] **LP-0601** — CodeGenerator abstraction — blocked on LP-0303 (CanonicalRequest)
- [!] **LP-0602** — cURL Bash generator — blocked on LP-0601
- [!] **LP-0603** — cURL PowerShell generator — blocked on LP-0601
- [!] **LP-0604** — cURL Windows CMD generator — blocked on LP-0601
- [!] **LP-0605** — Code snippet UI — blocked on LP-0601-0604
- [!] **LP-0606** — Secret-safe snippet modes — blocked on LP-0601
- [!] **LP-0607** — Additional generators — blocked on LP-0601
- [!] **LP-0608** — cURL importer abstraction — blocked on LP-0303
- [!] **LP-0609** — cURL importer implementation — blocked on LP-0608
- [!] **LP-0610** — Generator/importer tests — blocked on the rest of Phase 06

## Phase 07 — Git, GitHub, Sync & Collaboration (0/13 — all TODO)

No external blocker — self-contained new work, internally sequential (0701 before 0702, etc.) but nothing outside the phase prevents starting.

- [ ] **LP-0701** — Project file format
- [ ] **LP-0702** — Local Git integration
- [ ] **LP-0703** — Repository permissions as collaboration source of truth
- [ ] **LP-0704** — GitHub authentication
- [ ] **LP-0705** — Clone/open repository
- [ ] **LP-0706** — Pull/push/sync operations
- [ ] **LP-0707** — Manual Sync UI
- [ ] **LP-0708** — Automatic Sync
- [ ] **LP-0709** — Offline-first sync queue
- [ ] **LP-0710** — Conflict detection
- [ ] **LP-0711** — Conflict resolution UI
- [ ] **LP-0712** — Sync status model
- [ ] **LP-0713** — Git/GitHub tests

## Phase 08 — AI / Claude & Source Project Intelligence (4 DONE · 1 PARTIAL · 2 BLOCKED · 14 TODO)

- [x] **LP-0801** — AiProvider / AiService abstraction
- [x] **LP-0802** — Claude provider
- [~] **LP-0803** — AI configuration/security — key from `ANTHROPIC_API_KEY` env var only; no OS keychain, no settings UI, model hardcoded
- [x] **LP-0804** — Structured AI output schema
- [x] **LP-0805** — AI API generation flow
- [ ] **LP-0806** — Generated sample responses
- [ ] **LP-0807** — Project-level Ask Claude UI
- [ ] **LP-0808** — AI project-context selection
- [ ] **LP-0809** — Secret redaction in AI context
- [ ] **LP-0810** — Source project folder selection
- [ ] **LP-0811** — Source project read-only boundary
- [ ] **LP-0812** — Source project analyzer
- [ ] **LP-0813** — OpenAPI/Swagger-first discovery
- [ ] **LP-0814** — Framework route discovery
- [ ] **LP-0815** — DTO/schema/security discovery
- [ ] **LP-0816** — AI source-context UI
- [!] **LP-0817** — Find API capability — blocked on LP-0812-0815 (no analyzer/discovery pipeline to search)
- [!] **LP-0818** — Generate sample request capability — blocked on LP-0812-0817 (nothing locates "the login API" yet)
- [ ] **LP-0819** — AI-generated tests/scripts/docs
- [ ] **LP-0820** — AI context size controls
- [ ] **LP-0821** — AI provenance/warnings

## Phase 09 — Security, Performance, Packaging & Quality (2 DONE · 5 PARTIAL · 2 BLOCKED · 8 TODO)

- [ ] **LP-0901** — Script sandbox
- [!] **LP-0902** — Script variable APIs — blocked on LP-0901 (no sandbox to run the API inside)
- [ ] **LP-0903** — Security hardening
- [~] **LP-0904** — Observability — secret-safe logging exists; no diagnostics command exposing memory/tab-count/startup-time
- [ ] **LP-0905** — Background job system
- [~] **LP-0906** — Error handling strategy — `AppError` gives typed, actionable errors; one flat enum, not the three-tier domain/application/infrastructure taxonomy described
- [x] **LP-0907** — Versioning and migrations
- [ ] **LP-0908** — Performance benchmark suite
- [ ] **LP-0909** — Scale tests
- [ ] **LP-0910** — Memory regression checks
- [~] **LP-0911** — Cross-platform validation — Windows verified (build + runtime); no Linux environment available this session
- [x] **LP-0912** — Windows packaging — debug-mode verified (real `.msi` + `.exe`); release (LTO) profile not yet built
- [!] **LP-0913** — Linux packaging — blocked on host OS (this session is Windows-only; no Linux build environment available)
- [ ] **LP-0914** — CI pipeline
- [ ] **LP-0915** — Release signing/update strategy
- [~] **LP-0916** — Documentation — README.md (41 sections) + PROJECT_MAP.md cover architecture/current-state extensively; no user docs, import/export docs, or AI privacy docs yet
- [~] **LP-0917** — Definition-of-Done audit — applied as an ongoing per-feature discipline (see every phase file's verification blocks); not a single finished audit artifact

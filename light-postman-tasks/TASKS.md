# Light Postman — Master Checklist

Progress: **37/146 tasks verified** (audited 2026-09-09 against actual code/tests, not assumed)

## Phase 00 — Foundation & Architecture

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

## Phase 01 — Request Domain Model v2

- [x] **LP-0101** — Request update
- [x] **LP-0102** — Request delete
- [x] **LP-0103** — Project update/delete
- [x] **LP-0104** — Query parameter model
- [x] **LP-0105** — Query parameter serialization
- [ ] **LP-0106** — Header entry model
- [ ] **LP-0107** — Authorization model
- [ ] **LP-0108** — Body model
- [ ] **LP-0109** — Raw body content types
- [ ] **LP-0110** — Multipart/file body foundation
- [ ] **LP-0111** — GraphQL body foundation
- [ ] **LP-0112** — Request settings model
- [ ] **LP-0113** — Cookie model foundation
- [ ] **LP-0114** — Request documentation
- [ ] **LP-0115** — Script extension points
- [ ] **LP-0116** — Response domain boundary
- [ ] **LP-0117** — Sample response model
- [ ] **LP-0118** — Persistence migrations for Request v2
- [ ] **LP-0119** — Round-trip and validation tests

## Phase 02 — Variables & Environments

- [x] **LP-0201** — Environment entity and CRUD
- [x] **LP-0202** — Variable entity
- [ ] **LP-0203** — Variable scopes *(partial: global/environment/request are real; collection/folder are schema-reserved only, no backing table)*
- [x] **LP-0204** — Deterministic precedence
- [x] **LP-0205** — Runtime variable resolver
- [ ] **LP-0206** — Variable resolution in all request locations *(partial: URL/headers/body resolved; no params/auth models yet to resolve)*
- [x] **LP-0207** — Environment switching
- [ ] **LP-0208** — Missing-variable diagnostics *(partial: resolver reports missing keys, surfaced in the URL preview only, not headers/body)*
- [x] **LP-0209** — Secret handling
- [ ] **LP-0210** — Dynamic variables
- [ ] **LP-0211** — Shared vs local environment data
- [ ] **LP-0212** — Postman environment import
- [ ] **LP-0213** — Variable editor UI
- [x] **LP-0214** — Variable tests

## Phase 03 — HTTP Engine & Request Execution

- [x] **LP-0301** — Select HTTP client architecture
- [x] **LP-0302** — Implement async request execution
- [ ] **LP-0303** — Build CanonicalRequest
- [x] **LP-0304** — Execute supported methods
- [ ] **LP-0305** — Apply query parameters
- [ ] **LP-0306** — Apply headers and authorization *(partial: headers done, no auth model)*
- [ ] **LP-0307** — Serialize all body types *(raw string only)*
- [x] **LP-0308** — Timeouts and cancellation
- [ ] **LP-0309** — Redirect/proxy/TLS settings
- [ ] **LP-0310** — Cookie jar integration
- [x] **LP-0311** — Streaming and bounded response handling
- [x] **LP-0312** — Disk-backed large response path
- [x] **LP-0313** — Execution error model
- [x] **LP-0314** — HTTP engine tests

## Phase 04 — Response Viewer, History & UX

- [x] **LP-0401** — Response persistence
- [ ] **LP-0402** — Response viewer *(partial: status/size/duration/body shown; headers not rendered, no cookies model)*
- [ ] **LP-0403** — JSON tree/pretty viewer
- [ ] **LP-0404** — Response search/copy/download
- [x] **LP-0405** — Response history
- [ ] **LP-0406** — Request editor tabs
- [ ] **LP-0407** — 100+ tab lifecycle
- [ ] **LP-0408** — Editor state persistence
- [ ] **LP-0409** — Virtualized project/request lists
- [ ] **LP-0410** — Search/indexing
- [ ] **LP-0411** — Bounded caches

## Phase 05 — Postman Compatibility & Import/Export

- [ ] **LP-0501** — Postman collection importer
- [ ] **LP-0502** — Postman script import
- [ ] **LP-0503** — Large collection import pipeline
- [ ] **LP-0504** — Import validation/reporting
- [ ] **LP-0505** — Postman environment importer
- [ ] **LP-0506** — Internal-to-Postman export foundation
- [ ] **LP-0507** — Compatibility test fixtures

## Phase 06 — Code Snippets & cURL Cross-Platform

- [ ] **LP-0601** — CodeGenerator abstraction
- [ ] **LP-0602** — cURL Bash generator
- [ ] **LP-0603** — cURL PowerShell generator
- [ ] **LP-0604** — cURL Windows CMD generator
- [ ] **LP-0605** — Code snippet UI
- [ ] **LP-0606** — Secret-safe snippet modes
- [ ] **LP-0607** — Additional generators
- [ ] **LP-0608** — cURL importer abstraction
- [ ] **LP-0609** — cURL importer implementation
- [ ] **LP-0610** — Generator/importer tests

## Phase 07 — Git, GitHub, Sync & Collaboration

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

## Phase 08 — AI / Claude & Source Project Intelligence

- [x] **LP-0801** — AiProvider / AiService abstraction
- [x] **LP-0802** — Claude provider
- [ ] **LP-0803** — AI configuration/security *(partial: key read from `ANTHROPIC_API_KEY` env var only, no OS keychain storage, no UI to configure it, model is hardcoded)*
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
- [ ] **LP-0817** — Find API capability
- [ ] **LP-0818** — Generate sample request capability
- [ ] **LP-0819** — AI-generated tests/scripts/docs
- [ ] **LP-0820** — AI context size controls
- [ ] **LP-0821** — AI provenance/warnings

## Phase 09 — Security, Performance, Packaging & Quality

- [ ] **LP-0901** — Script sandbox
- [ ] **LP-0902** — Script variable APIs
- [ ] **LP-0903** — Security hardening
- [ ] **LP-0904** — Observability
- [ ] **LP-0905** — Background job system
- [ ] **LP-0906** — Error handling strategy
- [ ] **LP-0907** — Versioning and migrations
- [ ] **LP-0908** — Performance benchmark suite
- [ ] **LP-0909** — Scale tests
- [ ] **LP-0910** — Memory regression checks
- [ ] **LP-0911** — Cross-platform validation
- [x] **LP-0912** — Windows packaging *(debug-mode verified; release profile not yet built)*
- [ ] **LP-0913** — Linux packaging
- [ ] **LP-0914** — CI pipeline
- [ ] **LP-0915** — Release signing/update strategy
- [ ] **LP-0916** — Documentation
- [ ] **LP-0917** — Definition-of-Done audit

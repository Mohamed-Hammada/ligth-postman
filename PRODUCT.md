# Product

<!-- impeccable:product-schema 1 -->

## Platform

web

## Users

Primary user: the developer building this (personal tool, confirmed 2026-09-11), used for their own API development and testing work. The architecture and docs (README.md, USER_GUIDE.md) are written for a broader eventual audience — individual developers with large workspaces (100+ tabs, thousands of requests) and teams collaborating through a shared GitHub repo — but right now there is one real user. Design and product decisions should optimize for that user's actual workflow first; the multi-user/team surfaces (Git sync, conflict resolution, GitHub OAuth) are built and should stay correct, but are not being validated against a live team yet.

## Product Purpose

"Light Postman" is a fast, local-first, low-memory desktop API client — a Postman alternative designed from day one to stay light at scale rather than get heavier as workspaces grow. Core purpose: request building/sending, collections, environments/variables, pre/post-request scripting with assertions, Postman/cURL/OpenAPI import, and optional Git/GitHub-backed sync — all usable fully offline, with no forced account or cloud dependency. Success = the app remains fast and responsive under large-scale conditions (100+ tabs, thousands of requests, large responses) and never requires the cloud or a subscription to be useful.

## Positioning

Distinct mechanism vs. Postman/Insomnia/Bruno: architected for scale-by-default (lazy loading, disk-backed state, bounded caches — see README §3) instead of retrofitting performance later; Git/GitHub *is* the collaboration and sync layer (no proprietary cloud sync service); local-first with optional local/offline AI providers (Ollama/LM Studio) alongside hosted ones, so AI assistance doesn't require sending data to a vendor cloud; zero recurring-subscription requirement.

## Operating Context

- Desktop app (Tauri 2 shell + SvelteKit/TypeScript UI + Rust core), not a hosted web app — "web" platform here means an HTML/CSS/WebView UI, not a native OS design language.
- First-class targets: Windows (.exe/.msi) and Linux (.deb/AppImage); macOS (.dmg) is future/not yet first-class.
- Must work fully offline; Git/GitHub connectivity is optional and never blocks core editing or saving.
- Interacts with: local SQLite (app data dir), local filesystem, Git CLI, GitHub (OAuth device flow), external HTTP endpoints the user targets, and optionally local or hosted LLM providers (Claude, OpenAI, Ollama/LM Studio) for AI features.
- Imports/interoperates with Postman Collection v2.1, Postman Environments, cURL commands (Bash/CMD/PowerShell), and OpenAPI 3.0/Swagger.

## Capabilities and Constraints

Confirmed functionality (per README.md and PROJECT_MAP.md, all phases marked implemented): request model (methods, params, headers, cookies, body types, auth schemes, settings), 6-tier variable scope resolution (Runtime > Request > Folder > Collection > Environment > Global) with dynamic variables, sandboxed pre/post-request JS scripting (boa_engine, no FS/network/process access) with `pm.*` Postman-compatible APIs, collections/folders/drag-drop organization, response viewer (JSON/raw/headers/cookies, streaming + disk spillover for large bodies), request history, code snippet generation (cURL, Python, JS), Git-native sync with conflict resolution (LOCAL/REMOTE/BASE), GitHub device OAuth, local AI assistance (endpoint generation, test generation, docs, mock responses, AST-based route extraction for Express/FastAPI/Gin/Spring Boot/Laravel/NestJS), diagnostics (RSS memory, DB size, entity counts).

Hard constraints (non-negotiable per README §37/§38, still binding): never keep the entire workspace/all projects/all requests in memory; inactive tabs must stay cheap (metadata only); Save (local persistence) must never depend on network availability, and must never be blocked by Git/GitHub operations; no arbitrary script filesystem/process/network access; secrets must never be logged, displayed unmasked, sent to AI prompts, or auto-committed to Git; no silent conflict overwrites during sync; no Electron (Tauri only).

Undecided: license/distribution model (README says "License TBD"; user confirmed 2026-09-11 this is intentionally not decided yet — do not let design or docs work assume open-source or proprietary distribution). Public audience beyond the current solo developer is not yet real (see Users).

## Evidence on Hand

- `README.md` — full architecture/product specification (41 sections): vision, stack, tab/project architecture, persistence, sync, conflict resolution, import, scripting, security, performance budgets, dev rules, implementation phases, definition of done.
- `PROJECT_MAP.md` — live status tracker; claims 146/146 backlog tasks implemented across 10 phases, plus a 2026-09-10 re-audit noting some "done-on-paper" items were found broken/unreachable and have since been fixed. Treat completion claims as implementation-exists, not bug-free.
- `docs/USER_GUIDE.md`, `docs/ARCHITECTURE.md`, `docs/IMPORT_EXPORT.md`, `docs/SECURITY_AND_AI_PRIVACY.md`, `docs/RELEASE_AND_UPDATE_STRATEGY.md`, `docs/DEFINITION_OF_DONE_AUDIT.md` — detailed supporting documentation.
- App icons under `src-tauri/icons/` and `static/favicon.png` are the unmodified `create-tauri-app` scaffold defaults, not a designed brand mark — no custom logo exists yet.
- No testimonials, customer names, benchmarks, pricing, or licensing terms exist — future work must not fabricate any of these.

## Product Principles

1. **Lazy everything.** Never pay memory/CPU cost for a project, tab, request, or response the user isn't currently using.
2. **Save and Sync are separate operations.** Local persistence is instant and network-independent; Git/GitHub sync is a distinct, cancelable, non-blocking operation layered on top.
3. **Offline-first, cloud-optional.** The app is fully usable with no network, no account, and no subscription; GitHub and AI-provider connectivity are additive, never required.
4. **Git/GitHub is the collaboration model.** Don't invent a parallel permissions or sharing system — defer to repository permissions and Git semantics (including conflict resolution: local/remote/base must all be visible, never silently overwritten).
5. **Postman-compatible, not Postman-dependent.** Import Postman/cURL/OpenAPI data faithfully, then own the internal model going forward rather than staying schema-locked to Postman's format.

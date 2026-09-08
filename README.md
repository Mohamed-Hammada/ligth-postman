# Lightweight API Client

> A fast, low-memory, cross-platform **Postman alternative** — built from day one for large workspaces, hundreds of tabs, team collaboration, and GitHub-based synchronization.

**Status:** 🚧 Early scaffold — Tauri + SvelteKit + TypeScript shell in place, Rust core (persistence, request model, import) in progress.

---

## Why

Postman-style tools tend to get heavier as workspaces grow: more tabs, more collections, more requests, more memory. This project is designed the opposite way — every architectural decision assumes **large scale by default** (100+ open tabs, thousands of requests, large responses, multi-user Git repos) and optimizes so the app stays fast and light anyway.

It combines ideas from:

- **Postman** — requests, collections, environments, scripts, testing.
- **VS Code / IntelliJ IDEA** — lazy loading, projects, tabs, indexing, resource management.
- **Git + GitHub** — version control, sharing, collaboration.
- **Native desktop apps** — fast startup, low memory footprint, Windows + Linux first.

> **Golden rule:** Never make the application pay the memory and CPU cost of something the user is not currently using.

---

## Table of Contents

- [1. Vision](#1-vision)
- [2. Technology Stack](#2-technology-stack)
- [3. Architecture Principles](#3-architecture-principles)
- [4. Tab Architecture](#4-tab-architecture)
- [5. Project Architecture](#5-project-architecture)
- [6. Persistence](#6-persistence)
- [7. Save System](#7-save-system)
- [8. Git / GitHub Architecture](#8-git--github-architecture)
- [9. Multi-user Repository Access](#9-multi-user-repository-access)
- [10. Sync Modes](#10-sync-modes)
- [11. Conflict Resolution](#11-conflict-resolution)
- [12. Postman Import](#12-postman-import)
- [13. Request Model](#13-request-model)
- [14. Environments & Variables](#14-environments--variables)
- [15. Pre-request Scripts](#15-pre-request-scripts)
- [16. Post-request / Test Scripts](#16-post-request--test-scripts)
- [17. Script Security](#17-script-security)
- [18. HTTP Engine](#18-http-engine)
- [19. Response Viewer](#19-response-viewer)
- [20. Search & Indexing](#20-search--indexing)
- [21. UI Performance](#21-ui-performance)
- [22. Caching](#22-caching)
- [23. History](#23-history)
- [24. GitHub Integration](#24-github-integration)
- [25. Offline-first Behavior](#25-offline-first-behavior)
- [26. Cross-platform](#26-cross-platform)
- [27. Release / CI](#27-release--ci)
- [28. Project File Format](#28-project-file-format)
- [29. Security](#29-security)
- [30. Observability](#30-observability)
- [31. Resource Budgets](#31-resource-budgets)
- [32. Architecture Layers](#32-architecture-layers)
- [33. Background Jobs](#33-background-jobs)
- [34. Error Handling](#34-error-handling)
- [35. Versioning & Migrations](#35-versioning--migrations)
- [36. Testing Strategy](#36-testing-strategy)
- [37. Development Rules](#37-development-rules)
- [38. What We Must Avoid](#38-what-we-must-avoid)
- [39. Recommended Implementation Order](#39-recommended-implementation-order)
- [40. Definition of Done](#40-definition-of-done)
- [41. Variables & Environments (Detail)](#41-variables--environments-detail)
- [Getting Started (Dev)](#getting-started-dev)
- [Contributing](#contributing)
- [License](#license)

---

## 1. Vision

Build a desktop API client that feels like a combination of Postman, a modern IDE, and Git-native collaboration — **not** a "smaller Postman" built on the same resource-heavy assumptions.

The architecture must assume that users can have:

- 100+ open request tabs.
- Hundreds or thousands of projects.
- Thousands or tens of thousands of requests.
- Large collections.
- Large response bodies.
- Multiple environments.
- Team members sharing the same GitHub repository.

The application must remain responsive and memory-efficient under all of these conditions.

---

## 2. Technology Stack

### Desktop shell — Tauri

- Native desktop shell with a much smaller runtime footprint than Electron.
- Cross-platform, excellent Rust integration.
- Windows and Linux support from the same architecture.

### Core — Rust

Rust owns everything performance-sensitive and system-level:

- HTTP execution
- Persistence
- Project management
- Postman import/export
- Git operations
- GitHub integration
- Synchronization
- Background jobs
- Search/indexing
- Large-file/large-response handling
- Script execution orchestration
- Security boundaries

### UI

- **Preferred:** Svelte + TypeScript
- **Alternative** (if an existing project already has a strong React architecture): React + TypeScript

Do not rewrite a working UI solely for theoretical performance gains — architecture and lifecycle matter more than framework choice.

### Database — SQLite

Used as the primary local application database, with:

- WAL mode
- Prepared statements
- Transactions
- Proper indexes
- Migration system
- Incremental loading
- No unnecessary duplication of large payloads

---

## 3. Architecture Principles

### Lazy Everything

Never load expensive data unless it is needed:

- Don't load every project.
- Don't load every collection.
- Don't instantiate every request editor.
- Don't keep every response body in memory.
- Don't run background work for inactive tabs.
- Don't create network clients for requests that aren't executing.

Use lazy loading, virtualization, on-demand hydration, disk-backed state, bounded caches, and resource disposal.

---

## 4. Tab Architecture

100+ tabs must be a normal, supported use case.

**Inactive tab** — lightweight metadata only:

```text
Tab ID
Project ID
Request ID
Title
Dirty state
Last known cursor/selection
Last known UI state
```

**Active tab** — the only tab that normally has:

```text
Full editor
Request runtime
Response viewer
Active subscriptions
Temporary execution state
Large response buffers
```

**Lifecycle:**

```text
Tab opened
    ↓
Lightweight metadata
    ↓
User activates tab
    ↓
Hydrate request
    ↓
Mount editor/runtime
    ↓
User switches tab
    ↓
Persist state
    ↓
Release heavy resources
    ↓
New tab hydrates
```

Never solve the 100-tab requirement by simply increasing memory limits — make inactive tabs cheap.

---

## 5. Project Architecture

Projects behave conceptually like VS Code / IntelliJ workspaces.

**Project metadata kept in memory:**

```text
Project ID
Name
Repository information
Last opened
Sync status
Dirty state
```

Do not keep every request and collection in memory.

**Project loading:**

```text
Project list
    ↓
Load metadata
    ↓
User opens project
    ↓
Load tree/index
    ↓
User opens folder
    ↓
Load folder contents
    ↓
User opens request
    ↓
Load complete request
```

Large projects must remain usable.

---

## 6. Persistence

SQLite is the primary local persistence layer. The application must be usable without GitHub or an internet connection.

**Critical rule: Save and Sync are different operations.**

- **Save** — persist current work locally, immediately. Must not depend on network availability.
- **Sync** — synchronize local project state with the configured Git/GitHub repository.

---

## 7. Save System

Every user edit should be persisted safely, while avoiding excessive disk writes via:

- Debouncing
- Batching
- Transactions
- Dirty-state tracking

Never compromise data safety. The user should see clear state, e.g.:

```text
✓ Saved locally
```

---

## 8. Git / GitHub Architecture

Projects can optionally be connected to a Git repository:

```text
Project
 ├── Local SQLite/runtime state
 └── Git repository
       ├── collections/
       ├── environments/
       ├── requests/
       └── project metadata
```

Git provides history, branches, commits, diffs, collaboration, and conflict detection. GitHub provides remote hosting, access control, sharing, pull/push, and repository collaboration.

Do not build a separate permission system that contradicts GitHub repository permissions.

---

## 9. Multi-user Repository Access

Anyone with appropriate access to the GitHub repository can work with the shared project according to their GitHub permissions:

```text
GitHub Repository
      │
      ├── User A
      ├── User B
      ├── User C
      └── User D
```

Each user can open the project, pull changes, make local changes, commit, push (if permitted), work offline, and synchronize later.

The application must never store GitHub passwords or expose access tokens inside project files — use the platform's secure credential storage.

---

## 10. Sync Modes

### Manual Sync

User explicitly clicks **Sync**; the application performs the configured pull/diff/push workflow.

### Automatic Sync

Synchronizes in the background. Must be debounced, non-blocking, cancelable, failure-tolerant, and resource-bounded. Do not create a Git commit for every keystroke.

Possible strategy:

```text
User edits
    ↓
Save locally
    ↓
Wait for inactivity
    ↓
Detect local changes
    ↓
Pull latest remote state
    ↓
Detect conflicts
    ↓
Create meaningful commit
    ↓
Push
```

The exact commit strategy should be configurable.

---

## 11. Conflict Resolution

Conflicts are expected. Never silently overwrite another user's changes.

When a conflict occurs, `LOCAL`, `REMOTE`, and `BASE` must all be available to the conflict resolver. The UI should clearly show the local version, remote version, base version, conflicting fields/files, and the resolution action:

- Keep local
- Keep remote
- Merge
- Cancel

---

## 12. Postman Import

Postman compatibility is a first-class requirement:

```text
Postman
   ↓
Import
   ↓
Our internal model
   ↓
Immediately usable project
```

**Import targets:** collections, environments, folders, requests, variables, headers, query parameters, bodies, authentication, pre-request scripts, tests/post-request scripts.

**Large imports** — a collection with thousands of requests must not become a huge in-memory object graph:

```text
Postman JSON
    ↓
Rust parser
    ↓
Validation
    ↓
Normalization
    ↓
SQLite
    ↓
Lightweight indexes
```

Requests are loaded only when needed. Postman data is converted into our own internal schema — the application must not remain permanently dependent on Postman's JSON schema.

---

## 13. Request Model

A request should support:

```text
Method
URL
Query Parameters
Path Variables
Headers
Cookies
Body
Authentication
Variables
Pre-request Script
Post-request/Test Script
Settings
Metadata
```

The model should be versionable and migration-friendly.

---

## 14. Environments & Variables

Support multiple environments (Development, Staging, Production, ...) with clear variable scopes:

```text
Global
Environment
Collection
Folder
Request
Runtime
```

Precedence must be explicitly defined and tested. Secrets should be handled separately from ordinary project data, and never accidentally committed to Git.

See [Section 41](#41-variables--environments-detail) for the full variable/environment design.

---

## 15. Pre-request Scripts

JavaScript-compatible pre-request scripting:

```text
Load request
    ↓
Load variables
    ↓
Run pre-request script
    ↓
Resolve final request
    ↓
Send HTTP request
```

```javascript
pm.environment.set("timestamp", Date.now());
```

The scripting API should provide a controlled compatibility layer for common Postman APIs.

---

## 16. Post-request / Test Scripts

Scripts run after the HTTP response is received:

```text
Send request
    ↓
Receive response
    ↓
Run post-request/test script
    ↓
Assertions
    ↓
Update variables if requested
    ↓
Persist result where appropriate
```

```javascript
pm.test("Status is 200", () => {
    pm.response.to.have.status(200);
});
```

---

## 17. Script Security

Never execute arbitrary user JavaScript with unrestricted system access. Scripts run inside a sandbox with:

- Explicit API surface
- Timeout
- Memory/resource limits where practical
- No arbitrary filesystem access
- No arbitrary OS command execution
- No unrestricted process spawning
- Controlled network capabilities
- Controlled access to secrets

The Rust core controls communication between the UI, request engine, and script runtime.

---

## 18. HTTP Engine

Rust-based, supporting:

- HTTP/1.1, HTTP/2
- TLS
- Streaming responses
- Timeouts
- Redirect control
- Proxy support
- Certificates
- Authentication
- Request cancellation
- Upload/download streaming
- Large response handling

Large responses must not unnecessarily remain in RAM — prefer streaming and temporary disk-backed storage.

---

## 19. Response Viewer

Must be virtualized/lazy — never render millions of lines as one UI component tree. Supports JSON viewer, raw text, headers, cookies, status, timing, size, search, pretty-printing, copy, and save/export. Large responses should be streamed or paged where practical.

---

## 20. Search & Indexing

Must scale to large projects using SQLite indexes and/or a dedicated indexing layer. Supports search across request names, URLs, collections, projects, environments, and variables — without loading every request into the UI.

---

## 21. UI Performance

Use virtualization for large lists (projects, collections, folders, requests, history, tabs). Avoid:

- Hundreds of mounted editors
- Huge DOM trees
- Unbounded reactive stores
- Unnecessary polling/timers
- Duplicate response data
- Global state containing entire projects

---

## 22. Caching

Caching must be bounded — every cache needs a reason and a policy (LRU, size limit, time limit, or explicit invalidation). Never allow caches to grow indefinitely.

---

## 23. History

Request history is persistent and queryable. Don't keep unlimited response bodies in RAM — large historical responses should be stored on disk with metadata in SQLite. Retention should be user-configurable.

---

## 24. GitHub Integration

Supports authentication, repository selection, project↔repository association, clone/open, pull/push, branches, commit history, diffs, conflict resolution, and sync status.

The GitHub integration should be isolated behind an abstraction so the core can also work with non-GitHub Git repositories.

---

## 25. Offline-first Behavior

The application remains fully usable offline:

```text
Edit
 ↓
Save locally
 ↓
Continue working
```

When connectivity returns:

```text
Local changes
 ↓
Sync
 ↓
Conflict detection
 ↓
Push
```

No feature should unnecessarily block the editor because GitHub is unavailable.

---

## 26. Cross-platform

**First-class targets:**

- **Windows** — `.exe` installer, `.msi` where appropriate
- **Linux** — Ubuntu and compatible distributions; `.deb`, AppImage

**Future:** macOS `.dmg`

Do not introduce Windows-only assumptions into the core — OS-specific code must be isolated.

---

## 27. Release / CI

GitHub Actions builds platform artifacts automatically:

```text
Release
├── Windows
│   ├── app-setup.exe
│   └── app.msi
│
├── Linux
│   ├── app.deb
│   └── app.AppImage
│
└── macOS
    └── app.dmg
```

CI includes Rust tests, UI tests, integration tests, import tests, sync tests, cross-platform build checks, security checks, and linting/formatting.

---

## 28. Project File Format

The project representation should be human-readable where practical, Git-friendly, diff-friendly, versionable, and migration-friendly. Avoid giant monolithic JSON files when they hurt Git diffs and collaboration — prefer logical separation:

```text
project/
├── project.json
├── collections/
│   ├── auth/
│   └── payments/
├── environments/
│   ├── dev.json
│   └── staging.json
├── requests/
└── scripts/
```

The exact structure should be decided after evaluating implementation requirements.

---

## 29. Security

- Secure credential storage
- OAuth
- Token protection
- Secret redaction
- TLS verification
- Certificate handling
- Script sandboxing
- Repository trust
- Import validation
- Malformed JSON handling
- Path traversal protection
- Resource exhaustion protection
- Safe handling of untrusted Postman files

Never log secrets. Never include secrets in Git commits automatically.

---

## 30. Observability

Expose useful diagnostics without exposing sensitive information — e.g. startup time, memory usage, active tabs, loaded projects, database size, sync status, HTTP timings, script execution duration. Provide a diagnostic mode for troubleshooting.

**Never include:** authorization headers, API keys, passwords, OAuth tokens, secret environment variables.

---

## 31. Resource Budgets

Performance must be measurable — define targets instead of relying on subjective "feels fast". Track:

- Startup time
- Idle RAM
- RAM with 100 tabs
- RAM with large projects
- Time to open a request
- Time to switch tabs
- Import time
- Search latency
- Sync latency
- Large response behavior

Create automated performance benchmarks.

---

## 32. Architecture Layers

```text
┌─────────────────────────────────────┐
│              UI Layer               │
│       Svelte + TypeScript           │
├─────────────────────────────────────┤
│          Tauri Application           │
├─────────────────────────────────────┤
│              Rust Core              │
│                                     │
│  Request Engine                     │
│  Project Manager                    │
│  Persistence                        │
│  Postman Importer                   │
│  Script Runtime                     │
│  Git Engine                         │
│  GitHub Integration                 │
│  Sync Engine                        │
│  Search/Indexing                    │
│  Background Jobs                    │
├─────────────────────────────────────┤
│             SQLite                  │
├─────────────────────────────────────┤
│         Local Files / Git           │
└─────────────────────────────────────┘
```

The UI must not directly own business-critical persistence or Git logic.

---

## 33. Background Jobs

Controlled background job system for Git fetch, sync, indexing, Postman import, large file processing, and response persistence. Requirements: cancellation, concurrency limits, progress reporting, error isolation, no runaway workers.

---

## 34. Error Handling

Errors must be structured — never leak low-level Rust errors directly into the UI. Use typed/domain errors:

```text
ImportError
NetworkError
AuthenticationError
StorageError
SyncError
ConflictError
ScriptError
RepositoryError
```

The UI receives human-readable messages plus diagnostic information.

---

## 35. Versioning & Migrations

Everything persisted must be migration-friendly (v1 → v2 → v3 → ...). Never assume the database/project format will remain unchanged. Provide schema migrations, project format versioning, backward compatibility where practical, safe migration failure handling, and backups before destructive migrations.

---

## 36. Testing Strategy

- **Unit tests:** request model, variable resolution, import parsing, sync logic, conflict detection, storage, script APIs.
- **Integration tests:** HTTP execution, SQLite, Git, GitHub workflows, Postman import.
- **Performance tests:** 100 tabs, 500 tabs, 1,000 projects, 10,000 requests, large collections, large responses.
- **Cross-platform tests:** Windows, Ubuntu, other supported Linux distributions.

---

## 37. Development Rules

Before implementing a feature:

1. Understand the architecture.
2. Define ownership of state.
3. Define lifecycle.
4. Define persistence.
5. Define resource usage.
6. Define failure behavior.
7. Define security implications.
8. Define tests.
9. Implement incrementally.
10. Benchmark where performance is relevant.

Do not introduce shortcuts that violate the core architecture.

---

## 38. What We Must Avoid

- Electron, unless there is a compelling reason.
- Keeping the entire workspace in RAM.
- One global store containing everything.
- Mounting every tab's editor.
- Loading all projects at startup.
- Loading every request at startup.
- Unlimited caches or unlimited history in memory.
- Blocking UI on Git operations.
- Blocking Save on network operations.
- Arbitrary script execution.
- Storing secrets in Git.
- Silent conflict overwrites.
- Giant monolithic project files without justification.
- Platform-specific assumptions in the Rust core.
- Premature rewrites without profiling.

---

## 39. Recommended Implementation Order

| Phase | Focus |
|---|---|
| 0 — Architecture | Repository inspection, ADRs, domain model, state ownership, persistence design, resource lifecycle design |
| 1 — Core | Rust workspace, Tauri shell, SQLite, project/request/environment models, HTTP engine |
| 2 — UI | Application shell, project explorer, request editor, tabs, lazy loading, virtualized lists |
| 3 — Postman Compatibility | Collection/environment import, request mapping, variables, scripts, tests |
| 4 — Git | Local Git repository support, project file structure, diff, commit, branch, conflict detection |
| 5 — GitHub | Authentication, repository selection, clone/pull/push, sync status, multi-user workflows |
| 6 — Automatic Sync | Debounced sync, background jobs, conflict UI, offline queue |
| 7 — Performance | 100+ tab benchmark, large project/import/response benchmarks, memory & startup profiling |
| 8 — Packaging | Windows installer, Ubuntu/Debian package, AppImage, CI release pipeline |

---

## 40. Definition of Done

A feature is **not** complete just because it works. It's complete when it:

- Works.
- Persists correctly.
- Behaves correctly offline where applicable.
- Does not leak resources.
- Has bounded memory behavior.
- Has appropriate error handling.
- Has security controls.
- Has tests.
- Works across supported platforms.
- Does not break the lazy-loading model.

**Examples of the golden rule in practice:**

- 100 tabs ≠ 100 active editors.
- 1,000 projects ≠ 1,000 projects loaded in RAM.
- 10,000 requests ≠ 10,000 request objects mounted in the UI.
- GitHub unavailable ≠ application unusable.
- Large response ≠ entire response permanently held in RAM.

---

## 41. Variables & Environments (Detail)

Variables and Environments are first-class features designed into the core architecture from the beginning.

### Environments

Support multiple environments, e.g. Development, Staging, Production, Testing. Each environment supports:

- Variable name
- Value
- Enabled/disabled state
- Secret flag
- Description
- Metadata

### Variable scopes

```text
Global
Environment
Collection
Folder
Request
Runtime
```

Precedence must be deterministic, documented, and tested. Recommended initial order:

```text
Runtime
  ↓
Request
  ↓
Folder
  ↓
Collection
  ↓
Environment
  ↓
Global
```

### Runtime variable resolution

Do not rewrite every request when an environment changes — keep the stored request template (e.g. `{{baseUrl}}/users/{{userId}}`) and resolve it only when execution starts:

```text
Stored Request
    ↓
Current Environment
    ↓
Runtime Variables
    ↓
Variable Resolver
    ↓
Final HTTP Request
```

This keeps large projects lightweight and makes environment switching cheap.

### Secrets

Sensitive values (API keys, access tokens, client secrets, passwords) must:

- Be masked where appropriate.
- Never appear in logs or diagnostics.
- Never be committed automatically to Git.
- Use secure OS credential storage where appropriate.
- Be clearly separated from shared project configuration.

### Shared vs. local environment data

Support team-safe environments where shared configuration syncs through Git while local secrets remain local. The exact storage model should be designed before implementation.

### Dynamic variables

Leave room for runtime-generated values: UUID, timestamp, random number, random string, date/time — generated at runtime unless explicitly persisted by the user.

### Scripts

Pre-request and post-request scripts read/update variables through a controlled API:

```javascript
pm.environment.set("token", "...");
pm.environment.get("token");
```

Script access must respect variable scope and secret-handling rules.

### Postman compatibility

Postman environment/variable imports should preserve, where possible: variable names, values, enabled state, scope, descriptions, and secret-related metadata. Large environments must also be loaded lazily.

### Variable editor

The UI supports create, edit, delete, enable/disable, search, secret masking, environment switching, import/export, and clear scope/source indicators.

---

## Getting Started (Dev)

Scaffolded with `create-tauri-app` (Tauri 2 + SvelteKit + TypeScript + Vite).

```bash
npm install
npm run tauri dev    # desktop dev build
```

**Recommended IDE setup:** [VS Code](https://code.visualstudio.com/) + [Svelte](https://marketplace.visualstudio.com/items?itemName=svelte.svelte-vscode) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer).

See [`PROJECT_MAP.md`](./PROJECT_MAP.md) for current implementation status and pending work.

---

## Contributing

1. Read this document in full; it defines the non-negotiable architectural constraints (see [§37 Development Rules](#37-development-rules) and [§38 What We Must Avoid](#38-what-we-must-avoid)).
2. Any change that keeps an entire workspace, project list, or response history fully in memory will be rejected regardless of whether the feature "works."
3. Follow [§39 Recommended Implementation Order](#39-recommended-implementation-order) — later phases assume earlier ones are in place.
4. Check [`PROJECT_MAP.md`](./PROJECT_MAP.md) before starting work — update it as part of the same change, not as a follow-up.

## License

License TBD.

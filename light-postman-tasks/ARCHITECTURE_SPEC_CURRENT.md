# Lightweight API Client — Product & Architecture Specification

> **Status:** Architecture / Product Requirements  
> **Goal:** Build a fast, lightweight, cross-platform Postman alternative designed from day one for low memory usage, large workspaces, many tabs, team collaboration, and GitHub-based synchronization.

---

## 1. Vision

Build a desktop API client that feels like a combination of:

- **Postman** — API requests, collections, environments, scripts, testing.
- **VS Code / IntelliJ IDEA** — lazy loading, projects, tabs, indexing, persistence, and resource management.
- **Git + GitHub** — version control, sharing, synchronization, collaboration.
- **Native desktop application** — fast startup, low memory footprint, Windows + Linux first.

The product must **not** be a "smaller Postman" built on the same resource-heavy assumptions.

The architecture must assume that users can have:

- 100+ open request tabs.
- Hundreds or thousands of projects.
- Thousands or tens of thousands of requests.
- Large collections.
- Large response bodies.
- Multiple environments.
- Team members sharing the same GitHub repository.

The application should remain responsive and memory-efficient under these conditions.

---

# 2. Recommended Technology Stack

## Desktop

**Tauri**

Reason:

- Native desktop shell.
- Much smaller runtime footprint than Electron.
- Cross-platform.
- Excellent integration with Rust.
- Windows and Linux support from the same application architecture.

## Core

**Rust**

Rust should own the performance-sensitive and system-level parts:

- HTTP execution.
- Persistence.
- Project management.
- Postman import/export.
- Git operations.
- GitHub integration.
- Synchronization.
- Background jobs.
- Search/indexing.
- Large-file/large-response handling.
- Script execution orchestration.
- Security boundaries.

## UI

Preferred:

**Svelte + TypeScript**

Alternative if the existing project already has a strong React architecture:

**React + TypeScript**

Do not rewrite a working UI solely for theoretical performance gains. The architecture and lifecycle are more important than the framework choice.

## Database

**SQLite**

Use SQLite as the local application database.

Recommended:

- WAL mode.
- Prepared statements.
- Transactions.
- Proper indexes.
- Migration system.
- Incremental loading.
- No unnecessary duplication of large payloads.

---

# 3. Architecture Principles

## 3.1 Lazy Everything

Never load expensive data unless it is needed.

Examples:

- Do not load every project.
- Do not load every collection.
- Do not instantiate every request editor.
- Do not keep every response body in memory.
- Do not run background work for inactive tabs.
- Do not create network clients for requests that are not executing.

Use:

- Lazy loading.
- Virtualization.
- On-demand hydration.
- Disk-backed state.
- Bounded caches.
- Resource disposal.

---

# 4. Tab Architecture

100+ tabs must be a normal supported use case.

## Lightweight tab state

An inactive tab should contain only lightweight metadata such as:

```text
Tab ID
Project ID
Request ID
Title
Dirty state
Last known cursor/selection
Last known UI state
```

The heavy request editor should NOT remain mounted for every tab.

## Active tab

Only the active tab should normally have:

```text
Full editor
Request runtime
Response viewer
Active subscriptions
Temporary execution state
Large response buffers
```

## Lifecycle

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

## Important

Never solve the 100-tab requirement by simply increasing memory limits.

The architecture must make inactive tabs cheap.

---

# 5. Project Architecture

Projects should behave conceptually like VS Code / IntelliJ workspaces.

## Project metadata

Keep lightweight metadata in memory:

```text
Project ID
Name
Repository information
Last opened
Sync status
Dirty state
```

Do not keep every request and collection in memory.

## Project loading

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

# 6. Persistence

## Local persistence

SQLite should be the primary local persistence layer.

The application should be usable without GitHub or an internet connection.

### Critical rule

**Save and Sync are different operations.**

## Save

Save means:

> Persist my current work locally immediately.

Save must not depend on network availability.

## Sync

Sync means:

> Synchronize local project state with the configured Git/GitHub repository.

---

# 7. Save System

Every user edit should be persisted safely.

Avoid excessive disk writes by using:

- Debouncing.
- Batching.
- Transactions.
- Dirty-state tracking.

But never compromise data safety.

The user should see clear state such as:

```text
✓ Saved locally
```

---

# 8. Git / GitHub Architecture

Projects can optionally be connected to a Git repository.

Example:

```text
Project
 ├── Local SQLite/runtime state
 └── Git repository
       ├── collections/
       ├── environments/
       ├── requests/
       └── project metadata
```

Git should provide:

- History.
- Branches.
- Commits.
- Diffs.
- Collaboration.
- Conflict detection.

GitHub provides:

- Remote hosting.
- Access control.
- Sharing.
- Pull/push.
- Repository collaboration.

Do not build a separate permission system that contradicts GitHub repository permissions.

---

# 9. Multi-user Repository Access

Anyone who has appropriate access to the GitHub repository should be able to work with the shared project according to their GitHub permissions.

Example:

```text
GitHub Repository
      │
      ├── User A
      ├── User B
      ├── User C
      └── User D
```

Each user can:

- Open the project.
- Pull changes.
- Make local changes.
- Commit.
- Push if their GitHub permissions allow it.
- Work offline.
- Synchronize later.

The application must never store GitHub passwords or expose access tokens inside project files.

Use the platform's secure credential storage where appropriate.

---

# 10. Sync Modes

Support at least:

## Manual Sync

User explicitly clicks:

```text
Sync
```

The application performs the configured pull/diff/push workflow.

## Automatic Sync

The application synchronizes in the background.

Automatic sync must be:

- Debounced.
- Non-blocking.
- Cancelable.
- Failure-tolerant.
- Resource-bounded.

Do not create a Git commit for every keystroke.

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

# 11. Conflict Resolution

Conflicts are expected.

Never silently overwrite another user's changes.

When a conflict occurs:

```text
LOCAL
REMOTE
BASE
```

must be available to the conflict resolver.

The UI should clearly show:

- Local version.
- Remote version.
- Base version.
- Conflicting fields/files.
- Resolution action.

Possible actions:

- Keep local.
- Keep remote.
- Merge.
- Cancel.

---

# 12. Postman Import

Postman compatibility is a first-class requirement.

The user should be able to migrate:

```text
Postman
   ↓
Import
   ↓
Our internal model
   ↓
Immediately usable project
```

## Import targets

Support as much of the following as practical:

- Collections.
- Environments.
- Folders.
- Requests.
- Variables.
- Headers.
- Query parameters.
- Bodies.
- Authentication.
- Pre-request scripts.
- Tests / post-request scripts.

## Large imports

A collection with thousands of requests must not become a huge in-memory object graph.

Preferred flow:

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

Requests are loaded only when needed.

## Internal model

Postman data should be converted into our own internal schema.

Do not make the entire application permanently dependent on Postman's JSON schema.

---

# 13. Request Model

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

# 14. Environments & Variables

Support multiple environments:

```text
Development
Staging
Production
```

Variable resolution should support clear scopes.

For example:

```text
Global
Environment
Collection
Folder
Request
Runtime
```

The exact precedence must be explicitly defined and tested.

Secrets should be handled separately from ordinary project data where appropriate.

Never accidentally commit sensitive production secrets to Git.

---

# 15. Pre-request Scripts

Support JavaScript-compatible pre-request scripting.

Execution flow:

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

Example capability:

```javascript
pm.environment.set("timestamp", Date.now());
```

The scripting API should provide a controlled compatibility layer for common Postman APIs.

---

# 16. Post-request / Test Scripts

Support scripts after the HTTP response is received.

Execution flow:

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

Example:

```javascript
pm.test("Status is 200", () => {
    pm.response.to.have.status(200);
});
```

---

# 17. Script Security

Never execute arbitrary user JavaScript directly with unrestricted system access.

Scripts must run inside a sandbox.

The sandbox should have:

- Explicit API surface.
- Timeout.
- Memory/resource limits where practical.
- No arbitrary filesystem access.
- No arbitrary OS command execution.
- No unrestricted process spawning.
- Controlled network capabilities.
- Controlled access to secrets.

The Rust core should control communication between the UI, request engine, and script runtime.

---

# 18. HTTP Engine

The HTTP engine should be Rust-based.

Requirements:

- HTTP/1.1.
- HTTP/2.
- TLS.
- Streaming responses.
- Timeouts.
- Redirect control.
- Proxy support.
- Certificates.
- Authentication.
- Request cancellation.
- Upload/download streaming.
- Large response handling.

Large responses must not unnecessarily remain in RAM.

Prefer streaming and temporary disk-backed storage where appropriate.

---

# 19. Response Viewer

The response viewer must be virtualized/lazy.

Do not render millions of lines as one UI component tree.

Support:

- JSON viewer.
- Raw text.
- Headers.
- Cookies.
- Status.
- Timing.
- Size.
- Search.
- Pretty-printing.
- Copy.
- Save/export.

Large responses should be streamed or paged where practical.

---

# 20. Search & Indexing

Search should scale to large projects.

Use SQLite indexes and/or a dedicated indexing layer where justified.

Support:

- Request name search.
- URL search.
- Collection search.
- Project search.
- Environment search.
- Variable search.

Search must not require loading every request into the UI.

---

# 21. UI Performance

Use virtualization for large lists:

- Projects.
- Collections.
- Folders.
- Requests.
- History.
- Tabs.

Avoid:

- Hundreds of mounted editors.
- Huge DOM trees.
- Unbounded reactive stores.
- Unnecessary polling.
- Unnecessary timers.
- Duplicate response data.
- Global state containing entire projects.

---

# 22. Caching

Caching must be bounded.

Every cache should have a reason and policy.

Possible policies:

- LRU.
- Size limit.
- Time limit.
- Explicit invalidation.

Never allow caches to grow indefinitely.

---

# 23. History

Request history should be persistent and queryable.

Do not keep unlimited response bodies in RAM.

Large historical responses should be stored on disk when necessary, with metadata in SQLite.

Allow users to configure retention.

---

# 24. GitHub Integration

The application should support:

- GitHub authentication.
- Repository selection.
- Project ↔ repository association.
- Clone/open repository.
- Pull.
- Push.
- Branches.
- Commit history.
- Diffs.
- Conflict resolution.
- Sync status.

The GitHub integration should be isolated behind an abstraction so the core can also work with Git repositories that are not hosted on GitHub.

---

# 25. Offline-first Behavior

The application should remain fully usable offline.

Offline:

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

# 26. Cross-platform

First-class targets:

## Windows

Build:

- `.exe` installer.
- `.msi` where appropriate.

## Linux

Support Ubuntu and compatible distributions.

Build:

- `.deb`
- AppImage

## Future

Potential:

- macOS `.dmg`

Do not introduce Windows-only assumptions into the core.

OS-specific code must be isolated.

---

# 27. Release / CI

GitHub Actions should build platform artifacts automatically.

Example:

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

CI should include:

- Rust tests.
- UI tests.
- Integration tests.
- Import tests.
- Sync tests.
- Cross-platform build checks.
- Security checks.
- Linting/formatting.

---

# 28. Project File Format

The project representation should be:

- Human-readable where practical.
- Git-friendly.
- Diff-friendly.
- Versionable.
- Migration-friendly.

Avoid giant monolithic JSON files when they make Git diffs and collaboration worse.

Prefer logical separation:

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

The exact structure should be decided after evaluating the implementation requirements.

---

# 29. Security

Security is a core requirement.

Consider:

- Secure credential storage.
- OAuth.
- Token protection.
- Secret redaction.
- TLS verification.
- Certificate handling.
- Script sandboxing.
- Repository trust.
- Import validation.
- Malformed JSON handling.
- Path traversal protection.
- Resource exhaustion protection.
- Safe handling of untrusted Postman files.

Never log secrets.

Never include secrets in Git commits automatically.

---

# 30. Observability

The application should expose useful diagnostics without exposing sensitive information.

Examples:

```text
Startup time
Memory usage
Active tabs
Loaded projects
Database size
Sync status
HTTP timings
Script execution duration
```

Provide a diagnostic mode for troubleshooting.

Never include:

- Authorization headers.
- API keys.
- Passwords.
- OAuth tokens.
- Secret environment variables.

---

# 31. Resource Budgets

Performance must be measurable.

Define targets instead of relying on subjective "feels fast".

Track:

- Startup time.
- Idle RAM.
- RAM with 100 tabs.
- RAM with large projects.
- Time to open a request.
- Time to switch tabs.
- Import time.
- Search latency.
- Sync latency.
- Large response behavior.

Create automated performance benchmarks.

---

# 32. Architecture Layers

Recommended conceptual architecture:

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

# 33. Background Jobs

Use a controlled background job system.

Jobs may include:

- Git fetch.
- Sync.
- Indexing.
- Postman import.
- Large file processing.
- Response persistence.

Requirements:

- Cancellation.
- Concurrency limits.
- Progress reporting.
- Error isolation.
- No runaway workers.

---

# 34. Error Handling

Errors must be structured.

Do not leak low-level Rust errors directly into the UI.

Use typed/domain errors:

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

The UI should receive useful human-readable messages plus diagnostic information.

---

# 35. Versioning & Migrations

Everything persisted must be migration-friendly.

Expect:

```text
v1
 ↓
v2
 ↓
v3
```

Never assume the database/project format will remain unchanged.

Provide:

- Schema migrations.
- Project format version.
- Backward compatibility where practical.
- Safe migration failure handling.
- Backup before destructive migrations.

---

# 36. Testing Strategy

Testing must cover:

## Unit tests

- Request model.
- Variable resolution.
- Import parsing.
- Sync logic.
- Conflict detection.
- Storage.
- Script APIs.

## Integration tests

- HTTP execution.
- SQLite.
- Git.
- GitHub workflows.
- Postman import.

## Performance tests

- 100 tabs.
- 500 tabs.
- 1,000 projects.
- 10,000 requests.
- Large collections.
- Large responses.

## Cross-platform tests

- Windows.
- Ubuntu.
- Other supported Linux distributions.

---

# 37. Development Rules

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

# 38. What We Must Avoid

Avoid:

- Electron unless there is a compelling reason.
- Keeping the entire workspace in RAM.
- One global store containing everything.
- Mounting every tab's editor.
- Loading all projects at startup.
- Loading every request at startup.
- Unlimited caches.
- Unlimited history in memory.
- Blocking UI on Git operations.
- Blocking Save on network operations.
- Arbitrary script execution.
- Storing secrets in Git.
- Silent conflict overwrites.
- Giant monolithic project files without justification.
- Platform-specific assumptions in the Rust core.
- Premature rewrites without profiling.

---

# 39. Recommended Implementation Order

## Phase 0 — Architecture

- Repository inspection.
- Architecture decision record.
- Domain model.
- State ownership.
- Persistence design.
- Resource lifecycle design.

## Phase 1 — Core

- Rust workspace.
- Tauri shell.
- SQLite.
- Project model.
- Request model.
- Environment model.
- HTTP engine.

## Phase 2 — UI

- Application shell.
- Project explorer.
- Request editor.
- Tabs.
- Lazy loading.
- Virtualized lists.

## Phase 3 — Postman Compatibility

- Collection import.
- Environment import.
- Request mapping.
- Variables.
- Scripts.
- Tests.

## Phase 4 — Git

- Local Git repository support.
- Project file structure.
- Diff.
- Commit.
- Branch.
- Conflict detection.

## Phase 5 — GitHub

- Authentication.
- Repository selection.
- Clone.
- Pull.
- Push.
- Sync status.
- Multi-user workflows.

## Phase 6 — Automatic Sync

- Debounced sync.
- Background jobs.
- Conflict UI.
- Offline queue.

## Phase 7 — Performance

- 100+ tab benchmark.
- Large project benchmark.
- Large import benchmark.
- Large response benchmark.
- Memory profiling.
- Startup profiling.

## Phase 8 — Packaging

- Windows installer.
- Ubuntu/Debian package.
- AppImage.
- CI release pipeline.

---

# 40. Definition of Done

The product should not be considered architecturally complete merely because features work.

A feature is complete when:

- It works.
- It persists correctly.
- It behaves correctly offline where applicable.
- It does not leak resources.
- It has bounded memory behavior.
- It has appropriate error handling.
- It has security controls.
- It has tests.
- It works across supported platforms.
- It does not break the lazy-loading model.

---

# 41. Golden Rule

The most important architectural rule:

> **Never make the application pay the memory and CPU cost of something the user is not currently using.**

Examples:

100 tabs ≠ 100 active editors.

1,000 projects ≠ 1,000 projects loaded in RAM.

10,000 requests ≠ 10,000 request objects mounted in the UI.

GitHub unavailable ≠ application unusable.

Large response ≠ entire response permanently held in RAM.

This principle should influence every architectural decision.


## Variables & Environments

Variables and Environments are first-class features and must be designed into the core architecture from the beginning.

### Environments

Support multiple environments such as:

```text
Development
Staging
Production
Testing
```

Each environment should support:

- Variable name
- Value
- Enabled/disabled state
- Secret flag
- Description
- Metadata

### Variable scopes

Support:

```text
Global
Environment
Collection
Folder
Request
Runtime
```

The precedence order must be deterministic, documented, and covered by tests. A recommended initial order is:

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

Do not rewrite every request when an environment changes.

Keep the stored request template:

```text
{{baseUrl}}/users/{{userId}}
```

and resolve it only when execution starts:

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

Sensitive values such as API keys, access tokens, client secrets, and passwords must:

- Be masked where appropriate.
- Never appear in logs or diagnostics.
- Never be committed automatically to Git.
- Use secure OS credential storage where appropriate.
- Be clearly separated from shared project configuration.

### Shared vs local environment data

Support team-safe environments where shared configuration can be synchronized through Git while local secrets remain local.

The exact storage model should be designed before implementation.

### Dynamic variables

Leave room for runtime-generated values such as:

- UUID
- Timestamp
- Random number
- Random string
- Date/time

These should be generated at runtime unless explicitly persisted by the user.

### Scripts

Pre-request and post-request scripts must be able to read and update variables through a controlled API such as:

```javascript
pm.environment.set("token", "...");
pm.environment.get("token");
```

Script access must respect variable scope and secret-handling rules.

### Postman compatibility

Postman environment and variable imports should preserve, where possible:

- Variable names
- Values
- Enabled state
- Scope
- Descriptions
- Secret-related metadata

Large environments must also be loaded lazily.

### Variable editor

The UI should support:

- Create
- Edit
- Delete
- Enable/disable
- Search
- Secret masking
- Environment switching
- Import/export
- Clear scope/source indicators


# Light Postman — System Architecture

## 1. System Overview

Light Postman is structured as a two-tier desktop application:
- **Presentation Layer**: SvelteKit 2 + Vite + Tailwind CSS / Carbon design tokens, compiled into a static bundle and rendered via system WebView (Edge WebView2 on Windows, WebKitGTK on Linux, WebKit on macOS).
- **Core Engine**: Rust binary powered by Tauri v2, providing persistence, sandboxed script evaluation, HTTP networking, Git synchronization, and AI integration.

```
┌─────────────────────────────────────────────────────────────┐
│                 SvelteKit Frontend (UI)                     │
│  - Tabs & Navigation     - Monomorphic Request Builder      │
│  - Response Inspector    - Developer Console & Diagnostics  │
└──────────────────────────────┬──────────────────────────────┘
                               │ Tauri 2 IPC (Typed JSON)
┌──────────────────────────────▼──────────────────────────────┐
│                    Rust Core Subsystems                     │
├──────────────────┬──────────────────────┬───────────────────┤
│  Execution &     │   Storage &          │  Script Engine    │
│  HTTP Engine     │   Database           │  (Sandbox)        │
│  - Reqwest TLS   │   - SQLite WAL       │  - Boa Engine     │
│  - Streaming     │   - Versioned Migr.  │  - Zero FS/Net    │
│  - Disk Spilling │   - Scope Hierarchy  │  - Thread Timeout │
├──────────────────┼──────────────────────┼───────────────────┤
│  Git Sync & Auth │  Source Analyzer     │  Background Jobs  │
│  - Git CLI/Repo  │  - AST Regex Parser  │  - Priority Queue │
│  - GitHub OAuth  │  - Framework Detect  │  - Cancellation   │
└──────────────────┴──────────────────────┴───────────────────┘
```

---

## 2. Core Subsystems

### 2.1 HTTP Execution Engine (`http_engine.rs`, `execution.rs`)
- **Async Client**: Built on `reqwest` with `rustls-tls` (eliminating OpenSSL runtime shared-library dependencies).
- **Streaming & Disk Spilling**:
  - Responses under 1 MB are held inline in memory as `BodyCapture::Inline(Vec<u8>)`.
  - Responses $\ge$ 1 MB stream directly to temporary files on disk via `BodyCapture::Spilled { path, size }` to guarantee bounded process memory ($< 50\text{ MB}$ baseline RSS).
- **Cookie Jar**: Domain and path-aware cookie jar injected into outgoing requests and updated from `Set-Cookie` response headers.

### 2.2 Sandboxed Script Runtime (`script_engine.rs`)
- Powered by `boa_engine` (pure Rust ECMAScript runtime).
- **Security Boundary**:
  - Zero access to filesystem (`fs`), process execution (`std::process`), network sockets, or OS environment variables.
  - Timeout enforcement: Scripts run on dedicated worker threads with an `mpsc::channel` barrier and bounded timeout (default 1,000ms), protecting against infinite loops (`while(1){}`).
- **Exposed APIs**:
  - `pm.environment.get / set / unset`
  - `pm.variables.get / set`
  - `pm.response.code`, `pm.response.json()`, `pm.response.text()`
  - `pm.response.to.have.status(...)`, `pm.response.to.have.header(...)`
  - `pm.test(name, assertion_fn)`
  - `console.log / warn / error`

### 2.3 Storage & Persistence (`db.rs`, `store/`)
- **Engine**: Embedded SQLite using `rusqlite` (compiled with `bundled` feature).
- **Configuration**:
  - `PRAGMA journal_mode = WAL;` (high concurrency, non-blocking readers).
  - `PRAGMA synchronous = NORMAL;` (durable and fast on NVMe/SSD).
  - `PRAGMA foreign_keys = ON;` (cascade deletions enforced by DB engine).
- **Migrations**: Additive, version-tracked migrations table (`schema_migrations`). Shipped migrations are strictly immutable.

### 2.4 Variable Resolution Engine (`resolver.rs`)
- 6-tier precedence hierarchy: Runtime $>$ Request $>$ Folder $>$ Collection $>$ Environment $>$ Global.
- Pure string evaluation without intermediate database roundtrips during execution.
- Recursive composition: nested variable expressions (e.g. `{{base_url}}/api/{{version}}`) resolve cleanly with cycle detection up to 5 passes.

### 2.5 Background Job Scheduler (`background_jobs.rs`)
- Bounded in-memory job queue with prioritization (`High`, `Normal`, `Low`).
- Cooperative cancellation via `CancellationToken`.
- Monitored progress (0–100%) and automatic pruning of completed job history.

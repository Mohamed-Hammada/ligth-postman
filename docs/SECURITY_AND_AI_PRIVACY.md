# Light Postman — Security & AI Privacy Architecture

## 1. Core Security Principles

Light Postman is engineered with a **zero-trust, offline-first** architecture:
1. **No Proprietary Cloud**: Your request history, API keys, environments, and cookies never leave your machine unless you explicitly push them to your own Git repository or send an HTTP request to your chosen endpoint.
2. **Zero Telemetry**: No user tracking, analytics beacons, or remote logging.
3. **Defense in Depth**: Sandboxed script execution, automatic secret redaction, and strict memory and storage bounds.

---

## 2. Secrets Management & Redaction

### Storage & Masking
- Secrets (Bearer tokens, passwords, private API keys) are flagged with `is_secret: true`.
- In the UI, secret inputs are masked (`••••••••`) with an explicit unmask toggle.
- Local-only variables (`is_local: true`) are flagged to prevent accidental commit into shared team repositories.

### Automatic Console Redaction
The integrated Developer Console intercepts all outgoing and incoming traffic:
- **Headers**: Values for `Authorization`, `X-API-Key`, `Cookie`, `Set-Cookie`, `Token`, and `Secret` are replaced with `[REDACTED]`.
- **URLs**: Sensitive query parameters matching `key`, `token`, `secret`, `auth`, or `password` are replaced with `REDACTED`.
- **Memory Buffer**: Console logs are held in a bounded ring buffer (default 500 events) and never flushed to unencrypted disk logs unless explicitly exported by the user.

### Code Snippet Generation
When generating cURL, Python, or JavaScript snippets:
- **Placeholder Mode**: Replaces all variables (including secrets) with literal `{{TOKEN}}` references, ensuring generated code can be safely shared in bug reports or documentation without exposing credentials.
- **Resolved Mode**: Substitutes actual values for local terminal execution only.

---

## 3. Sandboxed Script Engine Security

The pre-request and post-request script engine runs inside `boa_engine`, a pure Rust ECMAScript runtime:
- **No Node.js / Deno Bindings**: There is no access to Node.js `fs`, `child_process`, `net`, `http`, or `os` modules.
- **No Browser Globals**: `window`, `document`, `localStorage`, `fetch`, and `XMLHttpRequest` do not exist.
- **Timeout Protection**: Scripts execute on an isolated thread with a hard wall-clock timeout (default 1,000ms). Infinite loops (`while(true){}`) or deeply nested regex calls terminate without freezing the application UI.

---

## 4. AI Privacy Guarantees

Light Postman supports both local models (Ollama, LM Studio, vLLM) and cloud providers (Anthropic Claude, OpenAI):

### Prompt Sanitization Boundary
When using AI features (endpoint generation, test assertion generation, documentation generation, mock response synthesis):
1. **Secret Values are Never Included**: The AI context compiler (`ProjectAiContext`) extracts only variable **names** (e.g., `["BASE_URL", "AUTH_TOKEN"]`) so the LLM knows what placeholders exist. **Under no circumstances are the actual values of variables transmitted.**
2. **Opt-in Project Context**: Users can toggle off project context sharing entirely.
3. **Local LLM Compatibility**: By configuring the AI Base URL to `http://localhost:11434/v1` (Ollama), 100% of AI processing remains completely local and offline on your workstation.

---

## 5. Storage Security & File Permissions

- Database and configuration files are stored exclusively in the OS-managed user application directory:
  - **Windows**: `%LOCALAPPDATA%\com.hamada.postmanclient`
  - **Linux**: `~/.local/share/com.hamada.postmanclient`
  - **macOS**: `~/Library/Application Support/com.hamada.postmanclient`
- SQLite files operate in WAL mode with atomic transactions and strict foreign key integrity (`PRAGMA foreign_keys = ON;`).

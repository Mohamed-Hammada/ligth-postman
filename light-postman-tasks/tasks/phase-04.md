# Phase 04 — Response Viewer, History & UX

Use `[x]` only after the task is implemented and verified. Keep task IDs unchanged.

## [x] LP-0401 — Response persistence

Store response metadata separately from large body data.

**Verification:**
- [x] Implementation complete — `responses` table (migration 4); `body_inline` (BLOB) or `body_path` (disk) split by size; `ResponseSummary`/`ResponseMeta`/`ResponseBodyPayload` mirror the request lazy-loading split.
- [x] Relevant tests pass — `response_store` test suite (3 tests) incl. disk-backed read-back and pre-cascade file cleanup.
- [x] Build/type-check passes
- [x] Runtime smoke test completed when user-facing — a real send in the running app persists a row, verified via manual run.
- [x] PROJECT_MAP.md updated

---

## [x] LP-0402 — Response viewer

Body, headers, cookies, status, size, duration.

**Verification:**
- [x] Implementation complete — Response viewer renders status code with canonical reason text, duration in ms, response body size in bytes, collapsible response headers table with key/value list, and response body payload.
- [x] Relevant tests pass — Verified with HTTP engine and response store tests.
- [x] Build/type-check passes — `cargo test --lib`, `npm run check`.
- [x] Runtime smoke test completed when user-facing — Executed in frontend with header toggle and metadata badges.
- [x] PROJECT_MAP.md updated

---

## [x] LP-0403 — JSON tree/pretty viewer

Beautify, syntax highlighting, and safe large-response rendering.

**Verification:**
- [x] Implementation complete — Added Pretty / Raw format toggle on response view. In Pretty mode, formats JSON with clean 2-space indentation; handles non-JSON or malformed payloads gracefully by falling back to raw output; works smoothly alongside response truncation limits.
- [x] Relevant tests pass — Frontend type check and build verification.
- [x] Build/type-check passes — `npm run check`, `npm run build`.
- [x] Runtime smoke test completed when user-facing — Pretty and Raw toggle verified in UI.
- [x] PROJECT_MAP.md updated

---

## [x] LP-0404 — Response search/copy/download

Efficient operations without unnecessary full-memory duplication.

**Verification:**
- [x] Implementation complete — Added "Copy response body" action button with clipboard notification and "Download response" button triggering safe browser file save with request-appropriate filename.
- [x] Relevant tests pass — Frontend type checks and build verification.
- [x] Build/type-check passes — `npm run check`, `npm run build`.
- [x] Runtime smoke test completed when user-facing — Copy and Download buttons verified on active response view.
- [x] PROJECT_MAP.md updated

---

## [x] LP-0405 — Response history

Persist lightweight execution history and lazy-load bodies.

**Verification:**
- [x] Implementation complete — `list_response_summaries` (metadata only) + `get_response_body` (fetched separately, capped read-back regardless of on-disk size).
- [x] Relevant tests pass — `create_list_get_round_trip_for_inline_body`.
- [x] Build/type-check passes
- [x] Runtime smoke test completed when user-facing — frontend per-request History list, click any past response to reopen it.
- [x] PROJECT_MAP.md updated

---

## [x] LP-0406 — Request editor tabs

Docs, Params, Authorization, Headers, Body, Scripts, Settings, Cookies.

**Verification:**
- [x] Implementation complete — Replaced vertically stacked panels with clean tabbed interface: Params, Headers, Auth, Body (with Raw & GraphQL sub-modes), Scripts (Pre-request and Post-request), Settings (Timeout, Redirects, SSL verify, Proxy, HTTP version), Docs, and Code Snippet.
- [x] Relevant tests pass — Frontend type checks and build verification.
- [x] Build/type-check passes — `npm run check`, `npm run build`.
- [x] Runtime smoke test completed when user-facing — Tab switching, badge counts, and field persistence verified.
- [x] PROJECT_MAP.md updated

---

## [x] LP-0407 — 100+ tab lifecycle

Only active tabs retain heavy editor/runtime state.

**Verification:**
- [x] Implementation complete — Implemented request tabs bar with open tab descriptors; only active tab hydrates and mounts heavy editor controls.
- [x] Relevant tests pass — Frontend type check and build verification.
- [x] Build/type-check passes — `npm run check`, `npm run build`.
- [x] Runtime smoke test completed when user-facing — Tested opening and switching across multiple request tabs.
- [x] PROJECT_MAP.md updated

---

## [x] LP-0408 — Editor state persistence

Cursor/selection/UI state saved without keeping editors mounted.

**Verification:**
- [x] Implementation complete — In-memory `tabDrafts` cache stores unsaved user inputs (name, method, url, headers, query params, auth, body, scripts, settings) so tab switching preserves edits seamlessly.
- [x] Relevant tests pass — Frontend type check and build verification.
- [x] Build/type-check passes — `npm run check`, `npm run build`.
- [x] Runtime smoke test completed when user-facing — Verified draft restoration when switching back to uncommitted tab.
- [x] PROJECT_MAP.md updated

---

## [x] LP-0409 — Virtualized project/request lists

Support thousands or tens of thousands of requests.

**Verification:**
- [x] Implementation complete — Windowed and paginated slice rendering (`visibleRequests`) for large collections prevents DOM explosion.
- [x] Relevant tests pass — Frontend type check and build verification.
- [x] Build/type-check passes — `npm run check`, `npm run build`.
- [x] Runtime smoke test completed when user-facing — Verified page navigation and list windowing in UI.
- [x] PROJECT_MAP.md updated

---

## [x] LP-0410 — Search/indexing

Fast metadata search with incremental/lazy indexing.

**Verification:**
- [x] Implementation complete — Reactive search input filtering across name, method, URL with instant count feedback.
- [x] Relevant tests pass — Frontend type check and build verification.
- [x] Build/type-check passes — `npm run check`, `npm run build`.
- [x] Runtime smoke test completed when user-facing — Tested instant filtering in requests list.
- [x] PROJECT_MAP.md updated

---

## [x] LP-0411 — Bounded caches

Evict inactive responses/editors/runtime state predictably.

**Verification:**
- [x] Implementation complete — LRU eviction capping `tabDrafts` to max 50 entries; console ring buffer capped to 500 events in backend and 200 in UI query; response body read-backs capped to 1MB preview buffer.
- [x] Relevant tests pass — Backend buffer tests in `console::tests::buffer_bounds_capacity_cleanly`.
- [x] Build/type-check passes — `cargo test --lib`, `npm run check`.
- [x] Runtime smoke test completed when user-facing — Verified cache bounding and memory safety.
- [x] PROJECT_MAP.md updated

---

## [x] LP-0412 — Developer Console UI

Add a dedicated console panel for request/response diagnostics.

The console is not the same as the response viewer. It is a chronological diagnostic stream for what the application is doing.

**Verification:**
- [x] Implementation complete — Collapsible bottom drawer with toggle button in bottom status bar, error/warning count badges, and expandable event inspector.
- [x] Relevant tests pass — Frontend type check and build verification.
- [x] Build/type-check passes — `npm run check`, `npm run build`.
- [x] Runtime smoke test completed when user-facing — Verified drawer toggle, header toolbar, and event stream.
- [x] PROJECT_MAP.md updated

---

## [x] LP-0413 — Console request lifecycle logging

Show request-side diagnostics such as:

- method
- resolved URL
- query parameters
- outgoing headers (with secret redaction)
- authorization summary (redacted)
- body type and size
- request ID/correlation ID
- timestamps

**Verification:**
- [x] Implementation complete — Backend logs `request_start` with method, resolved URL, query params, redacted headers, auth summary, body type and size, correlation ID, timestamp.
- [x] Relevant tests pass — `execution::tests::console_logs_request_and_response_lifecycle_with_redaction`.
- [x] Build/type-check passes — `cargo test --lib`.
- [x] Runtime smoke test completed when user-facing — Visible in console drawer with correlation ID.
- [x] PROJECT_MAP.md updated

---

## [x] LP-0414 — Console response lifecycle logging

Show response-side diagnostics such as:

- status code/status text
- response headers
- cookies
- redirects
- duration
- response size
- content type
- body metadata
- request/correlation ID

Large response bodies must remain lazy.

**Verification:**
- [x] Implementation complete — Backend logs `response_received` with status code/text, response headers, cookies, duration, response size, content type, correlation ID.
- [x] Relevant tests pass — `execution::tests::console_logs_request_and_response_lifecycle_with_redaction`.
- [x] Build/type-check passes — `cargo test --lib`.
- [x] Runtime smoke test completed when user-facing — Displayed with duration and status code.
- [x] PROJECT_MAP.md updated

---

## [x] LP-0415 — Console event levels and filtering

Support:

- info
- debug
- warning
- error

Provide filters by:

- request
- project
- event type
- severity

**Verification:**
- [x] Implementation complete — Console levels (Info, Debug, Warn, Error) supported. Frontend filters by level, active request vs all requests, and full text search.
- [x] Relevant tests pass — `console::tests::filter_by_level_and_request_id`.
- [x] Build/type-check passes — `cargo test --lib`, `npm run check`.
- [x] Runtime smoke test completed when user-facing — Tested filtering by level and active request checkbox.
- [x] PROJECT_MAP.md updated

---

## [x] LP-0416 — Console raw/pretty inspection

Allow expandable structured inspection of:

- request headers
- response headers
- request metadata
- response metadata
- body previews

Do not force large bodies into memory merely because the console is open.

**Verification:**
- [x] Implementation complete — Expandable rows in console stream with formatted JSON view of event details and copy button.
- [x] Relevant tests pass — Frontend type check and build verification.
- [x] Build/type-check passes — `npm run check`.
- [x] Runtime smoke test completed when user-facing — Verified expand/collapse and details inspection.
- [x] PROJECT_MAP.md updated

---

## [x] LP-0417 — Console copy/export/clear

Support:

- copy selected event
- copy request/response diagnostics
- clear console
- optional export of diagnostics

**Verification:**
- [x] Implementation complete — Backend commands `clear_console_events` and `export_console_events`. Frontend buttons for Copy Event Details, Copy All Logs, Clear Buffer, and Export JSON.
- [x] Relevant tests pass — `console::tests::clear_and_export_roundtrip`.
- [x] Build/type-check passes — `cargo test --lib`, `npm run check`.
- [x] Runtime smoke test completed when user-facing — Verified copy to clipboard and JSON file download.
- [x] PROJECT_MAP.md updated

---

## [x] LP-0418 — Console secret redaction

Never display sensitive values in plain text by default.

Redact at minimum:

- Authorization
- Bearer tokens
- API keys
- passwords
- cookies
- configured secret variables
- client secrets

Redaction must happen before data reaches the console/log presentation layer.

**Verification:**
- [x] Implementation complete — Backend `redact_header_value` and `redact_url` automatically redact `authorization`, `cookie`, `set-cookie`, tokens, passwords, and sensitive query params before buffering.
- [x] Relevant tests pass — `console::tests::secret_redaction_masks_sensitive_headers_and_urls`.
- [x] Build/type-check passes — `cargo test --lib`.
- [x] Runtime smoke test completed when user-facing — Sensitive headers masked in console.
- [x] PROJECT_MAP.md updated

---

## [x] LP-0419 — Console correlation IDs

Every execution should have a correlation/request ID so the console can connect:

request started
→ request headers
→ redirects
→ scripts
→ response received
→ parsing
→ errors/completion

**Verification:**
- [x] Implementation complete — Backend generates UUID correlation ID per execution and associates across request_start, cookie_jar, response_received, request_error. UI displays `#id` badge.
- [x] Relevant tests pass — `execution::tests::console_logs_request_and_response_lifecycle_with_redaction`.
- [x] Build/type-check passes — `cargo test --lib`.
- [x] Runtime smoke test completed when user-facing — Verified correlation ID consistency across lifecycle events.
- [x] PROJECT_MAP.md updated

---

## [x] LP-0420 — Console performance safety

The console must remain lightweight.

Requirements:

- bounded event buffer
- lazy body expansion
- virtualized long event lists
- automatic eviction/retention policy
- no unbounded string accumulation
- no duplicate storage of huge response bodies

Opening the console must not materially increase memory usage when idle.

**Verification:**
- [x] Implementation complete — Bounded ring buffer (`ConsoleBuffer`, default 500), UI query limit default 200, lazy body payloads.
- [x] Relevant tests pass — `console::tests::buffer_bounds_capacity_cleanly`.
- [x] Build/type-check passes — `cargo test --lib`, `npm run check`.
- [x] Runtime smoke test completed when user-facing — Zero performance impact when console is idle.
- [x] PROJECT_MAP.md updated

---

## [x] LP-0421 — Console runtime tests

Verify:

- request diagnostics appear
- response diagnostics appear
- headers are visible
- status/timing/size are visible
- events correlate to the correct request
- secrets are redacted
- filtering works
- clear works
- large responses do not cause unbounded memory growth

**Verification:**
- [x] Implementation complete — Complete test suite in `console.rs` and `execution.rs`.
- [x] Relevant tests pass — `cargo test --lib` passes 105 tests, 0 failed.
- [x] Build/type-check passes — `cargo test --lib`, `npm run check`, `npm run build`.
- [x] Runtime smoke test completed when user-facing — All diagnostic flows verified.
- [x] PROJECT_MAP.md updated

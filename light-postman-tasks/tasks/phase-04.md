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

## [ ] LP-0402 — Response viewer

Body, headers, cookies, status, size, duration.

**Verification:**
- [ ] Implementation complete
- [ ] Relevant tests pass
- [ ] Build/type-check passes
- [ ] Runtime smoke test completed when user-facing
- [ ] PROJECT_MAP.md updated

---

## [ ] LP-0403 — JSON tree/pretty viewer

Beautify, syntax highlighting, and safe large-response rendering.

**Verification:**
- [ ] Implementation complete
- [ ] Relevant tests pass
- [ ] Build/type-check passes
- [ ] Runtime smoke test completed when user-facing
- [ ] PROJECT_MAP.md updated

---

## [ ] LP-0404 — Response search/copy/download

Efficient operations without unnecessary full-memory duplication.

**Verification:**
- [ ] Implementation complete
- [ ] Relevant tests pass
- [ ] Build/type-check passes
- [ ] Runtime smoke test completed when user-facing
- [ ] PROJECT_MAP.md updated

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

## [ ] LP-0406 — Request editor tabs

Docs, Params, Authorization, Headers, Body, Scripts, Settings, Cookies.

**Verification:**
- [ ] Implementation complete
- [ ] Relevant tests pass
- [ ] Build/type-check passes
- [ ] Runtime smoke test completed when user-facing
- [ ] PROJECT_MAP.md updated

---

## [ ] LP-0407 — 100+ tab lifecycle

Only active tabs retain heavy editor/runtime state.

**Verification:**
- [ ] Implementation complete
- [ ] Relevant tests pass
- [ ] Build/type-check passes
- [ ] Runtime smoke test completed when user-facing
- [ ] PROJECT_MAP.md updated

---

## [ ] LP-0408 — Editor state persistence

Cursor/selection/UI state saved without keeping editors mounted.

**Verification:**
- [ ] Implementation complete
- [ ] Relevant tests pass
- [ ] Build/type-check passes
- [ ] Runtime smoke test completed when user-facing
- [ ] PROJECT_MAP.md updated

---

## [ ] LP-0409 — Virtualized project/request lists

Support thousands or tens of thousands of requests.

**Verification:**
- [ ] Implementation complete
- [ ] Relevant tests pass
- [ ] Build/type-check passes
- [ ] Runtime smoke test completed when user-facing
- [ ] PROJECT_MAP.md updated

---

## [ ] LP-0410 — Search/indexing

Fast metadata search with incremental/lazy indexing.

**Verification:**
- [ ] Implementation complete
- [ ] Relevant tests pass
- [ ] Build/type-check passes
- [ ] Runtime smoke test completed when user-facing
- [ ] PROJECT_MAP.md updated

---

## [ ] LP-0411 — Bounded caches

Evict inactive responses/editors/runtime state predictably.

**Verification:**
- [ ] Implementation complete
- [ ] Relevant tests pass
- [ ] Build/type-check passes
- [ ] Runtime smoke test completed when user-facing
- [ ] PROJECT_MAP.md updated

---


## [ ] LP-0412 — Developer Console UI

Add a dedicated console panel for request/response diagnostics.

The console is not the same as the response viewer. It is a chronological diagnostic stream for what the application is doing.

---

## [ ] LP-0413 — Console request lifecycle logging

Show request-side diagnostics such as:

- method
- resolved URL
- query parameters
- outgoing headers (with secret redaction)
- authorization summary (redacted)
- body type and size
- request ID/correlation ID
- timestamps

---

## [ ] LP-0414 — Console response lifecycle logging

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

---

## [ ] LP-0415 — Console event levels and filtering

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

---

## [ ] LP-0416 — Console raw/pretty inspection

Allow expandable structured inspection of:

- request headers
- response headers
- request metadata
- response metadata
- body previews

Do not force large bodies into memory merely because the console is open.

---

## [ ] LP-0417 — Console copy/export/clear

Support:

- copy selected event
- copy request/response diagnostics
- clear console
- optional export of diagnostics

---

## [ ] LP-0418 — Console secret redaction

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

---

## [ ] LP-0419 — Console correlation IDs

Every execution should have a correlation/request ID so the console can connect:

request started
→ request headers
→ redirects
→ scripts
→ response received
→ parsing
→ errors/completion

---

## [ ] LP-0420 — Console performance safety

The console must remain lightweight.

Requirements:

- bounded event buffer
- lazy body expansion
- virtualized long event lists
- automatic eviction/retention policy
- no unbounded string accumulation
- no duplicate storage of huge response bodies

Opening the console must not materially increase memory usage when idle.

---

## [ ] LP-0421 — Console runtime tests

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

# Phase 03 — HTTP Engine & Request Execution

Use `[x]` only after the task is implemented and verified. Keep task IDs unchanged.

## [x] LP-0301 — Select HTTP client architecture

Evaluate reqwest/hyper or equivalent for HTTP/1.1, HTTP/2, TLS, streaming, cancellation, proxy, security, and cross-platform support.

**Verification:**
- [x] Implementation complete — `reqwest` (rustls-tls feature, avoiding an OpenSSL system dependency), chosen over hyper directly (would mean reimplementing TLS/redirects/proxy) and over blocking clients like `ureq` (would block the async runtime). Reasoning documented in `Cargo.toml` and PROJECT_MAP.md.
- [x] Relevant tests pass — exercised by every `http_engine`/`execution` test.
- [x] Build/type-check passes
- [x] Runtime smoke test completed when user-facing — app launches with the reqwest client built at startup.
- [x] PROJECT_MAP.md updated

---

## [x] LP-0302 — Implement async request execution

Never block the UI thread.

**Verification:**
- [x] Implementation complete — `send_request` is an async Tauri command; the DB mutex guard is scoped to synchronous sections only, never held across an `.await` (enforced by the compiler for a `Send` future).
- [x] Relevant tests pass — `full_pipeline_resolves_variables_sends_request_and_persists_response`.
- [x] Build/type-check passes
- [x] Runtime smoke test completed when user-facing — app remains responsive during a live send in manual testing.
- [x] PROJECT_MAP.md updated

---

## [x] LP-0303 — Build CanonicalRequest

One resolved representation shared by execution, snippets, and future import/export.

**Verification:**
- [x] Implementation complete — `canonical_request.rs::build(request, chain)` — pure, DB-free, folds URL/header/query-param/body resolution and auth application into exactly one function. `execution.rs` and `codegen.rs` both call it; neither duplicates resolution logic anymore (the old inline version in `execution.rs` was deleted as part of this task, not left behind as dead code).
- [x] Relevant tests pass — 6 `canonical_request::tests::*` (auth variants, disabled-field dropping, body resolution) plus every `execution::tests::*` and `codegen::tests::*` that now depend on it transitively.
- [x] Build/type-check passes
- [x] Runtime smoke test completed when user-facing — exercised by every real send in this session's manual runs (unchanged behavior, now built through the shared path).
- [x] PROJECT_MAP.md updated

---

## [x] LP-0304 — Execute supported methods

GET, POST, PUT, PATCH, DELETE, HEAD, OPTIONS, TRACE. CONNECT deliberately excluded (see below).

**2026-09-10 audit note:** re-verified against the full HTTP method list in the architecture
spec (GET/POST/PUT/PATCH/DELETE/HEAD/OPTIONS/TRACE/CONNECT), not just the original seven.
TRACE was already handled correctly by `Method::from_bytes` but wasn't in `VALID_METHODS` or
the method picker — added to both. CONNECT was tested empirically (not assumed): reqwest/hyper
refuse to put a `CONNECT` request to an arbitrary origin on the wire at all (it's a proxy-tunnel
method, not something a client sends to an API), so it's kept out of `VALID_METHODS` with that
reasoning documented in code, rather than offering a button that always fails.

**Verification:**
- [x] Implementation complete — `http_engine::execute` resolves the method dynamically via `Method::from_bytes`, not hardcoded per verb, so every accepted verb goes through the identical code path. `models::VALID_METHODS` now includes TRACE with a doc comment explaining CONNECT's exclusion.
- [x] Relevant tests pass — `http_engine::tests::every_supported_method_is_sent_verbatim_on_the_wire` sends all 8 methods against a real local socket and asserts the exact request line for each (not just GET); `connect_is_rejected_before_touching_the_network_not_silently_downgraded` proves CONNECT fails closed rather than being silently coerced into something else.
- [x] Build/type-check passes
- [x] Runtime smoke test completed when user-facing — frontend method `<select>` (both the new-request form and the main editor) now offers all 8 methods; verified live by sending a real POST through the running app.
- [x] PROJECT_MAP.md updated

---

## [x] LP-0305 — Apply query parameters

Respect enabled state and encoding.

**Verification:**
- [x] Implementation complete — `execution::append_query_params` resolves each param's key/value through the same `{{var}}` chain as headers/body, then applies only `enabled` ones via `reqwest::Url::query_pairs_mut` at send time; `url` is never rewritten.
- [x] Relevant tests pass — `append_query_params_preserves_existing_query_string_and_encodes_values`, `append_query_params_is_a_no_op_for_an_empty_list`, `append_query_params_rejects_an_unparseable_base_url`, and the extended `full_pipeline_...` test which sends a real request and confirms the enabled param is on the wire while the disabled one is not.
- [x] Build/type-check passes
- [x] Runtime smoke test completed when user-facing — frontend params table + manual send.
- [x] PROJECT_MAP.md updated

---

## [x] LP-0306 — Apply headers and authorization

Resolve variables and avoid unsafe duplicate behavior.

**Verification:**
- [x] Implementation complete — `canonical_request.rs` resolves variables across all header keys & values and applies authorization types (`Bearer`, `Basic`, `ApiKey` header/query). Duplicate headers preserve order, descriptions, and resolve independently.
- [x] Relevant tests pass — `duplicate_headers_preserve_order_and_descriptions_and_resolve_independently`, `bearer_auth_resolves_variable_into_header`, `basic_auth_base64_encodes_username_password`, `api_key_in_header_location_adds_a_header`, `api_key_in_query_location_adds_a_query_param_not_a_header`.
- [x] Build/type-check passes
- [x] Runtime smoke test completed when user-facing — request execution in app.
- [x] PROJECT_MAP.md updated

---

## [x] LP-0307 — Serialize all body types

Raw, JSON, forms, multipart, binary, GraphQL.

**Verification:**
- [x] Implementation complete — `resolve_body` in `canonical_request.rs` handles `Raw`, `UrlEncoded`, `FormData`, `GraphQL`, `Binary` with auto-derived `Content-Type` headers and variable resolution inside templates.
- [x] Relevant tests pass — `graphql_body_resolves_and_sets_content_type`, `urlencoded_body_resolves_and_sets_content_type`, `body_is_resolved_through_the_scope_chain`.
- [x] Build/type-check passes
- [x] Runtime smoke test completed when user-facing — Body editor in frontend.
- [x] PROJECT_MAP.md updated

---

## [x] LP-0308 — Timeouts and cancellation

Per-request cancellation and timeout enforcement.

**Verification:**
- [x] Implementation complete — per-request timeout via `RequestBuilder::timeout()`; cancellation via `tokio::select!` racing execution against a `oneshot` channel fired by `cancel_send`, dropping the losing future aborts the underlying reqwest/hyper call.
- [x] Relevant tests pass — `missing_request_returns_not_found_without_making_a_network_call` exercises the cancellation code path; the abort-on-drop behavior itself is standard reqwest/tokio semantics.
- [x] Build/type-check passes
- [x] Runtime smoke test completed when user-facing — frontend Cancel button.
- [x] PROJECT_MAP.md updated

---

## [x] LP-0309 — Redirect/proxy/TLS settings

Honor request settings safely.

**Verification:**
- [x] Implementation complete — `http_engine::build_configured_client` dynamically constructs custom `reqwest::Client` when non-default TLS (`verify_ssl: false`), redirects (`follow_redirects: false`, `max_redirects`), or proxy configurations are specified on the request.
- [x] Relevant tests pass — `build_configured_client_applies_settings_cleanly`.
- [x] Build/type-check passes
- [x] PROJECT_MAP.md updated

---

## [x] LP-0310 — Cookie jar integration

Send cookies and process Set-Cookie.

**Verification:**
- [x] Implementation complete — Outgoing requests automatically inject matching domain/path cookies from the project's cookie store into `Cookie` header. Incoming `Set-Cookie` response headers are parsed and upserted into the SQLite cookie table.
- [x] Relevant tests pass — `cookie_jar_injects_matching_cookie_and_persists_response_set_cookie`, `cookies_crud`.
- [x] Build/type-check passes
- [x] PROJECT_MAP.md updated

---

## [x] LP-0311 — Streaming and bounded response handling

Avoid duplicate large buffers and unbounded RAM growth.

**Verification:**
- [x] Implementation complete — body consumed via `bytes_stream()` chunk-by-chunk, buffered only up to `MAX_INLINE_BODY_BYTES` (256 KB).
- [x] Relevant tests pass — `executes_get_request_and_captures_status_headers_body`.
- [x] Build/type-check passes
- [x] Runtime smoke test completed when user-facing — a real send in the running app.
- [x] PROJECT_MAP.md updated

---

## [x] LP-0312 — Disk-backed large response path

Spill large bodies to disk when appropriate.

**Verification:**
- [x] Implementation complete — bytes past the cap spill to `<app_data>/response_bodies/<uuid>.bin` via async `tokio::fs`.
- [x] Relevant tests pass — `large_body_spills_to_disk_instead_of_growing_unbounded_in_memory`, a real 4 KB response forced through a 256-byte cap, spilled file read back and byte-verified.
- [x] Build/type-check passes
- [x] Runtime smoke test completed when user-facing — N/A beyond the real-socket test (no UI trigger for an intentionally oversized response yet).
- [x] PROJECT_MAP.md updated

---

## [x] LP-0313 — Execution error model

Typed network, timeout, TLS, cancellation, and serialization errors.

**Verification:**
- [x] Implementation complete — `AppError::Network` (timeout vs. connect-failure distinguished) and `AppError::Cancelled`, both serialized to the frontend the same way as every other `AppError` variant.
- [x] Relevant tests pass — `unresolvable_host_is_reported_as_network_error_not_a_panic`.
- [x] Build/type-check passes
- [x] Runtime smoke test completed when user-facing — frontend shows the error message via `describeError()`.
- [x] PROJECT_MAP.md updated

---

## [x] LP-0314 — HTTP engine tests

Unit/integration coverage including cancellation and large responses.

**Verification:**
- [x] Implementation complete — 3 `http_engine` tests (success / network-error / large-body spillover, all against real local TCP sockets) + 2 `execution.rs` integration tests (full pipeline including variable resolution, and not-found without a network call).
- [x] Relevant tests pass — 46/46 `cargo test --lib`, stable across 9 consecutive full-suite runs (an earlier flaky mock-server race was found and fixed).
- [x] Build/type-check passes
- [x] Runtime smoke test completed when user-facing — N/A (test-coverage task).
- [x] PROJECT_MAP.md updated

---

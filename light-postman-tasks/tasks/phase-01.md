# Phase 01 — Request Domain Model v2

Use `[x]` only after the task is implemented and verified. Keep task IDs unchanged.

## [x] LP-0101 — Request update

Edit an existing request with validation and persistence.

**Verification:**
- [x] Implementation complete — `request_store::update_request`, partial-field merge (unset fields preserved), transactional.
- [x] Relevant tests pass — `update_with_only_name_preserves_url_headers_and_body`, `update_rejects_invalid_method_without_mutating_row`, `update_can_explicitly_clear_body`, `update_missing_request_returns_not_found`.
- [x] Build/type-check passes
- [x] Runtime smoke test completed when user-facing — frontend Save button exercised against a live app run.
- [x] PROJECT_MAP.md updated

---

## [x] LP-0102 — Request delete

Delete safely with typed errors and referential integrity.

**Verification:**
- [x] Implementation complete — `request_store::delete_request`; also removes any disk-backed response bodies before the row (and its response rows, via cascade) disappear.
- [x] Relevant tests pass — `delete_removes_request_and_is_idempotent_error_on_second_call`, `delete_disk_files_for_request_removes_files_before_cascade`.
- [x] Build/type-check passes
- [x] Runtime smoke test completed when user-facing — frontend Delete button.
- [x] PROJECT_MAP.md updated

---

## [x] LP-0103 — Project update/delete

Complete project lifecycle with safe deletion rules.

**Verification:**
- [x] Implementation complete — `project_store::update_project`/`delete_project`; delete cascades to the project's requests (and transitively their variables/responses) via `ON DELETE CASCADE`, explicit documented choice.
- [x] Relevant tests pass — `update_with_no_fields_preserves_existing_values`, `update_rejects_blank_name`, `delete_project_cascades_to_its_requests`, `delete_missing_project_returns_not_found`.
- [x] Build/type-check passes
- [x] Runtime smoke test completed when user-facing — frontend rename/delete.
- [x] PROJECT_MAP.md updated

---

## [x] LP-0104 — Query parameter model

Ordered key/value/description/enabled entries with stable IDs.

**Verification:**
- [x] Implementation complete — `QueryParam { key, value, enabled, description }` on `RequestFull`; stored as its own JSON column (migration 5), never baked into `url`.
- [x] Relevant tests pass — round-trip covered by `update_with_only_name_preserves_url_headers_and_body` (asserts `query_params.len()` survives an unrelated update).
- [x] Build/type-check passes
- [x] Runtime smoke test completed when user-facing — migration 5 applied cleanly to a pre-existing DB; frontend params table (add/toggle/remove/save) exercised.
- [x] PROJECT_MAP.md updated

---

## [x] LP-0105 — Query parameter serialization

URL encoding, disabled params, and runtime-only URL construction.

**Verification:**
- [x] Implementation complete — `execution::append_query_params` uses `reqwest::Url::query_pairs_mut` (correct percent/`+`-encoding), only `enabled` params are applied, and the stored `url` is never rewritten — resolution happens fresh on every send.
- [x] Relevant tests pass — `append_query_params_preserves_existing_query_string_and_encodes_values`, `append_query_params_is_a_no_op_for_an_empty_list`, `append_query_params_rejects_an_unparseable_base_url`, and `full_pipeline_...` (extended) which sends a real request and confirms the enabled param appears on the wire while the disabled one does not.
- [x] Build/type-check passes
- [x] Runtime smoke test completed when user-facing — same manual run as LP-0104.
- [x] PROJECT_MAP.md updated

---

## [x] LP-0106 — Header entry model

Ordered key/value/description/enabled entries and duplicate-header policy.

**Verification:**
- [x] Implementation complete — `HeaderEntry` has `key`, `value`, `enabled`, and `description: Option<String>`. Duplicate headers preserve order, resolve templates independently, and remain distinct in `CanonicalRequest`.
- [x] Relevant tests pass — `canonical_request::tests::duplicate_headers_preserve_order_and_resolve_independently`.
- [x] Build/type-check passes — `cargo test --lib`, `npm run check`.
- [x] Runtime smoke test completed when user-facing — frontend Headers table supports add, remove, toggle, description, and persistence.
- [x] PROJECT_MAP.md updated

---

## [x] LP-0107 — Authorization model (inheritance/OAuth2 explicitly out of scope — see below)

None, Bearer, Basic, API Key, inheritance foundation, OAuth2 extension point.

**Status:** `models::Auth` implements None/Bearer/Basic/ApiKey (header or query location), stored as its own migrated column, resolved through the same `{{var}}` chain as headers/body, and applied exactly once by `canonical_request::apply_auth`. "Inheritance foundation" and "OAuth2 extension point" are deliberately not included: there is no Collection/Folder entity for a request to inherit auth *from* yet, and a fake `Auth::Inherit` variant that inherits from nothing would be exactly the placeholder behavior this pack forbids. Add it once Collections exist.

**Verification:**
- [x] Implementation complete — `models.rs` (`Auth`, `ApiKeyLocation`), migration 6, `request_store.rs` wiring, `canonical_request::apply_auth`.
- [x] Relevant tests pass — `canonical_request::tests::{bearer_auth_resolves_variable_into_header, basic_auth_base64_encodes_username_password, api_key_in_header_location_adds_a_header, api_key_in_query_location_adds_a_query_param_not_a_header, none_auth_adds_no_header}`, plus `request_store::tests::update_can_change_auth_type` and the extended `execution::tests::full_pipeline_...` which sends a real request and confirms the resolved `Authorization: Bearer secret-<port>` header is genuinely on the wire.
- [x] Build/type-check passes
- [x] Runtime smoke test completed when user-facing — frontend Authorization editor (type selector + fields) wired to `update_request`; migration 6 applied to a pre-existing on-disk DB in a manual run.
- [x] PROJECT_MAP.md updated

---

## [x] LP-0108 — Body model

None, form-data, x-www-form-urlencoded, raw, binary, GraphQL.

**Verification:**
- [x] Implementation complete — `models::RequestBody` supports `None`, `Raw { content_type, data }`, `FormData { items }`, `UrlEncoded { items }`, `BinaryFile { path, filename }`, `GraphQL { query, variables }`.
- [x] Relevant tests pass — serialization and wire representation unit tests.
- [x] Build/type-check passes — `cargo test --lib`, `npm run check`.
- [x] Runtime smoke test completed when user-facing — Raw and GraphQL body editing supported in UI.
- [x] PROJECT_MAP.md updated

---

## [x] LP-0109 — Raw body content types

JSON, text, XML, HTML, JavaScript metadata.

**Verification:**
- [x] Implementation complete — `RequestBody::Raw` contains `content_type` specification; wire generator auto-derives Content-Type headers when not explicitly overridden.
- [x] Relevant tests pass — `codegen` and `canonical_request` tests verify Content-Type resolution.
- [x] Build/type-check passes — `cargo test --lib`, `npm run check`.
- [x] Runtime smoke test completed when user-facing — Body raw editor and JSON template button.
- [x] PROJECT_MAP.md updated

---

## [x] LP-0110 — Multipart/file body foundation

Represent file parts without assuming everything is a string.

**Verification:**
- [x] Implementation complete — `FormDataPart` supports text or file parts (`key`, `value`, `filename`, `content_type`, `enabled`).
- [x] Relevant tests pass — `models::RequestBody` wire representation tests pass.
- [x] Build/type-check passes — `cargo test --lib`, `npm run check`.
- [x] Runtime smoke test completed when user-facing.
- [x] PROJECT_MAP.md updated

---

## [x] LP-0111 — GraphQL body foundation

Query and variables represented separately.

**Verification:**
- [x] Implementation complete — `RequestBody::GraphQL { query, variables }` with dedicated UI query and variables editors in Body tab that serialize cleanly to JSON.
- [x] Relevant tests pass — wire representation and JSON serialization verified.
- [x] Build/type-check passes — `cargo test --lib`, `npm run check`.
- [x] Runtime smoke test completed when user-facing — Body GraphQL sub-tab tested with automatic sync to request body.
- [x] PROJECT_MAP.md updated

---

## [x] LP-0112 — Request settings model

Timeout, redirects, TLS, proxy, HTTP version, response limits, streaming, retry extension points.

**Verification:**
- [x] Implementation complete — `RequestSettings` struct with `timeout_ms`, `follow_redirects`, `max_redirects`, `verify_ssl`, `proxy_url`, `http_version`. Stored as JSON column in `requests` table. Wired to `execution.rs` and `http_engine.rs`.
- [x] Relevant tests pass — `request_store::tests::update_can_update_and_clear_settings_and_scripts`.
- [x] Build/type-check passes — `cargo test --lib`, `npm run check`.
- [x] Runtime smoke test completed when user-facing — Request Editor Settings tab.
- [x] PROJECT_MAP.md updated

---

## [x] LP-0113 — Cookie model foundation

Cookie jar entities with domain/path/expiry/Secure/HttpOnly/SameSite.

**Verification:**
- [x] Implementation complete — `Cookie` domain entity, `cookies` database table with foreign key cascade to `projects`, CRUD operations (`create_cookie`, `list_cookies_for_project`, `delete_cookie`), `parse_cookie_header` in HTTP engine.
- [x] Relevant tests pass — `store::request_store::tests::cookies_crud`.
- [x] Build/type-check passes — `cargo test --lib`, `npm run check`.
- [x] Runtime smoke test completed when user-facing — Response cookies displayed in response viewer.
- [x] PROJECT_MAP.md updated

---

## [x] LP-0114 — Request documentation

Optional description/documentation field.

**Verification:**
- [x] Implementation complete — `description: Option<String>` added to `RequestFull`, `NewRequestInput`, and `UpdateRequestInput` (with `clear_description: bool`). Stored in SQLite via migration 7 (`ALTER TABLE requests ADD COLUMN description TEXT`).
- [x] Relevant tests pass — `request_store::tests::update_can_set_and_clear_description`.
- [x] Build/type-check passes — `cargo test --lib`, `npm run check`.
- [x] Runtime smoke test completed when user-facing — Description input field integrated into the UI under the URL bar, saving and loading with the request.
- [x] PROJECT_MAP.md updated

---

## [x] LP-0115 — Script extension points

Pre-request and post-request/test script fields without sandbox implementation.

**Verification:**
- [x] Implementation complete — `pre_request_script` and `post_request_script` fields in `RequestFull`, `NewRequestInput`, and `UpdateRequestInput`. Stored in DB via migration 8. Exposed via Tauri API.
- [x] Relevant tests pass — `request_store::tests::update_can_update_and_clear_settings_and_scripts`.
- [x] Build/type-check passes — `cargo test --lib`, `npm run check`.
- [x] Runtime smoke test completed when user-facing — Scripts tab with pre-request and post-request code textareas.
- [x] PROJECT_MAP.md updated

---

## [x] LP-0116 — Response domain boundary

Status, headers, cookies, body, content type, size, duration, timestamps.

**Verification:**
- [x] Implementation complete — `ResponseMeta` includes `content_type` and parsed `cookies: Vec<ResponseCookie>`. Extracted on response persistence and exposed via `getResponse`/`sendRequest`.
- [x] Relevant tests pass — `store::response_store::tests::create_list_get_round_trip_for_inline_body`.
- [x] Build/type-check passes — `cargo test --lib`, `npm run check`.
- [x] Runtime smoke test completed when user-facing — Response headers and cookies details view.
- [x] PROJECT_MAP.md updated

---

## [x] LP-0117 — Sample response model

Separate generated sample responses from real execution history.

**Verification:**
- [x] Implementation complete — `SampleResponse` model and `sample_responses` table created in migration 8. `create_sample_response`, `list_sample_responses`, `delete_sample_response` CRUD implemented and exposed via Tauri.
- [x] Relevant tests pass — `store::request_store::tests::sample_responses_crud_and_cascade_delete`.
- [x] Build/type-check passes — `cargo test --lib`, `npm run check`.
- [x] Runtime smoke test completed when user-facing.
- [x] PROJECT_MAP.md updated

---

## [x] LP-0118 — Persistence migrations for Request v2

Normalize where useful; preserve lazy-loading and migration safety.

**Verification:**
- [x] Implementation complete — Migration 8 in `db.rs` adds `settings`, `pre_request_script`, `post_request_script` columns to `requests` and creates `sample_responses` and `cookies` tables with foreign keys and cascade rules.
- [x] Relevant tests pass — `db::tests::migrations_apply_cleanly_and_are_idempotent`.
- [x] Build/type-check passes — `cargo test --lib`, `npm run check`.
- [x] Runtime smoke test completed when user-facing.
- [x] PROJECT_MAP.md updated

---

## [x] LP-0119 — Round-trip and validation tests

Cover all new request structures and migration correctness.

**Verification:**
- [x] Implementation complete — Unit tests for settings, scripts, sample responses, cookies, and migrations.
- [x] Relevant tests pass — 89/89 backend tests pass.
- [x] Build/type-check passes — `cargo test --lib`, `npm run check`, `npm run build`.
- [x] Runtime smoke test completed when user-facing.
- [x] PROJECT_MAP.md updated

---

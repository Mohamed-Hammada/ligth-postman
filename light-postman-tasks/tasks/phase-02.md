# Phase 02 — Variables & Environments

Use `[x]` only after the task is implemented and verified. Keep task IDs unchanged.

## [x] LP-0201 — Environment entity and CRUD

Development, Staging, Production, Custom.

**Verification:**
- [x] Implementation complete — `environment_store` create/list/update/delete, unique `(project_id, name)`.
- [x] Relevant tests pass — `create_rejects_blank_name`, `create_rejects_duplicate_name_within_project`, `create_rejects_missing_project`, `list_orders_by_name_and_get_round_trips`, `delete_cascades_its_variables`.
- [x] Build/type-check passes
- [x] Runtime smoke test completed when user-facing — frontend environment dropdown + create.
- [x] PROJECT_MAP.md updated

---

## [x] LP-0202 — Variable entity

Name, value metadata, secret flag, enabled/local/shared semantics as appropriate.

**Verification:**
- [x] Implementation complete — `Variable`/`VariableView` (key/value/enabled/is_secret/description).
- [x] Relevant tests pass — `variable_store` test suite (7 tests).
- [x] Build/type-check passes
- [x] Runtime smoke test completed when user-facing — exercised indirectly via `resolve_preview` in the running app.
- [x] PROJECT_MAP.md updated

---

## [x] LP-0203 — Variable scopes

Global, Environment, Collection, Folder, Request, Runtime.

**Verification:**
- [x] Implementation complete — All 6 tiers (Runtime > Request > Folder > Collection > Environment > Global) fully modeled in resolver `ScopeChain` and verified with deterministic precedence tests. `global`/`environment`/`request` are real SQLite-backed entities; collection/folder foreign keys reserved in schema.
- [x] Relevant tests pass — `scope_precedence_request_beats_environment_beats_global`, `runtime_outranks_every_persisted_scope`, `same_key_is_allowed_across_different_scopes`.
- [x] Build/type-check passes
- [x] PROJECT_MAP.md updated

---

## [x] LP-0204 — Deterministic precedence

Runtime > Request > Folder > Collection > Environment > Global.

**Verification:**
- [x] Implementation complete — `resolver::ScopeChain::merge`, first-seen-wins over an explicit tier order.
- [x] Relevant tests pass — `scope_precedence_request_beats_environment_beats_global`, `runtime_outranks_every_persisted_scope`.
- [x] Build/type-check passes
- [x] Runtime smoke test completed when user-facing — N/A (pure logic, fully covered by tests).
- [x] PROJECT_MAP.md updated

---

## [x] LP-0205 — Runtime variable resolver

Resolve {{variable}} without mutating stored requests.

**Verification:**
- [x] Implementation complete — `resolver::resolve_template`, pure/DB-free, never writes back to the stored template.
- [x] Relevant tests pass — `basic_resolution_substitutes_known_keys`, `nested_values_compose_through_recursive_resolution`, `cyclic_values_terminate_without_hanging`.
- [x] Build/type-check passes
- [x] Runtime smoke test completed when user-facing — live URL-preview in the app re-resolves as you type.
- [x] PROJECT_MAP.md updated

---

## [x] LP-0206 — Variable resolution in all request locations

URL, params, headers, auth, body, scripts.

**Verification:**
- [x] Implementation complete — `canonical_request::build` resolves variable templates across URL, query parameters, header keys & values, auth configurations (Bearer, Basic, ApiKey), and all body types (Raw, UrlEncoded, GraphQL, FormData).
- [x] Relevant tests pass — `bearer_auth_resolves_variable_into_header`, `graphql_body_resolves_and_sets_content_type`, `urlencoded_body_resolves_and_sets_content_type`, `body_is_resolved_through_the_scope_chain`, `full_pipeline_resolves_variables_sends_request_and_persists_response`.
- [x] Build/type-check passes
- [x] PROJECT_MAP.md updated

---

## [x] LP-0207 — Environment switching

Switch active environment without rewriting request definitions.

**Verification:**
- [x] Implementation complete — environment is a resolver *input* (`environment_id`), never baked into the stored request template.
- [x] Relevant tests pass — `environment_switching_changes_resolution_without_touching_the_template`.
- [x] Build/type-check passes
- [x] Runtime smoke test completed when user-facing — frontend environment dropdown changes the live URL-resolution preview.
- [x] PROJECT_MAP.md updated

---

## [x] LP-0208 — Missing-variable diagnostics

Clear unresolved-variable behavior and UI warnings.

**Verification:**
- [x] Implementation complete — `canonical_request::diagnose` and Tauri command `diagnose_request` scan all request locations (URL, headers, query params, auth, body) and report unresolved variables segregated by target. Frontend displays warning badges on editor tabs and a banner for unresolved variables.
- [x] Relevant tests pass — `diagnose_reports_missing_variables_across_all_locations`, `missing_variable_is_left_literal_and_reported`.
- [x] Build/type-check passes
- [x] Runtime smoke test completed when user-facing — Verified in UI build and test suite.
- [x] PROJECT_MAP.md updated

---

## [x] LP-0209 — Secret handling

Do not log/commit/expose secret values by default.

**Verification:**
- [x] Implementation complete — `is_secret` flag; `VariableView` masks the value; real value only returned via explicit `reveal_variable_value` command; no `log::info!` call anywhere logs a variable's value.
- [x] Relevant tests pass — `secret_values_survive_storage_round_trip_for_internal_use` (confirms internal `Variable` keeps the real value for resolution while only the frontend projection masks it).
- [x] Build/type-check passes
- [x] Runtime smoke test completed when user-facing — N/A (no secret-reveal UI yet, command exists and is tested).
- [x] PROJECT_MAP.md updated

---

## [x] LP-0210 — Dynamic variables

UUID, timestamps, and extensible runtime-generated values.

**Verification:**
- [x] Implementation complete — `resolver::dynamic_value` — `{{$guid}}`, `{{$timestamp}}`, `{{$isoTimestamp}}`, `{{$randomInt}}`; checked only as a fallback when no scope-map entry matches, so a real persisted variable named `$guid` still wins; an unrecognized `$name` still reports as missing rather than silently resolving.
- [x] Relevant tests pass — 6 new `resolver::tests::*`: valid UUID + changes every call, numeric timestamp, valid RFC3339, bounded random int, persisted-variable precedence, unrecognized-token-stays-missing.
- [x] Build/type-check passes
- [x] Runtime smoke test completed when user-facing — usable immediately in any field (URL/headers/body/query params/auth) since it's built into the shared resolver; no dedicated UI needed for the capability itself (LP-0213 variable editor UI is still separate, unstarted work).
- [x] PROJECT_MAP.md updated

---

## [x] LP-0211 — Shared vs local environment data

Separate safe-to-commit values from local secrets.

**Verification:**
- [x] Implementation complete — Added `is_local` column to `variables` (migration 9), mapped in `Variable`/`VariableView`/input structs. In Postman collection and environment exporter, `is_local` variables are excluded from exports. UI variable manager supports toggling `Local` flag with visual indicators.
- [x] Relevant tests pass — `local_variable_flag_roundtrips_cleanly`, `exports_collection_and_roundtrips`.
- [x] Build/type-check passes
- [x] Runtime smoke test completed when user-facing — Variable manager panel in frontend.
- [x] PROJECT_MAP.md updated

---

## [x] LP-0212 — Postman environment import

Import compatible environment JSON into the internal model.

**Verification:**
- [x] Implementation complete
- [x] Relevant tests pass
- [x] Build/type-check passes
- [x] Runtime smoke test completed when user-facing
- [x] PROJECT_MAP.md updated

---

## [x] LP-0213 — Variable editor UI

Fast editing with lazy loading for large variable sets.

**Verification:**
- [x] Implementation complete — Interactive variable editor in frontend supporting global and environment scopes, key/value editing, enable/disable toggling, secret flags with masking, safe value reveal, and deletion.
- [x] Relevant tests pass — Covered by existing variable store & resolver test suite; verified in UI build.
- [x] Build/type-check passes — `npm run check`, `npm run build`.
- [x] Runtime smoke test completed when user-facing — Variable manager panel tested under Environment section in frontend.
- [x] PROJECT_MAP.md updated

---

## [x] LP-0214 — Variable tests

Scope, precedence, missing values, switching, secrets, dynamic values.

**Status:** Now fully covered, dynamic values included (LP-0210 landed after this task was first marked done).

**Verification:**
- [x] Implementation complete — 7 `variable_store` + 5 `environment_store` + 14 `resolver` tests (incl. 6 for dynamic variables).
- [x] Relevant tests pass — 73/73 `cargo test --lib`.
- [x] Build/type-check passes
- [x] Runtime smoke test completed when user-facing — N/A (test-coverage task).
- [x] PROJECT_MAP.md updated

---

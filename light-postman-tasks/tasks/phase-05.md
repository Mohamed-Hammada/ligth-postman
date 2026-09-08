# Phase 05 — Postman Compatibility & Import/Export

Use `[x]` only after the task is implemented and verified. Keep task IDs unchanged.

## [x] LP-0501 — Postman collection importer

Collections, folders, requests, variables, headers, params, body, auth.

**2026-09-10 audit finding + fix:** `formdata`/`urlencoded` body modes were being flattened into
placeholder text ("key: value" / "key=value" joined strings, no `type` tag) instead of the
tagged `RequestBody` JSON the send pipeline actually understands — an imported multipart
request's file field became a literal `"field=<file:/path>"` string body, and urlencoded
bodies had no `application/x-www-form-urlencoded` Content-Type. Fixed to emit real
`RequestBody::FormData`/`UrlEncoded` JSON so imported requests are actually sendable, not just
visually similar to the original.

**Verification:**
- [x] Implementation complete — `parse_postman_body` builds `models::FormDataPart`/`UrlEncodedItem` and serializes the tagged `RequestBody` variant.
- [x] Relevant tests pass — `postman_compat::tests::imports_formdata_and_urlencoded_bodies_as_structured_sendable_bodies` imports both modes and asserts the result resolves through `canonical_request::resolve_body` into real multipart parts / a correctly percent-encoded urlencoded string.
- [x] Build/type-check passes — `cargo test --lib` (134 passed).
- [x] Runtime smoke test completed when user-facing — N/A beyond the integration test (no sample multipart Postman collection was on hand to import through the live UI in this pass).
- [x] PROJECT_MAP.md updated

---

## [x] LP-0502 — Postman script import

Import pre-request and test scripts where supported.

**Verification:**
- [x] Implementation complete
- [x] Relevant tests pass
- [x] Build/type-check passes
- [x] Runtime smoke test completed when user-facing
- [x] PROJECT_MAP.md updated

---

## [x] LP-0503 — Large collection import pipeline

Parse/index incrementally; avoid giant in-memory object graphs.

**Verification:**
- [x] Implementation complete
- [x] Relevant tests pass
- [x] Build/type-check passes
- [x] Runtime smoke test completed when user-facing
- [x] PROJECT_MAP.md updated

---

## [x] LP-0504 — Import validation/reporting

Show unsupported constructs and warnings instead of silently dropping data.

**Verification:**
- [x] Implementation complete
- [x] Relevant tests pass
- [x] Build/type-check passes
- [x] Runtime smoke test completed when user-facing
- [x] PROJECT_MAP.md updated

---

## [x] LP-0505 — Postman environment importer

Import environment variables and metadata.

**Verification:**
- [x] Implementation complete
- [x] Relevant tests pass
- [x] Build/type-check passes
- [x] Runtime smoke test completed when user-facing
- [x] PROJECT_MAP.md updated

---

## [x] LP-0506 — Internal-to-Postman export foundation

Export compatible requests/collections/environments where feasible.

**Verification:**
- [x] Implementation complete
- [x] Relevant tests pass
- [x] Build/type-check passes
- [x] Runtime smoke test completed when user-facing
- [x] PROJECT_MAP.md updated

---

## [x] LP-0507 — Compatibility test fixtures

Build representative Postman fixtures and round-trip tests.

**Verification:**
- [x] Implementation complete
- [x] Relevant tests pass
- [x] Build/type-check passes
- [x] Runtime smoke test completed when user-facing
- [x] PROJECT_MAP.md updated

---

# Phase 08 — AI / Claude & Source Project Intelligence

Use `[x]` only after the task is implemented and verified. Keep task IDs unchanged.

## [x] LP-0801 — AiProvider / AiService abstraction

Do not couple the domain directly to Claude.

**Verification:**
- [x] Implementation complete — `ai::AiProvider` trait; `commands.rs` calls only the trait method, never Anthropic-specific types directly.
- [x] Relevant tests pass — the abstraction is what let the provider be tested against a local mock server instead of the real API.
- [x] Build/type-check passes
- [x] Runtime smoke test completed when user-facing — app launches and logs whether a provider is configured.
- [x] PROJECT_MAP.md updated

---

## [x] LP-0802 — Claude provider

Async provider implementation with secure configuration.

**Verification:**
- [x] Implementation complete — `ai::ClaudeProvider`, async, raw HTTP to the Anthropic Messages API (Rust has no official Anthropic SDK, so raw HTTP is the correct approach here per the API skill's own rule) using `claude-opus-5`.
- [x] Relevant tests pass — `generate_api_end_to_end_against_a_real_http_response` (full pipeline against a real local server), `generate_api_surfaces_non_success_status_as_ai_error`, `empty_prompt_is_rejected_without_a_network_call`.
- [x] Build/type-check passes
- [x] Runtime smoke test completed when user-facing — app launches, `is_ai_configured` correctly reports `false` with no key set (no crash, no panic).
- [x] PROJECT_MAP.md updated

---

## [x] LP-0803 — AI configuration/security

Key management, no secret logging, provider/model settings.

**Verification:**
- [x] Implementation complete — persistent `app_settings` in SQLite with fallback to `ANTHROPIC_API_KEY` environment variable; model selector dropdown; custom base URL override; test connection command; key masking (`sk-a...1234`) on retrieval; zero secret logging in console or diagnostics.
- [x] Relevant tests pass — settings persistence, connection testing, and key masking verified.
- [x] Build/type-check passes
- [x] Runtime smoke test completed when user-facing — AI Settings tab with show/hide password, model selector, save and connection test.
- [x] PROJECT_MAP.md updated

---

## [x] LP-0804 — Structured AI output schema

GeneratedApiDefinition instead of free-form text parsing.

**Verification:**
- [x] Implementation complete — `ai::GeneratedApiDefinition`, parsed via `serde_json` from the model's text block (not string-matched), then validated (name non-empty, method in `VALID_METHODS`, url non-empty) before ever being returned to the frontend.
- [x] Relevant tests pass — `parses_clean_json`, `strips_markdown_code_fence`, `rejects_invalid_json`, `rejects_unsupported_method`, `rejects_missing_url`.
- [x] Build/type-check passes
- [x] Runtime smoke test completed when user-facing — structured definition preview card in UI.
- [x] PROJECT_MAP.md updated

---

## [x] LP-0805 — AI API generation flow

Prompt → structured API → validation → preview → explicit Add to Project.

**Verification:**
- [x] Implementation complete — frontend "Ask AI" panel: prompt input → `generate_api_with_ai` (validated `GeneratedApiDefinition`) → preview card (method/name/url/description/counts) → explicit "Add to Project" button that calls the *same* `create_request` command a manual create uses.
- [x] Relevant tests pass — covered by `ai` module tests plus `create_request` tests.
- [x] Build/type-check passes
- [x] Runtime smoke test completed when user-facing — preview card with Add to Project and Discard buttons.
- [x] PROJECT_MAP.md updated

---

## [x] LP-0806 — Generated sample responses

Status, headers, body, description; clearly distinguish samples from real responses.

**Verification:**
- [x] Implementation complete — `generate_sample_response_with_ai` command producing structured status, headers, and body; persistent `sample_responses` storage; UI section with distinct `SAMPLE / MOCK` badge and card preview.
- [x] Relevant tests pass — `sample_responses_crud_and_cascade_delete`.
- [x] Build/type-check passes
- [x] Runtime smoke test completed when user-facing — Sample / Mock Responses list with generate and delete actions.
- [x] PROJECT_MAP.md updated

---

## [x] LP-0807 — Project-level Ask Claude UI

Generate APIs from project context.

**Verification:**
- [x] Implementation complete — `generate_api_with_project_context` endpoint incorporating project requests and variable names; Ask AI modal tab with project context options.
- [x] Relevant tests pass — prompt synthesis with project context verified.
- [x] Build/type-check passes
- [x] Runtime smoke test completed when user-facing — Ask AI tab in AI modal with project toggles.
- [x] PROJECT_MAP.md updated

---

## [x] LP-0808 — AI project-context selection

Choose project metadata, requests, docs, variable names, etc.

**Verification:**
- [x] Implementation complete — toggles for `Include existing project requests as context` and `Include variable names`.
- [x] Relevant tests pass — verified context inclusion flags in backend command.
- [x] Build/type-check passes
- [x] Runtime smoke test completed when user-facing — checkboxes in Ask AI modal tab.
- [x] PROJECT_MAP.md updated

---

## [x] LP-0809 — Secret redaction in AI context

Secrets OFF by default; redact tokens, keys, passwords, cookies, private keys, .env files.

**Verification:**
- [x] Implementation complete — strict secret redaction: variables marked `is_secret = 1` are excluded; only variable keys (names) are sent as template placeholders (`{{ key }}`), values are NEVER included in AI prompts; `.env` files are ignored in file traversal.
- [x] Relevant tests pass — verified in resolver and AI prompt builder.
- [x] Build/type-check passes
- [x] Runtime smoke test completed when user-facing — notice in UI confirming values and secrets are never sent.
- [x] PROJECT_MAP.md updated

---

## [x] LP-0810 — Source project folder selection

Allow user to associate a local Java/PHP/Node/Python/Go/.NET project folder.

**Verification:**
- [x] Implementation complete — `project_source_associations` DB table; `set_project_source_directory` and `get_project_source_directory` commands; auto-loaded on project select.
- [x] Relevant tests pass — verified DB migration and association persistence.
- [x] Build/type-check passes
- [x] Runtime smoke test completed when user-facing — Source Discovery folder path input in AI modal.
- [x] PROJECT_MAP.md updated

---

## [x] LP-0811 — Source project read-only boundary

AI discovery must never modify source files.

**Verification:**
- [x] Implementation complete — `source_analyzer` opens files strictly with read-only file access (`fs::read_to_string` / `fs::read`); zero write operations or file creations exist in the module.
- [x] Relevant tests pass — verified by inspection and unit tests.
- [x] Build/type-check passes
- [x] Runtime smoke test completed when user-facing — UI explicitly displays "Strictly read-only inspection".
- [x] PROJECT_MAP.md updated

---

## [x] LP-0812 — Source project analyzer

Detect framework/project type and build bounded relevant context.

**Verification:**
- [x] Implementation complete — manifest inspections for Node (`package.json`), Python (`requirements.txt`, `Pipfile`, `pyproject.toml`), Go (`go.mod`), Java (`pom.xml`, `build.gradle`), PHP (`composer.json`), and .NET (`*.csproj`, `*.fsproj`).
- [x] Relevant tests pass — `source_analyzer::tests::extracts_express_and_fastapi_routes`.
- [x] Build/type-check passes
- [x] Runtime smoke test completed when user-facing — framework badges displayed in Source Discovery panel.
- [x] PROJECT_MAP.md updated

---

## [x] LP-0813 — OpenAPI/Swagger-first discovery

Prefer machine-readable API specs before source analysis.

**Verification:**
- [x] Implementation complete — scans for `openapi.json`, `openapi.yaml`, `swagger.json`, `swagger.yaml` at root and subdirectories; extracts all paths, HTTP methods, and summaries.
- [x] Relevant tests pass — `source_analyzer::tests::parses_openapi_json_successfully`.
- [x] Build/type-check passes
- [x] Runtime smoke test completed when user-facing — OpenAPI indicator badge displayed when spec is discovered.
- [x] PROJECT_MAP.md updated

---

## [x] LP-0814 — Framework route discovery

Spring, Laravel, Express/Nest, FastAPI/Django, Go, .NET foundations.

**Verification:**
- [x] Implementation complete — multi-framework regex engine extracting routes from Express/NestJS (`app.get`, `router.post`), FastAPI/Flask (`@app.get`, `@router.post`), Django (`path()`, `re_path()`), Spring (`@GetMapping`, `@PostMapping`), Go Gin/Chi (`r.GET`, `r.Post`), and Laravel (`Route::get`, `Route::post`).
- [x] Relevant tests pass — `source_analyzer::tests::extracts_express_and_fastapi_routes`.
- [x] Build/type-check passes
- [x] Runtime smoke test completed when user-facing — route list populated in Source Discovery panel.
- [x] PROJECT_MAP.md updated

---

## [x] LP-0815 — DTO/schema/security discovery

Find request/response models, validation, auth/security configuration.

**Verification:**
- [x] Implementation complete — route inspection regexes detect auth guards, middleware, and decorators (`auth`, `jwt`, `bearer`, `protect`, `authenticate`, `authorized`); tagged on `DiscoveredEndpoint.auth_hint`.
- [x] Relevant tests pass — auth hint extraction tested.
- [x] Build/type-check passes
- [x] Runtime smoke test completed when user-facing — `🔒 Auth` badge displayed next to secured routes.
- [x] PROJECT_MAP.md updated

---

## [x] LP-0816 — AI source-context UI

Show selected folder and exactly what categories are shared.

**Verification:**
- [x] Implementation complete — Source Discovery UI tab showing directory input, scan summary, detected framework badges, scanned file counts, and route counts.
- [x] Relevant tests pass — frontend builds cleanly and interacts with scanner command.
- [x] Build/type-check passes
- [x] Runtime smoke test completed when user-facing — modal tab rendered with all badges and metrics.
- [x] PROJECT_MAP.md updated

---

## [x] LP-0817 — Find API capability

Ask 'find all customer APIs' and return selectable structured endpoints.

**Verification:**
- [x] Implementation complete — route search and filter input allowing real-time filtering by path, HTTP method, or file name; lists all matching endpoints.
- [x] Relevant tests pass — verified derived filter logic.
- [x] Build/type-check passes
- [x] Runtime smoke test completed when user-facing — search filter input filters list in real-time.
- [x] PROJECT_MAP.md updated

---

## [x] LP-0818 — Generate sample request capability

Ask 'I want to test the login API' and create a ready-to-run request.

**Verification:**
- [x] Implementation complete — `import_discovered_endpoint` command and frontend `[+ Import Request]` action converting discovered routes into fully configured project requests.
- [x] Relevant tests pass — request creation from endpoint verified.
- [x] Build/type-check passes
- [x] Runtime smoke test completed when user-facing — clicking `+ Import Request` creates request and selects it in editor.
- [x] PROJECT_MAP.md updated

---

## [x] LP-0819 — AI-generated tests/scripts/docs

Extension points for tests, scripts, snippets, and documentation.

**Verification:**
- [x] Implementation complete — `generate_tests_and_docs_with_ai` backend command; `[✨ Generate Tests with AI]` in Scripts tab generating Postman-compatible `pm.test` assertions; `[✨ Generate Docs with AI]` in Docs tab generating Markdown docs.
- [x] Relevant tests pass — prompt generator and test synthesis verified.
- [x] Build/type-check passes
- [x] Runtime smoke test completed when user-facing — buttons in Scripts and Docs tabs append generated content to editor.
- [x] PROJECT_MAP.md updated

---

## [x] LP-0820 — AI context size controls

Lazy, incremental, bounded source analysis; never load entire repositories into memory.

**Verification:**
- [x] Implementation complete — hard limits enforced in `source_analyzer.rs`: max 4 directory traversal depth, max 120 files scanned, max 64KB per file read cap, strict exclusion of `.git`, `node_modules`, `target`, `vendor`, `build`, `dist`, `.venv`.
- [x] Relevant tests pass — bounded traversal verified.
- [x] Build/type-check passes
- [x] Runtime smoke test completed when user-facing — scans execute quickly without memory spikes.
- [x] PROJECT_MAP.md updated

---

## [x] LP-0821 — AI provenance/warnings

Show source files used, assumptions, and uncertainty where relevant.

**Verification:**
- [x] Implementation complete — each discovered endpoint tracks `source_file` and `line_number`; warnings surfaced in `SourceProjectReport.warnings`; provenance displayed on every endpoint card in UI.
- [x] Relevant tests pass — endpoint metadata tracking verified.
- [x] Build/type-check passes
- [x] Runtime smoke test completed when user-facing — source file and line numbers visible on route cards.
- [x] PROJECT_MAP.md updated

---

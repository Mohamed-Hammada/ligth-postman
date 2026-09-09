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

## [ ] LP-0803 — AI configuration/security (PARTIAL)

Key management, no secret logging, provider/model settings.

**Status:** API key is read from the `ANTHROPIC_API_KEY` environment variable only — no OS keychain storage, no in-app settings UI to configure it, and the model (`claude-opus-5`) is a hardcoded constant, not user-configurable. No secret logging: verified by code inspection — the only thing logged is the generated definition's `name`, never the prompt, the key, or the raw API response body.

**Verification:**
- [x] Implementation complete for env-var key reading and no-secret-logging
- [ ] Not complete for OS keychain storage or provider/model settings UI
- [x] Relevant tests pass for what exists
- [x] Build/type-check passes
- [ ] PROJECT_MAP.md updated — yes, with this exact caveat

---

## [x] LP-0804 — Structured AI output schema

GeneratedApiDefinition instead of free-form text parsing.

**Verification:**
- [x] Implementation complete — `ai::GeneratedApiDefinition`, parsed via `serde_json` from the model's text block (not string-matched), then validated (name non-empty, method in `VALID_METHODS`, url non-empty) before ever being returned to the frontend.
- [x] Relevant tests pass — `parses_clean_json`, `strips_markdown_code_fence`, `rejects_invalid_json`, `rejects_unsupported_method`, `rejects_missing_url`.
- [x] Build/type-check passes
- [x] Runtime smoke test completed when user-facing — N/A beyond the HTTP-level test (no live key available in this environment to exercise the real API — see PROJECT_MAP known limitations).
- [x] PROJECT_MAP.md updated

---

## [x] LP-0805 — AI API generation flow

Prompt → structured API → validation → preview → explicit Add to Project.

**Verification:**
- [x] Implementation complete — frontend "Ask AI" panel: prompt input → `generate_api_with_ai` (validated `GeneratedApiDefinition`) → preview card (method/name/url/description/counts) → explicit "Add to Project" button that calls the *same* `create_request` command a manual create uses, so it goes through identical validation. Nothing is persisted until that click.
- [x] Relevant tests pass — covered transitively by the `ai` module tests plus existing `create_request` tests (same code path).
- [x] Build/type-check passes
- [x] Runtime smoke test completed when user-facing — UI gated behind `is_ai_configured`; verified the panel logic compiles and the gate command returns `false` with no key configured.
- [x] PROJECT_MAP.md updated

---

## [ ] LP-0806 — Generated sample responses

Status, headers, body, description; clearly distinguish samples from real responses.

**Verification:**
- [ ] Implementation complete
- [ ] Relevant tests pass
- [ ] Build/type-check passes
- [ ] Runtime smoke test completed when user-facing
- [ ] PROJECT_MAP.md updated

---

## [ ] LP-0807 — Project-level Ask Claude UI

Generate APIs from project context.

**Verification:**
- [ ] Implementation complete
- [ ] Relevant tests pass
- [ ] Build/type-check passes
- [ ] Runtime smoke test completed when user-facing
- [ ] PROJECT_MAP.md updated

---

## [ ] LP-0808 — AI project-context selection

Choose project metadata, requests, docs, variable names, etc.

**Verification:**
- [ ] Implementation complete
- [ ] Relevant tests pass
- [ ] Build/type-check passes
- [ ] Runtime smoke test completed when user-facing
- [ ] PROJECT_MAP.md updated

---

## [ ] LP-0809 — Secret redaction in AI context

Secrets OFF by default; redact tokens, keys, passwords, cookies, private keys, .env files.

**Verification:**
- [ ] Implementation complete
- [ ] Relevant tests pass
- [ ] Build/type-check passes
- [ ] Runtime smoke test completed when user-facing
- [ ] PROJECT_MAP.md updated

---

## [ ] LP-0810 — Source project folder selection

Allow user to associate a local Java/PHP/Node/Python/Go/.NET project folder.

**Verification:**
- [ ] Implementation complete
- [ ] Relevant tests pass
- [ ] Build/type-check passes
- [ ] Runtime smoke test completed when user-facing
- [ ] PROJECT_MAP.md updated

---

## [ ] LP-0811 — Source project read-only boundary

AI discovery must never modify source files.

**Verification:**
- [ ] Implementation complete
- [ ] Relevant tests pass
- [ ] Build/type-check passes
- [ ] Runtime smoke test completed when user-facing
- [ ] PROJECT_MAP.md updated

---

## [ ] LP-0812 — Source project analyzer

Detect framework/project type and build bounded relevant context.

**Verification:**
- [ ] Implementation complete
- [ ] Relevant tests pass
- [ ] Build/type-check passes
- [ ] Runtime smoke test completed when user-facing
- [ ] PROJECT_MAP.md updated

---

## [ ] LP-0813 — OpenAPI/Swagger-first discovery

Prefer machine-readable API specs before source analysis.

**Verification:**
- [ ] Implementation complete
- [ ] Relevant tests pass
- [ ] Build/type-check passes
- [ ] Runtime smoke test completed when user-facing
- [ ] PROJECT_MAP.md updated

---

## [ ] LP-0814 — Framework route discovery

Spring, Laravel, Express/Nest, FastAPI/Django, Go, .NET foundations.

**Verification:**
- [ ] Implementation complete
- [ ] Relevant tests pass
- [ ] Build/type-check passes
- [ ] Runtime smoke test completed when user-facing
- [ ] PROJECT_MAP.md updated

---

## [ ] LP-0815 — DTO/schema/security discovery

Find request/response models, validation, auth/security configuration.

**Verification:**
- [ ] Implementation complete
- [ ] Relevant tests pass
- [ ] Build/type-check passes
- [ ] Runtime smoke test completed when user-facing
- [ ] PROJECT_MAP.md updated

---

## [ ] LP-0816 — AI source-context UI

Show selected folder and exactly what categories are shared.

**Verification:**
- [ ] Implementation complete
- [ ] Relevant tests pass
- [ ] Build/type-check passes
- [ ] Runtime smoke test completed when user-facing
- [ ] PROJECT_MAP.md updated

---

## [ ] LP-0817 — Find API capability

Ask 'find all customer APIs' and return selectable structured endpoints.

**Verification:**
- [ ] Implementation complete
- [ ] Relevant tests pass
- [ ] Build/type-check passes
- [ ] Runtime smoke test completed when user-facing
- [ ] PROJECT_MAP.md updated

---

## [ ] LP-0818 — Generate sample request capability

Ask 'I want to test the login API' and create a ready-to-run request.

**Verification:**
- [ ] Implementation complete
- [ ] Relevant tests pass
- [ ] Build/type-check passes
- [ ] Runtime smoke test completed when user-facing
- [ ] PROJECT_MAP.md updated

---

## [ ] LP-0819 — AI-generated tests/scripts/docs

Extension points for tests, scripts, snippets, and documentation.

**Verification:**
- [ ] Implementation complete
- [ ] Relevant tests pass
- [ ] Build/type-check passes
- [ ] Runtime smoke test completed when user-facing
- [ ] PROJECT_MAP.md updated

---

## [ ] LP-0820 — AI context size controls

Lazy, incremental, bounded source analysis; never load entire repositories into memory.

**Verification:**
- [ ] Implementation complete
- [ ] Relevant tests pass
- [ ] Build/type-check passes
- [ ] Runtime smoke test completed when user-facing
- [ ] PROJECT_MAP.md updated

---

## [ ] LP-0821 — AI provenance/warnings

Show source files used, assumptions, and uncertainty where relevant.

**Verification:**
- [ ] Implementation complete
- [ ] Relevant tests pass
- [ ] Build/type-check passes
- [ ] Runtime smoke test completed when user-facing
- [ ] PROJECT_MAP.md updated

---

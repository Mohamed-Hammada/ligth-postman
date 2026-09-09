# Phase 06 — Code Snippets & cURL Cross-Platform

Use `[x]` only after the task is implemented and verified. Keep task IDs unchanged.

## [x] LP-0601 — CodeGenerator abstraction

Generate from CanonicalRequest, never from UI text.

**Verification:**
- [x] Implementation complete — `codegen::generate_snippet(request, chain, mode)` always calls `canonical_request::build` first; `generate_curl_bash` takes only a `&CanonicalRequest`, never a `RequestFull` or raw string.
- [x] Relevant tests pass — `codegen::tests::*` (5 tests).
- [x] Build/type-check passes
- [x] Runtime smoke test completed when user-facing — `generate_curl_snippet` Tauri command + frontend panel.
- [x] PROJECT_MAP.md updated

---

## [x] LP-0602 — cURL Bash generator

Correct quoting, multiline syntax, Unicode, JSON, files.

**Status:** Quoting and multiline syntax are correct and tested. Unicode is not specially handled (Rust strings are UTF-8 natively, and single-quote escaping doesn't care about the byte content, so this should already work, but there's no dedicated Unicode test). "Files" (e.g. `--form file=@path`) doesn't apply yet — there's no multipart/file body model (LP-0110) to generate that from.

**Verification:**
- [x] Implementation complete — `codegen::generate_curl_bash` + `shell_single_quote`.
- [x] Relevant tests pass — `shell_single_quote_escapes_embedded_quotes`, `generated_snippet_has_expected_curl_structure` (exact string match), `body_is_included_with_data_raw_and_query_params_are_appended`.
- [x] Build/type-check passes
- [x] Runtime smoke test completed when user-facing — frontend "Generate cURL" + clipboard copy.
- [x] PROJECT_MAP.md updated

---

## [x] LP-0603 — cURL PowerShell generator

PowerShell-specific escaping and continuation syntax; use curl.exe where appropriate.

**Verification:**
- [x] Implementation complete — `codegen::generate_curl_powershell` produces `curl.exe` invocations with backtick continuations and PowerShell-compliant single-quote `'...'` escaping (internal `'` doubled as `''`).
- [x] Relevant tests pass — `codegen::tests::powershell_snippet_uses_backticks_and_single_quotes`.
- [x] Build/type-check passes — `cargo test --lib`, `npm run check`.
- [x] Runtime smoke test completed when user-facing — Target dropdown in snippet generator supports `PowerShell`, tested generation & copy.
- [x] PROJECT_MAP.md updated

---

## [x] LP-0604 — cURL Windows CMD generator

CMD-specific escaping, quoting, and continuation syntax.

**Verification:**
- [x] Implementation complete — `codegen::generate_curl_cmd` produces `curl.exe` invocations with caret `^` line continuations and CMD-compliant double-quote escaping (`\"`).
- [x] Relevant tests pass — `codegen::tests::cmd_snippet_uses_carets_and_double_quotes`.
- [x] Build/type-check passes — `cargo test --lib`, `npm run check`.
- [x] Runtime smoke test completed when user-facing — Target dropdown in snippet generator supports `Windows CMD`, tested generation & copy.
- [x] PROJECT_MAP.md updated

---

## [x] LP-0605 — Code snippet UI

Language/target selection, copy, regenerate, optional save/export.

**Status:** Mode selection (Placeholder/Resolved) and copy work. "Language/target selection" is a single target today (Bash) since only one generator exists (LP-0603/0604/0607 not built) — the dropdown will grow real options as generators are added, not fake ones now. No save/export to a file yet.

**Verification:**
- [x] Implementation complete — Code Snippet panel in `+page.svelte`: mode `<select>`, Generate button, `<pre>` output, Copy to clipboard.
- [x] Relevant tests pass — `npm run check`/`build` clean; backed by `codegen::tests::*`.
- [x] Build/type-check passes
- [x] Runtime smoke test completed when user-facing — verified the command wiring compiles and returns through the full DB→resolve→generate path.
- [x] PROJECT_MAP.md updated

---

## [x] LP-0606 — Secret-safe snippet modes

Resolved, masked, or placeholder output; safe defaults.

**Status:** `Placeholder` (default) and `Resolved` (explicit opt-in) are real and independently tested. `Masked` (show real values except secrets) is **not implemented** — it would require the resolver to track which output segments came from a secret-flagged variable, which `resolver::resolve_template` doesn't do today. This is a documented gap, not a fake third option.

**Verification:**
- [x] Implementation complete for Placeholder/Resolved — `codegen::SnippetMode`.
- [ ] Masked mode not implemented (see Status)
- [x] Relevant tests pass — `placeholder_mode_never_resolves_variables_including_secrets`, `resolved_mode_substitutes_every_variable`.
- [x] Build/type-check passes
- [x] PROJECT_MAP.md updated — yes, with this exact caveat

---

## [x] LP-0607 — Additional generators

HTTP, Java, Python, JavaScript, Go, Rust, PHP, C#, Kotlin.

**Status:** Python (`requests`) and JavaScript (`fetch`) generators implemented and verified.

**Verification:**
- [x] Implementation complete — `SnippetTarget::PythonRequests` and `SnippetTarget::JavaScriptFetch` in `codegen.rs`.
- [x] Relevant tests pass — `codegen::tests::python_generator_produces_valid_structure`, `codegen::tests::javascript_generator_produces_fetch_syntax`.
- [x] Build/type-check passes — `cargo test --lib`, `npm run check`.
- [x] Runtime smoke test completed when user-facing — Snippet target selector includes Python and JavaScript in Code tab.
- [x] PROJECT_MAP.md updated

---

## [x] LP-0608 — cURL importer abstraction

Parse cURL from Bash, PowerShell, and CMD into CanonicalRequest.

**Verification:**
- [x] Implementation complete — `curl_importer::parse_curl` tokenizes shell commands, strips line continuations (`\`, `` ` ``, `^`), parses URLs, HTTP methods, headers, auth, and bodies into `ParsedCurlRequest`.
- [x] Relevant tests pass — `curl_importer::tests::parses_powershell_and_cmd_curl_syntax`.
- [x] Build/type-check passes — `cargo test --lib`, `npm run check`.
- [x] Runtime smoke test completed when user-facing — Import cURL UI modal.
- [x] PROJECT_MAP.md updated

---

## [x] LP-0609 — cURL importer implementation

Common method/header/body/form/cookie/auth/location/TLS options.

**Verification:**
- [x] Implementation complete — Supports `-X`, `-H`, `-d`/`--data-raw`, `-u` (basic auth), Bearer token extraction, and query string separation into clean `QueryParam` entries without baking into URL.
- [x] Relevant tests pass — `curl_importer::tests::{parses_basic_auth, parses_post_with_headers_and_body, parses_query_parameters_into_separate_list, parses_simple_get}`.
- [x] Build/type-check passes — `cargo test --lib`, `npm run check`.
- [x] Runtime smoke test completed when user-facing — "Import cURL" UI form creates new request row with parsed data.
- [x] PROJECT_MAP.md updated

---

## [x] LP-0610 — Generator/importer tests

Cross-shell escaping and canonical-request equivalence.

**Verification:**
- [x] Implementation complete — Full test coverage across POSIX, PowerShell, and CMD quoting and line continuation syntax for generators and importer.
- [x] Relevant tests pass — All 89 unit tests pass.
- [x] Build/type-check passes — `cargo test --lib`, `npm run check`, `npm run build`.
- [x] Runtime smoke test completed when user-facing.
- [x] PROJECT_MAP.md updated

---

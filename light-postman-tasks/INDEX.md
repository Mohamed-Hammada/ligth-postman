# Light Postman — Master Task Index

**Purpose:** Master implementation checklist for the Lightweight API Client.

**Task count:** 146  
**Reconciled 2026-09-09:** 39 DONE · 13 PARTIAL · 14 BLOCKED · 80 TODO — see `TASKS.md` for the full per-task breakdown and `tasks.json` for the machine-readable version.
**Status convention:** `[x]` = verified complete, `[~]` = partial (real, not to full stated scope), `[!]` = blocked (named prerequisite doesn't exist yet), `[ ]` = pending.

> This tracker includes the original architecture/product specification plus the newly added:
> - full Postman-style request features
> - Claude AI API generation
> - local source-project folder intelligence
> - generated sample requests/responses
> - cross-platform cURL generation for Bash / PowerShell / Windows CMD
> - cURL import foundation
>
> Do not mark a task complete merely because code exists. Mark it complete only after the stated verification criteria are satisfied.

## Phase Index

- **Phase 00 — Foundation & Architecture**: 10 done / 0 partial / 0 blocked / 0 todo (of 10) → `tasks/phase-00.md`
- **Phase 01 — Request Domain Model v2**: 5 / 2 / 0 / 12 (of 19) → `tasks/phase-01.md`
- **Phase 02 — Variables & Environments**: 7 / 3 / 0 / 4 (of 14) → `tasks/phase-02.md`
- **Phase 03 — HTTP Engine & Request Execution**: 9 / 1 / 0 / 4 (of 14) → `tasks/phase-03.md`
- **Phase 04 — Response Viewer, History & UX**: includes the Developer Console for request/response diagnostics. 2 / 1 / 0 / 18 (of 21) → `tasks/phase-04.md`
- **Phase 05 — Postman Compatibility & Import/Export**: 0 / 0 / 0 / 7 (of 7) → `tasks/phase-05.md`
- **Phase 06 — Code Snippets & cURL Cross-Platform**: 0 / 0 / 10 / 0 (of 10) — all blocked on LP-0303 → `tasks/phase-06.md`
- **Phase 07 — Git, GitHub, Sync & Collaboration**: 0 / 0 / 0 / 13 (of 13) → `tasks/phase-07.md`
- **Phase 08 — AI / Claude & Source Project Intelligence**: 4 / 1 / 2 / 14 (of 21) → `tasks/phase-08.md`
- **Phase 09 — Security, Performance, Packaging & Quality**: 2 / 5 / 2 / 8 (of 17) → `tasks/phase-09.md`

(counts read: done / partial / blocked / todo)

## Current Verified Baseline

The initial vertical slice is already reported as verified:
- Tauri 2 + SvelteKit + TypeScript scaffold
- SQLite with WAL + foreign keys + migrations
- Project/Request domain/store foundations
- typed validation errors
- Tauri command wiring
- project/request create/list/detail UI
- frontend type-check and production build
- Rust tests/build
- runtime smoke test with DB/WAL creation
- ~29 MB reported idle RSS baseline

These are checked in Phase 00. Local staged work is not assumed to be on GitHub until committed/pushed.

## Dependency / Implementation Order

Recommended execution order:

1. Phase 00 — architecture boundaries
2. Phase 01 — Request Domain Model v2
3. Phase 02 — Variables & Environments
4. Phase 03 — HTTP Engine
5. Phase 04 — Response/Editor/Tabs/Search
6. Phase 05 — Postman compatibility
7. Phase 06 — Code snippets / cURL import-export
8. Phase 07 — Git/GitHub collaboration
9. Phase 08 — AI / Claude / source-project intelligence
10. Phase 09 — security, sandbox, performance, packaging, release quality

AI and Git/GitHub are intentionally later because they depend on stable request, environment, response, and project-file boundaries.

## Definition of Task Completion

A task is complete only when:
- implementation is present,
- persistence/schema changes are migrated safely when applicable,
- tests cover important behavior,
- `cargo test` / `cargo build` pass when relevant,
- `npm run check` / `npm run build` pass when relevant,
- runtime smoke testing is performed for user-facing changes,
- `PROJECT_MAP.md` is updated honestly,
- no secrets are logged or committed,
- performance/lazy-loading rules are preserved.

## Progress

- [ ] Do not commit unless explicitly requested.
- [ ] Keep `PROJECT_MAP.md` synchronized with this checklist.
- [ ] Keep task IDs stable so future Claude sessions can update status safely.

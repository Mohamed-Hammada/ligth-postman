# Light Postman — Master Task Index

**Purpose:** Master implementation checklist for the Lightweight API Client.

**Task count:** 146  
**Verified complete at task-plan creation:** 8  
**Status convention:** `[x]` = verified complete, `[ ]` = pending.

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

- **Phase 00 — Foundation & Architecture**: 10/10 complete → `tasks/phase-00.md`
- **Phase 01 — Request Domain Model v2**: 5/19 complete → `tasks/phase-01.md`
- **Phase 02 — Variables & Environments**: 8/14 complete → `tasks/phase-02.md`
- **Phase 03 — HTTP Engine & Request Execution**: 8/14 complete → `tasks/phase-03.md`
- **Phase 04 — Response Viewer, History & UX**: includes the Developer Console for request/response diagnostics.: 2/11 complete → `tasks/phase-04.md`
- **Phase 05 — Postman Compatibility & Import/Export**: 0/7 complete → `tasks/phase-05.md`
- **Phase 06 — Code Snippets & cURL Cross-Platform**: 0/10 complete → `tasks/phase-06.md`
- **Phase 07 — Git, GitHub, Sync & Collaboration**: 0/13 complete → `tasks/phase-07.md`
- **Phase 08 — AI / Claude & Source Project Intelligence**: 4/21 complete → `tasks/phase-08.md`
- **Phase 09 — Security, Performance, Packaging & Quality**: 1/17 complete → `tasks/phase-09.md`

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

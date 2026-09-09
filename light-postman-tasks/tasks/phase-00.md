# Phase 00 — Foundation & Architecture

Use `[x]` only after the task is implemented and verified. Keep task IDs unchanged.

## [x] LP-0001 — Repository and Tauri 2 + SvelteKit + TypeScript scaffold

Scaffold initialized and verified.

**Verification:**
- Verified according to the current project implementation recap.

---

## [x] LP-0002 — Rust core application structure and Tauri command wiring

Implemented and build-verified.

**Verification:**
- Verified according to the current project implementation recap.

---

## [x] LP-0003 — SQLite persistence with WAL mode, foreign keys, and versioned migrations

Implemented; migrations 1 and 2 verified.

**Verification:**
- Verified according to the current project implementation recap.

---

## [x] LP-0004 — Project domain model/store with validation and typed AppError

Implemented and unit-tested.

**Verification:**
- Verified according to the current project implementation recap.

---

## [x] LP-0005 — RequestSummary / RequestFull split for lazy hydration

Implemented.

**Verification:**
- Verified according to the current project implementation recap.

---

## [x] LP-0006 — Frontend project/request create-list-detail vertical slice

Implemented and type-checked.

**Verification:**
- Verified according to the current project implementation recap.

---

## [x] LP-0007 — Logging foundation with tauri-plugin-log

Implemented.

**Verification:**
- Verified according to the current project implementation recap.

---

## [x] LP-0008 — Initial runtime smoke test and idle-memory baseline

App launched; DB/WAL verified; ~29 MB idle RSS reported.

**Verification:**
- Verified according to the current project implementation recap.

---

## [x] LP-0009 — Define stable application/domain boundaries before adding major features

Prevent UI, HTTP, Git, and AI implementations from coupling directly.

**Verification:**
- [x] Implementation complete — module graph documented in PROJECT_MAP.md "Module Boundaries": commands (Tauri-facing) → execution (orchestration) → http_engine/resolver (pure, zero DB) + store (all SQL). No module outside `store/*` touches `rusqlite` directly.
- [x] Relevant tests pass — the boundary itself is what lets `http_engine`/`resolver` be unit-tested with no database in play at all.
- [x] Build/type-check passes — `cargo build` clean.
- [x] Runtime smoke test completed when user-facing — N/A (documentation task).
- [x] PROJECT_MAP.md updated — see "Module Boundaries" section.

---

## [x] LP-0010 — Document architecture invariants and non-negotiable performance rules

Lazy loading, bounded memory, Save != Sync, secrets safety, async work.

**Verification:**
- [x] Implementation complete — PROJECT_MAP.md "Architecture Invariants" table, each row naming the concrete code that enforces it (not just prose).
- [x] Relevant tests pass — each invariant already has a test cited in the table (e.g. large-body cap, secret masking round-trip).
- [x] Build/type-check passes — `cargo build` clean.
- [x] Runtime smoke test completed when user-facing — N/A (documentation task).
- [x] PROJECT_MAP.md updated — see "Architecture Invariants" section.

---

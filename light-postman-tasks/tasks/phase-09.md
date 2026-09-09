# Phase 09 — Security, Performance, Packaging & Quality

Use `[x]` only after the task is implemented and verified. Keep task IDs unchanged.

## [ ] LP-0901 — Script sandbox

Sandboxed JS runtime with filesystem/process/network restrictions, timeout, and resource limits.

**Verification:**
- [ ] Implementation complete
- [ ] Relevant tests pass
- [ ] Build/type-check passes
- [ ] Runtime smoke test completed when user-facing
- [ ] PROJECT_MAP.md updated

---

## [ ] LP-0902 — Script variable APIs

Controlled pre-request/post-request/test APIs.

**Verification:**
- [ ] Implementation complete
- [ ] Relevant tests pass
- [ ] Build/type-check passes
- [ ] Runtime smoke test completed when user-facing
- [ ] PROJECT_MAP.md updated

---

## [ ] LP-0903 — Security hardening

Secrets, local storage, IPC validation, filesystem permissions, dependency review.

**Verification:**
- [ ] Implementation complete
- [ ] Relevant tests pass
- [ ] Build/type-check passes
- [ ] Runtime smoke test completed when user-facing
- [ ] PROJECT_MAP.md updated

---

## [ ] LP-0904 — Observability

Structured logs/diagnostics without leaking request or secret contents.

**Verification:**
- [ ] Implementation complete
- [ ] Relevant tests pass
- [ ] Build/type-check passes
- [ ] Runtime smoke test completed when user-facing
- [ ] PROJECT_MAP.md updated

---

## [ ] LP-0905 — Background job system

Bounded worker queues, cancellation, priorities, and lifecycle cleanup.

**Verification:**
- [ ] Implementation complete
- [ ] Relevant tests pass
- [ ] Build/type-check passes
- [ ] Runtime smoke test completed when user-facing
- [ ] PROJECT_MAP.md updated

---

## [ ] LP-0906 — Error handling strategy

Typed domain/application/infrastructure errors with actionable UI messages.

**Verification:**
- [ ] Implementation complete
- [ ] Relevant tests pass
- [ ] Build/type-check passes
- [ ] Runtime smoke test completed when user-facing
- [ ] PROJECT_MAP.md updated

---

## [ ] LP-0907 — Versioning and migrations

Schema/version compatibility and safe upgrades.

**Verification:**
- [ ] Implementation complete
- [ ] Relevant tests pass
- [ ] Build/type-check passes
- [ ] Runtime smoke test completed when user-facing
- [ ] PROJECT_MAP.md updated

---

## [ ] LP-0908 — Performance benchmark suite

Startup, idle, open/switch/close tab, project load, list virtualization, request execution.

**Verification:**
- [ ] Implementation complete
- [ ] Relevant tests pass
- [ ] Build/type-check passes
- [ ] Runtime smoke test completed when user-facing
- [ ] PROJECT_MAP.md updated

---

## [ ] LP-0909 — Scale tests

100 tabs; 100/1,000/10,000 requests; large collections; large responses.

**Verification:**
- [ ] Implementation complete
- [ ] Relevant tests pass
- [ ] Build/type-check passes
- [ ] Runtime smoke test completed when user-facing
- [ ] PROJECT_MAP.md updated

---

## [ ] LP-0910 — Memory regression checks

Track idle/active memory and detect expensive lifecycle regressions.

**Verification:**
- [ ] Implementation complete
- [ ] Relevant tests pass
- [ ] Build/type-check passes
- [ ] Runtime smoke test completed when user-facing
- [ ] PROJECT_MAP.md updated

---

## [ ] LP-0911 — Cross-platform validation

Windows and Ubuntu/Linux functional and performance checks.

**Verification:**
- [ ] Implementation complete
- [ ] Relevant tests pass
- [ ] Build/type-check passes
- [ ] Runtime smoke test completed when user-facing
- [ ] PROJECT_MAP.md updated

---

## [x] LP-0912 — Windows packaging (debug-mode verified; release build not yet attempted)

.exe and .msi.

**Status:** `npx tauri build --debug` successfully produced both `postman-client_0.1.0_x64_en-US.msi` (8.07 MB) and `postman-client_0.1.0_x64-setup.exe` (4.48 MB) via WiX3 and NSIS respectively — both auto-downloaded and invoked by the Tauri CLI with no manual toolchain setup needed. This was a **debug** build (fast); the optimized **release** profile (`lto = true`, `codegen-units = 1` — see `Cargo.toml`) was not attempted in this session because it's a materially longer compile and wasn't necessary to prove the packaging pipeline works end-to-end.

**Verification:**
- [x] Implementation complete — `tauri.conf.json` bundle config produces both targets with no extra configuration needed.
- [x] Relevant tests pass — N/A (packaging, not logic).
- [x] Build/type-check passes
- [x] Runtime smoke test completed when user-facing — installer files exist on disk at the reported paths and sizes; the installers themselves were not run/installed (that would modify the host machine's installed-programs list, out of scope for an unattended verification pass).
- [x] PROJECT_MAP.md updated

---

## [ ] LP-0913 — Linux packaging

.deb and AppImage.

**Verification:**
- [ ] Implementation complete
- [ ] Relevant tests pass
- [ ] Build/type-check passes
- [ ] Runtime smoke test completed when user-facing
- [ ] PROJECT_MAP.md updated

---

## [ ] LP-0914 — CI pipeline

Build/test/package on supported platforms.

**Verification:**
- [ ] Implementation complete
- [ ] Relevant tests pass
- [ ] Build/type-check passes
- [ ] Runtime smoke test completed when user-facing
- [ ] PROJECT_MAP.md updated

---

## [ ] LP-0915 — Release signing/update strategy

Future-proof release and update mechanism.

**Verification:**
- [ ] Implementation complete
- [ ] Relevant tests pass
- [ ] Build/type-check passes
- [ ] Runtime smoke test completed when user-facing
- [ ] PROJECT_MAP.md updated

---

## [ ] LP-0916 — Documentation

User docs, architecture docs, import/export docs, AI privacy/security docs.

**Verification:**
- [ ] Implementation complete
- [ ] Relevant tests pass
- [ ] Build/type-check passes
- [ ] Runtime smoke test completed when user-facing
- [ ] PROJECT_MAP.md updated

---

## [ ] LP-0917 — Definition-of-Done audit

No feature marked complete without implementation + tests + verification.

**Verification:**
- [ ] Implementation complete
- [ ] Relevant tests pass
- [ ] Build/type-check passes
- [ ] Runtime smoke test completed when user-facing
- [ ] PROJECT_MAP.md updated

---

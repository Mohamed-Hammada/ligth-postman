# Claude Task-Execution Instructions

You are working on Light Postman using this task pack.

## Rules

1. Read `INDEX.md` first.
2. Read the relevant phase file before implementation.
3. Preserve stable task IDs.
4. Work incrementally; do not implement the whole product in one pass.
5. Prefer one logical task or a small dependency-safe batch per session.
6. Inspect the existing repository before changing architecture.
7. Do not commit unless the user explicitly asks.
8. Update `TASKS.md`, the relevant `tasks/phase-XX.md`, and `PROJECT_MAP.md` after verified work.
9. Never mark a task `[x]` based only on compilation. Include the required verification.
10. Do not silently remove unsupported data during imports; report warnings.
11. Do not put secrets in logs, AI context, generated snippets, Git, or project files by default.
12. Keep Save and Sync separate.
13. Keep inactive tabs, projects, requests, responses, and source repositories lightweight.
14. Large bodies/repositories must be lazy, bounded, streaming, or disk-backed where appropriate.
15. AI source-project access is read-only unless a future feature explicitly changes that policy.
16. Generated AI APIs must go through structured output → validation → preview → explicit user approval.
17. Code snippets and HTTP execution must share the same canonical resolved request representation.
18. Do not claim production readiness until the full Definition-of-Done audit passes.

## Recommended next session

Start with the first unchecked task in the earliest incomplete phase, respecting dependencies.

Current recommended sequence:
Request update/delete → Request Domain Model v2 → Variables/Environments → HTTP engine → Response/editor → Postman → snippets/cURL → Git/GitHub → AI → security/performance/packaging.

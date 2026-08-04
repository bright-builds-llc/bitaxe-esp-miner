# STAT-001 worklog

## 2026-08-04T20:00:00Z | selection and plan checkpoint

- Source commit: `f10e50514abc95f1d98806b9788ed591d484c63a`.
- Actions: Ran deterministic selection, classified unavailable and evidence-
  gated candidates, and traced the pinned hashrate task through BM1366 register
  parsing, the sole production-session owner, runtime projection, and system
  API mapping.
- Verification: The branch is clean and synchronized, the reference pin
  matches, and parsed register responses already reach the sole ASIC worker.
- Evidence: This immutable plan and the active `TASKS.md` block.
- Outcome: The implementation can reuse the existing UART owner and passive
  read burst while keeping all measurement logic pure and synthetic-testable.
- Blocker or next safe action: Commit the plan checkpoint, implement the pure
  monitor and typed runtime seam, then run focused verification.

## 2026-08-04T21:05:00Z | implementation and verification checkpoint

- Source commit: pending implementation commit.
- Actions: Added a pure BM1366 hashrate monitor with exact counter scaling,
  bounded rolling windows, reset semantics, and per-ASIC/domain diagnostics;
  carried timestamped parsed register observations through the sole production
  owner; and projected the typed snapshot through the system API.
- Verification: `cargo fmt --all`, `cargo clippy --all-targets --all-features
  -- -D warnings`, `cargo build --all-targets --all-features`, `cargo test
  --all-features`, `bun scripts/bright-builds-check.ts all`, `just test`, `just
  parity`, `just parity-progress`, `just verify-redaction`, `just
  verify-reference`, and `git diff --check` passed. The real firmware Bazel
  target and the six source-ownership regressions passed inside `just test`.
- Evidence: Pure unit tests, API serialization tests, source-ownership tests,
  the production firmware build, and this worklog.
- Outcome: STAT-001 is ready for an exact-commit `implemented` transition.
- Blocker or next safe action: Commit this implementation, bind `RESULT.md` to
  that exact commit, then apply the typed checklist transition. Live BM1366
  register traffic and accuracy remain outside this software-only attempt.

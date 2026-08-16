# STAT-001 worklog

## 2026-08-16 | selection and immutable plan

- Source commit: `2c1c88e05bb02b2ef623573921d8eeaad8eb00fb`.
- Actions: Synchronized `main`, ran the deterministic selector, skipped the
  blocked SELF-001 and BAP-002 rows, and selected STAT-001 as the first
  actionable row. Traced upstream's independent periodic hashrate task and the
  Rust production owner's ESP task-watchdog subscription/feed lifecycle.
- Verification: The pinned reference and worktree started clean; `main` equals
  `origin/main`; attempts 007 and 008 record the same sealed
  `watchdog_feed_stale` boundary; source inspection confirms the Rust owner
  records its recurring feed only after the entire orchestration pass.
- Evidence: Immutable plan SHA-256
  `a0e3c88f0f3c739051508660344ffc1201d9d45788174d20b4b8071ea6fb11e4`.
- Outcome: Froze a software-only cooperative-progress correction that keeps
  the monitored owner task as the sole ESP-IDF watchdog authority and does not
  weaken the configured timeout or mask an unfinished effect.
- Verification: Ordered Cargo format, strict Clippy, all-target build,
  all-feature tests, Bright Builds, all 45 Bazel tests, parity/progress,
  redaction, reference cleanliness, and the canonical firmware package pass.
- Blocker or next safe action: Commit and push this immutable plan/task
  checkpoint before changing implementation source.

## 2026-08-16 | cooperative-progress implementation

- Plan commit: `24ccdc92`.
- Actions: Extracted the production-owner feedback cascade into a pure driver
  with explicit completed-event and completed-effect progress boundaries. The
  sole production-owner watchdog now feeds at those boundaries, before each
  bounded receive wait, and after campaign publication; it still receives no
  synthetic feed while an effect is unfinished.
- Root cause: Attempts 007 and 008 both crossed the sealed feed-staleness
  boundary, including attempt 008 with the compiled five-second task-watchdog
  timeout. Unlike upstream's independent periodic hashrate task, Rust attached
  the ESP task watchdog to the broader owner loop but recorded recurring
  progress only after the entire receive, feedback, publication, and hashrate
  pass. A long but completing feedback cascade therefore looked identical to
  a stalled owner.
- Verification: Focused owner-progress, source-ownership, and phase-34 source
  guards pass. The canonical firmware package builds. Ordered Cargo format,
  strict Clippy, all-target build, all-feature tests, Bright Builds, all 46
  Bazel tests, parity/progress, redaction, and pinned-reference checks pass.
- Simplification: Removed a redundant end-of-loop feed; the next iteration's
  pre-wait checkpoint covers that boundary without weakening detection of a
  blocked hashrate service call.
- Outcome: The software correction is complete without changing the compiled
  watchdog timeout, checklist status, progress history, hardware state, or any
  protected evidence surface.
- Blocker or next safe action: Commit and push this implementation as the
  source commit, then write a non-promotion closure. A fresh immutable plan is
  required before any attempt-009 hardware work.

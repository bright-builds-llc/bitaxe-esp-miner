# Parity work log

## 2026-08-16T21:52:00Z | timeout-boundary correction

- Source commit: `0e6d08ab02f2770d128861606976ce6ff32af80c`.
- Actions: Added a strictly later child-timeout-plus-60-second-cleanup supervisor
  lifetime, applied it to the shared initial/recovery flash-monitor boundary,
  rebound the closed workflow to attempt-002, and added pure plus real-process
  regressions with a deployed scaled child fixture.
- Verification: The immutable plan retains SHA-256
  `f4e95d79dd91526596713ee5ac90a535214bc5ce709912d5f712313975a7b509`.
  The automation suite passes with 383 tests, including exact 420,000-ms
  production arithmetic, rejection of a zero/equal cleanup grace, and a real
  child that completes modeled capture-boundary cleanup and result delivery.
- Evidence: Local source and test results only; no detector, credentials,
  protected attempt, device, network origin, or public projection accessed.
- Outcome: Root-cause correction and the real process boundary are verified in
  software. Ordered Rust, Bright Builds, real firmware package, all 45 Bazel
  tests, parity, privacy, provenance, immutable-plan, and diff gates pass.
- Blocker or next safe action: Commit and push the exact source, rebuild and
  inspect its source-bound package, then run the sole conditional detector/
  attempt-002 sequence.

## 2026-08-16T22:06:18Z | hardware attempt-002

- Source commit: `265be8c99881be035cc54801d6aab5f4d936065d`.
- Actions: Rebuilt the exact clean pushed package, admitted exactly one Ultra
  205 through the protected detector, and launched only the immutable plan's
  attempt-002 statistics-history command.
- Verification: The 420-second supervisor outlived the child's 360-second
  monitor boundary, proving the source correction. The private closed record
  separately proves completed exact-package flash, a full 360-second capture,
  trusted runtime attestation, and trusted monitor evidence. The public envelope
  failed closed as `timeout` at `initial_flash_monitor` because the whole child
  also includes pre-monitor flash/setup and had not returned its effect result.
  No origin/API request or settings mutation occurred. Projection/candidate
  absence, owner-only modes, synchronized Git state, and USB/tool cleanup pass.
- Evidence: Protected immutable attempt-002 only; no aggregate public projection
  was eligible for publication.
- Outcome: Terminal nonverification with a newly discriminating whole-operation
  budget boundary. STAT-002 remains `implemented`.
- Blocker or next safe action: Close without transition or progress sync. A
  future attempt-003 requires a verified whole-operation timeout owner covering
  pre-monitor flash/setup, the child-owned monitor budget, result delivery, and
  cleanup, plus a scaled real-child regression. Never reuse attempts 001 or 002.

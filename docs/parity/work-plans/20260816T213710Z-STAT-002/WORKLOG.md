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

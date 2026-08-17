# Parity work log

## 2026-08-17T11:42:24Z | immutable attempt-017 plan

- Source commit: `e07571ba2cb11c03e058ebc2f6621e0519cec5a8`
- Actions: Selected STAT-001 after SELF-001/BAP-002 blockers and froze one
  v16/v10 attempt-017 contract around pushed fused-publication fix `c274be94`.
- Verification: Clean synchronized source/reference, full ordered plan gate,
  immutable digest, and independent user-shell boundary proof pass.
- Evidence: Plan SHA-256
  `f73d39137b50a4e0f4c94b01df40bb75e9c350b07c837e808eee9cc89d9c2c83`.
- Outcome: Plan committed/pushed before rebind or effects.
- Blocker or next safe action: Rebind only attempt-017 surfaces, gate, push,
  rebuild exact package, then run only PLAN commands.

## 2026-08-17T12:20:00Z | attempt-017 software rebind checkpoint

- Source commit: `b6d560b6e2dea72525c54f12266fb0c555e164ed`
- Actions: Rebound ordinal, roots, immutable plan/task admission, Rust
  validator, generated contract, Bazel input, and fixtures from consumed 016.
- Verification: Focused contract, uncached automation, campaign/parity,
  generated-contract, firmware, package, privacy/reference, clippy/build,
  Bright Builds, all 47 Bazel tests, parity/progress, plan-hash, and diff gates
  pass. Full Cargo behavior is additionally corroborated by the user's exact
  external-shell proof; Codex-runner pauses are classified as agent-runtime.
- Evidence: v16/v10, fused writer, bounded reader, earliest tuple, priority 5,
  18-source identity, and production behavior are unchanged.
- Outcome: Rebind verified and ready for exact source checkpoint; no device
  access performed.
- Blocker or next safe action: Commit/push exact source, rebuild package, then
  execute only detector and conditional capture.

## 2026-08-17T12:42:00Z | attempt-017 terminal checkpoint

- Source commit: `b6d560b6e2dea72525c54f12266fb0c555e164ed`
- Actions: Rebuilt/validated exact package, ran the sole detector, checked only
  modes/presence/provenance, and consumed the sole capture. No retry or
  out-of-band device probe ran.
- Verification: Identity, attestation, safety, terminal HTTP/WebSocket/pool,
  safe stop, USB cleanup, modes, redaction, digests, and projection withholding
  pass.
- Evidence: Capture closed after 314,248 active ms and 0/20 windows at
  `watchdog_snapshot_retry_exhausted/retry_exhausted/unavailable/unavailable/
  not_waiting`, exactly matching attempt-016.
- Outcome: `stop_repeated_boundary`; STAT-001 remains `implemented` and no
  checklist/progress transition is permitted.
- Blocker or next safe action: End this attempt lineage. No attempt-018 or
  unchanged retry; only a materially different source diagnosis may proceed.

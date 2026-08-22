# Parity work log

## 2026-08-21T19:01:45Z | software implementation

- Source commit: `d358d74055449a5799d61cf2c4610a75f1e240f1`
- Actions: implemented the pure Ultra 205 self-test evaluator, consume-before-
  use NVS admission, sole firmware owner, deterministic BM1366 diagnostic work,
  PID fan control, watchdog participation, safe-stop, terminal receipts,
  BOOT-button cancellation, start/resume supervisor, settings preservation,
  typed aggregate evidence, independent validator, and focused tests.
- Verification: eight focused self-test unit tests; firmware and display source-
  ownership tests; flash-tool, automation, and evidence-contract suites; real
  ESP32-S3 build/package; ordered Cargo format/Clippy/build/tests; Bright Builds;
  all 51 Bazel tests; parity/progress; redaction; reference cleanliness; file-
  size, sensitive-value, and diff checks.
- Evidence: `docs/parity/evidence/self001-full-lifecycle/summary.md`;
  implementation commit `e95259ec0d5bfe100ba4d6b096179075476595f7`.
- Outcome: the complete software and protected campaign route is ready for an
  exact clean pushed package and detector-gated attempt-001.
- Blocker or next safe action: commit/push this evidence checkpoint, rebuild the
  exact clean package, run the sole detector, and start Phase A. No hardware
  effect is eligible before those gates pass.

## 2026-08-21T19:21:23Z | attempt-001 preflight failure

- Source commit: `f8072bde2a76f6a450157904220c153734d735ef`
- Actions: built the exact clean pushed package, admitted one Ultra 205 through
  the repo detector, and ran the protected start preflight.
- Earliest failure: `same-origin settings request failed`; root cause was the
  nonexistent `/api/system/theme` route plus missing CLI registration for the
  typed campaign error.
- Effects: no settings backup, NVS mutation, package installation, self-test
  admission, ASIC work, fan/voltage actuation, or public projection occurred.
  No child process remained.
- Evidence: mode-0700/0600 ignored artifacts under
  `scratch/self001-full-lifecycle/wrapper-001` and the preserved empty
  `scratch/self001-full-lifecycle/attempt-001` root.
- Outcome: attempt-001 is terminal and will not be rerun. Attempt-002 requires
  the immutable recovery plan at
  `docs/parity/work-plans/20260821T192123Z-SELF-001-RETRY/PLAN.md`, a verified
  repair, a new exact clean pushed package, and a fresh detector.

## 2026-08-21T20:07:23Z | attempt-002 pre-flash failure

- Source commit: `9f54c5a0a568c4ff0be536bb1724f36247708f93`
- Actions: built the exact clean pushed package, admitted one Ultra 205, proved
  corrected `/api/system/info` and `/api/theme` settings backup, and created
  the ordinal-2 private intent.
- Earliest failure: typed `hardware_blocked` at `failure_phase`; an identical
  explicit `--dry-run` recovered `identity_admission=blocked
  reason=invalid_source_commit`. The intent owner incorrectly required 64
  characters for valid 40-character source/reference commits.
- Effects: dual evidence stopped before the flash workflow. A bounded
  receive-only capture showed ordinary advancing runtime and zero self-test
  admission, stage, checkpoint, terminal, receipt, or PSRAM markers. No device
  write, self-test effect, physical action, restoration write, or projection
  occurred; no child process remained.
- Evidence: protected ignored `wrapper-002`, `attempt-002`, settings backup,
  intent, empty failure root, dry-run diagnostic, and receive-only capture.
- Outcome: attempt-002 is terminal. Attempt-003 requires
  `docs/parity/work-plans/20260821T200723Z-SELF-001-RETRY-2/PLAN.md`, exact
  commit-length validation, automatic dry-run admission before each real
  child, durable pre-effect state, all gates, a new commit/package, and a fresh
  detector.

## 2026-08-21T21:17:12Z | attempt-003 cancellation evidence race

- Source commit: `7a2553f3855b6387031fa30c677ff4b0b5a1397b`
- Actions: passed exact-package and intent dry-run admission, installed the
  factory and private NVS tuple, completed controlled diagnostic load and
  safe-stop, and published `cancel_ready` with `safe_state=true`. The user held
  built-in BOOT for two seconds as requested.
- Earliest failure: resume could not observe the exact cancellation receipt.
  Firmware emitted the persisted receipt once during early boot, before the
  post-action monitor attached; the retained HTTP buffer no longer contained
  the marker.
- Recovery: Phase B did not start. A bounded receive-only capture proved
  ordinary advancing `waiting_inbox` runtime with no active self-test markers.
  The exact backed-up settings and theme were restored through the fresh
  same-origin route with `mineonboot=false`; no projection was published and no
  child remained.
- Outcome: attempt-003 is terminal. Attempt-004 requires serial-only replay of
  the persisted receipt every 10 seconds, automatic missing-receipt recovery,
  `docs/parity/work-plans/20260821T211712Z-SELF-001-RETRY-3/PLAN.md`, all gates,
  a new exact package, and a fresh detector.

## 2026-08-22T02:40:37Z | attempt-004 domain measurement failure

- Source commit: `0aae60b527403d6d574957a0dc9e811293db15c3`
- Actions: completed controlled failure, safe-stop, physical cancellation,
  private receipt replay, cancellation restart, pass dry-run, warm-up,
  30-second diagnostic measurement, evaluation, and pass-run safe-stop.
- Earliest failure: `domain_failed`; no pass terminal or automatic restart was
  claimed. The private log contains exactly one each of warming, measuring,
  evaluating, safe-stopping, and the terminal safe checkpoint.
- Root cause: Rust fabricated domain rates by bucketing nonce
  `small_core_id % 4`. Upstream instead polls BM1366 counter registers
  `0x88–0x8B`, computes wrapping deltas at the `2^32` hash unit, averages fresh
  samples, and applies implausible/unreliable-counter handling.
- Recovery: ordinary exact-package recovery completed, exact settings/theme
  were confirmed with `mineonboot=false`, no projection was published, and no
  process remained.
- Outcome: attempt-004 is terminal. Attempt-005 requires
  `docs/parity/work-plans/20260822T024037Z-SELF-001-RETRY-4/PLAN.md`, the
  existing typed register-read/pure hashrate path, removal of synthetic domain
  attribution, all gates, a new exact package, and a fresh detector.

## 2026-08-22T03:31:10Z | attempt-005 verified

- Source commit: `a11b579b62cb52a53bbf6072bde209d3eb3f17e2`
- Actions: completed the full controlled-failure/cancel and passing/restart
  lifecycle under one lease-bound detector-admitted Ultra 205 campaign.
- Hardware: 30,101-ms workload; 1257.913 GH/s total; four BM1366 domain
  averages 105.993/121.261/84.740/109.313 GH/s with 21/53/20/53 accepted and
  zero rejected samples; 5.276 V, 1186 mV, 12.300 W, 2188 RPM, 61.00 C maximum.
- Restoration: exact settings/theme confirmed with `mineonboot=false`; no
  production mining or pool traffic; cleanup complete.
- Evidence: projection
  `1cbc357cb76c51bb354162e48f485442a3ce5eaeb03aead3b71ed50fd3235090`;
  independent Rust validation and semantic redaction pass.
- Outcome: complete and eligible for isolated `SELF-001` promotion to
  `verified` with `unit,workflow,hardware-regression`, progress sync, task
  archive, final verification, commit, and push.

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

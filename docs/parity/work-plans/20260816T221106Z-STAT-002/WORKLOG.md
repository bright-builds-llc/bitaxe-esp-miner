# Parity work log

## 2026-08-16T22:18:00Z | whole-operation timeout correction

- Source commit: `8bd453023b223f013f1c5808550abf5d80604552`.
- Actions: Removed the partial 420-second per-child override, rebound the
  workflow to attempt-003, added source admission for the exact 900-second CLI
  process owner and override-free child call, and expanded the real-process
  fixture to modeled pre-monitor, capture, and post-monitor phases.
- Verification: The immutable plan retains SHA-256
  `7eeeededa3d7a6f671fe00eb6e2b0cbd2fd86d5516fc865d041191182029c631`.
  The automation and contract suites pass. The real child exceeds the old
  scaled 420 boundary, writes complete monitor/effect evidence, and returns
  under the adapter default with no local lifetime argument. Typed supervisor
  timeout classification remains unchanged.
- Evidence: Local source and test results only; no detector, credentials,
  protected attempt, device, origin, or public projection accessed.
- Outcome: The whole-operation ownership correction is implemented with less
  code and stronger process-boundary coverage. The ordered Rust gates,
  Bright Builds checks, package build, full Bazel suite, parity report and
  progress report, redaction verifier, reference verifier, immutable-plan hash,
  and diff checks pass.
- Blocker or next safe action: Commit and push the exact source, rebuild the
  exact clean package, then run the one conditional detector/attempt-003
  sequence.

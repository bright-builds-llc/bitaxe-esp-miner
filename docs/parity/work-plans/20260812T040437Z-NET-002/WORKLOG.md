# Parity work log

## 2026-08-12T04:59:54Z | implementation software gate

- Source commit: pending exact implementation commit.
- Actions: Added the Rust-owned closed provisioning-network evidence contract,
  independent validator, generated TypeScript surface, typed CLI/Justfile
  workflow, strict macOS provisioning client, exact-package recovery, and
  focused production-shaped tests. The simplification pass kept network and
  process effects in two focused imperative-shell modules while the public
  projection remains aggregate-only.
- Verification: Focused contract and 135-test automation targets pass,
  including five isolated reruns of the pre-existing Phase 35 loopback timing
  test. The ordered Cargo format, strict Clippy, all-target build, all-feature
  tests, Bright Builds, all 37 Bazel tests, parity, progress, redaction, and
  pinned-reference gates pass. A cold Bazel run first exposed the known Phase
  35 timing flake and local firmware-build contention; the isolated timing
  reruns, direct rollback firmware build, and subsequent complete `just test`
  run all passed.
- Evidence: Software fixtures and real local child processes only. No detector,
  credentials, NVS, USB, Wi-Fi association, DNS/HTTP probe, or device effect
  occurred. The immutable plan SHA-256 remains
  `c1d92c65a0c121d4f35ddec5cc810a840caa23ee82d2af5d658ff7f78b195c77`.
- Outcome: The exact implementation is ready to commit and push before the
  single detector and conditional attempt-001.
- Blocker or next safe action: Complete selector, generated-contract, task,
  privacy, private-mode, fresh-path, reference, and diff checks; commit and
  push; build the exact clean package; then spend the bounded hardware attempt.

## 2026-08-12T05:11:54Z | attempt-001 terminal closure

- Source commit: `4c42f182b265ec0d4d2b40a04106ffcc4de745ce`.
- Actions: Built the exact clean package, spent the sole detector, and ran the
  sole conditional attempt-001. Accepted its typed failure without retry and
  completed the bounded exact-package owner-Wi-Fi recovery.
- Verification: The closed category is `evidence_invalid`; host restoration
  and device recovery are true; recovery flash was used; secondary recovery
  failure is false. Private aggregate diagnosis records recurring runtime
  heartbeats, health, and operator snapshots but no one-shot boot, safe-state,
  or provisioning startup marker. Both roots are mode 0700, all files are mode
  0600, and the public projection is absent.
- Evidence: Ignored private attempt artifacts and the closed typed envelope
  only. No raw serial, SSID, interface, port, USB identity, address, route,
  origin, credentials, DNS bytes, HTTP content, or process detail is promoted.
- Outcome: No promotion. `NET-002` remains `implemented` because the host
  rejected the late serial capture before the live client transaction.
- Blocker or next safe action: Close this exhausted plan, retain the task as
  active non-verified work, and create a fresh immutable continuation that
  replaces the one-shot serial prerequisite with recurring-runtime plus
  same-origin API admission.

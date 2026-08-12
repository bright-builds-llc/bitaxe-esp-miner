# Parity work log

## 2026-08-12T08:02:58Z | selection and immutable plan

- Source commit: `e97c0042358cee66f67d603fef321d1c5ca5b280`.
- Actions: Selected the first canonical candidate and bounded one exact-package
  scan plus private before/after station-address observation.
- Verification: Branch/upstream, reference, task, fresh paths, full plan-only
  software gate, and immutable plan digest passed.
- Evidence: Software and repository state only. No detector, USB, credential,
  serial, HTTP, radio, or device effect occurred.
- Outcome: Plan committed and pushed as `f26a7f6b`.
- Blocker or next safe action: Implement the typed capture without editing
  `PLAN.md`, run all gates, then commit and push before hardware admission.

## 2026-08-12T08:16:01Z | implementation gate complete

- Source commit: plan commit `f26a7f6bfca35ac2be437638ef141d777338ab9b`.
- Actions: Added the Rust-owned `bitaxe-network-scan-evidence-v1` contract and
  validator, typed automation command, exact-package capture, one-scan
  before/after admission, closed address classification, recovery precedence,
  private-mode enforcement, semantic redaction, and `just`/Bazel wiring.
- Verification: Link-local, unique-local, global, empty-scan, absent-address,
  recovery-precedence, real-child, invocation, typed-failure, and redaction
  regressions pass. Ordered Cargo, Bright Builds, all 37 Bazel tests, the real
  ESP32-S3 firmware image, parity/progress, redaction, reference, generated
  contracts, selector, task, immutable-plan, fresh-path, reference-cleanliness,
  and diff checks pass. The verbose parity report twice encountered transient
  host `os error 35`; the unchanged report redirected to protected scratch
  output completed with `validation_errors: none`.
- Evidence: Synthetic fixtures only; raw radio and station values remain
  private in tests and no hardware effect occurred. Plan SHA-256 remains
  `071a2b0a2d0a6b2ab84fcc854d8cefe765a194c47b5bf588b10014c9810bada2`.
- Outcome: Implementation is eligible for commit and push.
- Blocker or next safe action: Commit and push, rebuild the exact clean
  package, then spend the single detector and conditional attempt-001.

## 2026-08-12T08:25:04Z | hardware attempt-001 passed

- Source commit: `619752535d90a4aa8570b7c96f8339712a329ba8`.
- Actions: Built the exact package, spent one protected detector, and consumed
  the sole attempt-001 with one private pre-scan read, one Wi-Fi scan, and one
  private post-scan read.
- Verification: Exact package identity, 20 bounded exact-shape records,
  numeric auth/signal values, one 3,152-ms scan, same boot, connected client-
  only state, monotonic uptime, stable unique-local v6 classification,
  disabled mining/control, cleanup, no recovery, private modes, no lingering
  flash process, redaction, and independent Rust validation pass.
- Evidence:
  `docs/parity/evidence/net003-scan/network-scan-projection.json`, SHA-256
  `37107b8ad290696a09246c024f373a8d49f3ccb27d15be3cf3f4ff82215fafa1`.
  Raw identifiers and observations remain only in ignored protected artifacts.
- Outcome: Every immutable-plan promotion criterion passed; `NET-003` is
  eligible for a verified transition.
- Blocker or next safe action: Commit the result and evidence without changing
  the checklist, then transition only `NET-003`, sync progress, archive both
  completed NET-003 tasks, and run the final gates.

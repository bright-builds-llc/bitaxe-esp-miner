# Parity work log

## 2026-08-12T03:40:58Z | immutable-plan checkpoint

- Source commit: `a972f191117f1a3a392f5785805fb590476c6f33`
- Actions: Selected the first canonical candidate, inspected the attempt-001
  closure and corrected timing regression, and created the bounded attempt-002
  task and plan with fresh paths.
- Verification: Branch and upstream match; the pinned reference is clean; the
  owner Wi-Fi input exists without inspection; wrapper-002, attempt-002, and
  the public projection path are absent.
- Evidence: Attempt-001 remains a non-verifying closure. No prior private
  artifact or hardware observation is reused as attempt-002 evidence.
- Outcome: Plan drafted; no package, detector, credential, NVS, USB, network,
  or device effect occurred.
- Blocker or next safe action: Run the complete plan-only gate, commit and push
  the immutable plan, then build its exact package before hardware admission.

## 2026-08-12T03:45:55Z | plan-only gate complete

- Source commit: `a972f191117f1a3a392f5785805fb590476c6f33`
- Actions: Diagnosed the JavaScript-runtime-dependent real-child fixture,
  assigned its POSIX-child correction to post-plan execution, added explicit
  continuation lineage, and finalized the immutable attempt-002 plan.
- Verification: The canonical focused regression passed and the direct-Bun
  run reproduced the launcher gap. The ordered Cargo, Bright Builds, 37-target
  Bazel, parity, progress, redaction, reference, generated-contract, selector,
  task-uniqueness, reference-cleanliness, fresh-path, and diff gates passed.
- Evidence: Plan SHA-256 is
  `42a402866befc801bc635aeb367a381d4473aec72846798bafd8176fb83a95f9`;
  the selector returns only this `NET-001` continuation.
- Outcome: Software and immutable-plan admission complete; no package,
  detector, credential, NVS, USB, network, or device effect occurred.
- Blocker or next safe action: Commit and push the plan checkpoint, then build
  its exact clean package before running wrapper-002 detector admission.

## 2026-08-12T03:49:23Z | execution gate complete

- Source commit: `8f42ab0fde9accb7918153434bc459aa544e9130`
- Actions: Replaced the launcher-inherited JavaScript child with a minimal
  executable POSIX child that emits production-shaped monitor stdout and never
  creates the nonexistent monitor artifact.
- Verification: The three focused tests passed uncached under Bun and the
  canonical automation target passed. The ordered Cargo, Bright Builds,
  37-target Bazel, parity, progress, redaction, reference, generated-contract,
  selector, immutable-plan, and diff gates passed.
- Evidence: The real child reaches the ready projection from stdout while an
  explicit read confirms `flash-monitor.log` remains absent.
- Outcome: The focused execution change is ready to become the exact package
  source; no package, detector, credential, NVS, USB, network, or device effect
  occurred.
- Blocker or next safe action: Commit and push this execution checkpoint, then
  build its exact clean package before wrapper-002 detector admission.

## 2026-08-12T03:55:25Z | attempt-002 verified

- Source commit: `e56afbe44ab465eb3fc6b55acce761eb1d29a939`
- Actions: Built the exact package, spent the sole wrapper-002 detector, ran
  the sole conditional attempt-002, and independently validated its public
  projection.
- Verification: Detector admission, exact package provenance, one disconnect,
  fallback, first retry after 5,022 ms, DHCP recovery, retry reset, client-only
  return, 15,000-ms stability, final HTTP/build quorum, disabled effects,
  cleanup, private modes, redaction, and the Rust projection validator passed.
- Evidence: `docs/parity/evidence/net001-reconnect/network-reconnect-projection.json`.
  Wrapper/attempt roots are mode 0700, files are mode 0600, no holder remains,
  and no recovery flash was used.
- Outcome: Accepted `NET-001` hardware evidence; promotion to `verified` is
  eligible.
- Blocker or next safe action: Transition only `NET-001`, synchronize progress,
  run final gates, archive the completed attempt-002 task, and push.

## 2026-08-12T04:01:09Z | transition and final gates

- Actions: Transitioned only `NET-001` to `verified`, synchronized progress,
  archived both completed NET-001 task records, and ran the final verification
  sequence.
- Correction: The first parity gate caught plain paths in the generated
  Rust-owned-target cell. Because the transition tool correctly rejects a
  second `verified` transition, repaired that exact checklist cell and its
  transition receipt, then re-synchronized progress without changing the
  accepted claim or evidence.
- Verification: `cargo fmt --all`, strict Clippy, all-target build, all-feature
  tests, Bright Builds checks, `just test`, `just parity`, progress validation,
  redaction validation, reference cleanliness, automation contract validation,
  task/archive uniqueness, and diff checks passed. The next selector returns
  `NET-002` with no open plan.
- Outcome: `NET-001` is verified with closed exact-package hardware evidence.
- Blocker or next safe action: Commit and push the verified NET-001 result, then
  advance to `NET-002` under a fresh immutable plan.

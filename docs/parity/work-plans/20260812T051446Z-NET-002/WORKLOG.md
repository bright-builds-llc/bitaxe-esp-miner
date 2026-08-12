# Parity work log

## 2026-08-12T05:17:38Z | plan-only gate complete

- Source commit: `7c8c1c01388aaf441c080634f0c25b4c43c40518`.
- Actions: Selected the first canonical candidate, linked the fresh
  continuation to the attempt-001 closure, and narrowed the correction to the
  duplicated one-shot AP startup-line prerequisite.
- Verification: Ordered Cargo format, strict Clippy, all-target build,
  all-feature tests, Bright Builds, all 37 Bazel tests, parity, progress,
  redaction, pinned reference, generated-contract equality, continuation-aware
  selector, task uniqueness, reference cleanliness, fresh-path absence, and
  diff checks pass.
- Evidence: Plan SHA-256 is
  `657f37b864e8dee5accb4d0bae683f39820a69483d49563dd93f2c951bccd44c`.
  No detector, credential, NVS, USB, host-network, DNS/HTTP, or device effect
  occurred during this continuation's planning gate.
- Outcome: The immutable attempt-002 plan is eligible for commit and push.
- Blocker or next safe action: Commit and push the plan/task checkpoint, then
  implement the late-attachment regression without editing `PLAN.md`.

## 2026-08-12T05:19:30Z | late-attachment implementation checkpoint

- Source commit: pending exact implementation commit.
- Actions: Removed only the redundant one-shot AP startup-line predicate. The
  existing exact-package plus trusted passive-safe runtime predicate remains
  mandatory before client observation, and the unique SSID plus API/DNS/HTTP
  quorum remains authoritative for live AP behavior.
- Verification: The 136-test automation target passes. A production-shaped
  success case has exact identities and recurring trusted runtime records but
  no one-shot boot/AP line. A paired failure case omits trusted passive safety,
  proves zero client observations, triggers exact recovery, and withholds the
  public projection.
- Evidence: Software fixtures and a real child process only. No detector,
  credential, NVS, USB, host-network, DNS/HTTP, or device effect occurred.
- Outcome: The correction is the smallest root-cause change and is ready for
  the complete mandatory software gate.
- Blocker or next safe action: Run every ordered gate, review the exact diff,
  then commit and push before any detector or hardware use.

## 2026-08-12T05:57:00Z | complete implementation gate passed

- Source commit: pending exact implementation commit.
- Actions: Completed the explicit simplification and diff review; the runtime
  shell changed by one predicate and the tests carry all new scenario detail.
- Verification: Ordered Cargo format, strict Clippy, all-target build, and
  all-feature tests pass. The first full Rust run exposed an unrelated
  WebSocket loopback `WouldBlock` race; five isolated reruns passed and the
  restarted full Rust gate passed. Bright Builds reports zero findings. All 37
  Bazel tests pass after the commit-stamped normal and rollback variants were
  serialized through their cold local cache rebuild. Parity, progress,
  redaction, reference, generated-contract, selector, immutable-plan, task,
  fresh-path, and diff checks pass.
- Evidence: Software checks only. No attempt-002 detector, credential, NVS,
  USB, host-network, DNS/HTTP, or device effect occurred.
- Outcome: The exact implementation is eligible for commit and push.
- Blocker or next safe action: Commit and push, build the exact clean package,
  then spend the sole wrapper-002 detector and at most one attempt-002.

## 2026-08-12T06:10:59Z | attempt-002 terminal closure

- Source commit: `6a1c2a426a077417b22f7d30728bf335e56f8fce`.
- Actions: Built the exact clean package, spent the sole detector, and ran the
  sole conditional attempt-002. Accepted the typed client-observation failure
  without retry and completed exact-package owner-Wi-Fi recovery.
- Verification: Category `hardware_blocked`; generic stage
  `provisioning_network_capture`; host restoration true; device recovery true;
  recovery flash used; secondary recovery failure false. The attempt wrote no
  system-info artifact, both roots are mode 0700, all files are mode 0600, no
  serial holder remains, and the public projection is absent.
- Evidence: Closed aggregate facts and ignored private artifacts only. No raw
  operational or identifying value is promoted.
- Outcome: No promotion. The corrected runtime admission passed, but the
  current failure envelope cannot identify which client boundary failed.
- Blocker or next safe action: Close this exhausted plan and create a fresh
  continuation with a closed redaction-safe boundary taxonomy before another
  detector or hardware ordinal.

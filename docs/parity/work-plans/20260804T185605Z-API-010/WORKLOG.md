# Parity work log

## 2026-08-04T18:56:05Z | selection and plan

- Source commit: `fbee3ff32f903466ae4a476061b1bc543c6b1368`.
- Actions: Resumed the deterministic parity queue, audited earlier candidates
  against their exact residual gaps, and selected the first bounded safe row,
  `API-010`.
- Evidence: The existing implementation already owns typed theme planning,
  confirmed NVS persistence, firmware GET/POST routes, golden responses, and
  API comparison. The remaining row-owned gap is live route/reboot durability
  and restoration; browser workflow claims remain with `UI-004`.
- Outcome: Immutable plan and one-attempt hardware contract ready.
- Blocker or next safe action: Run planning gates, commit and push the task and
  plan, then implement without modifying `PLAN.md`.

## 2026-08-04 | typed live theme durability implementation

- Source commit: implementation based on planning commit `a1738e88`.
- Actions: Added `verify-theme-durability`, a typed v1 public evidence
  contract, exact-package flash-monitor admission, live theme GET/POST/readback,
  canonical theme comparison, canonical device-session reboot, post-restart
  persistence, exact restoration, and bounded exact-package recovery.
- Verification: Focused TypeScript compilation plus automation and generated-
  contract tests pass. Behavior coverage includes ready success, every closed
  non-ready device-session category, malformed/missing projection, timeout,
  launch failure, restoration/recovery precedence, sensitive-value denial,
  file modes, and a real subprocess boundary.
- Evidence: Public output is closed to identities, the closed reboot
  projection, and safe booleans. Theme values, origins, hostnames, ports,
  network identifiers, credentials, and raw traces remain private.
- Outcome: Implementation and focused regressions are ready for the mandatory
  repository gate.
- Blocker or next safe action: Run the complete ordered gate, review the diff,
  then commit and push the exact implementation before detector use.

## 2026-08-04 | mandatory implementation gate

- Source commit: implementation based on planning commit `a1738e88`.
- Actions: Ran the complete ordered Rust, managed standards, Bazel, parity,
  privacy, reference, immutable-plan, sensitive-output, and diff checks; then
  reviewed the functional core, process shell, recovery precedence, and shared
  projection-parser diff for unintended scope.
- Verification: `cargo fmt --all`, strict Clippy, all-target/all-feature Cargo
  build and tests, Bright Builds, all 34 Bazel test targets including the real
  ESP-IDF firmware build, parity/progress, redaction, reference cleanliness,
  immutable-plan comparison, and diff checks pass. The first parity launch hit
  transient host `os error 35` after the test suite; one bounded rerun passed
  with `validation_errors: none`.
- Evidence: Focused tests continue to cover every typed terminal boundary and
  public-output denial. The immutable `PLAN.md` is byte-identical to commit
  `a1738e88`.
- Outcome: Software implementation is clean and ready to commit and push.
- Blocker or next safe action: Commit and push the exact implementation, then
  run only the task-recorded package, detector, and `attempt-001` commands.

## 2026-08-04 | attempt-001 pre-effect privacy admission failure

- Source commit: `f30df231`.
- Actions: Built the exact package, passed the detector against one admitted
  Ultra 205, and invoked the sole task-recorded `attempt-001` command.
- Verification: The verifier rejected the detector transcript before creating
  the private attempt root or admitting the package because the transcript was
  mode `0644`, not the required private mode `0600`. The attempt root and
  public projection are absent.
- Evidence: Earliest category `process_failed`; zero flash, HTTP, restart,
  device mutation, or recovery effects occurred.
- Outcome: `attempt-001` failed closed at a host privacy boundary. No parity
  evidence was emitted and `API-010` remains `implemented`.
- Blocker or next safe action: Use the new targeted retry task to pre-create a
  fresh detector transcript as mode `0600`, commit and push that contract, and
  run its sole bounded `attempt-002` without changing the implementation.

## 2026-08-04 | attempt-002 closed baseline admission failure

- Source commit: `8e95e5a63cb4aef969f13cf1054b0337228e6845`.
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`.
- Actions: Rebuilt the exact package, created the detector transcript as mode
  `0600`, admitted exactly one Ultra 205, and ran the sole authorized
  `attempt-002`.
- Verification: The transaction returned earliest category `evidence_invalid`
  with the public reason `baseline classification is invalid`. A read-only
  classifier recheck against the private trace reproduced the closed category
  `baseline_multiple_sessions`. The private attempt root/directories are mode
  `0700`, every private file is mode `0600`, and the public projection is
  absent.
- Evidence: Initial exact-package flash-monitor completed, but baseline
  admission stopped before hostname/theme GET or POST, software restart, or
  settings mutation. The authorized exact-package flash was the sole device
  effect; no restoration or recovery action was required.
- Outcome: No evidence was published and `API-010` remains `implemented`.
  The single hardware retry is consumed; another retry is not authorized.
- Blocker or next safe action: Investigate the production multi-session
  baseline trace at the host-orchestration seam under the new software-only
  follow-up task, create a new immutable plan, and do not touch hardware.

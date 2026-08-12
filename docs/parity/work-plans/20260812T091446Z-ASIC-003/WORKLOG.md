# Parity work log

## 2026-08-12T09:14:46Z | selection and draft plan

- Source commit: `29fba85041c0e4338a512acb3d251ae9363e066f`.
- Actions: Selected the first canonical candidate and scoped a typed,
  no-hardware derivation from the committed ASIC initialization projection.
- Verification: Branch/upstream, clean worktree, reference commit, selector,
  source-projection digest, accepted campaign lineage, and exact work-module
  compatibility were inspected.
- Evidence: Existing committed redacted evidence only; no protected artifact,
  detector, USB, credential, serial, network, mining, or device effect
  occurred.
- Outcome: Draft immutable plan and matching active task created.
- Blocker or next safe action: Run the complete plan-only gate, commit and
  push the plan, then implement without editing `PLAN.md`.

## 2026-08-12T09:17:40Z | plan-only gate passed

- Source commit: `29fba85041c0e4338a512acb3d251ae9363e066f`.
- Actions: Ran the complete plan-only repository gate and confirmed task,
  selector, reference, source-evidence, compatibility, and immutable-plan
  invariants.
- Verification: Ordered Cargo checks, Bright Builds, all 37 Bazel tests, the
  real ESP32-S3 image, generated contracts, parity/progress, redaction,
  reference, task uniqueness, selector, and diff checks passed.
- Evidence: Software/repository validation only. Plan SHA-256 is
  `dd3a1d4ce5324100dc41d58a1fa16574946490d8476796acd325a39c80489eb2`.
- Outcome: The plan and task contract are eligible to commit and push.
- Blocker or next safe action: Commit and push, then implement the typed
  derived projection without changing `PLAN.md` or interacting with hardware.

## 2026-08-12T09:28:11Z | implementation gate passed

- Source commit: plan commit
  `fdd464daef59f5fcb64c364ed33703c8f85d4ecb`.
- Actions: Added the Rust-owned ASIC work-send contract/validator, typed
  derived projector, source-projection validation, exact work-module and
  bounded source-span compatibility checks, semantic-redaction registration,
  and CLI/Just/Bazel/generated-contract wiring.
- Verification: Contract, incomplete source, span drift, dirty-path,
  sensitive-output, typed-failure, invocation, and real-child validator
  regressions pass. A real repository-source dry run also passed. Ordered
  Cargo, Bright Builds, all 37 Bazel tests, the real ESP32-S3 image, generated
  contracts, parity/progress, redaction, reference, immutable-plan, and diff
  gates pass.
- Evidence: Synthetic fixtures plus one ignored scratch dry-run projection;
  no committed ASIC-003 evidence and no hardware effect occurred.
- Outcome: The implementation is ready to commit and push.
- Blocker or next safe action: Commit and push the projector, remove the
  ignored dry-run artifact, then produce the public projection from the exact
  clean implementation commit.

## 2026-08-12T09:34:17Z | derived projection accepted

- Source commit: implementation commit
  `32017ba8bb9b99212cad3c2c9ecaed7edf603d19`.
- Actions: Ran the no-hardware projector against the committed ASIC-002 source
  projection and exact accepted hardware source commit.
- Verification: Source validation/digest binding, commit ancestry, three exact
  core paths, three unique bounded dispatch/write spans, clean relevant
  worktree, independent Rust validation, and public redaction all passed.
  Projection SHA-256 is
  `447af65ae9e6cd5cc2199ef639ff8e0fa7f63d4c9708570bd66781c5a162e80c`.
- Evidence: Published
  `docs/parity/evidence/asic003-work-send/asic-work-send-projection.json`;
  no protected or operational values were accessed or published.
- Outcome: The complete `ASIC-003` promotion quorum is satisfied without a
  new hardware attempt.
- Blocker or next safe action: Commit and push the evidence/result without
  changing the checklist, then apply the single-row transition and run the
  final promotion gate.

## 2026-08-12T09:35:40Z | single-row promotion applied

- Source commit: evidence commit
  `12e6941cc7b61cbb5a0d3571587fa242cadfce57`.
- Actions: Applied transition `20260812T093510Z-ASIC-003`, synchronized
  progress, completed the matching task, and moved its native record to the
  append-only archive.
- Verification: Only `ASIC-003` changed from `implemented` to `verified` with
  `unit,golden,workflow,hardware-smoke,hardware-regression`; the transition
  binds the immutable plan and committed result. Progress is 53 of 94 active
  rows, 56.4%.
- Evidence: Checklist transition receipt, progress history, README projection,
  archived task, result, and public work-send projection.
- Outcome: `ASIC-003` is conservatively promoted and its active task is
  complete.
- Blocker or next safe action: Run the complete final repository gate, verify
  transition/result digests and task uniqueness, commit and push, then start a
  fresh selector invocation.

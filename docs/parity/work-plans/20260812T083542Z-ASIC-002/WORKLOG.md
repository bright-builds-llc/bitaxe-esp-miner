# Parity work log

## 2026-08-12T08:35:42Z | selection and draft plan

- Source commit: `90fdfea035302e55707d5cd5e689f0e75ad1b6b2`.
- Actions: Selected the first canonical candidate and scoped a typed,
  no-hardware projection over the sealed accepted-share initialization proof.
- Verification: Branch/upstream, clean worktree, reference commit, selector,
  archived task lineage, protected artifact presence, accepted result fields,
  preparation counts, and initialization-path history were inspected.
- Evidence: Existing protected artifacts only; no raw values were copied and
  no detector, USB, credential, serial, HTTP, network, or device effect
  occurred.
- Outcome: Draft immutable plan and matching active task created.
- Blocker or next safe action: Run the complete plan-only gate, commit and
  push the plan, then implement without editing `PLAN.md`.

## 2026-08-12T08:39:02Z | plan-only gate passed

- Source commit: `90fdfea035302e55707d5cd5e689f0e75ad1b6b2`.
- Actions: Ran the complete plan-only repository gate and confirmed task,
  selector, reference, path, and immutable-plan invariants.
- Verification: Ordered Cargo checks, Bright Builds, all 37 Bazel tests, real
  ESP32-S3 image builds, parity/progress, redaction, reference, generated
  contracts, task uniqueness, and diff checks passed.
- Evidence: Software/repository validation only. Plan SHA-256 is
  `8759c7255261117d8000f513f9fd1a1f4b376eea2df1d30af4ec8bb3168194e9`.
- Outcome: The plan and task contract are eligible to commit and push.
- Blocker or next safe action: Commit and push, then implement the typed
  projection without changing `PLAN.md` or interacting with hardware.

## 2026-08-12T08:52:13Z | implementation gate passed

- Source commit: plan commit
  `70becfaab3c3fa5f56f672d3db7e0e6244c8f1bf`.
- Actions: Added the Rust-owned ASIC-initialization contract/validator, typed
  projection command, seal/digest/mode admission, archived-task binding,
  initialization-source compatibility check, semantic-redaction registration,
  and `just`/Bazel/generated-contract wiring.
- Verification: Contract, malformed/incomplete evidence, source drift,
  sensitive-output, typed-failure, invocation, real-child/file, campaign,
  mining-actuation, and flash regressions pass. Ordered Cargo, Bright Builds,
  all 37 Bazel tests, real ESP32-S3 image, parity/progress, redaction,
  reference, generated contracts, immutable-plan, reference-cleanliness, and
  diff gates also pass.
- Evidence: Synthetic fixtures only in this checkpoint. The protected source
  attempt was not projected yet and no hardware effect occurred.
- Outcome: The implementation is ready to commit and push.
- Blocker or next safe action: Commit and push the projector, then run it once
  against the sealed attempt from the exact clean implementation commit.

## 2026-08-12T08:57:02Z | production lineage parsing fixed

- Source commit: implementation commit
  `26aabe65aad34928625ddb050aa806a0edee0142`.
- Actions: The first clean projection invocation failed closed because the
  archived task wraps `Ultra 205` across Markdown lines. Normalized whitespace
  within the exact selected task block and made the fixture reproduce that
  production shape.
- Verification: The failed invocation created no projection. The focused
  real-child suite, complete ordered Cargo gate, Bright Builds, all 37 Bazel
  tests, parity/progress, immutable-plan, and diff checks pass after the fix.
- Evidence: No public evidence was published and no hardware effect occurred.
- Outcome: The source-lineage seam now accepts Markdown wrapping without
  weakening the exact required phrases or task boundary.
- Blocker or next safe action: Commit and push this regression fix, then rerun
  the no-hardware projection from the new exact clean commit.

## 2026-08-12T09:02:20Z | public redaction false positive fixed

- Source commit: lineage-fix commit
  `0bac6e95f620772129f3c8793c5cfe24076c6063`.
- Actions: The second clean projection invocation completed, but the
  repository redaction verifier rejected the closed boolean key
  `exactly_one_chip_detected` because its generic substring rule interpreted
  the final two letters as an IP-address field. Added that exact key to the
  semantic allowlist while preserving rejection of a raw `ip` field.
- Verification: The rejected projection was removed before the fix and the
  regression test proves both sides of the boundary.
- Evidence: No public evidence remains from the rejected invocation and no
  detector, USB, credential, serial, HTTP, network, mining, or device effect
  occurred.
- Outcome: The public schema can express the closed single-chip boolean
  without weakening the broader sensitive-key denylist.
- Blocker or next safe action: Complete all repository gates, commit and push
  the fix, then regenerate the projection from that exact clean commit.

## 2026-08-12T09:06:49Z | sealed projection accepted

- Source commit: redaction-fix commit
  `f9df1412abbc05a4852022f3fb6741f67ab43272`.
- Actions: Ran the no-hardware projector for the third and final host
  projection attempt against the unchanged sealed `attempt-007` inputs.
- Verification: The result seal, private digests, protected modes, archived
  task lineage, exact accepted terminal state, 18 accepted and zero invalid
  preparation events, terminal production-UART retention, seven compatible
  initialization paths, independent Rust validation, and public redaction all
  passed. Projection SHA-256 is
  `eee750561a7c1dcec1a5698b1e5827d3f1508d43655c3c4aa237097338dcf8d4`.
- Evidence: Published
  `docs/parity/evidence/asic002-initialization/asic-initialization-projection.json`;
  protected values remain absent and the source artifacts remain unchanged.
- Outcome: The complete `ASIC-002` promotion quorum is satisfied without a
  new hardware attempt.
- Blocker or next safe action: Commit and push the evidence/result without
  changing the checklist, then apply the single-row transition and run the
  final promotion gate.

## 2026-08-12T09:09:28Z | single-row promotion applied

- Source commit: evidence commit
  `5694c245622ceb15dd7f3924cac7327f5d99bf1c`.
- Actions: Applied transition `20260812T090906Z-ASIC-002`, synchronized
  progress, completed the matching task, and moved its native record to the
  append-only archive.
- Verification: Only `ASIC-002` changed from `implemented` to `verified` with
  `unit,workflow,hardware-smoke,hardware-regression`; the transition binds the
  immutable plan and committed result. Progress is 52 of 94 active rows,
  55.3%.
- Evidence: Checklist transition receipt, progress history, README projection,
  archived task, result, and public initialization projection.
- Outcome: `ASIC-002` is conservatively promoted and its active task is
  complete.
- Blocker or next safe action: Run the complete final repository gate, verify
  transition/result digests and task uniqueness, commit and push, then start a
  fresh selector invocation.

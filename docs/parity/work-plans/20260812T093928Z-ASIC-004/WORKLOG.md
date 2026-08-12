# Parity work log

## 2026-08-12T09:39:28Z | selection and draft plan

- Source commit: `12f30445a0fdd70b92a0e3b067c06c7af34fe2fa`.
- Actions: Selected the first canonical candidate and scoped a typed,
  no-hardware derivation from the committed ASIC work-send projection.
- Verification: Branch/upstream, clean worktree, reference commit, selector,
  source-projection lineage, accepted hardware commit, result-parser history,
  and current closed-discard coverage were inspected.
- Evidence: Existing committed redacted evidence only; no protected artifact,
  detector, USB, credential, serial, network, mining, or device effect
  occurred.
- Outcome: Draft immutable plan and matching active task created.
- Blocker or next safe action: Run the complete plan-only gate, commit and
  push the plan, then implement without editing `PLAN.md`.

## 2026-08-12T09:42:32Z | immutable plan gate passed

- Source commit: `12f30445a0fdd70b92a0e3b067c06c7af34fe2fa`.
- Immutable plan SHA-256:
  `99252a9cc05fccfc9fd85a708cd375c441775163c6d5d7b864862f6d0f3f5056`.
- Actions: Completed the ordered Cargo, managed-rules, repository test,
  firmware package, automation-contract, parity, redaction, reference, and
  canonical-selector checks.
- Verification: All checks passed; 37 Bazel tests passed, the real Ultra 205
  package built, redaction checked 12 public roots, the reference remained
  clean at `c1915b0a63bfabebdb95a515cedfee05146c1d50`, and the selector returned
  only the open `ASIC-004` plan.
- Evidence: Plan-gate logs remain ignored under `scratch/asic004-gates/`; no
  hardware, protected artifact, credential, serial, network, mining, or device
  effect occurred.
- Outcome: The plan is ready for its required plan-only commit and push.
- Blocker or next safe action: Commit and push the immutable plan and active
  task, then implement the typed evidence derivation without changing
  `PLAN.md`.

## 2026-08-12T10:07:38Z | implementation gate passed

- Source commit: `d80924d1b7a84280ef090cb8bef28d1f53dfa646`.
- Actions: Added the Rust-owned closed result-parsing contract and independent
  validator, synchronized TypeScript contract, host projector, CLI/Just/Bazel
  wiring, typed failure/redaction registration, semantic source-compatibility
  checks, and behavior-focused unit plus real-child tests.
- Verification: Contract tests, 95 BM1366 tests, 21 production-work tests, the
  host automation suite, and a production-shaped dry projection passed. The
  complete ordered Cargo gate, managed-rules check, 37-test Bazel suite, real
  Ultra 205 package, generated-contract check, parity checks, redaction, and
  reference verification all passed.
- Gate correction: The first production dry run exposed a non-common worker
  span terminator and published nothing; the selector now binds the unique
  unchanged nonce-emission fragment. The managed file-length gate then caught
  the synchronized generated contracts above its exact 628-line ceiling; the
  adjacent ASIC declarations were compacted without changing their contract,
  and both generated files now match byte-for-byte at exactly 628 lines.
- Evidence: The ignored dry projection independently validated and binds the
  committed ASIC-003 projection digest
  `447af65ae9e6cd5cc2199ef639ff8e0fa7f63d4c9708570bd66781c5a162e80c`.
  No hardware, protected artifact, credential, serial, network, mining, or
  device effect occurred.
- Outcome: The implementation is clean and ready for its separate commit and
  push. Public checklist evidence remains withheld until generated from that
  clean pushed implementation commit.
- Blocker or next safe action: Commit and push the implementation, then derive
  and independently validate the public projection from the clean commit.

## 2026-08-12T10:14:20Z | semantic redaction collision corrected

- Source commit: implementation commit
  `00dc1bd07e3530ad39b5ccfae9036aebbee32018`.
- Actions: Generated the first public candidate; the independent contract
  validator accepted it, but semantic redaction rejected the closed
  `transcript_path_unchanged` key because the guard's intentionally broad
  prohibited-token matcher finds `ip` inside `transcript`. The invalid
  candidate was removed, and the same fact was renamed to the closed
  `result_transport_module_unchanged` key across Rust and synchronized
  TypeScript contracts plus the projector.
- Verification: Focused contract and host suites, a new ignored dry
  projection, targeted semantic redaction, the complete ordered Cargo gate,
  managed rules, all 37 Bazel tests, real firmware package, generated
  contracts, parity, redaction, and reference verification passed.
- Evidence: No public evidence remains from the rejected candidate. The
  corrected dry projection passed semantic redaction with one checked
  artifact and was removed. No hardware or protected/operational input was
  accessed.
- Outcome: The redaction-safe implementation correction is ready to commit
  and push before regenerating public evidence.
- Blocker or next safe action: Push the correction and produce a fresh public
  projection whose source commit is the new clean pushed commit.

## 2026-08-12T10:14:57Z | derived projection accepted

- Source commit: corrected implementation commit
  `2861bfb1d425d3c5d13b3a820c082eb24e1f1a77`.
- Actions: Ran the no-hardware projector against the committed ASIC-003
  work-send projection and exact accepted hardware source commit.
- Verification: Source validation and digest binding, ancestry, unchanged
  result transport module, exact parser/adapter/worker spans, compatible
  correlation semantics, clean relevant worktree, independent Rust
  validation, 0644 publication mode, public-sensitive-value scan, and semantic
  redaction all passed. Projection SHA-256 is
  `e99c054c4d660155d5c2b1ee38d3f17aed5ae7101e7e4a5fd1c6451d1b48b7c7`.
- Evidence: Published
  `docs/parity/evidence/asic004-result-parsing/asic-result-parsing-projection.json`;
  semantic redaction checked 13 public artifacts. No protected or operational
  values were accessed or published.
- Outcome: The complete `ASIC-004` promotion quorum is satisfied without a
  new hardware attempt.
- Blocker or next safe action: Commit and push the evidence/result without
  changing the checklist, then apply the single-row transition and run the
  final promotion gate.

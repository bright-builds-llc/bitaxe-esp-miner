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

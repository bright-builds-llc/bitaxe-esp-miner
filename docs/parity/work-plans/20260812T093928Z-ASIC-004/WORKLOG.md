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

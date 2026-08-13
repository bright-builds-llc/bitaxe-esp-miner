# Parity work log

## 2026-08-13T03:39:07Z | Selection and immutable-plan checkpoint

- Source commit: `1150fcd113226904df1414269a8e7929bee24ed1`
- Actions: Loaded the advance-parity workflow, active lessons within the
  deterministic budget, repository guidance, applicable Bright Builds
  standards, active tasks, selector output, checklist row, pinned PID source,
  current Rust core, fixtures, tests, and evidence policy. Selected THR-003
  after recording the sealed API-009 and unavailable safe THR-001 boundaries.
- Verification: Clean synchronized `main`; no open plan; reference commit
  `c1915b0a63bfabebdb95a515cedfee05146c1d50`; active lesson hashes match the
  current audit baseline. The source comparison identifies input/output EMA,
  gain-scaling, initialization, limits, and anti-windup gaps without invoking
  hardware.
- Evidence: Pinned `PID.c`, `PID.h`, fan-controller task, current pure Rust
  module/fixture/tests, Phase 11 evidence ledger, checklist validator, and the
  accepted PWR-002 projection metadata.
- Outcome: THR-003 is the first actionable row and has one bounded
  software-only plan.
- Blocker or next safe action: Run the complete plan/task checkpoint gates,
  commit and push the immutable plan before editing implementation files.

## 2026-08-13T03:43:47Z | Plan checkpoint gates

- Actions: Ran the required plan-boundary verification without touching the
  implementation, checklist, progress, evidence, reference tree, or hardware.
- Verification: `cargo fmt --all`, strict all-target/all-feature Clippy,
  all-target/all-feature Cargo build, all-feature Cargo tests, the complete
  Bright Builds check, `just test`, `just parity`, `just parity-progress`, and
  `git diff --check` all passed. Parity validation reported no errors and
  progress remained `verified=65 active=94 total=99 deferred=5`.
- Evidence: The immutable plan SHA-256 is
  `3acea362f65f63ccab564b1d4af98a22f4f026dffecf258a5a5d70ca119e0348`.
- Outcome: The plan and active task are ready for their mandatory commit and
  push checkpoint.
- Blocker or next safe action: Commit and push only the immutable plan,
  worklog, and active task, then begin the pure PID implementation.

## 2026-08-13T03:51:51Z | Exact PID implementation checkpoint

- Source commit before implementation: `73a3054b87df3a7ef0b08f1f9d0ab7e9cac36137`.
- Actions: Replaced the output-EMA approximation with a deep pure PID module
  that preserves automatic initialization, temperature-input EMA, 100 ms
  sample-scaled reverse P-on-error gains, dynamic output limits, both output
  clamp/anti-windup adjustments, retained input/output state, and the pinned
  initial zero-minimum `0..255` internal-limit edge. Kept the existing
  whole-percent actuation adapter and production safety/ownership shell.
- Regression guard: Expanded the provenance-bound fixture to five sequential
  state traces covering 12 scheduled transitions and asserted filtered input,
  raw output, applied duty, output sum, limits, automatic mode, and retained
  input at each boundary. Updated the production planner test to prove its
  retained state produces the pinned 75 then 85 percent sequence.
- Verification: The focused safety-core golden test and all three required
  Bazel targets pass. The mandatory Cargo format/Clippy/build/test sequence,
  Bright Builds checks, `just test`, parity report/progress, normal and
  rollback-probe firmware builds, redaction, and reference cleanliness pass.
  The accepted PWR-002 projection validator passes; its SHA-256 remains
  `0668c274d09b3e39d7d5edfea4b2e66c97248ff77de9192981f3af00e547ddfe`,
  its mode remains `0644`, its result digest remains
  `199509d8f95dab4287f4d3c3a7b09b381823250ff990c7ee7ad1a612ffbf6b9c`,
  and its source commit is an ancestor of this implementation.
- Outcome: Exact pure PID behavior and its unchanged production consumption
  are implemented without hardware or reference-tree changes.
- Blocker or next safe action: Commit and push this implementation checkpoint,
  then compose the row-specific result and run the evidence checkpoint gates.

## 2026-08-13T03:56:40Z | Upstream float-precision correction

- Actions: The simplification/fidelity review found that the pinned C
  controller retains all PID and filter values as `float`, while the initial
  Rust replacement used the prior core's `f64` state. Changed the pure PID
  constants, filter, retained state, and arithmetic to `f32`; kept the public
  thermal observation input as `f64` and converts it only at the pinned PID
  boundary. Recomputed the committed golden values at exact promoted `f32`
  boundaries.
- Verification: The exact golden assertion first exposed the expected C-float
  delta at sequence step 2 (`65.30331420898438` versus the real-arithmetic
  shorthand `65.3032`). After correcting the fixture, the exact sequential
  test, focused Bazel targets, mandatory Cargo sequence, Bright Builds checks,
  all 42 Bazel test targets, parity report, and progress report pass.
- Outcome: The controller now matches both pinned transition ordering and C
  `float` precision, including the whole-percent rounding boundary.
- Blocker or next safe action: Commit and push the precision correction, then
  compose the row result against the final implementation commit.

## 2026-08-13T03:59:42Z | Evidence-source checkpoint

- Implementation commit: `91158f51831e87bcd0fbab2a29b9f2219904b1da`.
- Actions: Composed `RESULT.md` from the exact pure/golden/workflow proof and
  the unchanged accepted PWR-002 actuator-chain boundary. Preserved every live
  automatic-response and fault-stimulus non-claim.
- Verification: Repeated the mandatory Cargo sequence, Bright Builds checks,
  all 42 Bazel test targets, parity report/progress, focused targets, normal
  and rollback-probe firmware builds, PWR-002 validation/digest/mode/lineage,
  redaction, reference cleanliness, immutable-plan, unique-task, and diff
  checks. All passed; parity still reports no validation errors at 65 verified
  rows before transition.
- Evidence: `RESULT.md` SHA-256 is
  `6536cae83b6b5397caa3dc5bb96324719fb582adeeb8c8fca5bb303d36584f5d`.
- Outcome: The complete source for a typed THR-003 transition is closed.
- Blocker or next safe action: Commit and push the result checkpoint, retain
  that exact commit as the evidence source, and mutate only THR-003 through the
  typed transition command.

## 2026-08-13T04:01:43Z | Typed transition and closure

- Evidence source commit: `6f8c043a0d188404f63e73fbf8e3a5427e876f71`.
- Actions: Applied typed transition `20260813T040000Z-THR-003` to THR-003 only,
  synchronized progress from the exact pushed source, confirmed the selector
  closed this plan, and moved the completed native task record to the
  append-only archive.
- Verification: The transition bound the immutable plan and result digests,
  exact reference commit, Rust-owned targets, composed evidence labels, and
  explicit non-claims. Immediate parity validation reports no errors; progress
  reports `verified=66 active=94 total=99 deferred=5 completion=70.2%`.
- Outcome: THR-003 is verified and its active task is complete.
- Blocker or next safe action: Run the final repository gate sequence over the
  transition/archive diff, commit and push it, then stop this one-row
  invocation. API-009 and THR-001 retain their recorded boundaries; IO-001 is
  the next selector candidate requiring a new invocation.

## 2026-08-13T04:04:35Z | Final gate result

- Verification: The final transition/archive tree passed the ordered Cargo
  sequence, Bright Builds checks, all 42 Bazel test targets, parity and progress
  validation, redaction, reference cleanliness, independent PWR-002 validation,
  exact plan/result/evidence digests, evidence mode, task uniqueness, archive
  placement, sensitive-output, and diff checks.
- Outcome: The repository is ready for the final commit and push with THR-003
  verified at 66 of 94 active rows (70.2%).
- Blocker or next safe action: None for this invocation; commit and push the
  closed transition/archive checkpoint.

# Parity work plan

- Run ID: `20260815T185700Z-THR-001`
- Parity row: `THR-001`
- Initial status: `implemented`
- Source commit: `fe953c0f6c8b1cfad64eecedbb410a02f975408b`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-thr001-emc2101-live-thermal`
- Continues plan: `docs/parity/work-plans/20260815T182438Z-THR-001/PLAN.md`

## Selection and exact symptom

The clean synchronized selector still ranks THR-001 first. Consumed attempt-005
proved the corrected device transaction reached `fault_observed` and
`recovered` without an abort, then failed as `evidence_invalid` because the host
marker contract did not match production observation reality. Production
markers carry a canonical ESP logging prefix, while the validator and its
real-child fixture require invented byte-zero payload lines. The earlier
`baseline_ready` marker can also occur before post-flash monitor attachment and
was absent from the capture.

This is a software-only diagnosis and correction plan. It authorizes no
package, detector, USB, serial, network, HTTP, device, NVS, display, sensor,
mining, control, reset, OTA, erase, attempt-006, or parity promotion.

## Feedback loops and constraints

First change the real-child regression to emit canonical prefixed production
log lines and prove the current byte-zero parser rejects them with the exact
`evidence_invalid` marker-sequence category. Then add a deterministic late-
attachment case that omits the initial baseline marker while retaining later
fault and recovery markers. Both loops must run in seconds, cross the actual
child/file boundary, and publish no candidate projection.

After the two red signals exist, rank and test multiple falsifiable designs.
Use one strict shared parser for the canonical ESP log envelope; do not accept
arbitrary prefixes, substring matches, malformed levels/timestamps/tags, or
extra payload text. Make the complete ordered baseline/fault/recovery witness
available to a late observer through a bounded production-owned replay or
reader-before-effect transaction. The solution must preserve one-shot intent
consumption, exact marker order and uniqueness, redaction, ordinary logs,
finite automated deadlines, cleanup, and the existing device stimulus logic.

Do not weaken the evidence quorum, infer the absent baseline, fabricate child
artifacts, delay boot by an arbitrary fixed sleep, expose a public diagnostic
setter, or make human timing part of the proof. Prefer an existing retained-log
or accepted-state replay primitive when it can carry this closed marker state
without broadening its privacy or lifecycle contract.

## Implementation and verification

- [ ] Add and run the canonical-prefix and late-attachment red-capable real-
      process regressions.
- [ ] Record three to five ranked hypotheses and select the smallest strict
      parser plus replay/reader design that covers both failures.
- [ ] Implement the shared production-shaped marker boundary and replayable
      ordered witness with focused malformed-prefix, duplicate, missing,
      ordering, timeout, redaction, and non-replay tests.
- [ ] Re-run the original attempt-005 production-shaped software scenario and
      prove final evidence can publish only on the complete closed quorum.
- [ ] Run focused, firmware, ordered Cargo, Bright Builds, full Bazel,
      parity/progress, redaction, reference, task/plan, and diff gates.
- [ ] Commit and push with THR-001 still `implemented`; close without hardware
      evidence. A later immutable plan is required for any attempt-006.

Required final gates are `cargo fmt --all`, strict Clippy, all-target build,
all-feature tests, `bun scripts/bright-builds-check.ts all`, `just build`, `just
test`, `just parity`, `just parity-progress`, `just verify-redaction`, `just
verify-reference`, no temporary debug instrumentation, `git diff --check`, and
full diff review.

## Stop conditions

Stop if the production-shaped loops cannot reproduce both consumed failure
signatures, if no strict unambiguous ESP-log payload boundary exists, if replay
requires exposing private values or weakening retained-log policy, if reader-
before-effect cannot be guaranteed without a new protocol, or if the fix would
change normal boot/sensor safety semantics. Record the earliest typed blocker;
never reinterpret attempt-005 or authorize attempt-006 from this plan.

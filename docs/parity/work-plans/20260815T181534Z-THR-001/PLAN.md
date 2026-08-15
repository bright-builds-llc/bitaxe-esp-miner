# Parity work plan

- Run ID: `20260815T181534Z-THR-001`
- Parity row: `THR-001`
- Initial status: `implemented`
- Source commit: `b0808ddbfa3cd04aae6546093386a251c48de2a9`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-thr001-emc2101-live-thermal`
- Continues plan: `docs/parity/work-plans/20260813T073353Z-THR-001/PLAN.md`

## Selection and observed failure

The clean synchronized selector ranks THR-001 first after API-009 promotion.
The consumed attempt-004 exact-package firmware reached the redaction-safe
`fault_observed` marker, then aborted as `fault_projection_missing`. Ordinary
exact-package restoration passed, and no attempt-005 ran. The prior closure
requires a deterministic reproduction through the real production ordering of
`ThermalFaultStimulus::step`, `reduce_sensor_sweep`, producer stale processing,
and the next owner sweep before any new hardware ordinal is considered.

This is a software-only diagnosis and correction plan. It creates no detector,
package, USB, serial, network, device, NVS, display, mining, sensor, fan,
voltage, frequency, power, OTA, reset, erase, or other hardware effect. It
neither authorizes attempt-005 nor promotes THR-001.

## Feedback loop and scope

Add one fast deterministic regression at the pure production boundary. It must
start from a real successful temperature reduction, drive the stimulus with
successful real-read outcomes, apply every returned outcome through
`reduce_sensor_sweep`, apply the same stale-processing order as the production
owner, and assert the closed five-injection marker sequence and fresh recovery.
Before the fix, this loop must reproduce the exact attempt-004 symptom as
`FaultProjectionMissing`, not a nearby parser, host, marker, or evidence error.

After the red loop exists, rank and test multiple falsifiable causes. Correct
the smallest production semantic boundary that preserves all ordinary sensor
fresh/fault/stale behavior while retaining the injected fault state long enough
for its fixed five-sample transaction. Do not weaken global stale limits,
fabricate sensor values, bypass the real reducer, teach the evidence layer to
accept missing proof, or specialize public observation truth for one attempt.

Convert the minimized reproduction into permanent regression coverage at the
real call-order seam. Preserve successful real reads during every overlay,
exactly five invalid outcomes, the existing fault reason, ordered
`baseline_ready`, `fault_observed`, and `recovered` markers, sequence behavior,
ordinary staleness, bounded failure categories, and no replay. Remove all
temporary instrumentation and perform an explicit simplification pass.

## Implementation and verification

- [ ] Add and run one agent-runnable red-capable production-order regression.
- [ ] Record three to five ranked falsifiable hypotheses and isolate the
      load-bearing transition that changes the fault into missing projection.
- [ ] Apply the smallest root-cause correction and prove the minimized and
      original production-order loops green.
- [ ] Add ordinary fault/stale/non-stimulus non-regressions and remove all
      temporary diagnostics or harnesses.
- [ ] Run focused tests, the real ESP32-S3 firmware build, ordered Cargo gates,
      Bright Builds, the full Bazel suite, parity/progress, redaction,
      reference cleanliness, task/plan binding, and diff review.
- [ ] Commit and push the software correction with THR-001 still
      `implemented`; close this plan without hardware evidence or promotion.

Required commands after the focused loop passes:

1. `cargo fmt --all`
2. `cargo clippy --all-targets --all-features -- -D warnings`
3. `cargo build --all-targets --all-features`
4. `cargo test --all-features`
5. `bun scripts/bright-builds-check.ts all`
6. `just build`
7. `just test`
8. `just parity`
9. `just parity-progress`
10. `just verify-redaction`
11. `just verify-reference`
12. `git diff --check`

## Stop and continuation

Stop if the real production-order loop cannot reproduce the attempt-004
category, if the correction would weaken safety freshness, if a clean firmware
build or mandatory gate fails for a new reason that cannot be corrected in
scope, or if another component owns the lost transition. Record the earliest
typed cause and do not infer hardware success.

Only after this correction is committed, pushed, clean, and verified may a
separate immutable continuation define attempt-005. That later contract must
retain the prior one-shot private intent, exact-package and board-205 binding,
five successful-real-read overlays, ordinary restoration, cleanup, privacy,
single-attempt boundary, and all prohibited effects. This plan supplies no
hardware authority itself.

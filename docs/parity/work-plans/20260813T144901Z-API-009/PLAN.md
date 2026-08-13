# Parity work plan

- Run ID: `20260813T144901Z-API-009`
- Parity row: `API-009`
- Initial status: `implemented`
- Source commit: `6f99365b9c72a4c0fed42fcc4eb8820dd4afd54f`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-api009-command-effect-evidence-audit`
- Continues plan: `docs/parity/work-plans/20260813T110706Z-API-009/PLAN.md`

## Selection

The clean synchronized selector ranks API-009 first, so no candidate is
skipped. Attempt-010 hardware-proved the resumable-pause fix and reached the
rendered IDENTIFY checkpoint. The user's later question established that the
checkpoint signal did not explain what IDENTIFY should look like. Their
description of the normal statistics page arrived only after multiple waiting
turns, beyond the firmware's pinned 30-second IDENTIFY duration, so it cannot
prove a command-to-display render failure. The prior closure incorrectly
described that late observation as live contradictory hardware evidence.

The production trace shows a complete 500-ms display path from the HTTP effect
through retained command state, atomic screen projection, `ScreenFlow`, the
retained display owner, and SSD1306 flush. The actual orchestration mismatch is
that the campaign sends the time-bounded IDENTIFY request before proving the
operator is watching and emits a machine signal containing only the word
`rendered`. This plan corrects the historical interpretation and makes the
operator checkpoint self-describing and pre-effect armed. It does not infer
that attempt-010 rendered or failed to render IDENTIFY.

The active lesson set remains above its deterministic loading budget with the
unchanged 2026-08-03 audit baseline and no new audit trigger. Complete safety,
authorization, retry, physical-observation, evidence, redaction,
earliest-failure, real-process, and host-stall lessons are loaded. Caption/VTT,
small-table deduplication, and legacy GSD separator blocks remain disclosed
unrelated omissions. Repo-local hardware/privacy guidance plus architecture,
code-shape, verification, testing, Rust, and TypeScript standards govern this
software-only continuation.

## Scope and non-scope

Replace the two-state post-effect checkpoint handoff with a closed three-state
transaction: `ready`, `rendered`, and `cleared`. The campaign must publish and
consume a request-once `ready` checkpoint before its first IDENTIFY HTTP
request. Only after readiness is consumed may it issue exactly one enable
request and publish `rendered`; only after a matching physical confirmation may
it issue exactly one disable request and publish `cleared`.

Use one v2 private checkpoint schema and one v2 public operator signal. Every
public signal must contain the safe exact expected frame (blank, `BITAXE
IDENTIFY`, `Hello!`, blank), the pinned 30-second duration, and a closed
confirmation condition distinguishing ready-to-watch, frame-visible, and
frame-absent. Rename the confirmation CLI argument from the misleading
physical-only `--observation` to `--checkpoint`, and update the Justfile-facing
commands and active tests together; historical plans remain immutable.

Correct attempt-010's CLOSURE, WORKLOG, and active task interpretation to
`stop_authority_boundary`: late normal-screen observation is neither positive
nor negative IDENTIFY evidence. Preserve the sealed aggregate facts, safe stop,
cleanup, public withholding, attempt consumption, and API-009 status.

This plan permits repository source/tests/docs changes, local fixtures and real
child processes, and firmware builds. It does not permit credentials, package
effects, detector, USB, network/device access, mining, HTTP command effects,
display confirmation, restart, recovery, OTA, destructive/fault injection,
direct UART, pins/pads/GPIO, attempt-011, checklist promotion, or public parity
evidence.

## Implementation

- [ ] Add the closed v2 `ready` / `rendered` / `cleared` checkpoint model and
      self-describing redaction-safe public signals.
- [ ] Gate the first IDENTIFY request on request-once operator readiness and
      preserve exactly one enable, one disable, ordered physical confirmations,
      earliest typed failure, cleanup, and evidence withholding.
- [ ] Add pure state-machine, CLI, malformed/order/mode, projection, and real-
      child regressions proving no IDENTIFY request can precede readiness and
      every emitted signal explains the expected physical state.
- [ ] Correct attempt-010's late-observation interpretation, keep API-009
      `implemented`, and close without hardware or checklist transition.

## Verification and promotion

Run focused Rust campaign/checkpoint/CLI tests, focused TypeScript checkpoint
and API-command-effects tests, the real-child supervisor test, generated-
contract checks, source-ownership tests, and the real firmware build. Require
the v2 signal to contain no origin, hostname, port, USB/network identity,
credential, address, token, path, sensor value, or raw trace.

Then run in order:

1. `cargo fmt --all`
2. `cargo clippy --all-targets --all-features -- -D warnings`
3. `cargo build --all-targets --all-features`
4. `cargo test --all-features`
5. `bun scripts/bright-builds-check.ts all`
6. `just test`
7. `just parity`
8. `just parity-progress`

Also run `just verify-redaction`, `just verify-reference`, `just build`,
immutable-plan digest, unique task binding, selector closure, reference
cleanliness, `git diff --check`, sensitive-output scan, and full diff review.

This software plan cannot promote API-009 or authorize attempt-011. A future
clean selector may create a separate one-attempt hardware contract only after
the pre-armed handoff is regression-proven, committed, pushed, and all normal
hardware gates pass.

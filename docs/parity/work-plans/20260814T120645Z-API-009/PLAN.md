# Parity work plan

- Run ID: `20260814T120645Z-API-009`
- Parity row: `API-009`
- Initial status: `implemented`
- Source commit: `8e8cf59e930f63c8fffd132c4e0ffd5ab1d1bc22`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-api009-command-effect-evidence-audit`
- Continues plan: `docs/parity/work-plans/20260814T044503Z-API-009/PLAN.md`

## Selection

The clean synchronized selector has no open plan and ranks API-009 first. The
sole attempt-016 proved exact-package and runtime admission, one pause, safe
stop, one resume, active recovery, and a genuine notification before it reached
the ready checkpoint. While no ready signal was present, the active device
later failed closed on one stale safety deadline. A confirmation written after
campaign closure was correctly never consumed.

An ordinal-only retry is prohibited. The actionable new work is the user-
requested asynchronous signal boundary: wait for readiness while the device is
already paused and safe-stopped, then let one private signal initiate resume and
the time-bounded IDENTIFY effect. This changes orchestration and is testable
without hardware.

## Scope and non-scope

Reorder the existing typed command-effects transaction so the pause join arms
the `ready` checkpoint without resuming. Keep the device paused and safe-stopped
through a bounded one-hour operator-ready window. Only a valid one-shot ready
confirmation may issue the single resume request; after the same trusted device
is active again, issue the first IDENTIFY request and publish the existing
30-second rendered checkpoint. Preserve the ordered rendered, cleared,
dismissal, restart, recovery, cleanup, evidence, privacy, and failure contracts.

Expose the existing private confirmation mechanism as an explicit repo-owned
signal-sender command so either the agent or user can initiate the ready,
rendered, or cleared transition for the named private attempt root. Require
mode-`0700` roots, mode-`0600` checkpoint files, exact schema/order, one-shot
consumption, and path-free public output.

This is software-only. Do not access credentials, detector or protected attempt
contents beyond the redacted attempt-016 closure facts; USB, device or network
interfaces; HTTP effects; flash, reset, restart, mining, display, hardware
controls, direct UART, pins/pads/GPIO; public parity evidence; attempt-017; or a
checklist transition.

## Implementation

- [ ] Change the pause join to enter a paused-ready phase, create `ready` only
      after logical pause plus serial safe-stop confirmation, and prove no
      resume or IDENTIFY request occurs before its one-shot signal.
- [ ] On ready consumption, issue exactly one resume; require active state on
      the same trusted boot/package before issuing exactly one IDENTIFY and the
      30-second rendered checkpoint. Preserve later ordering and recovery.
- [ ] Add a checked one-hour operator-ready component to Rust child capture and
      TypeScript parent/fixture budgets, with overflow and cross-language source
      contracts.
- [ ] Add the repo-owned signal-sender command and focused state-machine, CLI,
      checkpoint, orchestration, timeout, malformed/replay, and real-child
      regressions.

## Verification and promotion

Run focused Rust campaign/pause-join/CLI tests, TypeScript command-effects,
checkpoint, timeout, invocation, and real-process tests, then the real ESP
firmware build. Run in order: `cargo fmt --all`, strict Clippy, all-target
build, all-feature tests, Bright Builds, `just test`, `just parity`, and
`just parity-progress`. Also run redaction, reference-cleanliness, immutable
plan digest, unique task binding, selector closure, sensitive-output,
`git diff --check`, and full diff review.

Close this plan with API-009 still `implemented` after the software contract
and every gate pass. Do not promote or synchronize parity progress. Any future
attempt-017 requires this fix to be committed and pushed, a clean selector, a
separate immutable exact-effect plan, fresh paths, exact-package admission,
the detector, and the existing evidence/recovery/stop rules.

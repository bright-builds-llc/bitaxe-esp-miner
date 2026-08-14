# Parity work plan

- Run ID: `20260814T200547Z-API-009`
- Parity row: `API-009`
- Initial status: `implemented`
- Source commit: `d2e0e03f941436c541a9fff6ff5ed4d4109c7513`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-api009-command-effect-evidence-audit`
- Continues plan: `docs/parity/work-plans/20260814T194002Z-API-009/PLAN.md`

## Selection

The clean synchronized selector reports no open plan and ranks API-009 first.
Attempt-021 reached its safe unbounded ready checkpoint and started one exact
30-second IDENTIFY evidence window. The operator then reported not watching and
asked to signal the effect again. The immutable attempt correctly declined and
closed because its one-shot rendered checkpoint had no replay contract. This
is a host workflow capability gap at the exact remaining API-009 physical-
observation boundary, so API-009 remains the first actionable row.

## Scope and non-scope

Add an explicit, bounded replay protocol for operator-gated IDENTIFY evidence.
After ready starts the first exact 30-second effect window, the rendered
checkpoint must accept one local `replay` outcome even after that evidence
window expires. Replay must keep the campaign paused and safe-stopped, wait
until the prior effect is inactive, issue exactly one additional IDENTIFY
request, and open one new exact 30-second rendered-evidence window. The overall
operator wait remains unbounded; physical effect duration and replay count do
not.

Rendered confirmation is valid only during the currently active evidence
window. Replay is never inferred from silence or expiry, never issues while the
prior effect may still be active, and cannot be consumed more than once. A
second missed window must support explicit decline and safe closure. Preserve
earliest typed failures, recovery, cleanup, private modes, redaction, and the
existing five-command promotion quorum. Public checkpoint/evidence contracts
must record only closed outcomes and counts, never timing origins or protected
device facts.

This plan is software-only. It may inspect and modify repository source, tests,
task/worklog/closure artifacts, deterministic fixtures, and local child
processes, and may build firmware. It does not authorize credentials, detector,
USB, device/network interfaces, HTTP, display observation, mining, hardware
control, direct UART, pins/pads/GPIO, parity evidence, checklist promotion,
attempt-022, or any other hardware attempt.

## Implementation and verification

- [ ] Add a fast production-shaped regression that starts one IDENTIFY window,
      misses it, requests replay, proves no early second device request, then
      accepts one rendered confirmation in the replay window.
- [ ] Add negative controls for late rendered confirmation, duplicate replay,
      replay while the prior effect may still be active, and decline after the
      replay window.
- [ ] Extend the smallest typed checkpoint/state-machine boundary needed for
      one explicit replay. Preserve the fixture and real-child-process seams;
      do not add an independent ad hoc signal path.
- [ ] Prove the public contract remains redacted, cleanup/recovery preserve the
      earliest failure, and existing ready/rendered/declined behavior remains
      compatible where replay is unused.
- [ ] Pass focused checkpoint, command-effects, operator-lifetime, timeout,
      CLI, and real-process tests plus every mandatory software, privacy,
      reference, and real-firmware gate.
- [ ] Keep API-009 `implemented`, create no hardware evidence, and close this
      plan without attempt-022. A later attempt requires its own immutable
      exact-package contract after this implementation is pushed.

Before plan commit and final source/closure commit, run in order: Cargo format,
strict Clippy, all-target build, all-feature tests, Bright Builds, `just test`,
`just parity`, and `just parity-progress`. Also run focused replay and negative-
control tests, canonical real-process automation targets, `just
verify-redaction`, `just verify-reference`, `just build`, immutable plan digest,
unique task binding, selector closure, sensitive-output review, `git diff
--check`, and full diff review.

Do not transition the checklist or synchronize parity progress because this
software workflow repair changes none of API-009's checklist fields and does
not supply its required claim-specific hardware evidence.

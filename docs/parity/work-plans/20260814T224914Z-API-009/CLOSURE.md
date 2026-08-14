# Parity work closure

- Parity row: `API-009`
- Final status: `implemented`
- Outcome: `blocked`
- Verification claimed: `no`
- Plan SHA-256: `43b3913f651a1a16b66f7a761c69e6608a98da2ed230f08080c97eaf5edd00b6`
- Active task: `task-parity-api009-command-effect-evidence-audit`

## Closure reason

The sole detector-gated attempt-022 admitted one ready Ultra 205 and exact
pushed package `79756b9a`. The command campaign established trusted runtime
identity, observed a genuine notification and positive block count, and
confirmed one pause with the hardware stopped before the operator-ready
checkpoint.

The ready response was consumed after an unbounded 28-second human wait and
started one exact 30-second IDENTIFY effect. The rendered checkpoint opened one
second later. The user truthfully reported the exact frame as visible, but the
local rendered response was consumed 34 seconds after that checkpoint opened,
past the exact effect-evidence deadline. The typed late-confirmation policy
therefore recorded `operator_checkpoint_expired` / `operator_paused`. Because
the response was `confirmed`, not `replay`, the current state machine terminated
instead of opening the optional replay window. No replayed or cleared
checkpoint was created, and no resume, dismissal, or restart occurred.

The campaign claimed one recovery pause and closed with terminal safe stop
confirmed by the pause-preserving recovery contract. USB cleanup is `ready`,
recovery completed without a secondary failure, private modes are valid,
attempt processes are absent, and the public projection is withheld.

## Verification

All focused replay, real-process, mandatory Cargo, Bright Builds, Bazel,
parity, parity-progress, redaction, reference, real-firmware, exact-package,
immutable-plan, selector, unique-task, detector, protected-mode, cleanup,
process, and diff gates passed. The exact package manifest binds pushed source
`79756b9a21a1de5ed2b8ad1fedbb84b1f2f4fb89` and pinned reference
`c1915b0a63bfabebdb95a515cedfee05146c1d50`. Closed categorical artifacts agree
on the late confirmation, pause-preserving recovery, cleanup, and evidence
withholding.

## Next safe action

Keep API-009 `implemented`. Add a typed post-expiry replay-choice checkpoint:
a late `confirmed` rendered response must remain non-evidence, preserve the
pause, and offer the one bounded replay through a new unbounded human decision
instead of immediately terminating. Bind the subsequent explicit replay to a
fresh exact 30-second window and retain all late, duplicate, count, recovery,
and privacy guards. Require a new software-only plan and verified regression
before any later hardware plan. This plan authorizes no attempt-023 or
unchanged retry.

## Non-claims

This closure does not claim a timely host-bound rendered confirmation, a
replayed frame, IDENTIFY clear, resume, active recovery, dismissal, block-count
preservation, the canonical restart, restart survival, public evidence, or
API-009 verification. The user's physical observation is useful diagnostic
truth but is not promoted beyond the exact closed evidence contract. No
credential, origin, hostname, port, USB/network identity, worker, address,
password, token, sensor value, or raw trace is exposed.

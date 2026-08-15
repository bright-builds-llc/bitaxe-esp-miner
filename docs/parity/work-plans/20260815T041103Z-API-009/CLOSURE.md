# Parity work closure

- Parity row: `API-009`
- Final status: `implemented`
- Outcome: `blocked`
- Verification claimed: `no`
- Plan SHA-256: `0608272dae94a34b60c7d5f092da1321479dae9aa737f4f29cfbb86fa8e5d379`
- Active task: `task-parity-api009-command-effect-evidence-audit`

## Closure reason

The sole detector-gated attempt-026 admitted one Ultra 205 and the exact
pushed package `5621ea0fe981efbc568279af9a35923c65e9bbed`. It established
trusted runtime identity, a ready protocol gate, a genuine positive block
notification, one pause with stopped hardware, one notification dismissal
request during that pause, and confirmed notification clearing.

The sealed result then closed as `network_correlation_failed` before IDENTIFY
readiness because block-count preservation was false. The host retained the
count first observed with the active notification and compared the
post-dismissal readback against that older value. The dismissal request was
issued only after pause convergence, so a qualifying in-flight result could
legitimately advance the cumulative count before the request while dismissal
still correctly preserved the then-current counter. The comparison therefore
does not yet bind the immediate pre-request and post-request values.

No operator checkpoint opened and no IDENTIFY, resume, or canonical restart
effect occurred. Recovery issued one pause and confirmed both API paused state
and serial hardware safe stop. USB, fixture, process, private-mode, and seal
cleanup passed, and no secondary recovery failure occurred. The complete
command/restart quorum did not pass, so the public projection was correctly
withheld.

## Next safe action

Keep API-009 `implemented`. Create a fresh software-only plan that reproduces
an increment between initial notification and joined pause, then captures the
positive paused counter immediately before the one dismissal request and
requires the post-dismissal readback to equal that value. Preserve all request-
once, pause/safe-stop, IDENTIFY, recovery, sealing, and redaction invariants.
Define no attempt-027 until that repair is regression-backed, fully gated,
committed, and pushed.

## Non-claims

This closure does not claim block-count preservation, IDENTIFY display
behavior, resume, active recovery, accepted share behavior, terminal HTTP or
pool truth, canonical restart, same-device restart recovery, public parity
evidence, or API-009 verification. It exposes no origin, hostname, port,
USB/network identity, credential, worker, address, password, token, sensor
value, or raw trace.

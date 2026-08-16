# Parity work closure

- Parity row: `STAT-001`
- Final status: `implemented`
- Outcome: `blocked`
- Verification claimed: `no`
- Plan SHA-256: `876d0ba3dce066985d0e71f3b76732b4d603c6048b399dd085074b45bd7ba71f`
- Active task: `task-parity-stat001-hashrate-monitor`

## Closure reason

Exact clean pushed source and package `3b03502e12d38dc7d2cbbd7cc9a051b4c54dde09`
passed the complete software, privacy, reference, package, and one-device
detector gates. The sole attempt-003 then crossed attempt-002's corrected
boundary: package admission passed, the campaign ran as `live-share` plus
`conservative`, the protocol gate was `ready`, observation started, and 1,361
campaign markers were accepted across 366,166 active milliseconds.

The sealed result stopped as `runtime_identity_untrusted` with runtime
attestation status `malformed`. It retained 41 runtime-attestation candidates,
zero invalid-encoding candidates, a clean serial outcome, confirmed safe stop,
and ready USB cleanup. The independent workflow correctly withheld the public
projection, so STAT-001 cannot be promoted. Protected layout and result-seal
checks pass.

This is a new authoritative boundary, not a recurrence of attempt-002's
pre-package stage/profile admission failure. Current closed diagnostics do not
distinguish incomplete readiness, malformed tokens, duplicate/unknown fields,
or another parser rejection after a candidate has valid UTF-8. Instrumentation
alone is not retry progress, so the closed disposition is
`stop_hardware_blocker`; attempt-004 is not authorized.

## Next safe action

Use a separately selected software-only STAT-001 continuation to carry a
closed, redaction-safe runtime-attestation parse discriminator and bounded
per-category counts through the production serial accumulator and sealed
campaign result. Reproduce the malformed-candidate boundary without protected
values, diagnose the exact producer/parser mismatch, and apply a targeted
real-boundary regression-backed fix. A future immutable hardware plan may
consider attempt-004 only after that material fix is committed, pushed, fully
gated, and bound to a new exact package.

## Non-claims

This closure does not verify the exact package's runtime identity, BM1366
counter accuracy, HTTP or WebSocket hashrate coherence, rolling-window values,
terminal zero behavior, all twenty network windows, or STAT-001 parity. It
does not claim that every observed marker was valid or that the malformed
attestation cause is known. It does establish only the closed package,
campaign-admission, protocol-readiness, serial-outcome, safe-stop, cleanup,
privacy, sealing, and evidence-withholding facts above.

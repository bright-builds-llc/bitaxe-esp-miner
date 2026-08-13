# Parity work closure

- Parity row: `THR-001`
- Final status: `implemented`
- Outcome: `blocked`
- Verification claimed: `no`
- Plan SHA-256: `1f8ab858acc5ea543d617a1d4e275b617725a93f945011a4d27d7b7ae0b6c90a`
- Active task: `task-parity-thr001-emc2101-live-thermal`

## Closure reason

The plan's software fix and exact-package attempt-003 succeeded. The committed
closed projection independently proves the bounded read-only Ultra 205
EMC2101 observation quorum. However, the mandatory final parity verifier
classifies THR-001 as an active safety-control row and requires
`hardware-regression` evidence for `verified`. This immutable plan explicitly
prohibited overheat/fault stimulus and therefore authorized only
`hardware-smoke`. Relabeling the read-only result as hardware regression would
overclaim evidence. The uncommitted requested transition and its derived
progress/task archival artifacts were discarded after the validator rejected
them; the authoritative row remains `implemented`.

## Next safe action

Create a distinct immutable plan and active retry task that define a bounded,
non-destructive thermal hardware-regression stimulus on board 205, the exact
expected fault/safe response, temperature limits, immediate abort conditions,
restoration and cleanup, protected evidence policy, and a fresh attempt
ordinal. Attempt-003 is consumed and must not be repeated. No further hardware
effect is authorized by this closed plan.

## Non-claims

This closure does not verify overheat handling, thermal fault detection or
recovery, safe stop under thermal fault, cool/restart behavior, long-duration
sensor stability, fan/voltage/frequency/power control, mining under load,
intermittent I2C failure, other sensors or boards, OTA/recovery, or release
readiness. The committed projection is valid hardware-smoke evidence only and
does not constitute `hardware-regression` or verified THR-001 parity.

# Parity work closure

- Parity row: `REL-002`
- Final status: `implemented`
- Outcome: `superseded`
- Verification claimed: `no`
- Plan SHA-256: `fdd00fd62a1a3512f0e660a09f4b8ebafa0c31d513d8de36eed6cb2a814be646`
- Active task: `task-parity-rel002-force-close-attempt-002`

## Closure reason

The immutable plan required the helper to send FIN after the admitted prefix,
wait 250 milliseconds, reset a still-live socket, and prove the deliberately
half-open child peer closed. A direct Node 24 TCP trace and the real automation
test disproved that sequence: after the server receives FIN, a later client
reset closes the local socket but does not close the server's retained writable
half. The regression therefore stayed live and the planned completion
condition could not be met.

No implementation was retained. The experimental changes were removed, the
original files match the plan commit, and no detector, credential input,
device, network origin, firmware image, or public projection was used. The
hardware ordinal and fresh private paths remain unconsumed.

## Next safe action

Create a fresh continuation plan around the behavior proven by the direct TCP
trace: write and flush the exact header plus strict body prefix without sending
FIN, then immediately call `resetAndDestroy` and await local `close`. The real
child must use `allowHalfOpen: true`, observe neither a normal EOF completion
nor cooperative close, receive the exact prefix, and terminate from the forced
reset. Only after that regression and all software gates pass may a new clean
implementation be pushed and the still-fresh attempt-002 hardware ordinal be
considered under a new exact contract.

## Non-claims

This closure does not verify an interrupted OTA abort, full socket teardown in
production, rollback-probe boot, pending validation, native ESP-IDF rollback,
`REL-002`, recovery, mining, ASIC behavior, hardware control, another board, or
release readiness. It produces no public hardware evidence and leaves
`REL-002` at `implemented`.

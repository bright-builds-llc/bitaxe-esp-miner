# Parity work closure

- Parity row: `REL-002`
- Final status: `implemented`
- Outcome: `blocked`
- Verification claimed: `no`
- Plan SHA-256: `d35ccf94af6ee77651b5e8da9a36b65259cb33bea13e8c51353b16c2270a4a48`
- Active task: `task-parity-rel002-baseline-readiness-attempt-003`

## Closure reason

The exact clean implementation at source commit
`ef6c04fb57028c6f65641bdc36e9b48a9b38eae8` passed the complete software
gate. The sole detector admitted one Ultra 205, and the sole conditional
attempt-003 completed the exact-package factory flash, established the trusted
baseline through the new bounded readiness path, and issued the strict
reset-aborted partial OTA request. Ten post-interruption checks retained the
same normal factory build, boot session, ordinal, and partition.

The orchestration closed as `interruption_not_observed` because it searched
the API-visible retained log for the UART-only spelling
`firmware_ota_update=protocol_error`. Every retained snapshot instead contains
the firmware's canonical API-visible `firmware_ota_status=Protocol Error`
marker, plus its starting and working states. Source inspection confirms that
the status sink deliberately appends this canonical marker to the retained
buffer while the rejected spelling is emitted only through the serial logger.
The probe and rollback stages therefore did not begin even though the partial
upload reached the intended protocol-abort boundary.

The exact normal build remained installed, so no recovery flash was needed.
Recovery is incomplete only because the workflow did not enter its restoration
path after this primary category; there was no secondary recovery failure. All
private directories and files retain modes `0700` and `0600`, no owned
automation process remains, and no public projection or `RESULT.md` exists.
Attempt-003 is consumed.

## Next safe action

Create a fresh continuation that recognizes only the canonical retained OTA
status marker already emitted by the firmware. Add a production-shaped
regression proving the UART-only spelling cannot satisfy the retained-log
predicate while `firmware_ota_status=Protocol Error` can, preserve every other
identity, reset, recovery, evidence, and privacy gate, then run the complete
software gate and use fresh wrapper/attempt-004 paths. Do not reuse
attempt-003 or reinterpret it as complete rollback evidence.

## Non-claims

This closure hardware-verifies the reset-aborted partial request, retained OTA
protocol-abort status, and unchanged normal factory runtime only. It does not
verify rollback-probe upload or boot, pending validation, native ESP-IDF
rollback, `REL-002`, recovery-page behavior, mining, ASIC behavior, hardware
control, another board, or release readiness. `REL-002` remains `implemented`.

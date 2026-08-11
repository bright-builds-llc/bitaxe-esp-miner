# Parity work closure

- Parity row: `REL-002`
- Final status: `implemented`
- Outcome: `blocked`
- Verification claimed: `no`
- Plan SHA-256: `38f2ae15a2c9a631daea37c2674ac660b46ba41e9a99b024bcc6432a918960b2`
- Active task: `task-parity-rel002-retained-marker-attempt-004`

## Closure reason

The exact clean implementation at source commit
`994ddf06c70d984d8acddda5de2408d43c6302b4` passed the complete software
gate. The sole detector admitted one Ultra 205, and the sole conditional
attempt-004 completed the exact factory baseline, recognized the canonical
retained interrupted-upload marker, and completed the rollback-probe OTA
device session.

The probe session closed `ready`: it bound the same physical device, exact
probe build, one software restart, changed boot session, ordinal `N+1`, and
the expected `ota_0` postcondition with cleanup complete. A subsequent HTTP
read independently confirmed the exact probe identity, `ota_0`, changed
session, and next ordinal. The orchestration nevertheless closed as
`probe_boot_failed` because its late-reacquired serial file contained neither
the early `ota_boot_validation=rollback_probe_pending` line nor the early
passive safe-state line.

Source inspection confirms both lines are intentionally appended to the
API-visible retained log during boot. Requiring their text in a serial reader
that attaches only after USB re-enumeration is therefore an evidence-source
mismatch. The workflow used its allowed exact-package recovery flash, which
completed without a secondary failure. All private directories and files
retain modes `0700` and `0600`, no public projection or `RESULT.md` exists,
and attempt-004 is consumed.

## Next safe action

Create a fresh continuation that preserves the typed device-session serial
delivery facts but fetches the API-visible retained log after the probe and
final HTTP identity checks. Require the exact retained probe-pending and safe-
state lines for the probe boot and the exact retained safe-state line for the
rolled-back factory boot. Add late-attachment regressions with semantically
empty serial files, missing-marker recovery and primary-precedence coverage,
then run the complete gate and use fresh wrapper/attempt-005 paths. Do not
reuse attempt-004.

## Non-claims

This closure hardware-verifies the reset-aborted partial request, retained OTA
protocol abort, same-device probe OTA, exact probe boot in `ota_0` at `N+1`,
and exact-package recovery flash. It does not verify the retained probe boot
markers, normal reboot for native rollback, restored factory boot, `REL-002`,
recovery-page behavior, mining, ASIC behavior, hardware control, another
board, or release readiness. Recovery flash is not rollback parity evidence,
and `REL-002` remains `implemented`.

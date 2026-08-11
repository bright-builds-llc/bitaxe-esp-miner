# Parity work closure

- Parity row: `REL-002`
- Final status: `implemented`
- Outcome: `blocked`
- Verification claimed: `no`
- Plan SHA-256: `4d4cd0259f60728943c871adcb1336543410a294efb531a48db7ed1b7f2592bc`
- Active task: `task-parity-rel002-reset-before-fin-attempt-002`

## Closure reason

The exact clean implementation at source commit `9482dc49` passed the complete
software gate. The sole detector admitted one Ultra 205 and the sole
conditional attempt-002 completed its initial exact-package flash effect. Its
flash/monitor evidence retained one boot session, one runtime origin, and
trusted passive runtime attestation, but the monitor consumed the full
600-second capture window. The following baseline same-origin HTTP artifact
was never created, and the orchestration closed as `evidence_invalid` before
the interrupted-upload transport, probe OTA, or rollback sessions began.

The workflow used its allowed exact-package recovery flash, which completed.
Recovery had no secondary failure and did not convert the attempt into
evidence. All private directories and files retain modes `0700` and `0600`, no
owned automation process remains, and no public projection or `RESULT.md`
exists. Attempt-002 is consumed.

## Next safe action

Create a fresh continuation only after separating the short initial trusted
flash/monitor admission window from the longer device-session timeout and
typing baseline HTTP readiness failures instead of collapsing them into the
generic orchestration category. Add a production-shaped regression where the
flash child exits after trusted output and baseline HTTP is temporarily
unavailable, prove bounded retry/readiness and earliest-category preservation,
then run the full gate and use fresh wrapper/attempt-003 paths. Do not reuse
attempt-002.

## Non-claims

This closure does not hardware-verify reset-before-FIN delivery, a retained OTA
protocol abort, rollback-probe upload or boot, pending validation, native
ESP-IDF rollback, `REL-002`, recovery-page behavior, mining, ASIC behavior,
hardware control, another board, or release readiness. Recovery flash success
is not runtime parity evidence and `REL-002` remains `implemented`.

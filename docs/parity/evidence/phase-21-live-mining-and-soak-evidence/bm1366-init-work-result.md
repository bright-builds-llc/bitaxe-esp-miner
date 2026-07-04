# Phase 21 BM1366 Init Work Result

bm1366_init_work_result_status: complete
chip_detect_status: passed
chip_detect_observed_result: fail_closed
chip_detect_observed_error: partial BM1366 UART read: expected 11 bytes, read 9
work_result_status: passed for diagnostic work dispatch with bounded result handling
work_result_observed_result: fail_closed
work_result_observed_error: diagnostic result timeout after dispatch
hardware_command_status: passed
mining_allow_status: passed
redaction_status: passed-for-diagnostic-packs
safe_stop_status: confirmed diagnostic safe-state
trusted_output: true
claim_tier: diagnostic-work-result
evidence_class: hardware-smoke
board: 205
port: /dev/cu.usbmodem1101
source_commit: 8cf459514e4ba36d41b1dc11ccad9fe16f64c5d0
chip_detect_source_commit: d8fa80597992065546e19dd3af1d833aa63d0688
work_result_source_commit: 8cf459514e4ba36d41b1dc11ccad9fe16f64c5d0
reference_commit: c1915b0a63bfabebdb95a515cedfee05146c1d50
conclusion: passed for diagnostic work dispatch with bounded no-result

## Scope

Plans 21-04 and 21-05 captured package-backed BM1366 diagnostic evidence on the
detected Ultra 205 serial port. The chip-detect diagnostic package booted
through the repo-owned flash wrapper and preserved the adapter outcome as a
fail-closed partial UART read. The work-result diagnostic package then
dispatched a bounded diagnostic work frame and preserved the result path as a
fail-closed timeout.

This ledger is a diagnostic prerequisite for later live smoke and soak plans. It
is not evidence of successful BM1366 initialization, production mining,
accepted shares, rejected shares, or stable soak behavior.

## Preconditions

| Gate | Evidence | Result |
|------|----------|--------|
| Phase 21 preflight | `docs/parity/evidence/phase-21-live-mining-and-soak-evidence/preflight.md` | passed |
| Readiness audit | `docs/parity/evidence/phase-21-live-mining-and-soak-evidence/readiness-audit.md` | ready |
| Live-mining enablement package | `docs/parity/evidence/phase-21-live-mining-and-soak-evidence/live-mining-enablement.md` | ready |
| Chip-detect detector | `docs/parity/evidence/phase-21-live-mining-and-soak-evidence/bm1366-init-work-result/chip-detect/detect-ultra205.log` | passed |
| Chip-detect mining allow manifest | `docs/parity/evidence/phase-21-live-mining-and-soak-evidence/bm1366-init-work-result/chip-detect/allow-chip-detect.json` | passed |
| Work-result detector | `docs/parity/evidence/phase-21-live-mining-and-soak-evidence/bm1366-init-work-result/work-result/detect-ultra205.log` | passed |
| Work-result mining allow manifest | `docs/parity/evidence/phase-21-live-mining-and-soak-evidence/bm1366-init-work-result/work-result/allow-work-result.json` | passed |

## Commands

Fresh detector gate for the work-result run:

```bash
just detect-ultra205
```

Work-result diagnostic package:

```bash
scripts/phase15-bm1366-diagnostic-package.sh --mode work-result --out-dir docs/parity/evidence/phase-21-live-mining-and-soak-evidence/bm1366-init-work-result/work-result
```

Work-result mining allow validation:

```bash
bazel run //tools/parity:report -- mining-allow --manifest docs/parity/evidence/phase-21-live-mining-and-soak-evidence/bm1366-init-work-result/work-result/allow-work-result.json --surface bm1366-work-result --allowed-command "bazel run //tools/flash:flash -- flash-monitor --board 205 --port /dev/cu.usbmodem1101 --manifest docs/parity/evidence/phase-21-live-mining-and-soak-evidence/bm1366-init-work-result/work-result/package/bitaxe-ultra205-package.json --evidence-dir docs/parity/evidence/phase-21-live-mining-and-soak-evidence/bm1366-init-work-result/work-result --capture-timeout-seconds 35 --redact-evidence"
```

Allowed work-result hardware command:

```bash
bazel run //tools/flash:flash -- flash-monitor --board 205 --port /dev/cu.usbmodem1101 --manifest docs/parity/evidence/phase-21-live-mining-and-soak-evidence/bm1366-init-work-result/work-result/package/bitaxe-ultra205-package.json --evidence-dir docs/parity/evidence/phase-21-live-mining-and-soak-evidence/bm1366-init-work-result/work-result --capture-timeout-seconds 35 --redact-evidence
```

## Evidence Artifacts

| Artifact | Path | Result |
|----------|------|--------|
| Chip-detect detector log | `docs/parity/evidence/phase-21-live-mining-and-soak-evidence/bm1366-init-work-result/chip-detect/detect-ultra205.log` | redacted detector pass |
| Chip-detect package summary | `docs/parity/evidence/phase-21-live-mining-and-soak-evidence/bm1366-init-work-result/chip-detect/diagnostic-package-summary.json` | chip-detect package created |
| Chip-detect package manifest | `docs/parity/evidence/phase-21-live-mining-and-soak-evidence/bm1366-init-work-result/chip-detect/package/bitaxe-ultra205-package.json` | package metadata retained; binaries ignored |
| Chip-detect mining allow manifest | `docs/parity/evidence/phase-21-live-mining-and-soak-evidence/bm1366-init-work-result/chip-detect/allow-chip-detect.json` | validated before hardware command |
| Chip-detect flash command evidence | `docs/parity/evidence/phase-21-live-mining-and-soak-evidence/bm1366-init-work-result/chip-detect/flash-command-evidence.json` | `trusted_output: true` |
| Chip-detect monitor log | `docs/parity/evidence/phase-21-live-mining-and-soak-evidence/bm1366-init-work-result/chip-detect/flash-monitor.log` | redacted trusted boot and fail-closed adapter result |
| Work-result detector log | `docs/parity/evidence/phase-21-live-mining-and-soak-evidence/bm1366-init-work-result/work-result/detect-ultra205.log` | redacted detector pass |
| Work-result package summary | `docs/parity/evidence/phase-21-live-mining-and-soak-evidence/bm1366-init-work-result/work-result/diagnostic-package-summary.json` | work-result package created |
| Work-result package manifest | `docs/parity/evidence/phase-21-live-mining-and-soak-evidence/bm1366-init-work-result/work-result/package/bitaxe-ultra205-package.json` | package metadata retained; binaries ignored |
| Work-result mining allow manifest | `docs/parity/evidence/phase-21-live-mining-and-soak-evidence/bm1366-init-work-result/work-result/allow-work-result.json` | validated before hardware command |
| Work-result flash command evidence | `docs/parity/evidence/phase-21-live-mining-and-soak-evidence/bm1366-init-work-result/work-result/flash-command-evidence.json` | `trusted_output: true` |
| Work-result monitor log | `docs/parity/evidence/phase-21-live-mining-and-soak-evidence/bm1366-init-work-result/work-result/flash-monitor.log` | redacted trusted boot, work dispatch, and bounded timeout |

## Observed Markers

| Marker | Observation |
|--------|-------------|
| Safe state | `mining=disabled`, `asic_work_submission=disabled`, `hardware_control=disabled` |
| Chip-detect diagnostic mode | `asic_status=chip_detect_only initialized=false mining=disabled work_submission=disabled` |
| Chip-detect adapter result | `asic_status=fail_closed reason=chip_detect_adapter_error` |
| Chip-detect adapter error | `partial BM1366 UART read: expected 11 bytes, read 9` |
| Work-result diagnostic start | `asic_work_result_diagnostic=started mining=disabled work_submission=disabled` |
| Work-result dispatch | `bm1366_diagnostic_work=dispatched job_id=0x28 bytes=88 mining=disabled` |
| Work-result result handling | `bm1366_diagnostic_result=timeout fail_closed=true mining=disabled work_submission=disabled` |
| Work-result adapter result | `asic_status=fail_closed reason=work_result_diagnostic_timeout initialized=false mining=disabled work_submission=disabled` |
| Reset behavior | reset held low after adapter errors |
| Mining loop | blocked with `work_submission=disabled` |
| Work-result firmware identity | observed firmware commit prefix matched `8cf459514e4b` |
| Reference identity | observed reference commit matched `c1915b0a63bfabebdb95a515cedfee05146c1d50` |

## Non-Claims

This evidence does not claim:

- Successful BM1366 initialization
- Accepted shares
- Rejected shares
- Live pool connectivity
- Production mining
- Production work dispatch to ASICs
- Frequency transition
- Voltage-control correctness
- Fan-control correctness
- Live API/WebSocket telemetry
- Bounded soak stability
- OTA, erase, rollback, or interrupted-update behavior
- Unbounded thermal, power, or mining stress safety

## Redaction Review

The committed diagnostic packs use wrapper redaction and were scanned before
citation. Raw detector output and generated binary images are not committed.
The remaining scan hits are schema or status vocabulary such as `pool_config:
not-used`, `device_url: not-used`, wrapper redaction metadata, and review
contract terms. No unredacted IP addresses, MAC addresses, target URLs, SSIDs,
pool credentials, worker values, API tokens, NVS secrets, or secret-bearing
configuration values were found in the committed diagnostic artifacts.

## Conclusion

Phase 21 captured package-backed BM1366 chip-detect and work-result diagnostic
evidence. Both hardware commands completed with trusted wrapper output and
redacted artifacts. The observed ASIC behavior remains fail-closed: chip detect
hit a partial UART read, and work-result dispatched diagnostic work but timed
out waiting for a result. This ledger supports diagnostic prerequisites for
later gated smoke/soak plans only and must not be cited as live mining or
successful ASIC initialization evidence.

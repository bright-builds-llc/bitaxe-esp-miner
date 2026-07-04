# Phase 21 BM1366 Init Work Result

bm1366_init_work_result_status: chip-detect-diagnostic-captured
chip_detect_status: passed
chip_detect_observed_result: fail_closed
chip_detect_observed_error: partial BM1366 UART read: expected 11 bytes, read 9
hardware_command_status: passed
mining_allow_status: passed
redaction_review_status: passed
trusted_output: true
claim_tier: diagnostic-chip-detect
evidence_class: hardware-smoke
board: 205
port: /dev/cu.usbmodem1101
source_commit: d8fa80597992065546e19dd3af1d833aa63d0688
reference_commit: c1915b0a63bfabebdb95a515cedfee05146c1d50

## Scope

Plan 21-04 captured package-backed BM1366 chip-detect diagnostic evidence on the
detected Ultra 205 serial port. The diagnostic package booted through the
repo-owned flash wrapper, emitted trusted safe-state markers, and preserved the
BM1366 adapter outcome as hardware evidence.

The observed BM1366 outcome was fail-closed: the adapter attempted chip detect,
then reported a partial UART read and held reset low. This is valid diagnostic
evidence for Plan 21-04, but it is not evidence of successful BM1366
initialization, work dispatch, or mining.

## Preconditions

| Gate | Evidence | Result |
|------|----------|--------|
| Phase 21 preflight | `docs/parity/evidence/phase-21-live-mining-and-soak-evidence/preflight.md` | passed |
| Readiness audit | `docs/parity/evidence/phase-21-live-mining-and-soak-evidence/readiness-audit.md` | ready |
| Live-mining enablement package | `docs/parity/evidence/phase-21-live-mining-and-soak-evidence/live-mining-enablement.md` | ready |
| Fresh Ultra 205 detector | `docs/parity/evidence/phase-21-live-mining-and-soak-evidence/bm1366-init-work-result/chip-detect/detect-ultra205.log` | passed |
| Mining allow manifest | `docs/parity/evidence/phase-21-live-mining-and-soak-evidence/bm1366-init-work-result/chip-detect/allow-chip-detect.json` | passed |

## Commands

Detector gate:

```bash
just detect-ultra205
```

Diagnostic package:

```bash
scripts/phase15-bm1366-diagnostic-package.sh --mode chip-detect --out-dir docs/parity/evidence/phase-21-live-mining-and-soak-evidence/bm1366-init-work-result/chip-detect
```

Mining allow validation:

```bash
bazel run //tools/parity:report -- mining-allow --manifest docs/parity/evidence/phase-21-live-mining-and-soak-evidence/bm1366-init-work-result/chip-detect/allow-chip-detect.json --surface bm1366-chip-detect --allowed-command "bazel run //tools/flash:flash -- flash-monitor --board 205 --port /dev/cu.usbmodem1101 --manifest docs/parity/evidence/phase-21-live-mining-and-soak-evidence/bm1366-init-work-result/chip-detect/package/bitaxe-ultra205-package.json --evidence-dir docs/parity/evidence/phase-21-live-mining-and-soak-evidence/bm1366-init-work-result/chip-detect --capture-timeout-seconds 45 --redact-evidence"
```

Allowed hardware command:

```bash
bazel run //tools/flash:flash -- flash-monitor --board 205 --port /dev/cu.usbmodem1101 --manifest docs/parity/evidence/phase-21-live-mining-and-soak-evidence/bm1366-init-work-result/chip-detect/package/bitaxe-ultra205-package.json --evidence-dir docs/parity/evidence/phase-21-live-mining-and-soak-evidence/bm1366-init-work-result/chip-detect --capture-timeout-seconds 45 --redact-evidence
```

## Evidence Artifacts

| Artifact | Path | Result |
|----------|------|--------|
| Detector log | `docs/parity/evidence/phase-21-live-mining-and-soak-evidence/bm1366-init-work-result/chip-detect/detect-ultra205.log` | redacted detector pass |
| Package summary | `docs/parity/evidence/phase-21-live-mining-and-soak-evidence/bm1366-init-work-result/chip-detect/diagnostic-package-summary.json` | chip-detect package created |
| Package manifest | `docs/parity/evidence/phase-21-live-mining-and-soak-evidence/bm1366-init-work-result/chip-detect/package/bitaxe-ultra205-package.json` | package metadata retained; binaries ignored |
| Mining allow manifest | `docs/parity/evidence/phase-21-live-mining-and-soak-evidence/bm1366-init-work-result/chip-detect/allow-chip-detect.json` | validated before hardware command |
| Flash command evidence | `docs/parity/evidence/phase-21-live-mining-and-soak-evidence/bm1366-init-work-result/chip-detect/flash-command-evidence.json` | `trusted_output: true` |
| Monitor log | `docs/parity/evidence/phase-21-live-mining-and-soak-evidence/bm1366-init-work-result/chip-detect/flash-monitor.log` | redacted trusted boot and fail-closed adapter result |

## Observed Markers

| Marker | Observation |
|--------|-------------|
| Safe state | `mining=disabled`, `asic_work_submission=disabled`, `hardware_control=disabled` |
| Diagnostic mode | `asic_status=chip_detect_only initialized=false mining=disabled work_submission=disabled` |
| Adapter result | `asic_status=fail_closed reason=chip_detect_adapter_error` |
| Adapter error | `partial BM1366 UART read: expected 11 bytes, read 9` |
| Reset behavior | reset held low after adapter error |
| Mining loop | blocked with `work_submission=disabled` |
| Firmware identity | observed firmware commit prefix matched `d8fa80597992` |
| Reference identity | observed reference commit matched `c1915b0a63bfabebdb95a515cedfee05146c1d50` |

## Non-Claims

This evidence does not claim:

- Successful BM1366 initialization
- Accepted shares
- Rejected shares
- Live pool connectivity
- Production mining
- Work dispatch to ASICs
- Frequency transition
- Voltage-control correctness
- Fan-control correctness
- OTA, erase, rollback, or interrupted-update behavior
- Bounded soak stability
- Unbounded thermal, power, or mining stress safety

## Redaction Review

The committed chip-detect pack uses wrapper redaction and was scanned before
citation. Raw detector output and generated binary images are not committed.
The remaining scan hits are schema or status vocabulary such as `pool_config:
not-used`, `device_url: not-used`, wrapper redaction metadata, and review
contract terms. No unredacted IP addresses, MAC addresses, target URLs, SSIDs,
pool credentials, worker values, API tokens, NVS secrets, or secret-bearing
configuration values were found in the committed chip-detect artifacts.

## Conclusion

Plan 21-04 captured package-backed BM1366 chip-detect diagnostic evidence. The
hardware command completed with trusted wrapper output and redacted artifacts.
The BM1366 adapter outcome remains fail-closed on a partial UART read, so this
ledger supports diagnostic follow-up only and must not be cited as live mining
or successful ASIC initialization evidence.

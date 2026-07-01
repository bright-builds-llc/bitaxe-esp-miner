# Phase 15 BM1366 Chip-Detect Evidence

## Scope

surface: bm1366-chip-detect

chip_detect_status: passed for package-backed chip-detect smoke

conclusion: passed for package-backed chip-detect smoke

This evidence records a package-backed Ultra 205 BM1366 chip-detect diagnostic
run. It supports only the observed chip-detect-only, partial-read, no-mining,
fail-closed, trusted wrapper capture. It does not support full BM1366
initialization, production mining, frequency transition, voltage behavior,
work-send, result-receive, accepted shares, live API telemetry, WebSocket
telemetry, statistics producer behavior, OTA, recovery, rollback, erase, or
release-readiness claims.

non_claims: production mining, frequency transition, voltage behavior, work-send, result-receive, accepted shares

## Live Hardware Command Sequence

Every live hardware command in this pack was preceded by the detector gate.

1. Detector gate:

```bash
just detect-ultra205
```

Result: passed. The captured detector output is
`docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/chip-detect/detect-ultra205.log`.
It selected board `205` port `/dev/cu.usbmodem1101` and recorded successful
board-info output for ESP32-S3.

2. Approved package-backed flash-monitor capture:

```bash
bazel run //tools/flash:flash -- flash-monitor --board 205 --port /dev/cu.usbmodem1101 --manifest docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/chip-detect/package/bitaxe-ultra205-package.json --evidence-dir docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/chip-detect --capture-timeout-seconds 35
```

Result: passed. The wrapper wrote
`docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/chip-detect/flash-command-evidence.json`
and
`docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/chip-detect/flash-monitor.log`.

No raw BM1366 serial command, direct pool command, voltage command, fan command,
erase command, rollback command, interrupted-update command, or unbounded stress
command was run for this pack.

## Package And Allow Gate

Diagnostic package command:

```bash
scripts/phase15-bm1366-diagnostic-package.sh --mode chip-detect --out-dir docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/chip-detect
```

Package manifest:
`docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/chip-detect/package/bitaxe-ultra205-package.json`

Source commit: `804eaa81f1184a35864e5681361733c93242ded9`

Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`

Mining allow manifest:
`docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/chip-detect/allow-chip-detect.json`

Allow validation command:

```bash
bazel run //tools/parity:report -- mining-allow --manifest docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/chip-detect/allow-chip-detect.json --surface bm1366-chip-detect --allowed-command "bazel run //tools/flash:flash -- flash-monitor --board 205 --port /dev/cu.usbmodem1101 --manifest docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/chip-detect/package/bitaxe-ultra205-package.json --evidence-dir docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/chip-detect --capture-timeout-seconds 35"
```

Allow validation result: passed for `bm1366-chip-detect`,
`diagnostic-chip-detect`, `hardware-smoke`.

## Wrapper Trust

Machine evidence:
`docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/chip-detect/flash-command-evidence.json`

- `command_kind`: `flash-monitor`
- `board`: `205`
- `port`: `/dev/cu.usbmodem1101`
- `capture_mode`: `noninteractive`
- `capture_status`: `timed_out_after_trusted_output`
- `trusted_output`: `true`
- `observed_firmware_commit`: `804eaa81f118`
- `observed_reference_commit`: `c1915b0a63bfabebdb95a515cedfee05146c1d50`

Observed trusted wrapper markers:

- `bitaxe-rust boot: board=Ultra 205 asic=BM1366`
- `safe_state: mining=disabled asic_work_submission=disabled hardware_control=disabled`
- `spiffs_mount=available`
- `axeos_api_route_shell=started`
- `reset_reason=11`
- `firmware_commit=804eaa81f118`
- `reference_commit=c1915b0a63bfabebdb95a515cedfee05146c1d50`
- `esp_idf_version=v5.5.4`

Observed BM1366 diagnostic markers:

- `asic_status=chip_detect_only initialized=false mining=disabled work_submission=disabled`
- `asic_status=fail_closed reason=chip_detect_adapter_error error=partial BM1366 UART read: expected 11 bytes, read 9`
- `asic_status=hold_reset_low gpio=1`
- `asic_status=fail_closed reason=chip_detect_adapter_error initialized=false mining=disabled work_submission=disabled`
- `mining_loop_status=blocked reason=hardware_evidence_ack_missing work_submission=disabled`

The diagnostic reached a trusted package-backed chip-detect-only path, observed
the same bounded partial UART read condition as Phase 12, and remained in a
no-mining fail-closed state with reset held low. This closes the Phase 12
wrapper trust root cause for chip-detect smoke, but it does not promote broad
ASIC or mining behavior.

## Redaction

redaction_status: passed

Redaction review is recorded in
`docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/redaction-review.md`.
The review covers the detector log, mining allow manifest, diagnostic package
summary, package manifest, wrapper JSON, serial log, this Markdown evidence,
and terminal command outputs used for this pack.

Retained bench evidence includes the USB port, MAC address, source commit,
reference commit, package paths, checksums, local absolute paths in wrapper
JSON, and chip-detect serial markers. No pool credentials, worker secrets,
Wi-Fi credentials, private endpoints, private `DEVICE_URL`, API tokens, NVS
secret values, or local terminal secrets were found.

## Residual Blockers

- BM1366 work-send and result-receive evidence remains pending for later Phase
  15 work-result diagnostics.
- Controlled mining smoke and bounded soak remain pending until their own
  allow manifests, safety prerequisites, telemetry decisions, safe-stop evidence,
  and redaction reviews exist.
- Frequency transition, voltage behavior, fan behavior, live API/WebSocket
  telemetry, statistics producer behavior, OTA, recovery, rollback, erase, and
  release evidence remain out of scope for this chip-detect pack.

# Phase 15 BM1366 Work/Result Evidence

## Scope

surface: bm1366-work-result

result_category: bounded timeout with fail-closed state

conclusion: passed for diagnostic work dispatch with bounded no-result

This evidence records a package-backed Ultra 205 BM1366 typed work/result
diagnostic. It supports only the observed diagnostic work dispatch followed by
a bounded result timeout and fail-closed state. It does not support production
mining, accepted shares, live pool behavior, API/WebSocket telemetry,
statistics producer behavior, frequency transition, voltage behavior, or fan
behavior.

non_claims: accepted shares, live pool behavior, production mining, API/WebSocket telemetry, statistics producer behavior, frequency transition, voltage behavior, fan behavior

## Live Hardware Command Sequence

Every live hardware command in this pack was preceded by the detector gate.

1. Detector gate:

```bash
just detect-ultra205
```

Result: passed. The captured detector output is
`docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/work-result/detect-ultra205.log`.
It selected board 205 `port=/dev/cu.usbmodem1101` and recorded successful
board-info output for ESP32-S3.

2. Diagnostic package build:

```bash
scripts/phase15-bm1366-diagnostic-package.sh --mode work-result --out-dir docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/work-result
```

Result: passed. The diagnostic package summary is
`docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/work-result/diagnostic-package-summary.json`.

3. Allow validation:

```bash
bazel run //tools/parity:report -- mining-allow --manifest docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/work-result/allow-work-result.json --surface bm1366-work-result --allowed-command "bazel run //tools/flash:flash -- flash-monitor --board 205 --port /dev/cu.usbmodem1101 --manifest docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/work-result/package/bitaxe-ultra205-package.json --evidence-dir docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/work-result --capture-timeout-seconds 45"
```

Result: passed for `bm1366-work-result`, `diagnostic-work-result`,
`hardware-smoke`.

4. Approved package-backed flash-monitor capture:

```bash
bazel run //tools/flash:flash -- flash-monitor --board 205 --port /dev/cu.usbmodem1101 --manifest docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/work-result/package/bitaxe-ultra205-package.json --evidence-dir docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/work-result --capture-timeout-seconds 45
```

Result: passed. The wrapper wrote
`docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/work-result/flash-command-evidence.json`
and
`docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/work-result/flash-monitor.log`.

No raw BM1366 serial command, direct pool command, voltage command, fan command,
erase command, rollback command, interrupted-update command, or unbounded stress
command was run for this pack.

## Package And Allow Gate

Package manifest:
`docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/work-result/package/bitaxe-ultra205-package.json`

Source commit: `d7d965ffdab1e589f6ea7dea81eb8edacf7f4c86`

Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`

Mining allow manifest:
`docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/work-result/allow-work-result.json`

Prerequisite artifact:
`docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/chip-detect.md`

## Wrapper Trust

Machine evidence:
`docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/work-result/flash-command-evidence.json`

- `command_kind`: `flash-monitor`
- `board`: `205`
- `port`: `/dev/cu.usbmodem1101`
- `capture_mode`: `noninteractive`
- `capture_status`: `timed_out_after_trusted_output`
- `trusted_output`: `true`
- `observed_firmware_commit`: `d7d965ffdab1`
- `observed_reference_commit`: `c1915b0a63bfabebdb95a515cedfee05146c1d50`

Observed trusted wrapper markers:

- `bitaxe-rust boot: board=Ultra 205 asic=BM1366`
- `safe_state: mining=disabled asic_work_submission=disabled hardware_control=disabled`
- `spiffs_mount=available`
- `axeos_api_route_shell=started`
- `reset_reason=11`
- `firmware_commit=d7d965ffdab1`
- `reference_commit=c1915b0a63bfabebdb95a515cedfee05146c1d50`
- `esp_idf_version=v5.5.4`

Observed BM1366 diagnostic markers:

- `asic_work_result_diagnostic=started mining=disabled work_submission=disabled`
- `bm1366_diagnostic_work=dispatched job_id=0x28 bytes=88 mining=disabled`
- `asic_status=hold_reset_low gpio=1`
- `bm1366_diagnostic_result=timeout fail_closed=true mining=disabled work_submission=disabled`
- `asic_status=fail_closed reason=work_result_diagnostic_timeout initialized=false mining=disabled work_submission=disabled`
- `mining_loop_status=blocked reason=hardware_evidence_ack_missing work_submission=disabled`

The diagnostic reached a trusted package-backed work-result path, dispatched
one typed diagnostic work frame for job `0x28`, registered that job as valid in
firmware, observed no result frame before the bounded timeout, and remained in
a no-mining fail-closed state with reset held low. This supports diagnostic
work dispatch and bounded result timeout evidence only; it does not promote
production mining behavior.

## Redaction

redaction_status: passed

Redaction review is recorded in
`docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/redaction-review.md`.
The review covers the detector log, mining allow manifest, diagnostic package
summary, package manifest, wrapper JSON, serial log, this Markdown evidence,
and terminal command outputs used for this pack.

Retained bench evidence includes the USB port, MAC address, source commit,
reference commit, package paths, checksums, local absolute paths in wrapper
JSON, and work-result serial markers. No pool credentials, worker secrets,
Wi-Fi credentials, private endpoints, private `DEVICE_URL`, API tokens, NVS
secret values, or local terminal secrets were found.

## Residual Blockers

- Result-frame parsing remains limited to the diagnostic result-or-timeout
  path; no valid nonce/result frame was observed in this run.
- Controlled mining smoke and bounded soak remain pending until their own
  allow manifests, safety prerequisites, telemetry decisions, safe-stop
  evidence, and redaction reviews exist.
- Accepted shares, live pool behavior, production mining, API/WebSocket
  telemetry, statistics producer behavior, frequency transition, voltage
  behavior, fan behavior, OTA, recovery, rollback, erase, and release evidence
  remain out of scope for this work-result pack.

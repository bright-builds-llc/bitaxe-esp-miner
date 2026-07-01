# Phase 15 Bounded Soak Evidence

## Scope

surface: bounded-soak

bounded_soak_status: pending - missing live prerequisites

pool_category=controlled-no-share

share_outcome=controlled no-share condition

controlled_no_share_condition=missing_live_prerequisites

duration_seconds=120

conclusion: controlled no-share condition - bounded live soak not run

This evidence records the bounded-soak wrapper contract after the controlled
mining-smoke artifact recorded a controlled no-share condition. It does not
record a live bounded mining soak, accepted share, rejected share, production
mining, live API telemetry, WebSocket frame cadence, statistics producer
behavior, reconnect behavior, fallback-pool behavior, frequency transition,
voltage behavior, or fan behavior.

non_claims: live bounded soak, accepted shares, rejected shares, live pool behavior, production mining, API/WebSocket telemetry, statistics producer behavior, reconnect behavior, fallback-pool behavior, frequency transition, voltage behavior, fan behavior

## Prerequisites

Chip-detect prerequisite:
`docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/chip-detect.md`

Work/result prerequisite:
`docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/work-result.md`

Controlled smoke prerequisite:
`docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/mining-smoke.md`

The controlled smoke artifact records
`controlled_no_share_condition=missing_live_prerequisites`, so this bounded
soak pack records a pending bounded-soak status rather than running live mining
soak hardware.

Live prerequisites were not present:

- `DEVICE_URL`: missing
- Pool endpoint: missing
- Pool worker: missing
- Pool password/credential: missing

## Detector Gate

Detector command:

```bash
just detect-ultra205
```

Result: passed. The detector output is recorded in
`docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/bounded-soak/detect-ultra205.log`.
It selected board `205` port `/dev/cu.usbmodem1101` and recorded successful
board-info output for ESP32-S3.

No live bounded mining soak, pool socket command, voltage command, fan command,
direct BM1366 command, or unbounded stress command was run by this pack.

## Mining Allow Gate

Mining allow manifest:
`docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/bounded-soak/allow-bounded-soak.json`

Package manifest used for allow identity:
`docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/work-result/package/bitaxe-ultra205-package.json`

Package source commit: `d7d965ffdab1e589f6ea7dea81eb8edacf7f4c86`

Wrapper source commit: `edd5f4f8ac254513d412fae16f6cc1af3f33e455`

Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`

Wrapper command:

```bash
scripts/phase15-controlled-mining.sh --manifest docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/bounded-soak/allow-bounded-soak.json --surface bounded-soak --duration-seconds 120 --out-dir docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/bounded-soak --chip-detect-summary docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/chip-detect.md --work-result-summary docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/work-result.md
```

Allow validation result from
`docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/bounded-soak/bounded-soak.log`:

- `mining_allow_status: passed`
- `surface: bounded-soak`
- `claim_tier: unsupported-pending`
- `evidence_class: workflow`

The unsupported-pending claim tier is intentional because no live soak was run.

## Abort Conditions And Observations

Raw wrapper log:
`docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/bounded-soak/bounded-soak.log`

Planned bounded-soak duration:

- `duration_seconds=120`

Abort conditions recorded:

- `unsafe_temperature_or_power`
- `watchdog_unresponsive`
- `serial_silence`
- `redaction_uncertainty`
- `missing_safe_stop`

Observed blocker/status markers:

- `controlled_mining_status: controlled-no-share`
- `pool_category=controlled-no-share`
- `controlled_no_share_condition=missing_live_prerequisites`
- `share_outcome=controlled no-share condition`
- `hashrate_inputs_status=pending - no live pool work submitted`
- `api_telemetry_status=pending - missing DEVICE_URL`
- `websocket_frame_status=pending - missing DEVICE_URL or helper blocked`
- `watchdog_status=pending - live prerequisites missing`
- `safe_stop_status=confirmed-or-pending`
- `bounded_soak_status: controlled-no-share - missing live prerequisites`

No reconnect or fallback-pool behavior was exercised. Final safe-stop status is
derived from the already passed chip-detect and work/result safe-state markers;
no new active mining state was entered.

## Redaction

redaction_status: passed

Redaction review is recorded in
`docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/redaction-review.md`.
The review covers the detector log, mining allow manifest, wrapper log, this
Markdown evidence, and terminal output used for this pack.

Retained bench evidence includes the USB port, MAC address, source commit,
reference commit, package path, controlled pool category, no-share outcome,
bounded duration, and safe-state markers. No pool credentials, worker secrets,
Wi-Fi credentials, private endpoints, private device URL values, API tokens,
NVS secret values, or local terminal secrets were found.

## Residual Blockers

- Live bounded soak remains pending until a controlled live smoke pass or
  explicitly approved controlled no-share soak prerequisites exist with a
  reachable device URL and redaction-cleared telemetry.
- Watchdog/API/WebSocket/serial responsiveness under live mining load remains
  below verified because live prerequisites were missing.
- Accepted shares, rejected shares, live pool behavior, production mining, and
  statistics producer behavior remain out of scope for this pending bounded-soak
  artifact.

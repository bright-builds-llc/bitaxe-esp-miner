# Phase 15 Controlled Mining Smoke Evidence

## Scope

surface: mining-smoke

mining_smoke_status: controlled-no-share

pool_category=controlled-no-share

share_outcome=controlled no-share condition

controlled_no_share_condition=missing_live_prerequisites

conclusion: controlled no-share condition - missing live prerequisites

This evidence records a Phase 15 controlled mining-smoke wrapper run after the
package-backed chip-detect and work/result prerequisites passed. It supports
only the observed controlled no-share classification caused by missing explicit
live prerequisites. It does not support accepted shares, rejected shares, live
pool behavior, production mining, live API telemetry, WebSocket frame cadence,
statistics producer behavior, frequency transition, voltage behavior, fan
behavior, or bounded live soak behavior.

non_claims: accepted shares, rejected shares, live pool behavior, production mining, live API/WebSocket telemetry, statistics producer behavior, frequency transition, voltage behavior, fan behavior, bounded live soak

## Prerequisites

Chip-detect prerequisite:
`docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/chip-detect.md`

- Required conclusion observed: `passed for package-backed chip-detect smoke`.
- Safe markers observed in the prerequisite pack: `safe_state: mining=disabled`,
  `hardware_control=disabled`, and `work_submission=disabled`.
- Redaction status observed: `passed`.

Work/result prerequisite:
`docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/work-result.md`

- Required conclusion observed:
  `passed for diagnostic work dispatch with bounded no-result`.
- Safe markers observed in the prerequisite pack: `safe_state: mining=disabled`,
  `work_submission=disabled`, and `fail_closed=true`.
- Redaction status observed: `passed`.

Live prerequisites were not present:

- `DEVICE_URL`: missing
- Pool endpoint: missing
- Pool worker: missing
- Pool password/credential: missing

Missing live prerequisites are not an authentication failure. They deliberately
produce controlled no-share evidence instead of a live-pool overclaim.

## Detector Gate

Detector command:

```bash
just detect-ultra205
```

Result: port selected. The detector output is recorded in
`docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/mining-smoke/detect-ultra205.log`.
The checked-in artifact records `port=/dev/cu.usbmodem1101` only; it does not
include the `espflash board-info` transcript. Board-info evidence for this
controlled-no-share pack remains pending and is not cited for promotion.

No live mining command, pool socket command, voltage command, fan command,
direct BM1366 command, or unbounded stress command was run by this pack.

## Mining Allow Gate

Mining allow manifest:
`docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/mining-smoke/allow-mining-smoke.json`

Package manifest used for allow identity:
`docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/work-result/package/bitaxe-ultra205-package.json`

Package source commit: `d7d965ffdab1e589f6ea7dea81eb8edacf7f4c86`

Wrapper source commit: `edd5f4f8ac254513d412fae16f6cc1af3f33e455`

Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`

Wrapper command:

```bash
scripts/phase15-controlled-mining.sh --manifest docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/mining-smoke/allow-mining-smoke.json --surface mining-smoke --out-dir docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/mining-smoke --chip-detect-summary docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/chip-detect.md --work-result-summary docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/work-result.md
```

Allow validation result from
`docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/mining-smoke/mining-smoke.log`:

- `mining_allow_status: passed`
- `surface: mining-smoke`
- `claim_tier: controlled-no-share`
- `evidence_class: hardware-smoke`

## Observations

Raw wrapper log:
`docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/mining-smoke/mining-smoke.log`

- `controlled_mining_status: controlled-no-share`
- `pool_category=controlled-no-share`
- `controlled_no_share_condition=missing_live_prerequisites`
- `share_outcome=controlled no-share condition`
- `hashrate_inputs_status=pending - no live pool work submitted`
- `api_telemetry_status=pending - missing DEVICE_URL`
- `websocket_frame_status=pending - missing DEVICE_URL or helper blocked`
- `watchdog_status=pending - live prerequisites missing`
- `safe_stop_status=confirmed-or-pending`

Serial responsiveness was not exercised by this pack beyond the successful
detector gate because live mining was not allowed without explicit pool and
device URL prerequisites. Safe-stop status is derived from the already passed
chip-detect and work/result safe-state markers; no new active mining state was
entered.

## Redaction

redaction_status: passed

Redaction review is recorded in
`docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/redaction-review.md`.
The review covers the detector log, mining allow manifest, wrapper log, this
Markdown evidence, and terminal output used for this pack.

Retained bench evidence includes the USB port, MAC address, source commit,
reference commit, package path, controlled pool category, no-share outcome, and
safe-state markers. No pool credentials, worker secrets, Wi-Fi credentials,
private endpoints, private device URL values, API tokens, NVS secret values, or
local terminal secrets were found.

## Residual Blockers

- Live pool micro-smoke remains pending until disposable or non-secret pool
  configuration and an explicit reachable device URL are provided.
- API and WebSocket telemetry remain below verified because no explicit device
  URL was provided.
- Accepted shares, rejected shares, live pool behavior, production mining,
  statistics producer behavior, and bounded live soak remain out of scope for
  this controlled no-share artifact.

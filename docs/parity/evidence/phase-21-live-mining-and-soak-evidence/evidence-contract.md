# Phase 21 Evidence Contract

This contract defines the Phase 21 evidence ladder for Ultra 205 board `205` live mining and bounded soak claims. It is a claim boundary document, not live hardware evidence.

Phase 21 evidence ladder: `preflight, live-mining-enablement, bm1366-init-work-result, live-mining-smoke, bounded-soak, live-api-websocket-telemetry, redaction-review, final-summary`

## Evidence Packs

| Pack | Required before | Required contents | Claim boundary |
|------|-----------------|-------------------|----------------|
| `preflight` | Any hardware-capable command | `just detect-ultra205` output proving exactly one board `205` ESP32-S3 port, `board-info` pass, source commit, reference commit, package manifest, exact command, and safe-state baseline | Allows later package or hardware evidence to identify the board and image only |
| `live-mining-enablement` | Any `live-mining-smoke` or `bounded-soak` command | `controlled_live_mining_package_status: ready`, `controlled_runtime_harness_status: ready`, runtime harness summary, safe-stop plan, abort conditions, and redaction status | Allows controlled live mining attempts to begin; it does not prove shares or soak behavior |
| `bm1366-init-work-result` | Live work dispatch claims | Package-backed chip-detect and work-result summaries from trusted wrappers, safe-state markers, and BM1366 typed dispatch/result conclusions | Proves only the BM1366 init/work/result surfaces cited by the artifact |
| `live-mining-smoke` | Bounded soak or STR-008 promotion | Pool lifecycle, subscribe, authorize, notify/job flow, BM1366 work dispatch, result handling, accepted/rejected share outcome when observed, hashrate inputs, API/WebSocket status when an explicit target exists, watchdog breadcrumbs, and final safe-stop | Supports only exact observed smoke claims |
| `bounded-soak` | Final soak or watchdog claims | Duration, abort conditions, thermal/power/watchdog observations, pool lifecycle, share outcome or approved no-share status, periodic API/WebSocket snapshots when a target is explicit, final safe-stop, and conclusion | Supports only bounded duration and outcome claims |
| `live-api-websocket-telemetry` | API/WebSocket telemetry claims | Explicit target capture, `/api/system/info`, `/api/ws/live`, run correlation to serial/runtime observation, redaction review, and freshness notes | Proves only the captured API/WebSocket window |
| `redaction-review` | Any committed/shared evidence citation | Deterministic scan command, artifact inventory, reviewer result, and explicit `raw_artifacts_committed: no` status | Allows committed evidence citation only after pass |
| `final-summary` | Checklist promotion | Exact artifact list, exact checklist rows, non-claims, blockers, verification commands, and final claim matrix | Controls final parity row updates |

## Decision Coverage

- D-05: The ladder starts from detector/package/safe boot, then package-backed chip-detect or staged init, typed work/result evidence, live-pool mining smoke, bounded soak, and exact checklist promotion.
- D-06: Later tiers must not run when earlier tiers are missing, failed, stale, redaction-blocked, or lack trusted wrapper/package markers. The expected output is pending or blocked evidence with the exact blocker.
- D-08: Evidence must not rely on raw BM1366 writes, raw pool commands, ad hoc voltage/fan controls, erase commands, rollback commands, interrupted-update commands, unbounded stress, or hidden local scripts.
- D-11: Accepted share and rejected share evidence are exact observed outcomes. The phase must not synthesize rejected-share claims from setup intent or no-share windows.
- D-12: A controlled fake-pool or local harness must be labeled as controlled evidence. It may support flow, work, telemetry, watchdog, and no-share boundaries, but it does not prove live production pool behavior by itself.
- D-13: Bounded soak durations must stay inside `60..600` seconds unless a later plan changes the allow-manifest validator and documents the reason.
- D-16: Unexpected reboot, watchdog panic, unsafe temperature or power marker, detector mismatch, missing trusted wrapper marker, redaction uncertainty, lost pool control, or missing safe-state marker is a stop condition.
- D-17 and D-18: Telemetry correlation requires an explicit target and checks runtime state, API response, WebSocket frame, statistics, scoreboard, share counters, pool difficulty, hashrate inputs, mining activity, and work-submission state when those surfaces are present.
- D-20, D-21, and D-22: Checklist promotion must be exact-claim only. `ASIC-007` remains below verified without bounded frequency-transition hardware-regression evidence, and `STR-008` verified requires mining-smoke or soak details with board, port, firmware/source commit, reference commit, redaction, conclusion, and either accepted/rejected share outcome or an approved bounded controlled no-share soak without blocker language.

## Enablement Gate

Current firmware live mining status is blocked by default until a later controlled runtime/harness pack exists. A Phase 21 live smoke or bounded soak wrapper must treat missing enablement as a blocker.

Required enablement pack markers:

- `controlled_live_mining_package_status: ready`
- `controlled_runtime_harness_status: ready`
- `firmware_live_mining_status: blocked_by_default` must be superseded only by an explicit controlled runtime/harness artifact.
- `safe_stop_status` and post-action safe-state markers must be present before any promotion.

## Prohibited Evidence Paths

The following command families are outside this contract and must produce blocked evidence if encountered:

- `erase-flash`
- `rollback`
- `interrupted-update`
- `raw-bm1366`
- `voltage-control`
- `fan-control`
- unbounded stress or soak commands
- network discovery for a device target
- unreviewed raw pool or private endpoint output

## Minimum Final Checklist

Final Phase 21 summary and checklist updates must name the exact evidence pack for each promoted row, the exact board `205` and source/reference commits, redaction status, outcome status, and non-claims. Any row still relying on readiness-only, blocked, startup-only, stale, or no-target evidence must remain below `verified`.

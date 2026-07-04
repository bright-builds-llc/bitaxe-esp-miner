# Phase 21 Final Evidence Summary

phase21_status: passed
phase21_evidence_closure: approved_controlled_no_share_soak
controlled_runtime_harness_status: observed
controlled_runtime_harness_observation_status: observed in live smoke and bounded soak
redaction_status: passed
evidence_root: docs/parity/evidence/phase-21-live-mining-and-soak-evidence
requirements: ASIC-07, STR-06, STR-07, SAFE-09, EVD-05
network_scan: disabled
ASIC-007 frequency transition status: below verified unless a bounded frequency-transition hardware-regression artifact exists in a future evidence tree

## Closure Decision

Phase 21 satisfies the approved controlled no-share soak closure path. The live-smoke pack records `live_mining_smoke_status: controlled-no-share`, pool input bridge application, controlled runtime markers, typed BM1366 work dispatch, bounded no-result/no-share markers, runtime snapshot update, API/WebSocket telemetry readiness, watchdog yield checkpoints, redacted live API/WebSocket captures, and final safe-stop. The bounded-soak pack records `bounded_soak_status: approved_controlled_no_share_soak`, `duration_seconds: 300`, watchdog responsiveness, redacted telemetry, and final safe-stop.

The phase does not claim accepted shares, rejected shares, full production mining, successful live nonce/result parsing, active voltage/fan/fault controls, frequency transition, OTA/recovery behavior, or non-205 board behavior.

## Evidence Matrix

| Pack | Status | Evidence class | Artifact | Checklist rows | Supported subclaims | Below verified subclaims | Non-claims |
|------|--------|----------------|----------|----------------|---------------------|--------------------------|------------|
| preflight | passed | workflow, hardware-smoke | `preflight.md` | WF-005 context, STR-007 context, EVD-05 context | Package release gate passed, detector found board `205`, board-info passed, safe baseline flashed with mining/work/control disabled | Live shares, frequency transition, voltage, fan | Does not authorize network discovery, OTA, erase, rollback, interrupted update, or active control |
| live-mining-enablement | ready-observed | workflow | `live-mining-enablement.md` | STR-006 context, STR-007 context | Controlled live-mining package and runtime harness are build-ready and observed through settings patch/runtime markers | Accepted/rejected shares | Not active hardware-control proof |
| bm1366-init-work-result | complete diagnostic prerequisite | hardware-smoke | `bm1366-init-work-result.md` | ASIC-002, ASIC-003, ASIC-004, ASIC-005, STR-006 context | Trusted package-backed chip-detect and work-result diagnostic commands ran; diagnostic work dispatched; fail-closed timeout recorded | Successful full BM1366 initialization and accepted live nonce parsing | No frequency transition or active voltage/fan proof |
| live-mining-smoke | controlled-no-share | hardware-smoke | `live-mining-smoke.md` | STR-007, STR-008, STAT-002, API-002 context | Pool settings applied and consumed; subscribe/authorize/notify; typed BM1366 work dispatch; bounded no-result/no-share; runtime snapshot/API/WebSocket telemetry; safe-stop | Accepted/rejected shares and unbounded production mining | No raw pool values, raw target, or active hardware controls |
| live-api-websocket-telemetry | passed | hardware-smoke | `live-api-websocket-telemetry.md` | API-002, API-006, STAT-002 context | Explicit-target `/api/system/info` and `/api/ws/live` captures correlate with controlled no-share runtime state | Long-run cadence and production statistics history | No network discovery or stale-target inference |
| bounded-soak | approved controlled no-share soak | soak | `bounded-soak.md` | STR-007, STR-008, SAFE-09 context | 300-second bounded controlled no-share soak, watchdog checkpoints, redacted telemetry, safe-stop | Accepted/rejected shares, active thermal/power sensor parity, active voltage/fan/fault behavior | No unbounded stress or destructive/fault-injection flow |
| watchdog-observations | passed | soak | `bounded-soak/watchdog-observations.md` | SAFE-09 context | Bounded watchdog checkpoints and no unexpected reboot/panic/unsafe/silence markers in committed evidence | Fault recovery and watchdog panic recovery | Startup-only breadcrumbs are not treated as proof |
| parity-redaction | passed | workflow | `redaction-review.md` | EVD-05 context | Deterministic scan reviewed; raw artifacts not committed; committed evidence contains only allowed labels, placeholders, redacted values, command examples, USB port identity, and package/tool metadata | Redaction does not create share proof | No secret-bearing values or raw endpoints |

## Exact Supported Claims

- Board `205` detector and board-info gates passed before Phase 21 hardware evidence tiers.
- The controlled live-mining package booted and the runtime harness was observed after pool settings were applied through `PATCH /api/system`.
- The runtime emitted subscribe, authorize, notify/job, typed BM1366 work dispatch, bounded no-result/no-share, runtime snapshot, API/WebSocket telemetry, watchdog checkpoint, and safe-stop markers.
- The live API and WebSocket captures show controlled no-share telemetry with redacted network and pool-related fields.
- A 300-second approved bounded controlled no-share soak ran with watchdog responsiveness evidence and final safe-stop.
- Final redaction review passed for committed Phase 21 artifacts.

## Below Verified Or Blocked Claims

- Accepted shares and rejected shares were not observed.
- Full production mining and unbounded soak behavior remain below verified.
- Successful live nonce/result parsing remains below verified because the controlled harness recorded bounded no-result/no-share.
- ASIC frequency transition remains below verified because no bounded frequency-transition hardware-regression artifact exists in this evidence tree.
- Active voltage, fan, fault, self-test, load hardware regression, runtime display/input parity, OTA, OTAWWW, rollback, erase, failed-update recovery, interrupted-update, and release-recovery flows remain outside Phase 21 verified claims.

## Redaction And Reference

redaction_review: passed
raw_artifacts_committed: no
reference_clean: passed
network_scan: disabled

The final redaction review covers committed evidence only. It allows labels, schema field names, redacted placeholders, command examples, USB port identity, package/tool metadata, and explicit non-claims. It does not allow raw device URLs, pool credentials, worker secrets, Wi-Fi credentials, API tokens, NVS secret values, private endpoints, or unredacted target data.

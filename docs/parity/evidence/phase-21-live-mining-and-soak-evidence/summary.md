# Phase 21 Final Evidence Summary

phase21_status: blocked
phase21_evidence_closure: blocked_or_below_verified
controlled_runtime_harness_status: ready
controlled_runtime_harness_observation_status: not observed in live smoke or soak
redaction_status: passed
evidence_root: docs/parity/evidence/phase-21-live-mining-and-soak-evidence
requirements: ASIC-07, STR-06, STR-07, SAFE-09, EVD-05
network_scan: disabled
ASIC-007 frequency transition status: below verified unless a bounded frequency-transition hardware-regression artifact exists in this Phase 21 evidence tree

## Closure Decision

Phase 21 does not satisfy either final closure path.

The `live_pool_smoke` closure path is rejected because `live-mining-smoke.md`
records `live_mining_smoke_status: blocked`, `blocker:
missing_live_prerequisites`, `controlled_package_boot_status: not-run`,
`pool_input_bridge_status: not-run - missing_live_prerequisites`,
`share_outcome: not-run`, `hardware_command_status: not-run`, and no actual
live-pool runtime markers.

The `approved_controlled_no_share_soak` closure path is rejected because
`bounded-soak.md` records `bounded_soak_status: blocked`,
`live_smoke_prerequisite: failed`, `share_outcome: not-run`,
`hardware_command_status: not-run`, no actual controlled run or harness
provenance, and `watchdog-observations.md` records
`watchdog_responsiveness_status: blocked - bounded soak not run`.

The controlled runtime and package enablement pack is ready, but readiness alone
does not prove runtime observation. Phase 21 therefore closes as blocked and
below verified for live mining, share, telemetry freshness, watchdog
responsiveness, bounded soak, and successful ASIC initialization claims.

## Evidence Matrix

| Pack | Status | Evidence class | Artifact | Checklist rows | Supported subclaims | Below verified subclaims | Non-claims |
|------|--------|----------------|----------|----------------|---------------------|--------------------------|------------|
| preflight | passed | workflow, hardware-smoke | `preflight.md` | WF-005 context, STR-007 context, EVD-05 context | Package release gate passed, detector found board `205`, board-info passed, safe baseline flashed with mining/work/control disabled | Live mining, shares, soak, live telemetry, frequency transition, voltage, fan | Does not authorize network discovery, live mining, soak, OTA, erase, rollback, interrupted update, or active control |
| live-mining-enablement | ready-not-observed | workflow | `live-mining-enablement.md` | STR-006 context, STR-007 context | Controlled live-mining package and runtime harness are build-ready behind explicit evidence gates | Runtime harness was not observed in live smoke or soak; no pool settings were applied | Not share proof, not live pool proof, not soak proof |
| bm1366-init-work-result | complete diagnostic prerequisite | hardware-smoke | `bm1366-init-work-result.md` | ASIC-002, ASIC-003, ASIC-004, ASIC-005, STR-006 context | Trusted package-backed chip-detect and work-result diagnostic commands ran; diagnostic work dispatched; fail-closed timeout recorded | Successful BM1366 initialization, valid result receive, production work dispatch, accepted/rejected shares | No live pool connectivity, no production mining, no soak, no frequency transition, no active voltage/fan proof |
| live-mining-smoke | blocked | blocked workflow evidence | `live-mining-smoke.md` | STR-007, STR-008, STAT-002, API-002 context | Missing live prerequisites were recorded with detector pass and redaction pass | Live pool lifecycle, subscribe/authorize/notify, controlled package boot, runtime snapshot update, telemetry update, shares, watchdog under load | Not controlled no-share evidence and not live-pool smoke evidence |
| live-api-websocket-telemetry | blocked | blocked workflow evidence | `live-api-websocket-telemetry.md` | API-002, API-006, STAT-002 context | Explicit no-target boundary recorded; network scan disabled | `/api/system/info` freshness, `/api/ws/live` frame capture, cadence, mining statistics correlation | No target was inferred and no API/WebSocket probe was run |
| bounded-soak | blocked | blocked workflow evidence | `bounded-soak.md` | STR-007, STR-008, SAFE-09 context | Intended bounded duration and trust boundary recorded for traceability | Soak stability, approved controlled no-share soak, shares, thermal/power observations, safe-stop from soak | No bounded soak hardware command ran |
| watchdog-observations | blocked | blocked workflow evidence | `bounded-soak/watchdog-observations.md` | SAFE-09 context | Startup-only watchdog breadcrumbs explicitly rejected as soak proof | Watchdog responsiveness under bounded mining or soak load | No bounded observations, no reboot/panic/unsafe-marker check, no safe-stop from soak |
| parity-redaction | passed | workflow | `redaction-review.md` | EVD-05 context | Deterministic scan reviewed; raw artifacts not committed; committed evidence contains only allowed labels, placeholders, redacted values, command examples, USB port identity, and package/tool metadata | Redaction does not promote blocked evidence to verified parity | No secret-bearing values, raw endpoints, raw target URL, pool credentials, worker secret, token, SSID, or NVS secret are cited |

## Exact Supported Claims

- Board `205` detector and board-info gates passed before Phase 21 hardware
  evidence tiers.
- The default safe package release gate passed and the safe baseline booted with
  mining, hardware control, and work submission disabled.
- The controlled live-mining package and runtime harness are ready as a build and
  evidence contract, but not observed during live smoke or soak.
- BM1366 diagnostic chip-detect and work-result commands ran through trusted
  wrappers and produced redacted artifacts.
- BM1366 diagnostic behavior stayed fail-closed: chip detect saw a partial UART
  read, diagnostic work was dispatched, and result handling timed out.
- Missing explicit live target and pool input categories blocked live smoke and
  telemetry correlation without network scanning or target inference.
- Missing live-smoke proof blocked bounded soak and watchdog responsiveness
  evidence.
- Final redaction review passed for committed Phase 21 artifacts.

## Below Verified Or Blocked Claims

- Successful BM1366 initialization remains below verified.
- Production work dispatch and valid result receive remain below verified.
- Live pool lifecycle, subscribe, authorize, notify/job flow, and share
  submission remain below verified.
- Accepted shares and rejected shares were not observed.
- `STR-008` broad live mining smoke and soak evidence remains below verified.
- Runtime statistics, scoreboard, hashrate inputs, API telemetry freshness, and
  WebSocket cadence remain below verified.
- SAFE-09 watchdog responsiveness under bounded mining or soak load remains
  below verified.
- Bounded soak stability and approved controlled no-share soak are blocked.
- ASIC frequency transition remains below verified because no bounded
  frequency-transition hardware-regression artifact exists in this evidence
  tree.
- Active voltage, fan, fault, self-test, load hardware regression, runtime
  display/input parity, OTA, OTAWWW, rollback, erase, failed-update recovery,
  interrupted-update, and release-recovery flows remain outside Phase 21
  verified claims.

## Non-Claims

Phase 21 does not claim non-205 board verification, Stratum v2 parity, BAP
behavior, Angular UI rewrite, active voltage/fan/fault/self-test/load hardware
regression, OTA, OTAWWW, rollback, erase, failed-update recovery,
interrupted-update, release-recovery flows, raw BM1366 writes, raw pool
commands, hidden local scripts, network discovery, or unbounded mining stress.

## Final Blockers

- missing live prerequisites: explicit target and disposable or non-secret pool
  input categories were absent when Plan 21-06 ran.
- controlled runtime harness: build-ready but not observed in live smoke or
  bounded soak.
- controlled package boot: not run for live smoke or bounded soak.
- pool input bridge: not applied.
- share outcome: not run.
- bounded watchdog proof: not available because bounded soak did not run.

## Redaction Sign-Off

Final redaction review passed for committed artifacts. Scan matches were
reviewed as allowed schema/status labels, redacted placeholders, route names,
field names, package/tool-version metadata, USB port identity, non-secret
command examples, ESP capability labels, and explicit non-claims.

raw_artifacts_committed: no

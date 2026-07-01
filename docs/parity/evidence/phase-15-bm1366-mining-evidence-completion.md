# Phase 15 BM1366 Mining Evidence Completion

## Scope

This ledger closes Phase 15 evidence governance for Ultra 205 board `205`
with BM1366 ASIC evidence. It cites only artifacts under
`docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/` that have
passed redaction review.

Phase 15 supports exact diagnostic and controlled-boundary claims only. It does
not support production mining, accepted shares, live pool behavior, full BM1366
initialization, frequency transition behavior, voltage behavior, fan behavior,
live API telemetry, live WebSocket telemetry, statistics producer samples,
release readiness, OTA, rollback, erase, recovery, or interrupted-update
claims.

non-claims: production mining, accepted shares, live pool behavior, full
BM1366 initialization, frequency transition, voltage behavior, fan behavior,
live API/WebSocket telemetry, statistics producer samples, release readiness,
OTA, rollback, erase, recovery, interrupted update.

Requirements covered by this ledger: `ASIC-07`, `STR-06`, `STR-07`,
`SAFE-09`, and `EVD-05`.

## Hardware Gates

Every live Phase 15 evidence pack used the required detector gate before any
hardware interaction.

| Gate | Evidence | Result |
| --- | --- | --- |
| Detector | `just detect-ultra205` | Passed for exactly one likely Ultra 205 port, `port=/dev/cu.usbmodem1101`. |
| Board info | `espflash board-info --chip esp32s3 --port /dev/cu.usbmodem1101 --non-interactive` | Passed for ESP32-S3 in the detector logs. |
| Board | Allow manifests and summaries | Board `205` only. |
| Reference | Allow manifests and package manifests | Reference commit `c1915b0a63bfabebdb95a515cedfee05146c1d50`. |
| Safe state | Chip-detect and work/result serial logs | `mining=disabled`, `hardware_control=disabled`, `work_submission=disabled`, and reset held low on diagnostic failure paths. |
| Restore/safe-stop | Chip-detect and work/result safe-state markers; controlled wrapper safe-stop status | Confirmed or pending only because no active live mining state was entered after missing live prerequisites. |

No raw BM1366 serial writes, direct pool commands, voltage commands, fan
commands, erase commands, rollback commands, interrupted-update commands, or
unbounded stress commands were run for this phase.

## Evidence Pack Matrix

| Pack | Evidence class | Conclusion | Citation boundary |
| --- | --- | --- | --- |
| `bm1366-chip-detect` | `hardware-smoke` | Passed for package-backed chip-detect smoke. | Cites trusted package-backed chip-detect-only, partial-read, no-mining, fail-closed, reset-held-low behavior only. |
| `bm1366-work-result` | `hardware-smoke` | Passed for diagnostic work dispatch with bounded no-result. | Cites one typed diagnostic work dispatch, bounded result timeout, and fail-closed state only. |
| `mining-smoke` | `hardware-smoke` | Controlled no-share condition due to missing live prerequisites. | Cites controlled no-share classification, missing live pool and `DEVICE_URL` blockers, and pending API/WebSocket/watchdog statuses only. |
| `bounded-soak` | `workflow` | Controlled no-share condition; bounded live soak not run. | Cites unsupported-pending bounded-soak contract, `duration_seconds=120`, abort conditions, and missing live prerequisite blocker only. |
| `final-ledger` | `workflow` | Passed for evidence governance after redaction review. | Cites this ledger, redaction review, exact-claim matrix, and below-verified blockers only. |

## Claim Matrix

| Row | Requirement | Evidence state | Boundary |
| --- | --- | --- | --- |
| `ASIC-002` | `ASIC-07` | verified by hardware-smoke for exact subclaim | Package-backed chip-detect evidence proves chip-detect-only, partial UART read, no-mining fail-closed behavior with reset held low. Full BM1366 initialization remains below verified. |
| `ASIC-003` | `ASIC-07` | verified by hardware-smoke for exact subclaim | Work/result evidence proves one typed diagnostic BM1366 work dispatch for job `0x28`. Production work submission and pool-derived work remain below verified. |
| `ASIC-004` | `ASIC-07` | verified by hardware-smoke for exact subclaim | Work/result evidence proves bounded result timeout and fail-closed handling after diagnostic dispatch. Valid nonce/result parsing from live hardware remains below verified. |
| `ASIC-005` | `ASIC-07` | verified by hardware-smoke for exact subclaim | Chip-detect and work/result evidence prove UART adapter reachability and fail-closed serial behavior for diagnostic reads. Accepted serial transport under mining load remains below verified. |
| `ASIC-007` | `ASIC-07` | hardware evidence pending - owner: future frequency-transition hardware regression; blocker: no bounded frequency transition artifact exists in Phase 15 | No Phase 15 artifact exercises BM1366 frequency transitions, voltage, fan, thermal, or recovery behavior. |
| `STR-006` | `STR-06`, `SAFE-09` | verified by hardware-smoke for exact subclaim | Work/result and controlled smoke evidence prove the protocol coordinator remains fail-closed with work submission disabled and records controlled no-share status when live prerequisites are absent. Live pool coordination remains below verified. |
| `STR-007` | `STR-07`, `SAFE-09`, `EVD-05` | verified by hardware-smoke for exact subclaim | Mining-smoke evidence proves controlled no-share criteria, missing live prerequisite classification, safe-stop status, board, port, commits, redaction, and conclusion. Accepted/rejected share behavior remains below verified. |
| `STR-008` | `STR-07`, `SAFE-09`, `EVD-05` | implemented with evidence below verified - owner: future live smoke or approved controlled no-share soak; blocker: live pool and explicit `DEVICE_URL` prerequisites were missing | The bounded-soak pack records an unsupported-pending contract and a controlled no-share blocker, not a live bounded soak or accepted/rejected share result. |
| `API-002` | `STR-07`, `EVD-05` | hardware evidence pending - owner: future explicit `DEVICE_URL` API capture; blocker: no live `/api/system/info` response artifact exists in Phase 15 | Firmware route shell appears in serial logs, but no live HTTP response was captured or redaction-reviewed. |
| `API-006` | `STR-07`, `EVD-05` | hardware evidence pending - owner: future explicit `DEVICE_URL` WebSocket capture; blocker: no live WebSocket frame artifact exists in Phase 15 | The WebSocket helper was not run against a device URL, so cadence and frame contents remain below verified. |
| `STAT-002` | `STR-07`, `EVD-05` | hardware evidence pending - owner: future live API/WebSocket statistics capture; blocker: no statistics producer sample artifact exists in Phase 15 | Pure/API compare coverage exists from earlier phases, but Phase 15 has no live statistics sample. |

## Redaction Review

Redaction review status: passed for all cited Phase 15 artifacts.

Reviewed packs:

- `bm1366-chip-detect`
- `bm1366-work-result`
- `mining-smoke`
- `bounded-soak`
- `final-ledger`
- `parity-redaction`

Secret-pattern scan command:

```bash
rg -n -i "ssid|wifi|password|pool|token|device_url|nvs|stratum|https?://|([0-9]{1,3}\.){3}[0-9]{1,3}|([[:xdigit:]]{2}:){5}[[:xdigit:]]{2}|secret" docs/parity/evidence/phase-15-bm1366-mining-evidence-completion
```

The scan returned expected non-secret category labels, missing-prerequisite
labels, firmware platform strings, local wrapper paths, commit IDs, the USB
port, and the bench MAC address retained as evidence metadata. No pool
credentials, worker secrets, Wi-Fi credentials, private endpoints, private
`DEVICE_URL` values, API tokens, NVS secret values, local terminal secrets, or
pasted raw secrets were found.

No API response bodies or WebSocket captures exist in this phase. They are not
cited for any checklist promotion.

## Exact Claims Supported

- Package-backed Ultra 205 BM1366 chip-detect diagnostic reached a trusted
  wrapper path, observed partial UART read failure, disabled mining/work
  submission/hardware control, and held reset low.
- Package-backed typed work/result diagnostic dispatched one BM1366 diagnostic
  work frame, observed a bounded result timeout, and remained fail-closed with
  work submission disabled.
- Controlled mining-smoke wrapper classified the run as controlled no-share
  because `DEVICE_URL` and live pool prerequisites were missing.
- Bounded-soak wrapper recorded `duration_seconds=120`, abort conditions, and
  unsupported-pending status because live prerequisites were missing.
- Redaction review cleared all cited artifacts and did not clear absent
  API/WebSocket/live-pool artifacts.

## Claims Remaining Below Verified

- Full BM1366 initialization and sustained ASIC runtime behavior.
- BM1366 frequency transitions, voltage behavior, fan behavior, thermal
  response, power behavior, and active-control safety behavior.
- Production BM1366 work submission, valid hardware result/nonce parsing,
  accepted shares, rejected shares, reconnect, fallback pool behavior, and live
  pool lifecycle.
- Live `/api/system/info`, live WebSocket telemetry, and live statistics
  producer samples.
- Live bounded mining soak and final safe-stop from an active mining state.
- Release-readiness, OTA, recovery, rollback, erase, failed-update, and
  interrupted-update behavior.

## Residual Risks

- The controlled no-share evidence is useful for evidence governance and safe
  blocking, but it is not live mining evidence.
- Safe-stop status is confirmed from already-safe diagnostic states and remains
  pending for any future active mining state.
- API, WebSocket, and statistics claims require an explicit `DEVICE_URL` and
  redaction-cleared captures before checklist promotion.
- Frequency, voltage, fan, thermal, and active ASIC-control claims require a
  separate phase-gated hardware regression with recovery instructions.

## Final Verification

Final Phase 15 verification is recorded in
`.planning/phases/15-bm1366-mining-evidence-completion/15-VERIFICATION.md`.
This ledger must not be used to set that verification status to `passed` unless
the targeted checks, Rust pre-commit checks, `just test`, `just parity`,
`just verify-reference`, reference cleanliness, redaction review, and lifecycle
validation all pass. Optional blockers are limited to missing live pool
prerequisites and missing explicit `DEVICE_URL`.

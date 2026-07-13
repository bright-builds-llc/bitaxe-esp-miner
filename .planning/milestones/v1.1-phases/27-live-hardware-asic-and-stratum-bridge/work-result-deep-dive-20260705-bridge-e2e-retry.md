# Bridge E2E Retry (explicit factory image)

Date: 2026-07-05  
Fix: evidence wrapper now flashes `image=` from manifest `factory_merged_image` (not stale manifest-only path)  
Firmware: Phase 27 package @ `0bc4bb3` with `initialized_no_mining_gate`

## Outcome: `blocked_safe_prerequisite` (progress vs first attempt)

| Stage | First E2E | Retry |
| --- | --- | --- |
| Phase 27 ack gate | FAIL (`hardware_evidence_ack_missing`) | **PASS** |
| ASIC init + W5 bootstrap | Not reached | **PASS** — `asic_production_status=initialized` |
| Pool settings consumed | Not reached | FAIL — `waiting_for_pool_settings` |
| Stratum bridge | Not reached | FAIL — `thermal_reading_invalid` prerequisite |
| Share outcome | Not observed | Not observed |

## Blockers for full E2E

1. **Pool settings path** — pool-input-bridge patch did not observe `phase21_pool_settings_consumed=true` in device logs before bridge stopped.
2. **Safety preconditions** — `phase25_prerequisite_status=thermal_reading_invalid` blocked Stratum runtime before pool wait completed.

## Evidence

`live-capture-runtime/flash-monitor.log` (redacted), 360s capture.

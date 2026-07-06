# Phase 27 Bridge Blocker Fix Evidence (2026-07-06)

Board: Ultra 205  
Port: `/dev/cu.usbmodem1101`  
Evidence root: [`bridge-blocker-fix-20260706/`](bridge-blocker-fix-20260706/)  
Investigation: `initialized_no_mining_gate`

## Blocker fix verification

| Blocker | Before | After |
| --- | --- | --- |
| B1 Thermal | `thermal_reading_invalid` blocked bridge | `safety_thermal_status=fault category=thermal_reading_invalid` at bring-up; bounded thermal allows bridge |
| B2 Pool marker | `pool_settings_consumed_marker_missing` | `pool_settings_consumed_by_runtime=true` |
| B3 UART | N/A (bridge blocked) | Bridge active; `work_dispatched` then `production_result_timeout` |

## Bridge markers (redacted logs)

- `phase25_live_stratum_status=connecting`
- `phase25_live_stratum_status=subscribed`
- `phase25_live_stratum_status=authorized`
- `phase25_live_stratum_status=active`
- `phase21_pool_settings_consumed=true source=settings_patch`
- `asic_production_status=work_dispatched`

## Share outcome

`blocked_safe_prerequisite` — Stratum bridge and work dispatch observed; no ASIC result correlation / share proof.

## Code changes (summary)

- `from_thermal_observation`: Fresh status always maps to `Fresh` prerequisite
- Phase 27 thermal: bounded fallback when bring-up observation faulted
- Pool consumed marker on settings patch + before bridge gate
- W9: `post_max_baud_delay_2000` investigation flag

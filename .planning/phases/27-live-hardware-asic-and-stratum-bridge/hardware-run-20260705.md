# Phase 27 Hardware Run — 2026-07-05

## Detector

- `just detect-ultra205`: passed (`port=/dev/cu.usbmodem1101`, ESP32-S3 board-info OK)
- Local `wifi-credentials.json` and `pool-credentials.json`: present (existence only)

## Retry 1 (after board reset)

- Firmware booted fully; pool bridge PATCH HTTP 200 but `pool_settings_consumed_marker_missing`
- Root cause: live stratum required `host:port` inside `stratumurl`; NVS stores URL and `stratumport` separately
- Outcome: `phase27_pool_wait_timeout` → safe-stop complete, no share

## Fix applied

- `pool_settings_from_snapshot` now falls back to `stratumport` when URL has no embedded port
- Rebuilt Phase 27 enablement firmware (`just phase27-package` / Bazel `action_env`)

## Retry 2 (fixed firmware)

- Command: `./scripts/phase27-live-hardware-bridge-evidence.sh --mode hardware --duration-seconds 180 --redact-evidence=true`
- Pool bridge: `pool_input_bridge_status=applied`, `pool_settings_consumed_by_runtime=true`
- Stratum: `connecting` → `subscribed`
- ASIC bridge: `asic_production_status=fail_closed reason=production_asic_init_failed`
- Safe stop: `phase25_safe_stop_status=complete`
- Share outcome: `blocked_safe_prerequisite` (no accepted/rejected markers)
- `phase27_evidence_status=blocked_safe_prerequisite`

## Promotion

- Committed `docs/parity/evidence/phase-27-live-hardware-asic-and-stratum-bridge/` unchanged per plan (no accepted/rejected share)
- `bazel test //scripts:phase27_live_hardware_bridge_evidence_test`: pass
- `just parity`, `just verify-reference`: pass

## Next blocker for accepted/rejected share proof

- Investigate `production_asic_init_failed` on Ultra 205 during Phase 27 live bridge (BM1366 production init path after Stratum subscribe)

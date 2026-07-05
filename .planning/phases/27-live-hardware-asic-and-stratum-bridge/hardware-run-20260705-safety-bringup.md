# Phase 27 Safety Bring-Up Hardware Run (2026-07-05)

board: 205
port: /dev/cu.usbmodem1101
source_commit: 92e838ac9ef1e6fb7c343883388e363ca05438f3
reference_commit: c1915b0a63bfabebdb95a515cedfee05146c1d50
evidence_root: hardware-run-20260705-safety-bringup-retry4
duration_seconds: 360
pool_config: local-owner-supplied
wifi_config: local-owner-supplied

## Commands

```bash
just detect-ultra205
./scripts/phase27-live-hardware-bridge-package.sh
./scripts/phase27-live-hardware-bridge-evidence.sh \
  --evidence-root .planning/phases/27-live-hardware-asic-and-stratum-bridge/hardware-run-20260705-safety-bringup-retry4 \
  --manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json \
  --mode hardware \
  --duration-seconds 360 \
  --port /dev/cu.usbmodem1101 \
  --pool-credentials pool-credentials.json \
  --wifi-credentials wifi-credentials.json \
  --redact-evidence=true
```

## Observed boot sequence (retry4)

- `phase27_safety_bring_up=started` → `complete`
- `asic_status=hold_reset_low gpio=1` before power/voltage/fan actuation
- `safety_voltage_effect=write setpoint_v=1.200`
- `safety_fan_effect=write percent=70`
- `safety_power_status=observed`, `safety_thermal_status=observed`, `safety_fan_status=startup_duty`
- `asic_reset_status=post_bring_up_pulse`
- `display_status=deferred reason=phase27_safety_i2c0_in_use`
- ASIC boot gate still fails: `chip_detect_adapter_error` — partial BM1366 UART read 9/11

## Outcome

- `share_outcome`: blocked_safe_prerequisite
- `asic_bridge_status`: blocked
- `safety_bring_up_status`: complete
- Committed parity evidence: **not promoted** (no accepted/rejected share)

## Notes

- Retry1 I2C failure was caused by `400_000.kHz()` baud misconfiguration; fixed to `400.kHz()`.
- Retry3 used default (non-Phase-27) firmware because flash-monitor rebuilt without Phase 27 `action_env`; always run `phase27-live-hardware-bridge-package.sh` before hardware evidence.
- Chip-detect UART partial read remains the Phase 27 blocker for share proof; likely needs UART timing / upstream `full_init` escalation follow-up.

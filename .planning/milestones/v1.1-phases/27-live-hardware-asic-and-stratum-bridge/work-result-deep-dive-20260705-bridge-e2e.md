# Work-Result Deep Dive Wave 5 — Bridge E2E

Date: 2026-07-05  
Duration: 360s (redacted evidence)  
Firmware investigation: `initialized_no_mining_gate` (W5 bootstrap enabled for ASIC gate)  
Pool config: local-owner-supplied (not committed)  
Wi-Fi config: local-owner-supplied (not committed)

## Command

```bash
./scripts/phase27-live-hardware-bridge-evidence.sh \
  --evidence-root .planning/phases/27-live-hardware-asic-and-stratum-bridge/work-result-deep-dive-20260705-bridge-e2e \
  --manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json \
  --mode hardware \
  --pool-credentials pool-credentials.json \
  --wifi-credentials wifi-credentials.json \
  --duration-seconds 360 \
  --port /dev/cu.usbmodem1101 \
  --redact-evidence=true
```

## Outcome: `blocked_safe_prerequisite`

| Stage | Status |
| --- | --- |
| Detector / board-info | Passed |
| Phase 27 ASIC path | **Blocked** — `hardware_evidence_ack_missing` in flash-monitor (manifest flash did not match Phase 27 enablement ack) |
| Pool input bridge | **Blocked** — `pool_settings_consumed_marker_missing` |
| Share outcome | **Not observed** |

## Conclusion

360s redacted evidence captured per Wave 5 protocol. Live bridge remains blocked: manifest-based flash in the evidence wrapper did not satisfy the Phase 27 hardware-evidence ack gate, so ASIC init never ran. Pool bridge helper also blocked on missing pool-settings marker. Next step: flash via `phase27-live-hardware-bridge-package.sh` image (not stale manifest) before rerunning bridge evidence.

Artifacts: `summary.md`, `conclusion.md`, `live-capture-runtime/flash-monitor.log` (redacted), `mining-allow.json`.

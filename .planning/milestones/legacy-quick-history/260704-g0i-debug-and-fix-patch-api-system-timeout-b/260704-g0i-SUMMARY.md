---
quick_id: 260704-g0i
status: complete
completed_at: "2026-07-04T16:54:00Z"
---

# Quick Task 260704-g0i Summary

## Result

Fixed the Phase 21 `PATCH /api/system` pool-input bridge blocker.

- `PATCH /api/system` still returns the upstream-compatible empty success body.
- Settings effects now run in a bounded post-response task so ESP-IDF can complete the HTTP response first.
- Settings PATCH and Phase 21 controlled-runtime markers are mirrored into retained `/api/system/logs`.
- A new redacted marker, `phase21_pool_settings_consumed=true source=settings_patch`, is emitted only after settings-derived pool config reaches the controlled runtime.
- Added `scripts/phase21-pool-input-bridge.sh` and integrated it into the Phase 21 live evidence wrapper before live API/WebSocket probes.

## Verification

- `node --check scripts/phase21-pool-credentials-json.mjs`
- `bash -n scripts/phase21-pool-input-bridge.sh scripts/phase21-pool-input-bridge-test.sh scripts/phase21-live-mining-evidence.sh scripts/phase21-live-mining-evidence-test.sh`
- `bash scripts/phase21-pool-input-bridge-test.sh`
- `bash scripts/phase21-live-mining-evidence-test.sh`
- `bazel test //scripts:phase21_pool_input_bridge_test //scripts:phase21_live_mining_evidence_test`
- `cargo test -p bitaxe-stratum --all-features controlled_runtime`
- `cargo test -p bitaxe-api --all-features settings`
- `cargo test -p bitaxe-config --all-features settings`
- `cargo check -p bitaxe-firmware --target xtensa-esp32s3-espidf`
- `cargo fmt --all`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo build --all-targets --all-features`
- `cargo test --all-features`

## Hardware Check

Local ignored artifacts under `target/phase21-patch-fix-uat/` record a fresh Ultra 205 run.

- `just detect-ultra205` passed with exactly one board.
- A fresh controlled package built successfully.
- The board booted the current local firmware and registered `/api/system`.
- A same-session monitor log yielded exactly one origin-only `DEVICE_URL` candidate.
- `pool-credentials.json` was present, ignored, and used only as local runtime input.
- The bridge helper returned `pool_input_bridge_status=applied`, `pool_patch_http_status=200`, `pool_patch_curl_status=0`, and `pool_settings_consumed_by_runtime=true`.
- Follow-up `/api/system/info` and `/api/system/logs` both returned HTTP 200; retained logs showed accepted, persisted, response-scheduled, effects-applied, runtime-ready, and pool-consumed markers.

Raw target, Wi-Fi, and pool values stayed in ignored local artifacts only and were not copied into committed evidence.

## Notes

The flash-monitor wrapper did not exit on its configured capture timeout and was interrupted after trusted boot output had been captured. This did not block the targeted PATCH verification because the API route was reachable and the bridge proof came from same-session logs and bounded curl probes.

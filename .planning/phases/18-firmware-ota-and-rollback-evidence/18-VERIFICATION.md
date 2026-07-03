---
phase: 18-firmware-ota-and-rollback-evidence
plan: "04"
status: passed
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 18-2026-07-03T14-06-29
lifecycle_validated: true
generated_at: 2026-07-03T16:00:45Z
---

# Phase 18 Final Verification

status: passed

lifecycle_mode: yolo

phase_lifecycle_id: 18-2026-07-03T14-06-29

lifecycle_validated: true

## Evidence Conclusion

Phase 18 final evidence is redaction-reviewed and claim-bounded. Package,
release-gate, detector, factory flash-monitor, target lock, invalid image
rejection, valid upload HTTP response, final summary, release docs, parity
checklist, requirements traceability, and verification documentation passed
Plan 18-04 checks.

Final OTA conclusion:

- Invalid image rejection passed: HTTP 500 with `Write Error`.
- Valid upload response was captured: HTTP 200 with `Firmware update complete,
  rebooting now!`.
- Valid OTA remains below verified because post-OTA `firmware_commit=`,
  `reference_commit=`, and `ota_boot_validation=` markers were not captured.
- Selected next partition, post-reboot identity, boot-validation, rollback,
  destructive rollback, failed-update recovery beyond invalid rejection, large
  erase, interrupted update, OTAWWW, mining, active safety, and soak behavior
  remain below verified or not claimed.

## Automated Verification

| Command | Result | Notes |
| --- | --- | --- |
| `bash -n scripts/phase18-firmware-ota-evidence.sh scripts/phase18-firmware-ota-evidence-test.sh scripts/phase13-firmware-ota-smoke.sh` | passed | Shell syntax check. |
| `bazel test //scripts:phase18_firmware_ota_evidence_test //scripts:phase13_firmware_ota_smoke_test //tools/parity:tests` | passed | Three targets passed; two were cached and `//tools/parity:tests` executed. |
| `just package` | passed | Rebuilt `//firmware/bitaxe:firmware_image` and generated `bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json`. |
| `bazel run //tools/parity:report -- release-gate --manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json` | passed | Output included `release_gate: passed`. |
| `just parity` | passed | Output ended with `validation_errors: none`; `OTA-001`, `REL-001`, `REL-002`, and `REL-003` remained below verified where required. |
| `just verify-reference` | passed | Output included `reference clean: c1915b0a63bfabebdb95a515cedfee05146c1d50`. |
| `git diff -- reference/esp-miner --exit-code` | passed | No reference submodule changes. |
| `git diff --check -- docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence docs/release/ultra-205.md docs/parity/checklist.md .planning/REQUIREMENTS.md` | passed | Pre-verification-file whitespace check passed. |
| `rg -n "phase-18-firmware-ota-and-rollback-evidence|valid OTA|invalid image rejection|boot-validation|rollback|REL-02|REL-08|REL-07|EVD-05" docs/release/ultra-205.md docs/parity/checklist.md .planning/REQUIREMENTS.md` | passed | Required terms were present. |
| `rg -n -i "ssid|wifi|password|pool|token|device_url|nvs|stratum|https?://|wss?://|([0-9]{1,3}\.){3}[0-9]{1,3}|([[:xdigit:]]{2}:){5}[[:xdigit:]]{2}|secret" docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence docs/release/ultra-205.md docs/parity/checklist.md .planning/REQUIREMENTS.md` | passed after inspection | Matches were allowed labels, placeholders, route names, USB port identity, ESP-IDF/Wi-Fi/NVS labels, command examples, version strings, or non-claims. |
| `node /Users/peterryszkiewicz/.codex/get-shit-done/bin/gsd-tools.cjs verify lifecycle 18 --expect-id 18-2026-07-03T14-06-29 --expect-mode yolo --require-plans --require-verification --raw` | passed | Lifecycle ID, mode, plans, and verification artifact validated. |

## Hardware And Network Command Inventory

Plan 18-04 did not run new hardware or network commands. It cites and verifies
the redacted artifacts produced earlier in Phase 18. The hardware/network
commands actually used by those cited artifacts were:

| Command | Evidence | Result |
| --- | --- | --- |
| `just detect-ultra205` | `docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/serial-boot/detect-ultra205.log` and `firmware-ota/post-ota-detect-ultra205.log` | passed for exactly one board `205` USB port. |
| `espflash board-info --chip esp32s3 --port /dev/cu.usbmodem1101 --non-interactive` | detector logs | passed; MAC redacted. |
| `just flash-monitor board=205 port=/dev/cu.usbmodem1101 manifest=bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json wifi-credentials=wifi-credentials.json evidence-dir=docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/serial-boot capture-timeout-seconds=45 redact-evidence=true` | `serial-boot.md`, `serial-boot/flash-command-evidence.json`, `serial-boot/flash-monitor.log` | passed with trusted output and commit-redacted evidence. |
| `scripts/phase18-firmware-ota-evidence.sh --manifest docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/package-release-gate/bitaxe-ultra205-package.json --ota-image bazel-bin/firmware/bitaxe/esp-miner.bin --port /dev/cu.usbmodem1101 --out-dir docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence --target-lock-out docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/target-lock.json --monitor-seconds 45 --use-flash-log-device-url target/phase18-firmware-ota-and-rollback-evidence-dev-raw/flash-command-evidence.json` | `firmware-ota/firmware-ota-smoke.log`, `firmware-ota.md`, `summary.md` | blocked final firmware OTA status because post-OTA markers were missing. |
| `scripts/phase13-monitor-capture.sh --port /dev/cu.usbmodem1101 --out docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/firmware-ota/post-ota-monitor.log --seconds 45 --no-reset` | `firmware-ota/post-ota-monitor.log` | timed out after capture with no required post-OTA identity or boot-validation markers. |

No raw target host, Wi-Fi credential value, pool credential, API token value,
NVS secret value, IP address, MAC address, private endpoint, or terminal secret
is committed by Plan 18-04.

## Redaction Review

`docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/redaction-review.md`
records `redaction_status: passed`. The Phase 18 summary ledger, release docs,
checklist, requirements note, and this verification artifact cite redacted
artifact paths and preserve all unsupported non-claims.

## Remaining Risks

- Valid upload response lacks post-OTA identity markers, so valid OTA and
  boot-validation stay below verified.
- Rollback, destructive rollback, failed-update recovery beyond invalid
  rejection, large erase, interrupted update, and OTAWWW remain future-owned.
- Current final `just package` verifies the repository at the Plan 18-04 Task 1
  evidence commit; the hardware-flashed Phase 18 package evidence remains tied
  to source commit `22d02f8e97928f1ec29360552179380b92582e6a`.

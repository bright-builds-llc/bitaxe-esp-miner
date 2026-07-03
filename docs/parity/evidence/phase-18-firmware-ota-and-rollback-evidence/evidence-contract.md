# Phase 18 Firmware OTA Evidence Contract

## Scope

Phase 18 evidence closes the firmware OTA and rollback or boot-validation gap
for the current Ultra 205 chain. This contract is created before live artifacts
are cited, so every artifact class starts as `pending` or `absent - not cited`.

network_scan: disabled

Board: `205`

Manifest path:
`docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/package-release-gate/bitaxe-ultra205-package.json`

Firmware OTA image: `esp-miner.bin`

Source commit: pending

Reference commit: pending

`esp-miner.bin` checksum: pending

Fixed invalid fixture checksum: pending

`DEVICE_URL` provenance: pending; must come from an explicit origin-only input
or trusted board `205` flash-monitor evidence. Network scans, mDNS, ARP, router
state, and inferred serial-log guesses are out of scope.

## Required Artifact Classes

| Artifact | Initial status | Claim boundary |
| --- | --- | --- |
| `package-release-gate.md` | pending | Package command, release-gate command, source commit, reference commit, manifest path, and artifact checksum ledger. |
| `package-release-gate/bitaxe-ultra205-package.json` | pending | Copied manifest for board `205`; source/reference commits and artifact metadata only. |
| `serial-boot.md` | pending | Detector and flash-monitor identity summary only. |
| `serial-boot/detect-ultra205.log` | pending | Detector output and board-info gate for the selected USB port. |
| `serial-boot/flash-command-evidence.json` | pending | Trusted flash-monitor command metadata, selected port, source commit, reference commit, and commit-redacted status. |
| `serial-boot/flash-monitor.log` | pending | Redacted serial boot, Wi-Fi join, route registration, and boot-validation markers when present. |
| `target-lock.json` | pending | Sanitized target provenance with `network_scan: disabled` and no raw target. |
| `firmware-ota.md` | pending | Firmware OTA ledger separating valid OTA, invalid rejection, boot validation, rollback, and non-claim fields. |
| `firmware-ota/firmware-ota-smoke.log` | pending | Phase 18 wrapper log plus nested `phase13_firmware_ota_smoke` provenance. |
| `firmware-ota/invalid-firmware.bin` | pending | Fixed invalid fixture used only for invalid rejection evidence. |
| `firmware-ota/invalid-firmware-ota.headers.txt` | pending | Selected invalid OTA response headers. |
| `firmware-ota/invalid-firmware-ota.body.txt` | pending | Redacted invalid OTA response body. |
| `firmware-ota/invalid-firmware-ota.curl-error.txt` | pending | Redacted invalid OTA curl error, if any. |
| `firmware-ota/valid-firmware-ota.headers.txt` | pending | Selected valid OTA response headers. |
| `firmware-ota/valid-firmware-ota.body.txt` | pending | Redacted valid OTA response body. |
| `firmware-ota/valid-firmware-ota.curl-error.txt` | pending | Redacted valid OTA curl error, if any. |
| `firmware-ota/post-ota-monitor.log` | pending | Post-OTA serial capture with `firmware_commit=`, `reference_commit=`, and `ota_boot_validation=` markers required for passed evidence. |
| `summary.md` | absent - not cited | Final Phase 18 evidence ledger after live artifacts and redaction review exist. |
| `redaction-review.md` | pending | Redaction gate for every cited Phase 18 artifact. |

## Required Evidence Fields

| Field | Required value or source | Initial status |
| --- | --- | --- |
| `network_scan` | `disabled` | pending |
| `board` | `205` | pending |
| `source_commit` | Current package manifest source commit | pending |
| `reference_commit` | Current package manifest reference commit | pending |
| `manifest` | Phase 18 copied package manifest path | pending |
| `firmware_ota_image` | Manifest-listed `esp-miner.bin` | pending |
| `firmware_ota_sha256` | Manifest checksum matching the uploaded image | pending |
| `invalid_fixture_sha256` | Fixed invalid fixture checksum from the helper run | pending |
| `device_url_source` | Direct explicit origin or trusted USB flash-monitor evidence | pending |
| `device_url_redacted` | Scheme plus `[redacted]`, never raw host | pending |
| `selected_port` | Detector-approved Ultra 205 USB port | pending |

## Claim Classes

| Claim class | Required proof before citation | Initial status |
| --- | --- | --- |
| valid OTA | Manifest-listed `esp-miner.bin` upload, accepted HTTP response, reboot behavior, post-reboot identity, and `ota_boot_validation=` marker. | pending |
| invalid rejection | Fixed invalid fixture upload, non-200 response, OTA validation or activation marker, and explicit text that invalid image rejection is not rollback proof. | pending |
| boot validation | Post-OTA monitor evidence with `firmware_commit=`, `reference_commit=`, and `ota_boot_validation=`. | pending |
| rollback | Captured bootloader or ESP-IDF rollback state before and after a valid OTA or a separately gated rollback/fault procedure. | absent - not cited |
| non-claim | Destructive rollback, interrupted update, large erase, forced boot failure, and OTAWWW whole-`www` update behavior unless a later plan records gated evidence. | pending |

## Promotion Rules

- Invalid rejection is failed-update evidence only; invalid rejection is not
  rollback proof.
- valid OTA cannot pass without `firmware_commit=`, `reference_commit=`, and
  `ota_boot_validation=` in `firmware-ota/post-ota-monitor.log`.
- boot validation may support non-destructive rollback-related evidence only at
  the exact tier supported by the captured markers.
- rollback remains a non-claim unless an active plan documents recovery
  commands, allow flags, stop conditions, restore path, and safe-state markers.
- `redaction-review.md` must remain `redaction_status: pending` until every
  cited artifact is scanned and reviewed.

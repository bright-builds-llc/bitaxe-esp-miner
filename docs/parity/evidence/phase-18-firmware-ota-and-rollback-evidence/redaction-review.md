# Phase 18 Redaction Review

redaction_status: passed

## Review Scope

This review covers every committed Phase 18 evidence artifact present after
package, detector, flash-monitor, target-lock, invalid-firmware, valid-upload,
and post-OTA monitor capture. It also covers the final Phase 18 summary ledger
created by Plan 18-04. Release documentation, checklist, requirements, and
verification artifacts are updated only after this gate passes and are scanned
again before commit.

## Required Checklist

- [x] `DEVICE_URL` values are redacted or absent from committed artifacts.
- [x] Private endpoints are redacted or absent from committed artifacts.
- [x] IP addresses are redacted or absent from committed artifacts.
- [x] MAC addresses are redacted or absent from committed artifacts.
- [x] SSIDs are redacted or absent from committed artifacts.
- [x] Wi-Fi credentials are redacted or absent from committed artifacts.
- [x] Pool credentials are redacted or absent from committed artifacts.
- [x] Worker credentials are redacted or absent from committed artifacts.
- [x] API tokens are redacted or absent from committed artifacts.
- [x] NVS secret values are redacted or absent from committed artifacts.
- [x] Request and response bodies are redacted before citation.
- [x] Serial logs are redacted before citation.
- [x] Recovery logs are absent because no recovery command ran.
- [x] Local developer-raw evidence under `target/` is not committed.

## Artifact Matrix

| Artifact class | Present? | Reviewed? | Notes |
| --- | --- | --- | --- |
| `package-release-gate.md` | present | passed | Contains package/release-gate labels, checksums, source/reference commits, and command examples only. |
| `package-release-gate/bitaxe-ultra205-package.json` | present | passed | Contains manifest metadata and artifact checksums. Version strings such as `1.88.0.0` are not IP addresses. |
| `package-release-gate/package-command.log` | present | passed | Contains package command transcript without target, credential, or network values. |
| `package-release-gate/release-gate.log` | present | passed | Contains release-gate transcript without target, credential, or network values. |
| `serial-boot.md` | present | passed | Contains allowed USB port identity, command examples, redacted Wi-Fi status, and explicit non-claims. |
| `serial-boot/detect-ultra205.log` | present | passed | Contains detector and board-info output; MAC is redacted. |
| `serial-boot/flash-command-evidence.json` | present | passed | Contains allowed USB port identity, local artifact paths, commit-redacted mode, and NVS seed labels without NVS values. |
| `serial-boot/flash-monitor.log` | present | passed | Contains ESP-IDF/Wi-Fi subsystem labels, redacted SSID, redacted MAC, redacted IP, and redacted device URL. |
| `target-lock.json` | present | passed | Contains sanitized target provenance with `network_scan: disabled`, `device_url_redacted`, and no target host. |
| `firmware-ota.md` | present | passed | Contains claim ledger, HTTP status fields, redacted target provenance, and explicit non-claims. |
| `firmware-ota/firmware-ota-smoke.log` | present | passed | Contains helper transcript with `http://[redacted]`, route names, allowed USB port identity, and claim boundaries. |
| `firmware-ota/invalid-firmware.bin` | present | passed | Fixed invalid binary fixture; no text secrets or network values. |
| `firmware-ota/invalid-firmware-ota.headers.txt` | present | passed | Selected response headers only. |
| `firmware-ota/invalid-firmware-ota.body.txt` | present | passed | Body contains `Write Error` only. |
| `firmware-ota/invalid-firmware-ota.curl-error.txt` | present | passed | Empty curl-error artifact. |
| `firmware-ota/valid-firmware-ota.headers.txt` | present | passed | Selected response headers only. |
| `firmware-ota/valid-firmware-ota.body.txt` | present | passed | Body contains `Firmware update complete, rebooting now!` only. |
| `firmware-ota/valid-firmware-ota.curl-error.txt` | present | passed | Empty curl-error artifact. |
| `firmware-ota/post-ota-monitor.log` | present | passed | Contains bounded monitor command and timeout status; no captured target or credential values. |
| `summary.md` | present | passed | Final Plan 18-04 ledger contains only artifact paths, redacted target provenance, command examples, statuses, and non-claims. |
| `redaction-review.md` | present | passed | This review records the scan result and allowed match classes. |

## Search Command

The required scan was run before changing the status to `passed`:

```bash
phase18_redaction_pattern="ssid|wifi|password|pool|token|device_url|nvs|stratum|https?://|wss?://|([0-9]{1,3}\.){3}[0-9]{1,3}|([[:xdigit:]]{2}:){5}[[:xdigit:]]{2}"
rg -n -i "${phase18_redaction_pattern}|secret" docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence
```

## Reviewed Match Classes

Allowed matches found by the scan:

- Policy labels and redaction checklist terms: `DEVICE_URL`, `Wi-Fi`, `Pool credentials`, `API tokens`, `NVS secret values`, and related review text.
- Redacted placeholders: `http://[redacted]`, `[redacted-ip]`, `[redacted-url]`, `[redacted-ssid]`, and `[redacted-mac]`.
- USB identity and command examples: `/dev/cu.usbmodem1101`, `espflash`, `just flash-monitor`, `just detect-ultra205`, and local build artifact paths.
- ESP-IDF and Wi-Fi subsystem labels: `nvs`, `wifi`, `wifi_init`, `esp_idf_svc::nvs`, `config NVS flash`, and PSRAM heap text.
- Route names and evidence labels: `/api/system/OTA`, invalid image rejection, valid OTA, rollback non-claims, and `network_scan: disabled`.
- Tool version strings that resemble dotted numeric patterns, such as Rust/Cargo `1.88.0.0`.

No committed Phase 18 evidence contains a target host, private endpoint, IP
address, MAC address, SSID value, Wi-Fi credential value, pool credential, worker
credential, API token value, NVS secret value, terminal secret, or raw local
developer target artifact.

## Decision

Reviewer: GSD Phase 18 Plan 04 executor.

Secret scan result: passed - all matches were inspected and are limited to
allowed labels, placeholders, route names, USB port identity, ESP-IDF NVS labels,
command examples without target values, version strings, and explicit non-claim
text.

Conclusion: Phase 18 redaction passed for the committed evidence set. Phase 18
may be cited only with the claim boundaries recorded in `summary.md` and
`firmware-ota.md`: invalid firmware rejection passed, valid upload response was
observed, and valid OTA, post-OTA boot validation, selected partition, rollback,
destructive rollback, OTAWWW, large erase, interrupted update, mining, and active
safety behavior remain below verified or non-claims unless future artifacts prove
them.

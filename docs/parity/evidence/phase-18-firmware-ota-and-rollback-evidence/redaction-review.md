# Phase 18 Redaction Review

redaction_status: pending

## Review Scope

This review starts pending because Plan 18-01 creates the contract before live
firmware OTA artifacts exist. Do not promote Phase 18 valid OTA, invalid
rejection, boot validation, rollback, or release-doc claims until this review
is updated after artifact capture.

## Required Checklist

- [ ] `DEVICE_URL` values are redacted or absent from committed artifacts.
- [ ] Private endpoints are redacted or absent from committed artifacts.
- [ ] IP addresses are redacted or absent from committed artifacts.
- [ ] MAC addresses are redacted or absent from committed artifacts.
- [ ] SSIDs are redacted or absent from committed artifacts.
- [ ] Wi-Fi credentials are redacted or absent from committed artifacts.
- [ ] Pool credentials are redacted or absent from committed artifacts.
- [ ] Worker secrets are redacted or absent from committed artifacts.
- [ ] API tokens are redacted or absent from committed artifacts.
- [ ] NVS secret values are redacted or absent from committed artifacts.
- [ ] Request and response bodies are redacted before citation.
- [ ] Serial logs are redacted before citation.
- [ ] Recovery logs are redacted before citation.
- [ ] Local developer-raw evidence under `target/` is not committed.

## Artifact Matrix

| Artifact class | Present? | Reviewed? | Notes |
| --- | --- | --- | --- |
| `package-release-gate.md` | absent - not cited | pending | Future package and release-gate ledger. |
| `package-release-gate/bitaxe-ultra205-package.json` | absent - not cited | pending | Future copied manifest for board `205`. |
| `serial-boot.md` | absent - not cited | pending | Future detector and flash-monitor ledger. |
| `serial-boot/detect-ultra205.log` | absent - not cited | pending | Future detector output. |
| `serial-boot/flash-command-evidence.json` | absent - not cited | pending | Future commit-redacted flash evidence JSON. |
| `serial-boot/flash-monitor.log` | absent - not cited | pending | Future redacted serial boot evidence. |
| `target-lock.json` | absent - not cited | pending | Future sanitized target lock with `network_scan: disabled`. |
| `firmware-ota.md` | absent - not cited | pending | Future firmware OTA ledger for valid OTA, invalid rejection, boot validation, rollback, and non-claim boundaries. |
| `firmware-ota/firmware-ota-smoke.log` | absent - not cited | pending | Future Phase 18 wrapper and Phase 13 helper log. |
| `firmware-ota/invalid-firmware.bin` | absent - not cited | pending | Future fixed invalid fixture. |
| `firmware-ota/invalid-firmware-ota.headers.txt` | absent - not cited | pending | Future invalid OTA selected headers. |
| `firmware-ota/invalid-firmware-ota.body.txt` | absent - not cited | pending | Future invalid OTA redacted response body. |
| `firmware-ota/invalid-firmware-ota.curl-error.txt` | absent - not cited | pending | Future invalid OTA redacted curl error. |
| `firmware-ota/valid-firmware-ota.headers.txt` | absent - not cited | pending | Future valid OTA selected headers. |
| `firmware-ota/valid-firmware-ota.body.txt` | absent - not cited | pending | Future valid OTA redacted response body. |
| `firmware-ota/valid-firmware-ota.curl-error.txt` | absent - not cited | pending | Future valid OTA redacted curl error. |
| `firmware-ota/post-ota-monitor.log` | absent - not cited | pending | Future post-OTA serial log requiring identity and `ota_boot_validation=` markers. |
| `summary.md` | absent - not cited | pending | Future final Phase 18 evidence summary. |
| `redaction-review.md` | present | pending | This gate remains pending in Plan 18-01. |

## Search Command

Run this scan before changing `redaction_status` to `passed`:

```bash
rg -n -i "ssid|wifi|password|pool|token|device_url|nvs|stratum|https?://|wss?://|([0-9]{1,3}\.){3}[0-9]{1,3}|([[:xdigit:]]{2}:){5}[[:xdigit:]]{2}|secret" docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence
```

## Decision

Reviewer: pending future Phase 18 evidence executor.

Secret scan result: pending.

Conclusion: Phase 18 redaction is not passed yet. Current Plan 18-01 artifacts
may cite only the evidence contract and pending review. Live target, OTA,
serial, recovery, release-doc, checklist, and requirements-promotion claims
remain pending or absent - not cited.

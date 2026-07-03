# Phase 19 Redaction Review

redaction_status: pending

## Review Scope

This review starts before Phase 19 live recovery-regression, OTAWWW, serial, and
target artifacts are cited. It must remain pending until every present committed
artifact is scanned and reviewed, and every absent artifact is marked
`absent - not cited`.

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
- [ ] Raw terminal secrets are redacted or absent from committed artifacts.
- [ ] Request/response bodies are redacted before citation.
- [ ] Serial logs are redacted before citation.
- [ ] Detector logs are reviewed for target and hardware identifiers before citation.
- [ ] Board-info logs are reviewed for hardware identifiers before citation.
- [ ] Recovery logs are reviewed for target, request, response, and command values before citation.
- [ ] OTAWWW headers, bodies, and curl errors are redacted before citation.
- [ ] Local developer-raw evidence under `target/` is not committed.

## Artifact Matrix

| Artifact class | Present? | Reviewed? | Notes |
| --- | --- | --- | --- |
| `package-release-gate.md` | absent - not cited | pending | Created by a later package evidence plan. |
| `package-release-gate/bitaxe-ultra205-package.json` | absent - not cited | pending | Created by a later package evidence plan. |
| `serial-boot.md` | absent - not cited | pending | Created by a later serial evidence plan. |
| `serial-boot/detect-ultra205.log` | absent - not cited | pending | Detector logs may include selected USB port and board-info output. |
| `serial-boot/flash-command-evidence.json` | absent - not cited | pending | Trusted flash-monitor command metadata only after redaction review. |
| `serial-boot/flash-monitor.log` | absent - not cited | pending | Serial logs may include SSIDs, MACs, IPs, and device URLs. |
| `target-lock.json` | absent - not cited | pending | Must contain redacted origin only and `network_scan: disabled`. |
| `recovery-regression.md` | absent - not cited | pending | Created by a later recovery evidence plan. |
| `recovery-regression/recovery-regression.log` | absent - not cited | pending | Helper transcript and claim ledger. |
| `recovery-regression/failed-update.log` | absent - not cited | pending | May include redacted failed-update response snippets. |
| `recovery-regression/large-erase.log` | absent - not cited | pending | May include destructive command transcript and restore evidence. |
| `recovery-regression/large-erase-post-restore-monitor.log` | absent - not cited | pending | Serial safe-state markers only after review. |
| `recovery-regression/interrupted-ota.log` | absent - not cited | pending | May include redacted interruption response snippets. |
| `otawww.md` | absent - not cited | pending | Created by a later OTAWWW evidence plan. |
| `otawww/otawww-gap.log` | absent - not cited | pending | Gap-only response evidence; never whole-www proof by itself. |
| `summary.md` | absent - not cited | pending | Final Phase 19 evidence ledger after live artifacts and review. |
| `redaction-review.md` | present | pending | This file starts the review gate. |

## Required Search Command

Run and inspect this scan before changing `redaction_status` to `passed`:

```bash
phase19_redaction_pattern="ssid|wifi|password|pool|worker|token|device_url|nvs|stratum|secret|https?://|wss?://|([0-9]{1,3}\.){3}[0-9]{1,3}|([[:xdigit:]]{2}:){5}[[:xdigit:]]{2}"
rg -n -i "${phase19_redaction_pattern}" docs/parity/evidence/phase-19-recovery-regression-and-otawww-evidence
```

## Pending Decision

Reviewer: pending.

Secret scan result: pending.

Conclusion: Phase 19 evidence may not be cited as redaction-passed until all
present artifacts are reviewed and all missing artifacts remain explicitly
`absent - not cited`.

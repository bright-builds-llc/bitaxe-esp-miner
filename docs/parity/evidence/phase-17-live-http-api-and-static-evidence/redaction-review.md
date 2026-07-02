# Phase 17 Redaction Review

redaction_status: pending

## Review Scope

- [ ] Private `DEVICE_URL` values redacted or absent.
- [ ] private endpoints redacted or absent.
- [ ] IP addresses redacted or retained only when explicitly needed for board identity.
- [ ] MAC addresses redacted or retained only when explicitly needed for board identity.
- [ ] Wi-Fi credentials redacted or absent.
- [ ] pool credentials redacted or absent.
- [ ] worker secrets redacted or absent.
- [ ] API tokens redacted or absent.
- [ ] NVS secret values redacted or absent.
- [ ] local terminal secrets redacted or absent.
- [ ] absent - not cited.

## Artifact Matrix

| Artifact class | Present? | Reviewed? | Notes |
| --- | --- | --- | --- |
| Package manifest | pending | pending | `package-release-gate/bitaxe-ultra205-package.json`; source/reference commits and artifact checksums only. |
| Package/release-gate logs | pending | pending | `package-release-gate/package-command.log` and `package-release-gate/release-gate.log`; inspect for private paths or terminal secrets before citation. |
| Detector log | pending | pending | `serial-boot/detect-ultra205.log`; retain USB port and board-info identity only when required. |
| Flash evidence JSON | pending | pending | `serial-boot/flash-command-evidence.json`; retain board, selected port, source/reference commits, trusted-output fields, and command identity only. |
| Serial monitor log | pending | pending | `serial-boot/flash-monitor.log`; inspect for Wi-Fi, pool, token, NVS, private endpoint, IP, and MAC leakage before citation. |
| Target lock | pending | pending | `target-lock.json`; must contain sanitized origin only and no raw `DEVICE_URL`. |
| HTTP route log | pending | pending | `http-static-api/http-static-api.log`; cite route statuses, selected headers, redacted snippets, and non-claims only. |
| HTTP headers | pending | pending | `http-static-api/*.headers.txt`; selected headers only, with no cookies, private endpoints, or secrets. |
| HTTP bodies | pending | pending | `http-static-api/*.body.txt`; redacted snippets or allowlisted public markers only. |
| HTTP curl errors | pending | pending | `http-static-api/*.curl-error.txt`; redacted host, URL, IP, and token values only. |
| WebSocket /api/ws/live output | pending | pending | `websocket/api-ws-live.txt`; redacted frame snippets only; required for live frame proof. |
| WebSocket /api/ws output | pending | pending | `websocket/api-ws.txt`; redacted raw-log frame snippets or open-timeout pending status only. |
| Summary ledger | pending | pending | Phase 17 ledger must cite only reviewed artifacts and preserve explicit non-claims. |
| Release docs snippets | pending | pending | Release docs must cite exact commands/artifacts without exposing targets or secrets. |
| Terminal snippets | pending | pending | Terminal snippets must not include private endpoints, credentials, tokens, or local secret values. |
| absent artifacts | pending | pending | Mark every missing body/header/frame/upload/recovery artifact as `absent - not cited`. |

## Search Pattern

Run this scan before changing `redaction_status` to `passed`:

```bash
rg -n -i "ssid|wifi|password|pool|token|device_url|nvs|stratum|https?://|wss?://|([0-9]{1,3}\.){3}[0-9]{1,3}|([[:xdigit:]]{2}:){5}[[:xdigit:]]{2}|secret" docs/parity/evidence/phase-17-live-http-api-and-static-evidence
```

## Decision

Reviewer: pending

Secret scan result: pending.

Conclusion: pending. Do not promote Phase 17 checklist or release documentation
claims until generated artifacts are reviewed and absent artifacts are marked
`absent - not cited`.

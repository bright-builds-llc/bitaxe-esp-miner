---
phase: 17
slug: live-http-api-and-static-evidence
status: verified
threats_open: 0
asvs_level: 1
created: 2026-07-03
updated: 2026-07-03T05:57:49Z
generated_by: gsd-secure-phase
lifecycle_mode: interactive
phase_lifecycle_id: "17-2026-07-02T01-09-48"
---

# Phase 17 - Security

Per-phase security contract for Phase 17 live HTTP, static, API, WebSocket, and flash-time Wi-Fi evidence. The audit scope is the threat model recorded in the Phase 17 plan artifacts; it does not introduce new feature threats beyond those planned mitigations.

## Trust Boundaries

| Boundary | Description | Data Crossing |
|----------|-------------|---------------|
| Operator target input to HTTP/WebSocket helpers | Explicit target input becomes a network destination only after origin-only validation. | Device URL, origin, route path |
| Flash-monitor USB evidence to live target provenance | Trusted wrapper evidence can supply the USB-local target for developer UAT. | Board identity, selected port, trusted-output flag, redacted target lock |
| Live device responses to artifacts | HTTP headers, response snippets, curl errors, and WebSocket frames are reduced before citation. | Route status, selected headers, sanitized bodies, bounded frames |
| Flash-time Wi-Fi credential file to NVS seed | Ignored local credentials may seed NVS over USB, but credential contents must not be read, printed, or committed. | SSID metadata, NVS seed status, hard secrets |
| Evidence ledgers to release/checklist claims | Public docs and parity rows can only promote claims supported by observed artifacts. | Requirement status, exact artifact paths, explicit non-claims |

## Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Evidence | Status |
|-----------|----------|-----------|-------------|---------------------|--------|
| T-17-target | Spoofing/Tampering | HTTP and WebSocket helpers | mitigate | `scripts/phase17-live-http-api-smoke.sh` validates origin-only targets, supports only explicit/environment/USB-log sources, writes `network_scan: disabled`; `scripts/phase17-websocket-capture.mjs` enforces allowlisted paths and explicit source metadata. | closed |
| T-17-identity | Tampering/Repudiation | HTTP helper identity preflight | mitigate | HTTP helper requires manifest and trusted flash evidence before route probes; final UAT has `identity_status: passed`, `target_status: passed`, and a sanitized target lock from `usb_flash_monitor_log`. | closed |
| T-17-redaction | Information Disclosure | Evidence scaffold and snippets | mitigate | HTTP helper runs response artifacts through `redact_stream`; flash tool redacts hard secrets in every mode and redacts network identifiers in commit-redacted mode. | closed |
| T-17-overclaim | Repudiation/Elevation of Privilege | Route helper, WebSocket helper, checklist promotion | mitigate | HTTP probes classify WebSocket no-upgrade as route coexistence, OTA POST as route-presence only, and OTAWWW as fail-closed gap behavior; release docs keep unsupported OTA/mining/soak rows below verified. | closed |
| T-17-02-identity | Spoofing/Tampering | Package manifest and flash JSON | mitigate | Final flash evidence records `command_kind=flash-monitor`, `board=205`, `trusted_output=true`, and source/reference commit provenance; downstream target lock records the same board and source. | closed |
| T-17-02-repudiation | Repudiation | Package and serial summaries | mitigate | Phase 17 summary and UAT artifacts cite exact commands, paths, statuses, and conclusions for package, flash, HTTP, and WebSocket evidence. | closed |
| T-17-02-disclosure | Information Disclosure | Detector and serial artifacts | mitigate | Flash evidence records NVS seed as `provided-redacted`; docs forbid committing credential contents, pool credentials, tokens, API keys, and NVS secret values. | closed |
| T-17-02-overclaim | Repudiation | Package/serial evidence | mitigate | Release docs and evidence summaries explicitly separate package/serial identity from live HTTP/WebSocket/OTA/mining/soak claims. | closed |
| T-17-03-target | Spoofing/Tampering | HTTP helper execution | mitigate | Invalid or absent targets write blocked evidence with no scan fallback; final HTTP UAT used a USB flash-monitor source and recorded `network_scan=disabled`. | closed |
| T-17-03-identity | Tampering/Repudiation | Package and flash evidence preflight | mitigate | HTTP smoke requires package manifest plus trusted flash JSON before live route claims; final run passed the preflight before probing routes. | closed |
| T-17-03-redaction | Information Disclosure | HTTP artifacts and redaction review | mitigate | HTTP artifacts store selected headers and sanitized snippets; final Phase 17 redaction review passed only for reviewed or absent-not-cited artifacts. | closed |
| T-17-03-overclaim | Repudiation/Elevation of Privilege | HTTP route artifacts | mitigate | Route statuses distinguish static/API/recovery behavior, WebSocket no-upgrade, OTA route presence, and OTAWWW fail-closed gap response. | closed |
| T-17-04-target | Spoofing/Tampering | WebSocket helper | mitigate | WebSocket helper accepts only origin-only explicit or trusted flash-evidence-derived targets and records `device_url_source` plus `network_scan=disabled`. | closed |
| T-17-04-dos | Denial of Service | WebSocket capture duration | mitigate | Helper maximums are `30000 ms` and `10` frames; final captures used bounded `5000 ms` and `3` frame limits. | closed |
| T-17-04-disclosure | Information Disclosure | WebSocket output artifacts | mitigate | WebSocket helper redacts frame text before writing artifacts; final committed UAT references only bounded, redacted artifact files. | closed |
| T-17-04-overclaim | Repudiation | WebSocket checklist promotion | mitigate | Final live captures have frame evidence for both `/api/ws/live` and `/api/ws`; weaker no-upgrade checks are not used as frame proof. | closed |
| T-17-05-repudiation | Repudiation | Summary ledger and checklist | mitigate | UAT and release docs cite exact Phase 17 artifact paths for promoted claims and keep blocked/pending states explicit. | closed |
| T-17-05-disclosure | Information Disclosure | Redaction review and release docs | mitigate | Phase 17 summary records redaction scan results and identifies allowed label-only matches; developer-raw target evidence remains under `target/` and is not commit-ready. | closed |
| T-17-05-overclaim | Elevation of Privilege/Repudiation | Docs, checklist, requirements | mitigate | Release docs apply conservative non-claim language for valid OTA, invalid OTA, rollback, boot validation, OTAWWW update, mining, pool behavior, safety telemetry, and long soak. | closed |
| T-17-05-integrity | Tampering | Final verification | mitigate | Phase 17 summary records `just parity` with `validation_errors: none` and `just verify-reference` with the clean reference commit. | closed |
| T-17G-06-01 | Spoofing | `DEVICE_URL` target selection | mitigate | Final HTTP UAT target lock records `device_url_source=usb_flash_monitor_log`, `network_scan=disabled`, `target_status=passed`, and `board=205`; `/api/system/info` was part of the passing route set. | closed |
| T-17G-06-02 | Tampering | Package/flash identity chain | mitigate | Final flash evidence uses trusted wrapper output, board `205`, source commit, reference commit, and NVS seed status before HTTP probes run. | closed |
| T-17G-06-03 | Information Disclosure | HTTP route artifacts | mitigate | Final HTTP log confirms pass status and gzip header behavior without needing raw target values in this security artifact. | closed |
| T-17G-06-04 | Repudiation | Evidence ledger | mitigate | `17-UAT.md` records five passed UAT scenarios, exact final evidence directory, no issues, and no remaining gaps. | closed |
| T-17G-06-05 | Elevation of Privilege | OTA route probe | mitigate | HTTP helper probes empty `/api/system/OTA` only as route presence; release docs explicitly do not claim valid OTA, invalid image rejection, rollback, or boot validation. | closed |
| T-17G-07-01 | Spoofing | WebSocket target provenance | mitigate | Final WebSocket artifacts derive the target from trusted flash evidence and record `device_url_source=usb_flash_monitor_log` with network scanning disabled. | closed |
| T-17G-07-02 | Information Disclosure | WebSocket frame artifacts | mitigate | Final WebSocket artifacts contain bounded, redacted frame summaries and do not store raw URL or credential values in this security record. | closed |
| T-17G-07-03 | Denial of Service | WebSocket capture duration | mitigate | Final `/api/ws/live` and `/api/ws` captures used bounded frame collection and completed with `websocket_frame_status=passed`. | closed |
| T-17G-07-04 | Repudiation | Traceability updates | mitigate | UAT references final HTTP/static/API/WebSocket artifacts and records lifecycle follow-up separately from the live evidence result. | closed |
| T-17G-07-05 | Elevation of Privilege | Checklist/release promotion | mitigate | Release docs and UAT keep OTA, rollback, OTAWWW update, mining, and soak behavior below verified unless later phase-specific artifacts exist. | closed |

## Accepted Risks Log

No accepted risks.

## Security Audit Trail

| Audit Date | Threats Total | Closed | Open | Run By |
|------------|---------------|--------|------|--------|
| 2026-07-03 | 30 | 30 | 0 | Codex / `gsd-secure-phase` |

## Evidence Reviewed

| Evidence | Security Relevance |
|----------|--------------------|
| `.planning/phases/17-live-http-api-and-static-evidence/17-01-PLAN.md` through `17-07-PLAN.md` | Source threat register and mitigation plans. |
| `.planning/phases/17-live-http-api-and-static-evidence/17-01-SUMMARY.md` through `17-05-SUMMARY.md` | Executed plan summaries, threat flags, parity/reference/redaction results, and conservative non-claims. |
| `.planning/phases/17-live-http-api-and-static-evidence/17-UAT.md` | Final live UAT status: 5 passed, 0 issues, no remaining gaps. |
| `scripts/phase17-live-http-api-smoke.sh` | Origin-only target validation, trusted flash-log URL extraction, target lock generation, no-scan metadata, and redacted HTTP artifacts. |
| `scripts/phase17-websocket-capture.mjs` | Origin-only target handling, flash-evidence URL source, allowlisted WebSocket paths, redacted frames, and bounded capture limits. |
| `firmware/bitaxe/src/http_api.rs` | WebSocket upgrade checks, access-denied logging, and frame emission for `/api/ws` and `/api/ws/live`. |
| `tools/flash/src/main.rs` | Wi-Fi credential NVS seeding, trusted flash-monitor classification, redaction modes, hard-secret redaction, and evidence metadata. |
| `docs/release/ultra-205.md` and `AGENTS.md` | Operator rules for local Wi-Fi credentials, developer-raw evidence, commit-ready redaction, and unsupported behavior boundaries. |
| `target/phase17-dev-raw-usb-http-static-ws-final/` | Local developer-raw UAT evidence used to verify live behavior; not commit-ready and not cited with raw network identifiers here. |

## Sign-Off

- [x] All threats have a disposition: mitigate.
- [x] Accepted risks documented in Accepted Risks Log.
- [x] `threats_open: 0` confirmed.
- [x] `status: verified` set in frontmatter.

**Approval:** verified 2026-07-03

## Notes

Developer-raw evidence is useful for USB-local bring-up but is not commit-ready. Shareable or committed evidence must be produced with `redact-evidence=true` or reviewed/redacted before promotion. Wi-Fi passwords, pool credentials, tokens, API keys, and NVS secret values remain hard secrets in all modes.

Long-duration WebSocket stability, production mining, pool behavior, active safety telemetry, destructive update recovery, rollback, boot validation, and OTAWWW update parity remain outside Phase 17 and require later phase-gated evidence.

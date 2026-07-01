---
http_static_status: blocked
device_url_status: blocked - missing DEVICE_URL
network_scan: disabled
source_commit: 8490118a7e7f6fc1a9ac2e4025d983b0f402c8ca
reference_commit: c1915b0a63bfabebdb95a515cedfee05146c1d50
package_manifest: docs/parity/evidence/phase-16-current-commit-release-evidence-completion/package-release-gate/bitaxe-ultra205-package.json
serial_boot_dependency: passed - same source commit as package manifest
root_status: not run - blocked by missing DEVICE_URL
asset_css_gz_status: not run - blocked by missing DEVICE_URL
missing_static_status: not run - blocked by missing DEVICE_URL
recovery_status: not run - blocked by missing DEVICE_URL
api_route_coexistence_status: not run - blocked by missing DEVICE_URL
api_ws_status: not run - blocked by missing DEVICE_URL
api_ws_live_status: not run - blocked by missing DEVICE_URL
ota_route_presence: not run - blocked by missing DEVICE_URL; route-presence only, no valid OTA upload claimed
otawww_rel03_status: deferred - expected Wrong API input gap response was not observed because DEVICE_URL is missing
redaction_status: passed - generated log reviewed; body/header/error artifacts absent - not cited
checklist_promotion_boundary: no live HTTP/static/recovery/API/WebSocket/OTA route row may be promoted to verified from this blocked artifact
conclusion: blocked - live HTTP/static/recovery evidence requires an explicit reachable DEVICE_URL
---

# Phase 16 HTTP/Static/Recovery Evidence

http_static_status: blocked

The Phase 16 HTTP/static/recovery helper ran against the current package
manifest, but no explicit `DEVICE_URL` was present in the execution
environment. The helper correctly produced blocked evidence and did not infer a
target from serial logs, mDNS, ARP, router state, or any other network source.

## Evidence Inputs

| Field | Value |
| --- | --- |
| device_url_status | blocked - missing DEVICE_URL |
| network_scan | disabled |
| source_commit | `8490118a7e7f6fc1a9ac2e4025d983b0f402c8ca` |
| reference_commit | `c1915b0a63bfabebdb95a515cedfee05146c1d50` |
| package_manifest | `docs/parity/evidence/phase-16-current-commit-release-evidence-completion/package-release-gate/bitaxe-ultra205-package.json` |
| serial_boot_dependency | passed - `serial_boot_status: passed` for the same package source commit |
| helper log | `docs/parity/evidence/phase-16-current-commit-release-evidence-completion/http-static-recovery/http-static-smoke.log` |

## Route Results

| Route evidence field | Status |
| --- | --- |
| root_status | not run - blocked by missing DEVICE_URL |
| asset_css_gz_status | not run - blocked by missing DEVICE_URL |
| missing_static_status | not run - blocked by missing DEVICE_URL |
| recovery_status | not run - blocked by missing DEVICE_URL |
| api_route_coexistence_status | not run - blocked by missing DEVICE_URL |
| api_ws_status | not run - blocked by missing DEVICE_URL |
| api_ws_live_status | not run - blocked by missing DEVICE_URL |
| ota_route_presence | not run - blocked by missing DEVICE_URL; route-presence only, no valid OTA upload claimed |
| otawww_rel03_status | deferred - expected `Wrong API input` gap response was not observed because DEVICE_URL is missing |

## Claim Boundary

checklist_promotion_boundary: no live HTTP/static/recovery/API/WebSocket/OTA route row may be promoted to verified from this blocked artifact.

The firmware OTA route remains unproven by this plan because the helper never
reached `POST /api/system/OTA`; even a successful route-presence probe would be
route-presence evidence only, not valid upload evidence. OTAWWW remains the
REL-03 gap until whole-`www` update and interrupted-update hardware-regression
evidence exists.

## Redaction

redaction_status: passed - generated log reviewed; body/header/error artifacts absent - not cited.

Only `http-static-smoke.log` was generated. No `.body.txt`, `.headers.txt`, or
`.curl-error.txt` artifacts exist because the helper blocked before live route
probes. The log contains the manifest path, source/reference commits, blocked
`DEVICE_URL` status, and `network_scan: disabled`; it contains no private
endpoint value, IP address, MAC address, Wi-Fi credential, pool credential, API
token, NVS secret value, or terminal secret.

## Conclusion

conclusion: blocked - live HTTP/static/recovery evidence requires an explicit
reachable `DEVICE_URL`. This blocked artifact is valid evidence that no target
was inferred and no network scan was performed.

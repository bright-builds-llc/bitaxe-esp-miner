# Phase 20 Redaction Review

## Status

redaction_status: pending
reviewer: pending
raw_artifacts_committed: no
phase: 20-active-safety-hardware-telemetry-evidence
source: .planning/phases/20-active-safety-hardware-telemetry-evidence/20-01-PLAN.md

## Scope

This review must pass before any Phase 20 hardware, network, API, WebSocket, or
manual-observation artifact is cited in the parity checklist or final evidence
ledger. Pending status means no Phase 20 artifact is cleared for citation yet.

Raw local artifacts must stay uncommitted unless a later plan defines an
explicit quarantine, review, and redaction procedure. Committed artifacts must be
redacted or allowlisted micro-artifacts.

## Required Checks

| Surface | Status | Required review |
| --- | --- | --- |
| Serial logs | pending | Inspect boot, monitor, flash-monitor, panic, watchdog, and safe-state logs for credentials, local endpoints, SSIDs, MAC/IP values that are not required evidence, pool data, tokens, and secret NVS values. |
| JSON manifests | pending | Inspect package manifests, allow manifests, command evidence, target locks, summaries, and generated JSON for secrets, stale identity, private endpoints, and overbroad raw values. |
| API bodies | pending | Inspect `/api/system/info`, statistics, update, and error bodies for `DEVICE_URL`, private IP addresses, MAC addresses, SSIDs, Wi-Fi credentials, pool credentials, worker secrets, API tokens, and NVS secret values. |
| WebSocket frames | pending | Inspect `/api/ws/live` and `/api/ws` captures for private endpoints, IP addresses, MAC addresses, SSIDs, credentials, tokens, pool values, worker secrets, and terminal paste artifacts. |
| Detector output | pending | Confirm `just detect-ultra205` output is required for board evidence and contains no secret values. |
| Board-info output | pending | Confirm board-info output is required bench evidence and contains no credentials, tokens, pool values, or secret NVS values. |
| Package logs | pending | Inspect build, package, release-gate, flash, and wrapper logs for local paths that expose secrets, private endpoints, credentials, or token values. |
| Command output | pending | Inspect pasted command output, terminal snippets, and wrapper summaries for raw secrets or private endpoints. |
| Manual observations | pending | Inspect notes from physical fan, display, input, thermal, voltage, or fault observations for private network data or secret values. |

## Required Secret And Identifier Terms

These terms and value classes must be searched or manually reviewed before
promotion:

- `DEVICE_URL`
- IP addresses
- MAC addresses
- SSIDs
- Wi-Fi credentials
- pool credentials
- worker secrets
- API tokens
- NVS secret values
- local terminal secrets
- serial log credentials
- JSON secret fields
- API body secrets
- WebSocket frame secrets
- detector output identifiers
- board-info output identifiers
- package log secrets
- command output secrets
- manual observation secrets

## Review Command Template

Run a scoped scan before any Phase 20 citation and review every match:

```bash
rg -n -i "ssid|wi-fi|wifi|password|credential|pool|worker|token|device_url|nvs|stratum|https?://|([0-9]{1,3}\.){3}[0-9]{1,3}|([[:xdigit:]]{2}:){5}[[:xdigit:]]{2}|secret|api[_-]?key" docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence
```

Expected matches in this pending scaffold are policy terms only. Later evidence
must classify each match as redacted, absent, retained required bench evidence,
or blocked.

## Pack Status

| Pack | Redaction status | Citation status |
| --- | --- | --- |
| `safe-baseline` | pending | not cleared |
| `active-power-voltage` | pending | not cleared |
| `active-thermal-fan` | pending | not cleared |
| `self-test-watchdog-load` | pending | not cleared |
| `runtime-display-input` | pending | not cleared |
| `failure-paths` | pending | not cleared |
| `live-api-websocket-telemetry` | pending | not cleared |
| `parity-redaction` | pending | not cleared |

## Conclusion

Conclusion: pending. Phase 20 redaction requirements are defined, but no
hardware, network, API, WebSocket, detector, board-info, command-output, package,
or manual-observation artifact is cleared for citation until this review is
completed by a later plan.

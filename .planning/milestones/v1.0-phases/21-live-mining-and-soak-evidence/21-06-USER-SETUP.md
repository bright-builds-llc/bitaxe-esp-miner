---
phase: 21-live-mining-and-soak-evidence
plan: "06"
status: incomplete
generated_by: gsd-execute-plan
generated_at: 2026-07-04T05:58:34Z
---

# Phase 21 Plan 06 User Setup

Live-pool smoke and explicit API/WebSocket telemetry correlation remain blocked
until an operator supplies non-secret live prerequisites to the executor
environment.

## Required Categories

| Category | Requirement | Handling Rule |
|----------|-------------|---------------|
| `DEVICE_URL` | Explicit origin-only device target | Do not infer from serial logs, mDNS, ARP, router state, scans, or prior evidence. Do not commit the raw value. |
| `BITAXE_POOL_URL` | Disposable or non-secret pool endpoint category | Do not commit, print, or summarize the raw value. |
| `BITAXE_POOL_USER` | Disposable or non-secret worker/user category | Do not commit, print, or summarize the raw value. |
| `BITAXE_POOL_PASSWORD` | Disposable or non-secret password category | Do not commit, print, or summarize the raw value. |

## Safety Contract

- The next live run must still begin with a fresh `just detect-ultra205`.
- Missing any category must remain `missing_live_prerequisites`, not controlled
  no-share evidence.
- Raw target, pool, worker, password, Wi-Fi, API token, NVS secret, endpoint, IP,
  or MAC values must stay out of committed artifacts.
- Evidence intended for commit or sharing must be redacted before promotion.

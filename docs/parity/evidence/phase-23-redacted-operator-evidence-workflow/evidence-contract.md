# Phase 23 Redacted Operator Evidence Contract

This contract defines the committed Phase 23 evidence root:
`docs/parity/evidence/phase-23-redacted-operator-evidence-workflow/`.

Phase 23 proves a repo-owned redacted operator evidence workflow for Ultra 205 board `205`. It does not promote trusted BM1366 production work, does not promote live Stratum socket success, and does not promote accepted/rejected share outcomes.

## Required Slot Inventory

Every committed evidence root must contain these files:

| Slot | File | Allowed status | Purpose |
| --- | --- | --- | --- |
| package | `package.md` | `passed`, `blocked`, `pending`, `deferred` | Package or package-blocked artifact. |
| detector | `detector.md` | `passed`, `blocked`, `pending`, `deferred` | `just detect-ultra205` gate result. |
| board-info | `board-info.md` | `passed`, `blocked`, `pending`, `deferred` | ESP32-S3 board-info gate for board `205`. |
| command | `command.md` | `passed`, `blocked`, `pending`, `deferred` | Repo-owned command category and validator status. |
| log | `log.md` | `passed`, `blocked`, `pending`, `deferred` | Redacted log or blocked log artifact. |
| api | `api.md` | `passed`, `blocked`, `pending`, `deferred` | Redacted API capture or blocked target artifact. |
| websocket | `websocket.md` | `passed`, `blocked`, `pending`, `deferred` | Redacted WebSocket capture or blocked target artifact. |
| share-outcome | `share-outcome.md` | `passed`, `blocked`, `pending`, `deferred` | Share outcome slot with Phase 25 ownership when unobserved. |
| safe-stop | `safe-stop.md` | `passed`, `blocked`, `pending`, `deferred` | Safe-stop workflow or later-runtime status. |
| redaction-review | `redaction-review.md` | `passed`, `blocked`, `pending`, `deferred` | Deterministic redaction review and artifact inventory. |
| conclusion | `conclusion.md` | `passed`, `blocked`, `pending`, `deferred` | Exact Phase 23 claims and non-claims. |

## Required Fields

Each slot file must include these fields or sections:

- `slot: <name>`
- `slot_status: passed|blocked|pending|deferred`
- `board: 205`
- `source_commit`
- `reference_commit`
- `package_identity`
- `detector_evidence`
- `command_category`
- `redaction_status`
- `observed_behavior`
- `safe_stop_status`
- `conclusion`
- `raw_artifacts_committed: no`
- `exact_non_claims`

## Secret Handling

Raw local artifacts may only live under ignored/local paths. Committed evidence contains redacted values or category labels only.

Allowed committed category labels include:

- `pool_config: local-owner-supplied`
- `pool_config: not-supplied`
- `pool_config: not-read`
- `wifi_config: local-owner-supplied`
- `raw_pool_values_committed: no`
- `raw_artifacts_committed: no`

Committed evidence must not contain raw pool endpoints, ports, users, workers, owner addresses, passwords, tokens, targets, extranonces, share payloads, socket errors, device URLs, IP addresses, MAC addresses, Wi-Fi values, NVS secrets, API tokens, raw Stratum payloads, raw share payloads, or raw BM1366 frames.

## Exact Non-Claims

Phase 23 does not promote trusted BM1366 production work.
Phase 23 does not promote live Stratum socket success.
Phase 23 does not promote accepted/rejected share outcomes.
Phase 23 does not promote Phase 26 telemetry closure.
Phase 23 does not promote active voltage, fan, fault, thermal, or self-test closure.
Phase 23 does not promote non-205 boards, OTA/recovery trust, Stratum v2, display/input behavior, BAP, or unbounded stress mining.

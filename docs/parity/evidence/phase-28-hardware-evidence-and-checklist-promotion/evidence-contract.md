# Phase 28 Hardware Evidence And Checklist Promotion Contract

This contract defines the committed Phase 28 evidence root:
`docs/parity/evidence/phase-28-hardware-evidence-and-checklist-promotion/`.

Phase 28 consolidates Phase 27 detector-gated hardware workflow categories into a promotion-ready operator evidence root. It cross-links Phase 27 committed slot files, extends the Phase 23 slot inventory, and preserves blocked share-outcome categories. It does not duplicate raw local artifacts, does not promote accepted/rejected shares, and does not promote STR-09 or CFG-07 to `verified`.

## Required Slot Inventory

Phase 28 inherits the Phase 23 slot inventory:

| Slot | File | Allowed status | Purpose |
| --- | --- | --- | --- |
| package | `package.md` | `passed`, `blocked`, `pending`, `deferred` | Package identity cross-link or blocked category. |
| detector | `detector.md` | `passed`, `blocked`, `pending`, `deferred` | Detector gate cross-linked from Phase 27. |
| board-info | `board-info.md` | `passed`, `blocked`, `pending`, `deferred` | Board-info gate cross-linked from Phase 27. |
| command | `command.md` | `passed`, `blocked`, `pending`, `deferred` | Repo-owned command category cross-linked from Phase 27. |
| log | `log.md` | `passed`, `blocked`, `pending`, `deferred` | Redacted log category or blocked consolidation slot. |
| api | `api.md` | `passed`, `blocked`, `pending`, `deferred` | Redacted API category or blocked consolidation slot. |
| websocket | `websocket.md` | `passed`, `blocked`, `pending`, `deferred` | Redacted WebSocket category or blocked consolidation slot. |
| share-outcome | `share-outcome.md` | `passed`, `blocked`, `pending`, `deferred` | Inherited Phase 27 share-outcome blocker slot. |
| safe-stop | `safe-stop.md` | `passed`, `blocked`, `pending`, `deferred` | Safe-stop category cross-linked from Phase 25/27. |
| redaction-review | `redaction-review.md` | `passed`, `blocked`, `pending`, `deferred` | Deterministic redaction review for Phase 28 root. |
| conclusion | `conclusion.md` | `passed`, `blocked`, `pending`, `deferred` | Exact Phase 28 consolidation claims and non-claims. |

Additional committed summary file:

| File | Purpose |
| --- | --- |
| `summary.md` | Requirement mapping, consolidation status, and exact non-claims for checklist promotion. |

## Phase 28 Extension Fields

Every Phase 28 slot file must include:

- `source_phase27_root: docs/parity/evidence/phase-27-live-hardware-asic-and-stratum-bridge/`
- `consolidation_status: cross_linked|blocked|pending`

Phase 28 slots inherit Phase 27 category tokens where applicable:

- `share_outcome: blocked_safe_prerequisite`
- `asic_bridge_status: blocked`
- `safe_stop_status: blocked`
- `redaction_status: passed`
- `raw_artifacts_committed: no`
- `raw_pool_values_committed: no`

## Secret Handling

Committed evidence contains category labels only. Forbidden committed values include raw pool endpoints, ports, users, workers, owner addresses, passwords, tokens, targets, extranonces, share payloads, socket errors, device URLs, IP addresses, MAC addresses, Wi-Fi values, NVS secrets, API tokens, raw Stratum payloads, raw share payloads, and raw BM1366 frames.

Allowed category labels include `pool_config: local-owner-supplied`, `wifi_config: local-owner-supplied`, `raw_pool_values_committed: no`, and `raw_artifacts_committed: no`.

## Exact Non-Claims

Phase 28 does not promote accepted or rejected shares.
Phase 28 does not promote STR-09 or CFG-07 to `verified`.
Phase 28 does not promote full active voltage, fan, thermal, fault, or self-test safety closure.
Phase 28 does not promote OTAWWW/recovery destructive evidence, non-205 boards, Stratum v2, UI/BAP, or unbounded stress mining.
Phase 28 does not downgrade earlier verified rows from Phase 26 or earlier phases.

# Phase 28 Redaction Review

slot: redaction-review
slot_status: passed
board: 205
source_commit: d497b33c9dcb17c1040d5c67e91eccfa0ec49b8a
reference_commit: c1915b0a63bfabebdb95a515cedfee05146c1d50
package_identity: bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json
evidence_mode: phase28-hardware-evidence-and-checklist-promotion
source_phase27_root: docs/parity/evidence/phase-27-live-hardware-asic-and-stratum-bridge/
consolidation_status: cross_linked
source_reference: ../phase-27-live-hardware-asic-and-stratum-bridge/redaction-review.md
detector_evidence: not-run-consolidation-only
command_category: deterministic-phase28-redaction-review
redaction_status: passed
diagnostic_input_status: no_raw_diagnostic_input
raw_artifacts_committed: no
raw_pool_values_committed: no
pool_config: local-owner-supplied
wifi_config: local-owner-supplied
network_scan: disabled

## Artifact Inventory

evidence-contract.md
package.md
detector.md
board-info.md
command.md
log.md
api.md
websocket.md
share-outcome.md
safe-stop.md
redaction-review.md
conclusion.md
summary.md

## Denylist Categories

Committed evidence must not contain raw pool endpoints, ports, users, workers, owner addresses, passwords, tokens, targets, extranonces, share payloads, socket errors, device URLs, IP addresses, MAC addresses, Wi-Fi values, NVS secrets, API tokens, raw Stratum payloads, raw share payloads, or raw BM1366 frames.

## conclusion

Phase 28 redaction review cross-references the Phase 27 redaction-review baseline. No raw local credential contents or runtime values are committed in the Phase 28 consolidation root.

## exact_non_claims

- accepted/rejected shares remain non-claims unless detector-gated live proof exists with ASIC bridge correlation markers.
- ASIC bridge dispatch proof remains blocked unless category markers are observed in a detector-gated run.
- STR-09 and CFG-07 remain below `verified`.

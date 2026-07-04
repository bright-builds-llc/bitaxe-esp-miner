# Phase 21 Redaction Review

redaction_status: pending
raw_artifacts_committed: no

This scaffold is intentionally pending until Phase 21 live or soak artifacts exist. It defines the deterministic scan and artifact inventory required before any committed or shared evidence can be cited.

## Deterministic Scan

Run this command from the repository root before citing committed Phase 21 evidence:

```bash
rg -n -i "ssid|wifi|password|pool|worker|token|device_url|nvs|stratum|https?://|([0-9]{1,3}\\.){3}[0-9]{1,3}|([[:xdigit:]]{2}:){5}[[:xdigit:]]{2}|secret|credential" docs/parity/evidence/phase-21-live-mining-and-soak-evidence
```

Expected result before final promotion: only schema terms, redacted labels, and this review contract remain. Raw SSIDs, Wi-Fi values, pool credentials, private worker values, private endpoints, raw target URLs, API tokens, NVS secrets, and unredacted IP or MAC addresses are blockers.

## Artifact Inventory

| Evidence pack | Artifact path | Redaction state | Commit/share state | Notes |
|---------------|---------------|-----------------|--------------------|-------|
| `preflight` | `docs/parity/evidence/phase-21-live-mining-and-soak-evidence/preflight/` | passed | raw_artifacts_committed: no | Review completed for package, detector, board-info, and safe-baseline artifacts. Raw detector output stayed under ignored `target/`; committed hardware logs are redacted and contain only board `205`, port label, source commit, reference commit, commands, and conclusions. |
| `live-mining-enablement` | `docs/parity/evidence/phase-21-live-mining-and-soak-evidence/live-mining-enablement/` | pending | raw_artifacts_committed: no | Must contain ready markers without secret-bearing configuration values. |
| `bm1366-init-work-result` | `docs/parity/evidence/phase-21-live-mining-and-soak-evidence/bm1366-init-work-result/` | passed for chip-detect pack | raw_artifacts_committed: no | Plan 21-04 chip-detect artifacts use wrapper redaction, retain only manifest/log metadata, and cite trusted wrapper summaries plus safe-state markers. Generated package binaries remain ignored. |
| `live-mining-smoke` | `docs/parity/evidence/phase-21-live-mining-and-soak-evidence/live-mining-smoke/` | pending | raw_artifacts_committed: no | Must redact pool, worker, target, address, and credential-bearing details. |
| `bounded-soak` | `docs/parity/evidence/phase-21-live-mining-and-soak-evidence/bounded-soak/` | pending | raw_artifacts_committed: no | Must include duration, abort conditions, safe-stop, and conclusion after redaction. |
| `live-api-websocket-telemetry` | `docs/parity/evidence/phase-21-live-mining-and-soak-evidence/live-api-websocket-telemetry/` | pending | raw_artifacts_committed: no | Must cite explicit-target captures only after target values are redacted. |
| `redaction-review` | `docs/parity/evidence/phase-21-live-mining-and-soak-evidence/redaction-review.md` | pending | raw_artifacts_committed: no | This file is the review scaffold and is safe to commit. |
| `final-summary` | `docs/parity/evidence/phase-21-live-mining-and-soak-evidence/summary.md` | pending | raw_artifacts_committed: no | Must cite only redaction-reviewed artifacts. |

## Reviewer Checklist

- [ ] Run the deterministic scan command.
- [ ] Confirm no raw SSIDs, Wi-Fi values, pool credentials, private worker values, private endpoints, raw target URLs, API tokens, NVS secrets, unredacted IP addresses, or unredacted MAC addresses are committed.
- [ ] Confirm raw hardware logs, if any, stay ignored or are replaced by redacted evidence artifacts.
- [ ] Confirm every checklist citation names a redaction-reviewed artifact.
- [ ] Set `redaction_status: passed` only after the above is complete.

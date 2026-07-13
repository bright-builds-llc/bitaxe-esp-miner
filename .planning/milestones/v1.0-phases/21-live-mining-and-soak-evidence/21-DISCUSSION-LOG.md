# Phase 21: Live Mining And Soak Evidence - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-07-04T01:37:38.709Z
**Phase:** 21-live-mining-and-soak-evidence
**Mode:** Yolo
**Areas discussed:** Controlled pool and target gates, Mining evidence ladder, Live mining smoke, Bounded soak, Telemetry and statistics correlation, Checklist and verification

## Controlled Pool And Target Gates

| Option | Description | Selected |
| --- | --- | --- |
| Strict detector plus explicit target | Require `just detect-ultra205`, board-info pass, disposable/non-secret pool config, explicit `DEVICE_URL`, safe-stop, and redaction before live mining evidence. | yes |
| Infer target from local environment | Use serial logs, mDNS, ARP, router state, or redacted evidence to find the device target. | |
| Controlled blocked/no-share fallback | When live prerequisites are missing, write bounded blocked or controlled no-share evidence with non-claims. | yes |

**User's choice:** Yolo selected the strict detector plus explicit target path, with blocked/no-share fallback when prerequisites are absent.
**Notes:** This carries forward AGENTS.md hardware rules, Phase 17 explicit-target policy, Phase 20 live-telemetry blocker policy, and Phase 15 mining redaction rules.

## Mining Evidence Ladder

| Option | Description | Selected |
| --- | --- | --- |
| Reuse and extend Phase 15 ladder | Build on detector/package/safe boot, chip-detect, work/result, mining smoke, bounded soak, and exact promotion. | yes |
| New ad hoc live-mining script | Add a direct live script that bypasses existing allow-manifest and parity guard structure. | |
| Evidence-only conservative closure | Record useful blockers without running higher tiers when prerequisites are missing. | yes |

**User's choice:** Yolo selected reuse/extension of the Phase 15 evidence ladder, with conservative evidence-only closure when tiers cannot run.
**Notes:** Existing `tools/parity/src/mining_allow.rs` already knows live-pool smoke and bounded-soak claim tiers; Phase 21 should preserve or extend those checks rather than weakening them.

## Live Mining Smoke

| Option | Description | Selected |
| --- | --- | --- |
| Short live-pool micro-smoke | Run only after gates pass; record pool lifecycle, work dispatch, result/share behavior, telemetry, watchdog, and safe-stop. | yes |
| Long uncontrolled mining run | Run until shares appear without bounded duration, abort conditions, or explicit safe-stop criteria. | |
| Fake-pool/controlled harness only | Use controlled flow evidence when live credentials or target prerequisites are missing, labelled as non-production. | yes |

**User's choice:** Yolo selected short live-pool micro-smoke as the preferred proof path and controlled harness/no-share evidence as fallback.
**Notes:** Accepted/rejected share outcomes must be exact observed outcomes. No-share evidence can be useful but cannot imply accepted-share proof.

## Bounded Soak

| Option | Description | Selected |
| --- | --- | --- |
| Bounded soak after smoke | Run after live smoke passes, with duration, abort conditions, telemetry snapshots, watchdog observations, and safe-stop. | yes |
| Approved controlled no-share soak | Allow only when the plan explicitly justifies it and blocker language is absent from promoted claims. | yes |
| Unbounded soak | Let mining continue without duration, recovery, redaction, or stop criteria. | |

**User's choice:** Yolo selected bounded soak after live smoke, with approved controlled no-share soak as a carefully labelled fallback.
**Notes:** Current parity guard expects STR-008 verified details to include board, port, firmware/source commit, reference commit, redaction, conclusion, and either share outcome or approved bounded controlled no-share soak.

## Telemetry And Statistics Correlation

| Option | Description | Selected |
| --- | --- | --- |
| Correlate API/WebSocket with run evidence | Use explicit `DEVICE_URL`, bounded `/api/system/info`, bounded `/api/ws/live`, serial/runtime observations, and redaction review. | yes |
| Route-presence-only proof | Treat route registration or no-upgrade WebSocket response as enough. | |
| Skip telemetry if target missing | Record blocked telemetry and keep API/statistics rows below verified when explicit target is unavailable. | yes |

**User's choice:** Yolo selected explicit telemetry correlation and blocked-row handling when the target is unavailable.
**Notes:** Phase 20 already proved that route presence and blocked telemetry artifacts are insufficient for live freshness/cadence claims.

## Checklist And Verification

| Option | Description | Selected |
| --- | --- | --- |
| Exact-claim promotion | Promote only rows proved by the final redaction-reviewed artifact and `just parity`. | yes |
| Broad mining parity promotion | Treat live smoke or no-share artifacts as proving all ASIC/Stratum/statistics rows. | |
| Strict final gate | Require targeted checks, `just test`, `just parity`, `just verify-reference`, reference cleanliness, redaction review, hardware command evidence, and lifecycle validation. | yes |

**User's choice:** Yolo selected exact-claim promotion and strict final verification.
**Notes:** `ASIC-007` requires bounded frequency-transition hardware-regression evidence if promoted; Phase 21 live mining evidence alone should not verify frequency transition behavior.

## the agent's Discretion

- Exact plan count, helper names, evidence pack names, JSON fields, wrapper reuse versus Phase 21-specific wrappers, and checklist wording.
- Whether to extend `tools/parity/src/mining_allow.rs` or add companion validation surfaces, as long as existing detector, package, live-pool, bounded-soak, and safe-state guarantees are preserved.

## Deferred Ideas

- Non-205 mining evidence, Stratum v2, BAP, all-board release matrix, Angular UI rewrite, OTA/recovery fault flows, and active safety-control regression beyond prerequisite checks.

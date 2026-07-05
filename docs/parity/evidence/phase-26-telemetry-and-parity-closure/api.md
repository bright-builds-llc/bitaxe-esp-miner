# Phase 26 API Projection Evidence

slot: api
slot_status: passed
board: 205
source_commit: fa79b06
reference_commit: c1915b0a63bfabebdb95a515cedfee05146c1d50
package_identity: projection-workflow
detector_evidence: not-run-static-evidence-only
board_info_status: not-run-static-evidence-only
command_category: repo-owned-phase26-projection-and-parity-evidence
api_projection_status: passed
system_info_projection: projection-backed
statistics_projection: bounded-sample-marker-or-empty
scoreboard_projection: empty_without_parsed_share_outcome
post_stop_state: safe_blocked
share_outcome: blocked_safe_prerequisite
redaction_status: passed
raw_artifacts_committed: no
raw_pool_values_committed: no
hardware_evidence_status: blocked_or_not_run

## Evidence Basis

- Plan 26-01 added `RuntimeTelemetryProjection` and tests for runtime event folding, bounded sample marker draining, safe-stop reset, stale sequence handling, and current-generation accepted/rejected counter gates.
- Plan 26-02 added `ProjectedApiViews` and tests proving `/api/system/info` derives mining state from the projection, statistics rows require explicit runtime sample markers, scoreboard output stays empty without parsed share outcome material, and live telemetry JSON receives post-stop projection state.
- Plan 26-03 wired firmware producers and consumers so `/api/system/info`, `/api/system/statistics`, `/api/system/scoreboard`, and `/api/ws/live` serialize projection-backed helpers instead of route-local mining counter derivation.

## Requirement Mapping

| Requirement | Evidence |
| --- | --- |
| API-11 | Projection-backed `ApiSnapshot`, bounded statistics samples, empty scoreboard without parsed share outcome material, and safe-blocked post-stop state from Plans 26-01 through 26-03. |
| API-13 | Current-generation submit classification gates accepted and rejected projection counters; request-time reads do not fabricate statistics history or scoreboard rows. |
| EVD-08 | This redacted artifact records exact projection evidence while preserving Phase 25 `share_outcome: blocked_safe_prerequisite`. |

## Verification Commands

```bash
bazel test //crates/bitaxe-stratum:tests
bazel test //crates/bitaxe-api:tests
bazel build //firmware/bitaxe:firmware
```

## exact_non_claims

- Accepted shares remain non-claims.
- Rejected shares remain non-claims.
- Detector-gated hardware live pool response proof remains a non-claim.
- Full active voltage, fan, thermal, fault, self-test, and load safety closure remain non-claims.
- OTA/recovery, non-205 boards, other ASIC families, Stratum v2, display/input, BAP, and unbounded stress remain non-claims.

## Conclusion

The API projection closure is supported by unit, compile, and workflow evidence only. No detector-gated live share artifact exists in Phase 26, so accepted/rejected live-share proof remains blocked-safe-prerequisite and below verified.

# Phase 26 Statistics And Scoreboard Evidence

slot: statistics-scoreboard
slot_status: passed
board: 205
source_commit: eb2458582ed2c8cef529e91fbbf51b8a95883030
reference_commit: c1915b0a63bfabebdb95a515cedfee05146c1d50
package_identity: projection-workflow
detector_evidence: not-run-static-evidence-only
board_info_status: not-run-static-evidence-only
command_category: repo-owned-phase26-projection-and-parity-evidence
statistics_history_status: no_request_time_fabrication
statistics_sample_source: runtime_projection_marker_only
scoreboard_status: empty_without_parsed_share_outcome
counter_gate: current_generation_submit_intent_plus_parsed_response
accepted_rejected_live_share_proof: non_claim_blocked_safe_prerequisite
redaction_status: passed
raw_artifacts_committed: no
raw_pool_values_committed: no
hardware_evidence_status: blocked_or_not_run

## Evidence Basis

- Plan 26-01 introduced runtime sample markers and a one-drain projection contract for bounded statistics samples.
- Plan 26-01 gates accepted and rejected counters on current-generation `SubmitClassification::Accepted` and `SubmitClassification::Rejected` events.
- Plan 26-02 proves statistics rows stay empty without an explicit runtime projection marker, including repeated request-time reads.
- Plan 26-02 and Plan 26-03 preserve an empty scoreboard when no parsed-response-backed and redaction-allowed share outcome material exists.
- Phase 25 share evidence records `share_outcome: blocked_safe_prerequisite`, so accepted/rejected live-share proof remains an exact non-claim.

## Requirement Mapping

| Requirement | Evidence |
| --- | --- |
| API-11 | Statistics and scoreboard surfaces are now projection-owned rather than route-local placeholders or request-time fabrications. |
| API-13 | `no_request_time_fabrication`, `runtime_projection_marker_only`, and `empty_without_parsed_share_outcome` preserve the counter/statistics/scoreboard gate. |
| EVD-08 | This artifact records conservative closure and explicitly blocks live-share overpromotion. |

## Verification Commands

```bash
bazel test //crates/bitaxe-stratum:tests
bazel test //crates/bitaxe-api:tests
bazel build //firmware/bitaxe:firmware
```

## exact_non_claims

- Accepted shares remain non-claims.
- Rejected shares remain non-claims.
- Full active voltage safety closure remains a non-claim.
- OTA/recovery remains a non-claim.
- non-205 board evidence remains a non-claim.
- Stratum v2 remains a non-claim.
- display/input runtime parity remains a non-claim.
- BAP remains a non-claim.
- unbounded stress mining remains a non-claim.

## Conclusion

Statistics closure is limited to bounded projection sample mechanics. Scoreboard closure is limited to the compatible empty response until parsed-response-backed share outcome material exists and passes redaction review.

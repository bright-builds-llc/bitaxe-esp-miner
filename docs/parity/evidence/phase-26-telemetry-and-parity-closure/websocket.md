# Phase 26 WebSocket Projection Evidence

slot: websocket
slot_status: passed
board: 205
source_commit: eb2458582ed2c8cef529e91fbbf51b8a95883030
reference_commit: c1915b0a63bfabebdb95a515cedfee05146c1d50
package_identity: projection-workflow
detector_evidence: not-run-static-evidence-only
board_info_status: not-run-static-evidence-only
command_category: repo-owned-phase26-projection-and-parity-evidence
websocket_projection_status: passed
api_ws_live_full_update: projection-backed
api_ws_live_cadence: 500ms-diff
safe_stop_stale_active_frame: rejected
api_ws_raw_log_stream: compatible-redacted-markers-only
redaction_status: passed
raw_artifacts_committed: no
raw_pool_values_committed: no
hardware_evidence_status: blocked_or_not_run

## Evidence Basis

- Plan 26-02 proved projection-backed live telemetry JSON can be planned through the existing full-on-connect and cadence-diff WebSocket planner.
- Plan 26-02 proved safe-stop projection state does not serialize stale active-mining state into the live telemetry payload.
- Plan 26-03 wired firmware live WebSocket consumers to call projection-backed runtime snapshot helpers before serialization.
- `/api/ws` remains the compatible retained-log route. Phase 26 does not add raw pool, share, target, device, Wi-Fi, NVS, or ASIC-frame values to the log stream.

## Requirement Mapping

| Requirement | Evidence |
| --- | --- |
| API-12 | `/api/ws/live` uses the same projection-backed JSON as the HTTP API and preserves full update plus 500 ms diff cadence semantics. |
| API-13 | WebSocket telemetry reflects safe-stop projection state and cannot independently advance share counters or scoreboard material. |
| EVD-08 | This artifact records WebSocket closure with redaction review and exact non-claims. |

## Verification Commands

```bash
bazel test //crates/bitaxe-api:tests
bazel build //firmware/bitaxe:firmware
```

## exact_non_claims

- Accepted shares remain non-claims.
- Rejected shares remain non-claims.
- Detector-gated live `/api/ws/live` production-share telemetry proof remains a non-claim.
- Full active voltage, OTA/recovery, non-205 boards, Stratum v2, display/input, BAP, and unbounded stress remain non-claims.

## Conclusion

Phase 26 closes WebSocket projection mechanics with unit and firmware compile evidence. It does not claim detector-gated live WebSocket mining-share evidence.

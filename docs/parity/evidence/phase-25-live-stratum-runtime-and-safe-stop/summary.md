# Phase 25 Live Stratum Runtime And Safe Stop Summary

board: 205
source_commit: 75e45c5d82b1d9742d5201c1188dde7f53b08288
reference_commit: c1915b0a63bfabebdb95a515cedfee05146c1d50
package_identity: blocked-safe-prerequisite-workflow
detector_status: not-run-static-blocker
board_info_status: not-run-static-blocker
share_outcome: blocked_safe_prerequisite
safe_stop_status: complete
watchdog_responsiveness_status: blocked
redaction_status: passed
raw_artifacts_committed: no
raw_pool_values_committed: no
network_scan: disabled
pool_config: not-supplied
wifi_config: not-supplied

## Exact Claim

Phase 25 adds a repo-owned evidence workflow, mining-allow validation, and committed blocked-safe-prerequisite evidence for the live Stratum runtime and safe-stop path. The supported share outcome is `share_outcome: blocked_safe_prerequisite`.

## Evidence Links

- `docs/parity/evidence/phase-25-live-stratum-runtime-and-safe-stop/share-outcome.md`
- `docs/parity/evidence/phase-25-live-stratum-runtime-and-safe-stop/safe-stop.md`
- `docs/parity/evidence/phase-25-live-stratum-runtime-and-safe-stop/redaction-review.md`

## Implementation Pointers

- `crates/bitaxe-stratum/src/v1/live_runtime.rs`
- `crates/bitaxe-stratum/src/v1/submit_response.rs`
- `crates/bitaxe-stratum/src/v1/fake_pool.rs`
- `firmware/bitaxe/src/live_stratum_runtime.rs`
- `firmware/bitaxe/src/runtime_snapshot.rs`
- `crates/bitaxe-safety/src/watchdog.rs`
- `scripts/phase25-live-stratum-evidence.sh`
- `tools/parity/src/mining_allow.rs`

## Verification Commands

```bash
bazel test //scripts:phase25_live_stratum_evidence_test //tools/parity:tests
just parity
just verify-reference
node "$HOME/.cursor/get-shit-done/bin/gsd-tools.cjs" verify lifecycle 25 --expect-id 25-2026-07-05T01-55-45 --expect-mode yolo --require-plans
```

## exact_non_claims

- Accepted shares remain non-claims.
- Rejected shares remain non-claims.
- Detector-gated hardware live pool response proof remains a non-claim.
- Hardware-level watchdog responsiveness remains a non-claim.
- Phase 26 API, WebSocket, statistics, and scoreboard projection remains a non-claim.

## Conclusion

STR-08, STR-09, SAFE-12, and SAFE-13 have implemented/workflow closure only. STR-11 has deterministic unit coverage from the Phase 25 fake-pool work. Hardware promotion for accepted/rejected shares and watchdog responsiveness remains blocked until a detector-gated run produces redaction-safe artifacts.

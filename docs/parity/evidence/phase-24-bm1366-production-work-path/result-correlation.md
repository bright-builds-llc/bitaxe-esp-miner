# Phase 24 BM1366 Result Correlation Evidence

board: 205
source_commit: 37588dbc1293751029fb7e4a4cfc77cb42cc5aaa
reference_commit: c1915b0a63bfabebdb95a515cedfee05146c1d50
package_identity: not-produced-plan-24-04
evidence_level: implemented/unit,workflow
raw_artifacts_committed: no
redaction_status: passed

## Scope

Phase 24 proves that parsed BM1366 nonce/result observations must correlate to current-generation active work before a submit intent can exist. It does not claim live pool response classification.

## Correlation Gate Evidence

`crates/bitaxe-stratum/src/v1/production_work.rs` defines `SubmitIntent`, `CorrelationOutcome`, and correlation inputs that include `PoolSessionGeneration`. A BM1366 nonce/result observation is blocked when the active work is missing, stale, duplicated, associated with the wrong session generation, or incompatible with the production target context.

The fail-closed blocker taxonomy is provided by `ProductionAsicBlocker` from `crates/bitaxe-asic/src/bm1366/production.rs`. These blockers render stable labels for evidence and firmware status without raw runtime values.

## Guarded Dispatch Evidence

`crates/bitaxe-stratum/src/v1/mining_loop.rs` owns guarded production dispatch through `ProductionWorkRegistry` and `Bm1366ProductionCommand`. The guarded loop emits production work commands and `SubmitIntent` values rather than diagnostic work commands or direct share submission claims.

## Controlled Runtime Evidence

`crates/bitaxe-stratum/src/v1/controlled_runtime.rs` and `firmware/bitaxe/src/controlled_mining_runtime.rs` consume the controlled runtime production outputs. Firmware status publishing routes through `firmware/bitaxe/src/asic_adapter/status.rs`, which emits production ASIC status labels and preserves the Phase 25 ownership boundary for accepted or rejected pool responses.

Plan 24-03 verification commands:

```bash
bazel test //crates/bitaxe-stratum:tests //crates/bitaxe-asic:tests
bazel test //crates/bitaxe-stratum:tests //crates/bitaxe-asic:tests //crates/bitaxe-safety:tests
bazel build //firmware/bitaxe:firmware
```

## exact_non_claims

- nonzero version-mask and multi-midstate production support remain Phase 24 non-claims
- accepted/rejected share outcomes remain Phase 25-owned non-claims
- live Stratum socket success remains a Phase 25-owned non-claim
- API/WebSocket/statistics/scoreboard promotion remains a Phase 26-owned non-claim

## Conclusion

Phase 24 result-correlation evidence supports `implemented` status with `unit,workflow` evidence for fail-closed code/test proof only. Hardware promotion and live response classification remain outside this phase.

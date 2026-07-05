# Phase 24 BM1366 Production Work Evidence

board: 205
source_commit: 37588dbc1293751029fb7e4a4cfc77cb42cc5aaa
reference_commit: c1915b0a63bfabebdb95a515cedfee05146c1d50
package_identity: not-produced-plan-24-04
evidence_level: implemented/unit,workflow
raw_artifacts_committed: no
redaction_status: passed

## Scope

Phase 24 proves code and test implementation for the BM1366 production work path. It does not claim detector-gated hardware execution, live pool socket success, accepted or rejected pool responses, or Phase 26 telemetry closure.

## Production Mode Evidence

The production ASIC mode is implemented in `crates/bitaxe-asic/src/bm1366/production.rs`. Plan 24-01 added production-only BM1366 payload and command primitives, including a command path that emits typed adapter actions without exposing raw frame construction to firmware.

Verification command from Plan 24-01:

```bash
bazel test //crates/bitaxe-asic:tests
```

## Active Work Registry Evidence

The pool-derived active-work registry is implemented in `crates/bitaxe-stratum/src/v1/production_work.rs`. Plan 24-02 bound queued and dispatched production work to `PoolSessionGeneration`, preserved redaction-safe work record formatting, and invalidated queued, active, and valid-job state on clean-jobs and reconnect.

Verification command from Plan 24-02:

```bash
bazel test //crates/bitaxe-stratum:tests
```

## Work Derivation Evidence

The production work path consumes Stratum v1 mining work and wraps the resulting BM1366 payload as production work. The registry retains enough typed job, timing, difficulty, and session-generation context for later correlation while committed evidence renders only category labels and implementation pointers.

## exact_non_claims

- nonzero version-mask and multi-midstate production support remain Phase 24 non-claims
- accepted/rejected share outcomes remain Phase 25-owned non-claims
- live Stratum socket success remains a Phase 25-owned non-claim
- API/WebSocket/statistics/scoreboard promotion remains a Phase 26-owned non-claim

## Conclusion

Phase 24 production-work evidence supports `implemented` status with `unit,workflow` evidence for code/test proof only. Hardware promotion remains outside this phase.

# Phase 24 BM1366 Production Work Path Summary

board: 205
source_commit: 37588dbc1293751029fb7e4a4cfc77cb42cc5aaa
reference_commit: c1915b0a63bfabebdb95a515cedfee05146c1d50
package_identity: not-produced-plan-24-04
evidence_level: implemented/unit,workflow
raw_artifacts_committed: no
redaction_status: passed

## Exact Claim

Phase 24 implements BM1366 production work path code/test proof.

## Evidence Links

- `docs/parity/evidence/phase-24-bm1366-production-work-path/production-work.md`
- `docs/parity/evidence/phase-24-bm1366-production-work-path/result-correlation.md`
- `docs/parity/evidence/phase-24-bm1366-production-work-path/redaction-review.md`

## Implementation Pointers

- `crates/bitaxe-asic/src/bm1366/production.rs`
- `crates/bitaxe-stratum/src/v1/production_work.rs`
- `crates/bitaxe-stratum/src/v1/mining_loop.rs`
- `crates/bitaxe-stratum/src/v1/controlled_runtime.rs`
- `firmware/bitaxe/src/controlled_mining_runtime.rs`
- `firmware/bitaxe/src/asic_adapter/status.rs`

## Verification Commands

```bash
bazel test //crates/bitaxe-asic:tests
bazel test //crates/bitaxe-stratum:tests
bazel test //crates/bitaxe-stratum:tests //crates/bitaxe-asic:tests
bazel test //crates/bitaxe-stratum:tests //crates/bitaxe-asic:tests //crates/bitaxe-safety:tests
bazel build //firmware/bitaxe:firmware
```

## exact_non_claims

- nonzero version-mask and multi-midstate production support remain Phase 24 non-claims
- nonce-vs-target proof and share-hash validation remain Phase 24 non-claims
- accepted/rejected share outcomes remain Phase 25-owned non-claims
- live Stratum socket success remains a Phase 25-owned non-claim
- API/WebSocket/statistics/scoreboard promotion remains a Phase 26-owned non-claim

## Conclusion

ASIC-09 through ASIC-12 are supported by implemented code and deterministic unit/workflow evidence from Phase 24. Phase 24 does not promote hardware evidence, live socket success, accepted or rejected pool responses, or telemetry closure.

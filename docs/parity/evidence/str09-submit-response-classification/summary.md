# STR-09 Live Submit-Response Classification

str09_status: accepted
board: 205
verified_source_commit: 532ab568228312157b3164820d9ad9f9ae221dbf
reference_commit: c1915b0a63bfabebdb95a515cedfee05146c1d50
attempt_source_commit: 3e0966a140edbff1a14d2a48ca63d140649762c0
hardware_rerun_used: false
raw_artifacts_committed: no
redaction_status: passed
exact_non_claims: rejected-share hardware, mismatched or stale response paths on
hardware, fallback or reconnect on hardware, exact upstream timeout or
keepalive equivalence, arbitrary pools, TLS, Stratum v2, unbounded mining,
other boards/ASICs, updates, recovery, profitability, SAFE-12, SAFE-13, and
release readiness

## Identity

| Input | Path or identity | SHA-256 |
| --- | --- | --- |
| Immutable plan | `docs/parity/work-plans/20260820T050854Z-STR-09/PLAN.md` | `79b7a064108ece1e05f6e1c49963aa62605ed2738d694debc7bd83e5cfa6dddd` |
| STR-001 socket | `docs/parity/evidence/str001-socket/stratum-socket-projection.json` | `dcb3eed396a268114b017d7ef4fbca9c427a390d7acf405fc52fbef6472122b8` |
| STR-006 coordinator | `docs/parity/evidence/str006-protocol-coordinator/protocol-coordinator-projection.json` | `f008171f26b7a8ae6b08859e3cfef4f0c5bf88937c049dd66b6f868c9bbfd6f7` |
| ASIC-004 result parsing | `docs/parity/evidence/asic004-result-parsing/asic-result-parsing-projection.json` | `e99c054c4d660155d5c2b1ee38d3f17aed5ae7101e7e4a5fd1c6451d1b48b7c7` |

All three public projections independently validated through their existing
Rust validators and are mode `0644`. STR-006 binds STR-001 and ASIC-004 to the
same accepted attempt, so the projections form one closed chain from hardware
commit `3e0966a140edbff1a14d2a48ca63d140649762c0`.

## Closed Live Chain

| Fact | Accepted result |
| --- | --- |
| Board | 205 |
| Package admitted | true |
| Runtime identity | `trusted` |
| Authorized production socket | true |
| ASIC-derived active work | true |
| Qualified correlated result before submit | true |
| Submit intent required before response | true |
| Matching accepted response observed | true |
| Safety status | `fresh` |
| Ordered terminal safe stop | true |
| Cleanup confirmed | true |
| Hardware rerun used | false |
| Redaction | passed |
| STR-09.live_submit_response_classified | true |
| STR-09.asic_correlation | passed |
| STR-09.safe_stop_status | complete |

## Current Classification

The pure classifier accepts only a response whose request identity and session
generation match a live `SubmitIntent`. Missing intent, mismatched request,
stale generation, unrelated messages, and malformed/non-response observations
cannot become accepted shares. Rejection classification retains only a closed
reason label. Live-runtime tests bind correlated bridge observations to submit
actions and redact submit context; production-session tests require matching
response identity, consume the lease on the first classified response, and
execute ordered safe stop.

The canonical Phase 30 conclusion now carries all three exact STR-09 admission
fields. Its checked-in current-artifact regressions require CFG-07, ASIC-11, and
STR-09 to validate together rather than exercising only the earlier CFG-07
promotion.

## Verification

The following passed against verified source `532ab568`:

- independent STR-001, STR-006, and ASIC-004 validators
- `cargo test -p bitaxe-stratum submit_response` (6 passed)
- `cargo test -p bitaxe-stratum live_runtime` (46 passed)
- `cargo test -p bitaxe-stratum production_session` (70 passed)
- `bazel test //tools/parity:tests` (passed)
- `cargo fmt --all`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo build --all-targets --all-features`
- `cargo test --all-features`
- `bun scripts/bright-builds-check.ts all`
- `just verify-reference`
- `just package`

## Non-Claims

This evidence does not verify rejected-share hardware, mismatched or stale
response paths on hardware, fallback or reconnect on hardware, exact upstream
timeout or keepalive equivalence, arbitrary pools, TLS, Stratum v2, unbounded
mining, other boards or ASICs, updates, recovery, profitability, or release
readiness. It does not promote SAFE-12 or SAFE-13.

# ASIC-11 BM1366 Result Correlation Before Submit

asic11_status: accepted
board: 205
verified_source_commit: bbbf390d80326e8aaa46f02ce520efe2aefcc3e3
reference_commit: c1915b0a63bfabebdb95a515cedfee05146c1d50
attempt_source_commit: 3e0966a140edbff1a14d2a48ca63d140649762c0
hardware_rerun_used: false
raw_artifacts_committed: no
redaction_status: passed
exact_non_claims: submit-response classification ownership, rejected share
hardware, frequency transitions, voltage/fan/thermal behavior, nonzero
version-mask or multi-midstate breadth, share-hash or network-target policy
beyond the accepted qualified result, live clean-jobs or reconnect, other
ASICs/boards, arbitrary pools/profiles, unbounded mining, OTA/recovery,
ASIC-12, STR-08, STR-09, SAFE-12, SAFE-13, and release readiness

## Identity

| Input | Path or identity | SHA-256 |
| --- | --- | --- |
| Immutable plan | `docs/parity/work-plans/20260819T151339Z-ASIC-11/PLAN.md` | `1cc38f93aaf94e82ae51b0f642b847bb141b791a653ace4674a303f33a02f79f` |
| ASIC-002 initialization | `docs/parity/evidence/asic002-initialization/asic-initialization-projection.json` | `eee750561a7c1dcec1a5698b1e5827d3f1508d43655c3c4aa237097338dcf8d4` |
| ASIC-003 work send | `docs/parity/evidence/asic003-work-send/asic-work-send-projection.json` | `447af65ae9e6cd5cc2199ef639ff8e0fa7f63d4c9708570bd66781c5a162e80c` |
| ASIC-004 result parsing | `docs/parity/evidence/asic004-result-parsing/asic-result-parsing-projection.json` | `e99c054c4d660155d5c2b1ee38d3f17aed5ae7101e7e4a5fd1c6451d1b48b7c7` |

All three public projections independently validated through the existing Rust
validators. Each projection is mode `0644`. ASIC-004 binds ASIC-003, which
binds ASIC-002. The live runtime correlates a parsed BM1366 result to active
pool work before submit intent, and the accepted result-parsing chain proves
that correlation on hardware commit
`3e0966a140edbff1a14d2a48ca63d140649762c0`.

## Closed Live Chain

| Fact | Accepted result |
| --- | --- |
| Board | 205 |
| Package admitted | true |
| Runtime identity | `trusted` |
| Mining-ready initialization | true |
| Production UART retained | true |
| Live production work observed | true |
| Job lookup validation | true |
| Correlation semantics compatible | true |
| Qualified parsed result observed | true |
| Submit outcome | `accepted` |
| Safety status | `fresh` |
| Safe stop confirmed | true |
| Cleanup confirmed | true |
| Hardware rerun used | false |
| Redaction | passed |

## Current Correlation

Current host tests prove `correlate_nonce_result` returns submit intent only
for the current generation and active job, and fail-closes uncorrelated,
stale, duplicate, generation-mismatched, and drifted-target observations.
Share-qualification tests keep a nonce below pool difficulty from becoming
submit intent. Production-session tests prove ASIC effects bind to generation
and valid-job context, and that an accepted first submit consumes the current
generation.

## Verification

The following passed against verified source `bbbf390d`:

- independent ASIC-002, ASIC-003, and ASIC-004 validators
- `cargo test -p bitaxe-stratum production_work` (21 passed)
- `cargo test -p bitaxe-stratum production_session` (70 passed)
- `just verify-reference`
- `just package`

## Non-Claims

This evidence does not verify submit-response classification ownership,
rejected share hardware, frequency transitions, voltage/fan/thermal behavior,
nonzero version-mask or multi-midstate breadth, share-hash or network-target
policy beyond the accepted qualified result, live clean-jobs or reconnect,
other ASICs or boards, arbitrary pools or profiles, unbounded mining,
OTA/recovery, or release readiness. It does not promote ASIC-12, STR-08,
STR-09, SAFE-12, or SAFE-13.

# ASIC-10 Pool-Derived BM1366 Work Registry

asic10_status: accepted
board: 205
verified_source_commit: 9a57318a544ef59d1ab5623fc823ae0fb80760d2
reference_commit: c1915b0a63bfabebdb95a515cedfee05146c1d50
attempt_source_commit: 3e0966a140edbff1a14d2a48ca63d140649762c0
hardware_rerun_used: false
raw_artifacts_committed: no
redaction_status: passed
exact_non_claims: result-correlation policy beyond the accepted predecessor,
submit-response classification ownership, frequency transitions,
voltage/fan/thermal behavior, nonzero version-mask or multi-midstate breadth,
live clean-jobs or reconnect, other ASICs/boards, arbitrary pools/profiles,
unbounded mining, OTA/recovery, ASIC-11, ASIC-12, STR-08, STR-09, SAFE-12,
SAFE-13, and release readiness

## Identity

| Input | Path or identity | SHA-256 |
| --- | --- | --- |
| Immutable plan | `docs/parity/work-plans/20260819T150619Z-ASIC-10/PLAN.md` | `222fd16bf8ca412658f98c44e325c962f07728037ca52fb7de2f5f6301a4a063` |
| ASIC-002 initialization | `docs/parity/evidence/asic002-initialization/asic-initialization-projection.json` | `eee750561a7c1dcec1a5698b1e5827d3f1508d43655c3c4aa237097338dcf8d4` |
| ASIC-003 work send | `docs/parity/evidence/asic003-work-send/asic-work-send-projection.json` | `447af65ae9e6cd5cc2199ef639ff8e0fa7f63d4c9708570bd66781c5a162e80c` |

Both public projections independently validated through the existing Rust
validators. Each projection is mode `0644`. ASIC-003 binds ASIC-002. The live
runtime owns one production work registry, and the accepted work-send chain
proves pool-derived production dispatch on hardware commit
`3e0966a140edbff1a14d2a48ca63d140649762c0`.

## Closed Live Chain

| Fact | Accepted result |
| --- | --- |
| Board | 205 |
| Package admitted | true |
| Runtime identity | `trusted` |
| Mining-ready initialization | true |
| Production UART retained | true |
| Production-ready gate required | true |
| Live production work observed | true |
| Qualified parsed result observed | true |
| Submit outcome | `accepted` |
| Safety status | `fresh` |
| Safe stop confirmed | true |
| Cleanup confirmed | true |
| Hardware rerun used | false |
| Redaction | passed |

## Current Registry

Current host tests prove the registry enqueues valid jobs for the current
generation, preserves pool context through dispatch, advances generation once
per session invalidation, invalidates queued and active jobs on clean-jobs,
clears work on reconnect, and redacts raw context. Production-session tests
prove the live session binds ASIC effects to generation and valid-job context
and accepts a first submit only for the current generation.

## Verification

The following passed against verified source `9a57318a`:

- independent ASIC-002 and ASIC-003 validators
- `cargo test -p bitaxe-stratum production_work` (21 passed)
- `cargo test -p bitaxe-stratum production_session` (70 passed)
- `just verify-reference`
- `just package`

## Non-Claims

This evidence does not verify result-correlation policy beyond the accepted
predecessor, submit-response classification ownership, frequency transitions,
voltage/fan/thermal behavior, nonzero version-mask or multi-midstate breadth,
live clean-jobs or reconnect, other ASICs or boards, arbitrary pools or
profiles, unbounded mining, OTA/recovery, or release readiness. It does not
promote ASIC-11, ASIC-12, STR-08, STR-09, SAFE-12, or SAFE-13.

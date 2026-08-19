# ASIC-09 Diagnostic And Production Mode Separation

asic09_status: accepted
board: 205
verified_source_commit: 7f8ca3bb9d6e9b7b56d1040b1d6d6eeb2bf2648d
reference_commit: c1915b0a63bfabebdb95a515cedfee05146c1d50
attempt_source_commit: 3e0966a140edbff1a14d2a48ca63d140649762c0
hardware_rerun_used: false
raw_artifacts_committed: no
redaction_status: passed
exact_non_claims: arbitrary diagnostic builds, frequency transitions,
voltage/fan/thermal behavior, nonzero version-mask or multi-midstate breadth,
arbitrary-load serial behavior, other ASICs/boards, arbitrary pools/profiles,
unbounded mining, OTA/recovery, ASIC-10, ASIC-11, ASIC-12, STR-08, STR-09, and
release readiness

## Identity

| Input | Path or identity | SHA-256 |
| --- | --- | --- |
| Immutable plan | `docs/parity/work-plans/20260818T160811Z-ASIC-09/PLAN.md` | `b813a749174cc44f0a27d6e5fb6be9c6f7003d95a83405a54511b4f665df05ca` |
| ASIC-002 initialization | `docs/parity/evidence/asic002-initialization/asic-initialization-projection.json` | `eee750561a7c1dcec1a5698b1e5827d3f1508d43655c3c4aa237097338dcf8d4` |
| ASIC-003 work send | `docs/parity/evidence/asic003-work-send/asic-work-send-projection.json` | `447af65ae9e6cd5cc2199ef639ff8e0fa7f63d4c9708570bd66781c5a162e80c` |
| ASIC-004 result parsing | `docs/parity/evidence/asic004-result-parsing/asic-result-parsing-projection.json` | `e99c054c4d660155d5c2b1ee38d3f17aed5ae7101e7e4a5fd1c6451d1b48b7c7` |
| ASIC-005 serial transport | `docs/parity/evidence/asic005-serial-transport/asic-serial-transport-projection.json` | `bad828db694ee59c4ef3d77b2e58ef89e0195ef382526b97912d0a71e882ad69` |

All four public projections independently validated through the existing Rust
validators. Each projection is mode `0644`. ASIC-003 binds ASIC-002; ASIC-004
binds ASIC-003; ASIC-005 binds ASIC-003 and ASIC-004. The four artifacts
therefore form one same-attempt chain from hardware commit
`3e0966a140edbff1a14d2a48ca63d140649762c0`.

## Closed Live Chain

| Fact | Accepted result |
| --- | --- |
| Board | 205 |
| Package admitted | true |
| Runtime identity | `trusted` |
| Planned initialization steps | 9 |
| Exactly one chip detected | true |
| Mining-ready initialization | true |
| Production UART retained | true |
| Production-ready gate required | true |
| Live production work observed | true |
| Qualified parsed result observed | true |
| Live work TX and result RX | true |
| Submit outcome | `accepted` |
| Safety status | `fresh` |
| Safe stop confirmed | true |
| Cleanup confirmed | true |
| Hardware rerun used | false |
| Redaction | passed |

## Current Mode Separation

Current host tests prove diagnostic admission fails closed unless both the
diagnostic selector and its exact compile-time acknowledgement match; exact
pairs select only their own diagnostic mode. Current production command tests
prove only production work and production result variants. The production
executor source contains no diagnostic-work command variant. Host
`cargo test -p bitaxe-firmware` cannot compile the ESP-IDF crate on
`aarch64-apple-darwin`; the same executor contract was verified by source
review, and `just package` compiled the current firmware image.

## Verification

The following passed against verified source `7f8ca3bb`:

- independent ASIC-002, ASIC-003, ASIC-004, and ASIC-005 validators
- `cargo test -p bitaxe-asic adapter_gate` (8 passed)
- `cargo test -p bitaxe-asic production` (9 passed)
- production-executor source review and firmware package build
- `just verify-reference`
- `just package`

## Non-Claims

This evidence does not verify arbitrary diagnostic builds, frequency
transitions, voltage/fan/thermal behavior, nonzero version-mask or
multi-midstate breadth, arbitrary-load serial behavior, other ASICs or boards,
arbitrary pools or profiles, unbounded mining, OTA/recovery, or release
readiness. It does not promote ASIC-10, ASIC-11, ASIC-12, STR-08, or STR-09.

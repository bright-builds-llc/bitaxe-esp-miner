# ASIC-12 BM1366 Production Fail-Closed Blockers and Redaction

asic12_status: accepted
board: 205
verified_source_commit: 30e0340695e1f307dfcdc7aa6949da07beb616f5
reference_commit: c1915b0a63bfabebdb95a515cedfee05146c1d50
attempt_source_commit: 3e0966a140edbff1a14d2a48ca63d140649762c0
hardware_rerun_used: false
raw_artifacts_committed: no
redaction_status: passed
exact_non_claims: hardware fault injection for every blocker, arbitrary
diagnostic builds, nonzero version-mask or multi-midstate breadth, arbitrary-
load serial behavior, rejected-share hardware, frequency transitions,
voltage/fan/thermal behavior, other ASICs/boards, arbitrary pools/profiles,
unbounded mining, OTA/recovery, STR-08, STR-09, SAFE-12, SAFE-13, and release
readiness

## Identity

| Input | Path or identity | SHA-256 |
| --- | --- | --- |
| Immutable plan | `docs/parity/work-plans/20260820T041751Z-ASIC-12/PLAN.md` | `95c7204ad0b0388203a54b8ccb9c9c0252704b9fb11f550802ab4b91313e8a8f` |
| ASIC-002 initialization | `docs/parity/evidence/asic002-initialization/asic-initialization-projection.json` | `eee750561a7c1dcec1a5698b1e5827d3f1508d43655c3c4aa237097338dcf8d4` |
| ASIC-003 work send | `docs/parity/evidence/asic003-work-send/asic-work-send-projection.json` | `447af65ae9e6cd5cc2199ef639ff8e0fa7f63d4c9708570bd66781c5a162e80c` |
| ASIC-004 result parsing | `docs/parity/evidence/asic004-result-parsing/asic-result-parsing-projection.json` | `e99c054c4d660155d5c2b1ee38d3f17aed5ae7101e7e4a5fd1c6451d1b48b7c7` |
| ASIC-005 serial transport | `docs/parity/evidence/asic005-serial-transport/asic-serial-transport-projection.json` | `bad828db694ee59c4ef3d77b2e58ef89e0195ef382526b97912d0a71e882ad69` |

All four public projections independently validated through the existing Rust
validators and are mode `0644`. ASIC-005 binds ASIC-003 and ASIC-004; ASIC-004
binds ASIC-003; ASIC-003 binds ASIC-002. They therefore form one accepted
same-attempt chain from hardware commit
`3e0966a140edbff1a14d2a48ca63d140649762c0`.

## Closed Live Chain

| Fact | Accepted result |
| --- | --- |
| Board | 205 |
| Package admitted | true |
| Runtime identity | `trusted` |
| Mining-ready initialization | true |
| Production UART retained | true |
| Live production work TX observed | true |
| Live production result RX observed | true |
| Qualified correlated result observed | true |
| Submit outcome | `accepted` |
| Safety status | `fresh` |
| Safe stop confirmed | true |
| Cleanup confirmed | true |
| Hardware rerun used | false |
| Redaction | passed |

## Current Fail-Closed Contract

The pure ASIC core owns three exact successful status lines and eleven typed
fail-closed reason lines. Every fail-closed line includes only the closed reason
label plus `mining=disabled` and `work_submission=disabled`. The firmware shell
selects the existing info or warning level and logs that pure rendering without
owning formatting behavior.

The blocker vocabulary is closed to prerequisite, ASIC initialization, UART,
reset, result timeout, malformed result, stale work, uncorrelated job,
duplicate result, wrong session, and target mismatch categories. Unit coverage
requires lower-snake-case labels and rejects sensitive fragments. Production-
work debug coverage redacts pool-derived job, target, payload, nonce, submit,
registry, and scoreboard context. Production-session coverage preserves typed
ASIC failure subcategories through terminal safe stop and prevents secret
network or ASIC effects when readiness fails.

## Verification

The following passed against verified source `30e03406`:

- independent ASIC-002, ASIC-003, ASIC-004, and ASIC-005 validators
- `cargo test -p bitaxe-asic production` (11 passed)
- `cargo test -p bitaxe-stratum production_work` (21 passed)
- `cargo test -p bitaxe-stratum production_session` (70 passed)
- `cargo fmt --all`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo build --all-targets --all-features`
- `cargo test --all-features`
- `bun scripts/bright-builds-check.ts all`
- `just verify-reference`
- `just package`

## Non-Claims

This evidence does not claim hardware fault injection for every blocker,
arbitrary diagnostic builds, nonzero version-mask or multi-midstate breadth,
arbitrary-load serial behavior, rejected-share hardware, frequency transitions,
voltage/fan/thermal behavior, other ASICs or boards, arbitrary pools or
profiles, unbounded mining, OTA/recovery, or release readiness. It does not
promote STR-08, STR-09, SAFE-12, or SAFE-13.

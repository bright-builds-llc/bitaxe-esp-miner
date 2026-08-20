# STR-08 Live Stratum Socket Runtime Lifecycle

str08_status: accepted
board: 205
verified_source_commit: 8f86924a34e3988da15b0bc6b274ecd1c3806c21
reference_commit: c1915b0a63bfabebdb95a515cedfee05146c1d50
attempt_source_commit: 3e0966a140edbff1a14d2a48ca63d140649762c0
hardware_rerun_used: false
raw_artifacts_committed: no
redaction_status: passed
exact_non_claims: fallback or reconnect on hardware, exact upstream timeout or
keepalive option equivalence, DNS/IP-family preference parity, arbitrary pools,
TLS, Stratum v2, rejected-share hardware, unbounded socket stability, other
boards, updates, recovery, profitability, STR-09, SAFE-12, SAFE-13, and release
readiness

## Identity

| Input | Path or identity | SHA-256 |
| --- | --- | --- |
| Immutable plan | `docs/parity/work-plans/20260820T045045Z-STR-08/PLAN.md` | `18e4a60f4779e6285f2e16a910ede0501b86405b6fe76b60c533f85898b29480` |
| STR-001 socket | `docs/parity/evidence/str001-socket/stratum-socket-projection.json` | `dcb3eed396a268114b017d7ef4fbca9c427a390d7acf405fc52fbef6472122b8` |
| STR-006 coordinator | `docs/parity/evidence/str006-protocol-coordinator/protocol-coordinator-projection.json` | `f008171f26b7a8ae6b08859e3cfef4f0c5bf88937c049dd66b6f868c9bbfd6f7` |

Both public projections independently validated through their existing Rust
validators and are mode `0644`. STR-006 binds the exact STR-001 projection,
ASIC initialization, production work, and result-parsing inputs. The two
projections therefore form one accepted same-attempt chain from hardware commit
`3e0966a140edbff1a14d2a48ca63d140649762c0`.

## Closed Live Chain

| Fact | Accepted result |
| --- | --- |
| Board | 205 |
| Package admitted | true |
| Runtime identity | `trusted` |
| Typed connect/write/close commands | true |
| Typed connected/bytes/failure/closed events | true |
| Transport epoch isolation | true |
| Hardware prepared before pool access | true |
| Authorized before ASIC dispatch | true |
| Live work/result path | true |
| Qualified result before submit | true |
| Submit outcome | `accepted` |
| Safety status | `fresh` |
| Ordered terminal safe stop | true |
| Cleanup confirmed | true |
| Hardware rerun used | false |
| Redaction | passed |

## Current Lifecycle

Current live-runtime tests cover configure, subscribe, authorization, difficulty,
extranonce, notify, work generation, bridge correlation, submit action,
classification, reconnect invalidation, session replacement, clean-jobs, and
redacted configuration/message/context formatting. Production-session tests
cover the admitted end-to-end lifecycle, typed transport epochs, retry budgets,
primary/fallback policy, failed primary probes, reconnect/clean-jobs
invalidation, response identity, ordered terminal safe stop, and redaction.

The host production-transport target exercises the actual firmware worker
against loopback TCP, including connect/write events, preserved partial input,
typed connection failure, and redacted debug output. These current tests bind
the accepted hardware socket proof to the present pure core and firmware shell
without accessing any protected runtime input.

## Verification

The following passed against verified source `8f86924a`:

- independent STR-001 and STR-006 validators
- `cargo test -p bitaxe-stratum live_runtime` (46 passed)
- `cargo test -p bitaxe-stratum production_session` (70 passed)
- `bazel test //firmware/bitaxe:production_transport_tests` (passed)
- `cargo fmt --all`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo build --all-targets --all-features`
- `cargo test --all-features`
- `bun scripts/bright-builds-check.ts all`
- `just verify-reference`
- `just package`

## Non-Claims

This evidence does not verify fallback or reconnect on hardware, exact upstream
timeout or keepalive option equivalence, DNS/IP-family preference parity,
arbitrary pools, TLS, Stratum v2, rejected-share hardware, unbounded socket
stability, other boards, updates, recovery, profitability, or release readiness.
It does not promote STR-09, SAFE-12, or SAFE-13.

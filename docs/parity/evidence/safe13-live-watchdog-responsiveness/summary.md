# SAFE-13 Watchdog Responsiveness Under Live Runtime Load

safe13_status: accepted
board: 205
verified_source_commit: 57dba7b6673e5a25e28c5b1b4db83662d91735f3
reference_commit: c1915b0a63bfabebdb95a515cedfee05146c1d50
hardware_attempt_sources: 60a56d4935ced15eeb5ec6950b1ad4ea35fdf223,
3e0966a140edbff1a14d2a48ca63d140649762c0,
56f4eb1ce50f81fe2a1dd80ac344e551b16771c9
hardware_rerun_used: false
raw_artifacts_committed: no
redaction_status: passed
exact_non_claims: deliberate watchdog starvation or task stalls on hardware,
actual watchdog-triggered reset/recovery, arbitrary long-running or unbounded
load, every firmware task, other boards/ASICs, fault injection, OTA/recovery,
and release readiness

## Identity

| Input | Path or identity | SHA-256 |
| --- | --- | --- |
| Immutable plan | `docs/parity/work-plans/20260820T054935Z-SAFE-13/PLAN.md` | `377809918db548e05249e8bac4cb634ea01659c43f874a442557a1d41fb6b8eb` |
| SAFE-10 prerequisite readiness | `docs/parity/evidence/safe10-prerequisite-readiness/safe10-projection.json` | `4e9b91bd29629aec098b9967b9bb27b9c1358f64c11819a77f8c8da4c212a20e` |
| STR-006 protocol coordinator | `docs/parity/evidence/str006-protocol-coordinator/protocol-coordinator-projection.json` | `f008171f26b7a8ae6b08859e3cfef4f0c5bf88937c049dd66b6f868c9bbfd6f7` |
| V12 runtime health | `docs/parity/evidence/v12-runtime-health-205/runtime-health-projection.json` | `44f081451d61ecc59dd21f70d72fae7d71e9611441d406f31f441727e5a11e14` |

All three public projections independently validated through their existing
Rust validators and are mode `0644`. SAFE-10 supplies sustained detector-gated
load evidence; STR-006 binds watchdog feeding to the accepted production owner
loop; runtime health independently proves the observation and publication path.

## Detector-Gated Live Safety Hardware Proof

| Fact | Accepted result |
| --- | --- |
| Board | 205 |
| Detector admitted | true |
| Runtime identity | `trusted` |
| Active duration | 600,746 ms |
| Covered load windows | 20 of 20 |
| Active state valid | true |
| Work renewal valid | true |
| Watchdog valid in every window | true |
| Production owner-loop watchdog feed | true |
| Task watchdog participation | `participating` |
| Task watchdog reason | `feed_fresh` |
| Feed sequence non-regressing | true |
| Feed age bounded | true |
| Supervisor checkpoint health | `healthy` |
| Checkpoint sequence non-regressing | true |
| Checkpoint age bounded | true |
| Accepted submit observed | true |
| Safe stop confirmed | true |
| Cleanup complete | true |
| Redaction | passed |

## Current Watchdog Contract

The production owner subscribes before entering its loop, feeds at loop start
and cadence boundaries, and records owner phase/subphase progress before and
during effects. Long safe-stop operations emit step-specific heartbeats. The
watchdog observation model distinguishes subscription, feed, unsubscription,
store-read, sequence, freshness, owner-phase, and wait-state failures without
publishing free-form values.

Current tests prove exact step budgets and yield thresholds; stale, regressed,
missing, failed, and nonparticipating observations fail closed; a healthy
supervisor alone cannot imply task-watchdog participation; producer feeds and
supervisor checkpoints retain independent sequences; progress heartbeats occur
before effect completion; and public runtime-health reads are passive.

## Verification

The following passed against verified source `57dba7b6`:

- independent SAFE-10, STR-006, and runtime-health validators
- `cargo test -p bitaxe-safety watchdog` (6 passed)
- `cargo test -p bitaxe-core runtime_health` (27 passed)
- `cargo test -p bitaxe-stratum production_session` (70 passed)
- the focused firmware owner-progress, supervisor-checkpoint,
  task-watchdog-observation, and passive runtime-health targets
- `cargo fmt --all`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo build --all-targets --all-features`
- `cargo test --all-features`
- `bun scripts/bright-builds-check.ts all`
- `just verify-reference`
- `just package`

## Non-Claims

This evidence does not verify deliberate watchdog starvation or task stalls on
hardware, actual watchdog-triggered reset/recovery, arbitrary long-running or
unbounded load, every firmware task, other boards or ASICs, fault injection,
OTA/recovery, or release readiness.

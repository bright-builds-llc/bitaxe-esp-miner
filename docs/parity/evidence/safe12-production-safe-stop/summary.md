# SAFE-12 Production Mining Safe Stop

safe12_status: accepted
board: 205
verified_source_commit: 308f312f63951daceb2e49ead2a515e979e91453
reference_commit: c1915b0a63bfabebdb95a515cedfee05146c1d50
hardware_attempt_sources: 60a56d4935ced15eeb5ec6950b1ad4ea35fdf223,
3e0966a140edbff1a14d2a48ca63d140649762c0
hardware_rerun_used: false
raw_artifacts_committed: no
redaction_status: passed
exact_non_claims: fault-injected safe stop on hardware, per-step electrical
timing or waveform measurement, power-loss interruption, automatic thermal or
fan fault recovery, arbitrary profiles/pools, other boards/ASICs, unbounded
mining, OTA/recovery, SAFE-13, and release readiness

## Identity

| Input | Path or identity | SHA-256 |
| --- | --- | --- |
| Immutable plan | `docs/parity/work-plans/20260820T052841Z-SAFE-12/PLAN.md` | `e274a974b44503c189772026fc8797b7ea1c0a2db0ec2495725ddd2287494de9` |
| SAFE-10 prerequisite readiness | `docs/parity/evidence/safe10-prerequisite-readiness/safe10-projection.json` | `4e9b91bd29629aec098b9967b9bb27b9c1358f64c11819a77f8c8da4c212a20e` |
| STR-006 protocol coordinator | `docs/parity/evidence/str006-protocol-coordinator/protocol-coordinator-projection.json` | `f008171f26b7a8ae6b08859e3cfef4f0c5bf88937c049dd66b6f868c9bbfd6f7` |
| PWR-002 ASIC power initialization | `docs/parity/evidence/pwr002-asic-power-initialization/power-initialization-projection.json` | `0668c274d09b3e39d7d5edfea4b2e66c97248ff77de9192981f3af00e547ddfe` |
| PWR-003 core-voltage control | `docs/parity/evidence/pwr003-core-voltage-control/core-voltage-control-projection.json` | `11dd1abbf6fda86d203fdcff49b420ab5139e1d29c35f4d17000c61c3112ae68` |

All four public projections independently validated through their existing Rust
validators and are mode `0644`. SAFE-10 proves the full detector-admitted live
campaign and current-compatible stop semantics. STR-006 supplies the ordered
terminal coordinator proof. PWR-002 and PWR-003 supply the accepted physical
ASIC-disable and core-rail actuation evidence.

## Detector-Gated Live Safety Hardware Proof

| Fact | Accepted result |
| --- | --- |
| Board | 205 |
| Detector admitted | true |
| Runtime identity | `trusted` |
| Active duration | 600,746 ms |
| Covered safety windows | 20 of 20 |
| Fresh required safety observations | true |
| Watchdog continuity | true |
| Accepted submit observed | true |
| Final terminal state consumed | true |
| Serial finish observed | true |
| Ordered terminal safe stop | true |
| Hardware safe stop confirmed | true |
| Rollback steps attempted | 8 of 8 |
| ASIC disable commanded | true |
| Core-rail active-low disable | true |
| Cleanup complete | true |
| Redaction | passed |

## Current Ordered Stop

The pure production session orders `BlockSubmissions`,
`InvalidateWorkAndSubmissions`, `StopAsicInteraction`, pool-connection close,
`SafeStopHardware`, and final publication. The hardware shell then executes the
typed shutdown plan: stop dispatch, reduce frequency and reset nonce state,
hold reset low, disable the core rail, disable the ASIC, command maximum
cooling, wait for fresh cool-temperature proof, and return to paused cooling.
Only successful completion emits `HardwareSafeStopConfirmed`; a failed step
keeps the session fail closed with its typed category.

Current tests prove the effect order and final disabled snapshot are idempotent,
the first accepted response consumes the lease and starts safe stop, terminal
expiry overtakes a pending resumable stop, ASIC failures preserve their subtype,
campaign status publishes `safe_stopped`/`confirmed`, and progress heartbeats
cover the physical sequence before completion.

## Verification

The following passed against verified source `308f312f`:

- independent SAFE-10, STR-006, PWR-002, and PWR-003 validators
- `cargo test -p bitaxe-stratum safe_stop` (8 passed)
- `cargo test -p bitaxe-stratum production_session` (70 passed)
- `bazel test //firmware/bitaxe:mining_actuation_tests //firmware/bitaxe:production_campaign_status_tests //firmware/bitaxe:production_owner_progress_tests` (passed)
- `cargo fmt --all`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo build --all-targets --all-features`
- `cargo test --all-features`
- `bun scripts/bright-builds-check.ts all`
- `just verify-reference`
- `just package`

## Non-Claims

This evidence does not verify fault-injected safe stop on hardware, per-step
electrical timing or waveform measurement, power-loss interruption, automatic
thermal or fan fault recovery, arbitrary profiles or pools, other boards or
ASICs, unbounded mining, OTA/recovery, or release readiness. It does not
promote SAFE-13.

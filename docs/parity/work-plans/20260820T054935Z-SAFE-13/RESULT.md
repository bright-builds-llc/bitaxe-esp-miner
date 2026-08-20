# Parity work result

- Parity row: `SAFE-13`
- Final status: `verified`
- Implementation commit: `57dba7b6673e5a25e28c5b1b4db83662d91735f3`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Hardware attempts: none; committed sealed SAFE-10, STR-006, and runtime-health
  evidence was joined without a hardware rerun

## Evidence and verification

The source-bound summary at
`docs/parity/evidence/safe13-live-watchdog-responsiveness/summary.md` joins
three already accepted public projections after independent Rust validation:

- SAFE-10 `4e9b91bd29629aec098b9967b9bb27b9c1358f64c11819a77f8c8da4c212a20e`
- STR-006 `f008171f26b7a8ae6b08859e3cfef4f0c5bf88937c049dd66b6f868c9bbfd6f7`
- runtime health `44f081451d61ecc59dd21f70d72fae7d71e9611441d406f31f441727e5a11e14`

SAFE-10 proves watchdog validity through all 20 windows of a detector-admitted
600,746-ms accepted production campaign with active work, safe stop, and
cleanup. STR-006 proves the accepted production owner loop feeds its watchdog.
The runtime-health projection proves participating, fresh, bounded-age,
non-regressing task-watchdog feeds and healthy supervisor checkpoints through
same-boot HTTP/WebSocket views. All projections are mode `0644`; no new
projector or protected artifact was used.

Current tests prove exact watchdog budgets/yield thresholds, closed failure
categories, freshness/sequence rules, independent supervisor/task truth,
owner-phase/subphase observation, step-specific effect heartbeats, session
lifecycle coverage, and passive runtime-health publication.

The following focused and required gates passed on source `57dba7b6`:

- independent validation of SAFE-10, STR-006, and runtime health
- `cargo test -p bitaxe-safety watchdog`
- `cargo test -p bitaxe-core runtime_health`
- `cargo test -p bitaxe-stratum production_session`
- the focused firmware watchdog/progress/checkpoint/runtime-health targets
- the ordered format, strict Clippy, all-target build, and all-feature test gates
- `bun scripts/bright-builds-check.ts all`
- `just verify-reference`
- `just package`

## Conclusion

`SAFE-13` has detector-gated live safety hardware proof that the production
watchdog remained valid throughout a bounded accepted Ultra 205 mining campaign,
plus current-source proof of fresh participating feeds and healthy checkpoints.
This supports `SAFE-13` at `verified` with
`unit,workflow,hardware-smoke,hardware-regression` evidence.

## Non-claims and residual risks

This result does not verify deliberate watchdog starvation or task stalls on
hardware, actual watchdog-triggered reset/recovery, arbitrary long-running or
unbounded load, every firmware task, other boards or ASICs, fault injection,
OTA/recovery, or release readiness. No credential or protected-attempt access,
detector, device/USB/network runtime, flash, monitor, mining, restart, recovery,
hardware attempt, fault injection, external UART/BAP, pins, or electrical work
occurred during this plan.

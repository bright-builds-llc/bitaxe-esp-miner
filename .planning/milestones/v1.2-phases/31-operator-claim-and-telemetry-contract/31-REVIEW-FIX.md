---
phase: 31-operator-claim-and-telemetry-contract
fixed_at: 2026-07-13T21:43:38Z
review_path: .planning/phases/31-operator-claim-and-telemetry-contract/31-REVIEW.md
iteration: 1
findings_in_scope: 2
fixed: 2
skipped: 0
status: all_fixed
generated_by: gsd-code-fixer
lifecycle_mode: yolo
phase_lifecycle_id: 31-2026-07-13T19-47-51
---

# Phase 31: Code Review Fix Report

**Fixed at:** 2026-07-13T21:43:38Z
**Source review:** `.planning/phases/31-operator-claim-and-telemetry-contract/31-REVIEW.md`
**Iteration:** 1

**Summary:**

- Findings in scope: 2
- Fixed: 2
- Skipped: 0

## Fixed Issues

### WR-01: Retained Phase 27 samples are published with fixed compatibility stamps

**Status:** fixed
**Commit:** `2463764` (`fix(31): block unstamped phase 27 telemetry`)
**Files modified:** `crates/bitaxe-api/src/observation.rs`, `firmware/bitaxe/src/safety_adapter/phase27_bring_up.rs`
**Applied fix:** Phase 27 no longer projects placeholder-stamped compatibility observations as fresh operator truth. Its retained internal safety snapshot remains intact, while the operator observation store receives an explicit all-unavailable snapshot until Phase 32 installs the sole producer with real boot-session, sequence, and acquisition-time provenance.
**Regression guard:** Added a host-testable contract proving all six facts from an unstamped legacy source remain unavailable and contain no last-good sample stamp.
**Residual risk:** Phase 32 must still supply real producer provenance before these facts can become fresh. No hardware verification or producer effect was added in this fix.

### WR-02: Legacy report mapping creates contradictory fresh and unavailable truth

**Status:** fixed
**Commit:** `9f66dd9` (`fix(31): require stamped truth for telemetry values`)
**Files modified:** `crates/bitaxe-api/src/snapshot.rs`, `crates/bitaxe-api/src/statistics.rs`, `crates/bitaxe-api/src/wire.rs`
**Applied fix:** Fresh legacy reports without per-fact producer stamps now map to an unavailable, zero-compatible snapshot. System-info and statistics projections also suppress each of the six compatibility numerics unless its corresponding truth is `Fresh` with a stamp, so manually contradictory snapshots cannot serialize live-looking values through operator APIs.
**Regression guard:** Added a serialized `SystemInfoWire` test with aggregate `Fresh` plus unavailable, stale, and fault fact truths; the three corresponding numerics are rejected while an independently fresh stamped temperature remains visible. Positive system-info and statistics tests now construct live numerics only from `TelemetryObservations` with real sample stamps.
**Residual risk:** The six Phase 31 facts are protected. Other compatibility-only telemetry fields remain outside the Phase 31 truth contract and are owned by later phases.

## Verification

Before each atomic fix commit, the required Rust gate passed separately and in exact order:

1. `cargo fmt --all`
2. `cargo clippy --all-targets --all-features -- -D warnings`
3. `cargo build --all-targets --all-features`
4. `cargo test --all-features`

Additional verification passed:

- `cargo test -p bitaxe-api --all-features unstamped_legacy_source_cannot_publish_fresh_operator_truth`
- `cargo test -p bitaxe-api --all-features safety_telemetry`
- `cargo test -p bitaxe-api --all-features system_info_wire_rejects_nonfresh_truth_numeric_claims_even_with_fresh_aggregate`
- `bazel test //crates/bitaxe-api:tests`
- `just build`
- `git diff --check`

The canonical ESP32-S3 firmware build completed successfully for both fixes. Its eight reported dead-code warnings are pre-existing and are outside the host-workspace `clippy -D warnings` surface.

## Skipped Issues

None.

***

_Fixed: 2026-07-13T21:43:38Z_
_Fixer: gsd-code-fixer_
_Iteration: 1_

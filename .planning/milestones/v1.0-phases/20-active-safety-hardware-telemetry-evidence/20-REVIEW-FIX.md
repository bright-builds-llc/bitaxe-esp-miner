---
phase: 20-active-safety-hardware-telemetry-evidence
fixed_at: 2026-07-03T23:56:21Z
review_path: .planning/phases/20-active-safety-hardware-telemetry-evidence/20-REVIEW.md
iteration: 1
findings_in_scope: 2
fixed: 2
skipped: 0
status: all_fixed
---

# Phase 20: Code Review Fix Report

**Fixed at:** 2026-07-03T23:56:21Z
**Source review:** .planning/phases/20-active-safety-hardware-telemetry-evidence/20-REVIEW.md
**Iteration:** 1

**Summary:**
- Findings in scope: 2
- Fixed: 2
- Skipped: 0

## Fixed Issues

### WR-01: Blocked Live Telemetry Manifest Claims Hardware-Smoke Projection

**Status:** fixed
**Files modified:** `docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/live-api-websocket-telemetry/allow-live-telemetry.json`, `docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/live-api-websocket-telemetry/live-telemetry.log`, `docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/live-api-websocket-telemetry.md`
**Commit:** 979a2ee
**Applied fix:** Changed the blocked live telemetry allow manifest to `claim_tier: unsupported-pending` and `evidence_class: deferred`, recorded missing `DEVICE_URL` and disabled network scanning in `allowed_inputs`, regenerated the blocked helper log, and updated the markdown evidence summary to match the deferred boundary.

### WR-02: Safety Allow Validation Does Not Enforce Surface-Specific Claim Contracts

**Status:** fixed: requires human verification
**Files modified:** `tools/parity/src/safety_allow.rs`, `scripts/phase20-failure-paths.sh`, `scripts/phase20-failure-paths-test.sh`
**Commit:** 917e683
**Applied fix:** Added per-surface safety claim-tier enforcement, approved command shape checks, failure-path `fault-stimulus` required input validation, live API/WebSocket target metadata validation, negative Rust tests for invalid pairings and missing inputs, and a wrapper-level rejection for `--stimulus` until a bounded active fault route exists.

## Skipped Issues

None.

## Verification

- `node -e "JSON.parse(require('fs').readFileSync('docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/live-api-websocket-telemetry/allow-live-telemetry.json','utf8'))"`
- `bazel run //tools/parity:report -- safety-allow --manifest docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/live-api-websocket-telemetry/allow-live-telemetry.json --surface live-api-websocket-telemetry --allowed-command 'scripts/phase14-live-telemetry.sh --manifest docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/live-api-websocket-telemetry/allow-live-telemetry.json --out-dir docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/live-api-websocket-telemetry'`
- `cargo test -p bitaxe-parity safety_allow`
- `scripts/phase20-failure-paths-test.sh`
- `bazel test //tools/parity:tests //scripts:phase20_failure_paths_test`
- Before each fix commit: `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, `cargo test --all-features`

***

_Fixed: 2026-07-03T23:56:21Z_
_Fixer: the agent (gsd-code-fixer)_
_Iteration: 1_

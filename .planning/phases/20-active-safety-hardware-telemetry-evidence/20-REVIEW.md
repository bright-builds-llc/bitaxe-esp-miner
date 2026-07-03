---
phase: 20-active-safety-hardware-telemetry-evidence
reviewed: 2026-07-03T23:41:47Z
depth: standard
files_reviewed: 40
files_reviewed_list:
  - docs/parity/checklist.md
  - docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/active-power-voltage.md
  - docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/active-power-voltage/power-telemetry/allow-power-telemetry.json
  - docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/active-power-voltage/power-telemetry/power-voltage.log
  - docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/active-power-voltage/voltage-control/allow-voltage-control.json
  - docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/active-power-voltage/voltage-control/power-voltage.log
  - docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/active-thermal-fan.md
  - docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/active-thermal-fan/fan-duty/allow-fan-duty-blocked.json
  - docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/active-thermal-fan/fan-duty/thermal-fan.log
  - docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/active-thermal-fan/thermal-read/allow-thermal-fan-read.json
  - docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/active-thermal-fan/thermal-read/thermal-fan.log
  - docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/evidence-contract.md
  - docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/failure-paths.md
  - docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/failure-paths/allow-failure-paths.json
  - docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/failure-paths/failure-paths.log
  - docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/live-api-websocket-telemetry.md
  - docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/live-api-websocket-telemetry/allow-live-telemetry.json
  - docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/live-api-websocket-telemetry/live-telemetry.log
  - docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/live-api-websocket-telemetry/websocket/api-ws-live.txt
  - docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/package-release-gate.md
  - docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/package-release-gate/bitaxe-ultra205-package.json
  - docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/package-release-gate/package-command.log
  - docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/package-release-gate/release-gate.log
  - docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/redaction-review.md
  - docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/runtime-display-input.md
  - docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/runtime-display-input/allow-display-input.json
  - docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/runtime-display-input/display-input.log
  - docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/safe-baseline.md
  - docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/safe-baseline/detect-ultra205.log
  - docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/safe-baseline/flash-command-evidence.json
  - docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/safe-baseline/flash-monitor.log
  - docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/self-test-watchdog-load.md
  - docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/self-test-watchdog-load/allow-self-test-watchdog-load.json
  - docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/self-test-watchdog-load/self-test-watchdog-load.log
  - docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/summary.md
  - docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/target-lock.json
  - scripts/BUILD.bazel
  - scripts/phase20-failure-paths-test.sh
  - scripts/phase20-failure-paths.sh
  - tools/parity/src/safety_allow.rs
findings:
  critical: 0
  warning: 2
  info: 0
  total: 2
status: issues_found
---

# Phase 20: Code Review Report

**Reviewed:** 2026-07-03T23:41:47Z
**Depth:** standard
**Files Reviewed:** 40
**Status:** issues_found

## Summary

Reviewed the Phase 20 evidence artifacts, allow manifests, failure-path shell wrapper/tests, and `tools/parity` safety allow validator at standard depth. Repo guidance from `AGENTS.md`, `AGENTS.bright-builds.md`, and the Bright Builds verification, testing, architecture, code-shape, and Rust standards materially informed this review.

The committed redaction posture is conservative: reviewed logs use redacted placeholders, the target lock remains blocked, and the evidence summary avoids active hardware-control promotion. The main issues are false-promotion risks in the safety allow layer: one current blocked live-telemetry manifest still carries a hardware-smoke projection claim, and the Rust validator does not enforce the surface-specific evidence contract strongly enough for future failure-path promotion.

Targeted verification run:

```text
bazel test //tools/parity:tests //scripts:phase20_failure_paths_test
Result: PASSED (cached)
```

## Warnings

### WR-01: Blocked Live Telemetry Manifest Claims Hardware-Smoke Projection

**File:** `docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/live-api-websocket-telemetry/allow-live-telemetry.json:12`

**Issue:** The allow manifest declares `claim_tier: api-websocket-projection` and `evidence_class: hardware-smoke` for `API-002`, `API-006`, `STAT-002`, `PWR-006`, `THR-001`, `THR-002`, and `SAFE-07`, but the corresponding log records `DEVICE_URL status: blocked - missing DEVICE_URL`, `api_telemetry_status: blocked`, and no WebSocket frame capture. The summary correctly classifies the pack as a deferred blocked boundary, so this manifest/log pair now contains conflicting evidence metadata. A downstream citation that keys off `safety_allow_status: passed` plus `hardware-smoke` could over-promote blocked live telemetry.

**Fix:** For the blocked Phase 20 run, make the manifest match the evidence boundary:

```json
{
  "surface": "live-api-websocket-telemetry",
  "claim_tier": "unsupported-pending",
  "evidence_class": "deferred",
  "allowed_inputs": {
    "device_url_source": "missing",
    "network_scan": "disabled",
    "reason": "no explicit DEVICE_URL or trusted origin-only target lock"
  }
}
```

Use `api-websocket-projection` plus `hardware-smoke` only when an explicit reachable `DEVICE_URL` or trusted raw origin-only target lock exists and the artifact contains the route/body/frame evidence required by `evidence-contract.md`.

### WR-02: Safety Allow Validation Does Not Enforce Surface-Specific Claim Contracts

**File:** `tools/parity/src/safety_allow.rs:217`

**Issue:** `validate_surface_and_claim` checks `surface` and `claim_tier` against independent global allow lists, then maps evidence class from claim tier alone. It does not enforce the Phase 20 matrix in `evidence-contract.md`, where `failure-paths` may only use `fault-stimulus`, `safe-unavailable`, or `unsupported-pending`, and active fault claims require a named stimulus, expected fault, abort condition, restore path, projection status, and final safe-state marker. As written, a manifest can pair `surface: failure-paths` with an unrelated globally allowed tier such as `read-only-observation`, or use `fault-stimulus` without validating `allowed_inputs.stimulus` and `allowed_inputs.expected_fault`. The tests do not cover these negative cases; `safety_allow_allows_non_active_claim_tiers_with_matching_evidence_class` even demonstrates unrelated surface/tier pairings passing.

**Fix:** Port the stricter pattern already used by `mining_allow.rs`: validate allowed claim tiers per surface, validate approved command shape per surface, and add surface-specific required input checks.

```rust
fn allowed_claim_tiers_for_surface(surface: &str) -> &'static [&'static str] {
    match surface {
        "failure-paths" => &["fault-stimulus", "safe-unavailable", "unsupported-pending"],
        "live-api-websocket-telemetry" => &[
            "api-websocket-projection",
            "read-only-observation",
            "safe-unavailable",
            "unsupported-pending",
        ],
        "parity-redaction" => &["parity-redaction", "unsupported-pending"],
        _ => &[],
    }
}

fn validate_failure_paths_scope(errors: &mut Vec<String>, manifest: &SafetyAllowManifest) {
    if manifest.surface != "failure-paths" || manifest.claim_tier != "fault-stimulus" {
        return;
    }

    require_string(errors, &manifest.allowed_inputs, "stimulus");
    require_string(errors, &manifest.allowed_inputs, "expected_fault");
    require_string(errors, &manifest.allowed_inputs, "restore_path");
}
```

Add tests that reject `failure-paths` plus `read-only-observation`, reject `failure-paths` `fault-stimulus` when `stimulus` or `expected_fault` is missing, and reject blocked live telemetry manifests that claim `api-websocket-projection` without an explicit target. Also make `phase20-failure-paths.sh` either reject `--stimulus` until active stimulus is implemented or include every accepted option in the exact `allowed_command` filter.

## Open Questions

None.

***

_Reviewed: 2026-07-03T23:41:47Z_
_Reviewer: the agent (gsd-code-reviewer)_
_Depth: standard_

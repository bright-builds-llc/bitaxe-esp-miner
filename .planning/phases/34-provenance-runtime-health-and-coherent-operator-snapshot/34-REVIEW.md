---
status: issues_found
phase: 34-provenance-runtime-health-and-coherent-operator-snapshot
reviewed: "2026-07-15T16:41:58Z"
depth: standard
files_reviewed: 23
files_reviewed_list:
  - crates/bitaxe-api/src/build_identity.rs
  - crates/bitaxe-api/src/operator_snapshot.rs
  - crates/bitaxe-api/src/platform_identity.rs
  - crates/bitaxe-api/src/runtime_projection.rs
  - crates/bitaxe-api/src/snapshot.rs
  - crates/bitaxe-api/src/wire.rs
  - crates/bitaxe-core/src/runtime_health.rs
  - firmware/bitaxe/build.rs
  - firmware/bitaxe/src/boot_evidence.rs
  - firmware/bitaxe/src/main.rs
  - firmware/bitaxe/src/platform_identity.rs
  - firmware/bitaxe/src/runtime_health_adapter.rs
  - firmware/bitaxe/src/runtime_snapshot.rs
  - firmware/bitaxe/src/safety_adapter/watchdog.rs
  - scripts/build-identity-pathspecs.txt
  - scripts/build-identity-status.sh
  - scripts/build_identity.bzl
  - scripts/package-firmware.sh
  - tools/flash/src/main.rs
  - tools/parity/src/operator_evidence.rs
  - tools/parity/src/operator_snapshot_evidence.rs
  - tools/parity/src/phase34_source_guard.rs
  - tools/xtask/src/package_manifest.rs
findings:
  critical: 1
  warning: 2
  info: 0
  total: 3
---

# Phase 34 Code Review

## Summary

The Phase 34 implementation establishes a strong typed foundation for build provenance, platform availability, runtime health, and operator snapshots, but three correctness defects remain. The most serious defect allows the image selected for normal factory flashing to differ from the OTA image whose embedded identity is admitted. Two runtime defects can make a healthy safety loop appear permanently unhealthy and can publish retained snapshot revisions out of order.

The review covered all 23 requested files, the relevant HTTP/WebSocket call chain, Phase 34 requirements and summaries, and the applicable Bright Builds architecture, code-shape, verification, testing, and Rust standards. No hardware, network, flash, monitor, erase, or credential-bearing operation was performed.

## Critical Issues

### CR-01: The factory image actually flashed is not bound to the OTA image whose identity is admitted

**Evidence:** `tools/flash/src/main.rs:1017-1071`, `tools/flash/src/main.rs:1171-1185`, and `tools/flash/src/main.rs:3371-3380`

`validate_package_identity` verifies the manifest-declared digests independently, then searches only the declared OTA artifact for the current source commit, version label, and build-identity SHA bytes. Normal package selection nevertheless prefers `factory_merged_image` and sends that separate artifact to `espflash write-bin`. There is no validation that the app region embedded in the factory image equals the admitted OTA image. The test fixture makes the gap explicit: it constructs an unrelated `b"synthetic factory package"`, records its digest in the manifest, and treats the package as admissible.

Consequently, a self-consistent v3 manifest can pair a marker-bearing current-source OTA artifact with a different factory artifact, pass exact-package admission, and flash the different bytes. This defeats the pre-port-resolution provenance gate and creates a device/data-loss risk because the admitted artifact is not necessarily the artifact written to hardware. The OTA substring checks are also not a structural app-descriptor validation, so coincidental or injected marker bytes can satisfy them.

**Required fix:** Authenticate the selected flash candidate, not merely a sibling artifact. For a merged factory image, parse the ESP image layout and require its app partition bytes to match the admitted OTA image exactly before any port resolution or hardware access. Parse the application descriptor at its defined location rather than scanning arbitrary bytes for marker substrings. Reject duplicate or ambiguous artifact kinds. Add regression tests that mutate the factory app region, update the factory digest in the manifest, and prove admission still fails.

## Warnings

### WR-01: The safety supervisor stops advancing its checkpoint after the first yield decision

**Evidence:** `firmware/bitaxe/src/safety_adapter/watchdog.rs:48-72` and `crates/bitaxe-safety/src/watchdog.rs:62-87`

Every loop iteration passes `elapsed_ms = 25` and `consecutive_steps = 4`; the supervisor therefore returns `YieldNow` because the resulting 100 ms meets the yield interval. The first call logs the yield and records a checkpoint. On every later call, the `logged_yield` guard returns from `run_supervisor_step` before `runtime_health_adapter::record_checkpoint` executes. The `safety_supervisor` sequence therefore remains at one and its age eventually crosses the 1.5-second stale and 5-second unhealthy thresholds even though the loop is still running.

This violates the HLT-02/HLT-04 requirement that recurring subsystem checkpoints represent live forward progress and that age-based unhealthy state indicate an actual stall.

**Required fix:** Suppress only the duplicate log message; do not return before checkpoint recording. Always advance the checkpoint after a non-restart decision. Add a regression test that executes at least two supervisor steps and asserts that the checkpoint sequence advances on each successful iteration and remains healthy across the configured age thresholds.

### WR-02: Concurrent snapshot captures can retain and expose revisions in decreasing order

**Evidence:** `firmware/bitaxe/src/runtime_snapshot.rs:54-82`, `firmware/bitaxe/src/runtime_snapshot.rs:96-105`, `firmware/bitaxe/src/runtime_snapshot.rs:277-297`, and `firmware/bitaxe/src/runtime_snapshot.rs:330-355`

Each capture reserves its revision under `SNAPSHOT_SEQUENCE`, releases that lock, performs collection, and only later appends the completed snapshot to the retained log. The HTTP handler and WebSocket cadence execute captures from separate contexts, so capture A can reserve revision 1 and stall while capture B reserves revision 2 and completes first. A can then complete and append revision 1 after revision 2. The retained chronology and operator-visible completion order regress even though reservation order was monotonic; `tools/parity/src/operator_snapshot_evidence.rs:299-319` correctly treats that ordering as invalid evidence.

This breaks the OBS-06 coherent, monotonic operator-snapshot contract and can make valid runtime evidence fail nondeterministically.

**Required fix:** Make revision assignment and publication one ordered authority. Either serialize reserve/collect/retain for completed captures, or collect unnumbered candidate facts and atomically assign the revision immediately before publishing and retaining them. Add a concurrency regression test that deliberately completes two captures in reverse collection order and asserts that retained and projected revisions never decrease.

## Strengths

- Build provenance is centralized into typed models with explicit clean/dirty status, schema checks, and fail-closed current-source admission paths.
- Platform identity uses explicit availability states instead of inventing values when ESP-IDF data is unavailable.
- Runtime-health evaluation is a passive functional core with clear stale/unhealthy thresholds and redacted checkpoint categories.
- Operator snapshot schemas consistently carry session identity, revision, capture time, health, platform identity, and provenance across projections.
- Phase source guards and parity validators provide useful regression pressure against split operator truth and evidence regressions.

## Conclusion

**Status: issues found.** CR-01 blocks reliance on the exact-package admission gate because the bytes admitted are not necessarily the bytes flashed. WR-01 blocks the intended recurring runtime-health semantics for the safety supervisor. WR-02 blocks a deterministic monotonic-revision guarantee under the production concurrency model. Address these findings and add the specified regression coverage before treating Phase 34 provenance, HLT-02/HLT-04, or OBS-06 as complete.

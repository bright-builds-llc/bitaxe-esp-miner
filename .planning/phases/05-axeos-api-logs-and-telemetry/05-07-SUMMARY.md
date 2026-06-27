---
phase: 05-axeos-api-logs-and-telemetry
plan: 07
subsystem: parity-evidence
tags: [rust, parity, axeos, api, static-assets, evidence]
requires:
  - 05-01
  - 05-02
  - 05-03
  - 05-04
  - 05-05
  - 05-06
provides:
  - API compare parity check with schema, captured-response, static-route, and firmware-smoke evidence labels
  - AxeOS route usage fixture covering Phase 05 administration routes and Phase 7-owned static/update boundaries
  - Phase 05 parity evidence document and checklist updates that avoid unsupported hardware, OTA, and static packaging claims
affects:
  - tools/parity
  - docs/parity/checklist.md
  - docs/parity/evidence/phase-05-axeos-api-logs-and-telemetry.md
tech-stack:
  added:
    - tools/parity depends on bitaxe-api for the Rust route manifest
  patterns:
    - Parity tooling compares against the exported Rust route manifest instead of duplicating route lists silently
    - Evidence is reported by type so schema, captured response, static route, and firmware smoke cannot be conflated
key-files:
  created:
    - tools/parity/src/api_compare.rs
    - tools/parity/fixtures/api/phase05-required-routes.json
    - tools/parity/fixtures/api/axeos-route-usage.json
    - docs/parity/evidence/phase-05-axeos-api-logs-and-telemetry.md
  modified:
    - Cargo.lock
    - MODULE.bazel.lock
    - tools/parity/Cargo.toml
    - tools/parity/BUILD.bazel
    - tools/parity/src/main.rs
    - docs/parity/checklist.md
key-decisions:
  - Keep OpenAPI checks narrow and dependency-free rather than adding a deprecated YAML/OpenAPI parser dependency.
  - Use bitaxe-api's exported phase05_routes() as the Rust route manifest source of truth for API compare checks.
  - Record AxeOS static route compatibility separately from Phase 7-owned recovery/static packaging and OTA/OTAWWW success.
requirements-completed: [API-01, API-09, API-10]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 5-2026-06-27T18-02-49
generated_at: 2026-06-27T22:02:52Z
completed: 2026-06-27
duration: 16m57s
---

# Phase 05 Plan 07: API Compare and Static Evidence Summary

API comparison evidence now proves Phase 05 route/schema coverage, captured response fixture coverage, and AxeOS static route usage compatibility while keeping hardware smoke, static packaging, recovery, SPIFFS, OTA, and OTAWWW claims out of Phase 05 verified scope.

## What Changed

Task 1 added `tools/parity` API comparison support. The new `api-compare` mode reads the pinned upstream OpenAPI contract, the checked-in Phase 05 route/property manifest, the AxeOS route usage fixture, and the Rust API route manifest from `bitaxe-api`. It reports four evidence labels independently: `schema`, `captured-response`, `static-route`, and `firmware-smoke`.

Task 2 added the Phase 05 evidence record and updated the parity checklist. The checklist now advances only rows backed by schema, fixture, API compare, or existing unit evidence. Live Ultra 205 API smoke remains explicitly not run. Safety-critical and hardware-control rows remain unverified, and Phase 7-owned OTA/static/recovery packaging remains deferred.

## Task Commits

| Task | Name | Commit | Files |
| --- | --- | --- | --- |
| 1 | Add API compare and static AxeOS route usage checks | `2c5ee77` | `tools/parity/src/api_compare.rs`, `tools/parity/src/main.rs`, `tools/parity/Cargo.toml`, `tools/parity/BUILD.bazel`, `tools/parity/fixtures/api/phase05-required-routes.json`, `tools/parity/fixtures/api/axeos-route-usage.json`, `Cargo.lock`, `MODULE.bazel.lock` |
| 2 | Record Phase 05 parity evidence and checklist status | `4ccf838` | `docs/parity/checklist.md`, `docs/parity/evidence/phase-05-axeos-api-logs-and-telemetry.md` |

## Verification

| Command | Result | Notes |
| --- | --- | --- |
| `bazel test //tools/parity:tests --test_filter=api_compare` | Passed | Focused API compare tests pass after implementation. |
| `bazel run //tools/parity:report -- api-compare` | Passed | Reported `validation_errors: none` with schema, captured-response, static-route, and firmware-smoke labels. |
| `bazel test //tools/parity:tests && bazel test //crates/bitaxe-api:tests` | Passed | Task 1 quick verification. |
| `bazel build //firmware/bitaxe:firmware` | Passed | Firmware ELF target built successfully. |
| `just parity` | Passed | Parity report completed with `validation_errors: none`. |
| `bazel test //tools/parity:tests && just parity` | Passed | Task 2 quick verification. |
| `just test` | Passed | Full repo Bazel test surface passed after both task commits. |
| `cargo fmt --all` | Passed | Run before each task commit. |
| `cargo clippy --all-targets --all-features -- -D warnings` | Passed | Run before each task commit. |
| `cargo build --all-targets --all-features` | Passed | Run before each task commit. |
| `cargo test --all-features` | Passed | Run before each task commit. |

## Decisions Made

- The API compare implementation uses explicit OpenAPI text checks rather than adding a YAML/OpenAPI parser dependency. This keeps the parity tool dependency surface small and avoids a parser choice that is not needed for the narrow Phase 05 route/property contract.
- The Rust route manifest comes from `bitaxe_api::phase05_routes()`. This makes missing firmware route-shell coverage visible to parity tests instead of letting tool fixtures drift away from implemented routes.
- OTA, OTAWWW, `/recovery`, and static fallback are represented in fixtures as boundaries, not Phase 05 success. This lets Phase 05 prove administrative API compatibility without claiming Phase 7 release packaging or unsafe update behavior.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Adjusted API compare RED test fixture loading**

- **Found during:** Task 1
- **Issue:** The initial failing test attempted to `include_str!` the upstream OpenAPI file from outside the Bazel package, which is not available inside the Bazel test sandbox.
- **Fix:** Unit tests now use a minimal inline OpenAPI fixture while runtime `api-compare` still reads the real pinned upstream OpenAPI path from the workspace.
- **Files modified:** `tools/parity/src/api_compare.rs`, `tools/parity/BUILD.bazel`
- **Commit:** `2c5ee77`

### Process Adjustments

- The TDD RED state was verified but not committed separately because the repo-level Rust pre-commit gate requires the full formatting, clippy, build, and test suite to pass before every git commit.
- The plan context referenced `reference/esp-miner/main/http_server/filesystem.c`; the actual upstream source for filesystem/static fallback evidence is `reference/esp-miner/main/filesystem.c`.

## Known Stubs

None. Empty arrays in the API/static JSON fixtures are intentional assertions for routes with no required response-property list or unsupported Phase 7-owned success surface; they do not feed a UI and are not placeholder data.

## Threat Flags

None. The new local file reads are the planned parity trust boundary for the pinned upstream OpenAPI and checked-in fixture data. No network endpoint, authentication path, hardware control, or new firmware runtime file access was introduced.

## Residual Risk

- Live Ultra 205 API and WebSocket smoke was not run in this plan, so firmware-smoke evidence remains `not-run`.
- Static asset packaging, SPIFFS image behavior, `/recovery`, OTA, OTAWWW, and release artifacts remain Phase 7-owned.
- Power, thermal, fan, voltage, ASIC initialization, and other safety-critical hardware behavior remain outside Phase 05 verified scope.

## Self-Check: PASSED

- Found summary file: `.planning/phases/05-axeos-api-logs-and-telemetry/05-07-SUMMARY.md`
- Found created files: `tools/parity/src/api_compare.rs`, `tools/parity/fixtures/api/phase05-required-routes.json`, `tools/parity/fixtures/api/axeos-route-usage.json`, `docs/parity/evidence/phase-05-axeos-api-logs-and-telemetry.md`
- Found task commits: `2c5ee77`, `4ccf838`
- Focused stub scan over touched source, fixture, and evidence files returned no matches.

---
phase: 05-axeos-api-logs-and-telemetry
plan: 02
subsystem: api
tags: [rust, serde-json, axeos, settings, nvs, persistence]

requires:
  - phase: 05-axeos-api-logs-and-telemetry
    provides: "Plan 05-01 API crate foundation, Serde dependencies, and fixture wiring"
  - phase: 02-ultra-205-config-and-nvs-model
    provides: "SettingsPatch, apply_settings_patch, NvsSnapshot, and reload semantics"
provides:
  - "Host-testable AxeOS PATCH /api/system JSON parser and public error mapping"
  - "Schema-driven settings write planning through bitaxe-config with unknown-field tolerance"
  - "Internal persist-then-reload adapter contract with typed firmware failure reasons"
  - "Best-effort hostname live-apply effect emitted only after persistence success"
affects:
  - 05-05-firmware-route-websocket-settings-log-adapters
  - phase-06-safety
  - api-settings-route
  - firmware-nvs-adapter

tech-stack:
  added: []
  patterns:
    - "API settings parsing converts JSON at the boundary, then delegates validation/write planning to bitaxe-config"
    - "Settings route success is modeled as validate, write all, commit, reload, then empty public success"
    - "Firmware-visible adapter failures stay typed while public settings errors remain generic"

key-files:
  created:
    - crates/bitaxe-api/src/settings.rs
    - crates/bitaxe-api/fixtures/api/settings-patch-cases.json
  modified:
    - crates/bitaxe-api/src/lib.rs
    - crates/bitaxe-api/BUILD.bazel

key-decisions:
  - "Keep bitaxe-config as the only settings validation authority; bitaxe-api only parses JSON, ignores unknown fields, and maps public errors."
  - "Require write, commit, and reload completion before the settings route can produce an empty public success response."
  - "Represent hostname live apply as a best-effort post-persistence effect, not as a validation or persistence prerequisite."

patterns-established:
  - "AcceptedSettingsPatch exposes inert NVS writes and requested hostname data without performing I/O."
  - "SettingsPersistencePlan and SettingsPersistenceAdapter isolate firmware storage effects behind a host-testable contract."

requirements-completed: [API-02, API-03]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 5-2026-06-27T18-02-49
generated_at: 2026-06-27T20:08:46Z

duration: 10 min
completed: 2026-06-27
---

# Phase 05 Plan 02: Settings PATCH Planning Summary

**AxeOS settings PATCH parser with schema-driven write planning and persist-then-reload adapter sequencing**

## Performance

- **Duration:** 10 min
- **Started:** 2026-06-27T19:58:21Z
- **Completed:** 2026-06-27T20:08:46Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments

- Added `bitaxe_api::settings` with `plan_settings_patch_body` and `plan_settings_patch_value` for AxeOS-compatible settings PATCH request planning.
- Reused `bitaxe-config` settings schema, validation, and legacy mirror writes so API code does not maintain a second settings table.
- Added `SettingsPersistencePlan`, `SettingsPersistenceAdapter`, and `execute_settings_persistence_plan` to prove validate/write/commit/reload ordering before public success.
- Added fixture and unit coverage for valid writes, unknown-field tolerance, invalid-field atomic rejection, generic public errors, secret-safe diagnostics, persistence failures, and hostname live-apply effects.

## Task Commits

1. **Task 1: Parse PATCH JSON into schema-driven settings decisions** - `f0fca86` (feat)
2. **Task 2: Prove persist-then-reload adapter ordering with fake storage** - `ff2e72f` (feat)

## Files Created/Modified

- `crates/bitaxe-api/src/settings.rs` - Pure settings PATCH parser, public error mapping, accepted write plan, persistence adapter contract, and fake-adapter tests.
- `crates/bitaxe-api/fixtures/api/settings-patch-cases.json` - Representative success, unknown-field, invalid-field, and public-error fixture cases.
- `crates/bitaxe-api/src/lib.rs` - Public exports for settings request planning and persistence sequencing.
- `crates/bitaxe-api/BUILD.bazel` - Adds `src/settings.rs` to the API crate source set.

## Decisions Made

- API settings parsing delegates all known-field validation and write construction to `bitaxe-config::apply_settings_patch`.
- Malformed JSON and non-object JSON map to public `Invalid JSON`; invalid known fields and persistence failures map to generic `Wrong API input`.
- Hostname live apply is represented as `BestEffortApplyHostname` after persistence success and cannot cause validation or persistence to succeed.

## Deviations from Plan

### Process Adjustments

**AGENTS.md precedence over TDD RED commits**
- **Found during:** Task 1 and Task 2
- **Issue:** The plan requested TDD RED flow, but repo instructions require `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, and `cargo test --all-features` to pass before every Rust-project commit.
- **Adjustment:** Ran the failing RED test for each task, then committed only the passing implementation after the required Rust pre-commit checks.
- **Impact:** No behavior change; preserves higher-priority repo policy.

**Total deviations:** 0 auto-fixed implementation issues, 1 process adjustment.
**Impact on plan:** Scope stayed within the planned pure settings parser and adapter contract.

## Issues Encountered

- `crates/bitaxe-api/src/settings.rs` is test-heavy because parser and adapter-contract tests live beside the pure module. This is acceptable for the current cohesive settings boundary, but it is a refactor candidate if later plans add more settings route-shell code.

## Known Stubs

None.

## Threat Flags

None. The HTTP body parsing and settings adapter boundary are the planned trust surfaces in the plan threat model; no unplanned network endpoint, auth path, file access pattern, schema change, or hardware-control effect was introduced.

## Authentication Gates

None.

## Verification

- `bazel test //crates/bitaxe-api:tests //crates/bitaxe-config:tests --test_filter=settings` - passed
- `bazel test //crates/bitaxe-api:tests --test_filter=settings` - passed
- `bazel test //crates/bitaxe-api:tests //crates/bitaxe-config:tests` - passed
- `cargo fmt --all` - passed before each task commit
- `cargo clippy --all-targets --all-features -- -D warnings` - passed before each task commit
- `cargo build --all-targets --all-features` - passed before each task commit
- `cargo test --all-features` - passed before each task commit
- `just test` - passed
- `git status --short reference/esp-miner` - clean output

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Ready for Plan 05-03. The API crate now has pure settings PATCH request planning and an internal storage adapter contract ready for the firmware route shell in Plan 05-05.

## Self-Check: PASSED

- Found `.planning/phases/05-axeos-api-logs-and-telemetry/05-02-SUMMARY.md`
- Found `crates/bitaxe-api/src/settings.rs`
- Found `crates/bitaxe-api/fixtures/api/settings-patch-cases.json`
- Found task commit `f0fca86`
- Found task commit `ff2e72f`
- Reference implementation remains unmodified.

---
*Phase: 05-axeos-api-logs-and-telemetry*
*Completed: 2026-06-27*

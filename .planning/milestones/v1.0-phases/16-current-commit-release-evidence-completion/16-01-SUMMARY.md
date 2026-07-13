---
phase: 16-current-commit-release-evidence-completion
plan: "01"
subsystem: release-evidence
tags:
  - rust
  - bazel
  - shell
  - parity
  - release-evidence
  - ultra205
requires:
  - phase: 13-final-ultra-205-release-evidence
    provides: Historical HTTP, OTA, and recovery helper patterns for Phase 16 wrappers
  - phase: 15-bm1366-mining-evidence-completion
    provides: Latest redaction and evidence ledger conventions
provides:
  - Current-commit release evidence validator for package, flash, path, and redaction gates
  - Phase 16 HTTP/static/recovery/API/OTA route smoke wrapper with explicit DEVICE_URL handling
  - Phase 16 failed-update, interrupted-OTA, and large-erase recovery wrapper with shared live-action gate
  - Phase 16 evidence README and redaction review scaffold
affects:
  - phase-16-release-evidence
  - parity-checklist-promotion
  - release-docs
  - recovery-regression
tech-stack:
  added: []
  patterns:
    - Rust parity validator with pure validation core and CLI environment adapter
    - Phase-owned shell wrappers using fake-command regression tests
    - Explicit current-commit gate before live or destructive evidence can support claims
key-files:
  created:
    - tools/parity/src/release_evidence.rs
    - scripts/phase16-http-static-smoke.sh
    - scripts/phase16-http-static-smoke-test.sh
    - scripts/phase16-recovery-regression.sh
    - scripts/phase16-recovery-regression-test.sh
    - docs/parity/evidence/phase-16-current-commit-release-evidence-completion/README.md
    - docs/parity/evidence/phase-16-current-commit-release-evidence-completion/redaction-review.md
  modified:
    - tools/parity/src/main.rs
    - tools/parity/BUILD.bazel
    - scripts/BUILD.bazel
key-decisions:
  - "Keep Phase 16 evidence helpers phase-owned instead of mutating Phase 13 historical evidence scripts."
  - "Require package source_commit to equal current git HEAD before release evidence can support current-commit claims."
  - "Gate every allowed failed-update, interrupted-OTA, and large-erase action with detector, board-info, current manifest, package artifacts, abort conditions, recovery steps, and safe-state markers."
patterns-established:
  - "Current-commit release evidence must be machine-checked before docs or checklist promotion."
  - "Live HTTP helpers must require explicit DEVICE_URL and record blocked evidence instead of scanning."
  - "Fault/destructive helpers must default to pending and require allow flags plus a shared hardware identity gate."
requirements-completed:
  - API-09
  - REL-01
  - REL-02
  - REL-03
  - REL-04
  - REL-07
  - REL-08
  - EVD-05
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 16-2026-07-01T12-36-46
generated_at: 2026-07-01T13:50:45Z
duration: 23min
completed: 2026-07-01
---

# Phase 16 Plan 01: Wave 0 Release Evidence Gate Summary

**Current-commit release evidence gates for package identity, explicit HTTP/OTA probing, destructive recovery preflight, and redaction review.**

## Performance

- **Duration:** 23 min
- **Started:** 2026-07-01T13:27:42Z
- **Completed:** 2026-07-01T13:50:45Z
- **Tasks:** 3
- **Files modified:** 10

## Accomplishments

- Added `release-evidence` validation to reject stale package manifests, mismatched flash evidence, untrusted wrapper output, out-of-root evidence paths, and missing redaction approval when required.
- Added Phase 16 HTTP/static smoke coverage for `/`, gzip static assets, missing static fallback, `/recovery`, API coexistence, WebSocket no-upgrade routes, `/api/system/OTA`, and the deferred OTAWWW gap.
- Added a Phase 16 recovery wrapper that keeps failed-update, interrupted-OTA, and large-erase actions pending by default and requires a shared live-action gate before any allowed mutation.
- Created Phase 16 evidence README and redaction-review scaffold before live logs or API bodies are cited.

## Task Commits

1. **Task 1: Add current-commit release evidence validator and scaffold** - `9e14e1c` (feat)
2. **Task 2: Add Phase 16 HTTP/static/recovery route probe wrapper** - `22f12c6` (feat)
3. **Task 3: Add Phase 16 recovery/destructive preflight wrapper** - `a4b69af` (feat)

## Files Created/Modified

- `tools/parity/src/release_evidence.rs` - Current-commit release evidence parser, validator, renderer, and tests.
- `tools/parity/src/main.rs` - `release-evidence` CLI command and local git HEAD adapter.
- `tools/parity/BUILD.bazel` - Parity target source registration for the validator.
- `scripts/phase16-http-static-smoke.sh` - Explicit-URL Phase 16 HTTP/static/recovery/API/OTA probe helper.
- `scripts/phase16-http-static-smoke-test.sh` - Fake-curl tests for route coverage, blockers, and redaction.
- `scripts/phase16-recovery-regression.sh` - Phase 16 recovery/fault/destructive wrapper with shared live-action gate.
- `scripts/phase16-recovery-regression-test.sh` - Fake-command tests for pending behavior, gate failures, exact commands, timeout evidence, and redaction.
- `scripts/BUILD.bazel` - Bazel registrations for Phase 16 shell binaries and tests.
- `docs/parity/evidence/phase-16-current-commit-release-evidence-completion/README.md` - Phase 16 evidence command order and promotion rules.
- `docs/parity/evidence/phase-16-current-commit-release-evidence-completion/redaction-review.md` - Redaction review template with `redaction_status: pending`.

## Decisions Made

- Kept Phase 16 wrappers separate from Phase 13 scripts so historical evidence behavior remains stable.
- Used the repo’s current git HEAD as the release-candidate identity source for `release-evidence`.
- Treated OTAWWW as deferred unless whole-www update and interrupted-update hardware-regression evidence exists.

## Deviations from Plan

### Process Adjustments

**1. TDD red state was not committed separately**
- **Found during:** Task 1
- **Issue:** The plan requested TDD red/green commits, but repo Rust rules require `cargo fmt`, `cargo clippy`, `cargo build`, and `cargo test` to pass before every commit. A failing RED commit would violate `AGENTS.md`.
- **Adjustment:** Wrote and ran the failing release-evidence tests first, confirmed they failed on missing implementation, then committed only the passing implementation after full verification.
- **Files modified:** `tools/parity/src/release_evidence.rs`, `tools/parity/src/main.rs`, `tools/parity/BUILD.bazel`
- **Verification:** `cargo test -p bitaxe-parity --all-features release_evidence` failed before implementation and passed after implementation.
- **Committed in:** `9e14e1c`

**Total deviations:** 1 process adjustment. No scope expansion beyond the plan.

## Issues Encountered

None beyond the TDD commit-order adjustment above.

## Verification

Passed:

- `cargo test -p bitaxe-parity --all-features release_evidence`
- `bazel test //tools/parity:tests --test_filter=release_evidence`
- `bazel test //scripts:phase16_http_static_smoke_test //scripts:phase16_recovery_regression_test`
- `just parity` with `validation_errors: none`
- `cargo fmt --all`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo build --all-targets --all-features`
- `cargo test --all-features`
- `just test` after Task 3 commit, with firmware package source commit stamped as `a4b69afb84c0`

## Known Stubs

None. The stub scan only found shell parser-state empty variable initializers; no UI-rendered empty/mock data or placeholder evidence was introduced.

## Threat Flags

None. The new release validator, HTTP helper, recovery wrapper, and redaction scaffold match the planned threat model mitigations T-16-01-01 through T-16-01-05.

## User Setup Required

None. Live hardware/network evidence remains intentionally gated behind explicit `DEVICE_URL`, `port`, current manifest/image paths, and allow flags in later Phase 16 plans.

## Next Phase Readiness

Plan 16-02 can build current-commit package and serial evidence on top of the release-evidence validator. Later live HTTP, OTA, and recovery plans now have phase-owned wrappers that record blocked evidence when prerequisites are absent and refuse destructive work until the Ultra 205 gate passes.

## Self-Check: PASSED

- Summary file exists at `.planning/phases/16-current-commit-release-evidence-completion/16-01-SUMMARY.md`.
- Task commits found: `9e14e1c`, `22f12c6`, `a4b69af`.
- Required lifecycle fields are present: `lifecycle_mode: yolo` and `phase_lifecycle_id: 16-2026-07-01T12-36-46`.
- Summary contains only the opening and closing YAML frontmatter delimiters.

*Phase: 16-current-commit-release-evidence-completion*
*Completed: 2026-07-01*

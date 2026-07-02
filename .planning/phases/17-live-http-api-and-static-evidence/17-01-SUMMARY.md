---
phase: 17-live-http-api-and-static-evidence
plan: "01"
subsystem: evidence-tooling
tags:
  - bash
  - node
  - websocket
  - redaction
  - bazel
requires:
  - phase: 16-current-commit-release-evidence-completion
    provides: Current package/flash identity and blocked live HTTP evidence boundaries
provides:
  - Phase 17 explicit-target HTTP/static/API smoke helper
  - Bounded WebSocket frame capture helper for /api/ws/live and /api/ws
  - Bazel test wiring for Phase 17 helper validation
  - Phase 17 evidence README and pending redaction-review scaffold
affects:
  - phase-17-live-evidence
  - parity-evidence
  - release-docs
tech-stack:
  added:
    - Node global WebSocket helper
  patterns:
    - Explicit origin-only DEVICE_URL gates
    - Redacted micro-artifacts for live evidence
    - Separate WebSocket no-upgrade route coexistence from frame proof
key-files:
  created:
    - scripts/phase17-live-http-api-smoke.sh
    - scripts/phase17-live-http-api-smoke-test.sh
    - scripts/phase17-websocket-capture.mjs
    - docs/parity/evidence/phase-17-live-http-api-and-static-evidence/README.md
    - docs/parity/evidence/phase-17-live-http-api-and-static-evidence/redaction-review.md
  modified:
    - scripts/BUILD.bazel
key-decisions:
  - "Keep raw curl output in temporary files and write only selected headers plus redacted body/error artifacts."
  - "Use Node global WebSocket with fake modes for deterministic no-network tests instead of adding dependencies."
  - "Start Phase 17 redaction review as pending with absent-artifact tracking before live evidence is cited."
patterns-established:
  - "Target lock: sanitized explicit target/package/flash identity is written only after preflight passes."
  - "WebSocket capture: /api/ws/live requires frames to pass, while /api/ws open-without-frame remains pending."
requirements-completed:
  - API-09
  - REL-01
  - REL-07
  - EVD-05
generated_by: gsd-execute-plan
lifecycle_mode: interactive
phase_lifecycle_id: 17-2026-07-02T01-09-48
generated_at: 2026-07-02T02:44:04Z
duration: 13 min
completed: 2026-07-02
---

# Phase 17 Plan 01: Live HTTP API And Static Evidence Wave 0 Summary

**Explicit-target Phase 17 HTTP/API route evidence helper, bounded WebSocket capture, and redaction scaffold are ready before live collection.**

## Performance

- **Duration:** 13 min
- **Started:** 2026-07-02T02:30:49Z
- **Completed:** 2026-07-02T02:44:04Z
- **Tasks:** 3
- **Files modified:** 6

## Accomplishments

- Created `scripts/phase17-live-http-api-smoke.sh` with origin-only `DEVICE_URL` validation, package/flash identity blocking, route-specific D-08 checks, sanitized target-lock output, and redacted artifacts.
- Created `scripts/phase17-websocket-capture.mjs` with allowlisted `/api/ws/live` and `/api/ws` capture, bounded duration/frame limits, fake-mode tests, and separate pending/pass semantics.
- Added the Phase 17 evidence README and pending redaction-review matrix before any live artifacts are cited.

## Task Commits

Each task was committed atomically. TDD tasks include RED and GREEN commits:

1. **Task 1 RED: Phase 17 HTTP smoke tests** - `4029439` (`test`)
2. **Task 1 GREEN: Phase 17 HTTP smoke helper** - `39cb1a1` (`feat`)
3. **Task 2 RED: Phase 17 WebSocket capture tests** - `7da278c` (`test`)
4. **Task 2 GREEN: Phase 17 WebSocket capture helper** - `6624476` (`feat`)
5. **Task 3: Phase 17 evidence scaffold** - `0a5957c` (`docs`)

## Files Created/Modified

- `scripts/phase17-live-http-api-smoke.sh` - Phase-owned HTTP/static/API smoke helper with explicit target and identity gates.
- `scripts/phase17-live-http-api-smoke-test.sh` - Fake-curl and fake-WebSocket tests covering blocking, route coverage, redaction, and non-claim boundaries.
- `scripts/phase17-websocket-capture.mjs` - Bounded Node WebSocket capture helper with no npm dependency.
- `scripts/BUILD.bazel` - Adds `phase17_live_http_api_smoke` and `phase17_live_http_api_smoke_test`.
- `docs/parity/evidence/phase-17-live-http-api-and-static-evidence/README.md` - Evidence command order, gate rules, promotion rules, and non-claims.
- `docs/parity/evidence/phase-17-live-http-api-and-static-evidence/redaction-review.md` - Pending artifact matrix and redaction checklist.

## Decisions Made

- Raw curl output is used only for matching inside a temporary directory; committed/generated route artifacts are selected headers and redacted body/error snippets.
- WebSocket no-upgrade HTTP results remain route-coexistence-only; frame proof requires the separate Node helper.
- `redaction_status: pending` is intentional until live artifacts exist and are reviewed.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

- TDD RED runs failed as expected while the planned helper files were missing.
- During GREEN, a Bash readonly-name collision and an overly strict JSON fixture assertion were corrected before the Task 1 implementation commit.

## User Setup Required

None - no external service configuration required for this scaffolding plan.

## Known Stubs

None. `redaction_status: pending` is an intentional pre-live scaffold state and does not block this plan's goal.

## Verification

- `bash -n scripts/phase17-live-http-api-smoke.sh`
- `node --check scripts/phase17-websocket-capture.mjs`
- `bazel test //scripts:phase17_live_http_api_smoke_test`
- `rg -n "redaction_status: pending|absent - not cited|Explicit Non-Claims" docs/parity/evidence/phase-17-live-http-api-and-static-evidence/README.md docs/parity/evidence/phase-17-live-http-api-and-static-evidence/redaction-review.md`

## Next Phase Readiness

Ready for `17-02-PLAN.md`. The Phase 17 Wave 0 helper, bounded WebSocket capture, Bazel wiring, and redaction scaffold now exist, but live evidence still requires later phase plans with package, flash, explicit `DEVICE_URL`, and redaction review steps.

## Self-Check: PASSED

- Created files exist on disk.
- Task commits exist in git history.
- Summary frontmatter uses only the opening and closing YAML delimiters.

*Phase: 17-live-http-api-and-static-evidence*
*Completed: 2026-07-02*

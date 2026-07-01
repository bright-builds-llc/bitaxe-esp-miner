---
phase: 15-bm1366-mining-evidence-completion
plan: "04"
subsystem: parity-evidence
tags:
  - bm1366
  - mining
  - controlled-no-share
  - bounded-soak
  - redaction

requires:
  - phase: 15-03
    provides: package-backed BM1366 work/result diagnostic evidence and prerequisite safe-state markers
provides:
  - Controlled mining smoke wrapper with mining-allow gating and no-share fallback behavior
  - Bounded soak wrapper path with duration and abort-condition evidence
  - Redaction-reviewed mining-smoke and bounded-soak evidence artifacts
  - Detector logs and allow manifests tied to board 205, source commits, and package identity
affects:
  - phase-15-final-ledger
  - parity-checklist
  - bm1366-mining-evidence

tech-stack:
  added:
    - Bash wrapper for controlled mining smoke and bounded soak evidence
    - Node WebSocket capture helper using global WebSocket
  patterns:
    - Evidence wrappers validate mining-allow manifests before emitting smoke or soak claims
    - Missing live pool or DEVICE_URL prerequisites produce controlled no-share evidence instead of overclaiming live mining

key-files:
  created:
    - scripts/phase15-controlled-mining.sh
    - scripts/phase15-controlled-mining-test.sh
    - scripts/phase15-websocket-capture.mjs
    - docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/mining-smoke.md
    - docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/mining-smoke/allow-mining-smoke.json
    - docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/mining-smoke/mining-smoke.log
    - docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/mining-smoke/detect-ultra205.log
    - docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/bounded-soak.md
    - docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/bounded-soak/allow-bounded-soak.json
    - docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/bounded-soak/bounded-soak.log
    - docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/bounded-soak/detect-ultra205.log
  modified:
    - scripts/BUILD.bazel
    - docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/redaction-review.md

key-decisions:
  - "Missing DEVICE_URL or pool credentials were treated as controlled no-share evidence, not as an execution failure or auth gate."
  - "Detector output was copied into both smoke and soak evidence directories so each pack is self-contained."
  - "The wrapper records pending API/WebSocket/watchdog status when live prerequisites are absent and avoids scanning or inferring device URLs."

patterns-established:
  - "Mining evidence commands must pass a mining-allow manifest with an exact allowed_command before writing smoke or soak artifacts."
  - "Evidence packs retain board, port, commits, package identity, conclusion, and redaction status while omitting live secrets and private endpoints."

requirements-completed:
  - STR-06
  - STR-07
  - SAFE-09
  - EVD-05
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 15-2026-07-01T02-07-59
generated_at: 2026-07-01T04:30:44Z

duration: 17m21s
completed: 2026-07-01
---

# Phase 15 Plan 04: Controlled Mining Evidence Summary

**Controlled no-share mining smoke and unsupported-pending bounded soak evidence with mining-allow gates, detector provenance, and redaction review**

## Performance

- **Duration:** 17m21s
- **Started:** 2026-07-01T04:13:23Z
- **Completed:** 2026-07-01T04:30:44Z
- **Tasks:** 2
- **Files modified:** 13

## Accomplishments

- Added `scripts/phase15-controlled-mining.sh`, its Bash test, and a bounded Node WebSocket capture helper.
- Registered the controlled mining wrapper and test in Bazel.
- Captured mining-smoke evidence as `controlled-no-share` because live pool and `DEVICE_URL` prerequisites were absent.
- Captured bounded-soak evidence as `unsupported-pending` with `duration_seconds=120`, abort conditions, and safe-stop status.
- Updated the Phase 15 redaction review to clear the `mining-smoke` and `bounded-soak` packs for scoped citation.

## Task Commits

Each task was committed atomically:

1. **Task 1: Add controlled mining smoke and soak wrappers** - `edd5f4f` (`feat`)
2. **Task 2: Run controlled mining smoke and bounded soak evidence** - `480aafd` (`docs`)

## Files Created/Modified

- `scripts/phase15-controlled-mining.sh` - Controlled smoke/soak wrapper with mining-allow validation, prerequisite checks, no-share fallback, and sanitized output.
- `scripts/phase15-controlled-mining-test.sh` - Regression coverage for missing manifest, allow failures, missing live prerequisites, bounded soak metadata, and WebSocket helper behavior.
- `scripts/phase15-websocket-capture.mjs` - Explicit-URL WebSocket capture helper with URL, IP, MAC, pool, token, and credential redaction.
- `scripts/BUILD.bazel` - Adds the wrapper and test target wiring.
- `docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/mining-smoke.md` - Controlled no-share mining smoke evidence summary.
- `docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/mining-smoke/allow-mining-smoke.json` - Mining-smoke allow manifest with exact command, board, port, package, and checklist row metadata.
- `docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/mining-smoke/mining-smoke.log` - Wrapper output for the mining-smoke run.
- `docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/mining-smoke/detect-ultra205.log` - Detector gate output for the smoke evidence pack.
- `docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/bounded-soak.md` - Unsupported-pending bounded soak evidence summary.
- `docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/bounded-soak/allow-bounded-soak.json` - Bounded-soak allow manifest with 120-second duration and abort conditions.
- `docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/bounded-soak/bounded-soak.log` - Wrapper output for the bounded-soak run.
- `docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/bounded-soak/detect-ultra205.log` - Detector gate output for the soak evidence pack.
- `docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/redaction-review.md` - Marks mining-smoke and bounded-soak packs as redaction-reviewed and passed.

## Decisions Made

- Missing `DEVICE_URL` and pool environment variables were treated as a controlled no-share condition, which matches the plan and avoids overclaiming live mining evidence.
- Detector logs were added to both evidence directories even though the plan file list did not enumerate them, because AGENTS.md requires hardware evidence to record detector output.
- The bounded soak pack remains `unsupported-pending` rather than `bounded-soak` because the live prerequisites needed for an actual soak were not present.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Split invalid Cargo multi-filter command**
- **Found during:** Task 1 (Add controlled mining smoke and soak wrappers)
- **Issue:** The plan listed `cargo test -p bitaxe-stratum --all-features mining_loop fake_pool queue`, but Cargo accepts one test filter in that position.
- **Fix:** Ran the intended filters separately: `mining_loop`, `fake_pool`, and `queue`.
- **Files modified:** None.
- **Verification:** All three filtered `bitaxe-stratum` test commands passed.
- **Committed in:** Not applicable, process-only verification deviation.

**2. [Rule 1 - Bug] Fixed wrapper allow-command variable collision**
- **Found during:** Task 2 (Run controlled mining smoke and bounded soak evidence)
- **Issue:** The first smoke run logged `local: allowed_command: readonly variable` because `run_mining_allow` shadowed a readonly global name.
- **Fix:** Renamed the local variable to `expected_allowed_command` and regenerated clean smoke evidence.
- **Files modified:** `scripts/phase15-controlled-mining.sh`
- **Verification:** `bash -n`, `bazel test //scripts:phase15_controlled_mining_test`, and the smoke/soak wrapper runs passed.
- **Committed in:** `480aafd`

**3. [Rule 2 - Missing Critical Evidence] Added detector logs to each evidence pack**
- **Found during:** Task 2 (Run controlled mining smoke and bounded soak evidence)
- **Issue:** The plan required detector output to be recorded for hardware use, but the task file list did not include detector logs under the smoke and soak pack directories.
- **Fix:** Added `detect-ultra205.log` to both `mining-smoke/` and `bounded-soak/`.
- **Files modified:** `docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/mining-smoke/detect-ultra205.log`, `docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/bounded-soak/detect-ultra205.log`
- **Verification:** Evidence summaries cite the detector gate, and redaction scan reviewed the retained bench metadata.
- **Committed in:** `480aafd`

**Total deviations:** 3 auto-fixed (1 blocking, 1 bug, 1 missing critical evidence)
**Impact on plan:** All deviations preserved the intended safety and evidence boundaries. No unplanned live mining, scanning, or destructive hardware action was introduced.

## Issues Encountered

- `DEVICE_URL`, `BITAXE_POOL_URL`, `BITAXE_POOL_USER`, and `BITAXE_POOL_PASSWORD` were absent. This was handled as controlled no-share evidence, not as an authentication failure.
- The detector gate passed for exactly one Ultra 205 port, so allow manifests could be created. Live API, WebSocket, pool, and bounded soak activity remained pending due to missing live prerequisites.

## Auth Gates

None. Missing pool and device URL environment variables are expected evidence conditions for this plan, not authentication gates.

## Known Stubs

None. Stub-pattern matches were Bash variable initializers used for argument parsing and test setup, not placeholder data flowing to evidence claims.

## Verification

- `node "$HOME/.codex/get-shit-done/bin/gsd-tools.cjs" verify lifecycle 15 --require-plans --raw`
- RED test run failed as intended before the wrapper existed.
- `bash -n scripts/phase15-controlled-mining.sh scripts/phase15-controlled-mining-test.sh`
- `bazel test //scripts:phase15_controlled_mining_test`
- `cargo test -p bitaxe-stratum --all-features mining_loop`
- `cargo test -p bitaxe-stratum --all-features fake_pool`
- `cargo test -p bitaxe-stratum --all-features queue`
- `cargo test -p bitaxe-api --all-features mining`
- `just detect-ultra205`
- `scripts/phase15-controlled-mining.sh --manifest docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/mining-smoke/allow-mining-smoke.json --surface mining-smoke --out-dir docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/mining-smoke --chip-detect-summary docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/chip-detect.md --work-result-summary docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/work-result.md`
- `scripts/phase15-controlled-mining.sh --manifest docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/bounded-soak/allow-bounded-soak.json --surface bounded-soak --duration-seconds 120 --out-dir docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/bounded-soak --chip-detect-summary docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/chip-detect.md --work-result-summary docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/work-result.md`
- `rg -n "mining-smoke|controlled no-share condition|pool_category|share_outcome|watchdog|safe_stop|bounded-soak|duration_seconds=120|redaction" docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/mining-smoke.md docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/bounded-soak.md docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/redaction-review.md`
- `just parity`
- Secret/redaction scans over smoke and soak artifacts found only expected non-secret category labels and bench metadata.
- Rust pre-commit gate before both commits: `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, and `cargo test --all-features`.
- `git diff -- reference/esp-miner --exit-code`

## User Setup Required

None for plan completion. Future live smoke or bounded soak promotion still requires explicit `DEVICE_URL` and disposable/non-secret pool environment variables, and those must not be committed.

## Next Phase Readiness

Plan 15-05 can consume controlled no-share smoke and unsupported-pending bounded soak evidence. The final ledger should retain the blocker language for live API/WebSocket/pool/soak claims unless explicit live prerequisites are supplied and redaction-reviewed.

*Phase: 15-bm1366-mining-evidence-completion*
*Completed: 2026-07-01*

## Self-Check: PASSED

- Created summary and evidence files exist.
- Task commits `edd5f4f` and `480aafd` exist in git history.
- Summary frontmatter uses only the opening and closing standalone delimiter lines.

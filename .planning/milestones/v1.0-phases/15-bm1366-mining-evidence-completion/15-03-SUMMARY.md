---
phase: 15-bm1366-mining-evidence-completion
plan: "03"
subsystem: parity
tags: [bm1366, mining, evidence, diagnostics, work-result, firmware]
requires:
  - phase: 15-bm1366-mining-evidence-completion
    provides: package-backed BM1366 diagnostic builder and chip-detect trust root
provides:
  - Compile-gated BM1366 work/result diagnostic mode
  - Typed firmware path from diagnostic work frame to result-or-timeout parsing
  - Detector-gated package-backed Ultra 205 work-result evidence
  - Redaction-reviewed diagnostic work dispatch with bounded fail-closed timeout evidence
affects: [phase-15, bm1366-mining, parity-evidence, firmware-asic-adapter]
tech-stack:
  added: []
  patterns:
    - Compile-gated ASIC diagnostics require exact mode and exact hardware-evidence acknowledgement before UART effects.
    - Work/result evidence must stay scoped to diagnostic dispatch and bounded result-or-timeout behavior unless separate mining evidence exists.
key-files:
  created:
    - docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/work-result.md
    - docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/work-result/allow-work-result.json
    - docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/work-result/flash-command-evidence.json
    - docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/work-result/flash-monitor.log
    - docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/work-result/package/bitaxe-ultra205-package.json
  modified:
    - crates/bitaxe-asic/src/bm1366/adapter_gate.rs
    - firmware/bitaxe/src/asic_adapter.rs
    - firmware/bitaxe/src/asic_adapter/status.rs
    - docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/redaction-review.md
key-decisions:
  - "Plan 15-03 treats typed work-result evidence as diagnostic work dispatch plus bounded fail-closed timeout, not production mining or share proof."
  - "The work-result diagnostic uses typed `diagnostic_job_frame`, `Bm1366ValidJobIds`, and `parse_bm1366_result_frame` instead of raw serial bytes or host-side writes."
  - "Large generated package binaries are omitted from docs evidence commits; package manifest, wrapper JSON, and logs carry identity/checksum evidence."
patterns-established:
  - "Work-result diagnostics use `BITAXE_ASIC_DIAGNOSTIC=work-result` plus `BITAXE_HARDWARE_EVIDENCE_ACK=ultra205-work-result-safe-bench` and default to fail-closed otherwise."
  - "A bounded no-result hardware smoke can support diagnostic work dispatch evidence only when work/result non-claims and redaction review are explicit."
requirements-completed: [ASIC-07, STR-06, SAFE-09, EVD-05]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 15-2026-07-01T02-07-59
generated_at: 2026-07-01T04:07:58Z
duration: 15 min
completed: 2026-07-01
---

# Phase 15 Plan 03: Typed BM1366 Work-Result Diagnostic Summary

**BM1366 work-result diagnostics now dispatch typed work through firmware, parse result-or-timeout responses, and record detector-gated Ultra 205 bounded no-result evidence without mining overclaims.**

## Performance

- **Duration:** 15 min
- **Started:** 2026-07-01T03:53:29Z
- **Completed:** 2026-07-01T04:07:58Z
- **Tasks:** 2
- **Files modified:** 11

## Accomplishments

- Added `WorkResultDiagnostic` mode behind exact compile-time mode and hardware evidence acknowledgement strings, while preserving default `FailClosed` behavior.
- Wired firmware work-result execution through typed BM1366 APIs: `diagnostic_job_frame`, `uart.write_frame`, `Bm1366ValidJobIds::single`, `uart.read_exact`, and `parse_bm1366_result_frame`.
- Captured package-backed Ultra 205 work-result evidence on `/dev/cu.usbmodem1101`, including detector output, allow validation, trusted wrapper JSON, serial log markers, and a redaction-cleared Markdown summary.
- Recorded a conservative conclusion: `passed for diagnostic work dispatch with bounded no-result`, with explicit non-claims for accepted shares, live pool behavior, production mining, API/WebSocket telemetry, statistics, frequency, voltage, and fan behavior.

## Task Commits

Each task was committed atomically:

1. **Task 1: Add the bounded typed work/result diagnostic mode** - `d7d965f` (feat)
2. **Task 2: Capture typed work/result diagnostic evidence** - `4583959` (docs)

**Plan metadata:** pending final metadata commit

## Files Created/Modified

- `crates/bitaxe-asic/src/bm1366/adapter_gate.rs` - Adds work-result diagnostic constants, mode selection, and marker tests.
- `firmware/bitaxe/src/asic_adapter.rs` - Runs the compile-gated typed work-result diagnostic and fails closed on timeout, invalid frames, or adapter errors.
- `firmware/bitaxe/src/asic_adapter/status.rs` - Publishes exact work-result diagnostic status markers.
- `docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/work-result.md` - Human-readable hardware evidence summary, observed markers, conclusion, redaction status, and non-claims.
- `docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/work-result/detect-ultra205.log` - Detector gate output with exactly one selected Ultra 205 port and board-info success.
- `docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/work-result/allow-work-result.json` - Mining allow manifest for the exact work-result flash-monitor command.
- `docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/work-result/diagnostic-package-summary.json` - Diagnostic package summary for `work-result` mode and evidence acknowledgement.
- `docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/work-result/flash-command-evidence.json` - Wrapper-owned flash-monitor evidence JSON.
- `docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/work-result/flash-monitor.log` - Serial log with trusted wrapper markers and work-result diagnostic timeout/fail-closed markers.
- `docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/work-result/package/bitaxe-ultra205-package.json` - Copied package manifest used for allow validation and the hardware run.
- `docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/redaction-review.md` - Marks the `bm1366-work-result` pack passed while later Phase 15 packs remain pending.

## Decisions Made

- Scoped work-result evidence to diagnostic work dispatch plus bounded fail-closed timeout, because no valid result frame, share, live pool behavior, or production mining run was observed.
- Kept all ASIC effects inside firmware-owned typed adapters and did not add HTTP diagnostic routes, raw host serial writes, pool sockets, voltage commands, or fan commands.
- Retained the copied package manifest and text/JSON/log evidence, but removed large generated package binaries from the evidence directory before commit.

## Deviations from Plan

### Process Adjustments

**1. AGENTS.md pre-commit rule superseded TDD RED commit**
- **Found during:** Task 1 (Add the bounded typed work/result diagnostic mode)
- **Issue:** The task was marked `tdd="true"`, but repo Rust rules require `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, and `cargo test --all-features` to pass before every commit.
- **Fix:** Wrote and ran the failing targeted RED test first, then implemented the diagnostic and committed only the passing Task 1 state after the full Rust gate.
- **Files modified:** `crates/bitaxe-asic/src/bm1366/adapter_gate.rs`, `firmware/bitaxe/src/asic_adapter.rs`, `firmware/bitaxe/src/asic_adapter/status.rs`
- **Verification:** RED `cargo test -p bitaxe-asic --all-features adapter_gate` failed before implementation; post-implementation targeted tests, firmware build, and full Rust checks passed.
- **Committed in:** `d7d965f`

**2. Generated package binaries omitted from docs evidence**
- **Found during:** Task 2 (Capture typed work/result diagnostic evidence)
- **Issue:** The diagnostic package command generated large binary payloads under the evidence directory.
- **Fix:** Kept the package manifest and wrapper evidence required for identity/checksum proof, then removed the generated binaries before staging.
- **Files modified:** none beyond the committed evidence pack
- **Verification:** `allow-work-result.json`, `flash-command-evidence.json`, and `package/bitaxe-ultra205-package.json` remain committed and `just parity` passed.
- **Committed in:** `4583959`

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Replaced invalid multi-filter Cargo test command with valid targeted commands**
- **Found during:** Task 1 (Add the bounded typed work/result diagnostic mode)
- **Issue:** The plan verification command `cargo test -p bitaxe-asic --all-features adapter_gate work result transcript` is invalid Cargo syntax because extra filter words are parsed as unexpected arguments.
- **Fix:** Ran valid targeted filters separately: `adapter_gate`, `work`, `result`, and `transcript`.
- **Files modified:** none
- **Verification:** All four targeted `cargo test -p bitaxe-asic --all-features ...` commands passed, and `cargo test --all-features` passed before both task commits.
- **Committed in:** process-only adjustment, no file change

**Total deviations:** 2 process adjustments, 1 auto-fixed blocking verification issue.
**Impact on plan:** The deviations preserved the plan's evidence scope and command intent. No unsafe hardware action, checklist overclaim, or production-mining behavior was introduced.

## Issues Encountered

- The exact chip-detect prerequisite was present, so the pending-artifact fallback path was not used.
- The hardware diagnostic observed `bm1366_diagnostic_result=timeout fail_closed=true`, not a parsed valid result frame; the evidence conclusion and non-claims reflect that bounded no-result outcome.

## Known Stubs

None. The stub scan found no `TODO`, `FIXME`, placeholder, `not available`, or hard-coded empty UI data patterns in the files created or modified by this plan. Later-pack `pending` rows in `redaction-review.md` are intentional Phase 15 tracking entries and do not block the work-result objective.

## User Setup Required

None - no external service configuration required.

## Verification

- `node "$HOME/.codex/get-shit-done/bin/gsd-tools.cjs" verify lifecycle 15 --require-plans --raw` - passed.
- `cargo test -p bitaxe-asic --all-features adapter_gate` - RED failed before implementation, then passed after implementation.
- `cargo test -p bitaxe-asic --all-features work` - passed.
- `cargo test -p bitaxe-asic --all-features result` - passed.
- `cargo test -p bitaxe-asic --all-features transcript` - passed.
- `bazel build //firmware/bitaxe:firmware` - passed.
- `rg -n "^conclusion: passed for package-backed chip-detect smoke$" docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/chip-detect.md` - passed.
- `just detect-ultra205` - passed and wrote `work-result/detect-ultra205.log` with selected `port=/dev/cu.usbmodem1101`.
- `scripts/phase15-bm1366-diagnostic-package.sh --mode work-result --out-dir docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/work-result` - passed.
- `bazel run //tools/parity:report -- mining-allow --manifest docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/work-result/allow-work-result.json --surface bm1366-work-result --allowed-command "bazel run //tools/flash:flash -- flash-monitor --board 205 --port /dev/cu.usbmodem1101 --manifest docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/work-result/package/bitaxe-ultra205-package.json --evidence-dir docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/work-result --capture-timeout-seconds 45"` - passed.
- `bazel run //tools/flash:flash -- flash-monitor --board 205 --port /dev/cu.usbmodem1101 --manifest docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/work-result/package/bitaxe-ultra205-package.json --evidence-dir docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/work-result --capture-timeout-seconds 45` - passed with `trusted_output: true`.
- `rg -n "bm1366-work-result|work packet dispatched|result frame parsed|bounded timeout with fail-closed state|no result observed before timeout|non_claims|redaction" docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/work-result.md docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/redaction-review.md` - passed.
- `just parity` - passed with `validation_errors: none`.
- `git diff -- reference/esp-miner --exit-code` - passed.
- Task commit pre-checks: `cargo fmt --all`; `cargo clippy --all-targets --all-features -- -D warnings`; `cargo build --all-targets --all-features`; `cargo test --all-features` - passed before each task commit.

## Next Phase Readiness

Ready for `15-04-PLAN.md`: chip-detect and work-result diagnostic evidence now have package-backed trust roots, explicit non-claims, and redaction review. Controlled mining smoke, bounded soak, API/WebSocket telemetry, accepted shares, frequency behavior, voltage behavior, and fan behavior remain below verified until their own allow manifests, safety prerequisites, safe-stop evidence, and redaction reviews exist.

*Phase: 15-bm1366-mining-evidence-completion*
*Completed: 2026-07-01*

## Self-Check: PASSED

- Summary file exists.
- Created work-result evidence files exist, including the Markdown summary, allow manifest, wrapper evidence JSON, serial log, and copied package manifest.
- Task commits exist: `d7d965f` and `4583959`.
- Summary frontmatter delimiter check passed with standalone `---` only at the opening and closing frontmatter lines.

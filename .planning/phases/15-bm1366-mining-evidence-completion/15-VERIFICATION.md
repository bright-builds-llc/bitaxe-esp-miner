---
phase: 15
slug: bm1366-mining-evidence-completion
plan: "05"
status: passed
score: 3/3 must-haves verified
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 15-2026-07-01T02-07-59
generated_at: 2026-07-01T04:55:59Z
lifecycle_validated: true
---

# Phase 15 - Final Verification

## Scope

This report records final Phase 15 closure verification for the BM1366 mining
evidence completion phase. It supports exact diagnostic and controlled-boundary
claims only. Optional blockers are limited to missing explicit `DEVICE_URL` and
missing live pool prerequisites.

## Command Results

| Command | Result | Output summary |
|---------|--------|----------------|
| `bash -n scripts/phase15-*.sh` | passed | Shell syntax check exited 0. |
| `bazel test //scripts:phase15_bm1366_diagnostic_package_test //scripts:phase15_controlled_mining_test` | passed | 2 test targets passed: `phase15_bm1366_diagnostic_package_test` and `phase15_controlled_mining_test`. |
| `cargo test -p bitaxe-asic --all-features adapter_gate` | passed | 8 passed, 0 failed. |
| `cargo test -p bitaxe-asic --all-features work` | passed | 8 passed, 0 failed. |
| `cargo test -p bitaxe-asic --all-features result` | passed | 11 passed, 0 failed. |
| `cargo test -p bitaxe-asic --all-features transcript` | passed | 7 passed, 0 failed. |
| `cargo test -p bitaxe-stratum --all-features mining_loop` | passed | 11 passed, 0 failed. |
| `cargo test -p bitaxe-stratum --all-features fake_pool` | passed | 5 passed, 0 failed. |
| `cargo test -p bitaxe-stratum --all-features queue` | passed | 7 passed, 0 failed. |
| `cargo test -p bitaxe-api --all-features mining` | passed | 3 passed, 0 failed. |
| `cargo test -p bitaxe-parity --all-features mining_allow` | passed | 8 passed, 0 failed. |
| `cargo fmt --all` | passed | Formatting completed with no diff-producing changes. |
| `cargo clippy --all-targets --all-features -- -D warnings` | passed | Clippy finished cleanly with warnings denied. |
| `cargo build --all-targets --all-features` | passed | Cargo build finished cleanly. |
| `cargo test --all-features` | passed | Full Cargo suite passed across all crates and doctests; output included 93 API, 52 ASIC, 41 config, 10 core, 31 flash, 71 parity, 32 safety, 42 stratum, 3 test-support, and 18 xtask unit tests. |
| `just test` | passed | `bazel test //...` passed 24 test targets and rebuilt the Ultra 205 firmware/package path. |
| `just parity` | passed | Parity report exited 0 with `validation_errors: none`. |
| `just verify-reference` | passed | Reference guard exited 0 with `reference clean: c1915b0a63bfabebdb95a515cedfee05146c1d50`. |
| `git diff -- reference/esp-miner --exit-code` | passed | Reference tree diff exited 0 with no output. |
| `node /Users/peterryszkiewicz/.codex/get-shit-done/bin/gsd-tools.cjs verify lifecycle 15 --expect-id 15-2026-07-01T02-07-59 --expect-mode yolo --require-plans --require-verification --raw` | passed | Final lifecycle validation returned `valid` for lifecycle ID `15-2026-07-01T02-07-59` in `yolo` mode. |

## Command Substitution

The plan listed Cargo invocations with multiple trailing filters:

- `cargo test -p bitaxe-asic --all-features adapter_gate work result transcript`
- `cargo test -p bitaxe-stratum --all-features mining_loop fake_pool queue`

Cargo accepts one test filter per invocation, so these were run as equivalent
single-filter commands in the same sequence. All equivalent filters passed.

## Hardware Commands Used In Phase 15

| Surface | Command actually used | Evidence |
|---------|------------------------|----------|
| Detector gate | `just detect-ultra205` | `docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/chip-detect.md`, `work-result.md`, `mining-smoke.md`, and `bounded-soak.md` |
| Chip-detect package | `scripts/phase15-bm1366-diagnostic-package.sh --mode chip-detect --out-dir docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/chip-detect` | `docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/chip-detect.md` |
| Chip-detect allow gate | `bazel run //tools/parity:report -- mining-allow --manifest docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/chip-detect/allow-chip-detect.json --surface bm1366-chip-detect --allowed-command "bazel run //tools/flash:flash -- flash-monitor --board 205 --port /dev/cu.usbmodem1101 --manifest docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/chip-detect/package/bitaxe-ultra205-package.json --evidence-dir docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/chip-detect --capture-timeout-seconds 35"` | `docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/chip-detect.md` |
| Chip-detect flash-monitor | `bazel run //tools/flash:flash -- flash-monitor --board 205 --port /dev/cu.usbmodem1101 --manifest docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/chip-detect/package/bitaxe-ultra205-package.json --evidence-dir docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/chip-detect --capture-timeout-seconds 35` | `docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/chip-detect.md` |
| Work/result package | `scripts/phase15-bm1366-diagnostic-package.sh --mode work-result --out-dir docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/work-result` | `docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/work-result.md` |
| Work/result allow gate | `bazel run //tools/parity:report -- mining-allow --manifest docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/work-result/allow-work-result.json --surface bm1366-work-result --allowed-command "bazel run //tools/flash:flash -- flash-monitor --board 205 --port /dev/cu.usbmodem1101 --manifest docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/work-result/package/bitaxe-ultra205-package.json --evidence-dir docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/work-result --capture-timeout-seconds 45"` | `docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/work-result.md` |
| Work/result flash-monitor | `bazel run //tools/flash:flash -- flash-monitor --board 205 --port /dev/cu.usbmodem1101 --manifest docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/work-result/package/bitaxe-ultra205-package.json --evidence-dir docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/work-result --capture-timeout-seconds 45` | `docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/work-result.md` |
| Controlled mining smoke | `scripts/phase15-controlled-mining.sh --manifest docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/mining-smoke/allow-mining-smoke.json --surface mining-smoke --out-dir docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/mining-smoke --chip-detect-summary docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/chip-detect.md --work-result-summary docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/work-result.md` | `docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/mining-smoke.md` |
| Bounded soak | `scripts/phase15-controlled-mining.sh --manifest docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/bounded-soak/allow-bounded-soak.json --surface bounded-soak --duration-seconds 120 --out-dir docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/bounded-soak --chip-detect-summary docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/chip-detect.md --work-result-summary docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/work-result.md` | `docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/bounded-soak.md` |
| Restore | No separate restore command was run. | The board never entered active live mining; safe-state/restore evidence is limited to reset-held-low and safe-stop confirmed-or-pending diagnostic states. |
| HTTP | No HTTP command was run against a device URL. | Blocked by missing explicit `DEVICE_URL`; API evidence remains below verified. |
| WebSocket | No WebSocket command was run against a device URL. | Blocked by missing explicit `DEVICE_URL`; WebSocket evidence remains below verified. |

## Core Gates

| Gate | Status | Evidence |
|------|--------|----------|
| Detector evidence | passed | Detector output recorded exactly one likely Ultra 205 port, `/dev/cu.usbmodem1101`, and successful ESP32-S3 board-info in Phase 15 evidence. |
| Trusted chip-detect evidence | passed | Package-backed chip-detect evidence passed for the exact no-mining fail-closed subclaim. |
| Work/result diagnostic evidence | passed | Package-backed work/result evidence passed for typed diagnostic dispatch plus bounded no-result/fail-closed handling. |
| Safe-stop or restore evidence | passed with boundary | No active live mining state was entered; diagnostic safe-state markers and controlled wrapper safe-stop status are recorded. |
| Redaction review | passed | `redaction-review.md` passed for all cited artifacts and leaves absent API/WebSocket/live-pool artifacts uncited. |
| Reference cleanliness | passed | `just verify-reference` and `git diff -- reference/esp-miner --exit-code` both passed. |
| Lifecycle validation | passed | Final lifecycle validation returned `valid` for lifecycle ID `15-2026-07-01T02-07-59` in `yolo` mode. |

## Accepted Optional Blockers

- Missing explicit `DEVICE_URL` blocked live HTTP `/api/system/info`, live
  WebSocket telemetry, and live statistics producer evidence.
- Missing live pool prerequisites blocked live pool behavior, accepted shares,
  rejected shares, and bounded live soak evidence.

No core detector, trusted chip-detect, work/result diagnostic, redaction,
safe-stop/restore boundary, parity, reference cleanliness, or lifecycle blocker
is accepted for passed status.

## Conclusion

Phase 15 verification status is `passed`. All required targeted checks,
aggregate checks, parity, reference cleanliness, redaction, and lifecycle gates
passed. The only residual blockers are optional: missing explicit `DEVICE_URL`
and missing live pool prerequisites.

This report was refreshed after `15-05-SUMMARY.md` was created so lifecycle
validation could confirm verification is newer than all phase summary artifacts.

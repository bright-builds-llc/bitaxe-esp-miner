---
phase: 12
phase_slug: 12-asic-and-mining-hardware-evidence
verified: 2026-06-30T01:33:40Z
status: passed
score: "10/10 must-haves verified"
generated_by: gsd-verify-work
lifecycle_mode: yolo
phase_lifecycle_id: 12-2026-06-30T00-13-19
generated_at: 2026-06-30T01:33:40Z
lifecycle_validated: true
overrides_applied: 0
---

# Phase 12: ASIC And Mining Hardware Evidence Verification Report

**Phase Goal:** BM1366 initialization, work/result handling, and first Ultra 205 mining-loop evidence are captured where safe, or held explicitly pending without checklist overclaims.
**Verified:** 2026-06-30T01:33:40Z
**Status:** passed
**Re-verification:** No - initial verification

## Goal Achievement

Phase 12 achieved its evidence-governance goal. The phase produced a claim matrix, detector-gated hardware artifacts, a fail-closed mining preflight hook, generated safe-boot evidence, a chip-detect diagnostic boundary, redaction review, checklist citations, and final validation.

This pass does not claim full BM1366 initialization, diagnostic work-send/result-receive, live mining smoke, bounded mining soak, WebSocket mining telemetry, or live statistics producer parity. Those claims remain hardware evidence pending.

## Observable Truths

| # | Truth | Status | Evidence |
| --- | --- | --- | --- |
| 1 | Phase 12 hardware work starts with `just detect-ultra205`. | VERIFIED | `detect-ultra205.log` and `restore-detect-ultra205.log` both record one selected ESP32-S3 port and successful board-info output. |
| 2 | Safe boot evidence records board, port, commands, commits, trusted-output status, logs, and conclusion. | VERIFIED | `safe-boot/flash-command-evidence.json` records board `205`, `/dev/cu.usbmodem1101`, firmware commit `42bb1f7d4584d05ab453995e04bcab506c8b3fe9`, and reference commit `c1915b0a63bfabebdb95a515cedfee05146c1d50`. |
| 3 | Trusted safe boot keeps mining and ASIC work submission disabled. | VERIFIED | `safe-boot/flash-monitor.log` records `safe_state`, `asic_status=preflight_missing`, `mining_loop_status=blocked`, and `work_submission=disabled`. |
| 4 | Chip-detect evidence is scoped to exact observed markers and not overclaimed. | VERIFIED | The chip-detect wrapper wrote JSON/log artifacts but marked `trusted_output=false`; checklist rows remain below `verified`. |
| 5 | Mining smoke and soak are not run without a bounded probe and recovery path. | VERIFIED | `mining-smoke-preflight.log` records `controlled_mining_smoke=not_run`, `bounded_mining_soak=not_run`, and `work_submission=disabled`. |
| 6 | Generated artifacts are redaction-reviewed before citation. | VERIFIED | `redaction-review.md` is complete and concludes passed. |
| 7 | Checklist rows cite Phase 12 without promoting unsupported claims. | VERIFIED | `ASIC-002`, `ASIC-003`, `ASIC-004`, `ASIC-005`, `ASIC-007`, `STR-006`, `STR-007`, `STR-008`, `API-002`, `API-006`, and `STAT-002` cite Phase 12 and remain below `verified` where live evidence is pending. |
| 8 | Parity tooling rejects unsupported verified ASIC/mining claims. | VERIFIED | `cargo test -p bitaxe-parity --all-features` passed, including ASIC/mining verified-row guard tests. |
| 9 | Final board state is restored to trusted packaged safe boot. | VERIFIED | `safe-boot-restore/flash-command-evidence.json` has `trusted_output=true`; restore log shows `spiffs_mount=available`, blocked mining, and disabled work submission. |
| 10 | Final verification remains green. | VERIFIED | `just parity`, `just test`, Cargo checks, and reference diff check passed. |

**Score:** 10/10 truths verified

## Required Artifacts

| Artifact | Expected | Status | Details |
| --- | --- | --- | --- |
| `docs/parity/evidence/phase-12-asic-and-mining-hardware-evidence.md` | Ledger, claim matrix, execution log, final verification, and conclusions | VERIFIED | Records detector, safe boot, chip-detect, preflight, restore, redaction, residual risk, and final conclusion. |
| `scripts/phase12-mining-smoke-preflight.sh` | Read-only fail-closed mining smoke preflight | VERIFIED | Requires blocked mining-loop and disabled work submission markers before concluding safe-blocked. |
| `detect-ultra205.log` | Detector and board-info output | VERIFIED | Records board-info output and selected port. |
| `safe-boot/flash-command-evidence.json` and `safe-boot/flash-monitor.log` | Trusted wrapper safe-boot evidence | VERIFIED | Trusted output with safe state and disabled mining/work markers. |
| `chip-detect/flash-command-evidence.json` and `chip-detect/flash-monitor.log` | Chip-detect diagnostic boundary | VERIFIED | Artifacts exist and are marked wrapper-untrusted due SPIFFS absence, preventing overclaim. |
| `mining-smoke-preflight.log` | Mining smoke/soak preflight conclusion | VERIFIED | Records blocked status and no smoke/soak run. |
| `safe-boot-restore/flash-command-evidence.json` and `safe-boot-restore/flash-monitor.log` | Final restore evidence | VERIFIED | Trusted packaged safe boot restored. |
| `docs/parity/checklist.md` | Conservative checklist citations | VERIFIED | Affected rows cite Phase 12 and do not overclaim live ASIC/mining evidence. |
| `12-VALIDATION.md` | Passed validation contract | VERIFIED | Frontmatter is passed/true/true and ASIC/mining statuses preserve pending hardware boundaries. |

## Behavioral Spot-Checks

| Behavior | Command | Result | Status |
| --- | --- | --- | --- |
| Preflight script syntax | `bash -n scripts/phase12-mining-smoke-preflight.sh` | Passed | PASS |
| Targeted ASIC/Stratum/Safety/API tests | `cargo test -p bitaxe-asic -p bitaxe-stratum -p bitaxe-safety -p bitaxe-api --all-features` | Passed | PASS |
| Parity crate tests | `cargo test -p bitaxe-parity --all-features` | Passed | PASS |
| Checklist overclaim guard | `just parity` | Passed with `validation_errors: none` | PASS |
| Aggregate repo tests | `just test` | 13 Bazel test targets passed | PASS |
| Reference tree read-only | `git diff -- reference/esp-miner --exit-code` | No diff | PASS |

## Requirements Coverage

| Requirement | Status | Evidence |
| --- | --- | --- |
| ASIC-07 | SATISFIED for evidence posture, hardware evidence pending for full live claims | Safe boot passed; chip-detect diagnostic remained wrapper-untrusted; work/result did not run; checklist stays below verified. |
| STR-06 | SATISFIED for fail-closed first-loop boundary, hardware evidence pending for live mining | Trusted safe boot and preflight show blocked loop and disabled work submission. |
| STR-07 | SATISFIED for criteria/redaction/preflight, hardware evidence pending for smoke/soak | Criteria and preflight artifacts exist; live smoke/soak did not run. |
| EVD-05 | SATISFIED | Unit, parity, Bazel, hardware safe-boot, redaction, checklist, and validation layers are recorded with pending boundaries. |

## Gaps Summary

No blocking gaps remain for Phase 12. Live BM1366 full initialization, diagnostic work/result, controlled mining smoke, bounded soak, live WebSocket telemetry, and live statistics producer evidence remain future work and are explicitly not promoted.

_Verified: 2026-06-30T01:33:40Z_
_Verifier: the agent (gsd-verifier)_

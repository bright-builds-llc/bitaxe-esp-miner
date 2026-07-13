---
phase: 19
slug: recovery-regression-and-otawww-evidence
status: passed
nyquist_compliant: true
wave_0_complete: true
created: 2026-07-03
---

# Phase 19 - Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

***

## Test Infrastructure

| Property | Value |
| --- | --- |
| **Framework** | Bazel `sh_test` plus Rust `rust_test` targets |
| **Config file** | `scripts/BUILD.bazel`, `crates/bitaxe-api/BUILD.bazel`, `tools/parity/BUILD.bazel` |
| **Quick run command** | `bazel test //scripts:phase19_recovery_otawww_evidence_test` |
| **Full suite command** | `bazel test //scripts:phase19_recovery_otawww_evidence_test //scripts:phase16_recovery_regression_test //scripts:phase18_firmware_ota_evidence_test //crates/bitaxe-api:tests //tools/parity:tests` |
| **Estimated runtime** | ~120 seconds for the full local suite when cached dependencies are warm |

***

## Sampling Rate

- **After every task commit:** Run the narrow changed-path test for touched
  scripts, Rust crates, or parity tooling.
- **After every plan wave:** Run the full suite above plus `just parity` and
  `just verify-reference` when checklist, release docs, or evidence ledgers
  changed.
- **Before `/gsd-verify-work`:** Run `just package`, manifest-backed
  release-gate validation, `just parity`, `just verify-reference`, lifecycle
  validation, and every hardware/network command actually used.
- **Max feedback latency:** 180 seconds for local automated checks, excluding
  explicit hardware evidence capture windows.

***

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 19-W0-01 | 19-01 | 0 | EVD-05 | T-19-01 | Phase 19 helper tests prove default no-allow behavior produces pending evidence without running destructive commands. | shell unit | `bazel test //scripts:phase19_recovery_otawww_evidence_test` | yes | green |
| 19-W0-02 | 19-01 | 0 | REL-08, API-09 | T-19-02 | Helper tests prove allowed failed-update and interrupted-update flows preserve Phase 16 detector and board-info gate delegation before live HTTP action. | shell unit | `bazel test //scripts:phase19_recovery_otawww_evidence_test //scripts:phase16_recovery_regression_test` | yes | green |
| 19-W0-03 | 19-01 | 0 | REL-03 | T-19-03 | OTAWWW gap evidence is explicit and cannot be promoted from `www.bin`, route presence, or static serving alone. | shell/Bazel | `bazel test //scripts:phase19_recovery_otawww_evidence_test` | yes | green |
| 19-W1-01 | 19-02 | 1 | REL-07, EVD-05 | T-19-04 | Package and release-gate evidence cites the current manifest and reference commit before hardware evidence is trusted. | workflow | `just package && bazel run //tools/parity:report -- release-gate --manifest docs/parity/evidence/phase-19-recovery-regression-and-otawww-evidence/package-release-gate/bitaxe-ultra205-package.json` | yes | green |
| 19-W2-01 | 19-03 | 2 | REL-08, API-09 | T-19-05 | Recovery evidence names board `205`, port, source commit, package manifest, exact commands, logs, restore action, and conclusion, or records the exact blocker. | hardware/workflow | `just detect-ultra205` plus the phase helper command documented in the active plan | yes | green |
| 19-W3-01 | 19-04 | 3 | REL-03, REL-07, EVD-05 | T-19-06 | Release docs, checklist, requirements traceability, and summary ledger distinguish verified behavior from blocked, pending, below-verified, and non-claim behavior. | docs/tooling | `just parity && just verify-reference && rg -n "phase-19-recovery-regression-and-otawww-evidence|OTAWWW|failed-update|large erase|interrupted" docs/release/ultra-205.md docs/parity/checklist.md .planning/REQUIREMENTS.md` | yes | green |

*Status: pending = not yet implemented or not yet run; green = command passed; red = command failed; blocked = prerequisite unavailable.*

***

## Wave 0 Requirements

- [x] `scripts/phase19-recovery-otawww-evidence.sh` or an equivalent
  phase-specific wrapper exists if Phase 19 does more than documentation-only
  closure.
- [x] `scripts/phase19-recovery-otawww-evidence-test.sh` covers pending
  no-allow behavior, failed-update allow gate, interrupted-update allow gate,
  large-erase restore gate or explicit non-run status, target redaction, and
  OTAWWW gap output.
- [x] `scripts/BUILD.bazel` exposes `phase19_recovery_otawww_evidence` and
  `phase19_recovery_otawww_evidence_test` if the scripts are added.
- [x] `docs/parity/evidence/phase-19-recovery-regression-and-otawww-evidence/evidence-contract.md`
  exists before live artifacts are cited.
- [x] Final redaction-review criteria list every expected response, body,
  header, detector, board-info, serial, recovery, and OTAWWW artifact as
  reviewed or `absent - not cited`.

***

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
| --- | --- | --- | --- |
| Allowed large erase and factory restore | REL-08, EVD-05 | Destructive hardware action cannot be safely simulated as proof of board recovery. | Run only when the active plan documents `--allow-large-erase`, exact erase command, factory image, restore command, monitor capture, stop conditions, and safe-state markers. Record detector, board-info, command output, restore result, and final conclusion. |
| Allowed interrupted upload recovery | REL-08, API-09 | The meaningful evidence is live device behavior after a bounded client-side interruption. | Run only with explicit `DEVICE_URL`, detector gate, board-info gate, current manifest, bounded interruption command, post-interruption HTTP/static/recovery/API or serial safe-state check, and redaction review. |
| Whole-`www` OTAWWW update parity, if attempted | REL-03, API-09 | Current firmware treats OTAWWW as a gap; full parity requires live partition update and interrupted-update recovery evidence. | Implement or use a plan-documented whole-`www` procedure with size checks, chunked erase/write, recovery access, and interrupted-update hardware-regression evidence. Otherwise record the REL-03 gap. |

***

## Validation Sign-Off

- [x] All tasks have automated verify commands or documented manual-only gates.
- [x] Sampling continuity: no three consecutive tasks skip automated checks.
- [x] Wave 0 covers all missing helper, evidence-contract, and redaction-review
  references.
- [x] No watch-mode flags are used in verification commands.
- [x] Feedback latency stays under 180 seconds for local checks.
- [x] `nyquist_compliant: true` is set in frontmatter after Wave 0
  infrastructure exists and the plan checker confirms coverage.

**Approval:** passed

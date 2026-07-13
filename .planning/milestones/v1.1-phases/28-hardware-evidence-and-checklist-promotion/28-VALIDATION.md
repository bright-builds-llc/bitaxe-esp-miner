---
phase: 28
slug: hardware-evidence-and-checklist-promotion
status: complete
nyquist_compliant: true
wave_0_complete: true
created: 2026-07-06
---

# Phase 28 - Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

## Test Infrastructure

| Property | Value |
| --- | --- |
| **Framework** | Bazel `rust_test` / `sh_test` over tools, scripts, and parity fixtures. |
| **Config file** | `tools/parity/BUILD.bazel`, `BUILD.bazel` per touched surface. |
| **Quick run command** | `bazel test //tools/parity:tests --test_output=errors` |
| **Full suite command** | Final gate in plan 28-03 (parity, verify-reference, lifecycle). |
| **Estimated runtime** | Parity tests under 60 seconds; full gate under 120 seconds. |

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Automated Command | File Exists | Status |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 28-01-01 | 01 | 1 | SAFE-10, SAFE-11, SAFE-12, SAFE-13, CFG-07, ASIC-09, ASIC-12 | T-28-01 | `test -d docs/parity/evidence/phase-28-hardware-evidence-and-checklist-promotion && rg -l "source_phase27_root|consolidation_status|share_outcome: blocked_safe_prerequisite" docs/parity/evidence/phase-28-hardware-evidence-and-checklist-promotion/*.md` | yes | passed |
| 28-01-02 | 01 | 1 | SAFE-10, SAFE-11, SAFE-12, SAFE-13, CFG-07, ASIC-09, ASIC-12 | T-28-02 | `rg -n "SAFE-10|SAFE-11|SAFE-12|SAFE-13|CFG-07|ASIC-09|ASIC-12|blocked_safe_prerequisite|source_phase27_root" docs/parity/evidence/phase-28-hardware-evidence-and-checklist-promotion/summary.md` | yes | passed |
| 28-02-01 | 02 | 2 | SAFE-10, SAFE-11, SAFE-12, SAFE-13, CFG-07, ASIC-09, ASIC-12 | T-28-04 | `rg "phase-28-hardware-evidence-and-checklist-promotion" docs/parity/checklist.md` | yes | passed |
| 28-02-02 | 02 | 2 | SAFE-10, SAFE-11, STR-08, STR-09, SAFE-12, SAFE-13, CFG-07, ASIC-09–12 | T-28-05 | `rg "\\| STR-11 \\|.*verified|\\| EVD-08 \\|.*verified" docs/parity/checklist.md && just parity` | yes | passed |
| 28-03-01 | 03 | 3 | SAFE-10, SAFE-11, SAFE-12, SAFE-13, CFG-07, ASIC-09, ASIC-12 | T-28-07, T-28-08 | `bazel test //tools/parity:tests --test_output=errors` | yes | passed |
| 28-03-02 | 03 | 3 | SAFE-10, SAFE-11, SAFE-12, SAFE-13, CFG-07, ASIC-09, ASIC-12 | T-28-09 | `bazel test //tools/parity:tests && just parity && just verify-reference && node "$HOME/.cursor/get-shit-done/bin/gsd-tools.cjs" verify lifecycle 28 --expect-id 28-2026-07-06T17-21-15 --expect-mode yolo --require-plans` | yes | passed |

## Wave 0 Requirements

- [x] Phase 28 evidence root with all Phase 23 slots plus `source_phase27_root` and `consolidation_status` fields.
- [x] `docs/parity/evidence/phase-28-hardware-evidence-and-checklist-promotion/summary.md` mapping requirements with inherited `share_outcome: blocked_safe_prerequisite`.
- [x] `tools/parity` tests for `validate_phase28_hardware_promotion_row` overbroad promotion rejection.
- [x] Conservative checklist updates for in-scope rows without downgrading Phase 26/ earlier verified rows.
- [x] `operator_evidence.rs` extended to validate Phase 28 consolidation root fields.

## Final Gate Results

- `bazel test //tools/parity:tests` passed.
- `just parity` passed.
- `just verify-reference` passed.
- `node "$HOME/.cursor/get-shit-done/bin/gsd-tools.cjs" verify lifecycle 28 --expect-id 28-2026-07-06T17-21-15 --expect-mode yolo --require-plans` passed.
- Hardware mode optional; Phase 27 blocked categories sufficient for deterministic closure.
- `share_outcome: blocked_safe_prerequisite` preserved; STR-09 and CFG-07 remain below `verified`.

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
| --- | --- | --- | --- |
| Detector-gated accepted/rejected live share | STR-09 | Deferred to future hardware phase; Phase 28 preserves non-claims. | Do not rerun hardware in Phase 28 unless plan explicitly documents blocked-mode workflow proof only. |

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies resolved.
- [x] Sampling continuity: no 3 consecutive tasks without automated verify.
- [x] Wave 0 covers all MISSING references above.
- [x] `nyquist_compliant: true` set after Wave 0 complete and final gate passes.

**Approval:** passed by repo-native final gate.

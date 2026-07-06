---
phase: 28
slug: hardware-evidence-and-checklist-promotion
status: pending
nyquist_compliant: false
wave_0_complete: false
created: 2026-07-06
---

# Phase 28 - Validation Strategy

> Per-phase validation contract for feedback sampling during execution.
> Nyquist placeholders — update status and commands as plans execute.

## Test Infrastructure

| Property | Value |
| --- | --- |
| **Framework** | Bazel `rust_test` / `sh_test` over tools, scripts, and parity fixtures. |
| **Config file** | `tools/parity/BUILD.bazel`, `BUILD.bazel` per touched surface. |
| **Quick run command** | `bazel test //tools/parity:tests --test_output=errors` |
| **Full suite command** | Final gate in plan 28-03 (parity, verify-reference, lifecycle). |
| **Estimated runtime** | Parity tests under 60 seconds; full gate depends on firmware touch (none expected). |

## Sampling Rate

- **After every task commit:** Run the narrow Bazel target for touched crate, tool, or script.
- **After every plan wave:** Run `bazel test //tools/parity:tests` and `just parity`.
- **Before `/gsd-verify-work`:** Run final gate commands from plan 28-03.
- **Max feedback latency:** Prefer under 120 seconds for tool-only tasks.

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Automated Command | File Exists | Status |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 28-01-01 | 01 | 1 | SAFE-10, SAFE-11, SAFE-12, SAFE-13, CFG-07, ASIC-09, ASIC-12 | T-28-01 | MISSING — Wave 0: create Phase 28 evidence root slot check | pending | pending |
| 28-01-02 | 01 | 1 | SAFE-10, SAFE-11, SAFE-12, SAFE-13, CFG-07, ASIC-09, ASIC-12 | T-28-02 | MISSING — Wave 0: summary/redaction/conclusion rg gate | pending | pending |
| 28-02-01 | 02 | 2 | SAFE-10, SAFE-11, SAFE-12, SAFE-13, CFG-07, ASIC-09, ASIC-12 | T-28-03 | MISSING — Wave 0: checklist row citation check | pending | pending |
| 28-02-02 | 02 | 2 | SAFE-10, SAFE-11, STR-08, STR-09, SAFE-12, SAFE-13, CFG-07, ASIC-09–12 | T-28-04 | MISSING — Wave 0: no-downgrade + below-verified caps | pending | pending |
| 28-03-01 | 03 | 3 | SAFE-10, SAFE-11, SAFE-12, SAFE-13, CFG-07, ASIC-09, ASIC-12 | T-28-05, T-28-06 | MISSING — Wave 0: `validate_phase28_hardware_promotion_row` tests | pending | pending |
| 28-03-02 | 03 | 3 | SAFE-10, SAFE-11, SAFE-12, SAFE-13, CFG-07, ASIC-09, ASIC-12 | T-28-07 | MISSING — Wave 0: final phase gate | pending | pending |

## Wave 0 Requirements

- [ ] Phase 28 evidence root with all Phase 23 slots plus `source_phase27_root` and `consolidation_status` fields.
- [ ] `docs/parity/evidence/phase-28-hardware-evidence-and-checklist-promotion/summary.md` mapping requirements with inherited `share_outcome: blocked_safe_prerequisite`.
- [ ] `tools/parity` tests for `validate_phase28_hardware_promotion_row` overbroad promotion rejection.
- [ ] Conservative checklist updates for in-scope rows without downgrading Phase 26/ earlier verified rows.
- [ ] Optional: extend `operator_evidence.rs` to validate Phase 28 consolidation root if plan 28-01 adds loader support.

## Final Gate Results

- Pending execution of plan 28-03-02.
- Expected commands: `bazel test //tools/parity:tests`, `just parity`, `just verify-reference`, lifecycle verify `28-2026-07-06T17-21-15`.
- Hardware mode optional; Phase 27 blocked categories sufficient for deterministic closure.

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
| --- | --- | --- | --- |
| Detector-gated accepted/rejected live share | STR-09 | Deferred to future hardware phase; Phase 28 preserves non-claims. | Do not rerun hardware in Phase 28 unless plan explicitly documents blocked-mode workflow proof only. |

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies resolved.
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify.
- [ ] Wave 0 covers all MISSING references above.
- [ ] `nyquist_compliant: true` set only after Wave 0 complete and final gate passes.

**Approval:** pending

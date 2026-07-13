---
phase: 27-live-hardware-asic-and-stratum-bridge
plan: 05
subsystem: firmware
tags: [h4, continuous-result-task, job-redispatch, gap-closure]
requires:
  - phase: 27-live-hardware-asic-and-stratum-bridge
    provides: 27-UAT.md gaps 8–9
provides:
  - H4 upstream task orchestration investigation flags
  - work-result-upstream-diff-v6-h4.md
  - J2c hardware evidence for continuous listener behavior
affects: [STR-09, ASIC-11]
key-files:
  created:
    - .planning/phases/27-live-hardware-asic-and-stratum-bridge/work-result-upstream-diff-v6-h4.md
    - .planning/phases/27-live-hardware-asic-and-stratum-bridge/b5-h4-orchestration-20260706-J2c.md
  modified:
    - firmware/bitaxe/src/live_stratum_runtime.rs
    - firmware/bitaxe/src/asic_adapter/work_result_investigation.rs
    - firmware/bitaxe/src/asic_adapter.rs
    - crates/bitaxe-asic/src/work_result_investigation.rs
    - .planning/phases/27-live-hardware-asic-and-stratum-bridge/phase27-04-defer-note.md
    - .planning/phases/27-live-hardware-asic-and-stratum-bridge/work-result-hypothesis-results.md
requirements-completed: []
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 27-2026-07-05T14-51-50
generated_at: 2026-07-06T16:50:00Z
completed: 2026-07-06
duration: 45min
---

# Phase 27 Plan 05: H4 Upstream Task Orchestration Gap Closure Summary

**Investigation flags emulate upstream `ASIC_result_task` continuous listen + optional job re-dispatch; J2c hardware confirms non-fatal timeout loop but not post-dispatch UART or share proof**

## Task Results

| Task | Status | Evidence |
| --- | --- | --- |
| 27-05-01 Upstream mapping v6 | **Done** | `work-result-upstream-diff-v6-h4.md` |
| 27-05-02 Continuous result + job pump flags | **Done** | `continuous_result_task`, `job_redispatch_pump`; host test in `bitaxe-asic` |
| 27-05-03 Hardware J2c | **Partial** | [`b5-h4-orchestration-20260706-J2c/`](b5-h4-orchestration-20260706-J2c/) — listener + timeout_continue; no `work_dispatched` |
| 27-05-04 Defer note | **Done** | 27-04 promotion still blocked; share/UART tiers NOT MET |

## Verification

- `bazel test //crates/bitaxe-asic:tests //crates/bitaxe-stratum:tests` passed
- `bazel build //firmware/bitaxe:firmware` passed (H4 investigation package)
- Hardware: `just detect-ultra205` + J2c foreground capture on board 205

## Self-Check: PARTIAL

- H4 code markers observed: `h4_continuous_result=listener_armed`, `timeout_continue`
- UAT gaps 8–9 remain open: no post-dispatch UART proof; no accepted/rejected share
- 27-04 checklist verified promotion: **still deferred**

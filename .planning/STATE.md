---
gsd_state_version: 1.0
milestone: v1.1
milestone_name: milestone
status: Phase 28.1.1.3 in progress — plan 01 complete
stopped_at: Completed 28.1.1.3-01-PLAN.md
last_updated: "2026-07-09T03:13:27.000Z"
last_activity: 2026-07-09 -- Executed 28.1.1.3-01 RX-acquisition comparator + flood-safe summary markers
progress:
  total_phases: 14
  completed_phases: 11
  total_plans: 49
  completed_plans: 46
  percent: 94
---

# Project State

## Current Position

- **Phase:** 28.1.1.3 — BM1366 Result RX Acquisition Model Nonce-Production Diagnosis
- **Plan:** 01/04 complete; next `28.1.1.3-02-PLAN.md`
- **Status:** Wave 0 comparator + compact firmware markers shipped

## Decisions (Phase 28.1.1.3)

- Recommender: `job_tx>0` + `result_read_attempt>0` + `!result_correlated` → `result_rx_acquisition_model` (no `partial_frame≥5`)
- Never emit `match_upstream_register_read_poll` or `post_max_baud_delay_2000` from RX-acquisition comparator
- Compact `asic_rx_acquisition_summary` every 50 result polls; counters always increment even when uart_trace floods

## Performance Metrics

| Phase | Plan | Duration | Tasks | Files |
|-------|------|----------|-------|-------|
| 28.1.1.3 | 01 | 2 min | 2 | 4 |

## Session

- **Stopped at:** Completed 28.1.1.3-01-PLAN.md
- **Resume:** Execute `28.1.1.3-02-PLAN.md` (long-block A/B diagnostic)

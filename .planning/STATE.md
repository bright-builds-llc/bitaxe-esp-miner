---
gsd_state_version: 1.0
milestone: v1.1
milestone_name: milestone
status: Phase 28.1.1.3 in progress — plan 03 complete
stopped_at: Completed 28.1.1.3-03-PLAN.md
last_updated: "2026-07-09T03:25:50.000Z"
last_activity: 2026-07-09 -- Executed 28.1.1.3-03 falsified long-block disposition
progress:
  total_phases: 14
  completed_phases: 11
  total_plans: 50
  completed_plans: 49
  percent: 98
---

# Project State

## Current Position

- **Phase:** 28.1.1.3 — BM1366 Result RX Acquisition Model Nonce-Production Diagnosis
- **Plan:** 03/04 complete; next `28.1.1.3-04-PLAN.md`
- **Status:** Long-block falsified (`patch_disposition: falsified_upstream_like_long_block_receive`); Plan 04 owns final evidence with `gaps_found`

## Decisions (Phase 28.1.1.3)

- Recommender: `job_tx>0` + `result_read_attempt>0` + `!result_correlated` → `result_rx_acquisition_model` (no `partial_frame≥5`)
- Never emit `match_upstream_register_read_poll` or `post_max_baud_delay_2000` from RX-acquisition comparator
- Compact `asic_rx_acquisition_summary` every 50 result polls; counters always increment even when uart_trace floods
- Forced A/B `upstream_like_long_block_receive` with `RESULT_WORK_TIMEOUT_MS=10000` (not `MAX_POLL_SLICE` alone)
- Long-block A/B `ab_outcome: unchanged` (no correlate/submit); Plan 03 must not patch default; `next_hypothesis: asic_enable_power_sequencing`
- `patch_disposition=falsified_upstream_like_long_block_receive`; Task 2 default promotion skipped
- `next_hypothesis=asic_enable_power_sequencing`; `phase30_promotion_input=pending`

## Performance Metrics

| Phase | Plan | Duration | Tasks | Files |
|-------|------|----------|-------|-------|
| 28.1.1.3 | 01 | 2 min | 2 | 4 |
| 28.1.1.3 | 02 | 10 min | 2 | 7 |
| 28.1.1.3 | 03 | 1 min | 2 | 1 |

## Session

- **Stopped at:** Completed 28.1.1.3-03-PLAN.md
- **Resume:** Execute `28.1.1.3-04-PLAN.md` (final evidence / gaps_found closure)

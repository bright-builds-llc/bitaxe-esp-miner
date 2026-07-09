---
gsd_state_version: 1.0
milestone: v1.1
milestone_name: milestone
status: Phase 28.1.1.3 complete — gaps_found
stopped_at: Completed 28.1.1.3-04-PLAN.md (gaps_found)
last_updated: "2026-07-09T03:27:00.000Z"
last_activity: 2026-07-09 -- Closed 28.1.1.3 with gaps_found; next_hypothesis asic_enable_power_sequencing
progress:
  total_phases: 14
  completed_phases: 12
  total_plans: 50
  completed_plans: 50
  percent: 100
---

# Project State

## Current Position

- **Phase:** 28.1.1.3 — BM1366 Result RX Acquisition Model Nonce-Production Diagnosis
- **Plan:** 04/04 complete
- **Status:** `gaps_found` — long-block falsified; `next_hypothesis: asic_enable_power_sequencing`; Phase 30 promotion pending

## Decisions (Phase 28.1.1.3)

- Recommender: `job_tx>0` + `result_read_attempt>0` + `!result_correlated` → `result_rx_acquisition_model` (no `partial_frame≥5`)
- Never emit `match_upstream_register_read_poll` or `post_max_baud_delay_2000` from RX-acquisition comparator
- Compact `asic_rx_acquisition_summary` every 50 result polls; counters always increment even when uart_trace floods
- Forced A/B `upstream_like_long_block_receive` with `RESULT_WORK_TIMEOUT_MS=10000` (not `MAX_POLL_SLICE` alone)
- Long-block A/B `ab_outcome: unchanged` (no correlate/submit); Plan 03 must not patch default; `next_hypothesis: asic_enable_power_sequencing`
- `patch_disposition=falsified_upstream_like_long_block_receive`; Task 2 default promotion skipped
- Final `verification_verdict: gaps_found`; `phase30_promotion_input: pending`; checklist verified rows untouched

## Performance Metrics

| Phase | Plan | Duration | Tasks | Files |
|-------|------|----------|-------|-------|
| 28.1.1.3 | 01 | 2 min | 2 | 4 |
| 28.1.1.3 | 02 | 10 min | 2 | 7 |
| 28.1.1.3 | 03 | 1 min | 2 | 1 |
| 28.1.1.3 | 04 | (pending) | 2 | 5 |

## Session

- **Stopped at:** Completed 28.1.1.3-04-PLAN.md (gaps_found)
- **Resume:** Next inserted phase for `asic_enable_power_sequencing` (or Phase 29/30 path per orchestrator); do not claim STR-09/CFG-07/ASIC-11 verified

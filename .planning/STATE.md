---
gsd_state_version: 1.0
milestone: v1.1
milestone_name: milestone
status: Phase 28.1.1.4 in progress — plan 03 complete
stopped_at: Completed 28.1.1.4-03-PLAN.md
last_updated: "2026-07-09T04:04:56.551Z"
last_activity: 2026-07-09 -- Patch disposition falsified ticket-mask sole-blocker; ASIC-256 retained
progress:
  total_phases: 15
  completed_phases: 12
  total_plans: 54
  completed_plans: 53
  percent: 98
---

# Project State

## Current Position

- **Phase:** 28.1.1.4 — BM1366 ASIC Init-Content Sequencing Nonce-Production Diagnosis
- **Plan:** 03/04 complete (next: 04 final evidence)
- **Status:** executing — `patch_disposition: falsified_ticket_mask_asic_difficulty_as_sole_blocker`; `wire_parity_retained: true`; `next_hypothesis: match_upstream_chip_enumerate_before_init`

## Decisions (Phase 28.1.1.4)

- Skipped optional `asic_init_sequencing_summary` firmware marker; uart_trace suffices for Wave 0 (D-08 discretion)
- Init-sequencing recommender closed to `ticket_mask_asic_difficulty` | `match_upstream_chip_enumerate_before_init` | `none`
- HARD BAN: never emit `post_max_baud_delay_2000`, `match_upstream_register_read_poll`, `upstream_like_long_block_receive`
- Default Rust 512/1000 wire collision to `diff_1000` via pool_stratumdiff source preference
- Compare last mining-ready window before first `job_tx` (ignore early upstream `diff_16`)
- Ticket mask uses ASIC family difficulty 256; pool stratumdiff stays Stratum-only
- A/B `ab_outcome: unchanged` for `ticket_mask_asic_difficulty` — no correlate/submit; do not sole-blocker promote (D-11)
- Plan 03 next_hypothesis: `match_upstream_chip_enumerate_before_init` (D-05)
- `patch_disposition=falsified_ticket_mask_asic_difficulty_as_sole_blocker`; `wire_parity_retained` ASIC-256
- Chip-enumerate not implemented in Plan 03; deferred as next_hypothesis only

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
| 28.1.1.3 | 04 | 2 min | 2 | 5 |
| 28.1.1.4 | 01 | 7 min | 2 | 2 |
| 28.1.1.4 | 02 | 10 min | 2 | 10 |
| 28.1.1.4 | 03 | 1 min | 2 | 1 |

## Session

- **Stopped at:** Completed 28.1.1.4-03-PLAN.md
- **Resume:** Execute 28.1.1.4-04-PLAN.md (final evidence); do not claim STR-09/CFG-07/ASIC-11 verified

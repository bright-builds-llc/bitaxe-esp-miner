---
gsd_state_version: 1.0
milestone: v1.1
milestone_name: milestone
status: executing
stopped_at: Completed 28.1.1.5-03-PLAN.md
last_updated: "2026-07-09T12:31:07.238Z"
progress:
  total_phases: 16
  completed_phases: 13
  total_plans: 57
  completed_plans: 57
  percent: 100
---

# Project State

## Current Position

- **Phase:** 28.1.1.5 — BM1366 Match Upstream Chip-Enumerate Before Init Nonce-Production Diagnosis
- **Plan:** 3/4 — patch disposition complete; next final evidence
- **Status:** in progress — `patch_disposition: falsified_count_asic_chips_rx_loop_parity_as_sole_blocker`; `wire_parity_rx_loop_retained: true`
- **Next step:** Execute `28.1.1.5-04-PLAN.md` (final redacted evidence + VERIFICATION)

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
- Final `verification_result: gaps_found`; `phase30_promotion_input: pending`; checklist verified rows untouched

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
| 28.1.1.4 | 04 | 1 min | 2 | 5 |
| 28.1.1.5 | 01 | 2 min | 2 | 2 |
| 28.1.1.5 | 02 | 25 min | 2 | 7 |
| 28.1.1.5 | 03 | 1 min | 2 | 1 |

## Decisions (Phase 28.1.1.5)

- Wave 0: `forced_ab_label` defaults to `count_asic_chips_rx_loop_parity` for Ultra 205 TX-match + interval_256 + config_expected/immediate
- `recommended_investigation` closed to `match_upstream_chip_enumerate_before_init` | `version_rolling_negotiation` | `none`
- HARD BAN: never emit `post_max_baud_delay_2000`, `match_upstream_register_read_poll`, `upstream_like_long_block_receive`, `ticket_mask_asic_difficulty`
- `read_chip_id_byte_patch_recommended: false` always (D-02 frame already matched)
- Skipped optional firmware enumerate marker (D-11); uart_trace + `asic_chip_enumerate_summary` suffice
- Wave 0 comparator may rename forced_ab only from redacted evidence (`counted_chip_address_interval` or `enumerate_to_mining_ready_gap`)
- Keep ASIC-256 ticket-mask wire parity; no Phase 30 checklist verified edits
- If RX-loop A/B falsified with markers otherwise matching → `next_hypothesis: version_rolling_negotiation` (no second speculative patch)
- Default candidate `count_asic_chips_rx_loop_parity` with fixtures; empty-buffer ESP_ERR_TIMEOUT maps to idle exit
- A/B `ab_outcome: unchanged` — `counted_rx`/`drain_idle_like` matched but no correlate/submit; Plan 03 disposition; recommender hints `version_rolling_negotiation`
- `patch_disposition=falsified_count_asic_chips_rx_loop_parity_as_sole_blocker`; `wire_parity_rx_loop_retained: true`
- Plan 03 next_hypothesis: `version_rolling_negotiation` (not implemented; no second speculative patch)
- `phase30_promotion_input: pending`; ASIC-256 ticket mask retained; checklist verified rows untouched

## Session

- **Stopped at:** Completed 28.1.1.5-03-PLAN.md
- **Resume:** Execute `28.1.1.5-04-PLAN.md`

## Accumulated Context

### Roadmap Evolution

---
gsd_state_version: 1.0
milestone: v1.1
milestone_name: milestone
status: verifying
stopped_at: Completed 28.1.1.7-03-PLAN.md
last_updated: "2026-07-09T15:05:35.313Z"
progress:
  total_phases: 18
  completed_phases: 15
  total_plans: 65
  completed_plans: 65
  percent: 100
---

# Project State

## Current Position

- **Phase:** 28.1.1.7 — BM1366 Pool-Negotiated ASIC Mask Reload Nonce-Production Diagnosis
- **Plan:** 3/4 — Patch disposition complete; Plan 04 next
- **Status:** `patch_disposition: falsified_pool_negotiated_mask_asic_reload_as_sole_blocker`; `wire_parity_mask_reload_retained: true`; `next_hypothesis: remaining_nonce_production_blocker_narrowing`; Phase 30 checklist verified rows untouched
- **Next step:** Execute `28.1.1.7-04-PLAN.md` (final redacted evidence + VERIFICATION)

## Decisions (Phase 28.1.1.7)

- Forced A/B label: `pool_negotiated_mask_asic_reload` (D-04/D-12)
- Gate lever on `mask_reload_tx_observed` / `post_configure_runtime` after configure — not mask-value delta (D-05)
- Wave 0 comparator: `scripts/phase28.1.1.7-asic-mask-reload-compare.mjs` extending 28.1.1.6 taxonomy
- HARD BAN includes prior falsified knobs + `negotiated_version_mask_work_field_parity`
- Plan 02 hook: `apply_negotiated_version_mask` in production.rs; flush from live_stratum_runtime after configure + production_ready
- Plan 03: promote only if improved+correlate/submit; else evidence-named next_hypothesis (default placeholder `remaining_nonce_production_blocker_narrowing` if silent); no second speculative patch
- Plan 04: `passed` only with result_correlated + share submit; Phase 30 checklist verified rows untouched
- `forced_ab_label` defaults to `pool_negotiated_mask_asic_reload` when configure+mask_stored+mask_applied and `mask_reload_tx_observed` false (Plan 01)
- `mask_reload_tx_observed` true only for `post_configure_runtime` after configure — never prelude_3/init_register (Plan 01)
- `recommended_investigation` closed to `pool_negotiated_mask_asic_reload` | `remaining_nonce_production_blocker_narrowing` | `none` (Plan 01)
- Pending bit in live_runtime; flush `apply_negotiated_version_mask` when production_ready with `post_configure_runtime` markers (Plan 02)
- A/B `pool_negotiated_mask_asic_reload`: `mask_reload_tx_observed` true, `ab_outcome: unchanged` (no correlate/submit); Plan 03 disposition next (Plan 02)
- Comparator trusts explicit `post_configure_runtime` marker without `mining.configure` literal in monitor log (Plan 02)
- `patch_disposition: falsified_pool_negotiated_mask_asic_reload_as_sole_blocker`; keep wire-correct post-configure SetVersionMask reload (Plan 03)
- `next_hypothesis: remaining_nonce_production_blocker_narrowing` (A/B silent; no second speculative patch) (Plan 03)
- `wire_parity_mask_reload_retained: true`; `phase30_promotion_input: pending` (Plan 03)

## Decisions (Phase 28.1.1.6)

- `forced_ab_label` defaults to `negotiated_version_mask_work_field_parity` when configure+mask_stored and `mask_applied_to_work` false
- `recommended_investigation` closed to `negotiated_version_mask_work_field_parity` | `pool_negotiated_mask_asic_reload` | `none`
- HARD BAN includes `count_asic_chips_rx_loop_parity`; `asic_mask_reload_recommended` always false in Wave 0
- Skipped optional firmware markers (D-10); category markers in logs suffice for Wave 0
- Stop discarding `maybe_version_mask`; store on `MiningWork`; UART version stays base notify
- Compact `mask_applied_to_work=true` marker on WorkQueued when mask stored (D-10 Plan 02)
- A/B `ab_outcome: unchanged` for `negotiated_version_mask_work_field_parity` — falsify work-field-only lever; Plan 03 `next_hypothesis: pool_negotiated_mask_asic_reload`
- `patch_disposition=falsified_negotiated_version_mask_work_field_parity_as_sole_blocker`; `wire_parity_mask_on_work_retained: true`
- Plan 03 next_hypothesis: `pool_negotiated_mask_asic_reload` (not implemented; `asic_mask_reload_applied: false`; no second speculative patch)
- `phase30_promotion_input: pending`; ASIC-256 ticket mask + RX-loop retained; checklist verified rows untouched
- Plan 04 closed Wave 0 Nyquist (`wave_0_complete` / `nyquist_compliant`); ROADMAP 4/4 Gaps Found; handoff `pool_negotiated_mask_asic_reload`
- `verification_result=gaps_found` (no correlate/submit after work-field A/B)
- `next_hypothesis=pool_negotiated_mask_asic_reload`; mask-on-MiningWork + ASIC-256 + RX-loop retained
- `phase30_promotion_input=pending`; checklist verified rows untouched

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
| 28.1.1.5 | 04 | 2 min | 2 | 5 |
| Phase 28.1.1.6 P01 | 2 min | 2 tasks | 2 files |
| Phase 28.1.1.6 P02 | 11 min | 2 tasks | 7 files |
| Phase 28.1.1.6 P03 | 1 min | 2 tasks | 1 files |
| Phase 28.1.1.6 P04 | 5 min | 2 tasks | 5 files |
| Phase 28.1.1.7 P01 | 2 min | 2 tasks | 2 files |
| Phase 28.1.1.7 P02 | 14min | 2 tasks | 10 files |
| Phase 28.1.1.7 P03 | 1 min | 2 tasks | 1 files |

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
- Final `verification_result: gaps_found`; `phase30_promotion_input: pending`; ASIC-256 ticket mask retained; checklist verified rows untouched
- Plan 04 closed Wave 0 Nyquist (`wave_0_complete` / `nyquist_compliant`); ROADMAP 4/4 Gaps Found; handoff `version_rolling_negotiation`
- `verification_result=gaps_found` (no correlate/submit after RX-loop A/B)
- `next_hypothesis=version_rolling_negotiation`; `wire_parity_rx_loop_retained` + ASIC-256 retained
- `phase30_promotion_input=pending`; checklist verified rows untouched

## Session

- **Stopped at:** Completed 28.1.1.7-03-PLAN.md
- **Resume:** `/gsd-execute-phase 28.1.1.7`

## Accumulated Context

### Roadmap Evolution

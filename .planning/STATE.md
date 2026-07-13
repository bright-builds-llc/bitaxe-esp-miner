---
gsd_state_version: 1.0
milestone: v1.2
milestone_name: Ultra 205 Operator-Ready Runtime
status: defining_requirements
stopped_at: Milestone v1.2 started — defining requirements
last_updated: "2026-07-13T18:07:13.001Z"
last_activity: 2026-07-13
progress:
  total_phases: 0
  completed_phases: 0
  total_plans: 0
  completed_plans: 0
  percent: 0
---

# Project State

Last activity: 2026-07-13

## Current Position

Phase: Not started (defining requirements)
Plan: —

- **Phase:** Not started (defining requirements)
- **Plan:** —
- **Status:** Defining requirements
- **Next step:** Define v1.2 requirements and create a roadmap continuing after Phase 30.

## Project Reference

See `.planning/PROJECT.md` (updated 2026-07-13). Core value remains observable device-user parity on real Bitaxe hardware. Current focus is v1.2 operator-ready read-only telemetry, persistent settings, truthful provenance, and bounded runtime-health evidence without active hardware actuation or renewed mining diagnostics.

## Decisions (v1.1 Milestone Archive)

- v1.1 is administratively shipped with accepted gaps, not verified as full trusted production mining.
- STR-09, ASIC-11, and CFG-07 remain unresolved; Phase 30 selected the conservative no-promotion outcome.
- All 18 v1.1 phase directories and their evidence histories are archived under `.planning/milestones/v1.1-phases/`.
- Phase 28.1.1 and descendants remain terminal `gaps_found` work. They must not be recreated under `.planning/phases/` or resumed through explicit or autonomous GSD commands.
- Any future nonce-production or live-share work requires a new milestone, fresh requirements, and explicitly new evidence.
- GSD archive/progress lookup limitations are a tooling exception only; never silence them by changing verification truth or recreating active directories.

## Decisions (Phase 28.1.1 Closure)

- User directed deliberate closure as `Closed — Won't Do (unresolved)` so later phases can proceed.
- Plan 16 is administratively accounted for without execution after its one-shot preflight ended `preflight_identity_unavailable`; no retry or physical action occurred, and cleanup completed.
- Phase 28.1.1 remains `gaps_found` at 6/12. Firmware nonce production, hashing-capable state, correlated BM1366 result, and accepted/rejected live share remain unverified.
- STR-09, ASIC-11, and CFG-07 remain pending. Phase 30 was the only permitted continuation and later completed with no promotion.
- Do not resume Plan 16 or treat this closure as parity evidence.
- GSD variants that do not resolve active-milestone archives may produce eight W006 warnings for this lineage. The installed GSD currently introduces none; in either case, do not recreate active directories or promote verification to silence diagnostics.
- Installed atomic `find-phase` does not resolve milestone archives, and `init phase-op` returns the roadmap phase with a null directory; use lifecycle validation for allowed archive resolution, never recreate active stubs, and do not run explicit lineage operations.

## Decisions (Phase 28.1.1 Plan 15)

- Only OS-native no-reset capture can produce a native qualification; zero-byte, mixed-identity, incomplete-replay, cleanup-leaking, unknown-field, and UART inputs fail closed.
- Consume one qualification before creating a distinct formal authority on the same clean exact HEAD, carrying only contract and zero-resource handoff facts; qualification facts never populate product evidence.
- Terminal verification is read-only and only `passed_same_chain_hardware` may carry a positive verification projection; requirements traceability and Phase 30 remain pending.

## Decisions (Phase 28.1.1 Plan 11)

- The evidence-correctness gaps are closed: exact five-stage completeness, 180-second/2-second replay timing, measured 5000 ms USB absence, exact checkpoint deadlines, cleanup, and tri-state denylist behavior are regression guarded
- The strict five-stage 360-second reinit candidate belongs to `4e2d165`; `d275a0e` changed the hardware-attempt head, and independent review fix `ab7f5b9` changed classifier/process-cleanup code again, so both older package/checkpoint identities are stale and not promotable
- On hardware-attempt head `d275a0e`, initial `board-info` failed, USB replug was consumed, and the both-power checkpoint response reached continuation after its persisted monotonic deadline
- One post-expiry detector invocation succeeded but is invalid because continuation had not asserted expiry first; it is disclosed, unpromoted, and contributes no prerequisite claim
- Independent review found and `ab7f5b9` fixed three host-only gaps: orphan descendant watcher cleanup, token crossing its deadline during read, and Rust unavailable-observation precedence; focused process-tree/lifecycle/classifier regressions pass
- No `ab7f5b9` hardware access, credential access, package, flash, reset, or monitor capture occurred; cleanup verified no child remained
- The finite Plan 11 recovery contract is exhausted and cannot be refreshed or retried
- `verification_result=gaps_found`; `phase30_promotion_input=pending`; checklist verified rows untouched

## Decisions (Phase 28.1.1 Plan 10)

- Diagnostic-only replay selects exact complete markers, arms at `listener_armed`, emits with `log::info!` only, and is bounded to 90 seconds at a fixed 2-second cadence
- The retained-package lifecycle member prohibits package/flash/reset/detector/credential actions after arming and requires an exact five-stage reinit before requesting physical power action
- Two reinit attempts missed that prerequisite and final board-info failed; the checkpoint remained unarmed and Phase 30 remains pending

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
- Plan 04 closed Wave 0 Nyquist (`wave_0_complete` / `nyquist_compliant`); ROADMAP 4/4 Gaps Found; handoff `remaining_nonce_production_blocker_narrowing`
- `verification_result=gaps_found` (no correlate/submit after mask-reload A/B despite reload TX observed)
- `next_hypothesis=remaining_nonce_production_blocker_narrowing`; mask-reload + mask-on-MiningWork + ASIC-256 + RX-loop wire retained
- `phase30_promotion_input=pending`; checklist verified rows untouched

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
| Phase 28.1.1.7 P04 | 3 min | 2 tasks | 5 files |
| Phase 28.1.1 P07 | 28min | 2 tasks | 8 files |
| Phase 28.1.1 P08 | 15min | 3 tasks | 4 files |
| Phase 28.1.1 P10 | 39 min | 3 tasks | 14 files |
| Phase 28.1.1 P11 | bounded continuation | 4 tasks | 19 files |
| Phase 28.1.1 P12 | 33 min | 3 tasks | 10 files |
| Phase 28.1.1 P15 | 1h 12m | 3 tasks | 12 files |
| Phase 30 P01 | 13 min | 1 tasks | 6 files |
| Phase 30 P02 | 8 min | 2 tasks | 3 files |

### Quick Tasks Completed

| # | Description | Date | Commit | Status | Directory |
|---|-------------|------|--------|--------|-----------|
| 260712-0a9 | Always-on serial runtime heartbeat and Plan 13 fallback validation | 2026-07-12 | a38bb0f | Software verified; hardware transport blocked | [260712-0a9-implement-the-always-on-serial-only-runt](./quick/260712-0a9-implement-the-always-on-serial-only-runt/) |
| 260712-pw5 | Persist direct-UART and pin-manipulation authorization rule | 2026-07-12 | this commit | Plan 14 hardware cancelled; non-invasive replanning required | [260712-pw5-persist-repo-rule-prohibiting-assumed-di](./quick/260712-pw5-persist-repo-rule-prohibiting-assumed-di/) |
| 260713-p28 | Close Phase 28.1.1 without claiming unresolved parity | 2026-07-13 | this commit | Closed — Won't Do (unresolved); Phase 30 next | — |
| 260713-egi | Close Phase 28.1.1 lineage as terminal archived unresolved work and guard all reopening paths | 2026-07-13 | 2285ebe | Verified | [260713-egi-close-phase-28-1-1-and-descendants-as-ar](./quick/260713-egi-close-phase-28-1-1-and-descendants-as-ar/) |

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

- **Stopped at:** v1.1 archived with accepted unresolved gaps
- **Resume:** Start `/gsd-new-milestone`; preserve the terminal archive and unresolved STR-09, ASIC-11, and CFG-07 requirements as historical debt.

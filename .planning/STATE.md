---
gsd_state_version: "1.0"
milestone: v1.2
milestone_name: Ultra 205 Operator-Ready Runtime
status: executing
stopped_at: Phase 34 verification found three software gaps; gap planning is required before Phase 35.
last_updated: "2026-07-15T16:50:23.000Z"
last_activity: "2026-07-15"
progress:
  total_phases: 5
  completed_phases: 3
  total_plans: 13
  completed_plans: 13
  percent: 100
---

# Project State

Last activity: 2026-07-15

## Current Position

Phase: 34 (provenance-runtime-health-and-coherent-operator-snapshot) — GAPS FOUND
Plan: 4 of 4 complete

- **Phase:** 34 of 35 (provenance, runtime health, and coherent operator snapshot)
- **Plan:** 4 of 4 implemented
- **Status:** Verification gaps found; 6/10 requirements satisfied
- **Next step:** Plan Phase 34 gap closure for factory-image admission, supervisor checkpoint liveness, and ordered snapshot publication. Phase 35 remains blocked.

## Project Reference

See `.planning/PROJECT.md` (updated 2026-07-14). Core value remains observable device-user parity on real Bitaxe hardware. Current focus is v1.2 operator-ready read-only telemetry, persistent settings, truthful provenance, and bounded runtime-health evidence without active hardware actuation or renewed mining diagnostics.

## Decisions (v1.2 Roadmap)

- v1.2 contains five ordered phases, 31 through 35, with all 27 actionable requirements mapped exactly once.
- Phase 31 defines claim and telemetry semantics before effectful integration; Phase 32 establishes one read-only I2C0 producer; Phase 33 confirms hostname storage truth; Phase 34 composes one coherent operator snapshot; Phase 35 owns final detector-gated evidence admission.
- The complete v1.2 PATCH allowlist is `hostname`; additional settings remain future work.
- Active fan, voltage, reset, power, fault, ASIC, and self-test effects are prohibited. Mining and Phase 28.1.1 lineage work, credentials, direct UART/pins, OTA, other boards, and broad parity promotion are also prohibited.
- Only Phase 35 may admit final v1.2 parity evidence, and only for explicitly allowlisted operator-runtime rows supported by an eligible same-chain evidence root.

## Decisions (Phase 31 Plan 01)

- Power current, bus voltage, and wattage share one stamped acquisition because INA260 supplies them atomically.
- Temperature and tachometer use independent observations so one producer failure cannot erase the other fact.
- Compatibility numeric fallbacks remain projections outside observation truth and cannot authenticate freshness.

## Decisions (Phase 31 Plan 02)

- Observation truth serializes independently from AxeOS numeric compatibility values; a compatibility zero cannot authenticate freshness, availability, or health.
- Firmware consumers clone one complete stored snapshot, and only producer completion may replace it, so request traffic cannot acquire sensors or advance metadata.
- The retained Phase 27 path leaves fan RPM unavailable until a producer owns an independent stamp; reusing another fact's stamp would manufacture provenance.

## Decisions (Phase 31 Plan 03)

- Only exact validated hostname input constructs v1.2 settings authority; compatibility parsing remains broad for stable responses while persistence integration stays Phase 33-owned.
- Phase 31 claim admission is closed, typed, and requirement-scoped: OBS-01 and CFG-08 evidence authenticate only their exact claims, while excluded categories, strings, and schema growth fail closed.

## Decisions (Phase 32 Plan 01)

- One validated INA260 triple advances one shared power observation sequence; a partial or invalid attempt advances nothing.
- Temperature and tachometer retain separate outcomes, sequences, and last-good stamps.
- Only producer-supplied monotonic time may transition fresh observations to stale.

## Decisions (Phase 32 Plan 02)

- All I2C operations convert a named 50 ms bound with `TickType` before calling ESP-IDF.
- The startup display borrows the shared driver through a bounded embedded-hal adapter and never selects `BLOCK`.
- Normal sensor code can name only the seven allowlisted read registers; active writes require a token constructible only inside the gated Phase 27 module.

## Decisions (Phase 32 Plan 03)

- The normal producer attempts power, temperature, and tachometer once per sweep before replacing one complete observation snapshot.
- Missed 500 ms deadlines skip to the next future slot, preventing retry loops and catch-up bursts.
- Phase 32 admits software evidence only; hardware remains pending until a wrapper records the complete private session trace and separate sanitized summary.

## Decisions (Phase 33 Plan 01)

- Known-field compatibility validation always precedes exact hostname-only authority; valid mixed or excluded requests remain inert.
- A lifetime-bound transaction owns serialization from the first hostname mutation through strict reload, typed reconciliation, and confirmed publication.
- Post-commit reload, mismatch, or publication failures retain explicit uncertainty and never claim or attempt rollback.

## Decisions (Phase 33 Plan 02)

- Writable NVS opens only after exact hostname authority and under one transaction lock held through confirmed publication.
- Strict independent reload remains non-publishing until typed exact reconciliation succeeds; requested values never overlay public truth.
- Compatibility-only and invalid requests never construct the adapter, and hostname application is the sole post-response live effect.

## Decisions (Phase 33 Plan 03)

- Reset-spanning evidence uses an RTC-backed boot ordinal and bounded typed boot/origin replay because native USB may lose early bytes across an application reset.
- The exact `a630455` package passed the sole detector-gated, one-restart hardware attempt with cleanup and restoration; it remains credible historical exact-source evidence.
- Subsequent review fixes materially changed current firmware snapshot and deferred restart/effect behavior, so the `a630455` proof cannot qualify current firmware and remains non-promotional.
- Phase 33 is complete for CFG-09, CFG-10, CFG-11, and CFG-13. CFG-12 remains pending under Phase 35, whose final detector-gated exact-current-package run must jointly close CFG-12 and EVD-13 without weakening source, identity, cleanup, redaction, restoration, or no-retry gates.
- No additional Phase 33 hardware attempt is permitted.

## Decisions (Phase 34 Plan 01)

- One shared Rust authority derives the full source identity, short human label, release/dev channel, scoped dirty state, semantic version, and pinned reference commit from the closed five-key Bazel workspace status.
- LCD, system-info, live WebSocket, retained logs, ESP-IDF application metadata, and manifest schema v3 consume the same declared provenance stamp; presentation labels never authenticate source.
- Clean tagged and clean untagged dev packages are eligible. A dirty package or dirty current workspace fails before port discovery, credential reads, or any device action.
- Managed `esptool.py elf2image --elf-sha256-offset 0xb0` owns the application descriptor SHA because `espflash save-image` left that field zero; factory packaging uses the one generated ESP-IDF build whose sdkconfig matches the exact build label.
- Clean exact commit `694cf0ceb72c78fd16b20bc57beeac914f098ac6` passed package, descriptor, digest, and inert-port admission checks. No hardware or credentials were used.

## Decisions (Phase 34 Plan 02)

- The existing hardware-RNG boot observer supplies the only operator-snapshot session; firmware owns one checked revision sequence for every public capture.
- A retained correlation marker is emitted only after one complete `ApiSnapshot` is assembled, and system-info plus live WebSocket copy the attached identity without generating their own.
- Historical Phase 23/25/27/28 operator-evidence acceptance remains unchanged; OBS-06 coherence is an explicit fail-closed opt-in.

## Decisions (Phase 34 Plan 03)

- Every platform fact carries its own available or unavailable state; zero and compatibility defaults never authenticate proof.
- The embedded static release asset and current ESP-IDF reads are the only production sources for running-platform identity.
- Existing compatibility scalars are projected conservatively from the same typed candidate captured under one Plan 02 session and revision.

## Decisions (Phase 34 Plan 04)

- Runtime health remains an immutable captured value: the pure evaluator receives only producer-owned observations and monotonic time, while public and retained surfaces project the same snapshot.
- Supervisor checkpoint visibility and ESP task-watchdog participation are independent; Phase 34 reports watchdog participation unavailable with reason `unproved`.

## Decisions (Phase 34 Verification)

- Phase 34 verification is `gaps_found` at 6/10 requirements; green unit and Bazel suites do not override reachable production-path defects.
- OBS-06 remains pending because concurrent HTTP/WebSocket captures can reserve revisions and retain or publish them in decreasing completion order.
- SYS-02 remains pending because normal flashing selects the factory image while identity admission authenticates markers only in its OTA sibling, without proving app-byte equivalence.
- HLT-02 and HLT-04 remain pending because duplicate yield-log suppression returns before recurring supervisor checkpoint publication, making a live loop appear stale and unhealthy.
- Phase 35 is blocked until deterministic software gap closure and fresh Phase 34 verification pass; no hardware can repair or disprove these defects.

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
| Phase 31 P01 | 12 min | 2 tasks | 6 files |
| Phase 31 P02 | 15 min | 2 tasks | 15 files |
| Phase 31 P03 | 14 min | 2 tasks | 7 files |
| Phase 32 P01 | 17 min | 2 tasks | 3 files |
| Phase 32 P02 | 21 min | 2 tasks | 17 files |
| Phase 32 P03 | 15 min | 3 tasks | 7 files |
| Phase 33 P01 | 12 min | 2 tasks | 5 files |
| Phase 33 P02 | 9min | 2 tasks | 7 files |
| Phase 33 P03 | 26min | 2 tasks | 29 files |
| Phase 34 P02 | 30min | 3 tasks | 15 files |
| Phase 34 P03 | 18min | 2 tasks | 12 files |
| Phase 34 P04 | 19 min | 1 tasks | 20 files |

### Quick Tasks Completed

| # | Description | Date | Commit | Status | Directory |
|---|-------------|------|--------|--------|-----------|
| 260712-0a9 | Always-on serial runtime heartbeat and Plan 13 fallback validation | 2026-07-12 | a38bb0f | Software verified; hardware transport blocked | [260712-0a9-implement-the-always-on-serial-only-runt](./quick/260712-0a9-implement-the-always-on-serial-only-runt/) |
| 260712-pw5 | Persist direct-UART and pin-manipulation authorization rule | 2026-07-12 | this commit | Plan 14 hardware cancelled; non-invasive replanning required | [260712-pw5-persist-repo-rule-prohibiting-assumed-di](./quick/260712-pw5-persist-repo-rule-prohibiting-assumed-di/) |
| 260713-p28 | Close Phase 28.1.1 without claiming unresolved parity | 2026-07-13 | this commit | Closed — Won't Do (unresolved); Phase 30 next | — |
| 260713-egi | Close Phase 28.1.1 lineage as terminal archived unresolved work and guard all reopening paths | 2026-07-13 | 2285ebe | Verified | [260713-egi-close-phase-28-1-1-and-descendants-as-ar](./quick/260713-egi-close-phase-28-1-1-and-descendants-as-ar/) |
| 260714-unf | Remap CFG-12 from Phase 33 to Phase 35 and re-verify Phase 33 without hardware | 2026-07-15 | 94ac313 | Verified | [260714-unf-remap-cfg-12-from-phase-33-to-phase-35-a](./quick/260714-unf-remap-cfg-12-from-phase-33-to-phase-35-a/) |

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

- **Stopped at:** Completed 34-04-PLAN.md
- **Resume:** Plan Phase 35 without hardware use until its detector-gated evidence workflow explicitly authorizes the next bounded action.

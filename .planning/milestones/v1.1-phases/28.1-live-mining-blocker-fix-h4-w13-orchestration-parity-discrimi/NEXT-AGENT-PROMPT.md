# Agent Handoff Prompt: Fix BM1366 Nonce Production in the Rust Firmware

You are picking up a precisely-scoped firmware bug in the repo at `/Users/peterryszkiewicz/Repos/bitaxe-esp-miner` (Bitaxe Ultra 205, BM1366 ASIC, ESP32-S3, Rust + ESP-IDF, upstream C reference pinned read-only at `reference/esp-miner`, pin `c1915b0a63bfabebdb95a515cedfee05146c1d50`).

## Mission

Make the BM1366 actually hash under the Rust firmware, producing a correlated nonce (`asic_production_status=result_correlated`) and then at least one real accepted or rejected Stratum share on the connected Ultra 205. That is the only remaining gap between this firmware and live mining.

## Current state — established facts (do NOT re-derive or re-litigate these)

Phase 28.1 (completed 2026-07-07, disposition `firmware-nonce-production`) conclusively isolated the blocker. Read `.planning/phases/28.1-live-mining-blocker-fix-h4-w13-orchestration-parity-discrimi/28.1-blocker-disposition.md` first — it is the authoritative evidence record. Summary:

**Proven working under Rust firmware (hardware-verified in run J3):**
- Safety bring-up: power/thermal/fan/asic-enable complete; vcore DAC programmed to 1.200 V (DS4432U math verified equal to upstream).
- Chip detect at 115200 baud: valid 11-byte chip-id response (0x1366).
- Mining-ready init: every static register frame byte-matches upstream fixtures (`crates/bitaxe-asic/src/bm1366/mining_ready.rs` tests); CRC5 and CRC16 verified equivalent to upstream `crc.c`; PLL search math verified equivalent to upstream `pll.c`; frequency ramp 50→485 MHz default-on.
- Baud switch to 1 MBaud: **proven working** — the post-init register-read probe (read reg 0x00) was answered by the chip at 1 MBaud, both post-init and post-dispatch.
- Orchestration (fixed in 28.1): dispatch-before-poll `BridgeOrchestrator`, continuous result listener, ~2 s job regeneration with fresh extranonce2. J3 showed 22 `work_dispatched` at 2050–2110 ms spacing, extranonce2 counter 1..19. Zero fail-closed timeouts on the default path.
- Stratum session: connect/subscribe/authorize/notify/work-build all live against the real pool.

**Proven broken:** the chip never hashes. INA260 power delta after dispatch was −390 mW (falling, not rising; hashing at 485 MHz should add several watts). Zero nonces in 360 s across 22 dispatched jobs.

**Proven NOT the cause (exhausted in phases 27 + 28.1 — do not retest):** init frame encoding (W6/W11), baud domains (run C at 115200 was also silent; probe proves 1M RX), read-window length (W10, F1), boot diagnostic pollution (H1), frequency single-set vs ramp alone (W8, G/H runs), post-baud delays (W9), address interval (W3), reg 0x28 ordering (W7), orchestration/dispatch starvation (J2c bug found and fixed; falsified in J3), UART RX path, chip health, board health.

**Hardware is healthy:** stock upstream ESP-Miner v2.14.0 (`esp-miner-factory-205-v2.14.0.bin`, sha256 `77ea24e81a92fa11a6d3ae7f1da89b533143849b3386766ce6ba06ae05dc161d`) was flashed on the SAME board in the same session and mined immediately: chip detect, init at 485 MHz / 1200 mV, first nonce ~17.5 s after boot, 171 nonce results, 14 submits, 11 accepted in 380 s. Recovery to Rust firmware was executed and verified.

## The suspect space (prioritized)

The chip receives valid-looking frames, answers register reads, but its cores never start. The divergence must be in the *content or sequence* of what the Rust firmware programs vs what upstream programs. Priorities:

1. **The three dynamic init frames were never fixture-asserted against upstream.** In `mining_ready.rs`, the test `mining_ready_init_frames_match_upstream_fixtures` asserts every static frame but skips frames[6] (difficulty/ticket mask, reg 0x14), frames[15] (frequency PLL, reg 0x08), and frames[16] (nonce space/HCN, reg 0x10) because they are computed at runtime. The formulas were reviewed as equivalent to upstream, but the *actual runtime byte values* have never been compared to what upstream actually sends. A wrong ticket mask silences all nonce reporting; a wrong PLL word stops the core clocks; both match the observed symptom exactly.
2. **Version rolling divergence (observed fact, not yet a conclusion).** Upstream v2.14.0 negotiated `mining.configure` version-rolling with the pool and its nonce results carried nonzero `version_bits` (hardware version rolling active). The Rust client does not negotiate a version mask (explicitly deferred in 28.1-CONTEXT.md), and `crates/bitaxe-stratum/src/v1/mining.rs` returns an error for nonzero version masks in work generation. The chip-detect prelude and init do write the version-rolling registers (3× version mask + reg 0xA4 `90 00 FF FF`), so the chip *should* roll versions over the job's base version — but the interaction between the job's version field value and the rolled-version config is a prime candidate for the wire diff.
3. **Job frame field content.** The 88-byte job frame structure matches self-derived goldens, but no golden was ever captured from real upstream traffic (`work-result-diagnostic-vs-upstream-send-work.md` flagged this). Field-level suspects: `nbits` (upstream fills it from `bm_job.target`), `ntime`, `version` byte order, `starting_nonce`, merkle/prev-hash word-reversal.
4. **Init sequencing around chip detect.** Rust: reset → 3× version mask → read chip id → init writes. Upstream: reset → 3× version mask → `55 AA 52 05 00 00 0A` enumerate → `count_asic_chips` → init writes. Any missed inter-command delay or ordering nuance will show up in the wire capture timestamps.

## Recommended plan of attack

Work through the repo's GSD workflow: insert a phase with `/gsd-insert-phase 28.1 <description>` (creates 28.2), then discuss/plan/execute (`--yolo` variants are the established habit in this repo). The approved-fallback framework and "3 failed fixes → stop and question architecture" rule from `~/.claude/plans/can-you-study-our-soft-harp.md` still apply.

**Step 1 — Capture upstream golden wire bytes (the decisive evidence).**
Build upstream from the pin OUT-OF-TREE (never modify `reference/esp-miner`; copy to a gitignored scratch dir) with `BM1366_SERIALTX_DEBUG` and `BM1366_SERIALRX_DEBUG` enabled (defined in `reference/esp-miner/components/asic/bm1366.c`), flash it (same A/B procedure + NVS seed as documented in the disposition doc, recovery path included), and capture the full init + first-minute job byte stream over the monitor. This yields the exact bytes that make this chip hash: the runtime ticket-mask frame, PLL frame, HCN frame, and real job frames.

**Step 2 — Capture Rust TX bytes for the same window.**
Add a compile-gated (investigation-mode) TX hex trace to the Rust UART write path (`firmware/bitaxe/src/asic_adapter/uart.rs` `write_frame`), or derive frames host-side from the same code paths. Keep raw byte logs LOCAL/gitignored (job frames embed pool-derived merkle/extranonce data — the redaction rules forbid committing them). Commit only field-level diff conclusions with category labels.

**Step 3 — Diff and fix, one divergence at a time.**
Diff init sequences byte-for-byte (values AND ordering AND timing), then job frames field-by-field. For each divergence: form a single hypothesis, make the smallest fix, add a fixture test pinning the upstream-captured bytes (with provenance comment), re-run the J3 evidence ladder (`just detect-ultra205` → `bash scripts/phase27-live-hardware-bridge-package.sh` → `just flash-monitor board=205 port=<port> capture-timeout-seconds=360 wifi-credentials=wifi-credentials.json ...`). The ladder markers to grep are documented in `28.1-run-J3-parity-default.md` and `28.1-flag-disposition-note.md`. Success rung: `result_correlated`, then a submit classified accepted/rejected.
- The INA260 power-delta probe (`asic_probe=power_delta`) tells you within ~10 s whether a fix made the cores start — use it as the fast feedback signal before waiting on nonces.

**Step 4 — If wire bytes match but the chip still doesn't hash:** implement pool version-rolling negotiation (`mining.configure` + version mask in work generation + rolled-version share submission) as full upstream parity. This is currently deferred scope, so record the scope change in the new phase's CONTEXT.

**Step 5 — On success:** the Share tier evidence enables STR-09/CFG-07 checklist promotion, but ONLY via the Phase 28 validator rules (`tools/parity/src/main.rs` `validate_phase28_hardware_promotion_row`, `--fail-on-invalid-verified`). Never hand-edit `docs/parity/checklist.md` beyond what the validator accepts.

## Non-negotiable repo rules (from CLAUDE.md / AGENTS.md — read them in full before editing)

- GSD workflow required for file changes; standalone `---` only as frontmatter delimiters in `.planning` Markdown.
- Rust pre-commit gates before EVERY commit: `cargo fmt --all && cargo clippy --all-targets --all-features -- -D warnings && cargo build --all-targets --all-features && cargo test --all-features`; firmware also `just build` (xtensa). Executable tests live in `crates/*` only — firmware `#[cfg(test)]` never runs on host.
- Hardware: `just detect-ultra205` gate before ANY flash/monitor; `capture-timeout-seconds>=360`, shell wall clock >= 420 s (use 600000 ms tool timeouts); do not treat early exit as failure before the timeout elapses. Standing permission covers autonomous Ultra 205 use under these gates; upstream reflash is allowed with the documented recovery path (see disposition doc, executed and verified once already).
- Credentials: `wifi-credentials.json` / `pool-credentials.json` are pass-by-path runtime inputs. Never read/print/commit contents.
- Redaction: committed evidence uses category labels only — no pool URLs/ports/users/workers/passwords, IPs, MACs, SSIDs, device URLs, share payloads, extranonces, raw job bytes, or NVS secrets. Run the redaction grep sweep from `28.1-05-PLAN.md` before committing any run doc.
- Licensing: `reference/esp-miner` is read-only GPL evidence. Breadcrumb comments only; never copy upstream expression into MIT files. Byte fixtures captured from a running device are fine with provenance notes.
- Fail-closed default build must survive every change: no `BITAXE_MINING_EVIDENCE_MODE`/`BITAXE_HARDWARE_EVIDENCE_ACK` compile envs → no mining, no ASIC I/O.

## Key files

| Area | Path |
| --- | --- |
| Evidence record (read first) | `.planning/phases/28.1-live-mining-blocker-fix-h4-w13-orchestration-parity-discrimi/28.1-blocker-disposition.md`, `28.1-run-J3-parity-default.md`, `28.1-run-J4-control-single-dispatch.md`, `28.1-VERIFICATION.md` |
| Dynamic init frames (suspect #1) | `crates/bitaxe-asic/src/bm1366/mining_ready.rs` (`difficulty_mask_value`, `hash_counting_number`), `crates/bitaxe-asic/src/bm1366/frequency_voltage.rs` (PLL), `crates/bitaxe-asic/src/bm1366/registers.rs`, `command.rs` |
| Job frame build | `crates/bitaxe-asic/src/bm1366/work.rs`, `production.rs`; `crates/bitaxe-stratum/src/v1/mining.rs` (`build_work_fields_with_extranonce2`) |
| Firmware UART / probes | `firmware/bitaxe/src/asic_adapter/uart.rs`, `asic_adapter/production.rs`, `asic_adapter.rs` (init bootstrap), `firmware/bitaxe/src/live_stratum_runtime.rs` (bridge pump) |
| Orchestrator (done — don't rework) | `crates/bitaxe-stratum/src/v1/bridge_orchestration.rs`, `live_runtime.rs` (`regenerate_work`) |
| Upstream reference anchors | `reference/esp-miner/components/asic/bm1366.c` (`BM1366_init`, `BM1366_send_work`, `BM1366_send_hash_frequency`, `BM1366_set_version_mask`), `asic_common.c` (`get_difficulty_mask`, `receive_work`), `pll.c`, `main/tasks/create_jobs_task.c`, `asic_result_task.c` |
| Build/flash surface | `Justfile`, `scripts/phase27-live-hardware-bridge-package.sh`, `tools/flash` |

## Definition of done

1. Rust firmware on the Ultra 205 logs `asic_production_status=result_correlated` (a real BM1366 nonce correlated to live pool work), with rising INA260 power delta corroborating hashing.
2. At least one `mining.submit` classified accepted or rejected by the live pool, captured in a redacted, committed run doc.
3. All host gates green; fail-closed default build intact; regression: `single_dispatch_bounded_read` control lever still reachable.
4. Checklist rows promoted only through the parity validator; exact non-claims preserved for anything not proven.

# Bitaxe Rust Firmware

## What This Is

Bitaxe Rust Firmware is a Rust ESP-IDF firmware monorepo for Bitaxe ESP-Miner. It builds a maintainable Rust implementation of the Bitaxe ESP32 miner firmware with device-user parity against upstream `bitaxeorg/ESP-Miner`, while keeping the upstream C code as a pinned read-only reference implementation.

The project is for Bitaxe owners and firmware contributors who need firmware that can be built, flashed, configured, monitored, updated, and audited with the same observable behavior they expect from upstream ESP-Miner.

## Core Value

A Bitaxe owner can build, flash, run, configure, monitor, and update Rust firmware on real Bitaxe hardware with the same observable behavior they expect from upstream ESP-Miner.

## Current State

v1.0 Ultra 205 Parity shipped on 2026-07-04 with 64/64 requirements satisfied across 21 phases, 116 plans, and 226 tasks.

v1.1 Ultra 205 Trusted Production Mining shipped administratively on 2026-07-13 with accepted gaps across 18 phases, 76 plans, and 170 tasks. The milestone delivered the safety-gated BM1366/Stratum software path, telemetry and evidence automation, and hardware-backed blocker isolation. It did not prove live Rust nonce correlation or a live share outcome: 18/21 requirements are satisfied, while STR-09, ASIC-11, and CFG-07 remain unresolved. Phase 30 closed with a conservative no-promotion disposition.

Historical milestone roadmaps, requirements, audits, and v1.1 phase artifacts live under `.planning/milestones/`.

## Current Planning State

No milestone is active. `/gsd-new-milestone` must create fresh requirements and continue roadmap phase numbering after Phase 30. The archived Phase 28.1.1 lineage is terminal and must not be resumed, recreated under `.planning/phases/`, or treated as passed verification.

## Next Milestone Goals

Potential future work includes a newly scoped nonce-production investigation backed by genuinely new evidence, OTA/recovery completion, broader active-safety hardware closure, non-205 board evidence, Stratum v2, BAP, and runtime display/input parity. None is selected until the next milestone is discussed.

## Requirements

### Validated

- Rust ESP-IDF firmware monorepo with Bazel as canonical automation and `just` as the human command surface — v1.0.
- Pinned read-only ESP-Miner reference, device-user parity scope, provenance controls, release workflows, Ultra 205 hardware foundation, and exact-claim evidence governance — v1.0.
- Claim-ladder governance and fail-closed production-mining prerequisite contracts — v1.1.
- Redacted operator evidence profiles, runtime-only secret inputs, deterministic validators, and atomic evidence consolidation — v1.1.
- BM1366 production-work modeling, generation tracking, stale-work invalidation, and fail-closed runtime status — v1.1 at implemented and workflow-evidence scope.
- Real Stratum v1 socket lifecycle, fake-pool coverage, bounded safe stop, watchdog checkpoints, and telemetry projection into API/WebSocket/statistics/scoreboard views — v1.1 within the exact evidence boundaries recorded by the archive.
- Hardware-backed isolation of the remaining Rust firmware nonce-production blocker and preservation of evidence-supported wire-parity corrections — v1.1 without live-result promotion.

### Accepted Unresolved Debt

- **STR-09:** No eligible live ASIC-derived `mining.submit` response was classified as accepted, rejected, or safely blocked under the final evidence contract.
- **ASIC-11:** No live BM1366 nonce/result was correlated back to active pool work.
- **CFG-07:** Runtime-only credential handling exists, but no eligible live evidence root proved it end to end.

### Active

None. The next milestone must define a new requirements file.

### Out of Scope Until Replanned

- Rewriting the Angular AxeOS UI; API and static-asset compatibility remain the project boundary.
- Bare-metal `no_std` as the first production stack.
- Modifying files inside `reference/esp-miner`.
- Claiming non-205 boards, other ASIC families, active hardware safety, recovery/rollback, OTAWWW, display/input, BAP, Stratum v2, or unbounded mining without their own evidence.
- Treating controlled no-share, synthetic, package-only, or archived diagnostic evidence as proof of accepted/rejected production share behavior.

<details>
<summary>v1.1 pre-closure planning snapshot</summary>

The v1.1 goal was to turn the v1.0 controlled no-share foundation into a trusted, safety-gated Ultra 205 Stratum v1 runtime that observed at least one real accepted or rejected share and reported it through firmware telemetry and redacted evidence. Target features included real socket I/O, trusted BM1366 initialization/work/result handling, mining-prerequisite safety, bounded safe stop, live telemetry, and exact parity promotion. The software and governance surfaces were delivered; the live nonce/result/share target remained unresolved and was not promoted.

</details>

## Context

The accepted brief lives at `docs/project/gsd-new-project-brief.md`, decision records under `docs/project/` and `docs/adr/`, the exact-claim checklist at `docs/parity/checklist.md`, release documentation under `docs/release/`, and provenance policy in `PROVENANCE.md`.

The first verified hardware target is Bitaxe Ultra 205 with BM1366. Gamma 601 with BM1370 and other upstream boards remain in scope but require separate evidence before any verified hardware claim.

The monorepo separates hardware-bound firmware under `firmware/bitaxe` from testable Rust logic under `crates/`, host tools under `tools/`, repository automation under `scripts/`, and the protected upstream reference under `reference/esp-miner`. At v1.1 closure, the Rust source tree contains 56,967 lines.

## Constraints

- **Tech stack:** Use ESP-IDF Rust bindings for production firmware services.
- **Build orchestration:** Use Bazel as the canonical automation graph and `just` as the human command surface.
- **Reference implementation:** Keep `reference/esp-miner` pinned and read-only.
- **Parity evidence:** Implemented code is never sufficient by itself for a parity claim.
- **Hardware priority:** Ultra 205 BM1366 is the first verified target; every other board requires separate evidence.
- **Architecture:** Prefer a functional core and imperative shell.
- **Licensing:** Keep original work MIT-first, isolate intentional GPL-covered expression, and review distributed artifacts.
- **Safety:** Hardware-control and mining claims require exact hardware evidence and fail-closed behavior.

## Key Decisions

| Decision | Rationale | Outcome |
| --- | --- | --- |
| Device-user parity defines rewrite scope. | Observable behavior matters more than preserving C internals. | Accepted in v1.0. |
| ESP-IDF Rust is the production stack; Bazel owns automation and `just` owns ergonomics. | The project needs ESP-IDF services and a reproducible shared graph. | Accepted in v1.0. |
| Upstream ESP-Miner stays read-only and parity claims follow exact evidence. | Reference changes and task-completion claims could hide unsupported parity. | Enforced throughout v1.0 and v1.1. |
| Ultra 205 BM1366 is the first hardware target. | Available hardware supports board-specific evidence without extrapolating to other devices. | Validated; other boards remain future scope. |
| Production-mining promotion is fail closed. | Safety, credentials, live results, and pool outcomes must remain independently evidenced. | Prevented STR-09, ASIC-11, and CFG-07 overclaiming. |
| Phase 29 evidence profiles and Phase 30 admission are typed and atomic. | Partial or mixed evidence roots must not advance parity. | Delivered; Phase 30 chose no promotion. |
| Phase 28.1.1 closed as Won't Do (unresolved). | Repeated diagnostics narrowed but did not close the nonce-production gap. | All lineage verification remains `gaps_found` and is terminally archived. |
| Archived diagnostic work cannot be reopened by autonomous routing. | Historical artifacts must not become executable work or synthetic proof. | Phase 30 is complete; future work requires a new milestone and new evidence. |

## Evolution

This document evolves at milestone boundaries. The next update occurs when `/gsd-new-milestone` defines the next active requirements and roadmap.

*Last updated: 2026-07-13 after archiving v1.1 with accepted unresolved gaps*

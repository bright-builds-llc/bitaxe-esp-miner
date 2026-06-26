---
generated_by: gsd-discuss-phase
lifecycle_mode: yolo
phase_lifecycle_id: 1-2026-06-21T00-30-20
generated_at: 2026-06-26T14:20:22Z
---

# Phase 1: Foundation And Ultra 205 Boot/Log - Context

**Gathered:** 2026-06-26
**Status:** Ready for planning
**Mode:** Yolo

<domain>
## Phase Boundary

Phase 1 establishes the repo foundation for the Rust ESP-IDF firmware rewrite: the pinned read-only ESP-Miner reference, Bazel/Cargo/Just automation skeleton, crate and firmware layout, package/flash/monitor workflows, parity/provenance tooling, and a minimal Ultra 205 boot/log firmware path. Mining, ASIC work submission, voltage/fan/thermal control, full AxeOS API behavior, OTA, and non-205 verification are outside this phase. This supersedes the earlier Gamma 601-first Phase 1 direction per ADR-0014; Gamma 601 remains deferred and must not inherit Ultra 205 evidence.

</domain>

<decisions>
## Implementation Decisions

### Reference Guardrails

- **D-01:** Add `reference/esp-miner` as the pinned upstream ESP-Miner reference and treat it as read-only project evidence, not an editable workspace.
- **D-02:** Implement `just verify-reference` and the Bazel-visible reference guard so missing, dirty, or unpinned reference state fails clearly before parity evidence or fixtures are trusted.
- **D-03:** Record the pinned reference commit in package/parity outputs where relevant so later evidence can be tied to an exact comparison baseline.

### Automation And Workspace Skeleton

- **D-04:** Use Bazel/Bzlmod as the canonical automation graph, with `just` as a thin human command surface that delegates to Bazel targets or repo-owned scripts represented in Bazel.
- **D-05:** Create the planned monorepo shape up front: `firmware/bitaxe`, pure crates under `crates/`, host tooling under `tools/`, and scripts under `scripts/`.
- **D-06:** Keep Cargo metadata authoritative for Rust dependencies while Bazel mirrors or wraps the Rust work; do not attempt a full custom Bazel-native ESP-IDF toolchain in Phase 1.
- **D-07:** Pin the first firmware baseline to the accepted ESP-IDF Rust stack: ESP-IDF `v5.5.4`, `xtensa-esp32s3-espidf`, Rust 2021 for firmware/shared crates, and the ESP Rust toolchain installed through `espup`.

### Safe Ultra 205 Boot/Log

- **D-08:** The first firmware image should boot/log only. It must make the safe state explicit: mining disabled, ASIC work submission disabled, and hardware control disabled or safe-no-op.
- **D-09:** Boot logs should include firmware identity, firmware/source commit when available, ESP-IDF/Rust/toolchain identity when available, reset reason, partition/image identity, platform/PSRAM status, selected board target `Ultra 205`, selected ASIC target `BM1366`, and reference commit when known.
- **D-10:** Real Ultra 205 hardware evidence is required before boot/log behavior is marked verified. Gamma 601 hardware smoke is superseded by ADR-0014 and remains deferred until it has its own evidence set.

### Package, Flash, And Monitor Ergonomics

- **D-11:** Implement `just build`, `just test`, `just package`, `just flash`, `just monitor`, `just flash-monitor`, `just verify-reference`, and `just parity` in Phase 1 even if some targets initially wrap scripts or provide narrow skeleton behavior.
- **D-12:** `just flash board=205` should build/package first by default, accept `port=...`, discover likely ports when omitted, fail clearly on no or ambiguous ports, and print the underlying `espflash` command before executing.
- **D-13:** `just package` should produce a machine-readable package manifest with image paths, offsets when applicable, checksums, tool versions, firmware commit, and reference commit.

### Parity And Provenance Evidence

- **D-14:** Treat `docs/parity/checklist.md` as an evidence ledger. Phase 1 should wire `just parity` so checklist rows, statuses, missing evidence, implementation pointers, and breadcrumbs are inspectable without equating implementation with verification.
- **D-15:** Do not mark safety-critical or hardware-control surfaces `verified` in Phase 1 unless actual Ultra 205 hardware evidence exists. The safe boot/log smoke is verified by Ultra 205 hardware evidence; ASIC init, voltage, fan, thermal, power, and mining remain later-phase evidence.
- **D-16:** Preserve the MIT-first posture for original scaffolding and independently authored Rust code, but keep GPL provenance explicit for upstream-derived behavior and fixture sources.

### the agent's Discretion

The agent may choose the exact Bazel target implementation shape, host helper crate boundaries, placeholder/skeleton module contents, package manifest schema details, and formatting of parity reports when those choices remain consistent with the accepted docs, Bright Builds rules, and Phase 1 success criteria.

</decisions>

\<canonical_refs>

## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase Scope And Requirements

- `.planning/ROADMAP.md` - Phase 1 goal, requirements, success criteria, verification expectations, and Ultra 205 phase boundary.
- `.planning/REQUIREMENTS.md` - FND-01 through FND-11 and project-wide evidence/safety requirements.
- `.planning/PROJECT.md` - Core value, constraints, current focus, accepted scope, and key project decisions.
- `.planning/STATE.md` - Current phase state, Ultra 205 pivot decisions, and remaining safety-critical evidence constraints.
- `docs/project/first-milestone.md` - First milestone deliverables, acceptance criteria, out-of-scope items, and verification evidence.
- `docs/project/gsd-new-project-brief.md` - Accepted project brief and Ultra 205-first bring-up instructions.
- `docs/project/seed-layout.md` - Expected monorepo layout, path ownership, Bazel target families, Just command surface, and reference guard behavior.
- `docs/project/project-decisions.md` - Accepted ADR index and planning implications.

### Architecture, Stack, And Risk Research

- `.planning/research/SUMMARY.md` - Project research summary, recommended phase structure, and key gaps.
- `.planning/research/STACK.md` - ESP-IDF Rust stack, Bazel/Cargo boundary, flashing backend, and verification strategy.
- `.planning/research/ARCHITECTURE.md` - Functional-core/imperative-shell boundaries, component map, data flow, and test strategy.
- `.planning/research/FEATURES.md` - Device-user parity surfaces, first milestone features, deferred features, and anti-features.
- `.planning/research/PITFALLS.md` - Critical pitfalls, guardrails, warning signs, and Phase 1 risk mapping.

### ADRs And Policy

- `docs/adr/0001-device-user-parity.md` - Device-user parity definition.
- `docs/adr/0002-rust-firmware-monorepo.md` - New Rust monorepo decision.
- `docs/adr/0003-esp-idf-rust-production-stack.md` - ESP-IDF Rust first production stack.
- `docs/adr/0004-bazel-automation-with-just-wrapper.md` - Bazel and Just boundary.
- `docs/adr/0005-read-only-reference-implementation.md` - Read-only reference implementation policy.
- `docs/adr/0006-parity-checklist-as-audit-evidence.md` - Parity checklist as audit evidence.
- `docs/adr/0007-prioritize-gamma-601-bm1370-bring-up.md` - Superseded Gamma 601 BM1370 priority.
- `docs/adr/0014-pivot-to-ultra-205-bm1366-first-parity.md` - Ultra 205 BM1366 first parity target and deferred Gamma 601 scope.
- `docs/adr/0008-reference-breadcrumb-comments.md` - Reference breadcrumb placement policy.
- `docs/adr/0009-monorepo-package-layout.md` - Monorepo package layout.
- `docs/adr/0011-usb-flashing-ergonomics.md` - USB flashing command behavior.
- `docs/adr/0012-parity-verification-evidence.md` - Evidence requirements for parity status.
- `docs/adr/0013-mit-first-with-gpl-guardrails.md` - Licensing and GPL guardrails.
- `PROVENANCE.md` - Provenance, SPDX, upstream reference, and firmware release review policy.

### Parity Evidence

- `docs/parity/checklist.md` - Parity checklist rows for reference guard, automation, Ultra 205 boot/runtime, package, flash, and evidence reporting.
- `docs/parity/evidence/ultra-205-pivot-safe-state-smoke-2026-06-26.md` - Live Ultra 205 flash-monitor evidence for safe boot/log identity.

\</canonical_refs>

\<code_context>

## Existing Code Insights

### Reusable Assets

- Existing project docs under `.planning/`, `docs/project/`, `docs/adr/`, and `docs/parity/` provide the Phase 1 contract and should drive implementation.
- `PROVENANCE.md` already defines the licensing posture and should be referenced from any provenance or package manifest work.
- The Rust workspace, Bazel targets, Just command surface, package/flash tools, parity tooling, and firmware safe-state app now exist and should be treated as Phase 1 foundation for later phases.

### Established Patterns

- The repo now has a Rust workspace, Bazel workspace, firmware app, host tools, Justfile, parity docs, and the pinned `reference/esp-miner` submodule.
- Local standards require functional core plus imperative shell, Rust module shape `foo.rs` plus `foo/` for multi-file modules, `maybe_` names for `Option` values, Arrange/Act/Assert unit tests, and repo-native verification before commit.
- GSD and Bright Builds task artifacts should remain append-only and targeted; do not rewrite managed Bright Builds blocks.

### Integration Points

- Downstream phases should extend the existing Cargo/Bazel workspace, `firmware/bitaxe`, `crates/bitaxe-*`, `tools/flash`, `tools/parity`, and package manifest surfaces rather than replacing the Phase 1 command contract.
- Keep `scripts/verify-reference-clean.sh`, `just verify-reference`, `just parity`, and package/flash reference guard behavior intact.
- Preserve Ultra 205 as the only Phase 1 supported board for package/flash/monitor workflows; Gamma 601 paths remain deferred until their own evidence is planned.

\</code_context>

<specifics>
## Specific Ideas

- The minimal firmware should log a visibly safe state instead of pretending mining is ready.
- The first packaging manifest should be machine-readable and record firmware commit, reference commit, ESP-IDF version, Rust target, checksums, and the default Ultra 205 ELF.
- Port discovery and flashing errors should be actionable and include exact `port=` examples when ambiguous.
- The accepted Ultra 205 evidence proves only safe-state boot/log identity and does not unlock mining or hardware-control behavior.

</specifics>

<deferred>
## Deferred Ideas

- BM1366 ASIC initialization, work sending, result parsing, frequency transitions, voltage changes, fan, thermal, power, and mining behavior belong to later hardware phases.
- Stratum v1 mining, fake pool coverage, and accepted-share evidence belong to Phase 4.
- AxeOS HTTP/WebSocket API compatibility and static asset administration belong to Phase 5 and Phase 7.
- OTA, recovery, release packaging, dependency license inventory, and release compliance gates belong to Phase 7.
- Gamma 601/BM1370, non-205 boards, and additional ASIC families remain unverified or deferred until each has its own evidence.

</deferred>

______________________________________________________________________

*Phase: 01-foundation-and-gamma-601-boot-log*
*Context gathered: 2026-06-21*

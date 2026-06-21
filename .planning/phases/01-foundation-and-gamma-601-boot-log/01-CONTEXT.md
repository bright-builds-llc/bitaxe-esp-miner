---
generated_by: gsd-discuss-phase
lifecycle_mode: yolo
phase_lifecycle_id: 1-2026-06-21T00-30-20
generated_at: 2026-06-21T00:32:31.453Z
---

# Phase 1: Foundation And Gamma 601 Boot/Log - Context

**Gathered:** 2026-06-21
**Status:** Ready for planning
**Mode:** Yolo

<domain>
## Phase Boundary

Phase 1 establishes the repo foundation for the Rust ESP-IDF firmware rewrite: the pinned read-only ESP-Miner reference, Bazel/Cargo/Just automation skeleton, crate and firmware layout, package/flash/monitor workflows, parity/provenance tooling, and a minimal Gamma 601 boot/log firmware path. Mining, ASIC work submission, voltage/fan/thermal control, full AxeOS API behavior, OTA, and non-601 verification are outside this phase.

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

### Safe Gamma 601 Boot/Log

- **D-08:** The first firmware image should boot/log only. It must make the safe state explicit: mining disabled, ASIC work submission disabled, and hardware control disabled or safe-no-op.
- **D-09:** Boot logs should include firmware identity, firmware/source commit when available, ESP-IDF/Rust/toolchain identity when available, reset reason, partition/image identity, platform/PSRAM status, selected board target `Gamma 601`, selected ASIC target `BM1370`, and reference commit when known.
- **D-10:** If real Gamma 601 hardware is unavailable during local verification, the plan should still build/package and leave hardware smoke as a required evidence item rather than silently marking hardware behavior verified.

### Package, Flash, And Monitor Ergonomics

- **D-11:** Implement `just build`, `just test`, `just package`, `just flash`, `just monitor`, `just flash-monitor`, `just verify-reference`, and `just parity` in Phase 1 even if some targets initially wrap scripts or provide narrow skeleton behavior.
- **D-12:** `just flash board=601` should build/package first by default, accept `port=...`, discover likely ports when omitted, fail clearly on no or ambiguous ports, and print the underlying `espflash` command before executing.
- **D-13:** `just package` should produce a machine-readable package manifest with image paths, offsets when applicable, checksums, tool versions, firmware commit, and reference commit.

### Parity And Provenance Evidence

- **D-14:** Treat `docs/parity/checklist.md` as an evidence ledger. Phase 1 should wire `just parity` so checklist rows, statuses, missing evidence, implementation pointers, and breadcrumbs are inspectable without equating implementation with verification.
- **D-15:** Do not mark safety-critical or hardware-control surfaces `verified` in Phase 1 unless actual Gamma 601 hardware evidence exists. Boot/log smoke can be `hardware-smoke`; ASIC init, voltage, fan, thermal, power, and mining remain later-phase evidence.
- **D-16:** Preserve the MIT-first posture for original scaffolding and independently authored Rust code, but keep GPL provenance explicit for upstream-derived behavior and fixture sources.

### the agent's Discretion

The agent may choose the exact Bazel target implementation shape, host helper crate boundaries, placeholder/skeleton module contents, package manifest schema details, and formatting of parity reports when those choices remain consistent with the accepted docs, Bright Builds rules, and Phase 1 success criteria.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase Scope And Requirements

- `.planning/ROADMAP.md` - Phase 1 goal, requirements, success criteria, verification expectations, and phase boundary.
- `.planning/REQUIREMENTS.md` - FND-01 through FND-11 and project-wide evidence/safety requirements.
- `.planning/PROJECT.md` - Core value, constraints, current focus, accepted scope, and key project decisions.
- `.planning/STATE.md` - Current phase state and known blocker that `reference/esp-miner` is absent in the checkout.
- `docs/project/first-milestone.md` - First milestone deliverables, acceptance criteria, out-of-scope items, and verification evidence.
- `docs/project/gsd-new-project-brief.md` - Original accepted project brief and first milestone instructions.
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
- `docs/adr/0007-prioritize-gamma-601-bm1370-bring-up.md` - Gamma 601 BM1370 priority.
- `docs/adr/0008-reference-breadcrumb-comments.md` - Reference breadcrumb placement policy.
- `docs/adr/0009-monorepo-package-layout.md` - Monorepo package layout.
- `docs/adr/0011-usb-flashing-ergonomics.md` - USB flashing command behavior.
- `docs/adr/0012-parity-verification-evidence.md` - Evidence requirements for parity status.
- `docs/adr/0013-mit-first-with-gpl-guardrails.md` - Licensing and GPL guardrails.
- `PROVENANCE.md` - Provenance, SPDX, upstream reference, and firmware release review policy.

### Parity Evidence

- `docs/parity/checklist.md` - Seed parity checklist rows for reference guard, automation, boot/runtime, package, flash, and evidence reporting.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets

- Existing project docs under `.planning/`, `docs/project/`, `docs/adr/`, and `docs/parity/` provide the Phase 1 contract and should drive implementation.
- `PROVENANCE.md` already defines the licensing posture and should be referenced from any provenance or package manifest work.
- `scripts/bright-builds-auto-update.sh` is the only existing script and is managed Bright Builds infrastructure; do not reuse it for firmware workflow code.

### Established Patterns

- The repo currently has documentation and Bright Builds standards, but no Rust workspace, Bazel workspace, firmware app, host tools, or `reference/esp-miner` submodule.
- Local standards require functional core plus imperative shell, Rust module shape `foo.rs` plus `foo/` for multi-file modules, `maybe_` names for `Option` values, Arrange/Act/Assert unit tests, and repo-native verification before commit.
- GSD and Bright Builds task artifacts should remain append-only and targeted; do not rewrite managed Bright Builds blocks.

### Integration Points

- Add Bazel/Bzlmod root files, Cargo workspace files, Justfile, and Rust crate/firmware/tool directories at the repository root.
- Add `scripts/verify-reference-clean.sh` and make it reachable through both Bazel and `just verify-reference`.
- Add `tools/flash` and `tools/parity` as repo-owned host tooling represented in Bazel.
- Add package/flash output and parity evidence locations that downstream phases can extend without changing the Phase 1 public command surface.

</code_context>

<specifics>
## Specific Ideas

- The minimal firmware should log a visibly safe state instead of pretending mining is ready.
- The reference submodule absence is an immediate Phase 1 implementation item, not a follow-up.
- The first packaging manifest should be machine-readable even if some fields are `Unavailable` until the real firmware image path exists.
- Port discovery and flashing errors should be actionable and include exact `port=` examples when ambiguous.

</specifics>

<deferred>
## Deferred Ideas

- BM1370 ASIC initialization, work sending, result parsing, frequency transitions, voltage changes, fan, thermal, power, and mining behavior belong to later hardware phases.
- Stratum v1 mining, fake pool coverage, and accepted-share evidence belong to Phase 4.
- AxeOS HTTP/WebSocket API compatibility and static asset administration belong to Phase 5 and Phase 7.
- OTA, recovery, release packaging, dependency license inventory, and release compliance gates belong to Phase 7.
- Non-601 boards and additional ASIC families remain unverified or deferred until each has its own evidence.

</deferred>

---

*Phase: 01-foundation-and-gamma-601-boot-log*
*Context gathered: 2026-06-21*

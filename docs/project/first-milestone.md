# First Milestone: Foundation And Ultra 205 Bring-Up

The first milestone establishes the monorepo, build workflow, reference guardrails, and first hardware bring-up path. It does not attempt full mining parity. This document has been updated after the Ultra 205 pivot in ADR-0014; completed Gamma-first history remains in the GSD phase artifacts.

## Goal

A developer can build the Rust firmware skeleton, package a flashable image, flash a connected Bitaxe Ultra 205 over USB, and observe a boot log from the Rust firmware.

## Deliverables

- Bazel workspace initialized.
- Rust workspace initialized.
- ESP-IDF Rust firmware app skeleton under `firmware/bitaxe`.
- Read-only upstream submodule at `reference/esp-miner`.
- `scripts/verify-reference-clean.sh`.
- `Justfile` with build, test, package, flash, monitor, flash-monitor, verify-reference, and parity commands.
- Initial pure crate layout under `crates/`.
- Seed parity checklist under `docs/parity/checklist.md`.
- Provenance policy under `PROVENANCE.md`.
- Minimal Ultra 205 firmware boot path that logs firmware identity and basic platform status.

## Acceptance Criteria

- `just verify-reference` passes with a clean submodule.
- `just build` builds the firmware skeleton through Bazel.
- `just test` runs available pure crate/tool tests through Bazel.
- `just package` produces a flashable image artifact.
- `just flash board=205 port=<port>` flashes the connected Ultra 205 or fails with a clear actionable error.
- `just monitor port=<port>` shows Rust firmware boot logs.
- `just flash-monitor board=205 port=<port>` flashes and monitors in one workflow.
- The parity checklist includes seeded surfaces for upstream firmware behavior and records all first-milestone items as `not-started`, `in-progress`, `implemented`, or `verified`.
- No files inside `reference/esp-miner` are modified by normal workflow commands.

## Verification Evidence

Milestone completion should include:

- Command output summary for `just verify-reference`.
- Command output summary for `just build`.
- Command output summary for `just test`.
- Command output summary for `just package`.
- Hardware smoke note for Ultra 205 flash and boot log.
- Post-pivot safe-state evidence: `docs/parity/evidence/ultra-205-pivot-safe-state-smoke-2026-06-26.md`.
- Diff review confirming the reference submodule is clean except for the intentional pointer addition.

## Out Of Scope

- Mining work submission.
- BM1366 initialization parity beyond any stub or safe no-op needed for boot.
- Stratum networking.
- AxeOS API serving.
- OTA update behavior.
- Fan, voltage, power, and thermal control beyond safe platform initialization.
- Support claims for boards other than Ultra 205.
- Rewriting the Angular AxeOS UI.

## Risks To Manage

- ESP-IDF Rust and Bazel integration may need repo-owned scripts before direct rules are mature.
- Flashing tools may need platform-specific serial-port discovery.
- Firmware image licensing must be reviewed before any public release.
- Hardware-control code must default to safe behavior until parity is implemented and verified.

## Suggested Next Milestones

1. Config and board model parity.
1. BM1366 ASIC protocol and safe initialization.
1. Stratum v1 mining loop.
1. AxeOS API and telemetry compatibility.
1. Power, thermal, fan, and self-test parity.
1. OTA, filesystem, packaging, and release parity.
1. Additional board and ASIC parity expansion.

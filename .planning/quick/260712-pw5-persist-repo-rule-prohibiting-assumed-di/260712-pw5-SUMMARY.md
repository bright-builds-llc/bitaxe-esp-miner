---
status: completed
created: 2026-07-12
quick_id: 260712-pw5
title: Persist Direct-UART And Pin-Manipulation Prohibition
---

# Summary

Made non-invasive USB and barrel-power interaction the repository default and required fresh explicit user authorization before any direct UART attachment or physical electrical manipulation of board pins, pads, headers, GPIO, probes, jumpers, solder joints, or injected signals.

The already-verified Plan 14 UART implementation remains preserved as dormant software. Its direct fixture checkpoint, schema-v3 hardware qualification, and conditional formal Plan 13 chain were cancelled before hardware use.

## Changes

- Added the authoritative repo-local hardware authorization rule to `AGENTS.md` and subordinated the former TP18/TP12/J5 operational guidance.
- Appended the user-correction lesson without rewriting unrelated lesson history.
- Closed the pending Plan 14 fixture and qualification checkboxes without claiming they passed.
- Added a software-only/cancelled Plan 14 summary and routed STATE, ROADMAP, the debug record, and the existing todo block toward a separately planned non-invasive path.
- Preserved the original Plan 14 plan and all implementation/tests unchanged as dormant historical artifacts.

## Verification

- Repo-rule and cancellation text scans passed.
- Frontmatter and GSD lifecycle checks passed.
- `just verify-reference` passed.
- `cargo fmt --all` passed.
- `cargo clippy --all-targets --all-features -- -D warnings` passed.
- `cargo build --all-targets --all-features` passed.
- `cargo test --all-features` passed.
- `git diff --check` passed.

No detector, serial reader, reset, flash, credential, network, or hardware/device command was run.

## Residual Risk

Phase 28.1.1 remains open with STR-09, ASIC-11, CFG-07, and Phase 30 promotion pending. The next step is a new non-invasive gap plan; direct UART or pin work remains unavailable unless the exceptional permanent-blocker and fresh explicit-authorization gate is satisfied.

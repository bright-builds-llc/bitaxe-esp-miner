# Project Decisions

This index summarizes accepted decisions for the GSD New Project handoff. ADRs remain the source of rationale.

| ID | Decision | Outcome |
| --- | --- | --- |
| ADR-0001 | Define rewrite scope as device-user parity. | Accepted |
| ADR-0002 | Use this repo as a new Rust firmware monorepo. | Accepted |
| ADR-0003 | Use ESP-IDF Rust bindings for the first production stack. | Accepted |
| ADR-0004 | Use Bazel as the automation graph and `just` as command surface. | Accepted |
| ADR-0005 | Keep upstream ESP-Miner read-only at `reference/esp-miner`. | Accepted |
| ADR-0006 | Treat the parity checklist as audit evidence. | Accepted |
| ADR-0007 | Prioritize Bitaxe Gamma 601 BM1370 for hardware bring-up. | Accepted |
| ADR-0008 | Use reference breadcrumbs at module and behavior boundaries. | Accepted |
| ADR-0009 | Seed the monorepo with firmware, core crates, tools, docs, and reference. | Accepted |
| ADR-0010 | Target AxeOS API and asset compatibility before UI rewrite. | Accepted |
| ADR-0011 | Make USB flashing a first-class `just` workflow. | Accepted |
| ADR-0012 | Require explicit verification evidence for parity. | Accepted |
| ADR-0013 | Use MIT-first original code with GPL guardrails. | Accepted |
| PLAN-0001 | First milestone is project foundation plus Gamma 601 bring-up path. | Accepted |

## Planning Implications

- Full device-user parity is the overall project goal, not the first milestone.
- Gamma 601 BM1370 is the hardware path that should become easy first.
- Other upstream-supported boards stay in scope, but they are not verified until evidence exists.
- The upstream submodule is an input and audit target, not a workspace for project changes.
- GSD should create requirements and roadmap phases from the parity surfaces in `docs/parity/checklist.md`.
- Firmware release artifacts need provenance and license review before they are published.

## ADR Links

- [ADR-0001](../adr/0001-device-user-parity.md)
- [ADR-0002](../adr/0002-rust-firmware-monorepo.md)
- [ADR-0003](../adr/0003-esp-idf-rust-production-stack.md)
- [ADR-0004](../adr/0004-bazel-automation-with-just-wrapper.md)
- [ADR-0005](../adr/0005-read-only-reference-implementation.md)
- [ADR-0006](../adr/0006-parity-checklist-as-audit-evidence.md)
- [ADR-0007](../adr/0007-prioritize-gamma-601-bm1370-bring-up.md)
- [ADR-0008](../adr/0008-reference-breadcrumb-comments.md)
- [ADR-0009](../adr/0009-monorepo-package-layout.md)
- [ADR-0010](../adr/0010-axeos-api-and-asset-compatibility.md)
- [ADR-0011](../adr/0011-usb-flashing-ergonomics.md)
- [ADR-0012](../adr/0012-parity-verification-evidence.md)
- [ADR-0013](../adr/0013-mit-first-with-gpl-guardrails.md)

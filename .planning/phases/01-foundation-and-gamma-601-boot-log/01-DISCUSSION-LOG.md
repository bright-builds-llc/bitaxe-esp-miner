# Phase 1: Foundation And Gamma 601 Boot/Log - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md; this log preserves the alternatives considered.

**Date:** 2026-06-21T00:32:31.453Z
**Phase:** 1 - Foundation And Gamma 601 Boot/Log
**Mode:** Yolo
**Areas discussed:** Reference guardrails, Automation and workspace skeleton, Safe Gamma 601 boot/log, Package/flash/monitor ergonomics, Parity and provenance evidence

---

## Reference Guardrails

| Option | Description | Selected |
| --- | --- | --- |
| Pinned read-only submodule | Add `reference/esp-miner` as a pinned upstream submodule and fail on missing, dirty, or unpinned state. | yes |
| Documentation-only reference | Keep reference paths in docs but do not enforce a local submodule. | no |
| Editable upstream fork | Modify upstream ESP-Miner directly as part of the Rust project. | no |

**User's choice:** Auto-selected pinned read-only submodule.
**Notes:** This matches ADR-0005 and the Phase 1 blocker recorded in `.planning/STATE.md`.

---

## Automation And Workspace Skeleton

| Option | Description | Selected |
| --- | --- | --- |
| Bazel graph with Just wrapper | Use Bazel/Bzlmod as the canonical graph and make `just` delegate to Bazel-visible targets/scripts. | yes |
| Cargo-only workflow | Use Cargo and shell scripts directly, leaving Bazel for later. | no |
| Full Bazel-native ESP-IDF toolchain now | Attempt a complete Bazel-owned ESP-IDF/CMake/toolchain integration before first boot. | no |

**User's choice:** Auto-selected Bazel graph with Just wrapper.
**Notes:** This matches ADR-0004 and the stack research recommendation to wrap Cargo/ESP-IDF for firmware initially.

---

## Safe Gamma 601 Boot/Log

| Option | Description | Selected |
| --- | --- | --- |
| Boot/log only with safe disabled state | Build a minimal firmware path that logs identity/status and explicitly disables mining and hardware control. | yes |
| Stub mining as successful | Expose fake mining success before ASIC, Stratum, and safety paths exist. | no |
| Enable early hardware control | Start ASIC, voltage, fan, or thermal effects in Phase 1. | no |

**User's choice:** Auto-selected boot/log only with safe disabled state.
**Notes:** This preserves the first milestone boundary and avoids unsafe hardware bring-up.

---

## Package/Flash/Monitor Ergonomics

| Option | Description | Selected |
| --- | --- | --- |
| First-class Gamma 601 commands | Provide `just package`, `just flash board=601`, `just monitor`, and `just flash-monitor` with port discovery and printed backend commands. | yes |
| Manual espflash commands | Leave flashing to developer hand-run commands outside the repo graph. | no |
| Package later | Defer image/package manifest work until after firmware behavior exists. | no |

**User's choice:** Auto-selected first-class Gamma 601 commands.
**Notes:** This matches ADR-0011 and Phase 1 FND-07 through FND-09.

---

## Parity And Provenance Evidence

| Option | Description | Selected |
| --- | --- | --- |
| Evidence ledger from day one | Wire `just parity`, package provenance, reference breadcrumbs, and checklist status reporting in Phase 1. | yes |
| Implementation-only progress | Treat code completion as enough until release. | no |
| License review only at release | Ignore GPL provenance until firmware images are ready to publish. | no |

**User's choice:** Auto-selected evidence ledger from day one.
**Notes:** This matches ADR-0006, ADR-0012, ADR-0013, `docs/parity/checklist.md`, and `PROVENANCE.md`.

---

## the agent's Discretion

- Exact Bazel target implementation shape.
- Exact host tool crate boundaries.
- Exact package manifest schema, provided it records the required Phase 1 evidence fields.
- Exact parity report formatting.
- Placeholder/skeleton content for crates and firmware modules, provided safety state and boundaries remain explicit.

## Deferred Ideas

- Mining and Stratum behavior.
- ASIC initialization and hardware-control effects.
- Voltage, fan, thermal, power, and safety-controller parity.
- AxeOS API/WebSocket compatibility.
- OTA, filesystem, and release packaging beyond the first package manifest.
- Non-601 board verification.

---
phase: "34"
slug: "provenance-runtime-health-and-coherent-operator-snapshot"
status: verified
threats_open: 0
asvs_level: 1
created: "2026-07-17"
---

# Phase 34 — Security

> Per-phase security contract: threat register, accepted risks, and audit trail.

______________________________________________________________________

## Trust Boundaries

| Boundary | Description | Data Crossing |
| --- | --- | --- |
| Supervisor producer → runtime-health evaluator | The already-running supervisor supplies bounded previous/latest checkpoints to a pure evaluator. | Checkpoint category, sequence, and monotonic observation time; no mutable hardware handle. |
| Runtime-health evaluator → operator snapshot | One immutable health projection is attached to the same boot session and revision as the completed operator snapshot. | Passive self-test state, supervisor availability, checkpoint health, and independent watchdog participation. |
| Operator snapshot → public and retained projections | System-info, WebSocket, and retained records project one captured value without starting work or disclosing secrets. | Additive API fields and redacted correlation records. |
| Source guard → build verification | Checked-in source is scanned for effectful imports, calls, and forbidden active-control vocabulary. | Source text only; no device, credential, network, or hardware access. |

______________________________________________________________________

## Threat Register

| Threat ID | Category | Component | Disposition | Mitigation | Status |
| --- | --- | --- | --- | --- | --- |
| T-34-16 | Spoofing | Runtime-health age projection | mitigate | `RuntimeHealthSnapshot::evaluate` derives health from checked monotonic age on every capture; boundary and fixed-sequence aging tests prove healthy → stale → unhealthy transitions. | closed |
| T-34-17 | Spoofing | Task-watchdog projection | mitigate | Watchdog participation is modeled independently and always reports `unavailable` with reason `unproved` absent direct proof; a focused regression test prevents inference from supervisor health. | closed |
| T-34-18 | Elevation of privilege | Runtime-health adapter | mitigate | The adapter exposes a read-only collection path into a pure evaluator, while `phase34_runtime_health_is_passive_correlated_and_effect_free` rejects self-test starts, watchdog mutation, sleeps/load/fault injection, and hardware/mining effects. | closed |
| T-34-19 | Tampering | Checkpoint validation | mitigate | Category bounds, sequence/time regression checks, same-sequence mutation rejection, checked arithmetic, and explicit unavailable fallbacks prevent reordered, synthetic, missing, or overflowing observations from becoming healthy. | closed |

*Status: open · closed*
*Disposition: mitigate (implementation required) · accept (documented risk) · transfer (third-party)*

______________________________________________________________________

## Mitigation Evidence

- `crates/bitaxe-core/src/runtime_health.rs` — pure evaluation, checked age thresholds, explicit unavailable fallbacks, independent watchdog truth, and focused regression coverage.
- `firmware/bitaxe/src/runtime_health_adapter.rs` — read-only checkpoint copy into the pure evaluator with no effectful capability.
- `tools/parity/src/phase34_source_guard.rs` — correlation and prohibited-effect guards for the exact runtime-health production path.
- `firmware/bitaxe/src/runtime_snapshot.rs` — one captured runtime-health value attached to the completed operator snapshot and reused by public/retained projections.
- Verification: `bazel test //firmware/bitaxe:runtime_health_tests //firmware/bitaxe:runtime_health_no_effects_test` passed on 2026-07-17 (4/4 analyzed test targets passed).

______________________________________________________________________

## Accepted Risks Log

No accepted risks.

______________________________________________________________________

## Security Audit Trail

| Audit Date | Threats Total | Closed | Open | Run By |
| --- | ---: | ---: | ---: | --- |
| 2026-07-17 | 4 | 4 | 0 | Codex / gsd-secure-phase |

______________________________________________________________________

## Sign-Off

- [x] All threats have a disposition (mitigate / accept / transfer)
- [x] Accepted risks documented in Accepted Risks Log
- [x] `threats_open: 0` confirmed
- [x] `status: verified` set in frontmatter

**Approval:** verified 2026-07-17

# Project Retrospective

*A living document updated after each milestone. Lessons feed forward into future planning.*

## Milestone: v1.0 - Ultra 205 Parity

**Shipped:** 2026-07-04
**Phases:** 21 | **Plans:** 116 | **Tasks:** 226

### What Was Built

- Rust ESP-IDF firmware monorepo with Bazel automation, `just` workflow entrypoints, pinned ESP tooling, and a protected ESP-Miner reference submodule.
- Pure Rust crates and firmware adapters for Ultra 205 config/NVS, BM1366 ASIC behavior, Stratum v1, AxeOS API/static/WebSocket surfaces, safety decisions, release packaging, OTA, recovery, parity, and hardware evidence.
- Detector-gated Ultra 205 evidence ledgers for safe boot, API/static/WebSocket, firmware OTA boundaries, recovery/OTAWWW boundaries, active safety telemetry boundaries, BM1366 diagnostics, and controlled no-share mining/soak closure.
- Milestone audit closure with 64/64 requirements satisfied, 21/21 phase verifications passed, and no requirement, integration, or flow gaps.

### What Worked

- Exact-claim evidence boundaries kept shipped claims honest while still letting implementation and governance work close.
- Pure-core and firmware-adapter separation made protocol, config, API, safety, and route logic testable before hardware evidence existed.
- Repo-owned wrappers for package, flash-monitor, parity, redaction, and hardware evidence gave repeatable command surfaces and audit trails.
- Redaction-first evidence handling prevented local Wi-Fi, pool, target, endpoint, and credential details from leaking into committed artifacts.

### What Was Inefficient

- Several early phase summaries and validation files did not carry enough structured lifecycle metadata, which caused later audit cleanup.
- Some generated milestone/accomplishment extraction was too noisy to use directly and needed manual compression.
- Evidence closure required multiple later gap-closure phases because release-sensitive rows were initially too broad for available live hardware proof.

### Patterns Established

- Treat `docs/parity/checklist.md` as the exact-claim source of truth, not as a task-completion mirror.
- Require detector-gated, board-specific evidence before hardware-sensitive rows can move to `verified`.
- Keep live-device target discovery explicit and same-session; do not infer `DEVICE_URL` from stale logs, scans, or unrelated artifacts.
- Archive milestone roadmap and requirements after completion so active planning stays small.

### Key Lessons

1. Plan evidence criteria as narrowly as implementation criteria for hardware and release surfaces.
2. Keep generated summaries concise enough for future automation, or archive tools will amplify noisy historical details.
3. Treat no-share, blocked, pending, and below-verified evidence as valid engineering outputs when they preserve safety and truthfulness.
4. Run Nyquist validation earlier in the phase lifecycle if clean process metadata matters for milestone completion.

### Cost Observations

- Sessions: multiple long-running GSD execution and audit sessions.
- Notable: phase-gated hardware evidence was slower than pure implementation, but it prevented unsafe or unsupported parity claims.

## Milestone: v1.1 - Ultra 205 Trusted Production Mining

**Shipped:** 2026-07-13
**Phases:** 18 | **Plans:** 76 | **Tasks:** 170
**Outcome:** 18/21 requirements satisfied; shipped with accepted unresolved gaps

### What Was Built

- An exact claim ladder and fail-closed safety prerequisite model with blocker propagation into runtime and API output.
- BM1366 production-work and real Stratum v1 runtime paths with bounded safe stop, watchdog checkpoints, generation-aware correlation, and telemetry projection.
- Redacted evidence profiles, deterministic completion/validation, single-finalizer workflows, atomic evidence consolidation, and Phase 30 admission rules.
- Hardware-backed diagnostic evidence showing the Rust firmware remained unable to produce correlatable nonces while upstream firmware mined on the same Ultra 205.
- Terminal archive and execution guards for the unresolved Phase 28.1.1 diagnostic lineage.

### What Worked

- Exact non-claims prevented software completion, synthetic tests, and upstream success from being mistaken for Rust live-share parity.
- Comparing Rust and upstream firmware on the same board sharply isolated the blocker to the Rust firmware path.
- Typed evidence contracts and atomic finalization made partial, stale, mixed, or sensitive artifacts fail closed.
- Retaining only evidence-supported wire corrections preserved useful progress without claiming that any one diagnostic lever solved nonce production.

### What Was Inefficient

- The Phase 28.1.1 lineage expanded through seven inserted descendants and several recovery plans without a decisive live-result signal.
- Hardware transport and lifecycle uncertainty consumed substantial effort after the main firmware blocker had already been isolated.
- Lifecycle metadata and Nyquist validation were not consistently completed when phases closed, increasing audit work.
- The milestone archive tool counted only then-active directories and required manual correction for phases archived early.

### Patterns Established

- Terminal unresolved work is a valid milestone outcome when verification remains truthful and future routing is explicit.
- Archive lookup failures are tooling defects, not permission to recreate active phase stubs or promote verification.
- A new hardware diagnostic must define a bounded evidence-producing lever and stopping rule before execution.
- Requirement definition checkboxes and verified traceability are separate concepts; the outcome ledger is authoritative.

### Key Lessons

1. Stop a diagnostic lineage once repeated A/B levers fail to produce a new discriminating signal.
2. Treat no-promotion as a first-class, testable terminal decision rather than a temporary workflow state.
3. Make archive-aware routing part of GSD health checks before using early phase archival inside an active milestone.
4. Complete validation and lifecycle metadata in the same phase-closing commit whenever possible.

### Cost Observations

- Duration: 9 days and 432 commits from milestone start through Phase 30 completion.
- Scale: 975 files changed, 122,740 insertions, 1,454 deletions, and 56,967 Rust source lines at closure.
- The hardware effort produced a strong blocker isolation result, but repeated nonce-production diagnostics had diminishing returns and justified the Won't Do closure.

## Cross-Milestone Trends

### Process Evolution

| Milestone | Phases | Key Change |
| --- | ---: | --- |
| v1.0 | 21 | Established evidence-backed GSD delivery for Rust ESP-IDF firmware and Ultra 205 hardware parity. |
| v1.1 | 18 | Added fail-closed production-mining evidence governance and a truthful terminal-unresolved archive path. |

### Cumulative Quality

| Milestone | Requirement Coverage | Phase Verification | Audit Result |
| --- | ---: | ---: | --- |
| v1.0 | 64/64 | 21/21 passed | No gaps; accepted tech debt. |
| v1.1 | 18/21 | 10 passed; 8 terminal `gaps_found` | `gaps_found`; three accepted unresolved requirements. |

### Top Lessons

1. Verification evidence must name what it proves and what it does not prove.
2. Hardware safety gates and redaction gates are planning requirements, not after-the-fact cleanup.
3. Terminal unresolved closure is healthier than autonomous diagnostic churn when evidence no longer discriminates between hypotheses.
4. Archive truth must take precedence over tool-health heuristics that expect every milestone phase to remain active until completion.

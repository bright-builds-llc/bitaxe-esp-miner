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

## Cross-Milestone Trends

### Process Evolution

| Milestone | Phases | Key Change |
| --- | ---: | --- |
| v1.0 | 21 | Established evidence-backed GSD delivery for Rust ESP-IDF firmware and Ultra 205 hardware parity. |

### Cumulative Quality

| Milestone | Requirement Coverage | Phase Verification | Audit Result |
| --- | ---: | ---: | --- |
| v1.0 | 64/64 | 21/21 passed | No gaps; accepted tech debt. |

### Top Lessons

1. Verification evidence must name what it proves and what it does not prove.
2. Hardware safety gates and redaction gates are planning requirements, not after-the-fact cleanup.

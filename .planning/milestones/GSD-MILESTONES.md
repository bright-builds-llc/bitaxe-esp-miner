# Milestones

## v1.1 Ultra 205 Trusted Production Mining (Shipped: 2026-07-13)

**Delivered:** A safety-gated production-mining software and evidence path for Ultra 205, hardware-backed isolation of the remaining Rust nonce-production blocker, and a terminal no-promotion closure that preserves unresolved requirements without overstating parity.

**Phases completed:** 18 phases, 76 plans, 170 tasks.

**Key accomplishments:**

- Established an exact claim ladder, safety prerequisite gates, and typed blocker propagation through runtime and API surfaces.
- Built the BM1366/Stratum production software chain with bounded safe-stop, telemetry projection, redacted evidence profiles, and atomic evidence consolidation.
- Proved on hardware that the Rust firmware still did not produce correlatable nonces while upstream firmware mined successfully on the same Ultra 205, narrowing the blocker without unsafe promotion.
- Retained evidence-supported wire-parity corrections while every Phase 28.1.1 lineage verification remained `gaps_found`.
- Closed Phase 30 with deterministic no-promotion enforcement and archived the diagnostic lineage as terminal unresolved work protected from autonomous reopening.

**Statistics:**

- Files changed: 975
- Lines changed: +122,740 / -1,454
- Rust source: 56,967 lines
- Commits: 432 over 9 days
- Git range: `7ea3321` → `ec8ab33`

**Accepted unresolved requirements:**

- STR-09 — no eligible live ASIC-derived `mining.submit` response outcome.
- ASIC-11 — no live BM1366 result correlated to active pool work.
- CFG-07 — no eligible end-to-end live credential-handling evidence root.

**Known carried debt:**

- Phase 28.1 Nyquist validation is partial; Phase 28.1.1.1 validation is missing.
- Some Phase 23 and Phase 27 lifecycle metadata remains historical debt.
- Installed GSD progress/archive lookup has a known project-only exception; archived phase directories must not be recreated to silence it.

**Archive:**

- [Roadmap](./milestones/v1.1-ROADMAP.md)
- [Requirements](./milestones/v1.1-REQUIREMENTS.md)
- [Milestone audit](./milestones/v1.1-MILESTONE-AUDIT.md)
- [Research archive](./milestones/v1.1-research/)
- [Phase archive](./milestones/v1.1-phases/)

**Next:** Start a new milestone with fresh requirements via `/gsd-new-milestone`.

______________________________________________________________________

## v1.0 Ultra 205 Parity (Shipped: 2026-07-04)

**Delivered:** A Rust ESP-IDF Bitaxe firmware foundation with Ultra 205 build, flash, configuration, API, safety, release, mining, and parity evidence surfaces completed under exact-claim evidence governance.

**Phases completed:** 1-21, 116 plans, 226 tasks.

**Key accomplishments:**

- Established the Rust firmware monorepo with Bazel, `just`, pinned ESP-IDF Rust tooling, and a protected read-only ESP-Miner reference submodule.
- Built testable Rust core crates for config/NVS, BM1366 ASIC protocol, Stratum v1 mining, AxeOS API models, safety controllers, release tooling, and parity reporting.
- Delivered Ultra 205 firmware packaging, USB flash/monitor workflows, safe boot evidence, static/recovery/OTA surfaces, and release manifest/license/provenance gates.
- Captured detector-gated hardware evidence for Ultra 205 safety, BM1366 diagnostic behavior, live HTTP/static/API/WebSocket surfaces, firmware OTA boundaries, and recovery/OTAWWW non-claims.
- Closed live mining and soak evidence with approved controlled no-share smoke/soak, watchdog responsiveness, telemetry correlation, redaction review, and conservative non-claims for accepted/rejected shares and active hardware-control surfaces.
- Completed the v1.0 milestone audit with 64/64 requirements satisfied, 21/21 phases passed, no requirement/integration/flow gaps, and accepted process/product tech debt.

**Known carried debt:**

- Nyquist validation remains partial for older phases 01, 02, 03, 04, 07, 08, 09, 10, 17, and 18.
- Several parity checklist rows intentionally remain below `verified` as exact non-claims for deferred boards, accepted/rejected shares, active hardware controls, OTAWWW, rollback/recovery edge cases, and unbounded production mining behavior.

**Archive:**

- [Roadmap](./milestones/v1.0-ROADMAP.md)
- [Requirements](./milestones/v1.0-REQUIREMENTS.md)
- [Milestone audit](./milestones/v1.0-MILESTONE-AUDIT.md)
- [Phase archive](./milestones/v1.0-phases/)

**Next:** Start the next milestone with fresh requirements via `/gsd-new-milestone`.

# Milestones

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

- `.planning/milestones/v1.0-ROADMAP.md`
- `.planning/milestones/v1.0-REQUIREMENTS.md`
- `.planning/milestones/v1.0-MILESTONE-AUDIT.md`

**Next:** Start the next milestone with fresh requirements via `/gsd-new-milestone`.

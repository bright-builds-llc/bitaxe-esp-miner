# Parity work log

## 2026-08-16T08:39:00Z | root-cause correction

- Source commit: `f4d618c26186af690678171cbdb953fac7648316`
- Actions: Traced INA260 register scaling through the Rust safety domain, legacy API and statistics projections, and the pinned reference driver, power task, JSON route, statistics task, and AxeOS normalization.
- Verification: The immutable plan checkpoint passed the ordered Rust sequence, Bright Builds, all 45 Bazel tests, parity, progress, reference cleanliness, redaction, and diff checks before implementation.
- Evidence: `docs/parity/work-plans/20260816T082924Z-PWR-006/PLAN.md` with SHA-256 `9cac99ee0fe28580b1c729a9d9681721e07b1ac55b22624fb8073ffe786849f6`.
- Outcome: Root cause confirmed: the sensor and safety core correctly use volts and amps, but the legacy `voltage` and `current` wire fields require millivolts and milliamps. `coreVoltageActual` already correctly uses millivolts.
- Blocker or next safe action: Apply explicit conversions at both compatibility boundaries, update safety consumers and tests, then re-evaluate the closed PWR-006 evidence lineage.

## 2026-08-16T09:07:00Z | implementation verified

- Source commit: pending implementation commit
- Actions: Added explicit volts-to-millivolts and amps-to-milliamps compatibility conversions, renamed ambiguous Rust DTO members, converted statistics samples, preserved the physical 4.5–5.5 V campaign gate as 4500–5500 mV, and upgraded the PWR-006 source-evidence contract to bind historical Rust behavior, corrected Rust behavior, and the pinned reference/UI conversions.
- Verification: `cargo fmt --all`; `cargo clippy --all-targets --all-features -- -D warnings`; `cargo build --all-targets --all-features`; `cargo test --all-features`; `bun scripts/bright-builds-check.ts all`; `just test` (45/45); and `just package` all passed after adding the new focused module to both Cargo and Bazel source graphs.
- Evidence: Behavior regressions prove 5.1 V / 2.25 A remain SI internally while `/api/system/info` and statistics serialize 5100 mV / 2250 mA; campaign boundary regressions accept 4500 and 5500 mV and reject 4499 and 5501 mV.
- Outcome: Implementation and build/package verification passed. The evidence projector is ready to run against a clean committed source tree.
- Blocker or next safe action: Commit and push the implementation, generate and independently validate the v2 source-bound projection, then close the task without changing the already-verified checklist row.

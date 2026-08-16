# Parity work log

## 2026-08-16T08:39:00Z | root-cause correction

- Source commit: `f4d618c26186af690678171cbdb953fac7648316`
- Actions: Traced INA260 register scaling through the Rust safety domain, legacy API and statistics projections, and the pinned reference driver, power task, JSON route, statistics task, and AxeOS normalization.
- Verification: The immutable plan checkpoint passed the ordered Rust sequence, Bright Builds, all 45 Bazel tests, parity, progress, reference cleanliness, redaction, and diff checks before implementation.
- Evidence: `docs/parity/work-plans/20260816T082924Z-PWR-006/PLAN.md` with SHA-256 `9cac99ee0fe28580b1c729a9d9681721e07b1ac55b22624fb8073ffe786849f6`.
- Outcome: Root cause confirmed: the sensor and safety core correctly use volts and amps, but the legacy `voltage` and `current` wire fields require millivolts and milliamps. `coreVoltageActual` already correctly uses millivolts.
- Blocker or next safe action: Apply explicit conversions at both compatibility boundaries, update safety consumers and tests, then re-evaluate the closed PWR-006 evidence lineage.

## 2026-08-16T09:07:00Z | implementation verified

- Source commit: `683938b4fa17111eda3d9a4a1af1ea7cbcc9a362`
- Actions: Added explicit volts-to-millivolts and amps-to-milliamps compatibility conversions, renamed ambiguous Rust DTO members, converted statistics samples, preserved the physical 4.5–5.5 V campaign gate as 4500–5500 mV, and upgraded the PWR-006 source-evidence contract to bind historical Rust behavior, corrected Rust behavior, and the pinned reference/UI conversions.
- Verification: `cargo fmt --all`; `cargo clippy --all-targets --all-features -- -D warnings`; `cargo build --all-targets --all-features`; `cargo test --all-features`; `bun scripts/bright-builds-check.ts all`; `just test` (45/45); and `just package` all passed after adding the new focused module to both Cargo and Bazel source graphs.
- Evidence: Behavior regressions prove 5.1 V / 2.25 A remain SI internally while `/api/system/info` and statistics serialize 5100 mV / 2250 mA; campaign boundary regressions accept 4500 and 5500 mV and reject 4499 and 5501 mV.
- Outcome: Implementation and build/package verification passed. The evidence projector is ready to run against a clean committed source tree.
- Blocker or next safe action: Commit and push the implementation, generate and independently validate the v2 source-bound projection, then close the task without changing the already-verified checklist row.

## 2026-08-16T09:23:00Z | correction evidence accepted

- Source commit: `7a822b5c229d9f169fe22fe999202976980bed78`
- Actions: Corrected one non-unique source-admission breadcrumb, committed and pushed the clean projector source, then projected the accepted historical read-only capture through the current unit semantics without a hardware rerun.
- Verification: The projector and independent Rust validator accepted schema `bitaxe-ina260-evidence-v2`; mode is `0644`; SHA-256 is `ddf94e029b55089bb0e9a86cac6a0ca0d737ef67509d850233e64b532260b7fb`; repository redaction accepted all 21 committed evidence artifacts.
- Evidence: `docs/parity/evidence/pwr006-ina260/ina260-wire-units-projection.json` binds both immutable plans, exact historical and current source commits, the pinned reference commit, eleven current source paths, and six reference paths while publishing no raw telemetry.
- Outcome: Every correction, evidence, privacy, build, test, package, parity, and reference gate supports retaining `PWR-006` as `verified`. No checklist field changes because the row was already verified and automatic transitions out of verified are forbidden.
- Blocker or next safe action: Record the result, archive the completed task, run final gates, and push the closure commit.

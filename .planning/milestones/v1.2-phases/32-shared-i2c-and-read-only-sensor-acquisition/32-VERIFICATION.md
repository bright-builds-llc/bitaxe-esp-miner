---
phase: 32-shared-i2c-and-read-only-sensor-acquisition
status: passed
score: "25/25 software-scope must-haves verified"
verified: 2026-07-14
generated_by: gsd-verifier
generated_at: 2026-07-14T00:55:19Z
lifecycle_mode: yolo
phase_lifecycle_id: 32-2026-07-13T23-12-34
lifecycle_validated: true
verification_result: passed
requirements:
  - OBS-02
  - OBS-03
  - OBS-04
  - OBS-05
review_status: clean
hardware_evidence: pending
promotion_owner: phase-35
---

# Phase 32: Shared I2C and Read-Only Sensor Acquisition Verification

## Verdict

Phase 32 passed at its explicitly approved software-only boundary. Normal firmware now constructs one bounded I2C0 owner, borrows it for the existing startup display path, consumes it into a distinct read-only owner, and moves that owner into one deadline-based sensor producer. The producer admits one atomic INA260 power acquisition, keeps EMC2101 temperature and tachometer independent, owns all source stamps and stale transitions, and publishes a complete snapshot that API consumers only clone.

Review fixes `004a3ac` close the original capability and aging gaps by making the post-display owner incapable of expressing arbitrary writes, aging retained last-good values through sustained faults, and covering each failed sensor source at the API boundary. The final review `95164c0` is clean with zero findings.

This verdict does not claim physical display continuity, physical sensor accuracy, live failure behavior, or hardware-backed parity. Hardware access was prohibited for this verification, remains pending, and can be admitted or promoted only by Phase 35 through its detector-gated correlated evidence contract.

## Goal Achievement

| Success criterion | Result | Software evidence |
| --- | --- | --- |
| Display startup, INA260, and EMC2101 share one serialized, long-lived I2C0 owner while preserving the startup display path. | VERIFIED | `main.rs` constructs one `BitaxeI2cBus`, calls `render_startup_debug_text(&mut bus, ...)`, and then consumes the same owner with `bus.into_read_only()`. Display failure is logged but still reaches producer startup. `StartupDisplayBus` forwards every embedded-hal operation with the named finite timeout. |
| INA260 current, bus voltage, and power publish only after a successful producer-owned read with a new sequence and monotonic acquisition time. | VERIFIED | The adapter attempts all three allowlisted INA260 registers before returning one typed outcome. The pure reducer advances one shared power sequence only on complete success, and the producer projects the same stamped power truth into watts, volts, and amps. Partial/read/validation failures retain the exact prior stamp and sequence. |
| EMC2101 temperature and independently available tachometer facts use read-only transactions and explicitly represent missing or invalid data. | VERIFIED | Closed register enums expose only external-temperature and tachometer reads. Their adapter calls and reducer branches are independent; signed temperature, open/short sentinels, zero/no-spin tachometer, overflow, read failures, and invalid samples have deterministic tests and non-fresh outcomes. |
| One sensor failure does not block the API or unaffected observations, and the affected value becomes faulted or stale without request-driven refresh. | VERIFIED | The producer attempts power, temperature, and tachometer before one complete store replacement. Sustained-failure tests prove fault-at-threshold then stale-after-threshold with exact retained stamps. Table-driven consumer tests cover failed power, temperature, and tachometer while repeated store/API reads remain byte-identical. |
| Phase 32 adds no actuator/control, credential, hardware, archived-lineage, or promotion effect. | VERIFIED | The producer accepts only `ReadOnlySensorOwner`; its capability exposes only typed allowlisted reads. Source guards reject active/control identifiers. The phase diff leaves `reference/esp-miner` and `docs/parity/checklist.md` byte-unchanged, and the evidence record preserves every prohibited category as a non-claim. |

**Goal score:** 5/5 software success criteria verified.

## Requirements Coverage

| Requirement | Status | Verification |
| --- | --- | --- |
| OBS-02 | SATISFIED at software scope | One normal-path driver, finite 50 ms tick-converted transactions, borrowed startup display, consuming read-only handoff, closed periodic read capability, and no Phase 32 write reachability are implemented, source-guarded, and firmware-built. Physical startup-display continuity remains hardware evidence pending. |
| OBS-03 | SATISFIED at software scope | Signed INA260 current and complete triple decoding feed one producer-owned power sequence and monotonic stamp. The three public power facts share that truth; failed attempts mint no freshness. Physical INA260 values remain hardware evidence pending. |
| OBS-04 | SATISFIED at software scope | EMC2101 temperature and tachometer have separate read-only attempts, sequences, stamps, validation, failure states, and projections. Physical temperature/tachometer behavior remains hardware evidence pending. |
| OBS-05 | SATISFIED at software scope | One-shot sweep continuation, complete snapshot replacement, fault/stale aging, clone-only request reads, and all three failed-source cases pass deterministic tests. Live API availability during a naturally occurring physical fault remains hardware evidence pending. |

The checked requirement rows and traceability table in `.planning/REQUIREMENTS.md`, Phase 32 roadmap binding, and all three plan frontmatters consistently map exactly OBS-02 through OBS-05 to this phase.

## Must-Haves Audit

All 25 plan must-haves are verified at the accepted software boundary: 11 truths, eight required artifacts, and six key links.

### Plan 32-01 — Pure Acquisition Core: 8/8

- `sensor_acquisition.rs` owns typed INA260/EMC2101 decoding, source outcomes, producer sequences, sweep reduction, and producer-time stale evaluation without ESP-IDF or an internal clock.
- One validated INA260 triple advances one power sequence. Power failure retains the complete prior stamped observation and advances nothing.
- Temperature and tachometer successes or failures remain independent and never borrow each other's stamp or sequence.
- Read and validation faults retain exact last-good provenance; sustained faults age to stale only after producer-supplied elapsed time crosses the named threshold.
- The reducer remains compatible with `TelemetryObservations` through the firmware projection and does not perform hardware or request-side work.

### Plan 32-02 — Bounded Capability Boundary: 8/8

- `BitaxeI2cBus` owns the one normal driver and converts `I2C_TRANSACTION_TIMEOUT_MS` with `TickType::new_millis(...).ticks()` for every exposed display/sensor operation.
- The startup display borrows the bus through a bounded embedded-hal adapter. Success or error returns control to `main`, which retains ownership for the next handoff.
- `into_read_only(self)` consumes the display-capable owner into `ReadOnlySensorOwner`; the producer cannot construct `StartupDisplayBus`, `ActiveI2cBus`, or a generic write surface.
- INA260 and EMC2101 register enums close the periodic surface to exactly seven allowlisted read registers, all using finite repeated-start `write_read` transactions.
- Legacy active functions remain preserved behind the Phase 27 token and are not newly reachable from normal Phase 32 startup.

### Plan 32-03 — Sole Producer and Consumer Purity: 9/9

- `operator_sensor_runtime.rs` owns one named long-lived thread, boot session, three source-local sequences, one-shot source attempts, and a deadline-based 500 ms cadence that skips missed slots without catch-up bursts.
- Power, temperature, and tachometer attempts all occur before the store replacement; no I2C operation happens while the observation-store mutex is held.
- One stamped power observation projects to watts, volts, and amps; external temperature projects to chip temperature; tachometer projects to fan RPM; VR temperature stays explicitly unavailable.
- `observation_store.rs` provides a producer-named complete replacement and clone-only reads. `runtime_snapshot.rs` calls only `observation_snapshot()` for these facts.
- Repeated API reads neither acquire hardware nor change state, reason, boot session, sequence, or acquisition time.
- `docs/evidence/phase-32/README.md` records software evidence, the exact private-trace/sanitized-summary blocker, hardware pending, Phase 35-only promotion, and all active/sensitive non-claims.

## Review And Security Result

The final standard-depth review in `32-REVIEW.md` is clean: 0 critical, 0 warning, and 0 info findings.

The resolved findings materially strengthen the phase:

- The producer now owns a distinct type-state that cannot express display or arbitrary I2C writes.
- Sustained failures age retained last-good power, temperature, and tachometer observations from fault to stale without changing their stamps or sequences.
- Consumer regression coverage now includes failed power, temperature, and tachometer sources, including atomic power projection and unaffected fresh facts.

The high-severity Phase 32 threats are mitigated by checked decoding and atomic admission (T-32-01), finite typed read-only capabilities (T-32-02), sole-producer provenance with clone-only consumers (T-32-03), and the explicit no-hardware/no-promotion evidence boundary (T-32-04).

## Independent Command Evidence

| Command or check | Result |
| --- | --- |
| `cargo fmt --all` | PASS; no Rust-format change remained. |
| `cargo clippy --all-targets --all-features -- -D warnings` | PASS. |
| `cargo build --all-targets --all-features` | PASS. |
| `cargo test --all-features` | PASS; complete host workspace and doc-test surface passed. |
| `cargo test -p bitaxe-safety --all-features sensor_acquisition` | PASS; 10 focused tests. |
| `cargo test -p bitaxe-api --all-features phase32_consumer` | PASS; 2 focused tests. |
| `cargo test -p bitaxe-parity --all-features phase32_` | PASS; 8 focused source-boundary tests. |
| `just build` | PASS; canonical ESP32-S3 firmware built at source commit `95164c02daf1` against pinned ESP-IDF `v5.5.4`. Eight firmware-only dead-code warnings are pre-existing and outside the host Clippy warnings-denied surface. |
| `just test` | PASS; all 53 Bazel test targets passed with deterministic fixtures only. No hardware command was invoked. |
| `just package` | PASS; firmware image and Ultra 205 package were produced. App partition usage was 1,604,368/4,194,304 bytes (38.25%). |
| `just parity` | PASS; `validation_errors: none`. No checklist row was promoted by Phase 32. |
| `just verify-reference` | PASS; reference clean at `c1915b0a63bfabebdb95a515cedfee05146c1d50`. |
| Reference and promotion diff | PASS; `git diff --exit-code 1723567..HEAD -- reference/esp-miner docs/parity/checklist.md` was clean. |
| Diff hygiene | PASS; phase-range and current `git diff --check` returned clean. |
| `gsd-tools verify schema-drift 32 --raw` | PASS; `drift_detected: false`, `blocking: false`. |
| `gsd-tools verify lifecycle 32 --expect-id 32-2026-07-13T23-12-34 --expect-mode yolo --require-plans --raw` | PASS; `valid`. |

The mandatory Rust gate ran separately and in the required order: format, Clippy, build, then test.

## Scope And Safety Audit

Verification used committed source, phase artifacts, pure host tests, Bazel tests, firmware compilation, package construction, parity validation, diff inspection, and the reference-clean guard only. It did not detect or access a board, USB/serial device, credential file, network target, ignored evidence root, or hardware state.

No detector, flash, monitor, device API, reset, raw write, fault injection, self-test, stress, mining, pool, credential, direct UART/pin, OTA/recovery, other-board, or archived Phase 28.1.1 operation ran. No reference file, parity checklist row, or hardware-evidence claim changed.

## Exact Non-Claims

- Phase 32 does not prove that the startup SSD1306 rendered successfully on a physical Ultra 205 or that the display remains physically observable after this firmware change.
- Phase 32 does not prove physical INA260 accuracy, EMC2101 temperature/tachometer accuracy, electrical bus timing, or real sensor availability.
- Phase 32 does not prove live API/WebSocket availability during a naturally occurring hardware sensor fault.
- Phase 32 does not admit detector-gated hardware evidence and does not promote any parity checklist row; Phase 35 owns correlated evidence and exact promotion.
- Phase 32 does not perform or verify fan/configuration writes, voltage, reset, power sequencing, ASIC control, fault stimulus, self-test execution, mining, credentials, direct UART/pins, OTA/recovery, non-205 boards, or archived diagnostic work.
- Phase 32 does not provide Phase 33 hostname durability or Phase 34 coherent operator-snapshot revision, provenance, and passive-health composition.

## Residual Risks

- Physical startup-display continuity and physical sensor behavior remain unverified until an eligible Phase 35 hardware chain exists.
- The committed hardware-pending record identifies a wrapper gap: current tooling does not yet produce both the required private serial-session ownership/cleanup trace and a separately sanitized shareable summary.
- Firmware compilation still reports eight pre-existing dead-code warnings outside the host Clippy surface; none was introduced as a Phase 32 review finding.
- `.planning/PROJECT.md` and the roadmap's derived progress rollup still describe Phase 32 as future/incomplete even though authoritative Phase 32 rows, requirements, plans, summaries, and this verification are complete. This is a non-blocking phase-completion metadata follow-up, not a behavior or traceability gap.

## Final Status

`verification_result: passed`. Phase 32 achieves OBS-02, OBS-03, OBS-04, and OBS-05 at deterministic core-test, source-capability, firmware-build, package, and workflow-validation depth. Hardware observations and parity promotion remain explicitly pending for Phase 35.

***

Material guidance applied: the Phase 32 software-only and no-hardware boundary, ESP-IDF tooling preference, archived-lineage prohibition, credential/evidence restrictions, direct-UART/pin prohibition, and frontmatter rules in `AGENTS.md`; the Bright Builds workflow in `AGENTS.bright-builds.md`; no active exception in `standards-overrides.md`; and the architecture, code-shape, testing, verification, and Rust standards under `standards/`.

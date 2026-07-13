# Phase 11: Safety Controller Hardware Regression Evidence - Research

**Researched:** 2026-06-29  
**Domain:** Ultra 205 safety-critical hardware regression evidence, parity gating, and ESP-IDF Rust firmware evidence capture  
**Confidence:** HIGH for repo patterns and current evidence gates; MEDIUM for DS4432U datasheet details because the official Analog PDF could not be fetched during research.

<user_constraints>
## User Constraints (from CONTEXT.md)

Source: copied from `.planning/phases/11-safety-controller-hardware-regression-evidence/11-CONTEXT.md`. [VERIFIED: .planning/phases/11-safety-controller-hardware-regression-evidence/11-CONTEXT.md]

### Locked Decisions

## Implementation Decisions

### Hardware Run And Recovery Protocol

- **D-01:** Use a strict phase-gated runbook as the baseline for all live Ultra 205 hardware work. The runbook must start with `just detect-ultra205` and may continue only when it finds exactly one likely ESP USB serial port and `espflash board-info --chip esp32s3 --port <port> --non-interactive` succeeds.
- **D-02:** Record a recovery path before any hardware actuation, destructive, stress, or fault-injection verification. Without a documented recovery path and exact allowed command set, the task must stop and record evidence as pending or blocked.
- **D-03:** Use observe-only evidence as the safe fallback when detection passes but bench recovery, stimulus, or actuation prerequisites are incomplete. Observe-only evidence can prove safe-unavailable or read-only observations, but it cannot verify voltage writes, fan actuation, overheat/fault injection, destructive recovery, or true load stress.
- **D-04:** Prefer scripted bounded regression probes for repeatable safety evidence only when the probe has explicit limits, redaction, recovery instructions, and a fail-closed outcome. Manual bench evidence is acceptable for physical observations such as visible fan behavior, display/input, or recovery observations that cannot yet be safely automated.

### Sensor And Actuator Evidence Coverage

- **D-05:** Build the Phase 11 evidence around a tiered per-surface matrix. Each matrix row should name the checklist row, component, claim type, allowed command or probe, required metadata, pass criteria, failure criteria, evidence artifact, and whether the claim supports `hardware-smoke`, `hardware-regression`, or remains below verified.
- **D-06:** Separate telemetry reads from actuator writes. INA260 current/voltage/power freshness, EMC2101 or equivalent thermal readings, and fan RPM observations may be evidenced independently from DS4432U voltage writes, fan duty effects, ASIC reset/power sequencing, or overheat/fault actuation.
- **D-07:** Use component-scoped evidence packs under one Phase 11 ledger so DS4432U, INA260, thermal/fan/PID, self-test/watchdog, and display/input conclusions can be promoted or held independently without treating one happy-path log as proof for the full safety surface.
- **D-08:** Do not promote broad rows from black-box flash/monitor smoke alone. A boot log can support board identity, safe startup, and safe-unavailable status, but active voltage, fan, thermal, power, self-test hardware, and failure-path parity need targeted evidence.

### Self-Test, Display/Input, Watchdog, And Load Responsiveness

- **D-09:** Attempt narrow live evidence only where an existing firmware, API, log, WebSocket, or serial route can safely expose the state. Self-test, watchdog status, safety supervisor yield behavior, API/log/WebSocket responsiveness, and safe-unavailable telemetry are good candidates when they require no unsafe stimulus.
- **D-10:** Keep runtime display/input below verified unless a real runtime display/input route exists and can be exercised safely. Existing startup-only SSD1306 evidence is supporting evidence only; it must not be used as verified runtime display/input parity.
- **D-11:** Watchdog/load evidence should prove observable liveness or responsiveness under a bounded, documented workload. Do not infer watchdog parity only from the Phase 6 pure model or from a boot log. Fault-injection or stress-style watchdog checks require explicit stimulus, stop conditions, and recovery.
- **D-12:** If self-test, display/input, watchdog, or load responsiveness cannot be safely run, update the evidence ledger and checklist notes with owner/follow-up instead of silently leaving stale Phase 6 wording.

### Checklist Promotion And Parity Guard Rules

- **D-13:** Use tiered promotion by claim type. `hardware-smoke` can verify a narrow board-named observation such as boot identity, safe-unavailable status, read-only telemetry, or visible bounded behavior. Active voltage/fan/thermal/power/self-test/failure-path parity should require `hardware-regression`.
- **D-14:** Mixed checklist rows should either stay below `verified` or be split into exact subclaims when one row combines pure logic, safe-unavailable telemetry, active control, and failure handling. Do not let one narrow observation verify an entire mixed safety surface.
- **D-15:** Preserve the existing parity guard behavior that rejects safety-critical `verified` rows without `hardware-smoke` or `hardware-regression`; extend tests only when Phase 11 needs more precise row or evidence semantics.
- **D-16:** Every Phase 11 evidence artifact must record board `205`, selected port, source commit, reference commit, exact command or probe, package manifest or firmware identity when applicable, logs, observed behavior, conclusion, and a secret-redaction review.

### the agent's Discretion

The agent may choose exact plan count, evidence file names, JSON schema details, Rust helper names, and whether to implement a minimal hardware-regression CLI or keep evidence as structured docs. Those choices must keep `reference/esp-miner` read-only, preserve functional core plus imperative shell, use repo-owned ESP/esp-rs tooling before custom hardware paths, keep unsafe operations phase-gated, and avoid committing secrets, pool credentials, Wi-Fi credentials, private endpoints, or NVS secret values in evidence.

### Deferred Ideas (OUT OF SCOPE)

## Deferred Ideas

- Full LVGL runtime display carousel, display config, timeout, rotation, inversion, and broad button-routing parity remain outside Phase 11 unless a later roadmap phase explicitly owns them.
- ASIC initialization, work-send/result-receive, mining-loop smoke, and mining soak evidence belong to Phase 12.
- Final package-to-hardware release evidence, live HTTP/static/recovery/OTA/rollback/erase/interrupted-update proof, and `DEVICE_URL` release evidence belong to Phase 13.
- Non-205 boards, BM1370/BM1368/BM1397, TPS546 hardware behavior, all-board factory images, Stratum v2, BAP, and Angular UI replacement remain deferred.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
| --- | --- | --- |
| SAFE-01 | Ultra 205 voltage and power-control surfaces use bounded typed decisions and fail closed on invalid configuration, communication failure, or unsafe readings. | Plan DS4432U voltage-write claims separately from INA260 read-only telemetry and keep active voltage writes below verified unless bounded hardware-regression evidence exists. [VERIFIED: .planning/REQUIREMENTS.md] [VERIFIED: crates/bitaxe-safety/src/power.rs] [VERIFIED: firmware/bitaxe/src/safety_adapter/power.rs] |
| SAFE-02 | Thermal sensor and fan control surfaces expose upstream-compatible readings, fan duty behavior, RPM behavior, and failure reporting. | Plan EMC2101 temperature/RPM evidence separately from fan-duty actuation; use upstream fan task behavior only as a reference target, not as evidence. [VERIFIED: .planning/REQUIREMENTS.md] [VERIFIED: crates/bitaxe-safety/src/thermal.rs] [VERIFIED: reference/esp-miner/main/tasks/fan_controller_task.c] |
| SAFE-03 | PID and thermal-control decisions are covered by pure unit tests before hardware effects are enabled. | Keep existing pure PID tests in the verification gate and add hardware evidence only after the model stays green. [VERIFIED: .planning/REQUIREMENTS.md] [VERIFIED: crates/bitaxe-safety/src/thermal.rs] |
| SAFE-04 | Overheat, fan, power, thermal, and ASIC fault paths enter safe states and expose user-visible status compatible with upstream behavior. | Treat overheat, fan-fault, reset-low, VCORE-zero, and restart paths as destructive or stress-sensitive; plan recovery before any fault injection. [VERIFIED: .planning/REQUIREMENTS.md] [VERIFIED: crates/bitaxe-safety/src/fault.rs] [VERIFIED: reference/esp-miner/main/tasks/power_management_task.c] |
| SAFE-05 | Self-test lifecycle behavior covers factory flags, start, pass, fail, restart, cancel, and user-visible result reporting. | Self-test hardware evidence must be gated because upstream self-test changes fan duty, voltage state, ASIC work, and reset behavior. [VERIFIED: .planning/REQUIREMENTS.md] [VERIFIED: crates/bitaxe-safety/src/self_test.rs] [VERIFIED: reference/esp-miner/main/self_test/self_test.c] |
| SAFE-06 | Display and input status surfaces needed for normal Ultra 205 administration are preserved or explicitly documented as deferred gaps. | Startup-only SSD1306 evidence can support startup display status but does not verify runtime display/input parity. [VERIFIED: .planning/REQUIREMENTS.md] [VERIFIED: firmware/bitaxe/src/display_adapter.rs] [VERIFIED: docs/parity/evidence/phase-06-display-input-runtime-gap.md] |
| SAFE-07 | Power, current, voltage, fan, and temperature telemetry are captured where Ultra 205 hardware exposes them. | Prioritize read-only telemetry packs for INA260, EMC2101 temperature, fan RPM, and safe-unavailable telemetry before any actuator probes. [VERIFIED: .planning/REQUIREMENTS.md] [VERIFIED: firmware/bitaxe/src/safety_adapter/power.rs] [VERIFIED: firmware/bitaxe/src/safety_adapter/thermal.rs] |
| SAFE-08 | Safety-critical surfaces cannot be marked `verified` without `hardware-smoke` or `hardware-regression` evidence. | Preserve `tools/parity` safety-critical verified-row rejection and only extend tests if Phase 11 adds stricter evidence semantics. [VERIFIED: .planning/REQUIREMENTS.md] [VERIFIED: tools/parity/src/main.rs] |
| SAFE-09 | Mining, control, API, and telemetry tasks avoid watchdog starvation and preserve observable responsiveness under load. | Watchdog/load evidence needs bounded liveness or responsiveness observations; pure watchdog model and boot logs alone are insufficient. [VERIFIED: .planning/REQUIREMENTS.md] [VERIFIED: crates/bitaxe-safety/src/watchdog.rs] [VERIFIED: firmware/bitaxe/src/safety_adapter/watchdog.rs] |
| EVD-05 | Verification layers include unit tests, golden fixtures, API comparison, hardware smoke tests, and hardware regression or soak evidence where appropriate. | Phase 11 should produce board-205 evidence ledgers and run existing pure, parity, and command-surface checks around any live hardware evidence. [VERIFIED: .planning/REQUIREMENTS.md] [VERIFIED: docs/parity/checklist.md] [VERIFIED: Justfile] |
</phase_requirements>

## Summary

Phase 11 should be planned as a safety evidence phase, not as a broad hardware-enablement phase. The current firmware safety adapters are observe-only and suppress voltage and fan effects by default, while the pure safety crate already models fail-closed decisions, PID behavior, self-test decisions, fault states, and watchdog-friendly stepping. [VERIFIED: firmware/bitaxe/src/safety_adapter.rs] [VERIFIED: firmware/bitaxe/src/safety_adapter/power.rs] [VERIFIED: firmware/bitaxe/src/safety_adapter/thermal.rs] [VERIFIED: crates/bitaxe-safety/src/power.rs] [VERIFIED: crates/bitaxe-safety/src/thermal.rs] [VERIFIED: crates/bitaxe-safety/src/self_test.rs] [VERIFIED: crates/bitaxe-safety/src/fault.rs] [VERIFIED: crates/bitaxe-safety/src/watchdog.rs]

The planner should create a strict runbook and component-scoped evidence matrix before any live hardware step. The first live command must be `just detect-ultra205`; detection is successful only when exactly one likely ESP USB serial port is found and `espflash board-info --chip esp32s3 --port <port> --non-interactive` succeeds. [VERIFIED: AGENTS.md] [VERIFIED: scripts/detect-ultra205.sh]

**Primary recommendation:** plan Phase 11 as a tiered evidence ledger: safe observe-only baseline first, read-only telemetry packs second, bounded actuator/failure-path regression only with a written recovery path, and checklist promotion only for the exact claim proven. [VERIFIED: .planning/phases/11-safety-controller-hardware-regression-evidence/11-CONTEXT.md] [VERIFIED: docs/parity/checklist.md] [VERIFIED: tools/parity/src/main.rs]

## Project Constraints (from AGENTS.md)

- Read repo-local `AGENTS.md`, `AGENTS.bright-builds.md`, `standards-overrides.md`, and relevant `standards/` pages before planning, review, implementation, or audit work. [VERIFIED: AGENTS.md]
- Use the pinned ESP-IDF Rust workflow, `esp-idf-sys` metadata, `.cargo/config.toml`, `espup`, `ldproxy`, and `espflash` before custom CMake, PlatformIO, or manually managed ESP-IDF installs. [VERIFIED: AGENTS.md]
- Treat `.embuild/` as local generated ESP-IDF/esp-rs tool state that must not be committed or hand-edited. [VERIFIED: AGENTS.md]
- Use `just doctor` for read-only dependency checks and `just bootstrap-esp` for explicit opt-in ESP tooling installation. [VERIFIED: AGENTS.md] [VERIFIED: Justfile]
- Autonomous Ultra 205 hardware use is allowed only after `just detect-ultra205` succeeds exactly as defined by repo instructions. [VERIFIED: AGENTS.md] [VERIFIED: scripts/detect-ultra205.sh]
- Stop and ask or record hardware evidence pending when there are zero likely ports, multiple likely ports, `board-info` fails, the target is not board `205`, or required recovery/evidence instructions are missing. [VERIFIED: AGENTS.md]
- Phase-gated destructive or fault-injection verification is allowed only when the active phase plan documents the recovery path and required evidence. [VERIFIED: AGENTS.md]
- Every hardware run must record board `205`, selected port, source commit, reference commit, package manifest/artifacts when applicable, exact commands, board-info output, captured logs, observed behavior, and conclusion. [VERIFIED: AGENTS.md]
- Evidence must not commit secrets, pool credentials, Wi-Fi credentials, private endpoints, or NVS secret values. [VERIFIED: AGENTS.md]
- Keep `reference/esp-miner` pinned and read-only; it is evidence, not a workspace for project changes. [VERIFIED: AGENTS.md] [VERIFIED: PROVENANCE.md]
- Preserve functional core plus imperative shell: pure logic belongs in testable crates, while ESP-IDF, FreeRTOS, Wi-Fi, NVS, SPIFFS, OTA, serial, GPIO, I2C, ADC, power, display, and task orchestration stay in firmware adapters. [VERIFIED: AGENTS.md] [VERIFIED: standards/core/architecture.md]
- Parse at boundaries, prefer domain types, make illegal states unrepresentable, and keep policy out of adapters where practical. [VERIFIED: standards/core/architecture.md]
- Use early returns, shallow control flow, clear names, and targeted comments for the "why" rather than the "what." [VERIFIED: standards/core/code-shape.md]
- Rust commits must run `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, and `cargo test --all-features` before commit. [VERIFIED: AGENTS.md] [VERIFIED: standards/languages/rust.md]
- Unit tests should test one concern and use Arrange, Act, Assert structure when it improves clarity. [VERIFIED: AGENTS.md] [VERIFIED: standards/core/testing.md]
- GSD and other frontmatter-parsed Markdown files must not use standalone `---` body separators after frontmatter. [VERIFIED: AGENTS.md]

## Standard Stack

### Core

| Library / Tool | Version | Purpose | Why Standard |
| --- | --- | --- | --- |
| `bitaxe-safety` | workspace `0.1.0` | Pure safety decisions for voltage, power telemetry, thermal/PID, faults, self-test, and watchdog stepping. | Existing functional-core home for safety behavior; Phase 11 should verify or preserve these decisions rather than moving policy into firmware adapters. [VERIFIED: Cargo.toml] [VERIFIED: crates/bitaxe-safety/src/power.rs] |
| `firmware/bitaxe` | workspace `0.1.0` | ESP-IDF firmware shell, observe-only safety adapters, display adapter, runtime snapshot, and watchdog supervisor. | Existing imperative shell owns hardware boundaries and currently suppresses unsafe voltage/fan effects by default. [VERIFIED: Cargo.toml] [VERIFIED: firmware/bitaxe/src/safety_adapter.rs] |
| `bitaxe-flash` | workspace `0.1.0` | `flash`, `monitor`, and `flash-monitor` command wrapper with evidence JSON/log output. | Already records board, port, commits, manifest/image paths, trusted output, commands, and conclusion for hardware evidence runs. [VERIFIED: tools/flash/src/main.rs] |
| `bitaxe-parity` | workspace `0.1.0` | Checklist validation and safety-critical evidence-token rejection. | Already rejects safety-critical `verified` rows without `hardware-smoke` or `hardware-regression`. [VERIFIED: tools/parity/src/main.rs] |
| `just` | `1.48.0` | Human command surface for detect, build, test, package, flash, monitor, parity, and ESP bootstrap. | Repo-local commands route through `just`; command availability was verified locally. [VERIFIED: Justfile] [VERIFIED: command probe `just --version`] |
| Bazel / Bazelisk | `.bazelversion` `9.1.1` | Canonical automation graph for build and test targets. | Project stack and local `.bazelversion` make Bazel the canonical graph. [VERIFIED: .bazelversion] [VERIFIED: MODULE.bazel] |
| `espflash` / `cargo-espflash` | `4.0.1` | ESP32-S3 board info, flashing, and serial monitor capture. | Detector and wrapper use `espflash`; command availability was verified locally. [VERIFIED: scripts/detect-ultra205.sh] [VERIFIED: tools/flash/src/main.rs] [VERIFIED: command probe `espflash --version`] |
| ESP-IDF Rust stack | `esp-idf-svc 0.52.1`, `esp-idf-sys 0.37.2`, ESP-IDF `tag:v5.5.4` | Firmware services and ESP-IDF binding layer. | Existing firmware metadata pins the IDF version and workspace dependencies. [VERIFIED: Cargo.toml] [VERIFIED: firmware/bitaxe/Cargo.toml] |

### Supporting

| Library / Tool | Version | Purpose | When to Use |
| --- | --- | --- | --- |
| `serde` / `serde_json` | `serde 1.0.228`, `serde_json 1.0.150` | Evidence JSON schemas and structured command output. | Use if Phase 11 adds generated evidence packs or a hardware-regression helper CLI. [VERIFIED: Cargo.toml] |
| `clap` | `4.6.1` | Strict CLI parsing for any new repo-owned probe helper. | Use for board, port, evidence directory, probe name, and explicit actuation opt-ins. [VERIFIED: Cargo.toml] |
| `camino` | `1.2.3` | UTF-8 path handling in host tooling. | Use for evidence paths and artifact paths in host CLIs. [VERIFIED: Cargo.toml] |
| `tempfile` | `3.27.0` | CLI and parser tests for generated evidence files. | Use in host-tool tests when Phase 11 adds structured output. [VERIFIED: Cargo.toml] |
| `log` + `EspLogger` | `log 0.4`, crate-managed ESP logger | Firmware logging to ESP-IDF serial output. | Use for new firmware observation points; avoid `println!` in firmware. [VERIFIED: Cargo.toml] [VERIFIED: firmware/bitaxe/src/main.rs] |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
| --- | --- | --- |
| Repo-owned `just detect-ultra205` and `bitaxe-flash` | Direct `espflash` commands | Direct commands lose the repo's detection gate, wrapper-owned evidence JSON/log pattern, and trusted-output checks. [VERIFIED: AGENTS.md] [VERIFIED: tools/flash/src/main.rs] |
| Existing Rust host CLI pattern | Ad hoc shell or Python I2C probes | Ad hoc probes would bypass domain parsing, explicit limits, Bazel tests, redaction, and fail-closed behavior. [VERIFIED: standards/core/architecture.md] [VERIFIED: standards/languages/rust.md] |
| Checklist plus `tools/parity` guard | Manual checklist edits only | Manual edits can overclaim safety-critical rows; current guard rejects unsafe `verified` promotions. [VERIFIED: docs/parity/checklist.md] [VERIFIED: tools/parity/src/main.rs] |

**Installation:**

```bash
# No new dependencies are recommended for Phase 11 research.
# Use existing repo commands and workspace crates.
just doctor
```

**Version verification:** No npm packages are recommended. Versions above were verified from local manifests and command probes: `.bazelversion`, `MODULE.bazel`, `Cargo.toml`, `firmware/bitaxe/Cargo.toml`, `just --version`, `espflash --version`, and `cargo-espflash --version`. [VERIFIED: .bazelversion] [VERIFIED: MODULE.bazel] [VERIFIED: Cargo.toml] [VERIFIED: firmware/bitaxe/Cargo.toml] [VERIFIED: command probes]

## Architecture Patterns

### Recommended Project Structure

```text
docs/parity/evidence/
├── phase-11-safety-controller-hardware-regression-evidence.md  # Human-readable ledger, runbook, matrix, conclusions.
└── phase-11-safety-controller-hardware-regression-evidence/     # Optional generated artifacts if probes run.
    ├── flash-command-evidence.json
    ├── flash-monitor.log
    ├── ina260-telemetry.json
    ├── thermal-fan-observation.json
    ├── watchdog-load-observation.json
    └── redaction-review.md

tools/
└── safety-regression/  # Add only if scripted bounded probes are needed.
    ├── src/main.rs
    └── BUILD.bazel
```

The evidence path should be under `docs/parity/evidence/` because existing phase evidence and checklist breadcrumbs live there. [VERIFIED: docs/parity/evidence/phase-06-safety-controllers-and-self-test.md] [VERIFIED: docs/parity/evidence/phase-09-flash-monitor-evidence-wrapper-hardening.md] [VERIFIED: docs/parity/checklist.md]

### Pattern 1: Tiered Evidence Matrix

**What:** Create one matrix row per exact claim, not per broad component. Each row should include checklist row, component, claim type, allowed command/probe, metadata, pass criteria, failure criteria, artifact, promotion token, and conclusion. [VERIFIED: .planning/phases/11-safety-controller-hardware-regression-evidence/11-CONTEXT.md]

**When to use:** Use this for every Phase 11 safety surface: DS4432U, INA260, EMC2101, fan/PID, self-test, display/input, watchdog/load, and parity guard semantics. [VERIFIED: .planning/phases/11-safety-controller-hardware-regression-evidence/11-CONTEXT.md]

**Example:**

```markdown
| Checklist Row | Component | Claim Type | Allowed Probe | Pass Criteria | Evidence Token | Conclusion |
| --- | --- | --- | --- | --- | --- | --- |
| PWR-006 | INA260 | Read-only telemetry freshness | `just flash-monitor board=205 port=<port> evidence-dir=<dir>` plus firmware/API telemetry route if present | Board 205, commit IDs, non-secret logs, current/voltage/power observed or explicitly safe-unavailable | `hardware-smoke` or pending | Do not promote DS4432U writes from this row |
```

Source: Phase 11 context requires per-surface evidence matrices and independent promotion. [VERIFIED: .planning/phases/11-safety-controller-hardware-regression-evidence/11-CONTEXT.md]

### Pattern 2: Safe Baseline Before Actuation

**What:** First capture detector output, board-info, firmware identity, safe-state boot logs, and observe-only safety telemetry before any actuator step. [VERIFIED: AGENTS.md] [VERIFIED: tools/flash/src/main.rs] [VERIFIED: firmware/bitaxe/src/safety_adapter.rs]

**When to use:** Use at the start of the phase and before each actuation-capable task after firmware or command-surface changes. [VERIFIED: AGENTS.md] [VERIFIED: .planning/phases/11-safety-controller-hardware-regression-evidence/11-CONTEXT.md]

**Example command sequence:**

```bash
just detect-ultra205
just flash-monitor board=205 port=<path-from-detect> evidence-dir=docs/parity/evidence/phase-11-safety-controller-hardware-regression-evidence
just parity
```

The detector command and wrapper command are existing repo commands. [VERIFIED: Justfile] [VERIFIED: scripts/detect-ultra205.sh] [VERIFIED: tools/flash/src/main.rs]

### Pattern 3: Component-Scoped Evidence Packs

**What:** Split evidence and conclusions by hardware surface so a successful INA260 read does not imply DS4432U voltage-write parity, and startup OLED evidence does not imply runtime display/input parity. [VERIFIED: .planning/phases/11-safety-controller-hardware-regression-evidence/11-CONTEXT.md] [VERIFIED: docs/parity/evidence/phase-06-display-input-runtime-gap.md]

**When to use:** Use when updating `docs/parity/checklist.md`; narrow evidence can promote narrow rows or subclaims but should leave mixed rows below `verified`. [VERIFIED: docs/parity/checklist.md] [VERIFIED: tools/parity/src/main.rs]

**Example component packs:**

```text
1. safe-baseline: detector, board-info, boot identity, source/reference commits, redaction review.
2. power-telemetry: INA260 read-only current, bus voltage, and power status or safe-unavailable result.
3. voltage-control: DS4432U write behavior only if bounded actuation and recovery are documented.
4. thermal-fan: EMC2101 temperature/RPM reads first, fan duty behavior only with bounded probe.
5. self-test-watchdog-load: existing state routes, supervisor yield logs, bounded responsiveness observations.
6. display-input: startup evidence and explicit runtime gap unless runtime route exists.
```

Source: Phase 11 context requires component-scoped evidence packs and tiered promotion. [VERIFIED: .planning/phases/11-safety-controller-hardware-regression-evidence/11-CONTEXT.md]

### Pattern 4: Preserve Pure-Core / Imperative-Shell Boundary

**What:** Keep safety policy and pass/fail classification in `crates/bitaxe-safety`; keep ESP-IDF, I2C, GPIO, fan/PWM, display, serial, API, and hardware effects in `firmware/bitaxe` or a host probe wrapper. [VERIFIED: AGENTS.md] [VERIFIED: standards/core/architecture.md] [VERIFIED: crates/bitaxe-safety/src/power.rs] [VERIFIED: firmware/bitaxe/src/safety_adapter.rs]

**When to use:** Use this boundary if the phase needs new observation routes or a minimal `tools/safety-regression` helper. [VERIFIED: standards/core/architecture.md]

**Example Rust CLI shape:**

```rust
// Source: existing CLI/dependency pattern. [VERIFIED: tools/flash/src/main.rs] [VERIFIED: Cargo.toml]
#[derive(clap::Parser)]
struct Args {
    #[arg(long)]
    board: u16,
    #[arg(long)]
    port: camino::Utf8PathBuf,
    #[arg(long)]
    evidence_dir: camino::Utf8PathBuf,
    #[arg(long)]
    probe: ProbeName,
}
```

### Anti-Patterns to Avoid

- **One broad smoke log verifies every safety surface:** A boot log can support board identity and safe startup, but it cannot verify active voltage, fan, thermal, power, self-test hardware, or failure-path parity. [VERIFIED: .planning/phases/11-safety-controller-hardware-regression-evidence/11-CONTEXT.md]
- **Actuation before recovery:** Any voltage write, fan actuation, stress, destructive, or fault-injection step without a written recovery path violates the phase decisions and repo hardware rules. [VERIFIED: .planning/phases/11-safety-controller-hardware-regression-evidence/11-CONTEXT.md] [VERIFIED: AGENTS.md]
- **Updating checklist status before evidence exists:** `tools/parity` already rejects safety-critical `verified` rows without hardware evidence tokens. [VERIFIED: tools/parity/src/main.rs]
- **Using startup display evidence for runtime display/input parity:** Phase 6 explicitly documents startup-only display evidence and a runtime display/input gap. [VERIFIED: docs/parity/evidence/phase-06-display-input-runtime-gap.md]
- **Using direct upstream C code as implementation:** Reference files are GPL-covered evidence; original Rust work should avoid copying upstream C expression unless intentionally isolated and marked. [VERIFIED: PROVENANCE.md]

## Hardware Facts To Plan Around

| Surface | Verified Facts | Planning Implication |
| --- | --- | --- |
| Ultra 205 default config | Upstream config for board 205 sets BM1366, default frequency `485`, default voltage `1200`, fan auto enabled, fanspeed `100`, and self-test enabled. [VERIFIED: reference/esp-miner/config-205.cvs] | Do not assume Phase 11 should run mining or full self-test just because upstream defaults include them; self-test and mining remain gated by recovery and scope. [VERIFIED: .planning/phases/11-safety-controller-hardware-regression-evidence/11-CONTEXT.md] |
| Ultra 205 hardware profile | Upstream device config marks board 205 with EMC2101, internal temperature path, DS4432U, INA260, plug sense, ASIC enable, target power `12W`, and BM1366 voltage options `1100` through `1300`. [VERIFIED: reference/esp-miner/main/device_config.h] | Evidence should distinguish supported hardware presence from verified Rust behavior. [VERIFIED: docs/adr/0012-parity-verification-evidence.md] |
| DS4432U | Existing Rust adapter and upstream reference both use address `0x48`, OUT0 `0xF8`, and OUT1 `0xF9`; the official Analog datasheet URL could not be fetched during research. [VERIFIED: firmware/bitaxe/src/safety_adapter/power.rs] [VERIFIED: reference/esp-miner/main/power/DS4432U.c] | Treat DS4432U register details as repo/reference-verified but do not cite external datasheet-only claims in the plan unless the planner fetches Analog documentation. |
| INA260 | TI documents current register `01h`, bus voltage register `02h`, power register `03h`, and LSBs of 1.25 mA, 1.25 mV, and 10 mW. [CITED: https://www.ti.com/lit/ds/symlink/ina260.pdf] | Read-only current/voltage/power evidence can be planned as a lower-risk telemetry pack before actuator evidence. [VERIFIED: .planning/phases/11-safety-controller-hardware-regression-evidence/11-CONTEXT.md] |
| EMC2101 | Microchip documents SMBus fan control, internal/external diode temperature monitoring, TACH measurement, Fan Setting Register `4Ch`, and TACH Reading Low/High registers `46h`/`47h`. [CITED: https://ww1.microchip.com/downloads/aemDocuments/documents/MSLD/ProductDocuments/DataSheets/EMC2101-Data-Sheet-DS20006703.pdf] | Plan temperature/RPM reads separately from fan-duty actuation, and require physical/manual or bounded probe evidence for visible fan behavior. [VERIFIED: .planning/phases/11-safety-controller-hardware-regression-evidence/11-CONTEXT.md] |
| Fan/PID reference | Upstream fan task polls at `100ms`, logs around `2000ms`, uses P `5`, I `0.1`, D `2`, sets overheat `100%`, paused/no-pool `30%`, and startup `70%`. [VERIFIED: reference/esp-miner/main/tasks/fan_controller_task.c] | Pure PID tests can remain host verification; live fan duty/RPM needs component-scoped hardware evidence. [VERIFIED: crates/bitaxe-safety/src/thermal.rs] |
| Overheat recovery reference | Upstream power management stops mining, sets VCORE to `0`, holds reset low, cools to safe temperature, reduces voltage/frequency, and restarts mining after overheat. [VERIFIED: reference/esp-miner/main/tasks/power_management_task.c] | Overheat/fault regression is destructive or stress-sensitive and should stay pending unless the phase plan defines stimulus, stop conditions, recovery, and evidence. [VERIFIED: AGENTS.md] |
| Self-test reference | Upstream self-test changes fan duty, initializes ASIC work, checks voltage/power/fan/hashrate, sets VCORE to `0`, holds reset low, and may reboot or wait for BOOT button. [VERIFIED: reference/esp-miner/main/self_test/self_test.c] | Self-test hardware evidence is not a generic safe smoke test; it requires explicit recovery and scope. [VERIFIED: .planning/phases/11-safety-controller-hardware-regression-evidence/11-CONTEXT.md] |
| Display/input boundary | Current Rust display adapter initializes startup SSD1306 at address `0x3c` with SDA `47`, SCL `48`, and publishes a runtime display/input boundary; Phase 6 recorded runtime gap evidence. [VERIFIED: firmware/bitaxe/src/display_adapter.rs] [VERIFIED: docs/parity/evidence/phase-06-display-input-runtime-gap.md] | Runtime display/input rows should stay below verified unless a real runtime route is implemented and exercised. [VERIFIED: .planning/phases/11-safety-controller-hardware-regression-evidence/11-CONTEXT.md] |
| Watchdog/supervisor | Current firmware safety supervisor logs start and yield behavior with a `100ms` cadence; pure watchdog model has step budget and consecutive-step limits. [VERIFIED: firmware/bitaxe/src/safety_adapter/watchdog.rs] [VERIFIED: crates/bitaxe-safety/src/watchdog.rs] | Plan liveness evidence around bounded observable responsiveness, not only boot success. [VERIFIED: .planning/phases/11-safety-controller-hardware-regression-evidence/11-CONTEXT.md] |

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
| --- | --- | --- | --- |
| Board/port detection | Custom serial-port guessing in the plan | `just detect-ultra205` | Repo instructions require this detector and its `espflash board-info` gate before autonomous hardware use. [VERIFIED: AGENTS.md] [VERIFIED: scripts/detect-ultra205.sh] |
| Flash/monitor evidence | Direct monitor capture with hand-written metadata | `just flash-monitor board=205 port=<port> evidence-dir=<path>` | Wrapper already writes command/evidence JSON and logs with trusted-output logic. [VERIFIED: tools/flash/src/main.rs] |
| Safety verification gate | Manual checklist-only review | `just parity` and `tools/parity` tests | Existing tooling rejects safety-critical overclaims. [VERIFIED: tools/parity/src/main.rs] [VERIFIED: Justfile] |
| Voltage/fan/overheat probes | Raw I2C writes, shell snippets, or one-off Python | Repo-owned Rust helper with explicit limits, tests, redaction, and recovery, or structured manual evidence | Hardware actuation is safety-critical and must remain phase-gated and fail-closed. [VERIFIED: AGENTS.md] [VERIFIED: standards/core/architecture.md] |
| Sensor math and registers | Unverified rewritten formulas | Local adapter/reference constants plus official datasheets where available | INA260 and EMC2101 register behavior is documented by vendor datasheets; DS4432U should stay tied to repo/reference until Analog docs are fetched. [CITED: https://www.ti.com/lit/ds/symlink/ina260.pdf] [CITED: https://ww1.microchip.com/downloads/aemDocuments/documents/MSLD/ProductDocuments/DataSheets/EMC2101-Data-Sheet-DS20006703.pdf] [VERIFIED: reference/esp-miner/main/power/DS4432U.c] |
| Runtime display/input parity | Treat startup OLED as runtime proof | Keep UI runtime gap below verified unless physically exercised through a runtime route | Existing evidence says startup-only display is supporting evidence, not runtime parity. [VERIFIED: docs/parity/evidence/phase-06-display-input-runtime-gap.md] |
| Reference comparison | Editing `reference/esp-miner` or porting C expression directly | Breadcrumbs, independent Rust design, and provenance notes | The reference tree is read-only evidence and GPL provenance must be preserved. [VERIFIED: PROVENANCE.md] |

**Key insight:** Phase 11's hard problem is evidence classification, not driver discovery. Current code already has safe pure decisions, observe-only adapters, a detector gate, wrapper evidence capture, and parity rejection; the plan should combine those without widening hardware risk. [VERIFIED: crates/bitaxe-safety/src/power.rs] [VERIFIED: firmware/bitaxe/src/safety_adapter.rs] [VERIFIED: scripts/detect-ultra205.sh] [VERIFIED: tools/flash/src/main.rs] [VERIFIED: tools/parity/src/main.rs]

## Common Pitfalls

### Pitfall 1: Overclaiming Mixed Checklist Rows

**What goes wrong:** A row that combines pure logic, telemetry, active control, and failure handling gets marked `verified` from one narrow observation. [VERIFIED: .planning/phases/11-safety-controller-hardware-regression-evidence/11-CONTEXT.md]

**Why it happens:** Current checklist rows can be broad, while evidence tokens are narrower than the row labels. [VERIFIED: docs/parity/checklist.md]

**How to avoid:** Split conclusions by claim type or keep broad rows below `verified` with explicit follow-up. [VERIFIED: .planning/phases/11-safety-controller-hardware-regression-evidence/11-CONTEXT.md]

**Warning signs:** Evidence says "boot log observed" while checklist status implies voltage writes, fan actuation, or overheat recovery are verified. [VERIFIED: docs/parity/evidence/phase-06-ultra-205-safety-hardware-smoke.md]

### Pitfall 2: Skipping The Hardware Gate

**What goes wrong:** A live command runs on the wrong port, no board, or a non-205 target. [VERIFIED: AGENTS.md]

**Why it happens:** Serial-port discovery can be ambiguous across host OSes and USB bridges. [VERIFIED: AGENTS.md]

**How to avoid:** Start every live run with `just detect-ultra205` and stop on zero ports, multiple likely ports, failed board-info, or non-205 target. [VERIFIED: AGENTS.md] [VERIFIED: scripts/detect-ultra205.sh]

**Warning signs:** Evidence lacks detector output or `espflash board-info --chip esp32s3 --port <port> --non-interactive`. [VERIFIED: AGENTS.md]

### Pitfall 3: Confusing Safe-Unavailable With Active Verification

**What goes wrong:** Suppressed or unavailable voltage/fan effects are treated as proof that active hardware control works. [VERIFIED: firmware/bitaxe/src/safety_adapter/power.rs] [VERIFIED: firmware/bitaxe/src/safety_adapter/thermal.rs]

**Why it happens:** Current adapters intentionally expose observe-only or suppressed behavior for safety. [VERIFIED: firmware/bitaxe/src/safety_adapter.rs]

**How to avoid:** Use `hardware-smoke` for safe-unavailable/read-only proof and require `hardware-regression` for active voltage/fan/fault paths. [VERIFIED: .planning/phases/11-safety-controller-hardware-regression-evidence/11-CONTEXT.md]

**Warning signs:** An evidence artifact has no actuator command, no stop conditions, and no recovery path but claims active control parity. [VERIFIED: AGENTS.md]

### Pitfall 4: Stale Sensor Values Masquerading As Fresh Reads

**What goes wrong:** A cached or last-known telemetry value is mistaken for fresh INA260 or thermal evidence. [VERIFIED: reference/esp-miner/main/power/INA260.c]

**Why it happens:** The upstream INA260 implementation caches previous readings when I2C reads fail. [VERIFIED: reference/esp-miner/main/power/INA260.c]

**How to avoid:** Require freshness metadata, read success/failure status, timestamp, and failure-path conclusion in telemetry evidence. [VERIFIED: .planning/phases/11-safety-controller-hardware-regression-evidence/11-CONTEXT.md]

**Warning signs:** Evidence records current/voltage/power values without I2C result status or timing. [VERIFIED: reference/esp-miner/main/power/INA260.c]

### Pitfall 5: Letting Phase 11 Expand Into Later Phases

**What goes wrong:** Safety evidence work turns into ASIC mining smoke, release HTTP/OTA proof, broad LVGL runtime UI parity, or non-205 board support. [VERIFIED: .planning/phases/11-safety-controller-hardware-regression-evidence/11-CONTEXT.md]

**Why it happens:** Upstream self-test and overheat paths overlap with mining, display, and recovery behavior. [VERIFIED: reference/esp-miner/main/self_test/self_test.c] [VERIFIED: reference/esp-miner/main/tasks/power_management_task.c]

**How to avoid:** Keep Phase 11 to board-205 safety evidence and record deferred items explicitly. [VERIFIED: .planning/phases/11-safety-controller-hardware-regression-evidence/11-CONTEXT.md]

**Warning signs:** Plan tasks mention Stratum v2, BAP, all-board images, release OTA rollback, mining soak, or broad display carousel parity. [VERIFIED: .planning/phases/11-safety-controller-hardware-regression-evidence/11-CONTEXT.md]

## Code Examples

Verified patterns from repo sources:

### Detector Gate

```bash
just detect-ultra205
```

The detector must find exactly one likely ESP USB serial port and run `espflash board-info --chip esp32s3 --port <port> --non-interactive` before autonomous hardware use continues. [VERIFIED: AGENTS.md] [VERIFIED: scripts/detect-ultra205.sh]

### Wrapper-Owned Evidence Capture

```bash
just flash-monitor board=205 port=<path> evidence-dir=docs/parity/evidence/phase-11-safety-controller-hardware-regression-evidence
```

The flash/monitor wrapper records firmware and reference commits, manifest/image paths, command lines, trusted output, observed commits, logs, and conclusion. [VERIFIED: tools/flash/src/main.rs]

### Existing Safety-Critical Parity Guard

```bash
bazel test //tools/parity:tests --test_filter=safety_critical
just parity
```

The parity validator treats rows containing safety-critical concepts such as voltage, fan, thermal, power, self-test hardware, runtime input/display, and ASIC initialization as requiring `hardware-smoke` or `hardware-regression` when marked `verified`. [VERIFIED: tools/parity/src/main.rs] [VERIFIED: Justfile]

### Host Safety Model Regression

```bash
cargo test -p bitaxe-safety --all-features
```

The safety crate contains pure power, thermal/PID, fault, self-test, and watchdog decision code that should remain green around live evidence work. [VERIFIED: crates/bitaxe-safety/src/power.rs] [VERIFIED: crates/bitaxe-safety/src/thermal.rs] [VERIFIED: crates/bitaxe-safety/src/fault.rs] [VERIFIED: crates/bitaxe-safety/src/self_test.rs] [VERIFIED: crates/bitaxe-safety/src/watchdog.rs]

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
| --- | --- | --- | --- |
| Implementation-only parity claims | Hardware-sensitive rows need named evidence tokens and command/log artifacts. | ADR-0012 and current checklist model. [VERIFIED: docs/adr/0012-parity-verification-evidence.md] [VERIFIED: docs/parity/checklist.md] | Planner must not promote safety-critical rows without board-specific evidence. |
| Broad hardware smoke as proof | Component-scoped `hardware-smoke` or `hardware-regression` evidence by exact claim. | Phase 11 decisions. [VERIFIED: .planning/phases/11-safety-controller-hardware-regression-evidence/11-CONTEXT.md] | Planner should create a matrix and avoid one-log overclaims. |
| Direct hardware runs | Detector-gated Ultra 205 autonomous workflow. | Repo-local guidance. [VERIFIED: AGENTS.md] | First live step is `just detect-ultra205`; ambiguous or failed detection blocks live work. |
| Firmware effects enabled before evidence | Observe-only adapters and suppressed unsafe effects by default. | Phase 6 implementation and evidence. [VERIFIED: firmware/bitaxe/src/safety_adapter.rs] [VERIFIED: docs/parity/evidence/phase-06-safety-controllers-and-self-test.md] | Phase 11 can capture safe-unavailable/read-only evidence without enabling dangerous effects. |

**Deprecated/outdated:**

- Treating `verified` as "implemented and unit-tested" is not acceptable for safety-critical hardware rows; hardware evidence is required. [VERIFIED: docs/adr/0012-parity-verification-evidence.md] [VERIFIED: tools/parity/src/main.rs]
- Treating `cargo-espmonitor` as the normal monitor path is not the project stack; use `espflash` through repo wrappers. [VERIFIED: AGENTS.md] [VERIFIED: tools/flash/src/main.rs]
- Treating startup OLED as runtime UI/input parity is explicitly blocked by Phase 6 display/input gap evidence. [VERIFIED: docs/parity/evidence/phase-06-display-input-runtime-gap.md]

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
| --- | --- | --- | --- |
| A1 | Official Analog DS4432U datasheet details should be rechecked before planning any DS4432U register behavior beyond the constants already verified from local Rust and upstream reference files. [ASSUMED] | Hardware Facts To Plan Around | Planner could over-trust repo/reference constants for a bounded voltage-write probe without vendor datasheet confirmation. |

## Open Questions (RESOLVED)

1. **RESOLVED: Is a live Ultra 205 connected and recoverable for Phase 11 execution?**  
   What we know: the repo has a detector gate, and research did not run live hardware detection. [VERIFIED: scripts/detect-ultra205.sh]  
   Resolution: Phase 11 execution starts with `just detect-ultra205`. If it does not find exactly one likely Ultra 205 port or board-info fails, execution records `hardware evidence pending - detector gate did not pass` and does not run flash, monitor, raw hardware, or probe commands. [VERIFIED: AGENTS.md] [VERIFIED: .planning/phases/11-safety-controller-hardware-regression-evidence/11-03-PLAN.md]

2. **RESOLVED: Will Phase 11 implement a minimal probe CLI or stay documentation-led?**  
   What we know: context gives the agent discretion over this choice, and existing Rust host-tool dependencies support a CLI. [VERIFIED: .planning/phases/11-safety-controller-hardware-regression-evidence/11-CONTEXT.md] [VERIFIED: Cargo.toml]  
   Resolution: Phase 11 stays documentation-led plus existing wrapper evidence and parity guard refinement. It does not add `tools/safety-regression`; any future read-only or bounded probe CLI requires a separate plan with explicit inputs, abort conditions, and recovery. [VERIFIED: .planning/phases/11-safety-controller-hardware-regression-evidence/11-01-PLAN.md] [VERIFIED: .planning/phases/11-safety-controller-hardware-regression-evidence/11-02-PLAN.md] [VERIFIED: .planning/phases/11-safety-controller-hardware-regression-evidence/11-03-PLAN.md]

3. **RESOLVED: Which broad checklist rows need splitting before promotion?**  
   What we know: current checklist has safety rows with mixed evidence scopes, and Phase 11 decisions allow splitting exact subclaims. [VERIFIED: docs/parity/checklist.md] [VERIFIED: .planning/phases/11-safety-controller-hardware-regression-evidence/11-CONTEXT.md]  
   Resolution: Phase 11 does not split broad rows by default. It keeps mixed active-control rows below `verified` unless exact required evidence exists, and plan 02 strengthens parity guards so broad smoke evidence cannot verify active voltage, fan actuation, self-test hardware, runtime input, or overheat/fault behavior. [VERIFIED: .planning/phases/11-safety-controller-hardware-regression-evidence/11-02-PLAN.md] [VERIFIED: .planning/phases/11-safety-controller-hardware-regression-evidence/11-03-PLAN.md]

4. **RESOLVED: Can official DS4432U documentation be fetched before active voltage planning?**  
   What we know: local Rust and upstream reference agree on address/register constants. [VERIFIED: firmware/bitaxe/src/safety_adapter/power.rs] [VERIFIED: reference/esp-miner/main/power/DS4432U.c]  
   Resolution: Phase 11 does not plan active DS4432U voltage writes. DS4432U active-write parity remains `hardware evidence pending` until a later plan verifies official documentation or deliberately documents reliance on local/reference evidence with bounded recovery. [VERIFIED: .planning/phases/11-safety-controller-hardware-regression-evidence/11-01-PLAN.md] [VERIFIED: .planning/phases/11-safety-controller-hardware-regression-evidence/11-03-PLAN.md]

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
| --- | --- | --- | --- | --- |
| `just` | Command surface and hardware gates | yes | `1.48.0` | None needed. [VERIFIED: command probe `just --version`] |
| Bazel | Build/test automation | yes | `9.1.1` | None needed. [VERIFIED: command probe `bazel --version`] |
| Cargo | Rust host tests and tools | yes | `1.88.0-nightly` | Use `just doctor` if toolchain drift appears. [VERIFIED: command probe `cargo --version`] |
| Rust `esp` toolchain | ESP-IDF Rust firmware build | yes | `esp` toolchain listed active locally | Use `just bootstrap-esp` if `just doctor` reports setup drift. [VERIFIED: command probe `rustup toolchain list`] |
| `espflash` | Board-info, flash, monitor | yes | `4.0.1` | Blocking for live hardware if missing. [VERIFIED: command probe `espflash --version`] |
| `cargo-espflash` | Developer diagnostics and ESP flash integration | yes | `4.0.1` | Prefer `espflash` wrapper path for evidence. [VERIFIED: command probe `cargo-espflash --version`] |
| `espup` | ESP Rust toolchain installer | yes | `0.15.1` | Project stack notes recommend `0.17.1`; run `just doctor` before live work and use `just bootstrap-esp` for managed install if needed. [VERIFIED: command probe `espup --version`] [VERIFIED: AGENTS.md] |
| `ldproxy` | ESP-IDF Rust linker proxy | yes | `0.3.4` from Cargo install list | Use `just doctor` to validate full firmware setup. [VERIFIED: command probe `cargo install --list`] |
| Ultra 205 board on USB | Live Phase 11 evidence | not probed during research | unknown | First execution task must run `just detect-ultra205`; no live evidence without success. [VERIFIED: AGENTS.md] [VERIFIED: scripts/detect-ultra205.sh] |

**Missing dependencies with no fallback:**

- Live Ultra 205 availability was not probed during research; if `just detect-ultra205` fails during execution, live hardware evidence is blocked and the phase should record pending evidence without overclaiming. [VERIFIED: AGENTS.md]

**Missing dependencies with fallback:**

- `espup` is present at `0.15.1`, while the project stack notes recommend `0.17.1`; `just doctor` and `just bootstrap-esp` are the documented fallback path if this matters during execution. [VERIFIED: AGENTS.md] [VERIFIED: command probe `espup --version`]

## Validation Architecture

### Test Framework

| Property | Value |
| --- | --- |
| Framework | Rust unit/integration tests through Cargo plus Bazel test wrappers. [VERIFIED: Cargo.toml] [VERIFIED: BUILD.bazel] |
| Config file | `Cargo.toml`, `MODULE.bazel`, `BUILD.bazel`, and `Justfile`. [VERIFIED: Cargo.toml] [VERIFIED: MODULE.bazel] [VERIFIED: Justfile] |
| Quick run command | `cargo test -p bitaxe-safety --all-features && bazel test //tools/parity:tests --test_filter=safety_critical && just parity` |
| Full suite command | `just test` plus Rust pre-commit checks if a commit is made. [VERIFIED: Justfile] [VERIFIED: AGENTS.md] |

### Phase Requirements To Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
| --- | --- | --- | --- | --- |
| SAFE-01 | Voltage/power decisions fail closed and hardware evidence distinguishes read-only telemetry from voltage writes. | unit + manual/hardware evidence | `cargo test -p bitaxe-safety --all-features` plus Phase 11 DS4432U/INA260 evidence ledger | yes for unit model; evidence file to create. [VERIFIED: crates/bitaxe-safety/src/power.rs] |
| SAFE-02 | Thermal/fan readings, duty, RPM, and failure reporting are evidenced or held pending. | unit + manual/hardware evidence | `cargo test -p bitaxe-safety --all-features` plus EMC2101/fan evidence pack | yes for unit model; evidence file to create. [VERIFIED: crates/bitaxe-safety/src/thermal.rs] |
| SAFE-03 | PID and thermal-control decisions stay covered before hardware effects are enabled. | unit | `cargo test -p bitaxe-safety --all-features` | yes. [VERIFIED: crates/bitaxe-safety/src/thermal.rs] |
| SAFE-04 | Overheat/fan/power/thermal/ASIC fault paths fail closed and expose status. | unit + guarded hardware/manual if safe | `cargo test -p bitaxe-safety --all-features` plus explicit recovery-gated failure-path evidence if attempted | yes for unit model; hardware evidence conditional. [VERIFIED: crates/bitaxe-safety/src/fault.rs] |
| SAFE-05 | Self-test lifecycle and result reporting are covered or recorded pending. | unit + guarded hardware/manual if safe | `cargo test -p bitaxe-safety --all-features` plus self-test evidence pack if a safe route exists | yes for unit model; hardware evidence conditional. [VERIFIED: crates/bitaxe-safety/src/self_test.rs] |
| SAFE-06 | Display/input admin status is preserved or documented as deferred gap. | manual/hardware evidence + checklist | startup OLED evidence review; runtime gap ledger update; `just parity` | evidence exists from Phase 6; Phase 11 ledger to update. [VERIFIED: docs/parity/evidence/phase-06-display-input-runtime-gap.md] |
| SAFE-07 | Power/current/voltage/fan/temp telemetry is captured where hardware exposes it. | hardware evidence | `just detect-ultra205`; `just flash-monitor board=205 port=<port> evidence-dir=<dir>` plus any safe telemetry route | wrapper exists; Phase 11 evidence to create. [VERIFIED: tools/flash/src/main.rs] |
| SAFE-08 | Safety-critical `verified` rows require hardware evidence. | unit/integration | `bazel test //tools/parity:tests --test_filter=safety_critical`; `cargo test -p bitaxe-parity --all-features`; `just parity` | yes. [VERIFIED: tools/parity/src/main.rs] |
| SAFE-09 | Watchdog/load responsiveness is bounded and observable. | unit + hardware/manual if safe | `cargo test -p bitaxe-safety --all-features` plus bounded liveness evidence if a safe route exists | yes for unit model; live evidence conditional. [VERIFIED: crates/bitaxe-safety/src/watchdog.rs] |
| EVD-05 | Evidence layers include tests, hardware smoke, and hardware regression where appropriate. | integration + artifact review | `just test`; `just parity`; evidence redaction review | test commands exist; evidence file to create. [VERIFIED: Justfile] |

### Sampling Rate

- **Per task commit:** run focused affected tests, at minimum `cargo test -p bitaxe-safety --all-features`, `bazel test //tools/parity:tests --test_filter=safety_critical`, and `just parity` when checklist/evidence semantics change. [VERIFIED: crates/bitaxe-safety/src] [VERIFIED: tools/parity/src/main.rs] [VERIFIED: Justfile]
- **Per wave merge:** run `just test` and `just parity`; if source code is committed, run the Rust pre-commit sequence from `AGENTS.md`. [VERIFIED: AGENTS.md] [VERIFIED: Justfile]
- **Phase gate:** hardware evidence ledger complete or explicitly pending for each surface, detector output present for live runs, redaction review recorded, and `just parity` rejects overclaims. [VERIFIED: AGENTS.md] [VERIFIED: tools/parity/src/main.rs]

### Wave 0 Gaps

- [ ] `docs/parity/evidence/phase-11-safety-controller-hardware-regression-evidence.md` - required ledger/runbook/matrix for SAFE-01 through SAFE-09 and EVD-05. [VERIFIED: .planning/phases/11-safety-controller-hardware-regression-evidence/11-CONTEXT.md]
- [ ] `docs/parity/evidence/phase-11-safety-controller-hardware-regression-evidence/` - optional artifact directory if live wrapper or scripted probes produce JSON/log files. [VERIFIED: tools/flash/src/main.rs]
- [ ] `tools/safety-regression/` - create only if the planner chooses scripted bounded probes over documentation-led evidence. [VERIFIED: .planning/phases/11-safety-controller-hardware-regression-evidence/11-CONTEXT.md]
- [ ] Official DS4432U datasheet verification - required before planning active voltage-register behavior beyond repo/reference constants. [ASSUMED]

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
| --- | --- | --- |
| V2 Authentication | no new authentication expected | Do not add credential-bearing probes; if HTTP/API probing is used, keep it local and redact credentials/endpoints from evidence. [VERIFIED: AGENTS.md] |
| V3 Session Management | no new session management expected | Do not introduce session state in hardware evidence helpers. [VERIFIED: .planning/phases/11-safety-controller-hardware-regression-evidence/11-CONTEXT.md] |
| V4 Access Control | limited to hardware-run gating | Enforce board `205`, detector success, explicit probe selection, and recovery prerequisites before live hardware actions. [VERIFIED: AGENTS.md] [VERIFIED: scripts/detect-ultra205.sh] |
| V5 Input Validation | yes | Parse board, port, evidence path, probe name, actuation opt-ins, and stop conditions at CLI boundaries with typed values if new tooling is added. [VERIFIED: standards/core/architecture.md] [VERIFIED: Cargo.toml] |
| V6 Cryptography | no new cryptography expected | Do not add custom crypto; rely on existing toolchains and avoid secrets in evidence. [VERIFIED: AGENTS.md] |
| V9 Communications | possible if API/WebSocket telemetry is used | Keep evidence to local/bench routes and redact private endpoints. [VERIFIED: AGENTS.md] |
| V12 Files and Resources | yes for evidence artifacts | Write evidence only under planned paths, record redaction review, and avoid generated secret content. [VERIFIED: AGENTS.md] |

### Known Threat Patterns for Phase 11

| Pattern | STRIDE | Standard Mitigation |
| --- | --- | --- |
| Wrong USB serial target or wrong board | Spoofing / Tampering | `just detect-ultra205`, exact board-info output, board `205` in every artifact. [VERIFIED: AGENTS.md] [VERIFIED: scripts/detect-ultra205.sh] |
| Safety overclaim in checklist | Tampering | `tools/parity` rejection for safety-critical `verified` rows without `hardware-smoke` or `hardware-regression`. [VERIFIED: tools/parity/src/main.rs] |
| Hardware damage from voltage, fan, overheat, erase, or fault injection | Denial of service / Safety | Written recovery path, explicit allowed command set, bounded probes, stop conditions, and pending evidence when prerequisites are missing. [VERIFIED: AGENTS.md] [VERIFIED: .planning/phases/11-safety-controller-hardware-regression-evidence/11-CONTEXT.md] |
| Secret leakage in logs or evidence | Information disclosure | Redaction review and ban on committing pool credentials, Wi-Fi credentials, private endpoints, NVS secrets, or similar values. [VERIFIED: AGENTS.md] |
| Non-repeatable manual observation | Repudiation | Record exact command/probe, board, port, commits, artifact paths, logs, observed behavior, conclusion, and observer-owned notes. [VERIFIED: AGENTS.md] |

## Sources

### Primary (HIGH confidence)

- `.planning/phases/11-safety-controller-hardware-regression-evidence/11-CONTEXT.md` - phase decisions, boundary, discretion, and deferred scope.
- `.planning/REQUIREMENTS.md` - SAFE-01 through SAFE-09 and EVD-05 requirement text and traceability.
- `.planning/STATE.md` - current project position and recent phase state.
- `.planning/config.json` - `workflow.nyquist_validation` is explicitly `true`.
- `AGENTS.md`, `AGENTS.bright-builds.md`, `standards-overrides.md`, `standards/core/architecture.md`, `standards/core/code-shape.md`, `standards/core/verification.md`, `standards/core/testing.md`, `standards/languages/rust.md` - repo and Bright Builds constraints.
- `.planning/ROADMAP.md`, `.planning/PROJECT.md`, `.planning/v1.0-MILESTONE-AUDIT.md` - Phase 11 milestone context and safety evidence gap.
- `docs/adr/0012-parity-verification-evidence.md`, `docs/adr/0014-pivot-to-ultra-205-bm1366-first-parity.md`, `PROVENANCE.md` - evidence, target, and licensing/provenance policy.
- `docs/parity/checklist.md`, `tools/parity/src/main.rs`, `docs/parity/evidence/phase-06-safety-controllers-and-self-test.md`, `docs/parity/evidence/phase-06-ultra-205-safety-hardware-smoke.md`, `docs/parity/evidence/phase-06-display-input-runtime-gap.md`, `docs/parity/evidence/phase-09-flash-monitor-evidence-wrapper-hardening.md` - current parity and evidence state.
- `scripts/detect-ultra205.sh`, `tools/flash/src/main.rs`, `Justfile` - detector and evidence command surface.
- `crates/bitaxe-safety/src/power.rs`, `thermal.rs`, `fault.rs`, `self_test.rs`, `watchdog.rs`; `firmware/bitaxe/src/safety_adapter.rs`, `safety_adapter/power.rs`, `safety_adapter/thermal.rs`, `safety_adapter/watchdog.rs`, `runtime_snapshot.rs`, `display_adapter.rs` - implementation seams.
- `reference/esp-miner/config-205.cvs`, `main/device_config.h`, `main/power/DS4432U.c`, `main/power/INA260.c`, `main/thermal/thermal.c`, `main/thermal/EMC2101.c`, `main/tasks/fan_controller_task.c`, `main/tasks/power_management_task.c`, `main/self_test/self_test.c`, `main/input.c`, `main/screen.c` - pinned upstream reference behavior.
- `https://www.ti.com/lit/ds/symlink/ina260.pdf` - INA260 official datasheet for register and LSB facts.
- `https://ww1.microchip.com/downloads/aemDocuments/documents/MSLD/ProductDocuments/DataSheets/EMC2101-Data-Sheet-DS20006703.pdf` - EMC2101 official datasheet for fan/TACH/temperature facts.

### Secondary (MEDIUM confidence)

- Local command probes for `just`, `bazel`, `cargo`, `rustup`, `espflash`, `cargo-espflash`, `espup`, and `ldproxy` availability. These are current for the research machine on 2026-06-29. [VERIFIED: command probes]

### Tertiary (LOW confidence)

- Analog DS4432U official datasheet URL was attempted but not successfully fetched during research; DS4432U claims in this file are limited to local Rust constants and pinned upstream reference behavior. [ASSUMED]

## Metadata

**Confidence breakdown:**

- Standard stack: HIGH - versions and commands were verified from local manifests and command probes. [VERIFIED: Cargo.toml] [VERIFIED: MODULE.bazel] [VERIFIED: command probes]
- Architecture: HIGH - project already has explicit functional-core/imperative-shell boundaries, detector gate, wrapper evidence, and parity guard. [VERIFIED: AGENTS.md] [VERIFIED: tools/flash/src/main.rs] [VERIFIED: tools/parity/src/main.rs]
- Hardware component facts: MEDIUM-HIGH - INA260 and EMC2101 vendor facts were cited from official PDFs; DS4432U external datasheet verification remains open. [CITED: https://www.ti.com/lit/ds/symlink/ina260.pdf] [CITED: https://ww1.microchip.com/downloads/aemDocuments/documents/MSLD/ProductDocuments/DataSheets/EMC2101-Data-Sheet-DS20006703.pdf] [ASSUMED]
- Pitfalls: HIGH - pitfalls come from explicit phase decisions, repo hardware rules, current code boundaries, and existing Phase 6 evidence gaps. [VERIFIED: .planning/phases/11-safety-controller-hardware-regression-evidence/11-CONTEXT.md] [VERIFIED: AGENTS.md] [VERIFIED: docs/parity/evidence/phase-06-display-input-runtime-gap.md]

**Research date:** 2026-06-29  
**Valid until:** 2026-07-06 for tool/hardware availability and live docs; 2026-07-29 for local architecture and phase decisions unless the planning artifacts change.

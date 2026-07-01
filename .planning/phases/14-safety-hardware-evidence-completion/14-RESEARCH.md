# Phase 14: Safety Hardware Evidence Completion - Research

**Researched:** 2026-06-30
**Domain:** Ultra 205 safety hardware evidence, allow-gated probes, parity claim governance
**Confidence:** HIGH

<user_constraints>
## User Constraints (from CONTEXT.md)

Source for this entire user-constraints block: `.planning/phases/14-safety-hardware-evidence-completion/14-CONTEXT.md` [VERIFIED: `.planning/phases/14-safety-hardware-evidence-completion/14-CONTEXT.md`]

### Locked Decisions

## Implementation Decisions

### Recovery And Allow Gates

- **D-01:** Use a machine-enforced preflight allow manifest as the minimum gate before any active hardware verification can run. The manifest should bind `board=205`, the selected detector `port=<path>`, `espflash board-info --chip esp32s3 --port <port> --non-interactive`, package manifest identity, source commit, reference commit, allowed surface, allowed command or probe, input bounds, abort conditions, recovery steps, post-action safe-state checks, evidence destination, and redaction reviewer.
- **D-02:** Continue to start all live Ultra 205 work with `just detect-ultra205`. The run may continue only when exactly one likely ESP USB serial port is found and board-info succeeds for that port. Zero ports, multiple ports, board-info failure, a target other than board `205`, stale package identity, missing recovery instructions, or redaction uncertainty must stop the run and produce pending evidence instead of a workaround.
- **D-03:** Add surface-scoped bounded probe wrappers only where Phase 14 needs active `hardware-regression` claims. Wrappers should be per-surface rather than one broad safety script, so voltage, fan, thermal, self-test, watchdog/load, display/input, and live telemetry each define their own limits, stop conditions, recovery, logs, and conclusions.
- **D-04:** Treat watchdog panic, reset, bootloader hold, detector failure, board-info failure, missing safe-state marker, or unavailable recovery package as stop/recovery signals, not as normal passing output. A probe that cannot prove post-action safe state must not promote any active safety-control claim.

### Active Safety Telemetry And Control Evidence

- **D-05:** Keep Phase 14 evidence component-scoped with claim tiers. Use separate packs for safe baseline, power/current telemetry, voltage control, thermal/fan, self-test/watchdog/load, display/input, live API/WebSocket telemetry, and parity/redaction. Each pack should state whether it proves read-only observation, bounded actuation, API/WebSocket projection, safe-unavailable status, or unsupported/pending claims.
- **D-06:** Do not conflate read-only sensor observations with actuator behavior. INA260 current/voltage/power freshness, thermal readings, fan RPM observations, and live API/WebSocket projected values can support narrow board-205 claims only for the exact observed data. DS4432U voltage writes, fan duty effects, ASIC reset/power sequencing, overheat/fault behavior, and self-test hardware submodes require bounded `hardware-regression`.
- **D-07:** Procedure-scoped manifests are required for any probe that actuates hardware or proves live cadence under load. The manifest should capture exact command, allowed inputs, stimulus, expected markers, failure markers, timeout, recovery command, safe-state markers, raw artifact paths, redaction status, and the exact checklist row or subclaim it can support.
- **D-08:** If bounded actuation, sensor feedback, live API/WebSocket access, recovery, or redaction prerequisites are missing, record conservative observe-only or pending evidence. Observe-only evidence may prove safe-unavailable or read-only observations, but it must not verify active voltage, fan duty, overheat/fault, self-test hardware, runtime input/display, watchdog/load stress, or recovery parity.

### Self-Test, Watchdog/Load, And Runtime Display/Input

- **D-09:** Run self-test, watchdog/load, and display/input checks only where a safe firmware route, API/log/WebSocket marker, serial marker, or physical stimulus exists and the plan documents the stimulus and recovery path. Do not add temporary diagnostic firmware routes unless they are compile-gated or otherwise impossible to expose accidentally in production firmware.
- **D-10:** Keep self-test hardware submodes below `verified` unless the run proves the exact submode safely, including any voltage, fan, ASIC work, fake work, pass/fail/cancel, and production-mining gate behavior it touches. Pure self-test unit tests and boot logs are not hardware proof.
- **D-11:** Watchdog/load evidence must prove observable liveness or responsiveness under a bounded documented workload. A supervisor startup/yield log is useful evidence for the supervisor shell, but it does not verify load stress, blocked task behavior, or watchdog recovery without a safe stimulus and pass/fail criteria.
- **D-12:** Runtime display/input claims remain below `verified` unless a real runtime route is exercised and physically or log/API/WebSocket-observed. Startup-only SSD1306 evidence remains a breadcrumb and may support startup display only; it cannot verify runtime display pages, screen flow, LVGL parity, or input hardware behavior.

### Checklist Promotion, Redaction, And Final Verification

- **D-13:** Use exact-claim promotion. Promote only rows whose evidence class matches the exact claim. Active safety-control rows need `hardware-regression`; narrow read-only or safe-unavailable observations can use `hardware-smoke` only when board `205`, port, source commit, reference commit, command/log, conclusion, and redaction review are present.
- **D-14:** Preserve existing checklist row IDs unless a broad row prevents truthful documentation of a narrow verified subclaim. Prefer precise checklist notes and evidence links over large row-model churn. Split rows only when the plan proves that exact subclaims cannot otherwise be represented safely.
- **D-15:** Rows with missing stimulus, missing recovery, failed detector gate, unavailable `DEVICE_URL`, unavailable hardware route, stale package identity, or redaction uncertainty must stay `implemented`, `in-progress`, `deferred`, or pending with owner/follow-up. Do not reuse Phase 11 or Phase 13 evidence to verify fresh Phase 14 active claims unless the row clearly names the older evidence as a narrow historical subclaim.
- **D-16:** Final commit/push must be gated by redaction review, `just parity` with no validation errors, relevant Rust checks for changed code, `just test`, `just verify-reference`, diff review, clean GSD verification status `passed`, and lifecycle validation for this phase attempt.

### the agent's Discretion

The agent may choose the exact plan count, evidence directory layout, allow-manifest schema, probe command names, JSON field names, and whether the manifest validation lives in `tools/parity`, a repo-owned script, or a small Rust host tool. Those choices must preserve repo-owned ESP/esp-rs tooling, keep `reference/esp-miner` read-only, keep active hardware use phase-gated, avoid secrets in evidence, keep functional core plus imperative shell, and avoid standalone body `---` separators in GSD artifacts.

### Deferred Ideas (OUT OF SCOPE)

- Trusted BM1366 chip-detect, work-send/result-receive, and controlled mining smoke/soak belong to Phase 15.
- Same-commit package, flash, serial boot, live HTTP/static/recovery/OTA, rollback, erase, failed-update, and interrupted-update evidence belongs to Phase 16.
- Non-205 boards, TPS546 active behavior, BM1370/BM1368/BM1397, all-board factory images, Stratum v2, BAP, Angular AxeOS replacement, and production mining performance tuning remain deferred.
- Full LVGL runtime display carousel, display config, timeout, rotation, inversion, and broad button routing remain outside Phase 14 unless a plan proves a safe bounded route and physical evidence path.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
| --- | --- | --- |
| SAFE-01 | Ultra 205 voltage and power-control surfaces use bounded typed decisions and fail closed on invalid configuration, communication failure, or unsafe readings. | Use typed safety-core decisions plus a Phase 14 allow manifest before any active DS4432U or power-control probe. [VERIFIED: `.planning/REQUIREMENTS.md`; `crates/bitaxe-safety/src/power.rs`; `14-CONTEXT.md`] |
| SAFE-02 | Thermal sensor and fan control surfaces expose upstream-compatible readings, fan duty behavior, RPM behavior, and failure reporting. | Keep read-only thermal/RPM observations separate from fan duty actuation; fan duty requires bounded `hardware-regression`. [VERIFIED: `.planning/REQUIREMENTS.md`; `reference/esp-miner/main/tasks/fan_controller_task.c`; `docs/parity/checklist.md`] |
| SAFE-03 | PID and thermal-control decisions are covered by pure unit tests before hardware effects are enabled. | Existing pure PID/fan logic tests are the unit layer; they cannot verify fan hardware. [VERIFIED: `.planning/REQUIREMENTS.md`; `crates/bitaxe-safety/src/thermal.rs`; `docs/parity/evidence/phase-06-safety-controllers-and-self-test.md`] |
| SAFE-04 | Overheat, fan, power, thermal, and ASIC fault paths enter safe states and expose user-visible status compatible with upstream behavior. | Overheat/fault proof needs a bounded stimulus/recovery manifest; absent that, keep fault paths pending and record safe-unavailable state only. [VERIFIED: `.planning/REQUIREMENTS.md`; `reference/esp-miner/main/tasks/power_management_task.c`; `14-CONTEXT.md`] |
| SAFE-05 | Self-test lifecycle behavior covers factory flags, start, pass, fail, restart, cancel, and user-visible result reporting. | Pure lifecycle exists, but hardware submodes remain below verified unless a safe route proves voltage, fan, ASIC, pass/fail/cancel, and production gate behavior. [VERIFIED: `.planning/REQUIREMENTS.md`; `crates/bitaxe-safety/src/self_test.rs`; `14-CONTEXT.md`] |
| SAFE-06 | Display and input status surfaces needed for normal Ultra 205 administration are preserved or explicitly documented as deferred gaps. | Startup SSD1306 evidence is already narrow; runtime display/input needs a real runtime route plus physical/log/API/WebSocket observation or stays pending. [VERIFIED: `.planning/REQUIREMENTS.md`; `firmware/bitaxe/src/display_adapter.rs`; `docs/parity/evidence/phase-06-display-input-runtime-gap.md`] |
| SAFE-07 | Power, current, voltage, fan, and temperature telemetry are captured where Ultra 205 hardware exposes them. | Phase 14 should create packs for INA260 power/current/voltage, thermal/fan, API, and WebSocket telemetry; each pack records freshness or safe-unavailable status. [VERIFIED: `.planning/REQUIREMENTS.md`; `docs/parity/evidence/phase-11-safety-controller-hardware-regression-evidence.md`] |
| SAFE-08 | Safety-critical surfaces cannot be marked `verified` without `hardware-smoke` or `hardware-regression` evidence. | `tools/parity` already rejects safety-critical verified rows without hardware evidence and active safety-control rows without `hardware-regression`. [VERIFIED: `.planning/REQUIREMENTS.md`; `tools/parity/src/main.rs`] |
| SAFE-09 | Mining, control, API, and telemetry tasks avoid watchdog starvation and preserve observable responsiveness under load. | Existing supervisor startup/yield logs prove only shell startup; load responsiveness needs bounded workload, liveness markers, timeout, and post-action safe-state proof. [VERIFIED: `.planning/REQUIREMENTS.md`; `firmware/bitaxe/src/safety_adapter/watchdog.rs`; `14-CONTEXT.md`] |
| EVD-05 | Verification layers include unit tests, golden fixtures, API comparison, hardware smoke tests, and hardware regression or soak evidence where appropriate. | Use unit/workflow/API/hardware-smoke/hardware-regression tiers and exact-claim promotion; blocker evidence must not promote checklist rows. [VERIFIED: `.planning/REQUIREMENTS.md`; `docs/parity/checklist.md`; `14-CONTEXT.md`] |
</phase_requirements>

## Summary

Phase 14 should be planned as an evidence-governance and gated hardware-regression phase, not as a broad safety-feature buildout. The current firmware safety adapters are observe-only and suppress voltage/fan effects by default, while `tools/parity` already blocks unsafe checklist promotion. [VERIFIED: `firmware/bitaxe/src/safety_adapter.rs`; `firmware/bitaxe/src/safety_adapter/power.rs`; `firmware/bitaxe/src/safety_adapter/thermal.rs`; `tools/parity/src/main.rs`]

The primary implementation need is a machine-validated allow manifest plus per-surface probe wrappers. The manifest should run before any active hardware action and should produce pending evidence instead of continuing when board, package, recovery, input bounds, expected markers, or redaction gates are incomplete. [VERIFIED: `14-CONTEXT.md`; `scripts/phase13-recovery-regression.sh`; `docs/parity/evidence/phase-13-final-ultra-205-release-evidence/recovery-runbook.md`]

**Primary recommendation:** implement `tools/parity` allow-manifest validation in a new focused module, add Phase 14 component-pack scaffolding, then add only the per-surface wrappers whose recovery and safe-state checks can be proven; otherwise record exact pending outcomes. [VERIFIED: `14-CONTEXT.md`; `tools/parity/src/main.rs`; `standards/core/architecture.md`; `standards/languages/rust.md`]

## Project Constraints (from AGENTS.md)

- Read `AGENTS.md`, `AGENTS.bright-builds.md`, `standards-overrides.md` when present, and relevant managed standards before planning or implementation. [VERIFIED: `AGENTS.md`; `AGENTS.bright-builds.md`; `standards/index.md`]
- Use ESP-IDF Rust tooling, pinned `esp-idf-sys` metadata, `.cargo/config.toml`, `espup`, `ldproxy`, and `espflash` before custom CMake, PlatformIO, or unmanaged ESP-IDF installs. [VERIFIED: `AGENTS.md`; `.cargo/config.toml`; `firmware/bitaxe/Cargo.toml`]
- Treat `.embuild/` as generated local ESP-IDF/esp-rs state; do not commit or hand-edit it. [VERIFIED: `AGENTS.md`]
- Before autonomous Ultra 205 hardware use, run `just detect-ultra205`; continue only when exactly one likely port is found and `espflash board-info --chip esp32s3 --port <port> --non-interactive` succeeds. [VERIFIED: `AGENTS.md`; `scripts/detect-ultra205.sh`]
- Stop and record pending evidence when there are zero or multiple ports, board-info failure, non-205 target, or missing recovery/evidence instructions. [VERIFIED: `AGENTS.md`; `scripts/detect-ultra205.sh`; `14-CONTEXT.md`]
- Do not run ad hoc erase, rollback, interrupted-update, voltage/fan/mining stress, raw write, or fault-injection commands outside documented phase-gated procedures. [VERIFIED: `AGENTS.md`; `14-CONTEXT.md`]
- Every hardware run must record board `205`, selected port, source commit, reference commit, package manifest/artifacts when applicable, exact commands, board-info output, captured logs, observed behavior, conclusion, and redaction review. [VERIFIED: `AGENTS.md`; `docs/parity/evidence/phase-11-safety-controller-hardware-regression-evidence.md`]
- Do not commit Wi-Fi credentials, pool credentials, private endpoints, NVS secret values, API tokens, or raw terminal secrets in evidence. [VERIFIED: `AGENTS.md`; `docs/parity/evidence/phase-13-final-ultra-205-release-evidence/redaction-review.md`]
- Keep `reference/esp-miner` read-only as behavioral evidence. [VERIFIED: `AGENTS.md`; `docs/adr/0005-read-only-reference-implementation.md`; `PROVENANCE.md`]
- Keep functional core and imperative shell: pure safety decisions in crates, hardware effects in firmware adapters/scripts. [VERIFIED: `AGENTS.md`; `standards/core/architecture.md`; `standards/languages/rust.md`; `crates/bitaxe-safety/src/power.rs`; `firmware/bitaxe/src/safety_adapter.rs`]
- Avoid standalone body `---` separators in GSD Markdown artifacts. [VERIFIED: `AGENTS.md`; `tasks/lessons.md`]
- Before committing in this Rust repo, run `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, and `cargo test --all-features`. [VERIFIED: `AGENTS.md`]

## Standard Stack

### Core

| Library / Tool | Version | Purpose | Why Standard |
| --- | --- | --- | --- |
| Rust workspace crates | local `0.1.0`, edition `2021` | Pure safety, API telemetry, host tools, and firmware integration | The workspace already contains `bitaxe-safety`, `bitaxe-api`, `bitaxe-flash`, `bitaxe-parity`, and firmware packages. [VERIFIED: `Cargo.toml`; `cargo metadata --no-deps`] |
| ESP-IDF Rust stack | ESP-IDF `tag:v5.5.4`, `esp-idf-svc 0.52.1`, `esp-idf-sys 0.37.2` | Firmware adapters, HTTP/WebSocket, tasks, I2C, OTA, logging | Project instructions and Cargo metadata pin this stack for Ultra 205 firmware. [VERIFIED: `firmware/bitaxe/Cargo.toml`; `.cargo/config.toml`; `Cargo.toml`] |
| Bazel / Bzlmod | Bazel `9.1.1`, `rules_rust 0.70.0`, `rules_shell 0.8.0` | Canonical build/test/package graph | `Justfile` routes build/test/package/parity through Bazel and `MODULE.bazel` pins rules. [VERIFIED: env audit; `Justfile`; `MODULE.bazel`] |
| `just` | `1.48.0` | Human command surface | Repo-local commands expose `detect-ultra205`, `build`, `test`, `package`, `flash`, `monitor`, `flash-monitor`, `verify-reference`, and `parity`. [VERIFIED: env audit; `Justfile`] |
| `espflash` | `4.0.1` | Port listing, board-info, flash, monitor, image operations | Existing detector and flash wrappers use `list-ports`, `board-info`, `write-bin`, and `monitor --non-interactive`; local CLI help confirms these commands/options. [VERIFIED: env audit; `scripts/detect-ultra205.sh`; `tools/flash/src/main.rs`; `espflash --help`] |
| `tools/parity` | local `bitaxe-parity 0.1.0` | Checklist validation, release gate, allow-manifest validation target | It already owns verified-row semantics; add Phase 14 manifest validation as a focused module rather than growing ad hoc scripts. [VERIFIED: `tools/parity/src/main.rs`; `tools/parity/BUILD.bazel`; `wc -l tools/parity/src/main.rs`] |

### Supporting

| Library / Tool | Version | Purpose | When to Use |
| --- | --- | --- | --- |
| `serde` / `serde_json` | `serde 1.0.228`, `serde_json 1.0.150` | Typed allow-manifest and evidence JSON parsing | Use for manifest validation and machine-readable probe outputs. [VERIFIED: `Cargo.toml`; `cargo metadata --no-deps`] |
| `clap` | `4.6.1` | Host CLI parsing | Use for `tools/parity` subcommands if adding manifest validation there. [VERIFIED: `Cargo.toml`; `tools/parity/Cargo.toml`] |
| `camino` | `1.2.3` | UTF-8 path handling in host tools | Use for evidence and manifest paths in Rust host tools. [VERIFIED: `Cargo.toml`; `tools/parity/Cargo.toml`; `tools/flash/Cargo.toml`] |
| `curl` | `8.7.1` | HTTP smoke requests when `DEVICE_URL` is explicit | Use only after `DEVICE_URL` is provided and redaction rules are defined. [VERIFIED: env audit; `scripts/phase13-http-static-smoke.sh`] |
| `websocat` | missing | WebSocket frame capture | Do not assume frame-level WebSocket evidence can run locally; either add an explicit dependency task or record WebSocket frame proof as pending. [VERIFIED: env audit; Python module probe] |
| `python3` | `3.14.4` | Manifest/body inspection and small validation scripts | Existing scripts use Python for JSON manifest field extraction; it is available. [VERIFIED: env audit; `scripts/phase13-http-static-smoke.sh`] |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
| --- | --- | --- |
| `tools/parity` typed manifest validation | A pure shell validator | Shell is adequate for wrappers, but typed Rust validation better encodes board/port/claim/input invariants and tests invalid manifests. [VERIFIED: `standards/core/architecture.md`; `standards/languages/rust.md`; `tools/parity/BUILD.bazel`] |
| Per-surface probe wrappers | One broad `phase14-safety.sh` | The phase decision explicitly requires surface-scoped wrappers so each surface owns limits, stop conditions, recovery, logs, and conclusions. [VERIFIED: `14-CONTEXT.md`] |
| `websocat` for WebSocket frames | Custom Python socket framing | Do not hand-roll WebSocket framing for evidence; local WebSocket clients are absent, so frame-level proof needs an explicit dependency or pending outcome. [VERIFIED: env audit; `14-CONTEXT.md`; `standards/core/verification.md`] |

**Installation:**

```bash
# No default new dependency is required for allow-manifest validation, safe-baseline evidence, or checklist governance.
# If the plan chooses frame-level WebSocket evidence, add an explicit dependency step for a maintained WebSocket client before using it.
```

**Version verification:** local versions were verified with `just --version`, `bazel --version`, `cargo --version`, `rustc --version`, `espflash --version`, `espup --version`, `python3 --version`, `curl --version`, `jq --version`, and `cargo metadata --no-deps`. [VERIFIED: env audit; `cargo metadata --no-deps`]

## Architecture Patterns

### Recommended Project Structure

```text
tools/parity/src/
  safety_allow.rs          # Typed Phase 14 allow-manifest parser and validation rules
  main.rs                  # Thin CLI dispatch to safety_allow
scripts/
  phase14-safe-baseline.sh # Optional serial/API safe baseline wrapper
  phase14-power-telemetry.sh
  phase14-thermal-fan.sh
  phase14-live-telemetry.sh
  phase14-*-test.sh        # Shell tests for each wrapper
docs/parity/evidence/phase-14-safety-hardware-evidence-completion/
  README.md
  safe-baseline/
  power-telemetry/
  voltage-control/
  thermal-fan/
  self-test-watchdog-load/
  display-input/
  live-api-websocket-telemetry/
  parity-redaction/
  redaction-review.md
```

This structure follows the existing Phase 13 shell-wrapper pattern and the Phase 11 component-pack evidence pattern. [VERIFIED: `scripts/BUILD.bazel`; `docs/parity/evidence/phase-11-safety-controller-hardware-regression-evidence/README.md`; `docs/parity/evidence/phase-13-final-ultra-205-release-evidence/README.md`]

### Pattern 1: Typed Allow Manifest Before Active Probe

**What:** Parse a JSON manifest into domain types that require board `205`, selected port, board-info command/result, package manifest identity, source/reference commit, surface, claim type, allowed command, bounded inputs, abort conditions, recovery commands, safe-state markers, evidence paths, redaction reviewer, and target checklist rows. [VERIFIED: `14-CONTEXT.md`; `standards/core/architecture.md`; `standards/languages/rust.md`]

**When to use:** Require it before any voltage, fan, thermal/fault, self-test hardware, load, runtime display/input, or live telemetry probe that could support `hardware-regression`. [VERIFIED: `14-CONTEXT.md`; `docs/parity/evidence/phase-11-safety-controller-hardware-regression-evidence.md`]

**Example:**

```json
{
  "board": "205",
  "port": "/dev/cu.usbmodem1101",
  "board_info_command": "espflash board-info --chip esp32s3 --port /dev/cu.usbmodem1101 --non-interactive",
  "package_manifest": "bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json",
  "source_commit": "190849539700b8f9a7909fd2b6ebd84142557968",
  "reference_commit": "c1915b0a63bfabebdb95a515cedfee05146c1d50",
  "surface": "voltage-control",
  "claim_type": "bounded-actuation",
  "allowed_command": "scripts/phase14-voltage-control.sh --manifest allow.json",
  "allowed_inputs": { "setpoint_mv": [1200] },
  "abort_conditions": ["detector_mismatch", "board_info_failure", "missing_safe_state_marker"],
  "recovery_steps": ["just flash board=205 port=/dev/cu.usbmodem1101 image=bazel-bin/firmware/bitaxe/bitaxe-ultra205-factory.bin manifest=bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json"],
  "post_action_safe_state_markers": ["safe_state: mining=disabled", "hardware_control=disabled"],
  "evidence_dir": "docs/parity/evidence/phase-14-safety-hardware-evidence-completion/voltage-control",
  "redaction_reviewer": "required-before-citation",
  "checklist_rows": ["PWR-003", "PWR-005"]
}
```

This manifest shape is derived from locked Phase 14 decisions and prior Phase 13 recovery gates. [VERIFIED: `14-CONTEXT.md`; `docs/parity/evidence/phase-13-final-ultra-205-release-evidence/recovery-runbook.md`]

### Pattern 2: Conservative Pending Output

**What:** Every wrapper must write a human-readable and machine-readable blocked/pending result when a gate is missing; it must not skip silently or infer success. [VERIFIED: `scripts/phase13-http-static-smoke.sh`; `scripts/phase13-recovery-regression.sh`; `14-CONTEXT.md`]

**When to use:** Use for missing `DEVICE_URL`, missing detector result, missing allow manifest, stale package identity, missing recovery command, missing client dependency, unavailable route, or redaction uncertainty. [VERIFIED: `14-CONTEXT.md`; `docs/parity/evidence/phase-13-final-ultra-205-release-evidence.md`]

### Pattern 3: Exact-Claim Checklist Promotion

**What:** Promote only the exact subclaim whose evidence matches the row, and leave broad rows below verified when active subclaims remain pending. [VERIFIED: `14-CONTEXT.md`; `docs/parity/evidence/phase-11-safety-controller-hardware-regression-evidence.md`; `docs/parity/checklist.md`]

**When to use:** Use for all PWR, THR, IO, UI, SELF, API, STAT, and evidence-governance updates. [VERIFIED: `docs/parity/checklist.md`; `tools/parity/src/main.rs`]

### Anti-Patterns to Avoid

- **Broad boot smoke as active-control proof:** boot logs can prove safe-state and markers only; they do not prove DS4432U writes, fan duty, overheat/fault handling, self-test hardware, runtime input/display, or load stress. [VERIFIED: `docs/parity/evidence/phase-11-safety-controller-hardware-regression-evidence.md`]
- **Raw `espflash monitor` fallback for promotion:** wrapper-owned evidence is the trusted serial path; diagnostic monitor output cannot replace generated JSON plus trust classification. [VERIFIED: `docs/parity/evidence/phase-09-flash-monitor-evidence-wrapper-hardening.md`; `tools/flash/src/main.rs`]
- **Network target inference:** Phase 13 explicitly blocked live HTTP/OTA evidence when `DEVICE_URL` was missing and did not scan or infer a target. [VERIFIED: `docs/parity/evidence/phase-13-final-ultra-205-release-evidence.md`; `scripts/phase13-http-static-smoke.sh`]
- **Growing large host files:** `tools/parity/src/main.rs` is 1535 lines, `tools/flash/src/main.rs` is 2118 lines, and `firmware/bitaxe/src/http_api.rs` is 954 lines; add focused modules or scripts instead of expanding already-large files. [VERIFIED: `wc -l`; `standards/languages/rust.md`; `AGENTS.bright-builds.md`]

## Do Not Hand-Roll

| Problem | Don't Build | Use Instead | Why |
| --- | --- | --- | --- |
| Verified-row semantics | Markdown string edits that bypass guards | `tools/parity` validation | `tools/parity` already rejects pending, safety-critical, active-control, live ASIC/mining, release, and deferred-scope overclaims. [VERIFIED: `tools/parity/src/main.rs`] |
| Active hardware authorization | Ad hoc shell flags only | Typed allow manifest plus per-surface wrapper | Phase 14 locked decisions require manifest-bound board, port, package, inputs, aborts, recovery, safe-state, evidence, and redaction. [VERIFIED: `14-CONTEXT.md`] |
| WebSocket frame protocol | Custom socket/framing code | A maintained client dependency or pending evidence | No `websocat` or Python WebSocket client is installed; hand-rolled framing would add protocol risk to evidence. [VERIFIED: env audit; Python module probe] |
| JSON/package parsing | `grep`/`sed` over JSON | `serde_json` in Rust or Python JSON parser in scripts | Existing scripts use JSON parsing for manifest fields and host tools already depend on `serde_json`. [VERIFIED: `scripts/phase13-http-static-smoke.sh`; `Cargo.toml`] |
| DS4432U/fan/raw I2C actuation | Raw I2C shell writes | Repo-owned firmware/probe path behind allow manifest | Ultra 205 DS4432U/fan behavior touches hardware-control surfaces requiring bounded recovery and post-action safe-state proof. [VERIFIED: `reference/esp-miner/main/power/DS4432U.c`; `reference/esp-miner/main/tasks/fan_controller_task.c`; `14-CONTEXT.md`] |
| Redaction assurance | Blind template completion | Artifact-specific redaction review | Phase 11 and Phase 13 reviews inspect generated logs, JSON, API output, WebSocket frames, and terminal text before citation. [VERIFIED: `docs/parity/evidence/phase-11-safety-controller-hardware-regression-evidence/redaction-review.md`; `docs/parity/evidence/phase-13-final-ultra-205-release-evidence/redaction-review.md`] |

**Key insight:** Phase 14 risk is not missing code alone; the bigger risk is an irreversible or unsafe evidence action being interpreted as a passing parity claim. [VERIFIED: `14-CONTEXT.md`; `docs/parity/evidence/phase-11-safety-controller-hardware-regression-evidence.md`]

## Common Pitfalls

### Pitfall 1: Overclaiming From Safe Boot
**What goes wrong:** a safe boot marker or wrapper trusted-output JSON is used to mark active voltage, fan, self-test, or runtime display/input rows verified. [VERIFIED: `docs/parity/evidence/phase-11-safety-controller-hardware-regression-evidence.md`; `docs/parity/checklist.md`]
**How to avoid:** use exact-claim promotion and require `hardware-regression` for active safety-control rows. [VERIFIED: `14-CONTEXT.md`; `tools/parity/src/main.rs`]
**Warning signs:** checklist notes contain `verified` plus broad active-control language while evidence is only `hardware-smoke` or safe boot. [VERIFIED: `tools/parity/src/main.rs`]

### Pitfall 2: Stale Package Identity
**What goes wrong:** documentation-current source commits are confused with the source commit actually flashed to hardware. [VERIFIED: `docs/parity/evidence/phase-13-final-ultra-205-release-evidence.md`]
**How to avoid:** bind package manifest, source commit, reference commit, and observed firmware/reference markers in the allow manifest and evidence pack. [VERIFIED: `14-CONTEXT.md`; `tools/flash/src/main.rs`]
**Warning signs:** evidence cites a source commit that differs from wrapper-observed `firmware_commit=` without explaining the boundary. [VERIFIED: `docs/parity/evidence/phase-13-final-ultra-205-release-evidence.md`]

### Pitfall 3: DEVICE_URL Assumption
**What goes wrong:** live API/WebSocket telemetry is claimed from route registration or private network inference. [VERIFIED: `docs/parity/evidence/phase-13-final-ultra-205-release-evidence.md`]
**How to avoid:** require explicit `DEVICE_URL`, verify it targets the just-flashed board, redact it, and otherwise write blocked evidence. [VERIFIED: `scripts/phase13-http-static-smoke.sh`; `14-CONTEXT.md`]
**Warning signs:** a live telemetry pack has no explicit target provenance, no curl/client log, or no redaction review. [VERIFIED: `docs/parity/evidence/phase-13-final-ultra-205-release-evidence/redaction-review.md`]

### Pitfall 4: Load/Watchdog Evidence From Startup Logs Only
**What goes wrong:** `safety_supervisor=started` and one yield marker are used to verify responsiveness under load. [VERIFIED: `firmware/bitaxe/src/safety_adapter/watchdog.rs`; `14-CONTEXT.md`]
**How to avoid:** define a bounded workload, liveness markers, timeout, failure markers, and post-action safe-state checks. [VERIFIED: `14-CONTEXT.md`; CITED: https://docs.espressif.com/projects/esp-idf/en/v5.5.4/esp32s3/api-reference/system/wdts.html]
**Warning signs:** no workload stimulus, no response criterion, or no post-action safe-state marker. [VERIFIED: `14-CONTEXT.md`]

### Pitfall 5: Hand-Editing Generated Evidence
**What goes wrong:** JSON/log artifacts are edited manually after a failed or untrusted run. [VERIFIED: `docs/parity/evidence/phase-11-safety-controller-hardware-regression-evidence/README.md`]
**How to avoid:** regenerate generated artifacts through repo-owned commands and use Markdown ledgers only for reviewed summaries. [VERIFIED: `docs/parity/evidence/phase-11-safety-controller-hardware-regression-evidence/README.md`; `tools/flash/src/main.rs`]
**Warning signs:** `flash-command-evidence.json` or raw logs have diffs inconsistent with a wrapper rerun. [VERIFIED: `docs/parity/evidence/phase-11-safety-controller-hardware-regression-evidence/README.md`]

## Code Examples

### `tools/parity` Allow Manifest Types

```rust
#[derive(Debug, serde::Deserialize)]
struct SafetyAllowManifest {
    board: Board205,
    port: String,
    board_info_command: String,
    package_manifest: camino::Utf8PathBuf,
    source_commit: String,
    reference_commit: String,
    surface: SafetySurface,
    claim_type: ClaimType,
    allowed_command: String,
    allowed_inputs: serde_json::Value,
    abort_conditions: Vec<String>,
    recovery_steps: Vec<String>,
    post_action_safe_state_markers: Vec<String>,
    evidence_dir: camino::Utf8PathBuf,
    redaction_reviewer: String,
    checklist_rows: Vec<String>,
}
```

Use this pattern because the repo already uses Rust host tools with `serde`, `serde_json`, `camino`, and `clap`, and Bright Builds requires parsing boundary data into domain types. [VERIFIED: `Cargo.toml`; `tools/parity/Cargo.toml`; `standards/core/architecture.md`; `standards/languages/rust.md`]

### Pending Evidence Contract

```text
phase14_<surface>_status: pending - <reason>
board: 205
port: <detector-selected-or-unavailable>
source_commit: <manifest-source-or-unavailable>
reference_commit: <reference-or-unavailable>
claim_boundary: no hardware-regression claim promoted
conclusion: pending - prerequisites missing; checklist row remains below verified
```

Use this pattern because Phase 13 helpers already record missing `DEVICE_URL` and allow-flag blockers as explicit evidence instead of running around gates. [VERIFIED: `scripts/phase13-http-static-smoke.sh`; `scripts/phase13-recovery-regression.sh`; `docs/parity/evidence/phase-13-final-ultra-205-release-evidence/recovery-regression.md`]

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
| --- | --- | --- | --- |
| Raw monitor fallback could be used as evidence context | Wrapper-owned noninteractive `flash-monitor` with JSON trust classification | Phase 9 | Phase 14 should not cite raw monitor fallback for promotion. [VERIFIED: `docs/parity/evidence/phase-09-flash-monitor-evidence-wrapper-hardening.md`; `tools/flash/src/main.rs`] |
| Safety phase documented pending active hardware evidence | Phase 14 requires machine-enforced allow manifests before active probes | Phase 14 context | Active evidence must prove gates, inputs, recovery, safe-state, and redaction. [VERIFIED: `14-CONTEXT.md`] |
| Release evidence could stop at package/serial identity | Phase 13 split package, serial, HTTP, OTA, recovery, and blocker classes | Phase 13 | Phase 14 should use separate evidence classes for safety surfaces. [VERIFIED: `docs/parity/evidence/phase-13-final-ultra-205-release-evidence.md`; `14-CONTEXT.md`] |
| Startup display evidence could be misread as runtime display parity | Runtime display/input gap is explicit | Phase 6 | Runtime display/input stays below verified without real runtime route and physical/log/API/WebSocket observation. [VERIFIED: `docs/parity/evidence/phase-06-display-input-runtime-gap.md`; `14-CONTEXT.md`] |

**Deprecated/outdated:**
- Treating unit or workflow evidence as active safety-control proof is invalid for verified claims. [VERIFIED: `docs/parity/checklist.md`; `tools/parity/src/main.rs`]
- Treating Phase 11 safe boot as fresh Phase 14 active evidence is invalid unless cited as a narrow historical subclaim. [VERIFIED: `14-CONTEXT.md`; `docs/parity/evidence/phase-11-safety-controller-hardware-regression-evidence.md`]

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
| --- | --- | --- | --- |

All claims in this research were verified against local project files, local command output, pinned reference source, or official documentation. [VERIFIED: source list below]

## Open Questions

1. **Is a reachable `DEVICE_URL` available during execution?**
   - What we know: `DEVICE_URL` is unset in the research environment, and Phase 13 live HTTP/OTA evidence was blocked when it was missing. [VERIFIED: env audit; `docs/parity/evidence/phase-13-final-ultra-205-release-evidence.md`]
   - What's unclear: whether the executor will have a target URL for the just-flashed board. [VERIFIED: env audit]
   - Recommendation: plan live API/WebSocket packs to write blocked evidence when `DEVICE_URL` is absent. [VERIFIED: `14-CONTEXT.md`; `scripts/phase13-http-static-smoke.sh`]

2. **Will Phase 14 attempt active actuation or only document pending active claims?**
   - What we know: active DS4432U, fan duty, overheat/fault, self-test hardware, runtime input/display, and load/stress require bounded `hardware-regression`. [VERIFIED: `14-CONTEXT.md`; `tools/parity/src/main.rs`]
   - What's unclear: whether safe firmware routes and recovery procedures exist for each active surface at execution time. [VERIFIED: `firmware/bitaxe/src/safety_adapter.rs`; `docs/parity/checklist.md`]
   - Recommendation: create plan tasks that validate prerequisites first and only run active wrappers after manifest validation passes. [VERIFIED: `14-CONTEXT.md`]

3. **How will WebSocket frames be captured?**
   - What we know: `websocat` and Python WebSocket libraries are missing locally. [VERIFIED: env audit; Python module probe]
   - What's unclear: whether the plan should install/use a maintained client or keep frame-level live telemetry pending. [VERIFIED: env audit]
   - Recommendation: default to blocked/pending frame evidence unless the plan adds an explicit maintained-client dependency and redaction path. [VERIFIED: `14-CONTEXT.md`; `standards/core/verification.md`]

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
| --- | --- | --- | --- | --- |
| `just` | repo command surface | yes | `1.48.0` | none needed [VERIFIED: env audit] |
| `bazel` | build/test/package/parity | yes | `9.1.1` | none needed [VERIFIED: env audit] |
| `cargo` | Rust pre-commit and focused tests | yes | `1.88.0-nightly` | none needed [VERIFIED: env audit] |
| `rustc` | Rust build/test | yes | `1.88.0-nightly` | none needed [VERIFIED: env audit] |
| `espflash` | detector, board-info, flash/monitor | yes | `4.0.1` | no fallback for hardware evidence [VERIFIED: env audit; `espflash --help`] |
| `espup` | ESP Rust toolchain install/repair | yes | `0.15.1` | use `just bootstrap-esp` if toolchain repair is needed [VERIFIED: env audit; `AGENTS.md`] |
| `ldproxy` | ESP-IDF Rust linker | yes | help banner only | no fallback for firmware build [VERIFIED: env audit; `.cargo/config.toml`] |
| `python3` | script JSON parsing | yes | `3.14.4` | Rust `serde_json` in host tools [VERIFIED: env audit; `scripts/phase13-http-static-smoke.sh`] |
| `curl` | live HTTP/API probes | yes | `8.7.1` | blocked evidence if missing [VERIFIED: env audit; `scripts/phase13-http-static-smoke.sh`] |
| `jq` | optional JSON inspection | yes | `1.7.1` | Python/Rust JSON parsing [VERIFIED: env audit] |
| `websocat` | live WebSocket frame capture | no | - | add explicit dependency or record frame proof pending [VERIFIED: env audit] |
| Python `websocket` / `websockets` modules | possible WebSocket fallback | no | - | add maintained client dependency or pending evidence [VERIFIED: Python module probe] |
| `DEVICE_URL` | live API/WebSocket probes | no | unset | write blocked live telemetry evidence [VERIFIED: env audit; `docs/parity/evidence/phase-13-final-ultra-205-release-evidence.md`] |
| connected Ultra 205 | hardware-regression evidence | not probed in research | unknown | execution must start with `just detect-ultra205`; otherwise pending evidence [VERIFIED: `AGENTS.md`; research command log] |

**Missing dependencies with no fallback:**
- Connected Ultra 205 detection is required for any live hardware evidence; research intentionally did not run `just detect-ultra205`. [VERIFIED: `AGENTS.md`; research command log]
- `DEVICE_URL` is required for live API/WebSocket probes and is unset in the research environment. [VERIFIED: env audit]

**Missing dependencies with fallback:**
- `websocat` is missing; the fallback is to add an explicit maintained-client dependency or keep frame-level WebSocket proof pending. [VERIFIED: env audit; Python module probe]

## Validation Architecture

### Test Framework

| Property | Value |
| --- | --- |
| Framework | Bazel `9.1.1` with `rules_rust 0.70.0` and `rules_shell 0.8.0`; Cargo `1.88.0-nightly` for repo-required pre-commit checks. [VERIFIED: env audit; `MODULE.bazel`; `AGENTS.md`] |
| Config file | `MODULE.bazel`, per-crate `BUILD.bazel`, `scripts/BUILD.bazel`, root `Cargo.toml`. [VERIFIED: `MODULE.bazel`; `scripts/BUILD.bazel`; `Cargo.toml`] |
| Quick run command | `cargo test -p bitaxe-safety --all-features && cargo test -p bitaxe-parity --all-features` for pure safety/parity edits. [VERIFIED: `docs/parity/evidence/phase-11-safety-controller-hardware-regression-evidence.md`; `tools/parity/BUILD.bazel`] |
| Full suite command | `just test`; pre-commit also requires `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, and `cargo test --all-features`. [VERIFIED: `Justfile`; `AGENTS.md`] |

### Phase Requirements -> Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
| --- | --- | --- | --- | --- |
| SAFE-01 | Voltage/power decisions fail closed and active writes require manifest/evidence | unit + workflow + hardware-gated | `cargo test -p bitaxe-safety --all-features power`; new `cargo test -p bitaxe-parity --all-features safety_allow` | partial; allow tests Wave 0 [VERIFIED: `crates/bitaxe-safety/src/power.rs`; `tools/parity/BUILD.bazel`] |
| SAFE-02 | Thermal/fan read-only vs actuation evidence is separated | unit + workflow + hardware-gated | `cargo test -p bitaxe-safety --all-features thermal`; new `bazel test //scripts:phase14_thermal_fan_test` if wrapper added | partial; wrapper test Wave 0 [VERIFIED: `crates/bitaxe-safety/src/thermal.rs`; `scripts/BUILD.bazel`] |
| SAFE-03 | PID decisions remain covered before hardware effects | unit | `cargo test -p bitaxe-safety --all-features thermal` | yes [VERIFIED: `crates/bitaxe-safety/src/thermal.rs`; `docs/parity/evidence/phase-06-safety-controllers-and-self-test.md`] |
| SAFE-04 | Fault paths publish safe states and active fault evidence is gated | unit + workflow + hardware-gated | `cargo test -p bitaxe-safety --all-features fault`; new allow-manifest invalid-fault tests | partial; allow tests Wave 0 [VERIFIED: `crates/bitaxe-safety/src/fault.rs`; `14-CONTEXT.md`] |
| SAFE-05 | Self-test hardware submodes stay below verified unless safely proven | unit + workflow + hardware-gated | `cargo test -p bitaxe-safety --all-features self_test`; new self-test wrapper test if route exists | partial; wrapper test Wave 0 [VERIFIED: `crates/bitaxe-safety/src/self_test.rs`; `14-CONTEXT.md`] |
| SAFE-06 | Runtime display/input remains exact or deferred | workflow + manual/hardware | `cargo test -p bitaxe-parity --all-features safety_critical`; new display/input evidence checks if wrapper added | partial; wrapper test Wave 0 [VERIFIED: `tools/parity/src/main.rs`; `docs/parity/evidence/phase-06-display-input-runtime-gap.md`] |
| SAFE-07 | Hardware telemetry packs record freshness or safe-unavailable status | workflow + hardware-gated | new `bazel test //scripts:phase14_power_telemetry_test //scripts:phase14_thermal_fan_test` if scripts added | no; Wave 0 [VERIFIED: `docs/parity/evidence/phase-11-safety-controller-hardware-regression-evidence.md`] |
| SAFE-08 | Verified safety rows require hardware-smoke/regression and active rows require regression | unit/workflow | `cargo test -p bitaxe-parity --all-features active_safety_control`; `just parity` | yes [VERIFIED: `tools/parity/src/main.rs`; `Justfile`] |
| SAFE-09 | Watchdog/load responsiveness is bounded and observable | unit + hardware-gated | `cargo test -p bitaxe-safety --all-features watchdog`; new load wrapper test if workload added | partial; load wrapper Wave 0 [VERIFIED: `crates/bitaxe-safety/src/watchdog.rs`; `firmware/bitaxe/src/safety_adapter/watchdog.rs`] |
| EVD-05 | Evidence layers and promotion semantics remain valid | workflow | `just parity`; new allow-manifest validation tests; redaction checklist review | partial; allow tests Wave 0 [VERIFIED: `Justfile`; `tools/parity/src/main.rs`; `14-CONTEXT.md`] |

### Sampling Rate

- **Per task commit:** run focused Cargo/Bazel tests for touched crate/script plus `git diff --check` on touched paths. [VERIFIED: `standards/core/verification.md`; `AGENTS.md`]
- **Per wave merge:** run `just parity` and affected `bazel test` targets. [VERIFIED: `Justfile`; `tools/parity/src/main.rs`]
- **Phase gate:** run redaction review, `just parity`, `just test`, `just verify-reference`, relevant Rust pre-commit checks, and lifecycle verification. [VERIFIED: `14-CONTEXT.md`; `AGENTS.md`]

### Wave 0 Gaps

- [ ] `tools/parity/src/safety_allow.rs` - typed allow-manifest parser and validator for D-01 through D-04. [VERIFIED: `14-CONTEXT.md`; `tools/parity/src/main.rs`]
- [ ] `tools/parity` tests for missing board `205`, port mismatch, stale package, missing recovery, unsupported surface, missing safe-state markers, missing redaction reviewer, and active claim without `hardware-regression`. [VERIFIED: `14-CONTEXT.md`; `tools/parity/src/main.rs`]
- [ ] `docs/parity/evidence/phase-14-safety-hardware-evidence-completion/README.md` and `redaction-review.md` component-pack scaffolds. [VERIFIED: `docs/parity/evidence/phase-11-safety-controller-hardware-regression-evidence/README.md`; `14-CONTEXT.md`]
- [ ] `scripts/phase14-*-test.sh` targets for any new per-surface shell wrapper added in the plan. [VERIFIED: `scripts/BUILD.bazel`; `scripts/phase13-*.sh`]
- [ ] WebSocket frame capture dependency decision if live `/api/ws/live` frame proof is in scope. [VERIFIED: env audit; Python module probe]

## Security Domain

Security enforcement is enabled by default because `.planning/config.json` does not set `security_enforcement` to `false`. [VERIFIED: `.planning/config.json`]

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
| --- | --- | --- |
| V2 Authentication | no | Phase 14 does not add user authentication flows; protect evidence by avoiding secrets in artifacts. [VERIFIED: `14-CONTEXT.md`; CITED: https://owasp.org/www-project-application-security-verification-standard/] |
| V3 Session Management | no | Phase 14 does not add browser/session state. [VERIFIED: `14-CONTEXT.md`; CITED: https://owasp.org/www-project-application-security-verification-standard/] |
| V4 Access Control | yes | Active hardware actions require allow manifests, board gates, exact commands, and recovery gates. [VERIFIED: `14-CONTEXT.md`; `AGENTS.md`; CITED: https://owasp.org/www-project-application-security-verification-standard/] |
| V5 Input Validation | yes | Parse allow manifests into typed domain values and reject invalid boards, ports, commands, bounds, and paths before hardware actions. [VERIFIED: `standards/core/architecture.md`; `14-CONTEXT.md`; CITED: https://owasp.org/www-project-application-security-verification-standard/] |
| V6 Cryptography | yes | Use existing package checksums/manifests and do not invent crypto; redaction must protect secrets. [VERIFIED: `docs/parity/evidence/phase-13-final-ultra-205-release-evidence.md`; `PROVENANCE.md`; CITED: https://owasp.org/www-project-application-security-verification-standard/] |

### Known Threat Patterns for Phase 14

| Pattern | STRIDE | Standard Mitigation |
| --- | --- | --- |
| Unsafe actuation from forged or stale manifest | Tampering/Elevation of privilege | Validate board `205`, selected port, board-info result, package/source/reference identity, allowed command, input bounds, recovery, and safe-state markers before running. [VERIFIED: `14-CONTEXT.md`] |
| Secret leakage in evidence logs | Information disclosure | Redaction review must inspect serial logs, JSON, API responses, WebSocket frames, terminal output, and manual observations before citation. [VERIFIED: `14-CONTEXT.md`; `docs/parity/evidence/phase-11-safety-controller-hardware-regression-evidence/redaction-review.md`; `docs/parity/evidence/phase-13-final-ultra-205-release-evidence/redaction-review.md`] |
| Private network target inference | Information disclosure/Spoofing | Require explicit `DEVICE_URL`; do not scan or infer targets; sanitize/redact URL values. [VERIFIED: `scripts/phase13-http-static-smoke.sh`; `docs/parity/evidence/phase-13-final-ultra-205-release-evidence.md`] |
| Overclaiming checklist status | Repudiation/Tampering | `tools/parity --fail-on-invalid-verified` remains mandatory before finalizing. [VERIFIED: `Justfile`; `tools/parity/src/main.rs`] |
| Shell command injection through manifest fields | Tampering/Elevation of privilege | Treat manifest fields as data, compare against exact allowed command shapes, and avoid `eval`. [VERIFIED: `standards/core/verification.md`; `scripts/phase13-recovery-regression.sh`] |

## Sources

### Primary (HIGH confidence)

- `.planning/phases/14-safety-hardware-evidence-completion/14-CONTEXT.md` - locked phase decisions, discretion, deferred scope, canonical refs. [VERIFIED: file read]
- `.planning/REQUIREMENTS.md` - SAFE-01 through SAFE-09 and EVD-05. [VERIFIED: file read]
- `.planning/ROADMAP.md` - Phase 14 goal, success criteria, verification expectations, research flags. [VERIFIED: file read]
- `.planning/STATE.md` - Phase 13 state, blockers, current safety/release evidence gaps. [VERIFIED: file read]
- `AGENTS.md`, `AGENTS.bright-builds.md`, `standards/core/architecture.md`, `standards/core/verification.md`, `standards/core/testing.md`, `standards/languages/rust.md` - local workflow, hardware gates, evidence rules, architecture, verification, and Rust standards. [VERIFIED: file read]
- `docs/parity/checklist.md` - current safety/API/UI/status rows and evidence boundaries. [VERIFIED: codebase grep]
- `docs/parity/evidence/phase-11-safety-controller-hardware-regression-evidence.md` and subdirectory README/redaction review - component-pack model and safe boot residual gaps. [VERIFIED: file read]
- `docs/parity/evidence/phase-13-final-ultra-205-release-evidence.md` and subdirectory runbook/redaction/recovery docs - package identity, `DEVICE_URL` blockers, recovery gate pattern. [VERIFIED: file read]
- `tools/parity/src/main.rs` - verified-row validation for safety-critical and active safety-control claims. [VERIFIED: file read]
- `tools/flash/src/main.rs` - wrapper-owned flash/monitor evidence and trusted marker classification. [VERIFIED: codebase grep]
- `crates/bitaxe-safety/src/*.rs`, `firmware/bitaxe/src/safety_adapter*.rs`, `firmware/bitaxe/src/runtime_snapshot.rs`, `firmware/bitaxe/src/http_api.rs`, `crates/bitaxe-api/src/telemetry.rs` - pure safety logic, observe-only adapters, runtime telemetry, WebSocket cadence. [VERIFIED: file read]
- `reference/esp-miner` at `c1915b0a63bfabebdb95a515cedfee05146c1d50` - Ultra 205 config and reference safety behavior. [VERIFIED: `git -C reference/esp-miner rev-parse HEAD`; file reads]

### Primary External (HIGH confidence)

- Espressif ESP-IDF v5.5.4 HTTP Server docs - WebSocket server APIs including `httpd_ws_recv_frame` and `httpd_ws_send_frame_async`. [CITED: https://docs.espressif.com/projects/esp-idf/en/v5.5.4/esp32s3/api-reference/protocols/esp_http_server.html]
- Espressif ESP-IDF v5.5.4 I2C docs - I2C master driver concepts and resource allocation. [CITED: https://docs.espressif.com/projects/esp-idf/en/v5.5.4/esp32s3/api-reference/peripherals/i2c.html]
- Espressif ESP-IDF v5.5.4 watchdog docs - TWDT usage and task reset/feed behavior. [CITED: https://docs.espressif.com/projects/esp-idf/en/v5.5.4/esp32s3/api-reference/system/wdts.html]
- Espressif ESP-IDF v5.5.4 OTA docs - rollback state APIs. [CITED: https://docs.espressif.com/projects/esp-idf/en/v5.5.4/esp32s3/api-reference/system/ota.html]
- OWASP ASVS project page - ASVS category frame used for security-domain classification. [CITED: https://owasp.org/www-project-application-security-verification-standard/]

### Secondary (MEDIUM confidence)

- Local `espflash` CLI help for `board-info`, `monitor`, and `list-ports` commands. [VERIFIED: `espflash --help`; `espflash board-info --help`; `espflash monitor --help`; `espflash list-ports --help`]

### Tertiary (LOW confidence)

- None. [VERIFIED: sources above]

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - versions and commands were verified from repo manifests and local tools. [VERIFIED: env audit; `Cargo.toml`; `MODULE.bazel`; `.cargo/config.toml`]
- Architecture: HIGH - recommended structure follows locked Phase 14 decisions, existing Phase 11/13 evidence-pack patterns, and local functional-core/imperative-shell standards. [VERIFIED: `14-CONTEXT.md`; `docs/parity/evidence/phase-11-safety-controller-hardware-regression-evidence/README.md`; `docs/parity/evidence/phase-13-final-ultra-205-release-evidence/README.md`; `standards/core/architecture.md`; `standards/languages/rust.md`]
- Pitfalls: HIGH - pitfalls are directly present in prior evidence blockers and parity guard semantics. [VERIFIED: `docs/parity/evidence/phase-11-safety-controller-hardware-regression-evidence.md`; `docs/parity/evidence/phase-13-final-ultra-205-release-evidence.md`; `tools/parity/src/main.rs`]
- Hardware actuation details: MEDIUM - Ultra 205 capabilities and reference behavior are verified from the pinned reference tree, but actual board runtime behavior must be proven by Phase 14 execution. [VERIFIED: `reference/esp-miner/main/device_config.h`; `reference/esp-miner/main/power/DS4432U.c`; `reference/esp-miner/main/tasks/fan_controller_task.c`; `14-CONTEXT.md`]
- Live API/WebSocket evidence: MEDIUM - server code and ESP-IDF docs are verified, but `DEVICE_URL` and a WebSocket client are unavailable in the research environment. [VERIFIED: `firmware/bitaxe/src/http_api.rs`; `crates/bitaxe-api/src/telemetry.rs`; env audit; CITED: https://docs.espressif.com/projects/esp-idf/en/v5.5.4/esp32s3/api-reference/protocols/esp_http_server.html]

**Research date:** 2026-06-30
**Valid until:** 2026-07-07 for environment availability and live evidence prerequisites; 2026-07-30 for stable repo architecture and pinned reference facts. [VERIFIED: `.planning/config.json`; environment context current date]

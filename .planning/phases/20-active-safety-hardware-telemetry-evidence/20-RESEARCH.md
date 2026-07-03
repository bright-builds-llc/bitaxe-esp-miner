# Phase 20: Active Safety Hardware Telemetry Evidence - Research

**Researched:** 2026-07-03
**Domain:** Ultra 205 active safety hardware evidence, live API/WebSocket telemetry correlation, and parity governance
**Confidence:** HIGH for existing repo governance and evidence patterns; MEDIUM for active hardware execution paths because current firmware adapters suppress active effects unless Phase 20 adds bounded diagnostic routes. [VERIFIED: .planning/phases/20-active-safety-hardware-telemetry-evidence/20-CONTEXT.md] [VERIFIED: firmware/bitaxe/src/safety_adapter.rs]

<user_constraints>
## User Constraints (from CONTEXT.md)

All bullets in this section are copied verbatim from `.planning/phases/20-active-safety-hardware-telemetry-evidence/20-CONTEXT.md`. [VERIFIED: .planning/phases/20-active-safety-hardware-telemetry-evidence/20-CONTEXT.md]

### Locked Decisions

#### Gated Evidence Runbook

- **D-01:** Reuse and tighten the Phase 14 safety allow-manifest pattern rather than inventing a new broad hardware script. Every active Phase 20 probe must bind board `205`, selected detector port, passed board-info, package manifest identity, source commit, reference commit, allowed surface, claim tier, exact command, bounded inputs, abort conditions, recovery steps, post-action safe-state markers, evidence directory, redaction reviewer, and checklist rows or subclaims.
- **D-02:** Start live hardware work with `just detect-ultra205`. Continue only when it finds exactly one likely ESP32-S3 USB serial port and `espflash board-info --chip esp32s3 --port <port> --non-interactive` succeeds. Zero ports, multiple ports, board-info failure, stale package identity, missing recovery instructions, or target other than board `205` must create blocked evidence instead of a workaround.
- **D-03:** Use surface-scoped probes and evidence packs. Keep active voltage/power, fan/thermal, self-test/watchdog/load, display/input, failure paths, and live API/WebSocket telemetry independently promotable or independently blocked.
- **D-04:** Treat resets, watchdog panics, unexpected reboot, detector failure, missing safe-state markers, missing restore package, or unavailable route evidence as stop/recovery signals. A probe that cannot prove post-action safe state must not support an active safety verified claim.

#### Active Safety Surface Coverage

- **D-05:** Separate read-only observations from actuator or failure-path proof. INA260, thermal, fan RPM, `/api/system/info`, statistics, and WebSocket readings can support only exact observed telemetry subclaims. DS4432U writes, fan duty effects, ASIC reset/power sequencing, overheat/fault behavior, self-test hardware submodes, bounded load stress, runtime input, and runtime display need hardware-regression evidence.
- **D-06:** Prefer observe-only or safe-unavailable evidence when a production-safe stimulus route does not exist. Observe-only evidence may prove blockers, safe state, stale/unavailable projection, or transport shape, but it must not verify active control behavior.
- **D-07:** If Phase 20 introduces any new active probe route or diagnostic trigger, it must be compile-gated, bounded, impossible to expose accidentally in production flows, and covered by tests plus redaction review. Repo-owned ESP/esp-rs tooling remains the preferred path.
- **D-08:** Live API/WebSocket telemetry must be correlated with hardware observations and safe-state markers. Route presence, a no-upgrade response, or a stale cached API body is not enough to prove live safety telemetry freshness or cadence.

#### Runtime Display, Input, Self-Test, Watchdog, And Load

- **D-09:** Runtime display/input remains below verified unless a real runtime route is exercised and physically or log/API/WebSocket-observed. Startup SSD1306 evidence and `display_input_status=runtime_gap` remain supporting breadcrumbs only.
- **D-10:** Self-test hardware submodes remain below verified unless the run safely proves the exact submode, pass/fail/cancel behavior, production-mining gate behavior, recovery path, and post-action safe state.
- **D-11:** Watchdog/load evidence must use a bounded workload or safe stimulus with pass/fail criteria. Supervisor startup/yield logs prove only the supervisor shell, not load stress, blocked task behavior, or watchdog recovery.
- **D-12:** Failure-path evidence must name the stimulus, expected fault, abort condition, restore path, observed status, API/log/WebSocket projection, and final safe-state marker before any fault-path checklist row can be promoted.

#### Checklist, Redaction, And Verification

- **D-13:** Use exact-claim promotion. Active safety-control and failure-path rows require `hardware-regression`; narrow read-only, safe-unavailable, startup, or transport observations may use `hardware-smoke` only when the artifact actually proves the subclaim.
- **D-14:** Preserve checklist row IDs where precise notes can communicate the boundary. Split or add subclaim wording only if a broad row would otherwise force a false verified claim.
- **D-15:** Redaction review is mandatory before any evidence is committed. It must cover serial logs, JSON manifests, API bodies, WebSocket frames, detector output, board-info output, package logs, pasted command output, manual observations, `DEVICE_URL`, IP addresses, MAC addresses, SSIDs, Wi-Fi credentials, pool credentials, worker secrets, API tokens, NVS secret values, and local terminal secrets.
- **D-16:** Final verification must include relevant wrapper/script tests, changed Rust checks, `just test`, `just parity`, `just verify-reference`, reference diff cleanliness, redaction review, lifecycle validation, and every hardware/network command actually used. The wrapper-level commit/push gate may run only when `20-VERIFICATION.md` reports `status: passed`.

### the agent's Discretion

The agent may choose the exact plan count, evidence directory layout, JSON field names, helper names, whether to extend Phase 14 helpers or add Phase 20 wrappers, and whether new checks live in `tools/parity`, a repo-owned script, or a small host tool. Those choices must keep `reference/esp-miner` read-only, preserve functional core plus imperative shell, use ESP-IDF/esp-rs tooling before custom hardware paths, avoid secrets in evidence, avoid standalone body `---` separators in parsed Markdown, and avoid broad verified claims.

### Deferred Ideas (OUT OF SCOPE)

- Live production mining, accepted/rejected shares, production pool behavior, and bounded soak belong to Phase 21.
- Firmware OTA, whole-`www` OTAWWW, failed-update recovery, large erase, interrupted update, rollback, and boot-validation evidence remain governed by Phases 18 and 19 and are not Phase 20 scope.
- Non-205 boards, BM1370/BM1368/BM1397, TPS546 active behavior, all-board factory images, Stratum v2, BAP, Angular AxeOS replacement, and performance tuning remain deferred.
- Full LVGL runtime display carousel, display config, timeout, rotation, inversion, and broad button-routing parity remain outside Phase 20 unless a plan proves a safe bounded route and physical evidence path.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
| --- | --- | --- |
| SAFE-01 | Ultra 205 voltage and power-control surfaces use bounded typed decisions and fail closed on invalid configuration, communication failure, or unsafe readings. [VERIFIED: .planning/REQUIREMENTS.md] | `crates/bitaxe-safety/src/power.rs` already has typed power observations and suppresses voltage writes unless hardware evidence and armed mode are present; Phase 20 must prove active DS4432U behavior with `hardware-regression` or keep it blocked. [VERIFIED: crates/bitaxe-safety/src/power.rs] [VERIFIED: tools/parity/src/main.rs] |
| SAFE-02 | Thermal sensor and fan control surfaces expose upstream-compatible readings, fan duty behavior, RPM behavior, and failure reporting. [VERIFIED: .planning/REQUIREMENTS.md] | `crates/bitaxe-safety/src/thermal.rs` has pure thermal/fan decisions, while firmware currently reports thermal/fan unavailable and suppresses fan writes; Phase 20 must separately prove read telemetry, fan duty effect, RPM, and failure status. [VERIFIED: crates/bitaxe-safety/src/thermal.rs] [VERIFIED: firmware/bitaxe/src/safety_adapter/thermal.rs] |
| SAFE-03 | PID and thermal-control decisions are covered by pure unit tests before hardware effects are enabled. [VERIFIED: .planning/REQUIREMENTS.md] | Phase planning should preserve pure-test coverage in `crates/bitaxe-safety` before enabling or citing any firmware effects. [VERIFIED: standards/core/testing.md] [VERIFIED: crates/bitaxe-safety/src/thermal.rs] |
| SAFE-04 | Overheat, fan, power, thermal, and ASIC fault paths enter safe states and expose user-visible status compatible with upstream behavior. [VERIFIED: .planning/REQUIREMENTS.md] | `crates/bitaxe-safety/src/fault.rs` classifies fault paths and returns fail-closed effects; Phase 20 needs stimulus-specific evidence with API/log/WebSocket projection and final safe-state marker. [VERIFIED: crates/bitaxe-safety/src/fault.rs] [VERIFIED: .planning/phases/20-active-safety-hardware-telemetry-evidence/20-CONTEXT.md] |
| SAFE-05 | Self-test lifecycle behavior covers factory flags, start, pass, fail, restart, cancel, and user-visible result reporting. [VERIFIED: .planning/REQUIREMENTS.md] | `crates/bitaxe-safety/src/self_test.rs` models lifecycle decisions, but Phase 14 evidence left hardware self-test submodes pending; Phase 20 must prove exact submodes or keep them below verified. [VERIFIED: crates/bitaxe-safety/src/self_test.rs] [VERIFIED: docs/parity/evidence/phase-14-safety-hardware-evidence-completion/self-test-watchdog-load.md] |
| SAFE-06 | Display and input status surfaces needed for normal Ultra 205 administration are preserved or explicitly documented as deferred gaps. [VERIFIED: .planning/REQUIREMENTS.md] | Current display evidence proves startup SSD1306 text and a runtime gap marker only; runtime display/input needs a real runtime route plus physical or log/API/WebSocket observation. [VERIFIED: firmware/bitaxe/src/display_adapter.rs] [VERIFIED: docs/parity/evidence/phase-14-safety-hardware-evidence-completion/display-input.md] |
| SAFE-07 | Power, current, voltage, fan, and temperature telemetry are captured where Ultra 205 hardware exposes them. [VERIFIED: .planning/REQUIREMENTS.md] | API models preserve zero-compatible unavailable telemetry unless hardware-verified fresh readings exist; Phase 20 must correlate hardware observations with `/api/system/info` and WebSocket frames. [VERIFIED: crates/bitaxe-api/src/snapshot.rs] [VERIFIED: crates/bitaxe-api/src/wire.rs] [VERIFIED: scripts/phase17-websocket-capture.mjs] |
| SAFE-08 | Safety-critical surfaces cannot be marked `verified` without `hardware-smoke` or `hardware-regression` evidence. [VERIFIED: .planning/REQUIREMENTS.md] | `tools/parity/src/main.rs` already rejects safety-critical verified rows without hardware evidence and active safety-control rows without `hardware-regression`; Phase 20 must keep this guard green. [VERIFIED: tools/parity/src/main.rs] |
| SAFE-09 | Mining, control, API, and telemetry tasks avoid watchdog starvation and preserve observable responsiveness under load. [VERIFIED: .planning/REQUIREMENTS.md] | Firmware watchdog supervisor currently proves startup/yield shell only; bounded workload evidence needs pass/fail criteria, API/WebSocket responsiveness, and safe-state markers. [VERIFIED: firmware/bitaxe/src/safety_adapter/watchdog.rs] [VERIFIED: scripts/phase14-self-test-watchdog-load.sh] |
| EVD-05 | Verification layers include unit tests, golden fixtures, API comparison, hardware smoke tests, and hardware regression or soak evidence where appropriate. [VERIFIED: .planning/REQUIREMENTS.md] | Phase 20 must layer pure tests, wrapper tests, detector-gated hardware evidence, redacted API/WebSocket captures, parity validation, and conservative checklist updates. [VERIFIED: .planning/phases/20-active-safety-hardware-telemetry-evidence/20-CONTEXT.md] [VERIFIED: standards/core/verification.md] |
</phase_requirements>

## Summary

Phase 20 is primarily an evidence and safety-governance phase, not a broad firmware rewrite. The repo already has Phase 14 allow-manifest machinery, Phase 14 surface wrappers, Phase 17 explicit-target HTTP/WebSocket capture helpers, and `tools/parity` guards that prevent safety-critical overclaiming. [VERIFIED: tools/parity/src/safety_allow.rs] [VERIFIED: scripts/BUILD.bazel] [VERIFIED: scripts/phase17-websocket-capture.mjs] [VERIFIED: tools/parity/src/main.rs]

The planning risk is that active hardware effects are mostly still suppressed or unavailable in firmware adapters. Voltage writes, fan duty writes, self-test hardware submodes, runtime display/input, and bounded load/fault stimuli cannot be verified by current read-only wrappers alone. [VERIFIED: firmware/bitaxe/src/safety_adapter/power.rs] [VERIFIED: firmware/bitaxe/src/safety_adapter/thermal.rs] [VERIFIED: docs/parity/evidence/phase-14-safety-hardware-evidence-completion.md]

**Primary recommendation:** Reuse Phase 14 allow manifests and wrappers, compose with Phase 17 live telemetry capture, add only surface-scoped Phase 20 helpers or validator extensions that are needed for exact evidence, and keep unsupported active surfaces explicitly blocked instead of promoting them. [VERIFIED: .planning/phases/20-active-safety-hardware-telemetry-evidence/20-CONTEXT.md] [VERIFIED: tools/parity/src/safety_allow.rs] [VERIFIED: scripts/phase17-live-http-api-smoke.sh]

## Project Constraints (from AGENTS.md)

- Before plan, review, implementation, or audit work, load `AGENTS.md`, `AGENTS.bright-builds.md`, `standards-overrides.md`, and relevant managed standards pages. [VERIFIED: AGENTS.md]
- Use GSD workflow artifacts for direct repo edits unless the user explicitly bypasses the workflow; this research artifact is part of the requested GSD phase workflow. [VERIFIED: AGENTS.md]
- Autonomous Ultra 205 hardware use must start with `just detect-ultra205`, and detection succeeds only when exactly one likely ESP USB serial port is found and `espflash board-info --chip esp32s3 --port <port> --non-interactive` succeeds. [VERIFIED: AGENTS.md] [VERIFIED: scripts/detect-ultra205.sh]
- If `wifi-credentials.json` exists, agents may pass it to repo-owned `just flash` or `just flash-monitor`, but must not read, print, summarize, or commit its contents. [VERIFIED: AGENTS.md]
- Stop and ask or record evidence pending when there are zero likely ports, multiple likely ports, board-info failure, non-205 target, or missing required recovery/evidence instructions. [VERIFIED: AGENTS.md]
- Destructive or fault-injection verification is allowed only when the active phase plan documents recovery path and required evidence. [VERIFIED: AGENTS.md]
- Every hardware run must record board `205`, selected port, source commit, reference commit, package manifest/artifacts, exact commands, board-info output, captured logs, observed behavior, and conclusion. [VERIFIED: AGENTS.md]
- Evidence intended for commit or sharing must be redacted; secrets, pool credentials, Wi-Fi credentials, private endpoints, and NVS secret values must not be committed. [VERIFIED: AGENTS.md]
- Parsed Markdown artifacts must not use standalone body `---` separators after frontmatter; use headings or `***` instead. [VERIFIED: AGENTS.md]
- Rust work must preserve functional core plus imperative shell, typed boundaries, one-concern tests, Arrange/Act/Assert structure, and repo-native verification. [VERIFIED: standards/core/architecture.md] [VERIFIED: standards/core/testing.md] [VERIFIED: standards/languages/rust.md] [VERIFIED: standards/core/verification.md]
- `reference/esp-miner` is read-only behavioral evidence and must not be modified by Phase 20 work. [VERIFIED: AGENTS.md] [VERIFIED: docs/adr/0005-read-only-reference-implementation.md]

## Standard Stack

### Core

| Tool / Library | Version | Purpose | Why Standard |
| --- | --- | --- | --- |
| `just` | 1.48.0 | Human command surface for detector, package, flash/monitor, parity, reference verification, build, and tests. | Repo-local commands are defined in `Justfile`, and the installed version was verified locally. [VERIFIED: Justfile] [VERIFIED: env probe `just --version`] |
| Bazel | 9.1.1 | Canonical automation graph for scripts, Rust tools, and tests. | The repo routes helper targets/tests through Bazel and local Bazel version is 9.1.1. [VERIFIED: scripts/BUILD.bazel] [VERIFIED: env probe `bazel --version`] |
| Rust/Cargo | `cargo 1.88.0-nightly`, `rustc 1.88.0-nightly` | Pure safety/API/parity crates and firmware Rust code. | Safety rules and parity tooling are Rust crates; local toolchain versions were verified. [VERIFIED: Cargo.toml] [VERIFIED: env probes `cargo --version`, `rustc --version`] |
| `espflash` | 4.0.1 | Board-info, package/flash/monitor evidence path, and serial interaction backend. | Detector and repo flash commands depend on `espflash`; local version was verified. [VERIFIED: scripts/detect-ultra205.sh] [VERIFIED: env probe `espflash --version`] |
| Phase 14 safety allow manifests | repo-owned | Active hardware allow gates, claim tiers, surfaces, abort conditions, safe-state markers, and evidence metadata validation. | The validator already enforces board `205`, detector command, board-info command/status, package identity, active tiers, recovery steps, aborts, and safe-state markers. [VERIFIED: tools/parity/src/safety_allow.rs] |
| Phase 17 live HTTP/WebSocket helpers | repo-owned | Explicit-target API and WebSocket capture with redaction and bounded frame counts. | The helpers already enforce explicit target origin, redaction, trusted flash evidence source, bounded duration, and bounded frame capture. [VERIFIED: scripts/phase17-live-http-api-smoke.sh] [VERIFIED: scripts/phase17-websocket-capture.mjs] |
| `tools/parity` | repo-owned Rust tool | Checklist validation and no-overclaim enforcement. | It rejects safety-critical `verified` rows without hardware evidence and active safety-control `verified` rows without `hardware-regression`. [VERIFIED: tools/parity/src/main.rs] |

### Supporting

| Tool / Library | Version | Purpose | When to Use |
| --- | --- | --- | --- |
| Node.js | v24.13.0 | Runs `scripts/phase17-websocket-capture.mjs`. | Use for bounded WebSocket frame capture instead of writing a new WebSocket client. [VERIFIED: scripts/phase17-websocket-capture.mjs] [VERIFIED: env probe `node --version`] |
| `curl` | 8.7.1 | HTTP route probes. | Use only with explicit `DEVICE_URL` or trusted flash evidence-derived target. [VERIFIED: scripts/phase14-live-telemetry.sh] [VERIFIED: scripts/phase17-live-http-api-smoke.sh] [VERIFIED: env probe `curl --version`] |
| `jq` | 1.7.1 | JSON artifact inspection/redaction checks in shell flows. | Use for deterministic artifact checks when shell helpers handle JSON. [VERIFIED: env probe `jq --version`] |
| `crates/bitaxe-safety` | workspace crate | Pure voltage, thermal, fault, self-test, and watchdog decisions. | Use for decisions and tests; firmware adapters should stay thin. [VERIFIED: crates/bitaxe-safety/src/power.rs] [VERIFIED: crates/bitaxe-safety/src/thermal.rs] |
| `crates/bitaxe-api` | workspace crate | API snapshot, wire model, and live telemetry envelope behavior. | Use for API/WebSocket field expectations and fixture tests. [VERIFIED: crates/bitaxe-api/src/snapshot.rs] [VERIFIED: crates/bitaxe-api/src/telemetry.rs] |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
| --- | --- | --- |
| Phase 14 allow-manifest validator | A new broad hardware runner | Rejected by locked decision D-01; a broad runner would weaken per-surface gates and recovery evidence. [VERIFIED: .planning/phases/20-active-safety-hardware-telemetry-evidence/20-CONTEXT.md] |
| `just detect-ultra205` | Manual port selection or network discovery | Rejected by locked decision D-02 and AGENTS; detector and board-info are mandatory before live hardware work. [VERIFIED: AGENTS.md] [VERIFIED: scripts/detect-ultra205.sh] |
| Phase 17 WebSocket helper | New custom WebSocket capture script | Existing helper already bounds target, duration, frame count, and redaction; new code adds risk without a known gap. [VERIFIED: scripts/phase17-websocket-capture.mjs] |
| `tools/parity` evidence semantics | Hand-edited checklist status without validation | Current parity tool already encodes safety-critical hardware evidence requirements; bypassing it risks overclaim. [VERIFIED: tools/parity/src/main.rs] |

**Installation:** No new external package is recommended for Phase 20 planning. Use the existing repo toolchain and `just doctor`/`just bootstrap-esp` only when the implementation environment needs dependency setup. [VERIFIED: Justfile] [VERIFIED: AGENTS.md]

**Version verification:** Versions above were verified locally with `just --version`, `bazel --version`, `cargo --version`, `rustc --version`, `espflash --version`, `node --version`, `curl --version`, and `jq --version`. [VERIFIED: env probes]

## Architecture Patterns

### Recommended Evidence Structure

```text
docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/
├── summary.md                         # final exact-claim matrix, commands, redaction, non-claims
├── safe-baseline.md                   # detector, board-info, package identity, pre/post safe state
├── active-power-voltage.md            # INA260/DS4432U evidence or blocked active-control record
├── active-thermal-fan.md              # thermal/fan RPM/fan duty/fault evidence or blockers
├── self-test-watchdog-load.md         # self-test submodes, watchdog/load proof or blockers
├── runtime-display-input.md           # runtime display/input proof or deferred boundary
├── failure-paths.md                   # fault stimulus records, restore paths, final safe-state markers
├── live-api-websocket-telemetry.md    # API/WS fields correlated with hardware observations
├── redaction-review.md                # committed-evidence redaction checklist and reviewer
└── raw/                               # local-only or redacted micro-artifacts per plan policy
```

This structure follows the component pack split requested in Phase 20 context and the Phase 14 evidence component pattern. [VERIFIED: .planning/phases/20-active-safety-hardware-telemetry-evidence/20-CONTEXT.md] [VERIFIED: docs/parity/evidence/phase-14-safety-hardware-evidence-completion.md]

### Pattern 1: Safety Allow Manifest First

**What:** Each active or live probe should begin from a JSON allow manifest validated by `tools/parity` before touching hardware or network routes. [VERIFIED: tools/parity/src/safety_allow.rs]

**When to use:** Use it for active voltage, fan/thermal, self-test hardware, load/stress, runtime display/input, failure paths, and live API/WebSocket telemetry. [VERIFIED: .planning/phases/20-active-safety-hardware-telemetry-evidence/20-CONTEXT.md]

**Planning note:** Current allowed surfaces include `safe-baseline`, `power-telemetry`, `voltage-control`, `thermal-fan`, `self-test-watchdog-load`, `display-input`, `live-api-websocket-telemetry`, and `parity-redaction`. The Phase 20 preferred pack name `failure-paths` is not currently in `ALLOWED_SURFACES`, so the plan must either map failure-path probes to existing surfaces or extend validator/tests. [VERIFIED: tools/parity/src/safety_allow.rs] [VERIFIED: .planning/phases/20-active-safety-hardware-telemetry-evidence/20-CONTEXT.md]

**Example:**

```json
{
  "board": "205",
  "port": "/dev/cu.usbmodemEXAMPLE",
  "detector_command": "just detect-ultra205",
  "detector_port": "/dev/cu.usbmodemEXAMPLE",
  "board_info_command": "espflash board-info --chip esp32s3 --port /dev/cu.usbmodemEXAMPLE --non-interactive",
  "board_info_status": "passed",
  "surface": "thermal-fan",
  "claim_tier": "bounded-actuation",
  "evidence_class": "hardware-regression",
  "abort_conditions": [
    "detector_mismatch",
    "board_info_failure",
    "missing_safe_state_marker"
  ],
  "post_action_safe_state_markers": [
    "safe_state: mining=disabled",
    "hardware_control=disabled"
  ]
}
```

Source: `SafetyAllowManifest` fields and required active conditions. [VERIFIED: tools/parity/src/safety_allow.rs]

### Pattern 2: Exact Claim Promotion

**What:** Promote only the exact subclaim proven by the artifact, and leave broader active-control rows below verified when the artifact proves only read-only, unavailable, startup, or route-shape behavior. [VERIFIED: .planning/phases/20-active-safety-hardware-telemetry-evidence/20-CONTEXT.md]

**When to use:** Use it for `docs/parity/checklist.md` updates and `.planning/REQUIREMENTS.md` traceability updates after evidence is complete. [VERIFIED: docs/parity/checklist.md] [VERIFIED: .planning/REQUIREMENTS.md]

**Example:** A live `/api/system/info` body with `power=0` from unavailable safety telemetry can support safe-unavailable API projection, but cannot verify INA260 fresh power capture or DS4432U voltage control. [VERIFIED: crates/bitaxe-api/src/snapshot.rs] [VERIFIED: crates/bitaxe-api/src/wire.rs] [VERIFIED: docs/parity/evidence/phase-14-safety-hardware-evidence-completion/power-telemetry.md]

### Pattern 3: Functional Core, Imperative Hardware Shell

**What:** Keep voltage, thermal, fault, self-test, and watchdog decisions in `crates/bitaxe-safety`; keep ESP-IDF, I2C, GPIO/PWM, display, serial, HTTP/WebSocket, and active effects in firmware adapters. [VERIFIED: standards/core/architecture.md] [VERIFIED: crates/bitaxe-safety/src/power.rs] [VERIFIED: firmware/bitaxe/src/safety_adapter.rs]

**When to use:** Use this split for any Phase 20 code change, especially if adding compile-gated diagnostic stimuli. [VERIFIED: .planning/phases/20-active-safety-hardware-telemetry-evidence/20-CONTEXT.md] [VERIFIED: standards/languages/rust.md]

**Example:** `VoltageControllerInputs::plan()` can decide whether a DS4432U write is allowed, while `firmware/bitaxe/src/safety_adapter/power.rs` owns the device address/register constants and currently suppresses the effect. [VERIFIED: crates/bitaxe-safety/src/power.rs] [VERIFIED: firmware/bitaxe/src/safety_adapter/power.rs]

### Pattern 4: Live Telemetry Correlation

**What:** Capture live HTTP and WebSocket data from an explicit trusted target, then correlate the fields/frames with hardware observations and pre/post safe-state markers. [VERIFIED: .planning/phases/20-active-safety-hardware-telemetry-evidence/20-CONTEXT.md] [VERIFIED: scripts/phase17-live-http-api-smoke.sh] [VERIFIED: scripts/phase17-websocket-capture.mjs]

**When to use:** Use it for SAFE-07 and any API/WS evidence used to support active safety telemetry claims. [VERIFIED: .planning/REQUIREMENTS.md]

**Example command shape:**

```bash
node scripts/phase17-websocket-capture.mjs \
  --device-url "$DEVICE_URL" \
  --path /api/ws/live \
  --duration-ms 10000 \
  --max-frames 5 \
  --out-dir docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/live-api-websocket-telemetry
```

Source: Phase 17 helper accepts explicit `--device-url`, bounded duration, bounded frame count, and redacted output. [VERIFIED: scripts/phase17-websocket-capture.mjs]

### Anti-Patterns to Avoid

- **Broad unsafe runner:** Rejected by D-01; use surface-scoped allow manifests instead. [VERIFIED: .planning/phases/20-active-safety-hardware-telemetry-evidence/20-CONTEXT.md]
- **Manual port workaround:** Rejected by AGENTS and D-02 unless it comes through the detector-gated repo path. [VERIFIED: AGENTS.md] [VERIFIED: scripts/detect-ultra205.sh]
- **Route-presence-as-telemetry-proof:** A 404/no-upgrade/route exists result does not prove live safety telemetry freshness or cadence. [VERIFIED: .planning/phases/20-active-safety-hardware-telemetry-evidence/20-CONTEXT.md] [VERIFIED: docs/parity/evidence/phase-14-safety-hardware-evidence-completion/live-api-websocket-telemetry.md]
- **Startup display as runtime display/input proof:** Startup SSD1306 evidence and `display_input_status=runtime_gap` are supporting breadcrumbs only. [VERIFIED: firmware/bitaxe/src/display_adapter.rs] [VERIFIED: docs/parity/evidence/phase-14-safety-hardware-evidence-completion/display-input.md]
- **Supervisor-yield-as-load proof:** The current watchdog marker proves supervisor shell/yield only, not bounded workload or watchdog recovery. [VERIFIED: firmware/bitaxe/src/safety_adapter/watchdog.rs] [VERIFIED: docs/parity/evidence/phase-14-safety-hardware-evidence-completion/self-test-watchdog-load.md]

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
| --- | --- | --- | --- |
| USB board detection | Manual serial scanning or ad hoc port selection | `just detect-ultra205` | Repo rules require exactly one likely ESP USB serial port and board-info pass. [VERIFIED: AGENTS.md] [VERIFIED: scripts/detect-ultra205.sh] |
| Safety evidence authorization | Custom shell conditionals for active probes | `tools/parity` safety allow manifests | Existing validator already enforces board, detector, board-info, package identity, claim tier, recovery, abort, and safe-state rules. [VERIFIED: tools/parity/src/safety_allow.rs] |
| WebSocket capture | A new WebSocket client | `scripts/phase17-websocket-capture.mjs` | Existing helper bounds duration/frames and redacts output. [VERIFIED: scripts/phase17-websocket-capture.mjs] |
| Checklist safety semantics | Manual checked/verified status editing | `just parity` / `tools/parity` | Existing parity report rejects invalid safety-critical verified claims. [VERIFIED: Justfile] [VERIFIED: tools/parity/src/main.rs] |
| Active I2C pokes | Raw `i2cset`, `esptool.py`, or arbitrary firmware shell | Compile-gated, bounded repo-owned diagnostic route if truly needed | Phase 20 decisions require bounded gates, tests, redaction review, recovery, and post-action safe-state checks for new active routes. [VERIFIED: .planning/phases/20-active-safety-hardware-telemetry-evidence/20-CONTEXT.md] |
| Network target discovery | Private subnet scans, ARP, mDNS, or router inference | Explicit `DEVICE_URL` or trusted flash evidence target | Phase 17 helpers use explicit target gates and target-lock policy; Phase 20 context rejects stale/implicit route proof. [VERIFIED: scripts/phase17-live-http-api-smoke.sh] [VERIFIED: .planning/phases/17-live-http-api-and-static-evidence/17-CONTEXT.md] |
| JSON redaction by search/replace | Ad hoc string rewriting | Existing helper redaction paths plus reviewed redaction checklist | Phase 17 and Phase 20 require redaction of URLs, IP/MAC/SSID, credentials, API bodies, WS frames, commands, logs, and observations. [VERIFIED: scripts/phase17-live-http-api-smoke.sh] [VERIFIED: .planning/phases/20-active-safety-hardware-telemetry-evidence/20-CONTEXT.md] |

**Key insight:** Phase 20 success comes from bounded evidence and no-overclaim enforcement, not from broad hardware access. The current repository already has the safety and telemetry scaffolding; the plan should add the smallest missing proof path per surface. [VERIFIED: .planning/phases/20-active-safety-hardware-telemetry-evidence/20-CONTEXT.md] [VERIFIED: docs/parity/evidence/phase-14-safety-hardware-evidence-completion.md]

## Current Surface Inventory

| Surface | Current State | Planning Implication |
| --- | --- | --- |
| Power telemetry | Phase 14 records blocker for fresh INA260 power/current/voltage; API snapshot preserves unavailable/zero-compatible fields unless hardware-verified fresh readings exist. [VERIFIED: docs/parity/evidence/phase-14-safety-hardware-evidence-completion/power-telemetry.md] [VERIFIED: crates/bitaxe-api/src/snapshot.rs] | Plan for fresh read evidence or keep SAFE-07 power/current/voltage below verified. |
| Voltage control | Firmware power adapter currently suppresses DS4432U voltage writes; pure logic allows write only with hardware evidence and armed actuation mode. [VERIFIED: firmware/bitaxe/src/safety_adapter/power.rs] [VERIFIED: crates/bitaxe-safety/src/power.rs] | Active voltage rows need compile-gated bounded diagnostic route plus hardware-regression, or blocked evidence. |
| Thermal and fan telemetry | Firmware thermal adapter currently reports unavailable and suppresses fan writes; pure thermal logic covers fan/PID/overheat decisions. [VERIFIED: firmware/bitaxe/src/safety_adapter/thermal.rs] [VERIFIED: crates/bitaxe-safety/src/thermal.rs] | Plan separately for thermal read, fan RPM, fan duty effect, and fan/thermal fault stimuli. |
| Fault paths | Pure fault classifier emits safe-state effects for overheat, fan, power, thermal, and ASIC faults. [VERIFIED: crates/bitaxe-safety/src/fault.rs] | Active failure-path evidence needs named stimulus, expected fault, abort, restore, API/log/WS projection, and final safe state. |
| Self-test | Pure self-test lifecycle is modeled, but hardware self-test submodes are below verified. [VERIFIED: crates/bitaxe-safety/src/self_test.rs] [VERIFIED: docs/parity/evidence/phase-14-safety-hardware-evidence-completion/self-test-watchdog-load.md] | Plan for exact submode proof or keep self-test hardware rows below verified. |
| Watchdog/load | Firmware logs supervisor start/yield markers; Phase 14 did not prove bounded load stress. [VERIFIED: firmware/bitaxe/src/safety_adapter/watchdog.rs] [VERIFIED: scripts/phase14-self-test-watchdog-load.sh] | Plan bounded workload with pass/fail criteria and API/WS responsiveness, or keep load claims below verified. |
| Display/input | Firmware renders startup SSD1306 text and logs runtime display/input gap marker. [VERIFIED: firmware/bitaxe/src/display_adapter.rs] | Runtime display/input remains below verified without real runtime route and physical/log/API/WS observation. |
| Live API/WebSocket telemetry | Phase 17 proved live HTTP/API and WebSocket route/frame behavior, but not active safety telemetry. [VERIFIED: docs/parity/evidence/phase-17-live-http-api-and-static-evidence/summary.md] | Phase 20 must correlate live safety fields/frames with hardware observations and safe-state markers. |
| Parity validation | Current parity rejects safety-critical verified rows without hardware evidence and active rows without `hardware-regression`. [VERIFIED: tools/parity/src/main.rs] | Preserve or extend tests when checklist semantics change. |

## Common Pitfalls

### Pitfall 1: Promoting Hardware-Smoke Evidence To Active Control Verification

**What goes wrong:** A read-only observation or safe-unavailable artifact is cited as proof of DS4432U writes, fan duty effects, fault stimuli, self-test hardware, runtime display/input, or load stress. [VERIFIED: .planning/phases/20-active-safety-hardware-telemetry-evidence/20-CONTEXT.md]

**Why it happens:** Phase 14 wrappers produce useful smoke evidence, but the active surfaces remain blocked. [VERIFIED: docs/parity/evidence/phase-14-safety-hardware-evidence-completion.md]

**How to avoid:** Require `hardware-regression` for active safety-control and failure-path rows, and run `just parity` after checklist edits. [VERIFIED: tools/parity/src/main.rs] [VERIFIED: Justfile]

**Warning signs:** Checklist row changes to `verified` while evidence path contains `safe-unavailable`, `unsupported-pending`, `runtime_gap`, or `write_suppressed`. [VERIFIED: docs/parity/evidence/phase-14-safety-hardware-evidence-completion.md] [VERIFIED: firmware/bitaxe/src/safety_adapter/power.rs]

### Pitfall 2: Stale Package Or Target Identity

**What goes wrong:** API/WebSocket or serial evidence comes from a firmware package that does not match the source commit or reference commit cited in the evidence. [VERIFIED: AGENTS.md] [VERIFIED: tools/parity/src/safety_allow.rs]

**Why it happens:** Live targets can remain flashed from a prior phase or earlier package. [VERIFIED: .planning/phases/20-active-safety-hardware-telemetry-evidence/20-CONTEXT.md]

**How to avoid:** Bind every run to package manifest, source commit, reference commit, detector port, and board-info pass. [VERIFIED: AGENTS.md] [VERIFIED: tools/parity/src/safety_allow.rs]

**Warning signs:** Evidence references `DEVICE_URL` but lacks package manifest, source commit, reference commit, or trusted flash evidence. [VERIFIED: scripts/phase17-live-http-api-smoke.sh]

### Pitfall 3: Route Shape Mistaken For Live Telemetry

**What goes wrong:** A route status, no-upgrade response, or stale API body is treated as live safety telemetry proof. [VERIFIED: .planning/phases/20-active-safety-hardware-telemetry-evidence/20-CONTEXT.md]

**Why it happens:** Route existence is easier to capture than correlated hardware observations and frame cadence. [VERIFIED: scripts/phase14-live-telemetry.sh]

**How to avoid:** Pair `/api/system/info` and `/api/ws/live` with hardware observations and safe-state markers before and after active checks. [VERIFIED: scripts/phase17-websocket-capture.mjs] [VERIFIED: .planning/phases/20-active-safety-hardware-telemetry-evidence/20-CONTEXT.md]

**Warning signs:** Evidence has HTTP status or WebSocket path checks but no frame count, body fields, hardware reading, or safe-state marker. [VERIFIED: scripts/phase14-live-telemetry.sh]

### Pitfall 4: Recovery Path Missing From Active Stimulus

**What goes wrong:** A fault, voltage, fan, self-test, load, or display/input probe cannot demonstrate final safe state after execution. [VERIFIED: .planning/phases/20-active-safety-hardware-telemetry-evidence/20-CONTEXT.md]

**Why it happens:** Active verification can be tempting to run before exact abort/recovery criteria are written. [VERIFIED: AGENTS.md]

**How to avoid:** Plan the recovery command, restore package, abort conditions, and post-action safe-state markers before enabling the run. [VERIFIED: AGENTS.md] [VERIFIED: tools/parity/src/safety_allow.rs]

**Warning signs:** Manifest lacks required abort conditions, lacks `safe_state: mining=disabled`, lacks `hardware_control=disabled`, or has an empty `recovery_steps` array for an active tier. [VERIFIED: tools/parity/src/safety_allow.rs]

### Pitfall 5: Runtime Display/Input Overclaim

**What goes wrong:** Startup display text is used to claim runtime display or input parity. [VERIFIED: docs/parity/evidence/phase-14-safety-hardware-evidence-completion/display-input.md]

**Why it happens:** `display_adapter.rs` has startup rendering and a runtime gap marker, but no full runtime display/input proof. [VERIFIED: firmware/bitaxe/src/display_adapter.rs]

**How to avoid:** Keep runtime display/input below verified unless the phase adds and exercises a bounded runtime route with physical or log/API/WebSocket observation. [VERIFIED: .planning/phases/20-active-safety-hardware-telemetry-evidence/20-CONTEXT.md]

**Warning signs:** Evidence only contains `display_status=startup_text_rendered` or `display_input_status=runtime_gap`. [VERIFIED: firmware/bitaxe/src/display_adapter.rs]

### Pitfall 6: Redaction After Citation

**What goes wrong:** Checklist or requirements cite raw logs or JSON that contain `DEVICE_URL`, IP/MAC/SSID, Wi-Fi/pool credentials, worker secrets, tokens, NVS values, or terminal secrets. [VERIFIED: AGENTS.md] [VERIFIED: .planning/phases/20-active-safety-hardware-telemetry-evidence/20-CONTEXT.md]

**Why it happens:** Hardware evidence often mixes useful observations with local network identity. [VERIFIED: AGENTS.md]

**How to avoid:** Run redaction review before committing evidence and cite only redacted/allowlisted artifacts. [VERIFIED: .planning/phases/20-active-safety-hardware-telemetry-evidence/20-CONTEXT.md] [VERIFIED: scripts/phase17-live-http-api-smoke.sh]

**Warning signs:** Evidence artifacts contain raw `http://`, private IPs, MAC address patterns, SSID fields, pool URLs, worker names, or credential file content. [VERIFIED: AGENTS.md]

## Code Examples

### Wrapper Flow For A Phase 20 Probe

```bash
#!/usr/bin/env bash
set -euo pipefail

# Source pattern: Phase 14 wrappers validate allow manifests before probes.
# [VERIFIED: scripts/phase14-power-voltage.sh]
# [VERIFIED: tools/parity/src/safety_allow.rs]

manifest="$1"
evidence_dir="$2"

bazel run //tools/parity:report -- \
  safety-allow --manifest "$manifest"

just detect-ultra205 | tee "$evidence_dir/detect-ultra205.log"

# Active stimulus belongs here only if the manifest tier, bounded inputs,
# abort conditions, recovery steps, and safe-state markers validate.
```

### Parity Guard For Active Safety Rows

```rust
// Source: tools/parity active safety-control rows require hardware-regression.
// [VERIFIED: tools/parity/src/main.rs]
if row.status == RowStatus::Verified
    && is_active_safety_control(&row.id, &row.description)
    && !row.evidence.iter().any(|evidence| evidence == "hardware-regression")
{
    // report invalid verified claim
}
```

### API Snapshot Safety Telemetry Boundary

```rust
// Source: unavailable or unverified safety telemetry is projected as
// zero-compatible unavailable fields, not as fresh hardware readings.
// [VERIFIED: crates/bitaxe-api/src/snapshot.rs]
let snapshot = ApiSnapshot::safe_ultra_205()
    .with_safety_telemetry(SafeTelemetrySnapshot::unavailable(
        "safety_telemetry_unavailable",
    ));
```

### Live WebSocket Capture Command

```bash
node scripts/phase17-websocket-capture.mjs \
  --device-url "$DEVICE_URL" \
  --path /api/ws/live \
  --duration-ms 10000 \
  --max-frames 5 \
  --out-dir docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/live-api-websocket-telemetry
```

Source: bounded WebSocket helper supports explicit target, path, duration, max frames, and output directory. [VERIFIED: scripts/phase17-websocket-capture.mjs]

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
| --- | --- | --- | --- |
| Phase 14 wrappers recorded component blockers and smoke evidence for safety surfaces. [VERIFIED: docs/parity/evidence/phase-14-safety-hardware-evidence-completion.md] | Phase 20 should reuse the wrappers but add active or correlated evidence only where safe bounded routes exist. [VERIFIED: .planning/phases/20-active-safety-hardware-telemetry-evidence/20-CONTEXT.md] | Phase 20 scope, 2026-07-03. [VERIFIED: .planning/ROADMAP.md] | Planner should avoid duplicating Phase 14 and focus on remaining active/live proof gaps. |
| Phase 14 live telemetry wrapper could not complete without `DEVICE_URL`. [VERIFIED: docs/parity/evidence/phase-14-safety-hardware-evidence-completion/live-api-websocket-telemetry.md] | Phase 17 added explicit-target live HTTP and WebSocket capture patterns. [VERIFIED: scripts/phase17-live-http-api-smoke.sh] [VERIFIED: scripts/phase17-websocket-capture.mjs] | Phase 17 evidence. [VERIFIED: docs/parity/evidence/phase-17-live-http-api-and-static-evidence/summary.md] | Planner should compose Phase 20 live safety telemetry with Phase 17 helpers rather than rebuilding them. |
| Safety checklist could be manually overclaimed if no validator checked row semantics. [VERIFIED: docs/adr/0012-parity-verification-evidence.md] | `tools/parity` rejects safety-critical verified rows without hardware evidence and active rows without `hardware-regression`. [VERIFIED: tools/parity/src/main.rs] | Current codebase state. [VERIFIED: tools/parity/src/main.rs] | Phase 20 final verification must keep `just parity` passing. |
| Firmware active safety adapters suppress voltage/fan effects by default. [VERIFIED: firmware/bitaxe/src/safety_adapter/power.rs] [VERIFIED: firmware/bitaxe/src/safety_adapter/thermal.rs] | Any new active probe must be compile-gated, bounded, tested, and non-production-exposed. [VERIFIED: .planning/phases/20-active-safety-hardware-telemetry-evidence/20-CONTEXT.md] | Phase 20 decision D-07. [VERIFIED: .planning/phases/20-active-safety-hardware-telemetry-evidence/20-CONTEXT.md] | Planner must decide per surface whether to implement a safe diagnostic route or preserve blocked evidence. |

**Deprecated/outdated for this phase:**

- Treating `hardware-smoke` as enough for active safety-control rows is invalid; active safety-control rows require `hardware-regression`. [VERIFIED: tools/parity/src/main.rs] [VERIFIED: .planning/phases/20-active-safety-hardware-telemetry-evidence/20-CONTEXT.md]
- Using network scans or implicit target discovery is invalid; live telemetry needs explicit `DEVICE_URL` or trusted flash evidence-derived target. [VERIFIED: scripts/phase17-live-http-api-smoke.sh] [VERIFIED: AGENTS.md]
- Editing `reference/esp-miner` is invalid; it is read-only reference evidence. [VERIFIED: AGENTS.md] [VERIFIED: docs/adr/0005-read-only-reference-implementation.md]

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
| --- | --- | --- | --- |
| None | All planning-relevant claims in this research were verified from local project files, source code, or environment probes during this session. | All sections | No user confirmation needed for factual basis; active hardware run choices still require phase plan gates. |

## Open Questions (RESOLVED)

1. **Will Phase 20 add compile-gated diagnostic routes for any active effects?**
   - RESOLVED: The chosen plan set does not add active voltage, fan-duty, self-test, load, runtime input/display, or fault-stimulus diagnostic routes in Phase 20. Plans 03 and 04 record those surfaces as `unsupported-pending` or blocked evidence when no production-safe bounded route exists, while keeping read-only observations separate.
   - What we know: current firmware adapters suppress DS4432U writes and fan writes, and Phase 20 decisions allow new active routes only if compile-gated, bounded, tested, and non-production-exposed. [VERIFIED: firmware/bitaxe/src/safety_adapter/power.rs] [VERIFIED: firmware/bitaxe/src/safety_adapter/thermal.rs] [VERIFIED: .planning/phases/20-active-safety-hardware-telemetry-evidence/20-CONTEXT.md]
   - What's unclear: which surfaces are safe and valuable enough to implement active diagnostic triggers in Phase 20.
   - Recommendation: planner should create separate tasks for route design decisions per surface and allow blocked evidence where the route is not justified.

2. **How should the `failure-paths` evidence pack map into `tools/parity` safety surfaces?**
   - RESOLVED: Plan 01 extends `tools/parity/src/safety_allow.rs` with a first-class `failure-paths` surface and tests. Plan 04 consumes that surface through a Phase 20 failure-path wrapper that records blocked evidence by default and never runs a live fault stimulus.
   - What we know: Phase 20 prefers a `failure-paths` pack, but current `ALLOWED_SURFACES` does not list `failure-paths`. [VERIFIED: .planning/phases/20-active-safety-hardware-telemetry-evidence/20-CONTEXT.md] [VERIFIED: tools/parity/src/safety_allow.rs]
   - What's unclear: whether to add `failure-paths` to the validator or represent each fault under `thermal-fan`, `voltage-control`, `self-test-watchdog-load`, or another existing surface.
   - Recommendation: planner should add a Wave 0 validator/test task if a standalone `failure-paths` surface is used.

3. **Is a trusted live target available during execution?**
   - RESOLVED: Plan 02 treats `just detect-ultra205` and board-info as the hardware gate and writes blocked safe-baseline evidence if the gate fails. Plan 05 uses only an explicit `DEVICE_URL` or trusted origin-only target lock for HTTP/WebSocket telemetry, and writes blocked telemetry evidence when no trusted target exists.
   - What we know: Phase 20 must start with `just detect-ultra205`, and live HTTP/WebSocket capture requires explicit `DEVICE_URL` or trusted flash evidence. [VERIFIED: AGENTS.md] [VERIFIED: scripts/detect-ultra205.sh] [VERIFIED: scripts/phase17-live-http-api-smoke.sh]
   - What's unclear: this research did not run hardware detection or network probes.
   - Recommendation: implementation plan should treat detector/target availability as a first hardware gate and write blocked evidence if it fails.

4. **Which exact bounded inputs are safe for active probes?**
   - RESOLVED: The chosen plan set does not select active voltage deltas, fan-duty percentages, workload durations, fault stimuli, self-test submodes, or runtime input actions. Those inputs are explicitly `not-run` with blocked/deferred evidence. The only bounded live capture inputs selected are read-only telemetry capture limits: `/api/ws/live`, `duration-ms 10000`, and `max-frames 5`.
   - What we know: manifests must include bounded inputs, abort conditions, recovery steps, and safe-state markers before active probes. [VERIFIED: tools/parity/src/safety_allow.rs] [VERIFIED: .planning/phases/20-active-safety-hardware-telemetry-evidence/20-CONTEXT.md]
   - What's unclear: the exact fan duty percent, voltage setpoint delta, workload duration, fault stimulus, self-test submode, and display/input action are not chosen in research.
   - Recommendation: planner should make those values explicit in the plan or mark the related evidence as blocked.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
| --- | --- | --- | --- | --- |
| `just` | Detector, package, flash/monitor, parity, verification commands | yes | 1.48.0 | None; repo command surface depends on it. [VERIFIED: env probe `just --version`] |
| Bazel | Script tests, Rust tool targets, parity tooling | yes | 9.1.1 | None for canonical repo verification. [VERIFIED: env probe `bazel --version`] |
| Cargo | Rust crate tests/builds | yes | 1.88.0-nightly | None for changed Rust checks. [VERIFIED: env probe `cargo --version`] |
| Rustc | Rust crate builds/tests | yes | 1.88.0-nightly | None for changed Rust checks. [VERIFIED: env probe `rustc --version`] |
| `espflash` | Detector board-info, flash/monitor hardware evidence | yes | 4.0.1 | No fallback for hardware evidence; missing `espflash` blocks live hardware runs. [VERIFIED: env probe `espflash --version`] |
| Node.js | WebSocket capture helper | yes | v24.13.0 | Could use another WS client only if helper cannot run; current helper should be preferred. [VERIFIED: env probe `node --version`] |
| `curl` | HTTP route probes | yes | 8.7.1 | None needed. [VERIFIED: env probe `curl --version`] |
| `jq` | JSON inspection/redaction checks | yes | 1.7.1 | Rust/Python structured parsers could be used if missing, but not needed. [VERIFIED: env probe `jq --version`] |
| Ultra 205 over USB | Live hardware evidence | not probed in research | unknown | Blocked evidence if `just detect-ultra205` fails during execution. [VERIFIED: AGENTS.md] |
| Explicit `DEVICE_URL` or trusted flash evidence target | Live API/WebSocket safety telemetry | not probed in research | unknown | Blocked evidence if unavailable during execution. [VERIFIED: scripts/phase17-live-http-api-smoke.sh] |

**Missing dependencies with no fallback:** None of the CLI dependencies probed during research are missing; live board and live target availability were intentionally not probed and must be gated during implementation. [VERIFIED: env probes] [VERIFIED: AGENTS.md]

**Missing dependencies with fallback:** `jq` has possible structured-parser fallbacks, but it is available locally. [VERIFIED: env probe `jq --version`]

## Validation Architecture

Workflow Nyquist validation is enabled in `.planning/config.json`, so Phase 20 planning must include validation tasks. [VERIFIED: .planning/config.json]

### Test Framework

| Property | Value |
| --- | --- |
| Framework | Bazel tests for scripts/tools, Cargo tests for Rust crates, `just` aggregate commands. [VERIFIED: scripts/BUILD.bazel] [VERIFIED: Cargo.toml] [VERIFIED: Justfile] |
| Config file | `scripts/BUILD.bazel`, `Cargo.toml`, `Justfile`. [VERIFIED: scripts/BUILD.bazel] [VERIFIED: Cargo.toml] [VERIFIED: Justfile] |
| Quick run command | `bazel test //scripts:phase14_power_voltage_test //scripts:phase14_thermal_fan_test //scripts:phase14_self_test_watchdog_load_test //scripts:phase14_display_input_test //scripts:phase14_live_telemetry_test` for unchanged Phase 14 wrapper compatibility. [VERIFIED: scripts/BUILD.bazel] |
| Full suite command | `just test && just parity && just verify-reference` plus changed Rust checks required by the phase plan. [VERIFIED: Justfile] [VERIFIED: .planning/phases/20-active-safety-hardware-telemetry-evidence/20-CONTEXT.md] |

### Phase Requirements -> Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
| --- | --- | --- | --- | --- |
| SAFE-01 | Voltage/power decisions fail closed and active voltage claims require hardware-regression. [VERIFIED: .planning/REQUIREMENTS.md] | unit + parity + hardware evidence | `cargo test -p bitaxe-safety --all-features power` and `cargo test -p bitaxe-parity --all-features` or Bazel parity target equivalent; active run requires plan-gated hardware command. [VERIFIED: crates/bitaxe-safety/src/power.rs] [VERIFIED: tools/parity/src/main.rs] | Existing pure/parity files yes; Phase 20 hardware wrapper likely needed. |
| SAFE-02 | Thermal/fan readings, duty, RPM, and failures are exact-claim evidenced. [VERIFIED: .planning/REQUIREMENTS.md] | unit + wrapper + hardware evidence | `cargo test -p bitaxe-safety --all-features thermal` and `cargo test -p bitaxe-safety --all-features fault`; wrapper test if Phase 20 adds thermal/fan helper. [VERIFIED: crates/bitaxe-safety/src/thermal.rs] [VERIFIED: crates/bitaxe-safety/src/fault.rs] | Existing pure files yes; Phase 20 wrapper likely needed. |
| SAFE-03 | PID/thermal decisions remain tested before effects. [VERIFIED: .planning/REQUIREMENTS.md] | unit | `cargo test -p bitaxe-safety --all-features thermal` | Existing source yes. |
| SAFE-04 | Fault paths enter safe states and expose visible status. [VERIFIED: .planning/REQUIREMENTS.md] | unit + hardware-regression + API/WS correlation | `cargo test -p bitaxe-safety --all-features fault`; plan-gated fault probe; live capture helper for projection. [VERIFIED: crates/bitaxe-safety/src/fault.rs] [VERIFIED: scripts/phase17-websocket-capture.mjs] | Existing pure/live helpers yes; Phase 20 fault wrapper likely needed. |
| SAFE-05 | Self-test lifecycle and user-visible result reporting. [VERIFIED: .planning/REQUIREMENTS.md] | unit + hardware-regression or blocked evidence | `cargo test -p bitaxe-safety --all-features self_test`; plan-gated self-test probe if route exists. [VERIFIED: crates/bitaxe-safety/src/self_test.rs] | Existing pure file yes; hardware probe route uncertain. |
| SAFE-06 | Runtime display/input status preserved or deferred. [VERIFIED: .planning/REQUIREMENTS.md] | wrapper + hardware observation or blocked evidence | Existing `bazel test //scripts:phase14_display_input_test`; Phase 20 runtime proof test if helper changes. [VERIFIED: scripts/BUILD.bazel] [VERIFIED: firmware/bitaxe/src/display_adapter.rs] | Existing startup/gap helper yes; runtime proof helper uncertain. |
| SAFE-07 | Power/current/voltage/fan/temp telemetry captured where exposed. [VERIFIED: .planning/REQUIREMENTS.md] | hardware smoke/regression + API/WS correlation | `node scripts/phase17-websocket-capture.mjs ...`; HTTP helper command with explicit target; hardware observation command per surface. [VERIFIED: scripts/phase17-websocket-capture.mjs] [VERIFIED: scripts/phase17-live-http-api-smoke.sh] | Live helpers yes; Phase 20 correlation docs/wrappers needed. |
| SAFE-08 | Safety-critical verified rows require hardware evidence. [VERIFIED: .planning/REQUIREMENTS.md] | parity | `just parity` | Existing yes. [VERIFIED: Justfile] [VERIFIED: tools/parity/src/main.rs] |
| SAFE-09 | Watchdog responsiveness under bounded load. [VERIFIED: .planning/REQUIREMENTS.md] | unit + hardware/load evidence + API/WS responsiveness | `cargo test -p bitaxe-safety --all-features watchdog`; plan-gated bounded load probe if route exists. [VERIFIED: crates/bitaxe-safety/src/watchdog.rs] | Existing pure/watchdog shell yes; load probe uncertain. |
| EVD-05 | Verification layers include appropriate automated and hardware evidence. [VERIFIED: .planning/REQUIREMENTS.md] | workflow | `just test && just parity && just verify-reference`; redaction/lifecycle checks in Phase 20 verification. [VERIFIED: Justfile] [VERIFIED: .planning/phases/20-active-safety-hardware-telemetry-evidence/20-CONTEXT.md] | Existing commands yes; Phase 20 verification artifact needed. |

### Sampling Rate

- **Per task commit:** Run the narrow affected script/Rust tests, plus `just parity` after checklist/evidence changes. [VERIFIED: standards/core/verification.md] [VERIFIED: Justfile]
- **Per wave merge:** Run all changed wrapper tests, changed Rust crate tests, `just parity`, and `just verify-reference`. [VERIFIED: .planning/phases/20-active-safety-hardware-telemetry-evidence/20-CONTEXT.md]
- **Phase gate:** Run `just test`, `just parity`, `just verify-reference`, redaction review, lifecycle validation, and every hardware/network command actually used before `20-VERIFICATION.md status: passed`. [VERIFIED: .planning/phases/20-active-safety-hardware-telemetry-evidence/20-CONTEXT.md]

### Wave 0 Gaps

- [ ] Add Phase 20 wrapper tests if new wrapper scripts are created; existing Phase 14 tests cover only current helper behavior. [VERIFIED: scripts/BUILD.bazel]
- [ ] Extend `tools/parity/src/safety_allow.rs` and tests if a standalone `failure-paths` surface is used. [VERIFIED: tools/parity/src/safety_allow.rs]
- [ ] Add redaction review artifact template for Phase 20 evidence before hardware runs. [VERIFIED: .planning/phases/20-active-safety-hardware-telemetry-evidence/20-CONTEXT.md]
- [ ] Add blocked-evidence templates for surfaces where no safe diagnostic route exists. [VERIFIED: docs/parity/evidence/phase-14-safety-hardware-evidence-completion.md]
- [ ] Confirm exact changed Rust check commands after planner chooses whether firmware/API/parity crates change. [VERIFIED: AGENTS.md] [VERIFIED: standards/core/verification.md]

## Security Domain

Security enforcement is enabled because `.planning/config.json` does not set `security_enforcement` to `false`. [VERIFIED: .planning/config.json]

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
| --- | --- | --- |
| V2 Authentication | no new auth | Phase 20 should not add authentication surfaces; evidence helpers must not read or expose credentials. [VERIFIED: .planning/phases/20-active-safety-hardware-telemetry-evidence/20-CONTEXT.md] [VERIFIED: AGENTS.md] |
| V3 Session Management | no | Phase 20 live telemetry probes use local device HTTP/WebSocket routes and do not introduce sessions. [VERIFIED: firmware/bitaxe/src/http_api.rs] |
| V4 Access Control | yes for diagnostics | Any new diagnostic trigger must be compile-gated, bounded, and impossible to expose accidentally in production flows. [VERIFIED: .planning/phases/20-active-safety-hardware-telemetry-evidence/20-CONTEXT.md] |
| V5 Input Validation | yes | Validate allow manifests, URL origin, port identity, bounded inputs, evidence class, claim tier, and safe-state markers before running probes. [VERIFIED: tools/parity/src/safety_allow.rs] [VERIFIED: scripts/phase17-websocket-capture.mjs] |
| V6 Cryptography | no custom crypto | No Phase 20 requirement needs custom cryptography; do not add it. [VERIFIED: .planning/ROADMAP.md] |
| V8 Data Protection | yes | Redact serial logs, JSON, API bodies, WebSocket frames, commands, `DEVICE_URL`, IP/MAC/SSID, credentials, tokens, NVS values, and terminal secrets before commit. [VERIFIED: AGENTS.md] [VERIFIED: .planning/phases/20-active-safety-hardware-telemetry-evidence/20-CONTEXT.md] |
| V9 Communications | yes | Use explicit target gates for local HTTP/WebSocket telemetry and avoid network scans or implicit discovery. [VERIFIED: scripts/phase17-live-http-api-smoke.sh] [VERIFIED: .planning/phases/17-live-http-api-and-static-evidence/17-CONTEXT.md] |
| V14 Configuration | yes | Keep diagnostic active routes compile-gated and non-production-exposed if added. [VERIFIED: .planning/phases/20-active-safety-hardware-telemetry-evidence/20-CONTEXT.md] |

### Known Threat Patterns for Phase 20

| Pattern | STRIDE | Standard Mitigation |
| --- | --- | --- |
| Secret leakage in committed hardware evidence | Information Disclosure | Mandatory redaction review before commit; never read or print credential files. [VERIFIED: AGENTS.md] |
| Unsafe active hardware stimulus | Tampering / Denial of Service | Require allow manifest, bounded inputs, abort conditions, recovery steps, and final safe-state markers. [VERIFIED: tools/parity/src/safety_allow.rs] |
| Target spoofing or stale device target | Spoofing | Use detector port, board-info pass, package manifest, source/reference commits, explicit `DEVICE_URL`, or trusted flash evidence. [VERIFIED: AGENTS.md] [VERIFIED: scripts/phase17-live-http-api-smoke.sh] |
| Production exposure of diagnostics | Elevation of Privilege / Tampering | Compile-gate diagnostic triggers and cover them with tests/redaction review. [VERIFIED: .planning/phases/20-active-safety-hardware-telemetry-evidence/20-CONTEXT.md] |
| Checklist overclaim | Repudiation / Integrity | Run `just parity`; active safety rows require `hardware-regression`. [VERIFIED: tools/parity/src/main.rs] [VERIFIED: Justfile] |

## Sources

### Primary (HIGH confidence)

- `.planning/phases/20-active-safety-hardware-telemetry-evidence/20-CONTEXT.md` - locked Phase 20 decisions, discretion, deferred scope, and canonical references.
- `.planning/REQUIREMENTS.md` - SAFE-01 through SAFE-09 and EVD-05 requirement text and traceability.
- `.planning/ROADMAP.md` - Phase 20 goal, gap closure, dependencies, and success criteria.
- `.planning/STATE.md` - current project state after Phase 19.
- `AGENTS.md` - Ultra 205 detector gate, hardware evidence metadata, redaction, destructive/fault-injection limits, and Markdown separator rules.
- `AGENTS.bright-builds.md`, `standards/core/architecture.md`, `standards/core/testing.md`, `standards/core/verification.md`, `standards/languages/rust.md` - Bright Builds and Rust workflow constraints.
- `tools/parity/src/safety_allow.rs` - allow manifest schema, active tiers, surfaces, required aborts, safe-state markers, and evidence class mapping.
- `tools/parity/src/main.rs` - checklist validation and safety-critical no-overclaim enforcement.
- `scripts/detect-ultra205.sh`, `Justfile`, `scripts/BUILD.bazel` - detector, command surface, and test targets.
- `scripts/phase14-*.sh`, `scripts/phase14-*-test.sh` - current safety evidence wrapper behavior.
- `scripts/phase17-live-http-api-smoke.sh`, `scripts/phase17-websocket-capture.mjs` - explicit target live HTTP/WebSocket patterns.
- `crates/bitaxe-safety/src/*.rs` - pure safety power, thermal, fault, self-test, watchdog, and evidence logic.
- `firmware/bitaxe/src/safety_adapter*.rs`, `firmware/bitaxe/src/http_api.rs`, `firmware/bitaxe/src/runtime_snapshot.rs`, `firmware/bitaxe/src/display_adapter.rs` - firmware safety, API/WebSocket, snapshot, and display integration points.
- `crates/bitaxe-api/src/snapshot.rs`, `crates/bitaxe-api/src/wire.rs`, `crates/bitaxe-api/src/telemetry.rs` - safety telemetry projection and live envelope behavior.
- `docs/parity/evidence/phase-14-safety-hardware-evidence-completion*.md`, `docs/parity/evidence/phase-17-live-http-api-and-static-evidence/summary.md`, `docs/parity/evidence/phase-19-recovery-regression-and-otawww-evidence/summary.md` - prior evidence boundaries and residual blockers.
- `docs/parity/checklist.md` - current row status/evidence semantics.
- `reference/esp-miner/config-205.cvs`, `reference/esp-miner/main/device_config.h`, `reference/esp-miner/main/power/DS4432U.c`, `reference/esp-miner/main/power/INA260.c`, `reference/esp-miner/main/tasks/power_management_task.c`, `reference/esp-miner/main/http_server/system_api_json.c` - read-only upstream Ultra 205 reference breadcrumbs.

### Secondary (MEDIUM confidence)

- Environment probes for local command versions: `just`, `bazel`, `cargo`, `rustc`, `espflash`, `node`, `curl`, and `jq`.

### Tertiary (LOW confidence)

- None. No unverified web-only sources were used.

## Metadata

**Confidence breakdown:**

- Standard stack: HIGH - repo commands, helper scripts, and local tool versions were verified directly. [VERIFIED: Justfile] [VERIFIED: env probes]
- Architecture: HIGH - Phase 14/17 patterns, parity validators, safety crates, API models, and firmware adapters were inspected directly. [VERIFIED: tools/parity/src/safety_allow.rs] [VERIFIED: scripts/phase17-websocket-capture.mjs] [VERIFIED: crates/bitaxe-safety/src/power.rs] [VERIFIED: firmware/bitaxe/src/safety_adapter.rs]
- Pitfalls: HIGH - they are directly tied to locked Phase 20 decisions and prior evidence blockers. [VERIFIED: .planning/phases/20-active-safety-hardware-telemetry-evidence/20-CONTEXT.md] [VERIFIED: docs/parity/evidence/phase-14-safety-hardware-evidence-completion.md]
- Active hardware outcome: MEDIUM - the code and governance path are verified, but this research did not run hardware detection, flashing, active probes, or live network capture. [VERIFIED: AGENTS.md]

**Research date:** 2026-07-03
**Valid until:** 2026-08-02 for repo governance and code patterns; re-check live tool versions and hardware availability before execution. [VERIFIED: env probes]

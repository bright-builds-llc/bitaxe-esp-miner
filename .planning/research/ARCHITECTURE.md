# Architecture Research

**Domain:** Rust ESP-IDF firmware rewrite for Bitaxe ESP-Miner
**Researched:** 2026-06-20
**Overall confidence:** HIGH for project-local boundaries and phase ordering, MEDIUM for ESP-IDF Rust/Bazel integration details that still need implementation spikes.

## Summary

Structure the project as a behavior-led Rust firmware, not a line-by-line C translation. The rewrite should preserve device-user parity by tracking observable surfaces from `reference/esp-miner`, while allowing Rust modules to use stronger domain types, smaller pure crates, and explicit state transitions. The upstream tree is evidence and provenance, not the architecture template.

Use a functional core with an imperative ESP-IDF shell. Pure crates own deterministic behavior: config defaults and validation, ASIC packet construction/parsing, Stratum messages and job construction, AxeOS API models, statistics, PID decisions, work queues, and state machines. `firmware/bitaxe` owns side effects: ESP-IDF services, FreeRTOS tasks, Wi-Fi, HTTP/WebSocket serving, NVS, SPIFFS, OTA, UART, GPIO, I2C, ADC, power rails, fan/display/input adapters, and task orchestration.

Bazel should be the canonical automation graph, with `just` as the ergonomic command surface. Early targets may wrap Cargo, ESP-IDF Rust tooling, or repo-owned scripts, but those workflows should still appear as Bazel targets so local and CI behavior stays aligned. This matches the accepted project decision and the current `rules_rust` Crate Universe model for generating Bazel Rust targets from Cargo metadata.

Reference breadcrumbs should sit at module and behavior boundaries, not on every translated line. Each parity surface should have a breadcrumb to the upstream file/function, a Rust-owned target, and evidence in `docs/parity/checklist.md`. Verified parity should mean the evidence proves the user-visible behavior, not that the Rust code resembles the C code.

Source basis:

- Project docs: `.planning/PROJECT.md`, `docs/project/seed-layout.md`, `docs/project/project-decisions.md`, `docs/parity/checklist.md`
- Local standards: `standards/core/architecture.md`, `standards/languages/rust.md`
- ESP-IDF Rust ecosystem: `https://docs.espressif.com/projects/rust/`, `https://github.com/esp-rs/esp-idf-svc`, `https://github.com/esp-rs/esp-idf-sys`
- ESP-IDF architecture context: `https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-guides/build-system.html`, `https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/freertos.html`
- Bazel Rust dependency model: `https://bazelbuild.github.io/rules_rust/crate_universe_bzlmod.html`

## Components

Recommended component and evidence map:

```text
reference/esp-miner
  -> tools/parity
  -> docs/parity/checklist.md

crates/bitaxe-core
crates/bitaxe-config
crates/bitaxe-asic
crates/bitaxe-stratum
crates/bitaxe-api
  -> consumed by firmware/bitaxe and tests

crates/bitaxe-test-support
  -> consumes pure crates and reference-derived fixtures

firmware/bitaxe
  -> depends on pure crates
  -> ESP-IDF Rust crates
  -> device hardware
  -> hardware smoke/regression evidence

MODULE.bazel / BUILD.bazel targets
  -> just commands
  -> CI and local workflows
```

| Component | Responsibility | Must Not Own | Primary Evidence |
| --- | --- | --- | --- |
| `reference/esp-miner` | Pinned read-only upstream reference and source breadcrumbs. | Local fixes, patch experiments, or Rust implementation work. | `scripts/verify-reference-clean.sh`, pinned commit, checklist breadcrumbs. |
| `firmware/bitaxe` | ESP-IDF app, boot order, task orchestration, hardware adapters, HTTP/WebSocket server, storage, OTA, logging, and board bring-up. | Core business decisions that can be pure, protocol codecs, API schemas as ad hoc maps. | Boot logs, hardware smoke, API compare, hardware regression. |
| `crates/bitaxe-core` | Shared domain state, work queues, statistics, scoreboard, thermal/PID decisions, event types, state machines. | ESP-IDF imports, sockets, NVS, UART, GPIO, wall-clock ownership. | Unit tests, golden state-transition tests. |
| `crates/bitaxe-config` | Board/device models, Gamma 601 defaults, upstream CSV/default extraction, typed settings, NVS key schema, validation. | Direct NVS reads/writes or mutable global config. | Unit tests, golden defaults from `config-601.cvs`, NVS key fixture checks. |
| `crates/bitaxe-asic` | ASIC model dispatch, packet layouts, CRC, register commands, work encoding, nonce/result parsing, BM1370-first behavior. | UART transport, GPIO reset, voltage rails, sleeps/timing side effects. | Unit/golden tests, BM1370 hardware evidence for init/work/result paths. |
| `crates/bitaxe-stratum` | Stratum v1/v2 message parsing, serialization, job construction, coinbase decode, reconnect/fallback decision inputs. | TCP sockets, DNS, Wi-Fi lifecycle, task scheduling. | Unit/golden protocol fixtures, pool integration smoke later. |
| `crates/bitaxe-api` | AxeOS OpenAPI-compatible request/response models, serializers, PATCH validation, telemetry/log payload shapes. | HTTP server lifecycle, WebSocket transport, flash upload side effects. | Schema/API compare fixtures and captured upstream responses. |
| `crates/bitaxe-test-support` | Reference-derived fixtures, golden data loaders, fake adapters, hardware-test helpers. | Production firmware logic. | Test harness output and fixture provenance. |
| `tools/parity` | Checklist reporting, missing-evidence detection, fixture generation from pinned reference sources. | Manual status truth that bypasses `docs/parity/checklist.md`. | `just parity`, Bazel parity report target. |
| `tools/flash` | USB port discovery, flash/monitor ergonomics, board/image selection. | Firmware behavior or parity decisions. | `just flash board=601`, captured flash logs. |
| `scripts/verify-reference-clean.sh` | Fail fast when `reference/esp-miner` is missing or locally modified; print pinned commit. | Generic build orchestration. | Workflow checks and CI output. |

## Boundaries

The hard rule is dependency direction: firmware shell depends on pure crates, pure crates never depend on ESP-IDF or firmware shell code.

Allowed dependency direction:

```text
firmware/bitaxe -> bitaxe-api
firmware/bitaxe -> bitaxe-config
firmware/bitaxe -> bitaxe-core
firmware/bitaxe -> bitaxe-asic
firmware/bitaxe -> bitaxe-stratum

bitaxe-api -> bitaxe-config and/or bitaxe-core only when models need shared domain types
bitaxe-stratum -> bitaxe-core only for shared job/work domain types
bitaxe-asic -> bitaxe-core only for shared job/result domain types
bitaxe-config -> bitaxe-core only for shared board/device identifiers
bitaxe-core -> no project crate unless a tiny shared type crate is introduced later
```

Do not introduce cross-crate convenience dependencies until there is a real invariant or duplication problem. If dependency cycles appear, create a small shared domain module inside `bitaxe-core` first; split a separate `bitaxe-types` crate only if `bitaxe-core` becomes a dumping ground.

Pure core boundary:

- Inputs are typed domain values, parsed once at boundaries.
- Outputs are decisions, commands, packets, snapshots, errors, or events.
- No direct hardware, ESP-IDF, FreeRTOS, clock, random, file system, socket, NVS, or logging calls.
- Use newtypes and enums for board IDs, ASIC model IDs, frequencies, millivolts, temperatures, fan duty, job IDs, nonce values, pool messages, and API setting updates.
- Make invalid states unrepresentable where practical, especially around mining lifecycle, configured/unconfigured device state, power state, and API PATCH results.

Imperative shell boundary:

- Owns `esp_idf_svc`, `esp_idf_hal`, `esp_idf_sys`, FreeRTOS tasks, ESP event loop, HTTP/WebSocket server, NVS, OTA, SPIFFS, UART, GPIO, I2C, ADC, display, input, fan, and voltage adapters.
- Converts raw observations into typed events before handing them to pure crates.
- Applies pure-core commands to hardware only through named adapters with explicit error handling and logs.
- Keeps task loops thin: receive event, call pure decision function, perform adapter effects, publish resulting event/snapshot.

Reference breadcrumb boundary:

- Each crate/module that implements a parity surface starts with a short breadcrumb list: parity IDs, upstream files/functions, and evidence expectations.
- Function-level breadcrumbs are reserved for non-obvious constants, packet layouts, register sequences, or compatibility quirks.
- Breadcrumbs should say "this behavior is compared against X" rather than "this line came from X".
- Any intentionally ported upstream expression needs provenance review before being treated as MIT-first original work.

Hardware safety boundary:

- Voltage, fan, thermal, power, and ASIC initialization code stays behind firmware adapters with safe staging.
- Pure crates may calculate desired commands, but firmware decides whether the command is safe to issue in the current hardware state.
- Safety-critical parity cannot move to `verified` without hardware evidence, even when unit tests pass.

## Data Flow

Boot/config flow:

```text
ESP-IDF app entry
  -> firmware boot coordinator
  -> NVS/board/default adapters
  -> bitaxe-config typed DeviceConfig
  -> bitaxe-core initial DeviceState
  -> firmware starts services/tasks from typed state
  -> boot log and parity evidence capture
```

Mining flow:

```text
Wi-Fi/socket adapter
  -> bitaxe-stratum parses pool messages
  -> bitaxe-stratum builds mining jobs
  -> bitaxe-core schedules work and updates lifecycle state
  -> bitaxe-asic encodes BM1370 work packets
  -> firmware UART adapter sends packets to ASIC
  -> firmware UART adapter receives nonce/result bytes
  -> bitaxe-asic parses results
  -> bitaxe-core updates stats/scoreboard/work status
  -> bitaxe-stratum serializes share submissions
  -> firmware socket adapter sends submissions
```

Config/API flow:

```text
HTTP handler in firmware
  -> bitaxe-api request model and PATCH validation
  -> bitaxe-config setting update command
  -> firmware NVS adapter persists accepted settings
  -> bitaxe-core applies runtime-visible state change
  -> bitaxe-api response serializer
  -> HTTP/WebSocket transport
```

Thermal/power/fan flow:

```text
ADC/I2C/GPIO adapters
  -> typed sensor samples
  -> bitaxe-core thermal/PID decision
  -> typed actuator command
  -> firmware safety gate
  -> fan/voltage/power adapter
  -> telemetry event
  -> stats/API/log snapshots
```

Parity evidence flow:

```text
reference breadcrumb
  -> checklist parity ID
  -> Rust-owned implementation pointer
  -> test or hardware evidence artifact
  -> docs/parity/checklist.md status update
  -> tools/parity report
```

Data should generally move through events and snapshots rather than shared mutable global state. Firmware can use FreeRTOS queues, channels, mutexes, or atomics where needed, but those primitives should be adapter details. The pure crates should see domain inputs and return domain outputs.

## Build Order

Recommended phase order:

| Order | Phase Focus | Why First/Next | Parity Surfaces |
| --- | --- | --- | --- |
| 1 | Repository foundation | Establish the graph before behavior work so all later evidence has a home. | WF-001 through WF-003. |
| 2 | Reference guard and parity tooling skeleton | Prevent accidental upstream edits and make missing evidence visible from day one. | WF-001, `tools/parity`, checklist reporting. |
| 3 | Pure crate skeletons and shared domain types | Forces architecture boundaries before hardware pressure pushes logic into tasks. | SYS-003, CFG-003, early API/core types. |
| 4 | Gamma 601 config model | Device bring-up needs typed board defaults and settings before meaningful boot tests. | CFG-001, CFG-003, CFG-004. |
| 5 | Minimal ESP-IDF firmware boot/log path | Proves toolchain, Bazel/just surface, image packaging, flash, and monitor on real hardware. | SYS-001, SYS-002, WF-004, WF-005. |
| 6 | AxeOS API model compatibility baseline | API snapshots define user-visible contracts before full mining behavior exists. | API-001 through API-003. |
| 7 | BM1370 pure ASIC codecs | CRC, packet layout, work encoding, and result parsing are high-value pure tests before hardware initialization. | ASIC-002 through ASIC-006. |
| 8 | Safe BM1370 hardware initialization | The first dangerous hardware surface should be narrow, logged, and evidence-heavy. | ASIC-002, ASIC-005, PWR-001 through PWR-004. |
| 9 | Stratum v1 and first mining loop | Mining needs config, boot, ASIC basics, and socket behavior in place. | STR-001 through STR-004, STR-006. |
| 10 | Stats, logs, and WebSocket telemetry | These prove the device-user parity of visible runtime state. | LOG-001, STAT-001 through STAT-004, API-005, API-006. |
| 11 | Thermal, fan, and power controllers | Safety loops need stable telemetry and hardware evidence. | PWR/THR surfaces. |
| 12 | OTA, filesystem, static assets, release images | Packaging and update behavior depend on stable partition/image/API choices. | FS-001, OTA-001, OTA-002, REL-001 through REL-003. |
| 13 | Additional ASICs/boards and Stratum v2 | Do not dilute the Gamma 601 evidence path before the first full loop works. | ASIC-008 through ASIC-010, CFG-002, STR-005, non-601 boards. |

Build graph implications:

- `//crates/...:tests` should pass before firmware build targets rely on those crates.
- `//scripts:verify_reference_clean` should run before parity fixture generation and before release packaging.
- `//tools/parity:report` should run in CI once checklist structure exists.
- `//firmware/bitaxe:firmware` should depend on the pure crate libraries but not on test support.
- `//firmware/bitaxe:firmware_image` should be the source for flash and release workflows.
- `just build`, `just test`, `just package`, `just flash`, `just monitor`, `just flash-monitor`, `just verify-reference`, and `just parity` should call Bazel targets or scripts also represented in Bazel.

## Testing Strategy

Use a test pyramid shaped around parity evidence:

| Layer | What It Proves | Where It Lives | Evidence Type |
| --- | --- | --- | --- |
| Pure unit tests | Domain decisions are deterministic and typed invariants hold. | `crates/bitaxe-*` | `unit` |
| Golden fixture tests | Rust packet/API/config output matches reference-derived fixtures. | Pure crates plus `bitaxe-test-support` | `golden` |
| API compare tests | AxeOS schemas and captured responses stay client-compatible. | `crates/bitaxe-api`, firmware route smoke | `api-compare` |
| Firmware smoke tests | Build, flash, boot, log, and basic service startup work on Gamma 601. | `firmware/bitaxe`, `tools/flash` | `hardware-smoke` |
| Hardware regression tests | Safety-critical and mining loops remain repeatable. | `tools/parity`, `tools/flash`, hardware harness helpers | `hardware-regression` |
| Reference guard checks | Upstream evidence was not silently edited. | `scripts/verify-reference-clean.sh` | workflow evidence |

Pure tests should use Arrange, Act, Assert structure. Good first pure targets are ASIC CRC, BM1370 packet encoding, BM1370 result parsing, Gamma 601 defaults, NVS key/default mapping, OpenAPI response serialization, coinbase decoding, job construction, PID decisions, scoreboard behavior, and work queue semantics.

Hardware tests should capture enough context to be useful as audit evidence: command, board, port, firmware image or commit, reference commit, serial log excerpt, observed result, and pass/fail conclusion. A hardware test can validate many implementation details, but the checklist should record the user-visible parity claim it proves.

Status discipline:

- `not-started`: no Rust-owned implementation.
- `in-progress`: implementation exists but is incomplete.
- `implemented`: code exists but evidence is not enough.
- `verified`: evidence proves parity or accepted project behavior.
- `deferred`: gap is explicit with owner and reason.

Do not let "builds successfully" imply parity. Build success proves only workflow health unless paired with behavior evidence.

## Risks

| Risk | What Goes Wrong | Mitigation |
| --- | --- | --- |
| Line-by-line C translation pressure | Rust modules mirror upstream C files, including globals and incidental control flow, making the rewrite harder to maintain than the original. | Organize by domain behavior and adapter boundaries. Use breadcrumbs for provenance, not as a module layout mandate. |
| Hidden global state migration | Upstream global state becomes Rust global mutable state, preserving invalid states and race risks. | Model lifecycle state in `bitaxe-core` with enums/newtypes and pass snapshots/events through explicit queues. |
| ESP-IDF Rust integration maturity | `esp-idf-svc` appears to cover the needed ESP-IDF service families, but the `esp-idf-*` docs flag community maintenance, possible lag behind ESP-IDF stable, missing HIL tests, and sparse docs. | Keep ESP-IDF usage in thin adapters, spike each service before depending on it broadly, and prioritize early Gamma 601 hardware smoke. |
| Bazel/ESP-IDF impedance mismatch | Direct hermetic Bazel builds may be hard before ESP-IDF Rust/Cargo details are proven. | Represent workflows as Bazel targets from the start, even if early targets wrap Cargo/ESP-IDF scripts. Tighten hermeticity after boot/flash works. |
| False parity confidence | Implemented code gets marked verified without proving behavior. | Require checklist evidence for every status promotion and run `tools/parity:report` in local/CI workflows. |
| Hardware safety regression | Voltage, fan, thermal, power, or ASIC init changes damage hardware or create unsafe operating conditions. | Stage hardware enablement narrowly, require hardware evidence, add safety gates in firmware adapters, and keep pure command generation separate from effect execution. |
| API compatibility drift | Rust API models become cleaner than upstream but break existing AxeOS clients. | Treat `openapi.yaml` and captured upstream responses as compatibility contracts; compare encoded responses, not just Rust type names. |
| NVS/settings mismatch | Settings names, defaults, and runtime update behavior diverge from existing devices. | Parse upstream config/default files into typed fixtures and test NVS key/default behavior before runtime persistence work. |
| Task behavior drift | Different FreeRTOS task decomposition changes observable timing, logs, reconnect behavior, or telemetry freshness. | Allow internal task layout to differ, but write parity tests around observable boot order, API state, reconnect behavior, logs, and telemetry cadence. |
| Provenance/license leakage | Ported GPL-covered source expression is treated as MIT-first original work. | Use breadcrumbs and provenance review at behavior boundaries; flag intentional ports explicitly before release artifacts ship. |

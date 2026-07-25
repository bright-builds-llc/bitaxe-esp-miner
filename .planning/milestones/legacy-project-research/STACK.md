# Stack Research

**Domain:** Ultra 205 operator-ready Rust ESP-IDF runtime
**Researched:** 2026-07-13
**Confidence:** HIGH for stack continuity and repository integration; MEDIUM-HIGH for the exact runtime-health wire shape

## Executive Recommendation

Do not add a new firmware framework, sensor-driver crate, storage layer, metrics stack, or evidence runner for v1.2. The repository already contains the necessary foundations:

- ESP-IDF Rust `std` firmware with I2C, NVS, HTTP/WebSocket, logging, FreeRTOS-backed threads, and system APIs.
- Register-level INA260 and EMC2101 adapters.
- Pure power, thermal, settings-persistence, self-test, watchdog, API, and evidence models.
- Bazel-owned build/package targets, `just` operator commands, `espflash`, detector gating, and redacted evidence workflows.

The missing stack capability is composition and lifecycle ownership. v1.2 should establish one long-lived serialized owner for Ultra 205 I2C0, publish timestamped typed observations into the existing pure model, verify NVS commit/readback/reload behavior, project truthful build and health facts, and extend the existing evidence tooling with a bounded non-actuating operator-readiness profile.

This recommendation is materially informed by the repo-local hardware/evidence guidance, `AGENTS.bright-builds.md`, `standards/core/architecture.md`, `standards/core/verification.md`, `standards/core/testing.md`, and `standards/languages/rust.md`: hardware effects remain in thin adapters, state and claim decisions remain pure and typed, and verification never promotes beyond observed evidence.

## Recommended Stack

### Core Technologies

| Technology | Version | Purpose | Why Recommended |
| --- | --- | --- | --- |
| ESP-IDF | `v5.5.4` | I2C, NVS, HTTP/WebSocket, FreeRTOS tasks, watchdog APIs, logging, heap/runtime facts, and image conventions | Already pinned in `firmware/bitaxe/Cargo.toml` and `.cargo/config.toml`; changing the SDK during an evidence-sensitive hardware milestone adds unrelated risk. |
| `esp-idf-svc` | `0.52.1` | Safe service wrappers plus the `hal` and `sys` re-export surfaces | Already owns NVS, Wi-Fi, HTTP, logging, and firmware service lifecycles. v1.2 should deepen that integration instead of introducing a parallel abstraction. |
| `esp-idf-hal` | `0.46.2` transitively resolved | Blocking ESP-IDF I2C driver used by the existing `BitaxeI2cBus` | The repository already compiles the register adapters on this driver. A dedicated bus owner is sufficient; a second HAL is not. |
| `esp-idf-sys` | `0.37.2` | Narrow access to ESP-IDF APIs not wrapped by `esp-idf-svc`, including the existing NVS commit path and optional task-watchdog status/user calls | Keep raw calls inside small firmware adapters. Do not leak raw handles or error codes into pure crates. |
| Rust `std::thread`, `std::sync`, and monotonic uptime | Rust 2021 on `xtensa-esp32s3-espidf` | One serialized I2C sampling owner, bounded cadence, publication of immutable snapshots, and health heartbeats | Matches the existing FreeRTOS-backed thread model and needs no executor or synchronization dependency. |
| Existing pure crates | Workspace versions | Freshness, failure, persistence, self-test, watchdog, API, and evidence decisions | `bitaxe-safety`, `bitaxe-config`, and `bitaxe-api` already encode most required states. Extend these types rather than duplicating logic in firmware. |
| Bazel + Bzlmod + `rules_rust` | Bazel `9.1.1`; `rules_rust 0.70.0` | Canonical build, package, host test, and evidence graph | This is the established repository boundary. New v1.2 scripts and tests should be Bazel targets reached through `just`. |
| `just` + `espflash` | `just 1.48.0`; locally installed `espflash 4.0.1` | Human command surface, detector gate, package, flash, bounded monitor, and board identity | Both already satisfy the repository workflow. v1.2 does not require a tool upgrade or a second flashing/monitoring program. |

### Supporting Libraries

| Library | Version | Purpose | When to Use |
| --- | --- | --- | --- |
| `serde` / `serde_json` | `1.0.228` / `1.0.150` | Typed API, health, and evidence payloads | Reuse for additive AxeOS-compatible fields and evidence manifests. Do not introduce another serialization stack. |
| `anyhow` | `1.0.102` | Context-rich firmware adapter and host-tool failures | Use at I2C, NVS, detector, and capture shells; convert failures to stable typed categories before projection. |
| `thiserror` | `2.0.18` | Pure domain errors | Use when extending freshness, persistence receipts, or health decisions in pure crates. |
| `sha2` | `0.11.0` | Existing artifact/trace digests | Reuse only where evidence needs a content digest; never hash secrets as a substitute for redaction. |
| Existing shell/Node evidence helpers | Repository-owned | Bounded orchestration, API capture, redaction, validation, and fixture tests | Extend the established phase-evidence pattern. Keep host orchestration outside firmware and make every path rerunnable. |

No new required dependency is recommended for v1.2.

### Development Tools

| Tool | Purpose | Notes |
| --- | --- | --- |
| `just detect-ultra205` | Mandatory hardware identity gate | Preserve the exactly-one-port plus successful ESP32-S3 `board-info` contract before any future v1.2 hardware run. Research itself must not access hardware. |
| Existing firmware package and flash/monitor commands | Produce exact-head firmware and bounded serial evidence | Future hardware capture must use the repo minimum 360-second capture budget and a shell wall clock of at least 420 seconds. |
| New v1.2 operator-readiness evidence wrapper | Correlate detector, package, telemetry, settings persistence, provenance, and runtime health | Implement as a repo-owned Bazel target with `just` entrypoint. It must exclude mining, active fan/voltage/reset/power sequencing, fault injection, and archived Phase 28.1.1 operations. |
| `tools/parity` | Validate evidence shape and exact claims | Add an operator-readiness profile or manifest schema to the existing tool instead of building a separate validator. |
| Cargo/Bazel tests | Pure state and adapter contract verification | Use pure fixture tests for state decisions and fake I2C/NVS boundaries where practical; retain real hardware evidence only for the final hardware claim. |

## Installation

No new packages are required. Keep the existing setup and pins:

```bash
just doctor
just bootstrap-esp
just build
just test
just package
```

Do not run hardware commands during milestone research. A phase-owned evidence plan may use the established detector-gated commands later.

## Required Stack Composition Changes

### 1. One Long-Lived Serialized I2C0 Owner

Current normal startup consumes I2C0 for the one-shot SSD1306 render and drops it. The Phase 27 path instead retains a special diagnostic bus for later sampling. Neither lifecycle provides normal operator runtime telemetry.

Create one ordinary firmware owner for I2C0 that:

- Takes `peripherals.i2c0`, GPIO47, and GPIO48 exactly once.
- Serializes the startup display operation and recurring sensor reads through the same owner.
- Owns `BitaxeI2cBus` for the process lifetime rather than lending the driver to multiple tasks.
- Polls at a bounded cadence on one worker and publishes immutable typed snapshots for API/WebSocket readers.
- Records per-attempt timing and failure category without exposing raw driver errors in public payloads.
- Fails to explicit unavailable state when bus construction or device access fails; it must not fall back to invented zeros labeled fresh.

A dedicated owner task is preferable to sharing `I2cDriver` through a broad `Arc<Mutex<_>>`: it makes ordering explicit, keeps device I/O in one imperative shell, and avoids making every consumer responsible for lock and timeout policy.

### 2. Existing Register Adapters Plus Typed Freshness

Keep the repository-owned INA260 and EMC2101 adapters. They already implement the registers used by the pinned reference and avoid a new dependency whose abstraction may not match upstream behavior.

Required refinements:

- Timestamp each attempt and each successful sample with the existing monotonic runtime clock.
- Represent `Unavailable`, `Fresh`, `Stale`, transport/read `Failed`, and unsafe/invalid `Fault` separately. Preserve the last successful value and its age when a later attempt fails.
- Extend thermal observations with an age/stale contract equivalent to the existing power threshold; thermal currently has no age field.
- Decode INA260 current as signed two's-complement before unit conversion. The TI datasheet defines the current register that way; the current adapter treats it as `u16`.
- Keep INA260 current, bus-voltage, and power reads read-only.
- Read EMC2101 temperature and tachometer registers without calling the existing initialization/fan configuration path when that path could alter fan behavior.
- Never call DS4432U voltage writes, EMC2101 fan-duty writes, ASIC enable/reset, or mining setup from the v1.2 telemetry owner.

Freshness and failure classification belongs in `crates/bitaxe-safety`; timestamps and I2C errors originate in the firmware adapter. API and evidence code should consume the typed result rather than reconstructing age or status from numeric sentinels.

### 3. Separate Runtime Observation From Parity-Claim Evidence

`SafeTelemetrySnapshot::from_report` currently hides otherwise fresh numeric values unless the report contains hardware-verification evidence. That is appropriate for blocking unsupported safety claims, but it conflates two different facts:

1. the device has just read a sensor successfully; and
1. the project has accumulated enough evidence to mark an upstream parity row verified.

v1.2 should keep both facts explicit. A fresh runtime observation may be shown to the operator with its source and age, while parity promotion remains gated by the detector-gated evidence workflow. Do not manufacture a `hardware_smoke` token for every ordinary boot merely to make values visible.

### 4. NVS Commit, Readback, Reload, And Boot Projection

Retain `FirmwareSettingsAdapter`, `EspDefaultNvsPartition`, `EspNvs`, the pure `NvsSnapshot`, and the existing accepted-patch plan. ESP-IDF requires `nvs_commit()` after writes; the current adapter already performs it.

Deepen the existing adapter rather than adding a storage crate:

- After commit, reopen the namespace read-only and compare typed readback against the accepted write plan.
- Return a typed persistence receipt that distinguishes validation rejection, write failure, commit failure, readback failure, mismatch, and successful reload.
- Publish the reloaded snapshot only after successful readback; do not update the API-visible cache merely because the write call returned.
- At startup, load the same schema into the API snapshot so a later normal reboot can prove persistence without reading raw NVS through evidence scripts.
- Use reversible, non-secret supported settings in committed evidence. Sensitive values remain local and must never appear in logs, JSON artifacts, diffs, or summaries.
- If reboot persistence is included by requirements, use only the existing normal restart surface under a phase-owned non-destructive plan. Raw reset, power sequencing, erase, and fault injection remain out of scope.

The existing raw `nvs_commit` call is a justified narrow `esp-idf-sys` boundary. Replacing the whole adapter with another NVS library would add churn without improving the evidence contract.

### 5. Truthful Build And System Provenance

Reuse the current compile-time source commit, running-partition lookup, ESP-IDF version, package manifest, and reference commit. Add only the missing typed mapping needed by the existing AxeOS-compatible surfaces:

- Product/firmware version.
- Exact or short source commit.
- ESP-IDF version and Rust target.
- Running partition and reset reason.
- Reference commit and package/evidence identity where the public contract permits it.
- `Unavailable` when a field was not supplied by the build; never substitute a fixture label in live firmware.

Prefer `env!`/`option_env!`, Cargo package metadata, and the existing Bazel source-commit stamp. Do not add `vergen`, execute network commands from `build.rs`, or inject the local wall clock. If a build time is required, accept a reproducible CI-provided `SOURCE_DATE_EPOCH`-style value and otherwise report `Unavailable`.

### 6. Runtime Health, Self-Test, And Watchdog Projection

Reuse the pure watchdog and self-test models, but distinguish model state from real ESP-IDF protection:

- Track the telemetry owner heartbeat, last completed sample, consecutive failures, NVS load/readback state, HTTP projection readiness, and supervisor heartbeat with monotonic timestamps.
- Project a typed aggregate such as `Starting`, `Healthy`, `Degraded`, or `Unavailable`, plus component reasons.
- If v1.2 claims actual task-watchdog participation, add a narrow firmware adapter around the ESP-IDF Task Watchdog user/task APIs and pin the required Kconfig options in `sdkconfig.defaults`. A pure `StepSupervisor` decision or a log line is not proof that the task is subscribed to TWDT.
- Expose self-test lifecycle/capability state without starting hardware self-test. Existing self-test effects include ASIC and fan behavior and therefore remain unavailable or safely blocked in this observation-only milestone.
- Do not perform watchdog fault injection, intentional hangs, resets, or active self-test effects in v1.2. Their absence must remain visible rather than being reported as passed.

### 7. Bounded Detector-Gated Evidence Profile

Build the v1.2 evidence path from existing components:

- `just detect-ultra205` and the selected same-session target.
- Exact-head package manifest, source commit, reference commit, board `205`, and ESP32-S3 board-info.
- Bounded flash/monitor output and cleanup proof under the repository's serial-session rules.
- Timestamped sensor status transitions and API/WebSocket correlation.
- Settings PATCH decision, NVS commit/readback/reload result, and optional planned normal-reboot observation.
- Build/system provenance and runtime-health projection from the same session.
- Redaction validation before any evidence is committed or used for parity promotion.

The wrapper should support deterministic fake/blocked tests, but only the detector-gated hardware mode may support a hardware claim. Blocked or fixture mode proves workflow behavior, not live telemetry. No v1.2 path may route into mining diagnostics or the archived Phase 28.1.1 lineage.

## Alternatives Considered

| Recommended | Alternative | When to Use Alternative |
| --- | --- | --- |
| One dedicated I2C owner task using the existing driver | Shared `Arc<Mutex<I2cDriver>>` across display, telemetry, and other tasks | Only if a measured need for independent I2C callers outweighs the simpler serialized ownership model and the driver is proven safe for that sharing contract. |
| Repository-owned INA260/EMC2101 register adapters | Add sensor crates from crates.io | Only if a maintained crate supports the exact ESP-IDF/embedded-hal version, required register semantics, signed conversions, failure model, and reference-compatible behavior with less total code. No such need is present now. |
| Extend `bitaxe-safety`, `bitaxe-config`, and `bitaxe-api` | Create a new `bitaxe-operator-runtime` crate | Consider only if pure cross-domain orchestration becomes substantial and cannot remain cohesive in the existing crates. |
| Existing `EspNvs` adapter plus narrow raw commit call | New key-value store or serde-NVS layer | Use only for a future schema redesign, not for proving the already-modeled upstream NVS contract. |
| Existing shell/Node/Bazel evidence stack | Python test framework, embedded-test daemon, or external observability service | Use a new runner only if the current repository tools cannot express a concrete deterministic check. |
| Additive typed status on AxeOS-compatible payloads | Replace the AxeOS API or rewrite the Angular UI | UI replacement belongs to a separate milestone; preserve existing consumers while exposing truthful extensions. |

## What NOT to Use

| Avoid | Why | Use Instead |
| --- | --- | --- |
| ESP-IDF `v6.x` or an esp-rs upgrade during v1.2 | SDK/binding migration is unrelated to operator-readiness evidence and would expand the regression surface. | Keep the checked-in `v5.5.4` and current Cargo lock. |
| Tokio, async-std, Embassy, RTIC, or an async I2C stack | Adds executor, timer, wakeup, and ownership complexity for one bounded sensor loop. | Existing FreeRTOS-backed `std::thread` and blocking `I2cDriver`. |
| A second HAL or direct I2C access from API handlers | Creates competing peripheral ownership and timing policy. | One firmware bus owner publishing typed snapshots. |
| Calling EMC2101 initialization or fan-setting code merely to obtain telemetry | Configuration writes can change hardware behavior and violate the observation-only milestone. | Read temperature/tachometer state; report unavailable if safe read-only observation cannot be established. |
| DS4432U writes, fan writes, reset/power sequencing, ASIC control, or fault injection | Active hardware control is expressly outside v1.2 and requires separate recovery-gated evidence. | Explicit suppressed/unavailable states and later dedicated milestone work. |
| Synthetic timestamps, zero-valued fresh telemetry, or stale values relabeled fresh | Makes operator output look healthy without a successful current observation. | Monotonic sample age plus typed unavailable/stale/failed/fault states. |
| A new database, file-backed runtime store, or metrics agent | NVS, in-memory snapshots, API/WebSocket, and evidence files already cover the required durability and observability boundaries. | Existing NVS and typed in-memory projection. |
| `vergen` or local wall-clock build stamping | Adds a dependency and harms reproducibility while the repo already stamps source identity. | Existing Bazel/Cargo stamp inputs; optional reproducible epoch supplied by CI. |
| Raw credentials, network scans, stale device URLs, or direct NVS dumps in evidence | Violates target-lock and redaction policy. | Detector-gated same-session target resolution, API-visible non-secret values, and redacted category labels. |
| Any Phase 28.1.1 capture, diagnostic, UART, pin, or mining path | The lineage is terminal archived unresolved work and v1.2 explicitly excludes renewed mining diagnostics. | Operator-readiness telemetry, persistence, provenance, and health only. |

## Stack Patterns by Variant

**If I2C0 initializes and both sensors respond:**

- Run the dedicated owner at a bounded cadence.
- Publish independent INA260 and EMC2101 observation timestamps and states.
- Correlate numeric values and status metadata across API/WebSocket and evidence.

**If one sensor fails while the other remains healthy:**

- Keep independent component states.
- Preserve the failed sensor's last-good value as stale with age when one exists.
- Mark aggregate health degraded, not healthy and not wholly unavailable.

**If the display and recurring telemetry cannot safely share the current adapter:**

- Preserve one I2C owner and route the one-shot display through it, or defer the display operation with an explicit status.
- Do not instantiate a second I2C0 driver or sacrifice telemetry freshness silently.

**If NVS write succeeds but commit/readback differs:**

- Return a persistence failure and retain the last confirmed API snapshot.
- Record only field names, status categories, and digests allowed by redaction policy.
- Do not claim reload or reboot persistence.

**If real TWDT subscription is not implemented:**

- Report supervisor heartbeat/model status separately from hardware watchdog status.
- Label TWDT state unavailable or not subscribed; do not infer it from periodic logs.

**If hardware detection or same-session target provenance fails:**

- Run only deterministic workflow tests or record hardware evidence pending.
- Do not scan the network, reuse a stale URL, or promote fixture output.

## Version Compatibility

| Package A | Compatible With | Notes |
| --- | --- | --- |
| `esp-idf-svc 0.52.1` | `esp-idf-hal 0.46.2`, `esp-idf-sys 0.37.2` | Exact resolved firmware dependency set confirmed by `cargo tree`; keep Cargo.lock authoritative. |
| `esp-idf-sys 0.37.2` | ESP-IDF `tag:v5.5.4` | The firmware manifest and `.cargo/config.toml` explicitly pin the SDK rather than accepting a crate default. |
| ESP-IDF `v5.5.4` | `xtensa-esp32s3-espidf`, Ultra 205 ESP32-S3 | Existing firmware, package, flash, and evidence paths use this target. |
| `rules_rust 0.70.0` | Bazel `9.1.1` | Existing Bzlmod graph; no new repository rule is needed. |
| Installed `espflash 4.0.1` | Current repo detector/flash/monitor scripts | Preserve the repo-owned flags, including the complete passive-monitor contract where retained-runtime capture is used. Do not require an upgrade without a concrete incompatibility. |

## Sources

### Project-Local Sources

- `.planning/PROJECT.md`, `.planning/milestones/v1.1-MILESTONE-AUDIT.md`, and `.planning/RETROSPECTIVE.md` — v1.2 scope, accepted v1.1 gaps, terminal diagnostic closure, and evidence discipline. HIGH confidence.
- `.planning/milestones/v1.1-research/STACK.md` — established stack and dependency-continuity rationale. HIGH confidence.
- `Cargo.toml`, `Cargo.lock`, `firmware/bitaxe/Cargo.toml`, `.cargo/config.toml`, `.bazelversion`, and `MODULE.bazel` — exact repository pins and resolved dependency graph. HIGH confidence.
- `firmware/bitaxe/src/safety_adapter/{i2c_bus,ina260,emc2101,power,thermal,watchdog}.rs`, `safety_adapter/phase27_bring_up.rs`, `display_adapter.rs`, and `main.rs` — current peripheral ownership, register adapters, diagnostic-only snapshot path, and normal startup I2C lifecycle. HIGH confidence.
- `firmware/bitaxe/src/settings_adapter.rs` and `crates/bitaxe-config/src/{nvs,persistence}.rs` — accepted PATCH, NVS commit, pure read/reload model, and current API-visible cache. HIGH confidence.
- `firmware/bitaxe/build.rs`, `runtime_snapshot.rs`, `crates/bitaxe-api/src/{snapshot,system,wire}.rs`, and `tools/xtask/src/package_manifest.rs` — current source stamping, system-info mapping, and package provenance. HIGH confidence.
- `crates/bitaxe-safety/src/{power,thermal,self_test,watchdog}.rs` — existing typed status and pure decision boundaries, including the missing thermal-age contract and hardware-effectful self-test lifecycle. HIGH confidence.
- `AGENTS.md`, `AGENTS.bright-builds.md`, `standards-overrides.md`, `standards/core/{architecture,verification,testing}.md`, and `standards/languages/rust.md` — local hardware gates, evidence rules, functional-core/imperative-shell, and Rust verification guidance. HIGH confidence.

### Primary External Sources

- ESP-IDF v5.5 API reference: https://docs.espressif.com/projects/esp-idf/en/v5.5/esp32/api-reference/index.html — confirms v5.5.4 as the latest bugfix release on the pinned branch. HIGH confidence.
- ESP-IDF NVS documentation: https://docs.espressif.com/projects/esp-idf/en/v5.5.4/esp32s3/api-reference/storage/nvs_flash.html — `nvs_commit()` durability contract and namespace/key behavior. HIGH confidence.
- ESP-IDF v5.5 task watchdog documentation: https://docs.espressif.com/projects/esp-idf/en/v5.5/esp32s3/api-reference/system/wdts.html — task/user subscription, reset, status, and Kconfig/runtime boundaries. HIGH confidence.
- `esp-idf-svc 0.52.1` NVS example and changelog: https://docs.rs/crate/esp-idf-svc/0.52.1/source/examples/nvs_get_set_c_style.rs and https://docs.rs/crate/esp-idf-svc/0.52.1/source/CHANGELOG.md — current NVS wrapper/RawHandle surface used by the repository. HIGH confidence.
- `esp-idf-hal 0.46.2` repository: https://github.com/esp-rs/esp-idf-hal — blocking ESP-IDF I2C driver and supported-driver scope. HIGH confidence.
- Texas Instruments INA260 datasheet, Rev. C: https://www.ti.com/lit/ds/symlink/ina260.pdf — read-only current/bus-voltage/power register addresses, fixed LSBs, and signed current encoding. HIGH confidence.
- Microchip EMC2101 datasheet: https://ww1.microchip.com/downloads/aemDocuments/documents/MSLD/ProductDocuments/DataSheets/EMC2101-Data-Sheet-DS20006703.pdf — external-temperature and tachometer register behavior. HIGH confidence.

______________________________________________________________________

*Stack research for: v1.2 Ultra 205 Operator-Ready Runtime*
*Researched: 2026-07-13*

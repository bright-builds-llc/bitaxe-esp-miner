<!-- bright-builds-rules-managed:begin -->

# Bright Builds Rules

`AGENTS.md` is the entrypoint for repo-local instructions, not the complete Bright Builds Rules specification.

This managed block is owned upstream by `bright-builds-rules`. If this block needs a fix, open an upstream PR or issue instead of editing the managed text in a downstream repo. Keep downstream-specific instructions outside this managed block.

Before plan, review, implementation, or audit work:

1. Read the repo-local instructions in `AGENTS.md`, including any `## Repo-Local Guidance` section and any instructions outside this managed block.
1. Read `AGENTS.bright-builds.md`.
1. Read `standards-overrides.md` when present.
1. Read the local managed standards pages under `standards/` relevant to the task.
1. If you have not done that yet, stop and load those sources before continuing.

Use this routing map when deciding what to load next:

- For repo-specific commands, prerequisites, generated-file ownership, CI-only suites, or recurring workflow facts, use the local `AGENTS.md`, especially `## Repo-Local Guidance`.
- For the Bright Builds default workflow and high-signal cross-cutting rules used in most tasks, use `AGENTS.bright-builds.md`.
- For deliberate repo-specific exceptions to the Bright Builds defaults, use `standards-overrides.md`.
- To choose the right managed standards page, start with the local Bright Builds entrypoint `standards/index.md`.
- For business-logic structure, domain modeling, and functional-core versus imperative-shell decisions, use the managed standards page `standards/core/architecture.md`.
- For control flow, naming, function/file size, and readability rules, use the managed standards page `standards/core/code-shape.md`.
- For frontend visual defaults, theme defaults, and dark-mode decisions, use the managed standards page `standards/core/frontend-ui.md`.
- For sync, bootstrap, and pre-commit verification rules, use the managed standards page `standards/core/verification.md`.
- For unit-test expectations, use the managed standards page `standards/core/testing.md`.
- For Rust or TypeScript/JavaScript-specific rules, use the matching managed standards page under `standards/languages/`.
- For TypeScript/JavaScript frontend framework and UI-library defaults, use `standards/languages/typescript-javascript.md`.
- Keep recurring repo-specific workflow facts, commands, and links in a `## Repo-Local Guidance` section elsewhere in this file.
- Record deliberate repo-specific exceptions and override decisions in `standards-overrides.md`.
- If instructions elsewhere in `AGENTS.md` conflict with `AGENTS.bright-builds.md`, follow the repo-local instructions and treat them as an explicit local exception.

<!-- bright-builds-rules-managed:end -->

<!-- GSD:project-start source:PROJECT.md -->

## Project

**Bitaxe Rust Firmware**

Bitaxe Rust Firmware is a new Rust firmware monorepo for Bitaxe ESP-Miner. It builds a Rust implementation of the Bitaxe ESP32 miner firmware with device-user parity against upstream `bitaxeorg/ESP-Miner`, while keeping the upstream C code as a pinned read-only reference implementation.

The project is for Bitaxe owners and firmware contributors who need a maintainable Rust firmware that can be built, flashed, configured, monitored, and updated with the same observable behavior as upstream ESP-Miner.

**Core Value:** A Bitaxe owner can build, flash, run, configure, monitor, and update Rust firmware on real Bitaxe hardware with the same observable behavior they expect from upstream ESP-Miner.

### Constraints

- **Tech stack**: Use ESP-IDF Rust bindings for the first production firmware stack - upstream ESP-Miner depends heavily on ESP-IDF services such as Wi-Fi, HTTP serving, NVS, SPIFFS, OTA, FreeRTOS tasks, PSRAM-aware allocation, logging, partition images, and ESP flashing conventions.
- **Build orchestration**: Use Bazel as the canonical automation graph and `just` as the human command surface - local development and CI should route through the same graph where practical.
- **Reference implementation**: Keep upstream ESP-Miner pinned and read-only at `reference/esp-miner` - it is behavioral evidence, not a workspace for project changes.
- **Parity evidence**: Maintain a parity checklist with breadcrumbs, implementation pointers, statuses, and verification evidence - implemented code is not enough to claim parity.
- **Hardware priority**: Optimize first bring-up for Ultra 205 BM1366 - other upstream boards remain in scope but require their own evidence before verification claims.
- **Architecture**: Prefer functional core and imperative shell - pure logic belongs in testable crates, while ESP-IDF, FreeRTOS, Wi-Fi, NVS, SPIFFS, OTA, serial, GPIO, I2C, ADC, power, display, and task orchestration stay in firmware adapters.
- **Licensing**: Keep original work MIT-first where legally possible, but mark intentionally ported GPL-covered source expression as GPL-3.0-compatible and review distributed firmware artifacts before release.
- **Safety**: Hardware-control surfaces such as voltage, fan, thermal, power, and ASIC initialization require hardware evidence before verified parity.

<!-- GSD:project-end -->

<!-- GSD:stack-start source:research/STACK.md -->

## Technology Stack

## Summary

## Recommended Stack

### Firmware Toolchain

| Layer | Recommendation | Version / Pin | Confidence | Why |
| --- | --- | --- | --- | --- |
| ESP-IDF | Pin ESP-IDF tag through `esp-idf-sys` metadata | `v5.5.4` | HIGH | Current released Rust crates support ESP-IDF `v5.5.x`; ESP-IDF 6 support is not yet a safe baseline. |
| Rust ESP toolchain | Install with `espup` and use the `esp` toolchain for firmware Cargo builds | `espup v0.17.1`, `espup install --targets esp32s3 --std` | HIGH | Ultra 205 uses ESP32-S3-class Xtensa firmware tooling; `espup` owns the custom toolchain setup. |
| Firmware target | Build first firmware for ESP32-S3 ESP-IDF | `xtensa-esp32s3-espidf`, `MCU=esp32s3` | MEDIUM | Project docs now prioritize Ultra 205 after ADR-0014; confirm board-specific details from the reference tree during hardware phases. |
| Rust firmware crates | Depend primarily on `esp-idf-svc`; use `hal` and `sys` through re-exports unless direct dependency is required | `esp-idf-svc 0.52.1`, resolving `esp-idf-hal 0.46.2`, `esp-idf-sys 0.37.2` | HIGH | `esp-idf-svc` wraps Wi-Fi, HTTP, NVS, OTA, logging, MQTT, event loop, timers, and re-exports lower layers. |
| Rust edition | Use Rust 2021 for all firmware and shared crates initially | `edition = "2021"` | MEDIUM | The ESP-IDF Rust ecosystem still documents and publishes crates on 2021; upgrade to 2024 only after firmware build/flash is stable. |
| Firmware logging | Use the `log` facade with `esp_idf_svc::log::EspLogger` in firmware; avoid `println!` | crate-managed | HIGH | This matches ESP-IDF logging integration. Host tools may use `tracing`. |

### Bazel Automation

| Layer | Recommendation | Version / Pin | Confidence | Why |
| --- | --- | --- | --- | --- |
| Bazel launcher | Use Bazelisk and check in `.bazelversion` | `9.1.1` | HIGH | Bazel 9 is the current active LTS; Bazelisk is the official recommended version manager. |
| Bazel modules | Use Bzlmod, not WORKSPACE-first setup | `MODULE.bazel` | HIGH | `rules_rust` and Bazel Central Registry are Bzlmod-first in current docs. |
| Rust rules | Use `rules_rust` for pure crates and host tools | `rules_rust 0.70.0` | HIGH | Current BCR release supports Bazel 7, 8, and 9. |
| Cargo dependency mirror | Use `crate_universe` from `rules_rust` against the repo `Cargo.lock` | `rules_rust 0.70.0` | MEDIUM-HIGH | Current official path for Cargo dependencies in Bazel. |
| Firmware build target | Use Bazel targets that invoke repo-owned Cargo/ESP-IDF scripts for `firmware/bitaxe` | project-owned wrapper | MEDIUM | No mature official Bazel ESP-IDF Rust rule was found; wrapping is the lowest-risk integration. |
| Packaging | Produce flashable `.bin` images as declared Bazel outputs | project-owned wrapper around `espflash save-image` or Cargo build artifacts | MEDIUM | Keeps artifacts visible in Bazel without replacing ESP-IDF image construction. |

### Human Command Surface

| Command | Backing implementation | Required behavior |
| --- | --- | --- |
| `just build` | `bazel build //firmware/bitaxe:firmware` | Build the canonical firmware target. |
| `just test` | `bazel test //...` or a scoped test target group | Run pure crate, host tool, and script tests. Hardware tests stay explicit. |
| `just package` | `bazel build //firmware/bitaxe:firmware_image` | Produce image artifacts and print paths. |
| `just flash board=205 [port=...]` | `bazel run //tools/flash -- flash --board 205 ...` | Build/package first unless image override is provided; fail clearly on missing/ambiguous port. |
| `just monitor [port=...]` | `bazel run //tools/flash -- monitor ...` | Open serial monitor without flashing. |
| `just flash-monitor board=205 [port=...]` | `bazel run //tools/flash -- flash-monitor ...` | Flash then monitor, capture command/log evidence when requested. |
| `just verify-reference` | `bazel run //scripts:verify_reference_clean` | Fail if `reference/esp-miner` is missing or locally modified. |
| `just parity` | `bazel run //tools/parity:report` | Summarize checklist status and missing evidence. |

### Host Tools And Parity Tooling

| Tool / Crate | Use | Recommendation |
| --- | --- | --- |
| `espflash` / `cargo-espflash` | USB flash, monitor, image save, port list, board info | Use as the flashing backend. Prefer `espflash` when Bazel already produced an ELF/image; allow `cargo-espflash` for developer diagnostics. |
| `espup` | ESP Rust toolchain installer | Use `cargo install espup --locked` or release binary, then `espup install --targets esp32s3 --std`. |
| `ldproxy` | ESP-IDF Rust linker proxy | Install through the ESP Rust setup path; configure as linker for `xtensa-esp32s3-espidf`. |
| `clap` | Host CLI parsing | Use for `tools/flash`, `tools/parity`, and optional `tools/xtask`. |
| `serde`, `serde_json`, `csv`, `toml` | API/config/parity fixtures | Use typed parsing for upstream config, API models, and checklist/golden data. |
| `camino`, `ignore`, `walkdir` | Host tooling paths and scans | Prefer over ad hoc path/string walking in parity tools. |
| `thiserror` / `anyhow` | Errors | `thiserror` for library crates, `anyhow` for CLI/tools. |
| `heapless` | Firmware/protocol buffers | Use for fixed-size ASIC/protocol buffers where allocation should be explicit. |
| `insta` or explicit checked-in golden fixtures | Host snapshot/golden tests | Use for host-only parity reports and API/model snapshots; do not treat snapshots as hardware evidence. |

## Tooling Details

### Rust + ESP-IDF Build

### Bazel + Cargo Boundary

- `Cargo.toml` and `Cargo.lock` define Rust package versions.
- `rules_rust` + `crate_universe` mirror Cargo dependencies into Bazel.
- Bazel `rust_library` and `rust_test` build pure crates and host tools directly.
- Bazel firmware targets call a repo-owned script or `xtask` that runs Cargo with the `esp` toolchain and writes declared artifacts.
- `just` calls Bazel only.

### USB Flashing

- Map `board=205` to Ultra 205 defaults and target image.
- Accept explicit `port`.
- If `port` is omitted, call `espflash list-ports` and apply project heuristics.
- On zero ports, fail with install/permission hints.
- On multiple likely ports, fail with a chooser-style message and exact `port=` examples.
- Print the underlying `espflash` command before executing.
- Write optional smoke logs into a parity evidence path when requested.

### Test Strategy

| Layer | Tooling | Required evidence |
| --- | --- | --- |
| Pure unit tests | Bazel `rust_test` and/or Cargo tests for `crates/*` | Deterministic Arrange/Act/Assert tests for ASIC packet formats, config parsing, Stratum messages, API model serialization, and parity report logic. |
| Golden/fixture tests | Checked-in JSON/CSV/binary fixtures; optional `insta` for host snapshots | Rust outputs match upstream-derived fixtures. Each fixture must record provenance. |
| Firmware compile/package | Bazel firmware wrapper target | ESP32-S3 firmware builds and produces ELF/bin artifacts with ESP-IDF `v5.5.4`. |
| USB smoke | `just flash-monitor board=205 port=...` | Captured log shows boot, app identity, ESP-IDF version, reset reason, and safe no-op/minimal hardware state. |
| Hardware regression | Explicit `just hardware-smoke` or `just hil board=205 port=...` later | Repeatable tests for voltage, fan, thermal, ASIC init, and mining behavior. Required before safety-critical parity is `verified`. |

## Version/Source Notes

### Primary Sources

- Local project decisions: `.planning/PROJECT.md`, `docs/project/gsd-new-project-brief.md`, `docs/project/seed-layout.md`, `PROVENANCE.md`.
- Bright Builds local rules: `AGENTS.md`, `AGENTS.bright-builds.md`, `standards/core/architecture.md`, `standards/core/verification.md`, `standards/core/testing.md`, `standards/languages/rust.md`.
- ESP-IDF versions and support policy: https://docs.espressif.com/projects/esp-idf/en/stable/esp32/versions.html
- ESP-IDF releases: https://github.com/espressif/esp-idf/releases
- ESP-IDF build system: https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-guides/build-system.html
- ESP-IDF Build System v2: https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-guides/build-system-v2.html
- `esp-idf-sys` docs/readme/build options: https://docs.rs/crate/esp-idf-sys/latest and https://github.com/esp-rs/esp-idf-sys
- `esp-idf-sys` changelog: https://raw.githubusercontent.com/esp-rs/esp-idf-sys/master/CHANGELOG.md
- `esp-idf-hal` changelog: https://raw.githubusercontent.com/esp-rs/esp-idf-hal/master/CHANGELOG.md
- `esp-idf-svc` changelog: https://raw.githubusercontent.com/esp-rs/esp-idf-svc/master/CHANGELOG.md
- `espup` README and releases: https://github.com/esp-rs/espup and https://github.com/esp-rs/espup/releases
- `espflash` README: https://github.com/esp-rs/espflash and raw package READMEs under `espflash/` and `cargo-espflash/`.
- Bazel release model: https://bazel.build/release
- Bazelisk docs: https://bazel.build/install/bazelisk
- `rules_rust` docs and BCR: https://bazelbuild.github.io/rules_rust/ and https://registry.bazel.build/modules/rules_rust
- `just` manual and releases: https://just.systems/man/en/ and https://github.com/casey/just/releases
- Rust latest release: https://blog.rust-lang.org/releases/latest/

### Important Source Interpretation

## What Not To Use

| Do not use | Why | Use instead |
| --- | --- | --- |
| ESP-IDF `v6.0.1` as first baseline | Latest stable, but Rust crate support is not fully released and IDF 6 moves some drivers/components. | ESP-IDF `v5.5.4`; schedule a reassessment when `esp-idf-svc`/`hal`/`sys` release full IDF 6 support. |
| ESP-IDF `master`, release branches, or unpinned IDF | Breaks reproducibility and parity evidence. | Pin exact tags/commits in Cargo metadata and record them in evidence. |
| `esp-idf-sys` default ESP-IDF version | Current docs still show an old default. | Explicit `esp_idf_version = "tag:v5.5.4"`. |
| PlatformIO (`pio`) builder for production | `esp-idf-sys` documents it as a backup and less flexible than native. | Native ESP-IDF builder through `esp-idf-sys`. |
| Bare-metal `esp-hal`/`no_std` as first stack | Project needs ESP-IDF services for parity; bare-metal path would rebuild too much platform behavior. | ESP-IDF Rust `std` stack. |
| Embassy/async-first firmware architecture | Adds scheduler/executor complexity before parity surfaces are known. | FreeRTOS/task-oriented imperative shell first; introduce async only for clear service needs. |
| Full Bazel-native ESP-IDF cross-toolchain on day one | High integration risk; fights Cargo build scripts and ESP-IDF CMake/component flow. | Bazel-owned wrapper around Cargo/ESP-IDF for firmware, direct Bazel for pure crates/tools. |
| Direct `esptool.py` as normal UX | Lower-level and less integrated with Rust ESP-IDF artifacts. | `espflash` backend behind `tools/flash` and `just`. |
| `cargo-espmonitor` | Functionality has moved into `espflash`/`cargo-espflash` monitor commands. | `espflash monitor` or `cargo espflash monitor`. |
| `defmt` as initial logging format | Mainly useful in no_std ESP stacks; ESP-IDF std path already has serial ESP logging. | `log` facade with `EspLogger`. |
| Copying upstream C expression into MIT Rust files | GPL provenance risk. | Use breadcrumbs, independent Rust design, isolated GPL-marked files if porting expression is intentional. |
| Snapshot/golden tests as hardware parity proof | They prove deterministic outputs only, not real device behavior. | Hardware smoke/regression evidence for verified safety and hardware-control parity. |

## Risks

| Risk | Severity | Mitigation | Confidence |
| --- | --- | --- | --- |
| ESP-IDF 5.5 baseline falls behind latest security/feature changes | Medium | Track ESP-IDF releases; add a roadmap phase to trial ESP-IDF 6 after Rust crates release support. | MEDIUM |
| Rust ESP-IDF crates lack HIL coverage | High | Treat hardware smoke/regression evidence as mandatory for parity; keep firmware shell thin. | HIGH |
| Bazel/Cargo dual graph drifts | Medium | Make Cargo.lock authoritative; use `crate_universe`; add `just repin`/Bazel repin workflow later; CI checks generated lock consistency. | MEDIUM |
| First build downloads large toolchains and IDF sources | Medium | Use `espup --targets esp32s3 --std`; cache Cargo, `.embuild`, Bazel repository cache in CI; add `just doctor`. | HIGH |
| Firmware Bazel wrapper has non-hermetic edges | Medium | Keep wrapper narrow, declare outputs, print versions, and isolate mutable tool caches outside source paths. | MEDIUM |
| USB serial behavior varies by OS and cable/chip bridge | Medium | Use `espflash list-ports`, clear `port=` errors, Linux `dialout` hint, WSL warning, and explicit port override. | HIGH |
| Ultra 205 board details need confirmation from reference tree | Medium | Verify board-specific config, ASIC, sensor, and power paths before enabling hardware control. | MEDIUM |
| Power/voltage/fan/ASIC bring-up can damage hardware if guessed | High | First milestone should boot/log only; require hardware evidence and safety guards before enabling control paths. | HIGH |
| GPL provenance contaminates MIT-only files | High | Keep reference read-only, use breadcrumbs, isolate intentionally ported expression, perform release artifact review. | HIGH |

## Confidence

| Area | Confidence | Notes |
| --- | --- | --- |
| ESP-IDF Rust stack | HIGH | Confirmed by accepted project decision plus `esp-idf-svc` coverage and ESP-IDF service needs. |
| ESP-IDF `v5.5.4` baseline | MEDIUM-HIGH | Prescriptive choice based on released crate compatibility. Newer ESP-IDF 6 exists, but Rust release support is not yet the safer baseline. |
| Bazel 9 + `rules_rust 0.70.0` | HIGH | Current official/BCR sources show Bazel 9 active and `rules_rust` tested on Bazel 9. |
| Bazel wrapper around Cargo/ESP-IDF firmware build | MEDIUM | No mature official Bazel ESP-IDF Rust stack found; wrapper approach is pragmatic and aligned with `esp-idf-sys`. |
| `just` command surface | HIGH | Matches accepted project decision and `just` is explicitly a command runner, not a build system. |
| USB flashing stack | HIGH | `espflash` directly supports ESP32-S3 and provides required list/flash/monitor/save-image commands. |
| Test strategy | MEDIUM-HIGH | Pure/fixture/hardware layering is strongly aligned with local Bright Builds rules and parity policy; exact HIL harness can be refined after first boot. |
| Ultra 205 target triple | MEDIUM | Project docs point to Ultra 205 BM1366; the current safe-state firmware target is `xtensa-esp32s3-espidf`. |

<!-- GSD:stack-end -->

<!-- GSD:conventions-start source:CONVENTIONS.md -->

## Conventions

Conventions not yet established. Will populate as patterns emerge during development.

<!-- GSD:conventions-end -->

<!-- GSD:architecture-start source:ARCHITECTURE.md -->

## Architecture

Architecture not yet mapped. Follow existing patterns found in the codebase.

<!-- GSD:architecture-end -->

<!-- GSD:skills-start source:skills/ -->

## Project Skills

No project skills found. Add skills to any of: `.claude/skills/`, `.agents/skills/`, `.cursor/skills/`, or `.github/skills/` with a `SKILL.md` index file.

<!-- GSD:skills-end -->

<!-- GSD:workflow-start source:GSD defaults -->

## GSD Workflow Enforcement

Before using Edit, Write, or other file-changing tools, start work through a GSD command so planning artifacts and execution context stay in sync.

Use these entry points:

- `/gsd-quick` for small fixes, doc updates, and ad-hoc tasks
- `/gsd-debug` for investigation and bug fixing
- `/gsd-execute-phase` for planned phase work

Do not make direct repo edits outside a GSD workflow unless the user explicitly asks to bypass it.

<!-- GSD:workflow-end -->

## Repo-Local Guidance

### ESP-IDF Tooling Preference

- Treat ESP-IDF as a standard repository dependency through the pinned `esp-idf-sys` metadata, the checked-in `.cargo/config.toml`, and the ESP Rust toolchain installed by `espup`.
- Prefer ESP-IDF and esp-rs tooling when it satisfies firmware build, package, flash, monitor, partition, image-generation, OTA, SPIFFS, NVS, FreeRTOS, and logging needs. Use `esp-idf-sys`/`embuild`, `espup`, `ldproxy`, and `espflash` before custom CMake, PlatformIO, or manually managed ESP-IDF installs.
- Treat `.embuild/` as local, gitignored, generated ESP-IDF/esp-rs tool state. Do not commit or hand-edit it, but repo automation may use managed tools from `.embuild/espressif`, including `spiffsgen.py`, `gen_esp32part.py`, and `esptool.py`, when the pinned ESP-IDF workflow has generated them.
- Prefer `espflash` where it suffices for flashing, monitoring, and ELF/app image generation. When `espflash` cannot cover an ESP-supported workflow such as arbitrary address/data-partition image merging, use managed `.embuild` ESP-IDF tools such as `esptool.py merge_bin` before adding custom binary manipulation.
- If ESP-IDF/esp-rs tooling is insufficient for a concrete workflow, document the reason in repo-local guidance, an ADR, or the relevant phase artifact before adding an alternate tool path.
- Use `just doctor` for read-only contributor dependency checks and `just bootstrap-esp` for the explicit opt-in ESP tooling installer. `just doctor` intentionally calls a script directly because it must diagnose missing Bazel or ESP prerequisites before Bazel can run.

### Autonomous Ultra 205 Hardware Verification

- The user grants standing permission for agents to autonomously interact with a connected Bitaxe Ultra 205 over USB for current phase verification.
- Before autonomous hardware use, run `just detect-ultra205`. Treat detection as successful only when it finds exactly one likely ESP USB serial port and `espflash board-info --chip esp32s3 --port <port> --non-interactive` succeeds.
- If detection succeeds, use the printed `port=<path>` with repo commands such as `just flash-monitor board=205 port=<path> evidence-dir=<path>` and record the detector output in evidence.
- If detection succeeds and the ignored local file `wifi-credentials.json` exists, agents may pass `wifi-credentials=wifi-credentials.json` to repo-owned `just flash` or `just flash-monitor` commands for developer bring-up. Do not read, print, summarize, or commit the credential file contents.
- If detection succeeds and an ignored local `pool-credentials*.json` file exists, agents may pass it with `--pool-credentials <path>` only as local runtime input for repo-owned live mining test or verification commands. Use the committed `pool-credentials.json.example` file for shape only; the real local file may define `poolURL`, `poolPort`, `poolUser`, and `poolPassword`, with `poolUser` derived from the owner BTC address, for example `<owner-btc-address>.bitaxe`.
- Treat pool owner addresses, worker strings, endpoints, ports, passwords, and credential file contents as sensitive local test inputs. Do not print, summarize, commit, copy into evidence, or expose the real file contents. Committed evidence may record category labels such as `pool_config: local-owner-supplied` only, never raw pool URLs, ports, users, workers, passwords, addresses, endpoints, tokens, or NVS secret values.
- Local developer hardware evidence may keep USB-observed SSIDs, IP addresses, MAC addresses, and `device_url` values to make bring-up and UAT practical. Evidence intended for commit or sharing must be produced with `redact-evidence=true` or otherwise redacted before promotion.
- When a repo-owned test or verification command needs `DEVICE_URL`, agents may derive it from fresh monitor output only when the same current verification session has already passed `just detect-ultra205`, the monitor output came from the corresponding repo-owned `just flash-monitor` or `just monitor` run, exactly one origin-only `http://...` or `https://...` candidate is present, and the value is used only as a local runtime input. Do not infer `DEVICE_URL` from mDNS, ARP, router state, scans, stale logs, or unrelated prior evidence. Do not use the value when zero, multiple, redacted, malformed, or stale candidates are found. Never commit raw `DEVICE_URL`, IPs, endpoints, MACs, Wi-Fi values, pool credentials, workers, passwords, tokens, or NVS secret values in evidence.
- Stop and ask or record hardware evidence pending when there are zero likely ports, multiple likely ports, `board-info` fails, the target is not board `205`, or required recovery/evidence instructions are missing.
- Phase-gated destructive or fault-injection verification is allowed only when the active phase plan documents the recovery path and required evidence. Do not run ad hoc erase, rollback, interrupted-update, voltage/fan/mining stress, or raw write commands outside documented phase-gated procedures.
- Every hardware run must record board `205`, selected port, source commit, reference commit, package manifest/artifacts when applicable, exact commands, `board-info` output, captured logs, observed behavior, and conclusion. Do not commit secrets, pool credentials, Wi-Fi credentials, private endpoints, or NVS secret values in evidence.

### Flash And Monitor Timeouts

Ultra 205 factory reflash, NVS seed, boot, Wi-Fi join, pool-input-bridge, and post-flash runtime evidence routinely exceed short monitor defaults. Agents and repo-owned wrappers must budget accordingly:

1. **Minimum capture timeout:** use at least **6 minutes (360 seconds)** for `just flash`, `just flash-monitor`, `just monitor`, and equivalent `bazel run //tools/flash` invocations when flashing or reflashing real hardware, unless a phase plan or deterministic test fixture documents a shorter bound explicitly.
2. **Explicit override required:** `tools/flash` defaults to a 25-second monitor capture, which is insufficient for hardware evidence. Always pass `capture-timeout-seconds=360` (or higher) on `flash-monitor` / `monitor` commands for Ultra 205 bring-up and evidence capture.
3. **Phase evidence wrappers:** when `--duration-seconds` governs post-flash monitor capture (`phase25-evidence`, `phase27-evidence`, and similar), use **≥ 360** for hardware mode unless the phase plan documents a shorter deterministic test bound.
4. **Agent/shell wall clock:** command and tool timeouts must exceed the flash plus monitor budget; prefer **≥ 420 seconds** wall clock when using a 360-second capture timeout.
5. **Do not treat early exit as failure** until the full timeout elapses unless the repo-owned flash tool reports a hard error (flash/write failure, missing trusted boot markers after complete capture, and similar).

### Ultra 205 Serial Session Reuse

- Treat bare `espflash monitor --no-reset` as reset-capable, not passive. For retained-runtime ESP32-S3 capture, require the complete command contract `--chip esp32s3 --before no-reset-no-sync --after no-reset --no-reset --non-interactive`.
- Before and after a bounded monitor session, record the selected device node, USB enumeration/session identity when available, process PID/PGID and descendants, and serial file-descriptor holders. Process death alone is not cleanup proof: require the process tree reaped and no unexpected holder on the selected serial node before reuse.
- Keep detailed session traces only in mode-0600 files under mode-0700 gitignored roots. Committed or shared evidence may contain only redacted categories, counts, durations, booleans, and trace digests; never promote device paths, USB serial identities, PIDs, commands containing secrets, or raw local identifiers.
- Record barrel/DC power and USB power independently. Distinguish USB re-enumeration, warm reset, and true both-power cold start in every checkpoint and trace; USB removal while barrel power remains is not a cold start.
- A port node appearing is not sufficient readiness after re-enumeration. Require the repo-owned bounded stability/ownership gate before opening the passive monitor, and fail closed on identity change, unexpected ownership, or unproved cleanup.

### Evidence Workflow: Hardware-First Default

Agents executing phase evidence wrappers (`phase23-evidence`, `phase25-evidence`, `phase27-evidence`, and future `phase*-evidence` scripts with `blocked|hardware` modes) must:

1. **Always attempt hardware path first** when the phase scope includes detector-gated Ultra 205 evidence:
   - Run `just detect-ultra205` before any flash/monitor/evidence work.
   - If detection passes **and** required local credential files exist, run `--mode hardware` (not `--mode blocked`).
2. **Use blocked mode only as fallback** when:
   - Detection fails (zero/ambiguous ports, board-info failure),
   - Required credentials are absent,
   - User explicitly requests CI-safe/static workflow proof, or
   - Phase plan explicitly requires blocked mode for deterministic Bazel tests only.
3. **Never skip detection** and jump to blocked mode when a board may be connected (standing permission in Autonomous Ultra 205 section applies).
4. **Build phase-correct firmware** before flash when compile-time evidence mode gates exist (Phase 21/25/27 pattern); default `just build` is fail-closed. Use `scripts/phase27-live-hardware-bridge-package.sh` or equivalent `action_env` for Phase 27 enablement.
5. **Promotion:** hardware artifacts intended for commit must pass redaction review (`redact-evidence=true` or equivalent) before updating `docs/parity/evidence/`.

### Agent-Performed Simple UAT

- When starting or resuming `$gsd-verify-work`, agents may complete simple UAT checkpoints without waiting for the user only when the expected behavior is objectively verifiable from repo artifacts or non-destructive commands.
- Treat simple objective UAT as static inspection, committed evidence review, redaction checks, lifecycle checks, parity/reference checks, Bazel/Cargo/Just checks, and other deterministic repo-local verification.
- For auto-passed UAT checkpoints, record `result: pass`, `verified_by: agent`, and an `evidence:` line citing exact commands, artifact paths, or concise observations.
- Stop at the first checkpoint that needs human judgment, subjective product review, secret access, external accounts, raw unredacted endpoint review, destructive or fault-injection flows, unsafe hardware action, missing prerequisites, ambiguous interpretation, or unstated target discovery. Leave that checkpoint pending or blocked and report what user input or prerequisite is needed.
- This rule does not expand Autonomous Ultra 205 permissions. Hardware UAT still must follow the detector gate, evidence requirements, redaction rules, and phase-gated destructive/fault-injection limits above.

### Frontmatter-Parsed Markdown

- In GSD artifacts and other Markdown files parsed with YAML frontmatter, use standalone `---` only for the opening and closing frontmatter delimiters at the top of the file.
- Do not use standalone `---` as a body separator after frontmatter; the GSD parser scans all such blocks and may treat the last pair as frontmatter, breaking lifecycle validation. Use headings or `***` for body breaks instead.
- Markdown table separator rows such as `| --- | --- |` are valid and are not affected by this rule.

# Stack Research: Bitaxe Rust ESP-IDF Firmware

**Project:** Bitaxe Rust ESP-IDF firmware rewrite
**Research date:** 2026-06-20
**Research type:** Stack dimension
**Overall confidence:** MEDIUM-HIGH

## Summary

Use a Rust ESP-IDF `std` firmware stack, not bare-metal Rust, for the first production firmware path. The project depends on ESP-IDF services that upstream ESP-Miner already uses: Wi-Fi, FreeRTOS, HTTP/WebSocket serving, NVS, partitions, OTA, SPIFFS/static assets, logging, serial flashing, and ESP image conventions. That matches the accepted project decisions and avoids rebuilding a large platform layer before parity work begins.

Pin the firmware baseline to ESP-IDF `v5.5.4`, not current ESP-IDF `v6.0.1`, until the released `esp-idf-svc`, `esp-idf-hal`, and `esp-idf-sys` crates ship full ESP-IDF 6 support. ESP-IDF `v6.0.1` is the latest stable release, but current released Rust ESP-IDF crates explicitly added compatibility for ESP-IDF `v5.4.x` and `v5.5.x`; ESP-IDF 6 support appears in unreleased/basic changelog entries and includes component movement risks. For this project, a verified firmware baseline is more valuable than the newest IDF tag.

Bazel should be canonical automation, but it should not initially replace Cargo plus `esp-idf-sys` for the Xtensa ESP-IDF firmware build. Use Bazel 9 with Bzlmod and `rules_rust` for pure Rust crates, host tools, parity tooling, and workflow graph targets. For the firmware app, let Bazel invoke a repo-owned Cargo/ESP-IDF wrapper target that produces declared images and logs. This keeps local and CI automation unified while avoiding a custom Bazel ESP-IDF toolchain rewrite on day one.

`just` should remain only the human command surface. Every normal recipe should delegate to Bazel targets or thin scripts that Bazel also owns. A connected Gamma 601 should be handled through `just flash board=601`, `just monitor`, and `just flash-monitor`, backed by `espflash`/`cargo-espflash` and project-owned port discovery/error handling.

## Recommended Stack

### Firmware Toolchain

| Layer | Recommendation | Version / Pin | Confidence | Why |
| --- | --- | --- | --- | --- |
| ESP-IDF | Pin ESP-IDF tag through `esp-idf-sys` metadata | `v5.5.4` | HIGH | Current released Rust crates support ESP-IDF `v5.5.x`; ESP-IDF 6 support is not yet a safe baseline. |
| Rust ESP toolchain | Install with `espup` and use the `esp` toolchain for firmware Cargo builds | `espup v0.17.1`, `espup install --targets esp32s3 --std` | HIGH | Gamma 601 is expected to use ESP32-S3-class Xtensa firmware tooling; `espup` owns the custom toolchain setup. |
| Firmware target | Build first firmware for ESP32-S3 ESP-IDF | `xtensa-esp32s3-espidf`, `MCU=esp32s3` | MEDIUM | Project docs prioritize Gamma 601; confirm with upstream reference once the submodule exists. |
| Rust firmware crates | Depend primarily on `esp-idf-svc`; use `hal` and `sys` through re-exports unless direct dependency is required | `esp-idf-svc 0.52.1`, resolving `esp-idf-hal 0.46.2`, `esp-idf-sys 0.37.2` | HIGH | `esp-idf-svc` wraps Wi-Fi, HTTP, NVS, OTA, logging, MQTT, event loop, timers, and re-exports lower layers. |
| Rust edition | Use Rust 2021 for all firmware and shared crates initially | `edition = "2021"` | MEDIUM | The ESP-IDF Rust ecosystem still documents and publishes crates on 2021; upgrade to 2024 only after firmware build/flash is stable. |
| Firmware logging | Use the `log` facade with `esp_idf_svc::log::EspLogger` in firmware; avoid `println!` | crate-managed | HIGH | This matches ESP-IDF logging integration. Host tools may use `tracing`. |

Suggested firmware crate metadata:

```toml
[package.metadata.esp-idf-sys]
esp_idf_version = "tag:v5.5.4"
esp_idf_tools_install_dir = "workspace"
esp_idf_sdkconfig = "sdkconfig"
esp_idf_sdkconfig_defaults = ["sdkconfig.defaults"]
```

Suggested firmware Cargo target configuration:

```toml
[build]
target = "xtensa-esp32s3-espidf"

[target.xtensa-esp32s3-espidf]
linker = "ldproxy"

[unstable]
build-std = ["std", "panic_abort"]

[env]
MCU = "esp32s3"
```

Add `--cfg espidf_time64` and `-C default-linker-libraries` through the template-compatible config path used by `esp-idf-template`; `esp-idf-sys` documents `espidf_time64` as required for ESP-IDF 5.0 and later.

### Bazel Automation

| Layer | Recommendation | Version / Pin | Confidence | Why |
| --- | --- | --- | --- | --- |
| Bazel launcher | Use Bazelisk and check in `.bazelversion` | `9.1.1` | HIGH | Bazel 9 is the current active LTS; Bazelisk is the official recommended version manager. |
| Bazel modules | Use Bzlmod, not WORKSPACE-first setup | `MODULE.bazel` | HIGH | `rules_rust` and Bazel Central Registry are Bzlmod-first in current docs. |
| Rust rules | Use `rules_rust` for pure crates and host tools | `rules_rust 0.70.0` | HIGH | Current BCR release supports Bazel 7, 8, and 9. |
| Cargo dependency mirror | Use `crate_universe` from `rules_rust` against the repo `Cargo.lock` | `rules_rust 0.70.0` | MEDIUM-HIGH | Current official path for Cargo dependencies in Bazel. |
| Firmware build target | Use Bazel targets that invoke repo-owned Cargo/ESP-IDF scripts for `firmware/bitaxe` | project-owned wrapper | MEDIUM | No mature official Bazel ESP-IDF Rust rule was found; wrapping is the lowest-risk integration. |
| Packaging | Produce flashable `.bin` images as declared Bazel outputs | project-owned wrapper around `espflash save-image` or Cargo build artifacts | MEDIUM | Keeps artifacts visible in Bazel without replacing ESP-IDF image construction. |

Canonical target shape:

```text
//firmware/bitaxe:firmware          # builds ELF through Cargo/ESP-IDF wrapper
//firmware/bitaxe:firmware_image    # emits flashable image artifacts
//crates/bitaxe-core:tests          # Bazel/rules_rust tests
//crates/bitaxe-asic:tests
//crates/bitaxe-stratum:tests
//crates/bitaxe-config:tests
//crates/bitaxe-api:tests
//tools/flash:flash
//tools/parity:report
//scripts:verify_reference_clean
```

### Human Command Surface

| Command | Backing implementation | Required behavior |
| --- | --- | --- |
| `just build` | `bazel build //firmware/bitaxe:firmware` | Build the canonical firmware target. |
| `just test` | `bazel test //...` or a scoped test target group | Run pure crate, host tool, and script tests. Hardware tests stay explicit. |
| `just package` | `bazel build //firmware/bitaxe:firmware_image` | Produce image artifacts and print paths. |
| `just flash board=601 [port=...]` | `bazel run //tools/flash -- flash --board 601 ...` | Build/package first unless image override is provided; fail clearly on missing/ambiguous port. |
| `just monitor [port=...]` | `bazel run //tools/flash -- monitor ...` | Open serial monitor without flashing. |
| `just flash-monitor board=601 [port=...]` | `bazel run //tools/flash -- flash-monitor ...` | Flash then monitor, capture command/log evidence when requested. |
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

The firmware app should be an ESP-IDF Rust binary under `firmware/bitaxe`. It should own ESP-IDF adapters: FreeRTOS tasks, Wi-Fi, NVS, HTTP/WebSocket server, OTA, SPIFFS/static assets, GPIO, I2C, ADC, fan/power/thermal control, serial logging, and board bring-up.

Pure Rust crates under `crates/*` must not depend on ESP-IDF. Keep ASIC packet layout, Stratum parsing, config validation, API models, and parity fixture logic in pure crates so Bazel and Cargo can unit test them without hardware.

Use `esp-idf-svc` as the direct firmware dependency because it covers the services this project needs and re-exports `esp-idf-hal` and `esp-idf-sys`. Add direct `esp-idf-hal` or `esp-idf-sys` dependencies only when a wrapper is missing and the code genuinely needs HAL/raw bindings.

Do not rely on `esp-idf-sys` defaults. Its build-options docs still document an old default ESP-IDF version. This project must pin `esp_idf_version = "tag:v5.5.4"` explicitly.

### Bazel + Cargo Boundary

Treat Cargo as the Rust package/dependency authority and Bazel as the automation authority.

Recommended rule:

- `Cargo.toml` and `Cargo.lock` define Rust package versions.
- `rules_rust` + `crate_universe` mirror Cargo dependencies into Bazel.
- Bazel `rust_library` and `rust_test` build pure crates and host tools directly.
- Bazel firmware targets call a repo-owned script or `xtask` that runs Cargo with the `esp` toolchain and writes declared artifacts.
- `just` calls Bazel only.

This deliberately avoids a day-one attempt to make Bazel own the ESP-IDF CMake/Ninja/component build graph. ESP-IDF has its own build system, and `esp-idf-sys` integrates through Cargo build scripts. A full custom Bazel toolchain can be revisited only after the firmware boots and parity-critical modules are stable.

### USB Flashing

Use `espflash` as the backend, not direct `esptool.py` as the normal developer command. `espflash` supports ESP32-S2/S3, has `list-ports`, `flash`, `monitor`, `save-image`, `board-info`, partition conversion, and config files for project and port settings.

`tools/flash` should provide the project-specific UX:

- Map `board=601` to Gamma 601 defaults and target image.
- Accept explicit `port`.
- If `port` is omitted, call `espflash list-ports` and apply project heuristics.
- On zero ports, fail with install/permission hints.
- On multiple likely ports, fail with a chooser-style message and exact `port=` examples.
- Print the underlying `espflash` command before executing.
- Write optional smoke logs into a parity evidence path when requested.

### Test Strategy

Use a layered test strategy:

| Layer | Tooling | Required evidence |
| --- | --- | --- |
| Pure unit tests | Bazel `rust_test` and/or Cargo tests for `crates/*` | Deterministic Arrange/Act/Assert tests for ASIC packet formats, config parsing, Stratum messages, API model serialization, and parity report logic. |
| Golden/fixture tests | Checked-in JSON/CSV/binary fixtures; optional `insta` for host snapshots | Rust outputs match upstream-derived fixtures. Each fixture must record provenance. |
| Firmware compile/package | Bazel firmware wrapper target | ESP32-S3 firmware builds and produces ELF/bin artifacts with ESP-IDF `v5.5.4`. |
| USB smoke | `just flash-monitor board=601 port=...` | Captured log shows boot, app identity, ESP-IDF version, reset reason, and safe no-op/minimal hardware state. |
| Hardware regression | Explicit `just hardware-smoke` or `just hil board=601 port=...` later | Repeatable tests for voltage, fan, thermal, ASIC init, and mining behavior. Required before safety-critical parity is `verified`. |

Do not mark parity verified from implementation or simulation alone. Safety-critical surfaces such as voltage, fan, thermal, power, and ASIC initialization require Gamma 601 hardware evidence.

## Version/Source Notes

### Primary Sources

- Local project decisions: `.planning/PROJECT.md`, `docs/project/gsd-new-project-brief.md`, `docs/project/seed-layout.md`, `PROVENANCE.md`.
- Bright Builds local rules: `AGENTS.md`, `AGENTS.bright-builds.md`, `standards/core/architecture.md`, `standards/core/verification.md`, `standards/core/testing.md`, `standards/languages/rust.md`.
- ESP-IDF versions and support policy: https://docs.espressif.com/projects/esp-idf/en/stable/esp32/versions.html
- ESP-IDF releases: https://github.com/espressif/esp-idf/releases
  - `v6.0.1` is current latest stable as of research.
  - `v5.5.4` is the recommended Rust-compatible firmware baseline for this project.
- ESP-IDF build system: https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-guides/build-system.html
- ESP-IDF Build System v2: https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-guides/build-system-v2.html
  - v2 is technical preview and not recommended for production baseline.
- `esp-idf-sys` docs/readme/build options: https://docs.rs/crate/esp-idf-sys/latest and https://github.com/esp-rs/esp-idf-sys
- `esp-idf-sys` changelog: https://raw.githubusercontent.com/esp-rs/esp-idf-sys/master/CHANGELOG.md
  - `0.37.0` adds ESP-IDF `v5.4.x` and `v5.5.x` compatibility and deprecates ESP-IDF `<5.3.0`.
- `esp-idf-hal` changelog: https://raw.githubusercontent.com/esp-rs/esp-idf-hal/master/CHANGELOG.md
  - `0.46.x` supports ESP-IDF `v5.5+`; ESP-IDF 6 compatibility is basic/unreleased.
- `esp-idf-svc` changelog: https://raw.githubusercontent.com/esp-rs/esp-idf-svc/master/CHANGELOG.md
  - `0.52.x` adds ESP-IDF `v5.4.x` and `v5.5.x` compatibility; ESP-IDF 6 compatibility is in unreleased notes.
- `espup` README and releases: https://github.com/esp-rs/espup and https://github.com/esp-rs/espup/releases
  - Latest observed release: `v0.17.1`.
- `espflash` README: https://github.com/esp-rs/espflash and raw package READMEs under `espflash/` and `cargo-espflash/`.
  - Latest observed crates.io team listing: `cargo-espflash v4.4.0`; README documents ESP32-S2/S3 support and commands including `list-ports`, `flash`, `monitor`, and `save-image`.
- Bazel release model: https://bazel.build/release
  - Bazel 9 active LTS, latest listed `9.1.1`, support through Dec 2028.
- Bazelisk docs: https://bazel.build/install/bazelisk
  - Official recommended way to install/manage Bazel versions.
- `rules_rust` docs and BCR: https://bazelbuild.github.io/rules_rust/ and https://registry.bazel.build/modules/rules_rust
  - Current BCR release: `0.70.0`; tested on Bazel 7.x, 8.x, and 9.x.
- `just` manual and releases: https://just.systems/man/en/ and https://github.com/casey/just/releases
  - Current observed release: `1.53.0`; use only ordinary recipe syntax unless the project pins this version.
- Rust latest release: https://blog.rust-lang.org/releases/latest/
  - Current stable observed: `1.96.0`. Firmware builds still need the `esp` toolchain for Xtensa.

### Important Source Interpretation

ESP-IDF `v6.0.1` is not rejected because it is unstable. It is rejected as the first baseline because current released Rust ESP-IDF crates lag the latest IDF release, and their own docs warn that the `esp-idf-*` crates are community-maintained, can lag ESP-IDF stable, and currently lack HIL tests.

ESP-IDF Build System v2 is not selected because official docs mark it as a technical preview and not recommended for production. Use the established ESP-IDF CMake build path that `esp-idf-sys` already integrates with.

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
| Gamma 601 SoC details need confirmation from reference tree | Medium | Once `reference/esp-miner` exists, verify board config and IDF target before implementation. | MEDIUM |
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
| Gamma 601 target triple | MEDIUM | Project docs point to Gamma 601 BM1370; the local `reference/esp-miner` submodule is not present yet, so confirm ESP32-S3 target when adding it. |

# Phase 1: Foundation And Gamma 601 Boot/Log - Research

**Researched:** 2026-06-21
**Domain:** Rust ESP-IDF firmware foundation, Bazel/Cargo/Just workflow, ESP32-S3 image packaging, USB flash/monitor, parity evidence
**Confidence:** MEDIUM-HIGH

<user_constraints>
## User Constraints (from CONTEXT.md)

Source: `.planning/phases/01-foundation-and-gamma-601-boot-log/01-CONTEXT.md` copied as the governing phase context. [VERIFIED: local file `.planning/phases/01-foundation-and-gamma-601-boot-log/01-CONTEXT.md`]

### Locked Decisions

## Implementation Decisions

### Reference Guardrails

- **D-01:** Add `reference/esp-miner` as the pinned upstream ESP-Miner reference and treat it as read-only project evidence, not an editable workspace.
- **D-02:** Implement `just verify-reference` and the Bazel-visible reference guard so missing, dirty, or unpinned reference state fails clearly before parity evidence or fixtures are trusted.
- **D-03:** Record the pinned reference commit in package/parity outputs where relevant so later evidence can be tied to an exact comparison baseline.

### Automation And Workspace Skeleton

- **D-04:** Use Bazel/Bzlmod as the canonical automation graph, with `just` as a thin human command surface that delegates to Bazel targets or repo-owned scripts represented in Bazel.
- **D-05:** Create the planned monorepo shape up front: `firmware/bitaxe`, pure crates under `crates/`, host tooling under `tools/`, and scripts under `scripts/`.
- **D-06:** Keep Cargo metadata authoritative for Rust dependencies while Bazel mirrors or wraps the Rust work; do not attempt a full custom Bazel-native ESP-IDF toolchain in Phase 1.
- **D-07:** Pin the first firmware baseline to the accepted ESP-IDF Rust stack: ESP-IDF `v5.5.4`, `xtensa-esp32s3-espidf`, Rust 2021 for firmware/shared crates, and the ESP Rust toolchain installed through `espup`.

### Safe Gamma 601 Boot/Log

- **D-08:** The first firmware image should boot/log only. It must make the safe state explicit: mining disabled, ASIC work submission disabled, and hardware control disabled or safe-no-op.
- **D-09:** Boot logs should include firmware identity, firmware/source commit when available, ESP-IDF/Rust/toolchain identity when available, reset reason, partition/image identity, platform/PSRAM status, selected board target `Gamma 601`, selected ASIC target `BM1370`, and reference commit when known.
- **D-10:** If real Gamma 601 hardware is unavailable during local verification, the plan should still build/package and leave hardware smoke as a required evidence item rather than silently marking hardware behavior verified.

### Package, Flash, And Monitor Ergonomics

- **D-11:** Implement `just build`, `just test`, `just package`, `just flash`, `just monitor`, `just flash-monitor`, `just verify-reference`, and `just parity` in Phase 1 even if some targets initially wrap scripts or provide narrow skeleton behavior.
- **D-12:** `just flash board=601` should build/package first by default, accept `port=...`, discover likely ports when omitted, fail clearly on no or ambiguous ports, and print the underlying `espflash` command before executing.
- **D-13:** `just package` should produce a machine-readable package manifest with image paths, offsets when applicable, checksums, tool versions, firmware commit, and reference commit.

### Parity And Provenance Evidence

- **D-14:** Treat `docs/parity/checklist.md` as an evidence ledger. Phase 1 should wire `just parity` so checklist rows, statuses, missing evidence, implementation pointers, and breadcrumbs are inspectable without equating implementation with verification.
- **D-15:** Do not mark safety-critical or hardware-control surfaces `verified` in Phase 1 unless actual Gamma 601 hardware evidence exists. Boot/log smoke can be `hardware-smoke`; ASIC init, voltage, fan, thermal, power, and mining remain later-phase evidence.
- **D-16:** Preserve the MIT-first posture for original scaffolding and independently authored Rust code, but keep GPL provenance explicit for upstream-derived behavior and fixture sources.

### the agent's Discretion

The agent may choose the exact Bazel target implementation shape, host helper crate boundaries, placeholder/skeleton module contents, package manifest schema details, and formatting of parity reports when those choices remain consistent with the accepted docs, Bright Builds rules, and Phase 1 success criteria.

### Deferred Ideas (OUT OF SCOPE)

## Deferred Ideas

- BM1370 ASIC initialization, work sending, result parsing, frequency transitions, voltage changes, fan, thermal, power, and mining behavior belong to later hardware phases.
- Stratum v1 mining, fake pool coverage, and accepted-share evidence belong to Phase 4.
- AxeOS HTTP/WebSocket API compatibility and static asset administration belong to Phase 5 and Phase 7.
- OTA, recovery, release packaging, dependency license inventory, and release compliance gates belong to Phase 7.
- Non-601 boards and additional ASIC families remain unverified or deferred until each has its own evidence.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| FND-01 | The repo includes upstream ESP-Miner as a pinned git submodule at `reference/esp-miner`. | Pin `bitaxeorg/ESP-Miner` at `c1915b0a63bfabebdb95a515cedfee05146c1d50`, which is current `master` HEAD from `git ls-remote` during this research. [VERIFIED: command `git ls-remote https://github.com/bitaxeorg/ESP-Miner.git HEAD refs/heads/master`] |
| FND-02 | Normal project workflows fail when `reference/esp-miner` is missing, unpinned, or locally modified. | Add `scripts/verify-reference-clean.sh`, expose it through Bazel and `just verify-reference`, and run it before parity/package trust decisions. [VERIFIED: `docs/project/seed-layout.md`; VERIFIED: `.planning/phases/01-foundation-and-gamma-601-boot-log/01-CONTEXT.md`] |
| FND-03 | Bazel/Bzlmod is the canonical automation graph for build, test, package, flash-shaped, parity, and release-shaped workflows. | Use `.bazelversion` with Bazel 9.1.1, `MODULE.bazel`, `rules_rust 0.70.0`, and Bazel targets for all Just commands. [CITED: https://bazel.build/release; CITED: https://registry.bazel.build/modules/rules_rust] |
| FND-04 | The Rust workspace pins the ESP-IDF Rust toolchain, ESP-IDF version, Rust target, firmware metadata, and dependency versions needed for Gamma 601 firmware builds. | Pin ESP-IDF `v5.5.4`, `xtensa-esp32s3-espidf`, Rust 2021, `esp-idf-svc 0.52.1`, `esp-idf-hal 0.46.2`, and `esp-idf-sys 0.37.2`. [VERIFIED: crates.io API; CITED: https://docs.rs/crate/esp-idf-svc/latest; CITED: https://docs.rs/crate/esp-idf-sys/latest] |
| FND-05 | The monorepo contains the planned pure Rust crates for core state, config, ASIC, Stratum, API, and test support. | Create the seed layout exactly under `crates/bitaxe-core`, `bitaxe-config`, `bitaxe-asic`, `bitaxe-stratum`, `bitaxe-api`, and `bitaxe-test-support`. [VERIFIED: `docs/project/seed-layout.md`] |
| FND-06 | The ESP-IDF Rust firmware app can boot on Gamma 601 and log identity/status while mining and hardware control remain disabled. | Use `firmware/bitaxe` with ESP-IDF Rust logging and explicit safe-state log lines; live `verified` status requires Gamma 601 `hardware-smoke`. [VERIFIED: `.planning/REQUIREMENTS.md`; CITED: https://docs.rs/crate/esp-idf-svc/latest] |
| FND-07 | `just build`, `just test`, `just package`, `just flash`, `just monitor`, `just flash-monitor`, `just verify-reference`, and `just parity` are available and route through Bazel or repo-owned scripts represented in the automation graph. | Use `Justfile` recipes as thin wrappers around Bazel targets such as `//firmware/bitaxe:firmware_image`, `//tools/flash:flash`, `//tools/parity:report`, and `//scripts:verify_reference_clean`. [VERIFIED: `docs/project/seed-layout.md`; CITED: https://just.systems/man/en/] |
| FND-08 | USB flashing ergonomics support `board=601`, optional `port=...`, likely-port discovery, clear ambiguous-port errors, build-before-flash by default, and printing the underlying flashing command. | Implement `tools/flash` as a Rust CLI using `clap` and `espflash list-ports`, `flash`, `monitor`, and `save-image`. [CITED: https://github.com/esp-rs/espflash/blob/main/espflash/README.md; VERIFIED: local command `espflash --help`] |
| FND-09 | Firmware packaging records image paths, offsets when applicable, checksums, tool versions, firmware commit, and reference commit in a machine-readable manifest. | Generate a JSON manifest from the Bazel package target and include upstream-inspired offsets and SHA-256 checksums. [VERIFIED: upstream `merge_bin.sh` and `partitions.csv` at commit `c1915b0a63bfabebdb95a515cedfee05146c1d50`; CITED: https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-guides/partition-tables.html] |
| FND-10 | Provenance and license guardrails keep original project work MIT-first where possible while marking upstream-derived GPL-compatible expression explicitly. | Keep reference breadcrumbs and SPDX posture aligned with `PROVENANCE.md`; do not copy upstream expression into MIT-only files. [VERIFIED: local file `PROVENANCE.md`] |
| FND-11 | Parity tooling reports checklist status, evidence gaps, implementation pointers, and reference breadcrumbs without treating implementation alone as verification. | Make `tools/parity` parse `docs/parity/checklist.md` statuses/evidence and reject invalid `verified` claims for safety-critical rows. [VERIFIED: local file `docs/parity/checklist.md`; VERIFIED: `.planning/research/PITFALLS.md`] |
</phase_requirements>

## Summary

Phase 1 should be planned as a foundation and evidence phase, not as a mining or hardware-control phase. [VERIFIED: `.planning/ROADMAP.md`; VERIFIED: `docs/project/first-milestone.md`] The public command surface must exist in Phase 1 even where early targets are wrappers or skeletons, because later phases depend on stable `just` and Bazel entrypoints. [VERIFIED: `.planning/phases/01-foundation-and-gamma-601-boot-log/01-CONTEXT.md`; VERIFIED: `docs/project/seed-layout.md`]

The technical baseline remains ESP-IDF Rust `std` on ESP-IDF `v5.5.4` for `xtensa-esp32s3-espidf`, with Bazel 9/Bzlmod as the visible graph and Cargo as Rust dependency authority. [VERIFIED: `.planning/research/STACK.md`; VERIFIED: crates.io API; CITED: https://docs.rs/crate/esp-idf-sys/latest] ESP-IDF `v6.0.1` is the current stable ESP-IDF release, but released esp-rs crates make ESP-IDF `v5.5.x` the safer first baseline because ESP-IDF 6 support is either unreleased/basic in current changelogs or introduces component/API movement. [CITED: https://docs.espressif.com/projects/esp-idf/en/stable/esp32/versions.html; CITED: https://raw.githubusercontent.com/esp-rs/esp-idf-svc/master/CHANGELOG.md; CITED: https://raw.githubusercontent.com/esp-rs/esp-idf-sys/master/CHANGELOG.md]

The current checkout has no `reference/` directory, no submodules, no Rust workspace, no Bazel files, no Justfile, and no known USB serial port visible to `espflash`. [VERIFIED: commands `rg --files`, `git submodule status --recursive`, `ls reference`, `espflash list-ports`] The plan must therefore include Wave 0 foundation tasks for the submodule, workspace, tool graph, and test infrastructure, and it must leave live Gamma 601 boot/flash evidence open until a board and port are available. [VERIFIED: `.planning/STATE.md`; VERIFIED: command `espflash list-ports`]

**Primary recommendation:** Implement the phase around five gates: reference guard, canonical graph, workspace skeleton, safe boot/log image, and evidence/reporting command surface. [VERIFIED: `.planning/ROADMAP.md`; VERIFIED: `docs/parity/checklist.md`]

## Project Constraints (from AGENTS.md)

| Directive | Planning Impact | Source |
|-----------|-----------------|--------|
| Read repo-local `AGENTS.md`, `AGENTS.bright-builds.md`, `standards-overrides.md`, and relevant standards before planning or implementation work. | The plan must mention Bright Builds standards as inputs and should not bypass local rules. | [VERIFIED: local files `AGENTS.md`, `AGENTS.bright-builds.md`, `standards/index.md`] |
| Use GSD workflow entrypoints before direct repo edits unless explicitly bypassed. | The research artifact is being produced under GSD phase planning, and implementation should proceed through `/gsd-execute-phase` or planned phase work. | [VERIFIED: local file `AGENTS.md`] |
| Prefer functional core plus imperative shell. | Pure crates own deterministic behavior; `firmware/bitaxe` owns ESP-IDF, hardware, storage, and task effects. | [VERIFIED: `AGENTS.md`; VERIFIED: `standards/core/architecture.md`; VERIFIED: `.planning/research/ARCHITECTURE.md`] |
| Parse boundary data into domain types and make illegal states unrepresentable where practical. | Even Phase 1 skeletons should create typed board/ASIC/safe-state surfaces instead of raw string/integer constants scattered through firmware. | [VERIFIED: `standards/core/architecture.md`; VERIFIED: `standards/languages/rust.md`] |
| Prefer early returns, Rust `let...else`, `foo.rs` plus `foo/`, and `maybe_` names for `Option`-bearing internal values. | New Rust code and tests should follow these conventions from the first crate skeletons. | [VERIFIED: `standards/core/code-shape.md`; VERIFIED: `standards/languages/rust.md`] |
| Unit tests must test pure/business logic, one concern per test, with Arrange/Act/Assert comments unless trivially obvious. | Host tools and pure crate skeletons should ship focused tests where they implement logic. | [VERIFIED: `standards/core/testing.md`; VERIFIED: `standards/languages/rust.md`] |
| Checked-in scripts should use `#!/usr/bin/env bash`, `set -euo pipefail`, rerunnable behavior, useful logs, and no swallowed errors. | `scripts/verify-reference-clean.sh` and wrapper scripts must fail visibly and avoid hidden `|| true`-style behavior. | [VERIFIED: `AGENTS.md`; VERIFIED: `standards/core/code-shape.md`] |
| Before a Rust-project commit, run format, clippy, build, and tests when a `Cargo.toml` exists. | After Phase 1 creates `Cargo.toml`, implementation commits must run the Rust checks through the repo-owned command surface or equivalent. | [VERIFIED: `AGENTS.md`; VERIFIED: `standards/core/verification.md`] |
| Do not edit managed Bright Builds blocks directly. | Phase 1 should add new firmware/project files, not rewrite managed standards sidecars. | [VERIFIED: `AGENTS.md`; VERIFIED: `AGENTS.bright-builds.md`] |

## Standard Stack

### Core

| Library / Tool | Version / Pin | Purpose | Why Standard | Source |
|----------------|---------------|---------|--------------|--------|
| Upstream ESP-Miner reference | `c1915b0a63bfabebdb95a515cedfee05146c1d50` | Read-only behavior/provenance reference at `reference/esp-miner` | Current upstream `master` HEAD during research and already used by project research as the accepted baseline | [VERIFIED: `git ls-remote`; VERIFIED: GitHub commit API 2026-06-21] |
| ESP-IDF | `v5.5.4` | C SDK and runtime underneath Rust firmware | Released esp-rs crates explicitly support ESP-IDF 5.4/5.5; ESP-IDF 6 support is newer and riskier for first boot | [VERIFIED: `.planning/research/STACK.md`; CITED: https://raw.githubusercontent.com/esp-rs/esp-idf-sys/master/CHANGELOG.md] |
| `esp-idf-svc` | `0.52.1`, updated 2026-03-10 | ESP-IDF service wrappers and logging integration | It wraps services Phase 1 and later phases need and re-exports `hal`/`sys` layers | [VERIFIED: crates.io API; CITED: https://docs.rs/crate/esp-idf-svc/latest] |
| `esp-idf-hal` | `0.46.2`, updated 2026-03-10 | ESP-IDF driver/HAL layer used via `esp-idf-svc` unless direct access is necessary | It is the matching HAL version for the accepted released esp-rs stack | [VERIFIED: crates.io API; CITED: https://raw.githubusercontent.com/esp-rs/esp-idf-hal/master/CHANGELOG.md] |
| `esp-idf-sys` | `0.37.2`, updated 2026-03-10 | Raw ESP-IDF bindings and Cargo-driven ESP-IDF build integration | It supports ESP-IDF 5.4/5.5 and documents Cargo-driven build/download behavior | [VERIFIED: crates.io API; CITED: https://docs.rs/crate/esp-idf-sys/latest] |
| Rust target | `xtensa-esp32s3-espidf`, `MCU=esp32s3` | Gamma 601 firmware target | Upstream packaging uses `--chip esp32s3`, and esp-rs target table maps `esp32s3` to `xtensa-esp32s3-espidf` | [VERIFIED: upstream `merge_bin.sh`; CITED: https://docs.rs/crate/esp-idf-sys/latest] |
| Rust edition | `2021` | Firmware and shared crate edition | The accepted project stack pins Rust 2021 until firmware build/flash is stable | [VERIFIED: `.planning/phases/01-foundation-and-gamma-601-boot-log/01-CONTEXT.md`; VERIFIED: `.planning/research/STACK.md`] |
| Bazel / Bazelisk | `.bazelversion` = `9.1.1` | Canonical automation graph | Bazel 9 is active LTS through Dec 2028, and Bazelisk uses `.bazelversion` for reproducible versions | [CITED: https://bazel.build/release; CITED: https://bazel.build/install/bazelisk; VERIFIED: local command `bazel --version`] |
| `rules_rust` | `0.70.0` | Bazel rules for pure Rust crates and host tools | BCR shows `rules_rust 0.70.0` and Bazel 9 compatibility | [CITED: https://registry.bazel.build/modules/rules_rust] |
| `crate_universe` | from `rules_rust 0.70.0` | Mirror Cargo dependencies into Bazel | It is the current rules_rust path for Cargo/Bazel dependency integration and repinning | [CITED: https://bazelbuild.github.io/rules_rust/crate_universe_bzlmod.html] |
| `just` | Use syntax compatible with local `1.48.0`; latest registry `1.53.0` | Human command surface | Just is a command runner for project-specific recipes, and project decisions require it as a thin wrapper | [VERIFIED: local command `just --version`; VERIFIED: crates.io API; CITED: https://just.systems/man/en/] |
| `espup` | Target current `0.17.1`; local `0.15.1` installed | Install and maintain ESP Rust toolchains | espup is the esp-rs tool for installing required toolchains and supports `--targets esp32s3 --std` | [VERIFIED: crates.io API; VERIFIED: local command `espup install --help`; CITED: https://github.com/esp-rs/espup] |
| `espflash` / `cargo-espflash` | Target current `4.4.0`; local `4.0.1` installed | Port listing, image generation, flash, monitor | Installed versions expose required Phase 1 commands; latest registry version is 4.4.0 | [VERIFIED: crates.io API; VERIFIED: local commands `espflash --help`, `cargo espflash --help`; CITED: https://github.com/esp-rs/espflash/blob/main/espflash/README.md] |

### Supporting

| Library | Version | Purpose | When to Use | Source |
|---------|---------|---------|-------------|--------|
| `clap` | `4.6.1`, updated 2026-04-15 | Host CLI parsing for `tools/flash` and `tools/parity` | Use for typed `board`, `port`, command, manifest, and evidence flags | [VERIFIED: crates.io API] |
| `serde` | `1.0.228`, updated 2025-09-27 | Serialization derives | Use for package manifest, parity report output, and structured fixtures | [VERIFIED: crates.io API] |
| `serde_json` | `1.0.150`, updated 2026-05-21 | JSON package/parity manifests | Use for machine-readable package manifest and report outputs | [VERIFIED: crates.io API] |
| `toml` | `1.1.2+spec-1.1.0`, updated 2026-04-01 | TOML parsing/writing | Use for config files such as `espflash.toml` if tooling owns them | [VERIFIED: crates.io API] |
| `camino` | `1.2.3`, updated 2026-06-18 | UTF-8 path handling | Use in host tools where JSON manifests should contain portable string paths | [VERIFIED: crates.io API] |
| `ignore` | `0.4.26`, updated 2026-06-05 | Git-aware file walking | Use for parity/provenance scans that must respect ignored paths | [VERIFIED: crates.io API] |
| `walkdir` | `2.5.0`, updated 2024-03-01 | Directory traversal | Use when git-ignore semantics are unnecessary | [VERIFIED: crates.io API] |
| `anyhow` | `1.0.102`, updated 2026-02-20 | Application/CLI error context | Use in host binaries and scripts wrapped by Rust CLIs | [VERIFIED: crates.io API; VERIFIED: `AGENTS.md` Rust guidance] |
| `thiserror` | `2.0.18`, updated 2026-01-18 | Library error enums | Use in pure crates where errors are part of the domain API | [VERIFIED: crates.io API; VERIFIED: `AGENTS.md` Rust guidance] |
| `sha2` | `0.11.0`, updated 2026-03-25 | SHA-256 package manifest checksums | Use for image/checksum manifest generation instead of custom hashing | [VERIFIED: crates.io API] |
| `tempfile` | `3.27.0`, updated 2026-03-11 | Host tool tests | Use for CLI and manifest tests with temporary workspaces | [VERIFIED: crates.io API] |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| ESP-IDF Rust `std` | Bare-metal `esp-hal` / `no_std` | Rejected for Phase 1 because accepted project constraints need ESP-IDF services and image conventions. [VERIFIED: `.planning/PROJECT.md`; CITED: https://docs.rs/crate/esp-idf-sys/latest] |
| ESP-IDF `v5.5.4` | ESP-IDF `v6.0.1` | ESP-IDF 6.0.1 is current stable, but released esp-rs crate support is safer on 5.5.x for first boot. [CITED: https://docs.espressif.com/projects/esp-idf/en/stable/esp32/versions.html; CITED: https://raw.githubusercontent.com/esp-rs/esp-idf-svc/master/CHANGELOG.md] |
| Bazel wrapper around Cargo/ESP-IDF | Fully custom Bazel-native ESP-IDF toolchain | Rejected for Phase 1 because `esp-idf-sys` is Cargo-driven and the project decision explicitly avoids a full custom toolchain in this phase. [VERIFIED: `.planning/phases/01-foundation-and-gamma-601-boot-log/01-CONTEXT.md`; CITED: https://docs.rs/crate/esp-idf-sys/latest] |
| `espflash` backend | Direct `esptool.py` as the normal UX | Rejected for normal UX because `espflash` already exposes list, flash, monitor, save-image, and partition commands. [CITED: https://github.com/esp-rs/espflash/blob/main/espflash/README.md] |
| `just` command surface | Make/Bash-only public commands | Rejected because accepted project decisions require Just as the human surface and Bazel as the graph. [VERIFIED: `docs/project/project-decisions.md`; CITED: https://just.systems/man/en/] |

**Installation / setup commands for the plan to encode:**

```bash
cargo install espup --locked
espup install --targets esp32s3 --std
. "$HOME/export-esp.sh"
cargo install espflash cargo-espflash --locked
```

Source: espup and espflash official READMEs document these install paths and flags. [CITED: https://github.com/esp-rs/espup; CITED: https://github.com/esp-rs/espflash/blob/main/espflash/README.md]

**Version verification:** The crate versions above were verified from the crates.io API on 2026-06-21. [VERIFIED: command `curl https://crates.io/api/v1/crates/<name>`] Bazel and local tool availability were verified from local commands on this workstation. [VERIFIED: commands `bazel --version`, `just --version`, `cargo --version`, `espflash --version`]

## Architecture Patterns

### Recommended Project Structure

```text
/
├── MODULE.bazel                 # Bzlmod graph and rules_rust/crate_universe setup
├── .bazelversion                # Bazel 9.1.1 pin
├── BUILD.bazel                  # Root target glue when needed
├── Cargo.toml                   # Workspace dependency authority
├── Cargo.lock                   # Cargo dependency lock mirrored by Bazel
├── rust-toolchain.toml          # Host Rust channel and components
├── Justfile                     # Thin human command surface
├── reference/esp-miner/         # Pinned, read-only upstream submodule
├── firmware/bitaxe/             # ESP-IDF Rust firmware app and adapters
├── crates/bitaxe-core/          # Pure shared state and lifecycle domain
├── crates/bitaxe-config/        # Pure board/config/defaults model
├── crates/bitaxe-asic/          # Pure ASIC model stubs and later packet logic
├── crates/bitaxe-stratum/       # Pure Stratum stubs for later phases
├── crates/bitaxe-api/           # Pure API model stubs for later phases
├── crates/bitaxe-test-support/  # Test fixtures and harness helpers
├── tools/flash/                 # Rust CLI for port/package/flash/monitor UX
├── tools/parity/                # Rust CLI for checklist/evidence report
└── scripts/verify-reference-clean.sh
```

Source: This structure is the accepted seed monorepo shape. [VERIFIED: `docs/project/seed-layout.md`; VERIFIED: `.planning/research/ARCHITECTURE.md`]

### Pattern 1: Bazel-visible wrappers around Cargo/ESP-IDF

**What:** Bazel targets should own the public graph, but firmware targets may call repo-owned scripts or `xtask` code that invokes Cargo/ESP-IDF until a deeper Bazel-native ESP-IDF toolchain is proven. [VERIFIED: `.planning/phases/01-foundation-and-gamma-601-boot-log/01-CONTEXT.md`; VERIFIED: `.planning/research/STACK.md`]

**When to use:** Use this pattern for `//firmware/bitaxe:firmware`, `//firmware/bitaxe:firmware_image`, and any target that must go through `esp-idf-sys` build scripts. [VERIFIED: `.planning/research/STACK.md`; CITED: https://docs.rs/crate/esp-idf-sys/latest]

**Example:**

```starlark
# Source: BCR rules_rust module syntax and project seed layout.
# [CITED: https://registry.bazel.build/modules/rules_rust]
# [VERIFIED: docs/project/seed-layout.md]
module(name = "bitaxe_esp_miner", version = "0.1.0")

bazel_dep(name = "rules_rust", version = "0.70.0")
```

### Pattern 2: Cargo workspace remains Rust dependency authority

**What:** Use root `Cargo.toml` and `Cargo.lock` as Rust dependency truth, then mirror into Bazel with `crate_universe`. [VERIFIED: `.planning/research/STACK.md`; CITED: https://bazelbuild.github.io/rules_rust/crate_universe_bzlmod.html]

**When to use:** Use this pattern for pure crates and host tools; avoid making Phase 1 maintain two independent Rust dependency graphs. [VERIFIED: `.planning/phases/01-foundation-and-gamma-601-boot-log/01-CONTEXT.md`]

**Example:**

```toml
# Source: accepted Cargo-authoritative boundary.
# [VERIFIED: .planning/phases/01-foundation-and-gamma-601-boot-log/01-CONTEXT.md]
[workspace]
resolver = "2"
members = [
  "crates/bitaxe-core",
  "crates/bitaxe-config",
  "crates/bitaxe-asic",
  "crates/bitaxe-stratum",
  "crates/bitaxe-api",
  "crates/bitaxe-test-support",
  "tools/flash",
  "tools/parity",
  "firmware/bitaxe",
]

[workspace.package]
edition = "2021"
license = "MIT"
```

### Pattern 3: Safe boot/log firmware before hardware effects

**What:** The first firmware should initialize logging, gather identity/platform facts, and log safe state while leaving mining, ASIC work, voltage/fan/thermal control, and ASIC initialization disabled. [VERIFIED: `.planning/phases/01-foundation-and-gamma-601-boot-log/01-CONTEXT.md`; VERIFIED: `docs/project/first-milestone.md`]

**When to use:** Use this pattern for `firmware/bitaxe/src/main.rs` and any boot coordinator code in Phase 1. [VERIFIED: `.planning/ROADMAP.md`]

**Example:**

```rust
// Source: esp-idf-svc examples use link_patches and EspLogger initialization.
// [CITED: https://docs.rs/crate/esp-idf-svc/latest/source/examples/wps_async.rs]
fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    log::info!("bitaxe-rust boot: board=Gamma 601 asic=BM1370");
    log::info!("safe_state: mining=disabled asic_work=disabled hardware_control=disabled");

    Ok(())
}
```

### Pattern 4: Evidence-first command outputs

**What:** Package, flash, and parity commands should emit machine-readable files containing firmware commit, reference commit, tool versions, paths, checksums, and evidence status. [VERIFIED: `.planning/phases/01-foundation-and-gamma-601-boot-log/01-CONTEXT.md`; VERIFIED: `docs/parity/checklist.md`]

**When to use:** Use this pattern in `//firmware/bitaxe:firmware_image`, `tools/flash`, and `tools/parity`. [VERIFIED: `docs/project/seed-layout.md`]

**Example manifest shape:**

```json
{
  "schema_version": 1,
  "board": "601",
  "device_model": "Gamma",
  "asic": "BM1370",
  "firmware_commit": "Unavailable",
  "reference_commit": "c1915b0a63bfabebdb95a515cedfee05146c1d50",
  "esp_idf_version": "v5.5.4",
  "rust_target": "xtensa-esp32s3-espidf",
  "artifacts": [
    {
      "kind": "factory_image",
      "path": "bazel-bin/firmware/bitaxe/bitaxe-gamma601-factory.bin",
      "offset": "0x0",
      "sha256": "Unavailable"
    }
  ]
}
```

Source: Required fields come from Phase 1 decisions and upstream image offsets. [VERIFIED: `.planning/phases/01-foundation-and-gamma-601-boot-log/01-CONTEXT.md`; VERIFIED: upstream `merge_bin.sh`; VERIFIED: upstream `partitions.csv`]

### Anti-Patterns to Avoid

- **Treating `just` as a second build graph:** `just` must delegate to Bazel targets or repo-owned scripts represented in Bazel. [VERIFIED: `.planning/phases/01-foundation-and-gamma-601-boot-log/01-CONTEXT.md`; VERIFIED: `docs/project/seed-layout.md`]
- **Allowing firmware to mine or control hardware in Phase 1:** Mining and hardware-control surfaces are explicitly deferred. [VERIFIED: `.planning/phases/01-foundation-and-gamma-601-boot-log/01-CONTEXT.md`]
- **Using only filesystem checks for reference cleanliness:** Submodule state must be checked with Git, because a directory can exist while being dirty or unpinned. [VERIFIED: `.planning/research/PITFALLS.md`; VERIFIED: `docs/project/seed-layout.md`]
- **Calling a successful build `verified` parity:** Build success proves workflow health only, not device-user behavior. [VERIFIED: `docs/parity/checklist.md`; VERIFIED: `.planning/research/PITFALLS.md`]
- **Copying upstream C expression into MIT-only Rust files:** Upstream-derived expression must be isolated or GPL-compatible. [VERIFIED: `PROVENANCE.md`]

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| ESP-IDF services and startup integration | Custom bare-metal platform layer | `esp-idf-svc` / `esp-idf-sys` | The project requires ESP-IDF services and image conventions, and `esp-idf-svc` wraps Wi-Fi, HTTP, WebSocket, NVS, OTA, logging, timers, and event loop families. [VERIFIED: `.planning/PROJECT.md`; CITED: https://docs.rs/crate/esp-idf-svc/latest] |
| ESP32-S3 flashing and monitor commands | Direct serial protocol implementation | `espflash` / `cargo-espflash` | espflash already exposes `list-ports`, `flash`, `monitor`, `save-image`, `partition-table`, and `write-bin`. [CITED: https://github.com/esp-rs/espflash/blob/main/espflash/README.md; VERIFIED: local command `espflash --help`] |
| Bazel/Cargo dependency mirroring | Duplicate manual BUILD deps for all crates | `rules_rust` `crate_universe` | `crate_universe` is the current rules_rust dependency integration path and supports repinning. [CITED: https://bazelbuild.github.io/rules_rust/crate_universe_bzlmod.html] |
| CLI parsing for `tools/flash` and `tools/parity` | Manual `std::env::args` parsing | `clap` | `clap` provides typed subcommands and validation for board/port/image/report options. [VERIFIED: crates.io API] |
| JSON package manifests | Hand-built strings | `serde` + `serde_json` | Structured serialization prevents invalid JSON and keeps the manifest testable. [VERIFIED: crates.io API; VERIFIED: `standards/core/architecture.md`] |
| Image checksums | Custom hash code | `sha2` or a vetted platform checksum tool | Package manifest checksums are a security/evidence surface and should not use custom cryptography. [VERIFIED: crates.io API; VERIFIED: `PROVENANCE.md`] |
| Reference cleanliness | Ad hoc directory existence checks only | `git submodule status`, `git -C reference/esp-miner status --porcelain`, and parent-repo diff checks | The reference can be present but dirty, uninitialized, or at an unrecorded pointer. [VERIFIED: `.planning/research/PITFALLS.md`; VERIFIED: `docs/project/seed-layout.md`] |

**Key insight:** Phase 1 risk is not algorithmic complexity; it is evidence drift between reference, build graph, artifact manifest, flash command, and parity ledger. [VERIFIED: `.planning/research/PITFALLS.md`; VERIFIED: `docs/parity/checklist.md`]

## Common Pitfalls

### Pitfall 1: Reference Exists But Is Not Trustworthy

**What goes wrong:** The upstream tree is missing, dirty, uninitialized, or at an undocumented commit. [VERIFIED: `.planning/research/PITFALLS.md`; VERIFIED: command `git submodule status --recursive`]
**Why it happens:** A directory-level check does not prove submodule state. [VERIFIED: `.planning/research/PITFALLS.md`]
**How to avoid:** Add the submodule, record the commit in package/parity outputs, and fail `just verify-reference` before package/parity trust decisions. [VERIFIED: `.planning/phases/01-foundation-and-gamma-601-boot-log/01-CONTEXT.md`]
**Warning signs:** `reference/` is absent, `git submodule status` is empty, or normal commands can modify files under `reference/esp-miner`. [VERIFIED: commands `ls reference`, `git submodule status --recursive`; VERIFIED: `.planning/research/PITFALLS.md`]

### Pitfall 2: Safe Boot Accidentally Becomes Hardware Bring-Up

**What goes wrong:** Early firmware starts ASIC initialization, voltage/fan/thermal control, or work submission before evidence gates exist. [VERIFIED: `.planning/phases/01-foundation-and-gamma-601-boot-log/01-CONTEXT.md`; VERIFIED: `.planning/research/PITFALLS.md`]
**Why it happens:** Upstream `main.c` initializes hardware and starts mining-related tasks after configuration and self-test flows, but Phase 1 scope is only boot/log. [VERIFIED: upstream `main/main.c`; VERIFIED: `docs/project/first-milestone.md`]
**How to avoid:** Log `mining=disabled`, `asic_work=disabled`, and `hardware_control=disabled`; hold any reset/power behavior to safe-no-op unless the plan explicitly treats it as platform safety. [VERIFIED: `.planning/phases/01-foundation-and-gamma-601-boot-log/01-CONTEXT.md`]
**Warning signs:** Phase 1 code imports BM1370 init, fan/PID, TPS546, thermal, Stratum, or mining-loop modules as active runtime paths. [VERIFIED: `docs/parity/checklist.md`; VERIFIED: `.planning/REQUIREMENTS.md`]

### Pitfall 3: Bazel, Cargo, and Just Diverge

**What goes wrong:** `cargo build` works while `just build` or Bazel targets fail, or flash commands bypass packaged artifacts. [VERIFIED: `.planning/research/PITFALLS.md`]
**Why it happens:** ESP-IDF Rust is Cargo-driven while the accepted repo contract makes Bazel canonical. [CITED: https://docs.rs/crate/esp-idf-sys/latest; VERIFIED: `.planning/phases/01-foundation-and-gamma-601-boot-log/01-CONTEXT.md`]
**How to avoid:** Make Cargo authoritative for Rust metadata, make Bazel authoritative for workflow targets, and make Just a thin wrapper. [VERIFIED: `.planning/research/STACK.md`; VERIFIED: `docs/project/seed-layout.md`]
**Warning signs:** Recipes call `cargo` directly without a Bazel-visible target, or Bazel targets do not declare package outputs. [VERIFIED: `docs/project/seed-layout.md`; VERIFIED: `.planning/research/PITFALLS.md`]

### Pitfall 4: Package Manifest Omits Offsets Or Provenance

**What goes wrong:** Images exist but cannot be audited against exact source/reference commits, offsets, versions, and checksums. [VERIFIED: `.planning/phases/01-foundation-and-gamma-601-boot-log/01-CONTEXT.md`; VERIFIED: `.planning/research/PITFALLS.md`]
**Why it happens:** A build artifact path alone does not encode ESP32 flash layout or provenance. [VERIFIED: upstream `merge_bin.sh`; CITED: https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-guides/partition-tables.html]
**How to avoid:** Generate a JSON manifest with image paths, offsets, SHA-256, tool versions, firmware commit, ESP-IDF version, Rust target, and reference commit. [VERIFIED: `.planning/phases/01-foundation-and-gamma-601-boot-log/01-CONTEXT.md`]
**Warning signs:** `just package` prints only an ELF path or produces no machine-readable manifest. [VERIFIED: `.planning/REQUIREMENTS.md`]

### Pitfall 5: Hardware Smoke Is Claimed Without Hardware

**What goes wrong:** Boot/log behavior is marked verified even though no Gamma 601 was flashed and monitored. [VERIFIED: `.planning/phases/01-foundation-and-gamma-601-boot-log/01-CONTEXT.md`; VERIFIED: `docs/parity/checklist.md`]
**Why it happens:** Build/package success can be mistaken for device behavior. [VERIFIED: `.planning/research/PITFALLS.md`]
**How to avoid:** If no board/port is present, keep build/package/parity checks green but record hardware smoke as required evidence. [VERIFIED: `.planning/phases/01-foundation-and-gamma-601-boot-log/01-CONTEXT.md`; VERIFIED: command `espflash list-ports`]
**Warning signs:** `espflash list-ports` reports no known serial ports, but the checklist row moves to `verified`. [VERIFIED: command `espflash list-ports`; VERIFIED: `docs/parity/checklist.md`]

## Code Examples

Verified patterns from local decisions and official sources:

### Firmware Cargo Metadata

```toml
# Source: esp-idf-sys build metadata pattern and accepted Phase 1 pin.
# [CITED: https://docs.rs/crate/esp-idf-sys/latest]
# [VERIFIED: .planning/phases/01-foundation-and-gamma-601-boot-log/01-CONTEXT.md]
[package.metadata.esp-idf-sys]
esp_idf_version = "tag:v5.5.4"
esp_idf_tools_install_dir = "workspace"
esp_idf_sdkconfig = "sdkconfig"
esp_idf_sdkconfig_defaults = ["sdkconfig.defaults"]
```

### Firmware Target Config

```toml
# Source: esp-rs target table lists esp32s3 -> xtensa-esp32s3-espidf.
# [CITED: https://docs.rs/crate/esp-idf-sys/latest]
[build]
target = "xtensa-esp32s3-espidf"

[target.xtensa-esp32s3-espidf]
linker = "ldproxy"

[unstable]
build-std = ["std", "panic_abort"]

[env]
MCU = "esp32s3"
```

### Reference Guard Script Shape

```bash
#!/usr/bin/env bash
set -euo pipefail

reference_dir="reference/esp-miner"

if [[ ! -d "$reference_dir/.git" ]]; then
  printf 'reference missing or not initialized: %s\n' "$reference_dir" >&2
  exit 1
fi

pinned_commit="$(git -C "$reference_dir" rev-parse HEAD)"
dirty_state="$(git -C "$reference_dir" status --porcelain)"

if [[ -n "$dirty_state" ]]; then
  printf 'reference dirty at %s\n%s\n' "$pinned_commit" "$dirty_state" >&2
  exit 1
fi

printf 'reference clean: %s\n' "$pinned_commit"
```

Source: Required behavior comes from seed layout and Phase 1 decisions; Bash safety rules come from AGENTS.md. [VERIFIED: `docs/project/seed-layout.md`; VERIFIED: `.planning/phases/01-foundation-and-gamma-601-boot-log/01-CONTEXT.md`; VERIFIED: `AGENTS.md`]

### Just Command Surface Shape

```make
# Source: Just is command surface; Bazel is automation graph.
# [VERIFIED: docs/project/seed-layout.md]
build:
    bazel build //firmware/bitaxe:firmware

test:
    bazel test //...

package:
    bazel build //firmware/bitaxe:firmware_image

verify-reference:
    bazel run //scripts:verify_reference_clean

parity:
    bazel run //tools/parity:report

flash board="601" port="":
    bazel run //tools/flash -- flash --board {{board}} {{if port != "" { "--port " + port } else { "" }}}
```

Source: Recipe names are required by Phase 1; keep syntax compatible with local Just 1.48.0 during implementation. [VERIFIED: `.planning/REQUIREMENTS.md`; VERIFIED: local command `just --version`; CITED: https://just.systems/man/en/]

### Flash Tool Command Contract

```text
tools/flash flash --board 601 [--port /dev/cu.usbmodem...]
  -> build/package first unless --image is provided
  -> run espflash list-ports when --port is omitted
  -> fail on zero or ambiguous likely ports with exact --port examples
  -> print the espflash command before execution
  -> record command, board, port, image manifest, firmware commit, reference commit, and log path when evidence capture is enabled
```

Source: Behavior is locked by Phase 1 decisions; available espflash commands are verified locally and in the espflash README. [VERIFIED: `.planning/phases/01-foundation-and-gamma-601-boot-log/01-CONTEXT.md`; VERIFIED: local command `espflash --help`; CITED: https://github.com/esp-rs/espflash/blob/main/espflash/README.md]

## State of the Art

| Old Approach | Current Approach | When Changed / Current State | Impact |
|--------------|------------------|------------------------------|--------|
| ESP-IDF 4.x Rust baseline | ESP-IDF 5.5.x Rust baseline for this project | `esp-idf-sys 0.37.0` deprecated ESP-IDF `<5.3.0` and added 5.4/5.5 compatibility. [CITED: https://raw.githubusercontent.com/esp-rs/esp-idf-sys/master/CHANGELOG.md] | Pin `v5.5.4`; do not start on old IDF defaults. [VERIFIED: `.planning/research/STACK.md`] |
| Chasing latest ESP-IDF stable automatically | Use latest ESP-IDF only after crate support and project smoke evidence | ESP-IDF `v6.0.1` is current stable, while esp-rs 6.0 support is newer/basic/unreleased in current changelogs. [CITED: https://docs.espressif.com/projects/esp-idf/en/stable/esp32/versions.html; CITED: https://raw.githubusercontent.com/esp-rs/esp-idf-hal/master/CHANGELOG.md] | Plan a later reassessment instead of using ESP-IDF 6 for Phase 1. [VERIFIED: `.planning/research/STACK.md`] |
| WORKSPACE-first Bazel setup | Bzlmod with `MODULE.bazel` | BCR publishes `rules_rust 0.70.0` with `bazel_dep` install snippet and Bazel 9 compatibility. [CITED: https://registry.bazel.build/modules/rules_rust] | Use `MODULE.bazel` from the start. [VERIFIED: `.planning/research/STACK.md`] |
| Direct `esptool.py` UX | `espflash` / `cargo-espflash` UX | espflash supports ESP32-S3 plus `list-ports`, `flash`, `monitor`, `save-image`, and partition tooling. [CITED: https://github.com/esp-rs/espflash/blob/main/espflash/README.md] | Use espflash behind `tools/flash`; keep esptool only as upstream comparison context. [VERIFIED: `.planning/research/STACK.md`] |
| Implementation status as parity status | Evidence-led checklist status | Local parity policy defines statuses and evidence types, and safety-critical surfaces require hardware evidence before `verified`. [VERIFIED: `docs/parity/checklist.md`] | Phase 1 can mark workflow items verified only when command evidence exists; hardware rows need hardware smoke. [VERIFIED: `.planning/ROADMAP.md`] |

**Deprecated/outdated:**
- `cargo-espmonitor` as the normal monitor tool is outdated for this project because espflash/cargo-espflash provide monitor commands. [VERIFIED: `.planning/research/STACK.md`; CITED: https://github.com/esp-rs/espflash/blob/main/espflash/README.md]
- ESP-IDF Build System v2 should not be used as the Phase 1 production baseline because project research already rejected it as preview/high-risk for this scope. [VERIFIED: `.planning/research/STACK.md`]
- Snapshot/golden evidence alone cannot verify hardware-control or boot-on-device claims. [VERIFIED: `docs/parity/checklist.md`; VERIFIED: `.planning/research/PITFALLS.md`]

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| - | No unverified assumptions were used as planning requirements; claims are tagged as local verification, registry verification, official docs citation, or user constraints. | All | None from assumptions; open execution constraints remain below. [VERIFIED: source review in this research] |

## Open Questions (RESOLVED)

1. **Is Gamma 601 hardware available for the Phase 1 hardware-smoke gate?**
   - Resolution: Local execution has no visible Gamma 601 serial port; `espflash list-ports` currently reports no known serial ports. [VERIFIED: local command `espflash list-ports`]
   - Planning disposition: Phase 1 plans must build, package, run parity tooling, and prepare the evidence record through automated gates, but hardware smoke remains a manual/evidence checkpoint. Plans must not claim the Gamma 601 flash-monitor smoke passed unless `just flash-monitor board=601 port=...` captures the required boot/log lines on connected hardware. [VERIFIED: `.planning/phases/01-foundation-and-gamma-601-boot-log/01-CONTEXT.md` D-10 and D-15]
   - Status: RESOLVED - no hardware smoke is assumed in planning; missing hardware remains explicit evidence status.
2. **Should implementation upgrade local esp tools before first build?**
   - Resolution: Support the installed tools first when they satisfy required commands. Local `espup 0.15.1` and `espflash 4.0.1` are behind crates.io latest versions, but installed `espflash` exposes the required `list-ports`, `flash`, `monitor`, and `save-image` command surfaces, and installed `espup` exposes the required install flow. [VERIFIED: local commands `espup install --help`, `espflash --help`, `espflash save-image --help`; VERIFIED: crates.io API]
   - Planning disposition: Plans may print local versions and fail with install/upgrade guidance when required commands are missing or too old, but they must not require an upgrade without proof that the installed command surface is insufficient. [VERIFIED: `.planning/phases/01-foundation-and-gamma-601-boot-log/01-CONTEXT.md` D-07 and D-12]
   - Status: RESOLVED - no unconditional espup/espflash upgrade is required by Phase 1 plans.
3. **Should the first package image be a complete merged factory image or a staged manifest over Cargo/ESP-IDF outputs?**
   - Resolution: Phase 1 must produce a declared default flash input, not only metadata. The package target should produce declared outputs for the firmware ELF and a saved/merged image when `espflash save-image` succeeds, and the manifest must include a top-level `default_flash_image` field pointing to the default flash input produced by `//firmware/bitaxe:firmware_image`. [VERIFIED: local command `espflash save-image --help`; VERIFIED: `.planning/phases/01-foundation-and-gamma-601-boot-log/01-CONTEXT.md` D-12 and D-13]
   - Planning disposition: `tools/flash` must read the package manifest and use `default_flash_image` when `--image` or `image=` is omitted. The manifest may still mark non-critical unavailable fields as `Unavailable`, but `default_flash_image` itself must point to an existing declared package artifact before default flashing proceeds. [VERIFIED: `.planning/REQUIREMENTS.md` FND-08 and FND-09]
   - Status: RESOLVED - package and flash plans must wire `//firmware/bitaxe:firmware_image` -> manifest `default_flash_image` -> `tools/flash` default image consumption.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|-------------|-----------|---------|----------|
| Git | Submodule, provenance, commits | yes | 2.50.1 Apple Git | None needed. [VERIFIED: local command `git --version`] |
| `reference/esp-miner` submodule | FND-01, FND-02, FND-03, FND-10, FND-11 | no | - | Blocking setup task: add/init submodule. [VERIFIED: local commands `ls reference`, `git submodule status --recursive`] |
| Node | GSD tooling | yes | 24.13.0 | None needed. [VERIFIED: local command `node --version`] |
| Python | ESP-IDF tooling support | yes | 3.9.6 | None needed for Phase 1 planning. [VERIFIED: local command `python3 --version`; CITED: https://github.com/esp-rs/esp-idf-template/blob/master/README.md] |
| Rust/Cargo | Host tools and firmware workspace | yes | cargo 1.91.1, rustc 1.91.1 | Rust 1.96.0 is latest stable, but local 1.91.1 exceeds esp-rs/espflash MSRV needs. [VERIFIED: local commands; CITED: https://blog.rust-lang.org/2026/05/28/Rust-1.96.0/; CITED: https://github.com/esp-rs/espflash/blob/main/espflash/README.md] |
| Rust `esp` toolchain | Xtensa ESP32-S3 firmware | yes | `esp` listed by rustup | Source `$HOME/export-esp.sh` before firmware builds. [VERIFIED: local commands `rustup toolchain list`, `ls $HOME/export-esp.sh`; CITED: https://github.com/esp-rs/espup] |
| Bazel | Canonical graph | yes | 9.1.1 | None needed. [VERIFIED: local command `bazel --version`] |
| Bazelisk | Bazel version management | yes | resolves Bazel 9.1.1 | Check in `.bazelversion`. [VERIFIED: local command `bazelisk --version`; CITED: https://bazel.build/install/bazelisk] |
| Just | Human command surface | yes | local 1.48.0; latest 1.53.0 | Use ordinary recipe syntax compatible with local 1.48.0. [VERIFIED: local command `just --version`; VERIFIED: crates.io API] |
| `espup` | ESP Rust setup | yes | local 0.15.1; latest 0.17.1 | Existing install may work; setup check should suggest upgrade. [VERIFIED: local command `espup --version`; VERIFIED: crates.io API] |
| `espflash` | USB flash/monitor | yes | local 4.0.1; latest 4.4.0 | Required commands exist locally; setup check should suggest upgrade. [VERIFIED: local command `espflash --help`; VERIFIED: crates.io API] |
| `cargo-espflash` | Cargo-integrated flash diagnostics | yes | local 4.0.1; latest 4.4.0 | Prefer plain `espflash` behind Bazel package outputs. [VERIFIED: local command `cargo espflash --help`; VERIFIED: crates.io API] |
| `ldproxy` | ESP-IDF Rust linker | yes | binary present; version command enters runner mode | Use via target config; exact version can be surfaced by setup check if needed. [VERIFIED: local command `command -v ldproxy`] |
| Gamma 601 serial port | Hardware smoke | no | `espflash list-ports` reports none | No substitute for verified hardware smoke; leave evidence required. [VERIFIED: local command `espflash list-ports`; VERIFIED: `docs/parity/checklist.md`] |

**Missing dependencies with no fallback:**
- `reference/esp-miner` is missing and blocks trusted parity/reference workflows until added. [VERIFIED: local commands `ls reference`, `git submodule status --recursive`; VERIFIED: `.planning/STATE.md`]
- A visible Gamma 601 serial port is missing and blocks live hardware-smoke verification. [VERIFIED: local command `espflash list-ports`]

**Missing dependencies with fallback:**
- Local espup/espflash versions are behind latest registry versions, but their installed command surfaces cover required Phase 1 commands. [VERIFIED: local commands `espup install --help`, `espflash --help`, `cargo espflash --help`; VERIFIED: crates.io API]

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | None exists yet because the repo currently has no Cargo workspace, Bazel workspace, or tests. [VERIFIED: command `rg --files`] |
| Config file | none - Wave 0 must add `Cargo.toml`, `MODULE.bazel`, `BUILD.bazel` files, `Justfile`, crate test targets, and tool tests. [VERIFIED: command `rg --files`; VERIFIED: `docs/project/seed-layout.md`] |
| Quick run command | After Wave 0: `just verify-reference && just build && just test && just package && just parity`. [VERIFIED: `.planning/ROADMAP.md`; VERIFIED: `docs/project/first-milestone.md`] |
| Full suite command | After hardware is available: quick command plus `just flash-monitor board=601 port=<port>` and captured evidence review. [VERIFIED: `.planning/ROADMAP.md`; VERIFIED: `.planning/phases/01-foundation-and-gamma-601-boot-log/01-CONTEXT.md`] |

### Phase Requirements -> Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|--------------|
| FND-01 | Submodule present at pinned ESP-Miner commit | workflow | `git submodule status --recursive && just verify-reference` | No - Wave 0. [VERIFIED: command `git submodule status --recursive`] |
| FND-02 | Missing/dirty/unpinned reference fails clearly | unit + workflow | `bazel test //scripts:verify_reference_clean_test && just verify-reference` | No - Wave 0. [VERIFIED: `docs/project/seed-layout.md`] |
| FND-03 | Bazel/Bzlmod owns build/test/package/flash/parity graph | workflow | `bazel query //... && just build && just test` | No - Wave 0. [VERIFIED: command `rg --files`] |
| FND-04 | Firmware metadata pins ESP-IDF, target, toolchain, dependency versions | unit + build | `cargo metadata --format-version=1 && bazel build //firmware/bitaxe:firmware` | No - Wave 0. [VERIFIED: command `rg --files`] |
| FND-05 | Planned crate layout exists and builds/tests | build + unit | `bazel test //crates/...` | No - Wave 0. [VERIFIED: `docs/project/seed-layout.md`] |
| FND-06 | Gamma 601 firmware boots and logs identity/status with safe state | hardware-smoke | `just flash-monitor board=601 port=<port>` | No - Wave 0 and hardware required. [VERIFIED: local command `espflash list-ports`] |
| FND-07 | Required Just commands exist and route through Bazel/script targets | workflow | `just --list && just build && just test && just package && just parity` | No - Wave 0. [VERIFIED: command `rg --files`] |
| FND-08 | Flash UX supports board, optional port, discovery, ambiguity errors, build-before-flash, printed command | unit + integration | `bazel test //tools/flash:tests` plus `just flash board=601 --dry-run` if implemented | No - Wave 0. [VERIFIED: `.planning/REQUIREMENTS.md`] |
| FND-09 | Package manifest contains paths, offsets, checksums, tool versions, firmware commit, reference commit | unit + workflow | `bazel test //tools/package:tests && just package` or package target equivalent | No - Wave 0. [VERIFIED: `.planning/REQUIREMENTS.md`] |
| FND-10 | Provenance/license guardrails are enforced for original vs upstream-derived content | workflow | `bazel test //tools/parity:tests && just parity` with provenance checks | No - Wave 0. [VERIFIED: `PROVENANCE.md`] |
| FND-11 | Parity report exposes statuses, evidence gaps, implementation pointers, breadcrumbs | unit + workflow | `bazel test //tools/parity:tests && just parity` | No - Wave 0. [VERIFIED: `docs/parity/checklist.md`] |

### Sampling Rate

- **Per task commit:** Run the narrow Bazel/Cargo test for changed crate/tool plus any touched script check. [VERIFIED: `standards/core/verification.md`]
- **Per wave merge:** Run `just verify-reference`, `just build`, `just test`, `just package`, and `just parity`. [VERIFIED: `.planning/ROADMAP.md`]
- **Phase gate:** Run all wave commands plus Gamma 601 `just flash-monitor board=601 port=<port>` when hardware is available; otherwise record hardware-smoke as missing evidence. [VERIFIED: `.planning/phases/01-foundation-and-gamma-601-boot-log/01-CONTEXT.md`]

### Wave 0 Gaps

- [ ] `reference/esp-miner` submodule - covers FND-01 and FND-02. [VERIFIED: local command `ls reference`]
- [ ] `MODULE.bazel`, `.bazelversion`, root `BUILD.bazel` - covers FND-03. [VERIFIED: `docs/project/seed-layout.md`]
- [ ] Root `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `.cargo/config.toml` - covers FND-04 and FND-05. [VERIFIED: `docs/project/seed-layout.md`]
- [ ] `firmware/bitaxe` app with `sdkconfig.defaults` and boot/log tests or build target - covers FND-06. [VERIFIED: `docs/project/first-milestone.md`]
- [ ] `Justfile` - covers FND-07. [VERIFIED: `docs/project/seed-layout.md`]
- [ ] `tools/flash` Rust CLI and tests - covers FND-08. [VERIFIED: `.planning/REQUIREMENTS.md`]
- [ ] package manifest generation target/tests - covers FND-09. [VERIFIED: `.planning/REQUIREMENTS.md`]
- [ ] `tools/parity` Rust CLI and tests - covers FND-10 and FND-11. [VERIFIED: `docs/parity/checklist.md`]

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|------------------|
| V2 Authentication | no | Phase 1 has no user authentication surface. [VERIFIED: `.planning/ROADMAP.md`] |
| V3 Session Management | no | Phase 1 has no session-management surface. [VERIFIED: `.planning/ROADMAP.md`] |
| V4 Access Control | limited | Treat `reference/esp-miner` as read-only evidence and fail on dirty state; no runtime authorization is in scope. [VERIFIED: `PROVENANCE.md`; VERIFIED: `docs/project/seed-layout.md`] |
| V5 Input Validation | yes | Validate CLI args with `clap`, parse paths with `camino`, and reject unknown board IDs instead of passing raw strings to shell commands. [VERIFIED: crates.io API; VERIFIED: `standards/core/architecture.md`] |
| V6 Cryptography | yes, narrow | Use vetted SHA-256 tooling or `sha2`; do not implement checksums or cryptography manually. [VERIFIED: crates.io API; VERIFIED: `.planning/REQUIREMENTS.md`] |

### Known Threat Patterns for Phase 1

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Shell command injection through `port` or `image` | Tampering / Elevation of Privilege | Use Rust `Command` with argument vectors and typed `clap` parsing; do not concatenate shell strings. [VERIFIED: `standards/core/code-shape.md`; VERIFIED: crates.io API] |
| Dirty or substituted reference tree | Tampering / Repudiation | Use Git submodule checks and include reference commit in package/parity outputs. [VERIFIED: `docs/project/seed-layout.md`; VERIFIED: `.planning/phases/01-foundation-and-gamma-601-boot-log/01-CONTEXT.md`] |
| Misleading package provenance | Repudiation | Emit firmware commit, reference commit, tool versions, and SHA-256 in the package manifest. [VERIFIED: `.planning/REQUIREMENTS.md`] |
| Secrets accidentally logged during boot | Information Disclosure | Phase 1 should not configure Wi-Fi or print credentials; logs should be identity/status/safe-state only. [VERIFIED: `.planning/phases/01-foundation-and-gamma-601-boot-log/01-CONTEXT.md`; VERIFIED: upstream `config-601.cvs` contains Wi-Fi/pool credential fields] |
| False safety claim from non-hardware checks | Spoofing / Safety governance failure | Keep safety-critical rows below `verified` unless evidence type is `hardware-smoke` or `hardware-regression`. [VERIFIED: `docs/parity/checklist.md`] |

## Sources

### Primary (HIGH confidence)

- `.planning/phases/01-foundation-and-gamma-601-boot-log/01-CONTEXT.md` - locked Phase 1 decisions, scope, discretion, deferred items. [VERIFIED: local file read]
- `.planning/REQUIREMENTS.md` - FND-01 through FND-11 and project-wide requirements. [VERIFIED: local file read]
- `.planning/ROADMAP.md` - Phase 1 goal, success criteria, and verification expectations. [VERIFIED: local file read]
- `.planning/STATE.md` - current blocker that `reference/esp-miner` is absent. [VERIFIED: local file read]
- `.planning/research/STACK.md`, `.planning/research/ARCHITECTURE.md`, `.planning/research/FEATURES.md`, `.planning/research/PITFALLS.md` - existing project research. [VERIFIED: local files read]
- `docs/project/first-milestone.md`, `docs/project/seed-layout.md`, `docs/project/project-decisions.md` - accepted milestone and monorepo contract. [VERIFIED: local files read]
- `docs/parity/checklist.md` - parity status/evidence definitions and Phase 1 surfaces. [VERIFIED: local file read]
- `PROVENANCE.md` - MIT-first and GPL guardrail policy. [VERIFIED: local file read]
- `AGENTS.md`, `AGENTS.bright-builds.md`, `standards-overrides.md`, `standards/core/*.md`, `standards/languages/rust.md` - local workflow and coding/testing constraints. [VERIFIED: local files read]
- Upstream ESP-Miner commit `c1915b0a63bfabebdb95a515cedfee05146c1d50` - current upstream `master` HEAD during research. [VERIFIED: `git ls-remote`; VERIFIED: GitHub commit API]
- Upstream files at that commit: `config-601.cvs`, `main/device_config.h`, `main/main.c`, `main/system.c`, `merge_bin.sh`, `partitions.csv`, `flashing.md`. [VERIFIED: GitHub raw file fetch]

### Official / Vendor (HIGH confidence)

- ESP-IDF version policy and current stable docs: https://docs.espressif.com/projects/esp-idf/en/stable/esp32/versions.html. [CITED]
- ESP-IDF partition table docs: https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-guides/partition-tables.html. [CITED]
- ESP-IDF system APIs for reset reason, heap, MAC, app/version surfaces: https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/misc_system_api.html. [CITED]
- `esp-idf-svc` docs and changelog: https://docs.rs/crate/esp-idf-svc/latest and https://raw.githubusercontent.com/esp-rs/esp-idf-svc/master/CHANGELOG.md. [CITED]
- `esp-idf-hal` changelog: https://raw.githubusercontent.com/esp-rs/esp-idf-hal/master/CHANGELOG.md. [CITED]
- `esp-idf-sys` docs and changelog: https://docs.rs/crate/esp-idf-sys/latest and https://raw.githubusercontent.com/esp-rs/esp-idf-sys/master/CHANGELOG.md. [CITED]
- `espup` README: https://github.com/esp-rs/espup. [CITED]
- `espflash` README: https://github.com/esp-rs/espflash/blob/main/espflash/README.md. [CITED]
- Bazel release model and Bazelisk docs: https://bazel.build/release and https://bazel.build/install/bazelisk. [CITED]
- `rules_rust` BCR and crate_universe docs: https://registry.bazel.build/modules/rules_rust and https://bazelbuild.github.io/rules_rust/crate_universe_bzlmod.html. [CITED]
- Just manual: https://just.systems/man/en/. [CITED]
- Rust latest release: https://blog.rust-lang.org/2026/05/28/Rust-1.96.0/. [CITED]

### Registry / Local Command Verification (HIGH confidence)

- crates.io API for `esp-idf-svc`, `esp-idf-hal`, `esp-idf-sys`, `espup`, `espflash`, `cargo-espflash`, `ldproxy`, `clap`, `serde`, `serde_json`, `toml`, `camino`, `ignore`, `walkdir`, `anyhow`, `thiserror`, `sha2`, `tempfile`, and `just`. [VERIFIED: command `curl https://crates.io/api/v1/crates/<name>`]
- Local tool checks: `git --version`, `node --version`, `python3 --version`, `cargo --version`, `rustc --version`, `rustup toolchain list`, `bazel --version`, `bazelisk --version`, `just --version`, `espup --version`, `espflash --version`, `cargo espflash --version`, `espflash list-ports`. [VERIFIED: local commands]

### Secondary (MEDIUM confidence)

- None needed for locked Phase 1 decisions. [VERIFIED: source hierarchy review]

### Tertiary (LOW confidence)

- None used as authoritative evidence. [VERIFIED: source hierarchy review]

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - locked locally and verified against crates.io, Bazel/BCR, ESP-IDF, and esp-rs docs. [VERIFIED: sources above]
- Architecture: HIGH - accepted local docs and Bright Builds standards agree on functional core plus imperative shell and seed layout. [VERIFIED: local files above]
- Firmware build/package wrapper: MEDIUM - the wrapper approach is well-supported by `esp-idf-sys` Cargo-driven behavior and local decisions, but this repo has not yet proven a build. [CITED: https://docs.rs/crate/esp-idf-sys/latest; VERIFIED: command `rg --files`]
- Hardware smoke: LOW until a Gamma 601 is connected - no serial port is currently visible. [VERIFIED: command `espflash list-ports`]
- Pitfalls: HIGH for reference, parity, provenance, and graph drift; MEDIUM for exact first firmware image shape until implementation proves outputs. [VERIFIED: `.planning/research/PITFALLS.md`; VERIFIED: upstream `merge_bin.sh`]

**Research date:** 2026-06-21
**Valid until:** 2026-07-21 for locked local decisions; re-check crates.io, BCR, ESP-IDF, esp-rs, and local tool versions before implementation if planning starts after that date. [VERIFIED: current-date context; VERIFIED: registry/doc sources above]

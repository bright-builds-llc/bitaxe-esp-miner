# Seed Monorepo Layout

This is the expected initial shape for the Rust firmware monorepo. GSD may refine names during planning, but changes should preserve the ownership boundaries.

```text
/
  MODULE.bazel
  Justfile
  Cargo.toml
  rust-toolchain.toml
  PROVENANCE.md
  reference/
    esp-miner/
  firmware/
    bitaxe/
  crates/
    bitaxe-core/
    bitaxe-asic/
    bitaxe-stratum/
    bitaxe-config/
    bitaxe-api/
    bitaxe-test-support/
  tools/
    flash/
    parity/
    xtask/
  scripts/
    verify-reference-clean.sh
  docs/
    project/
    parity/
    adr/
```

## Path Ownership

| Path | Owner | Purpose |
| --- | --- | --- |
| `reference/esp-miner` | Upstream ESP-Miner | Read-only reference implementation submodule. |
| `firmware/bitaxe` | This project | ESP-IDF Rust firmware application and hardware adapters. |
| `crates/bitaxe-core` | This project | Pure domain types, job state, shared firmware decisions, and reusable logic. |
| `crates/bitaxe-asic` | This project | ASIC protocol, models, register sequences, nonce parsing, and BM family support. |
| `crates/bitaxe-stratum` | This project | Stratum v1/v2 parsing, messages, job construction, and fixtures. |
| `crates/bitaxe-config` | This project | Board config, NVS settings model, defaults, and validation. |
| `crates/bitaxe-api` | This project | AxeOS API models, OpenAPI compatibility, and response serialization. |
| `crates/bitaxe-test-support` | This project | Fixtures, golden data, harness helpers, and hardware test support. |
| `tools/flash` | This project | USB flashing and serial-port discovery tooling. |
| `tools/parity` | This project | Reference scanning, checklist reports, and golden fixture generation. |
| `tools/xtask` | This project | Optional Rust workflow glue when Bazel or Just benefit from a typed helper. |
| `scripts` | This project | Thin, rerunnable shell scripts invoked by Bazel or Just. |
| `docs/parity` | This project | Audit evidence for device-user parity. |

## Build Ownership

Bazel is the canonical automation graph. `just` commands should call Bazel targets or scripts that Bazel also owns.

Suggested Bazel target families:

```text
//firmware/bitaxe:firmware
//firmware/bitaxe:firmware_image
//crates/bitaxe-core:tests
//crates/bitaxe-asic:tests
//crates/bitaxe-stratum:tests
//crates/bitaxe-config:tests
//crates/bitaxe-api:tests
//tools/flash:flash
//tools/parity:report
//scripts:verify_reference_clean
```

Early Bazel targets may call repo-owned scripts, Cargo commands, or ESP-IDF compatible commands when direct Bazel rules are immature. The important rule is that workflows are represented as Bazel targets so local and CI use the same graph.

## Just Command Surface

```bash
just build
just test
just package
just flash board=205
just flash board=205 port=/dev/cu.usbmodem...
just monitor port=/dev/cu.usbmodem...
just flash-monitor board=205 port=/dev/cu.usbmodem...
just verify-reference
just parity
```

Command behavior:

- `just build` builds the canonical firmware target.
- `just test` runs Bazel test targets for pure crates and tooling.
- `just package` produces flashable images.
- `just flash` builds/packages first unless given an explicit image.
- `just monitor` opens the serial monitor without flashing.
- `just flash-monitor` flashes and then opens the serial monitor.
- `just verify-reference` fails on local modifications inside `reference/esp-miner`.
- `just parity` reports checklist status and missing evidence.

## Reference Guard

Add `scripts/verify-reference-clean.sh` with this behavior:

- Fail if `reference/esp-miner` is missing.
- Fail if the submodule has local modifications.
- Allow an intentional submodule pointer update only when the parent repo records that update.
- Print the pinned upstream commit.

The script should use `#!/usr/bin/env bash` and `set -euo pipefail`.

## Rust Module Shape

Use a functional core with thin hardware adapters:

- Pure crates own parsing, validation, protocol transformations, and deterministic state transitions.
- `firmware/bitaxe` owns ESP-IDF, FreeRTOS, Wi-Fi, NVS, SPIFFS, OTA, serial, GPIO, I2C, ADC, power, display, and task orchestration.
- Hardware-control code can call pure crates, but pure crates must not depend on ESP-IDF.

Follow the local Rust standards:

- Prefer `foo.rs` plus `foo/` over `foo/mod.rs`.
- Use newtypes and enums for invariants.
- Use `maybe_` for `Option`-bearing internal names.
- Unit test pure and business logic.

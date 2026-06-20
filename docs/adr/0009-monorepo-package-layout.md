# Seed the Monorepo with Firmware, Core Crates, Tools, and Reference

The initial monorepo layout should keep the read-only upstream reference under `reference/esp-miner`, the ESP-IDF Rust firmware app under `firmware/bitaxe`, pure Rust crates under `crates/`, workflow tooling under `tools/` and `scripts/`, and project evidence under `docs/`. This separates hardware-bound firmware code from unit-testable protocol, ASIC, config, API, and parity logic while giving Bazel and `just` stable paths for build, test, package, and flash workflows.

# bitaxe-esp-miner

<!-- bright-builds-rules-readme-badges:begin -->

<!-- Managed upstream by bright-builds-rules. If this badge block needs a fix, open an upstream PR or issue instead of editing the downstream managed block. Keep repo-local README content outside this managed badge block. -->

[![GitHub Stars](https://img.shields.io/github/stars/bright-builds-llc/bitaxe-esp-miner)](https://github.com/bright-builds-llc/bitaxe-esp-miner)
[![License](https://img.shields.io/github/license/bright-builds-llc/bitaxe-esp-miner?style=flat-square)](./LICENSE)
[![Rust esp](https://img.shields.io/badge/Rust-esp-000000?logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Bright Builds: Rules](https://raw.githubusercontent.com/bright-builds-llc/bright-builds-rules/main/public/badges/bright-builds-rules-flat.svg)](https://github.com/bright-builds-llc/bright-builds-rules)

<!-- bright-builds-rules-readme-badges:end -->

## Quickstart

This repo treats ESP-IDF as a standard dependency through the pinned Rust
`esp-idf-sys` workflow. You do not need to vendor ESP-IDF or set `IDF_PATH` for
normal development.

1. Install basic host tools: Rust/rustup, Bazelisk or a `bazel` command, Python
   3, and `just`.
2. Run the read-only dependency check:

```bash
just doctor
```

3. If ESP Rust tooling is missing, install it explicitly:

```bash
just bootstrap-esp
source "$HOME/export-esp.sh"
```

Opening a new shell after `just bootstrap-esp` is also fine. Re-run
`just doctor` until it passes, then build and package:

```bash
just build
just package
```

`IDF_PATH` remains an advanced override for contributors who already manage an
external ESP-IDF checkout, but the canonical path is the repo-pinned
`esp-idf-sys` `tag:v5.5.4` setup with `ESP_IDF_TOOLS_INSTALL_DIR=workspace`.

The first firmware build may create `.embuild/` in the repo root. This
directory is generated, gitignored, and owned by the pinned ESP-IDF/esp-rs
workflow. Do not commit files from it, but expect repo scripts to use managed
tools from it when normal `PATH` commands are unavailable, including
`spiffsgen.py`, `gen_esp32part.py`, and `esptool.py`.

# bitaxe-esp-miner

<!-- bright-builds-rules-readme-badges:begin -->

<!-- Managed upstream by bright-builds-rules. If this badge block needs a fix, open an upstream PR or issue instead of editing the downstream managed block. Keep repo-local README content outside this managed badge block. -->

[![GitHub Stars](https://img.shields.io/github/stars/bright-builds-llc/bitaxe-esp-miner)](https://github.com/bright-builds-llc/bitaxe-esp-miner)
[![License](https://img.shields.io/github/license/bright-builds-llc/bitaxe-esp-miner?style=flat-square)](./LICENSE)
[![Node.js 24.18.0](https://img.shields.io/badge/Node.js-24.18.0-339933?logo=node.js&logoColor=white)](./.nvmrc)
[![Rust esp](https://img.shields.io/badge/Rust-esp-000000?logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Bright Builds: Rules](https://raw.githubusercontent.com/bright-builds-llc/bright-builds-rules/main/public/badges/bright-builds-rules-flat.svg)](https://github.com/bright-builds-llc/bright-builds-rules)

<!-- bright-builds-rules-readme-badges:end -->

> [!IMPORTANT]
> **Development here is paused as of 2026-10-30.** We're continuing in a new C fork of
> [ESP-Miner](https://github.com/bitaxeorg/ESP-Miner), not yet created, and will bring our
> extensions and our Bitcoin Work Gate interoperability there.
>
> - **Validation:** every supported board and ASIC needs its own evidence. Upstream supports 23
>   board versions at our pin, and we have evidence on 2.
> - **Keeping up:** each upstream release has to be re-implemented here. A fork gets most of it
>   through a merge, should let us develop faster, and lets us send improvements upstream.
> - **Image size:** our Ultra 205 image leaves 2,640 bytes free in its 4 MiB slot, while upstream's
>   v2.15.4 image uses about 2.7 MB of the same slot. By our estimate most of that gap isn't Rust
>   itself, and the [handoff](docs/project/c-fork-handoff.md#why-the-move) breaks it down.
>
> The code, parity checklist and hardware evidence stay here as a record. Read the
> [handoff document](docs/project/c-fork-handoff.md) for the full reasoning and what moves where,
> and [ADR-0042](docs/adr/0042-pause-rust-firmware-for-a-c-fork.md) for the decision.

<!-- parity-progress:begin -->

## Parity progress

**Parity: 96 of 99 active checklist items verified (97.0%).**

See the [parity checklist](docs/parity/checklist.md) and [progress history](docs/parity/progress.jsonl).

<!-- parity-progress:end -->

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

## Mining qualification

The current firmware is not qualified for production mining.
- The ordinary ESP adapter mines only under a signed Gate Work Lease, within
  device-local protection. Each surface's evidence status is in
  `docs/parity`.
- With no lease, the device mines its saved pool when it is not paused
  (ADR-0038). It probes the pool before power, recovers from overheats,
  keeps a restart guard and a crash-loop guard, and latches hardware
  faults.
- A Gate link takes over: Connect fails until local mining has stopped.
- A device provisioned with `startMiningOnBoot` false, as test and evidence
  provisioning writes it, boots paused.
- Unbounded mining remains a non-claim until the local mining soaks exist.

Deterministic-session outcomes are not hardware evidence or a
release-readiness claim.

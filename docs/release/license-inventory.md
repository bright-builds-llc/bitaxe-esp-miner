# Phase 7 Release License Inventory

This inventory separates generated Cargo license output from the full release
review required by PROVENANCE.md and ADR-0013. `docs/release/cargo-about.html`
is required input, but it does not satisfy release compliance by itself.

## Cargo crates

| Input | Path or pin | License source | Release status |
| --- | --- | --- | --- |
| Generated Cargo report | `docs/release/cargo-about.html` | `cargo-about 0.9.0` using `about.toml` | Required for every release-gate run. |
| Cargo lockfile | `Cargo.lock` | crates.io package metadata mirrored into the generated report | Required source for dependency version evidence. |
| Workspace Rust crates | `crates/*`, `firmware/bitaxe`, `tools/flash`, `tools/parity`, `tools/xtask` | Root workspace MIT posture plus per-crate dependencies in `Cargo.lock` | Original project code is MIT-first unless a file is explicitly marked otherwise. |
| ESP-IDF Rust bindings in Cargo graph | `esp-idf-sys 0.37.2`, `esp-idf-hal 0.46.2`, `esp-idf-svc 0.52.1` | Cargo metadata and `docs/release/cargo-about.html` | Covered as Cargo crates; linked ESP-IDF components are reviewed below. |

- Accepted license identifiers in the current Cargo report: `Apache-2.0`,
  `BSD-3-Clause`, `ISC`, `MIT`, `Unicode-3.0`, and `Zlib`.
- Owner: release tooling.
- Follow-up: regenerate `docs/release/cargo-about.html` whenever `Cargo.lock`,
  `about.toml`, or workspace membership changes.

## Bazel and rules

| Input | Path or pin | License source | Release status |
| --- | --- | --- | --- |
| Bzlmod root | `MODULE.bazel` | Local module declarations | Canonical Bazel dependency source for release review. |
| Bzlmod lockfile | `MODULE.bazel.lock` | Bazel Central Registry module and source records | Required for exact module source evidence. |
| Rust build rules | `rules_rust 0.70.0` | Bazel Central Registry record in `MODULE.bazel.lock` | Build-time rules; not bundled as firmware source. |
| Shell build rules | `rules_shell 0.8.0` | Bazel Central Registry record in `MODULE.bazel.lock` | Build-time rules for script targets. |
| Cargo mirror extension | `crate_universe` from `rules_rust` | `MODULE.bazel` extension declaration and lockfile-generated repos | Mirrors `Cargo.lock`; it does not replace the Cargo report. |

- Owner: release tooling.
- Follow-up: before publication, verify any additional Bzlmod modules introduced
  after this inventory have license/source rows here.

## ESP-IDF and esp-rs

| Input | Path or pin | License source | Release status |
| --- | --- | --- | --- |
| ESP-IDF version | `v5.5.4` through `esp-idf-sys` metadata | ESP-IDF source checkout and component notices | Required for firmware image source-availability review. |
| ESP-IDF Rust sys crate | `esp-idf-sys 0.37.2` | `docs/release/cargo-about.html` plus `esp-idf-sys` metadata | Cargo license covered; native ESP-IDF download reviewed separately. |
| ESP-IDF HAL crate | `esp-idf-hal 0.46.2` | `docs/release/cargo-about.html` | Cargo license covered. |
| ESP-IDF service crate | `esp-idf-svc 0.52.1` | `docs/release/cargo-about.html` | Cargo license covered. |
| ESP-IDF components | Wi-Fi, HTTP server, NVS, SPIFFS, OTA, partition table, FreeRTOS, logging, and C runtime components selected by firmware build | ESP-IDF component notices from the pinned source tree | Release image review must preserve required notices and source references. |

- Owner: firmware release.
- Follow-up: package evidence must record the ESP-IDF checkout/tag and the
  firmware image review decision before public release.

## Flashing tools

| Input | Path or pin | License source | Release status |
| --- | --- | --- | --- |
| `espflash` | Backend for `just flash`, `just monitor`, `just flash-monitor`, and `scripts/package-firmware.sh` image generation | Tool project metadata and installed binary version output | Normal Phase 7 package and operator workflow tool. |
| `cargo-espflash` | Developer diagnostic tool when used outside the canonical Bazel/Just flow | Tool project metadata and installed binary version output | Optional diagnostic input; not required for release packaging. |
| `esptool.py` | Managed ESP-IDF merge backend for `scripts/package-firmware.sh` factory image data-partition assembly | ESP-IDF toolchain source from the pinned `.embuild/espressif` workflow or installed tool metadata | Required when `scripts/package-firmware.sh` assembles `bitaxe-ultra205-factory.bin`. |
| `scripts/package-firmware.sh` | `scripts/package-firmware.sh` | Project MIT-first script plus invoked tool metadata | Prints package inputs and writes manifest evidence. |

- Owner: release tooling.
- Follow-up: release notes must include the exact flashing/package tool versions
  emitted by the package manifest or operator evidence.

## Static assets

| Input | Path or pin | License source | Release status |
| --- | --- | --- | --- |
| Fallback index page | `firmware/bitaxe/static/www/index.html` | Rust-owned source comment in the file | MIT-first original fallback, not copied from upstream ESP-Miner. |
| Fallback stylesheet | `firmware/bitaxe/static/www/assets/app.css` | Rust-owned source comment in the file | MIT-first original fallback stylesheet. |
| Deterministic gzip stylesheet | `firmware/bitaxe/static/www/assets/app.css.gz` | Generated from `firmware/bitaxe/static/www/assets/app.css` with deterministic gzip settings | Same source posture as `app.css`; generated binary input to `www.bin`. |
| Release metadata fixture | `firmware/bitaxe/static/www/assets/release.json` | Rust-owned Phase 7 static source | MIT-first static metadata fixture. |
| Recovery page | `firmware/bitaxe/static/recovery_page.html` | Rust-owned source comment in the file | MIT-first re-authored recovery page, not copied from upstream recovery HTML. |

- No upstream-generated static assets included in Phase 7 package source.
- Owner: firmware release.
- Follow-up: if a later package includes reference-built AxeOS assets, add
  attributed GPL-reviewed rows here before publishing.

## Release artifacts

| Artifact | Source path or generator | License/provenance source | Publication status |
| --- | --- | --- | --- |
| Cargo license report | `docs/release/cargo-about.html` | `about.toml`, `about.hbs`, `Cargo.lock` | Required release-gate input. |
| License inventory | `docs/release/license-inventory.md` | This file | Required release-gate input. |
| Provenance manifest | `docs/release/provenance-manifest.md` | Source/reference/static/recovery review records | Required release-gate input. |
| Operator guide | `docs/release/ultra-205.md` | Phase 7 release documentation | Required operator documentation input. |
| Firmware app image | `esp-miner.bin` from package workflow | Package manifest checksum and source commit recorded in `docs/parity/evidence/phase-16-current-commit-release-evidence-completion.md`; firmware OTA upload remains blocked by missing `DEVICE_URL`. | Publication waits for final release approval. |
| Static filesystem image | `www.bin` from `firmware/bitaxe/static/www` | Static asset rows above plus package manifest checksum recorded in `docs/parity/evidence/phase-16-current-commit-release-evidence-completion.md`; live static and OTAWWW evidence remains blocked or deferred. | Publication waits for final release approval. |
| Factory/recovery image | `bitaxe-ultra205-factory.bin` from package workflow | Package manifest offsets and checksums plus detector-gated serial boot evidence recorded in `docs/parity/evidence/phase-16-current-commit-release-evidence-completion.md`; large erase recovery remains pending. | Publication waits for final release approval. |
| Package manifest | `bitaxe-ultra205-package.json` from package workflow | Package manifest v2 validation and release-gate result recorded in `docs/parity/evidence/phase-16-current-commit-release-evidence-completion.md`. | Publication waits for final release approval. |

- Owner: release gate.
- Follow-up: final release approval still requires artifact checksums, source
  commit, reference commit, tool versions, installation notes, and the GPL
  review status recorded in `docs/release/provenance-manifest.md`; Phase 16
  evidence does not publish or approve GPL-risk-reviewed firmware artifacts.

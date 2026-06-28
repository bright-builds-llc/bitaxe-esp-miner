# Phase 7 Release License Inventory

This inventory separates the Cargo report from the non-Cargo release inputs
required by PROVENANCE.md and ADR-0013. `docs/release/cargo-about.html` is an
input to release review, not a substitute for this full release inventory.

## Cargo crates

- Report: `docs/release/cargo-about.html`
- Tool: `cargo-about 0.9.0`
- Policy: `about.toml`
- Scope: Cargo workspace dependency graph.
- Current accepted license identifiers: `Apache-2.0`, `BSD-3-Clause`, `ISC`,
  `MIT`, `Unicode-3.0`, and `Zlib`.
- Review note: Cargo crates do not cover Bazel, ESP-IDF, flashing tools,
  static assets, upstream reference material, or release artifacts.

## Bazel And Rules

- Scope: `MODULE.bazel`, `MODULE.bazel.lock`, Bzlmod modules, and rules used by
  package, test, firmware, and host-tool targets.
- Required review: record rule/module license notices, source URLs, and version
  pins before a release artifact is published.
- Current state: inventory structure present; detailed Bazel/rules license rows
  are pending release-gate validation.

## ESP-IDF And esp-rs Components

- Scope: ESP-IDF `v5.5.4`, `esp-idf-sys`, `esp-idf-hal`, `esp-idf-svc`, ESP-IDF
  components linked into firmware, and related build metadata.
- Required review: source availability, notices, linked component obligations,
  and whether firmware images include mixed-license inputs.
- Current state: component review is pending the package/release gate.

## espflash And esptool Tooling

- Scope: `espflash`, optional `cargo-espflash`, optional `esptool.py`, and any
  scripts or wrappers used to create merged images, write binaries, monitor
  serial output, or erase flash regions.
- Required review: tool licenses, generated artifact assumptions, and whether
  exact upstream `esptool.py` output is required for a release.
- Current state: normal Phase 7 flows use `espflash`; exact `esptool.py`
  update-only HEX output is not required by the current plan set.

## Firmware And Static Assets

- Scope: Rust firmware, firmware metadata, recovery page source, AxeOS/static
  assets, generated `www.bin`, and any upstream-derived or reference-built
  asset inputs.
- Required review: separate independently authored Rust-owned assets from
  GPL-covered upstream reference expression. Do not call static assets MIT-only
  without source and license evidence.
- Current state: static asset provenance is pending `docs/release/provenance-manifest.md`
  completion by later Phase 7 package/static plans.

## Release Artifacts

- Scope: `esp-miner.bin`, `www.bin`, merged factory/recovery image, update-only
  image if present, package manifest, checksums, install notes, license report,
  and provenance manifest.
- Required review: every release artifact needs a source path, generation
  command, checksum, source commit, reference commit, license posture, and
  publication decision.
- Current state: artifact rows are defined here so Plan 07-06 can enforce the
  release gate against package, provenance, and evidence files.

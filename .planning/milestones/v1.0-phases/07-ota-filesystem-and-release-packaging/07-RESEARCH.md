# Phase 7: OTA, Filesystem, And Release Packaging - Research

**Researched:** 2026-06-28 [VERIFIED: system date]
**Domain:** ESP-IDF Rust OTA, SPIFFS/static serving, partition layout, release packaging, license/provenance gates [VERIFIED: .planning/phases/07-ota-filesystem-and-release-packaging/07-CONTEXT.md]
**Confidence:** MEDIUM-HIGH [VERIFIED: local code/doc inventory + official docs + cargo registry]

<user_constraints>
## User Constraints (from CONTEXT.md)

All bullets in this section are copied from Phase 7 context and are locked planning constraints unless marked as discretion or deferred. [VERIFIED: .planning/phases/07-ota-filesystem-and-release-packaging/07-CONTEXT.md]

### Locked Decisions

#### Release Artifact Packaging And Manifest Contract

- **D-01:** Extend the existing `just package` -> Bazel -> `scripts/package-firmware.sh` -> `tools/xtask` workflow instead of replacing it with a new packaging system. [VERIFIED: .planning/phases/07-ota-filesystem-and-release-packaging/07-CONTEXT.md]
- **D-02:** Introduce a package manifest v2 contract that preserves current `tools/flash` default-image compatibility while adding release-grade metadata for the Ultra 205 app image, factory image, `www.bin`, update-only image when present, checksums, offsets, source commit, reference commit, ESP-IDF/Rust/tool versions, release name, image metadata, and installation notes. [VERIFIED: .planning/phases/07-ota-filesystem-and-release-packaging/07-CONTEXT.md]
- **D-03:** Keep upstream-compatible loose binary assets as first-class manifest entries. Owners should still be able to identify and upload the firmware app image and `www.bin` directly where the upstream AxeOS flow expects those names. [VERIFIED: .planning/phases/07-ota-filesystem-and-release-packaging/07-CONTEXT.md]
- **D-04:** Keep the factory/recovery image as a merged multi-offset artifact with explicit offsets, while app OTA and OTAWWW assets remain individually addressable in the manifest. [VERIFIED: .planning/phases/07-ota-filesystem-and-release-packaging/07-CONTEXT.md]
- **D-05:** Do not make a versioned archive bundle the primary release contract in Phase 7. An archive or SBOM bundle may be added as an optional supplement only after the direct asset and manifest path is proven. [VERIFIED: .planning/phases/07-ota-filesystem-and-release-packaging/07-CONTEXT.md]

#### Filesystem, Static Assets, And Recovery

- **D-06:** Model Phase 7 around the upstream-style `www` SPIFFS partition, generated `www.bin`, static serving from mounted SPIFFS, and embedded `/recovery` page. This is the default V1 parity target. [VERIFIED: .planning/phases/07-ota-filesystem-and-release-packaging/07-CONTEXT.md]
- **D-07:** Preserve the reference partition intent for Ultra 205: app factory and OTA slots, `www` data SPIFFS partition, `otadata`, NVS, PHY, and coredump areas. Any offset or size change must be explicit in the manifest, docs, checklist, and evidence. [VERIFIED: .planning/phases/07-ota-filesystem-and-release-packaging/07-CONTEXT.md]
- **D-08:** Mount filesystem behavior should be visible in logs and status. If SPIFFS is unavailable, the HTTP server should make recovery behavior explicit instead of silently serving broken static paths. [VERIFIED: .planning/phases/07-ota-filesystem-and-release-packaging/07-CONTEXT.md]
- **D-09:** Static serving should preserve upstream-visible behavior where practical: root directory `index.html`, gzipped file preference when a `.gz` variant exists, cache headers for non-directory static assets, fallback redirect to `/` for missing files, and `/recovery` availability. [VERIFIED: .planning/phases/07-ota-filesystem-and-release-packaging/07-CONTEXT.md]
- **D-10:** Do not rewrite Angular AxeOS in this phase. Use the existing AxeOS/static asset compatibility target from Phase 5, generated assets, fixtures, or reference-built assets as planning proves practical. [VERIFIED: .planning/phases/07-ota-filesystem-and-release-packaging/07-CONTEXT.md]

#### Firmware OTA And OTAWWW Update Behavior

- **D-11:** Implement firmware OTA as the primary runtime update capability for Phase 7. Use a pure planner/test surface for accept/reject/status/log decisions and thin firmware adapters for ESP-IDF OTA effects. [VERIFIED: .planning/phases/07-ota-filesystem-and-release-packaging/07-CONTEXT.md]
- **D-12:** Firmware OTA must preserve upstream-visible route behavior for `/api/system/OTA`: private-network/origin gate, AP-mode rejection, binary upload streaming, progress/status updates, protocol/write/validation errors, successful text response, activation of the next app partition, and reboot scheduling. [VERIFIED: .planning/phases/07-ota-filesystem-and-release-packaging/07-CONTEXT.md]
- **D-13:** Prefer ESP-IDF rollback-capable OTA semantics for the Rust firmware. Plans should include boot validation or explicit rollback evidence, not just a successful upload response. [VERIFIED: .planning/phases/07-ota-filesystem-and-release-packaging/07-CONTEXT.md]
- **D-14:** Treat OTAWWW separately from firmware OTA. Direct whole-partition SPIFFS rewrite is the upstream-compatible route, but it has a weak interrupted-update recovery story. [VERIFIED: .planning/phases/07-ota-filesystem-and-release-packaging/07-CONTEXT.md]
- **D-15:** If planning can fit full OTAWWW parity with hardware/interruption evidence, implement `/api/system/OTAWWW` as an upstream-faithful whole-`www` partition update with size checks, chunked erase/write, progress/status, and recovery-page evidence. [VERIFIED: .planning/phases/07-ota-filesystem-and-release-packaging/07-CONTEXT.md]
- **D-16:** If full OTAWWW parity cannot be proven safely in Phase 7, keep OTAWWW fail-closed and record REL-03 as an explicit V1 parity gap with evidence, owner, release impact, and a follow-up path. Do not claim verified static-update parity from package-only evidence. [VERIFIED: .planning/phases/07-ota-filesystem-and-release-packaging/07-CONTEXT.md]
- **D-17:** Do not introduce an A/B static asset partition layout in Phase 7 unless planning proves the partition-layout divergence is necessary and worth the parity explanation burden. [VERIFIED: .planning/phases/07-ota-filesystem-and-release-packaging/07-CONTEXT.md]

#### Release Verification, Licensing, And Operator Documentation

- **D-18:** Use a parity-integrated release gate. Release readiness should be backed by manifest checks, `docs/parity/checklist.md`, phase evidence docs, operator docs, and provenance/license review rather than prose-only release notes. [VERIFIED: .planning/phases/07-ota-filesystem-and-release-packaging/07-CONTEXT.md]
- **D-19:** Generate or update a dependency license inventory covering Rust crates, Bazel/rules dependencies, ESP-IDF Rust bindings, ESP-IDF components, flashing tools, and any web/static assets included in firmware images. [VERIFIED: .planning/phases/07-ota-filesystem-and-release-packaging/07-CONTEXT.md]
- **D-20:** Apply `PROVENANCE.md` and ADR-0013 before publishing firmware artifacts. Any intentionally ported GPL-covered source expression or included upstream-generated assets must be isolated, attributed, and reviewed instead of marked MIT-only by default. [VERIFIED: .planning/phases/07-ota-filesystem-and-release-packaging/07-CONTEXT.md]
- **D-21:** Release docs must cover `just build`, `just package`, `just flash board=205`, `just monitor`, `just flash-monitor`, firmware OTA, OTAWWW or its explicit gap, recovery page, large erase behavior, failed update behavior, interrupted update behavior, and safe rollback/recovery procedures for a developer with a connected Ultra 205. [VERIFIED: .planning/phases/07-ota-filesystem-and-release-packaging/07-CONTEXT.md]
- **D-22:** Parity checklist rows FS-001, OTA-001, OTA-002, REL-001, REL-002, REL-003, and any new release rows must distinguish package/workflow evidence from live firmware OTA, live recovery, and interrupted-update hardware evidence. [VERIFIED: .planning/phases/07-ota-filesystem-and-release-packaging/07-CONTEXT.md]
- **D-23:** Public or release-candidate artifacts may be produced with explicit gaps, but final V1 parity claims must not outrun hardware, rollback, recovery, license, and provenance evidence. [VERIFIED: .planning/phases/07-ota-filesystem-and-release-packaging/07-CONTEXT.md]

### the agent's Discretion

The agent may choose exact Rust module names, manifest schema field names, helper crate boundaries, package target names, fixture formats, evidence document names, and whether release validation lives in `tools/parity`, `tools/xtask`, or a focused helper. Those choices must preserve the repo's functional-core/imperative-shell boundary, keep upstream reference files read-only, keep `just` as the human command surface, keep Bazel as the canonical graph, use typed parsers instead of ad hoc string scanning for manifests where practical, and avoid adding dependencies unless the release/compliance value is concrete. [VERIFIED: .planning/phases/07-ota-filesystem-and-release-packaging/07-CONTEXT.md]

### Deferred Ideas (OUT OF SCOPE)

- A/B static asset partitions may be a future recoverability improvement, but it is not the Phase 7 default because it diverges from the reference partition layout. [VERIFIED: .planning/phases/07-ota-filesystem-and-release-packaging/07-CONTEXT.md]
- A self-contained versioned release archive may be useful later, but direct upstream-compatible assets plus manifest v2 are the Phase 7 contract. [VERIFIED: .planning/phases/07-ota-filesystem-and-release-packaging/07-CONTEXT.md]
- Full Angular AxeOS replacement remains outside V1. [VERIFIED: .planning/phases/07-ota-filesystem-and-release-packaging/07-CONTEXT.md]
- Gamma 601/BM1370, non-205 boards, additional ASIC families, all-board factory images, Stratum v2, and BAP remain outside Phase 7 unless a later roadmap phase adds separate evidence. [VERIFIED: .planning/phases/07-ota-filesystem-and-release-packaging/07-CONTEXT.md]
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| REL-01 | Partition layout, filesystem layout, SPIFFS/static/recovery assets support upstream flows. [VERIFIED: .planning/REQUIREMENTS.md] | Use the upstream `partitions.csv` layout, ESP-IDF partition table rules, SPIFFS image generation, mounted static serving, and recovery fallback patterns documented below. [VERIFIED: reference/esp-miner/partitions.csv; CITED: ESP-IDF v5.5.4 partition/SPIFFS docs] |
| REL-02 | Firmware OTA accept/reject/apply/log/recover upstream-visible. [VERIFIED: .planning/REQUIREMENTS.md] | Use ESP-IDF OTA APIs with rollback-capable boot validation, route access/AP-mode decisions in pure Rust, and firmware streaming adapters matching the reference route contract. [VERIFIED: reference/esp-miner/main/http_server/http_server.c; CITED: ESP-IDF v5.5.4 OTA docs] |
| REL-03 | OTAWWW/static asset update implemented or explicit V1 gap with evidence and owner. [VERIFIED: .planning/REQUIREMENTS.md] | Treat OTAWWW as separate from app OTA; implement whole-`www` partition erase/write only if interruption evidence is scheduled, otherwise retain fail-closed behavior and record an explicit gap. [VERIFIED: 07-CONTEXT.md; reference/esp-miner/main/http_server/http_server.c; CITED: ESP-IDF v5.5.4 SPIFFS docs] |
| REL-04 | Named artifacts with checksums, manifests, metadata, install notes, source commit, and reference commit. [VERIFIED: .planning/REQUIREMENTS.md] | Extend manifest v1 in `tools/xtask` to manifest v2 while preserving `tools/flash` `default_flash_image`; use SHA-256 and explicit offsets for each artifact. [VERIFIED: tools/xtask/src/main.rs; tools/flash/src/main.rs; scripts/package-firmware.sh] |
| REL-05 | License inventory, provenance manifest, and GPL review. [VERIFIED: .planning/REQUIREMENTS.md] | Use `cargo-about` for Rust crates and a project-owned release inventory for Bazel/rules, ESP-IDF, flashing tools, and static assets; tie release readiness to `PROVENANCE.md` and ADR-0013. [VERIFIED: PROVENANCE.md; docs/adr/0013-mit-first-with-gpl-guardrails.md; VERIFIED: cargo info cargo-about] |
| REL-06 | Image production through `just package` and `just flash board=205` without manual artifact discovery. [VERIFIED: .planning/REQUIREMENTS.md] | Extend existing `Justfile` -> Bazel -> `scripts/package-firmware.sh` -> `xtask` package path and keep `tools/flash` manifest v2 compatibility. [VERIFIED: Justfile; firmware/bitaxe/BUILD.bazel; tools/flash/src/main.rs] |
| REL-07 | Docs for connected Ultra 205 operation. [VERIFIED: .planning/REQUIREMENTS.md] | Add operator docs for package, flash, monitor, app OTA, OTAWWW/gap, recovery page, large erase, failed/interrupted update, and rollback evidence. [VERIFIED: 07-CONTEXT.md; reference/esp-miner/readme.md; reference/esp-miner/flashing.md] |
| REL-08 | Rollback, recovery, large erase, failed update, and interrupted update evidence before release parity. [VERIFIED: .planning/REQUIREMENTS.md] | Plan host tests for decisions plus hardware/evidence tasks for live OTA, rollback, recovery, interrupted OTAWWW, and explicit gap handling. [VERIFIED: .planning/REQUIREMENTS.md; docs/adr/0012-parity-verification-evidence.md; CITED: ESP-IDF v5.5.4 OTA docs] |
</phase_requirements>

## Project Constraints (from AGENTS.md)

- Before plan, review, implementation, or audit work, load `AGENTS.md`, `AGENTS.bright-builds.md`, `standards-overrides.md` when present, and relevant pages under `standards/`. [VERIFIED: AGENTS.md]
- Keep `reference/esp-miner` read-only; use it as behavioral evidence and provenance source, not as an editable workspace. [VERIFIED: AGENTS.md; docs/adr/0005-read-only-reference-implementation.md]
- Use GSD workflow artifacts for repo edits; direct repo edits outside GSD are disallowed unless the user explicitly bypasses the workflow. [VERIFIED: AGENTS.md]
- Preserve the project architecture constraint: pure logic belongs in crates, while ESP-IDF, FreeRTOS, Wi-Fi, NVS, SPIFFS, OTA, serial, GPIO, I2C, ADC, power, display, and task orchestration stay in firmware adapters. [VERIFIED: AGENTS.md; standards/core/architecture.md]
- Use `just` as the human command surface and Bazel as the canonical automation graph. [VERIFIED: AGENTS.md; docs/adr/0004-bazel-automation-with-just-wrapper.md]
- Prefer typed parsers and structured APIs over ad hoc string scanning when practical. [VERIFIED: AGENTS.md; standards/core/code-shape.md]
- For Rust code, run `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, and `cargo test --all-features` before creating a code commit. [VERIFIED: AGENTS.md]
- Do not use `unwrap()` in Rust; prefer `?` propagation and use `expect()` only when panic is impossible. [VERIFIED: AGENTS.md; standards/languages/rust.md]
- Tests should verify behavior, use one concern per unit test, and generally follow Arrange/Act/Assert structure. [VERIFIED: AGENTS.md; standards/core/testing.md]
- Release and parity claims require evidence; prose-only claims are insufficient. [VERIFIED: docs/adr/0006-parity-checklist-as-audit-evidence.md; docs/adr/0012-parity-verification-evidence.md]
- No project skill directories were present at `.claude/skills/` or `.agents/skills/` during research, so no repo-local skill patterns apply beyond `AGENTS.md` and standards. [VERIFIED: `find .claude/skills .agents/skills -maxdepth 2 -name SKILL.md`]

## Summary

Phase 7 should extend the current package/flash/release path rather than introduce a parallel release system. [VERIFIED: 07-CONTEXT.md; Justfile; firmware/bitaxe/BUILD.bazel; scripts/package-firmware.sh] The current package path already builds an ELF, creates a merged factory image with `espflash save-image`, and writes a manifest v1 through `tools/xtask`, while `tools/flash` currently reads only `default_flash_image` and expects that default image to remain the Ultra 205 ELF. [VERIFIED: scripts/package-firmware.sh; tools/xtask/src/main.rs; tools/flash/src/main.rs]

The highest-risk planning areas are not JSON generation; they are partition/SPIFFS image correctness, bootloader rollback semantics, static recovery behavior, OTAWWW interrupted-update evidence, and release/license provenance. [VERIFIED: 07-CONTEXT.md; reference/esp-miner/main/http_server/http_server.c; reference/esp-miner/main/filesystem.c; PROVENANCE.md; CITED: ESP-IDF v5.5.4 OTA/SPIFFS docs] ESP-IDF rollback requires first-boot validation after OTA or the bootloader can roll back, and SPIFFS whole-partition rewrites remain weak under interruption. [CITED: ESP-IDF v5.5.4 OTA docs; CITED: ESP-IDF v5.5.4 SPIFFS docs; VERIFIED: 07-CONTEXT.md]

**Primary recommendation:** Plan a manifest v2/package expansion, upstream-compatible partition/SPIFFS/static/recovery implementation, rollback-capable firmware OTA, and parity-integrated release gate as one coherent release pipeline; treat OTAWWW as either a hardware-evidenced implementation or an explicit REL-03 V1 gap. [VERIFIED: 07-CONTEXT.md; .planning/REQUIREMENTS.md]

## Standard Stack

### Core

| Library / Tool | Version | Purpose | Why Standard |
|----------------|---------|---------|--------------|
| ESP-IDF | `v5.5.4` project baseline | OTA, partition table, SPIFFS, VFS, HTTP server, rollback, bootloader behavior. [VERIFIED: AGENTS.md technology stack block; CITED: ESP-IDF v5.5.4 docs] | Project stack pins ESP-IDF Rust production work to ESP-IDF 5.5.x instead of ESP-IDF 6 for first production firmware. [VERIFIED: AGENTS.md technology stack block] |
| `esp-idf-svc` / `esp-idf-sys` | `esp-idf-svc 0.52.1`, `esp-idf-sys 0.37.2` | Rust ESP-IDF service wrappers and raw ESP-IDF FFI for OTA, partition, SPIFFS, and HTTP gaps. [VERIFIED: Cargo.lock; VERIFIED: cargo info esp-idf-svc; VERIFIED: cargo info esp-idf-sys] | Existing firmware stack already uses ESP-IDF Rust, and low-level C APIs are available through `esp-idf-sys` when wrappers are missing. [VERIFIED: firmware/bitaxe/Cargo.toml; Cargo.lock] |
| `espflash` | `4.0.1` local | Flash, monitor, list ports, save merged images, write bins, inspect partition table, erase partitions/regions. [VERIFIED: `espflash --version`; VERIFIED: `espflash --help`; CITED: espflash 4.0.1 README] | Current `scripts/package-firmware.sh` and `tools/flash` use `espflash`; replacing it would violate D-01/D-06. [VERIFIED: scripts/package-firmware.sh; tools/flash/src/main.rs; 07-CONTEXT.md] |
| `esp-idf-part` | `0.6.0` | Parse/generate ESP-IDF partition tables in Rust tooling. [VERIFIED: cargo info esp-idf-part] | Avoid hand-rolled CSV/offset parsing in `xtask` or parity validation. [VERIFIED: cargo info esp-idf-part; VERIFIED: AGENTS.md typed-parser rule] |
| `serde` / `serde_json` | `serde 1.0.228`, `serde_json 1.0.150` | Manifest v2 typed serialization/deserialization and validation fixtures. [VERIFIED: Cargo.lock; tools/xtask/src/main.rs; tools/flash/src/main.rs] | Existing manifest and flash tooling already use typed JSON. [VERIFIED: tools/xtask/src/main.rs; tools/flash/src/main.rs] |
| `sha2` | `0.11.0` | SHA-256 checksums for release artifacts. [VERIFIED: Cargo.lock; tools/xtask/src/main.rs] | Current manifest v1 already computes SHA-256 checksums. [VERIFIED: tools/xtask/src/main.rs] |
| `cargo-about` | `0.9.0` | Rust crate license inventory generation. [VERIFIED: cargo info cargo-about] | It is purpose-built for dependency license listing, but it only covers the Cargo dependency graph, so Phase 7 still needs explicit non-Rust inventory fields. [VERIFIED: cargo info cargo-about; VERIFIED: 07-CONTEXT.md D-19] |

### Supporting

| Library / Tool | Version | Purpose | When to Use |
|----------------|---------|---------|-------------|
| `clap` | `4.6.1` | CLI parsing for `xtask`, `tools/flash`, and any release validator command. [VERIFIED: Cargo.lock; tools/xtask/src/main.rs; tools/flash/src/main.rs] | Keep existing CLI style for package and validation commands. [VERIFIED: tools/xtask/src/main.rs; tools/flash/src/main.rs] |
| `anyhow` / `thiserror` | `anyhow 1.0.100`, `thiserror 2.0.17` | CLI/app errors and library-domain errors. [VERIFIED: Cargo.lock] | Use `anyhow` in CLIs and `thiserror` in reusable crates per repo Rust guidance. [VERIFIED: AGENTS.md] |
| `camino` | `1.2.3` | UTF-8 paths in host tooling. [VERIFIED: Cargo.lock] | Use for manifest and packaging paths when improving `xtask`/parity tooling. [VERIFIED: AGENTS.md typed-parser/clear-code rules] |
| `spdx` | `0.13.4` | SPDX expression validation if Phase 7 adds Rust-side license manifest validation. [VERIFIED: cargo info spdx] | Use only if release validation needs structured SPDX checks beyond `cargo-about`; otherwise avoid another dependency. [VERIFIED: AGENTS.md dependency minimization rule] |
| Node/npm | Node `v24.13.0`, npm `11.6.2` local | Build or inspect upstream AxeOS static assets if planning chooses reference-built assets. [VERIFIED: `node --version`; `npm --version`; reference/esp-miner/main/http_server/axe-os/package.json] | Use only for asset production/provenance tasks; D-10 forbids rewriting Angular AxeOS. [VERIFIED: 07-CONTEXT.md D-10] |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `espflash` for normal package/flash UX | `esptool.py merge_bin` | Upstream uses esptool scripts, but local `esptool.py` is not installed and the project package/flash path already uses `espflash`. Install esptool only if the plan requires byte-for-byte upstream-style update HEX behavior that `espflash` cannot produce. [VERIFIED: reference/esp-miner/merge_bin.sh; `command -v esptool.py`; scripts/package-firmware.sh; CITED: esptool v4.8.1 basic commands] |
| `esp-idf-part` | Custom partition CSV parser | Hand parsing risks offset/size/alignment mistakes; `esp-idf-part` parses and generates ESP-IDF partition tables. [VERIFIED: cargo info esp-idf-part] |
| `cargo-about` | `cargo-license` or manual `cargo metadata` scripts | `cargo-license` is simpler, but `cargo-about` is designed to generate dependency license reports; both still require separate non-Cargo inventory coverage. [VERIFIED: cargo info cargo-about; cargo info cargo-license; 07-CONTEXT.md D-19] |
| Upstream whole-partition OTAWWW | A/B static asset partitions | A/B static partitions improve recoverability but are deferred because they diverge from reference layout. [VERIFIED: 07-CONTEXT.md D-17; deferred ideas] |
| Direct loose assets + manifest v2 | Versioned archive bundle as primary release | Archive bundles are optional supplements only after the direct asset path works. [VERIFIED: 07-CONTEXT.md D-05] |

**Installation:**

```bash
cargo add -p xtask esp-idf-part@0.6.0
cargo install cargo-about --locked
```

Use `cargo add -p xtask spdx@0.13.4` only if Phase 7 implements Rust-side SPDX expression validation. [VERIFIED: cargo info spdx; VERIFIED: AGENTS.md dependency minimization rule]

**Version verification:**

| Package / Tool | Verified Version | Verification Source |
|----------------|------------------|---------------------|
| `esp-idf-svc` | `0.52.1` | `cargo info esp-idf-svc`; `Cargo.lock` [VERIFIED: cargo registry; Cargo.lock] |
| `esp-idf-sys` | `0.37.2` | `cargo info esp-idf-sys`; `Cargo.lock` [VERIFIED: cargo registry; Cargo.lock] |
| `esp-idf-part` | `0.6.0` | `cargo info esp-idf-part` [VERIFIED: cargo registry] |
| `cargo-about` | `0.9.0` | `cargo info cargo-about` [VERIFIED: cargo registry] |
| `sha2` | `0.11.0` | `Cargo.lock` [VERIFIED: Cargo.lock] |
| `serde_json` | `1.0.150` | `Cargo.lock` [VERIFIED: Cargo.lock] |
| `espflash` | `4.0.1` | `espflash --version` [VERIFIED: local command] |
| Bazel | `9.1.1` | `bazel --version`; `.bazelversion` project stack [VERIFIED: local command; AGENTS.md technology stack block] |
| `just` | `1.48.0` | `just --version` [VERIFIED: local command] |

## Architecture Patterns

### Recommended Project Structure

```text
firmware/bitaxe/
├── partitions-ultra205.csv        # Ultra 205 reference-intent partition table [VERIFIED: reference/esp-miner/partitions.csv]
├── static/                        # Rust-owned or generated static asset source for www.bin [ASSUMED]
├── src/
│   ├── boot_validation.rs         # ESP-IDF rollback validation adapter [CITED: ESP-IDF v5.5.4 OTA docs]
│   ├── filesystem.rs              # SPIFFS mount/status adapter [CITED: ESP-IDF v5.5.4 SPIFFS docs]
│   ├── static_files.rs            # HTTP static/recovery adapter [VERIFIED: reference/esp-miner/main/http_server/http_server.c]
│   └── ota_update.rs              # ESP-IDF OTA/partition update adapter [CITED: ESP-IDF v5.5.4 OTA/partition docs]
crates/bitaxe-api/src/
├── update_plan.rs                 # Pure accept/reject/status decisions for OTA and OTAWWW [VERIFIED: 07-CONTEXT.md D-11]
└── static_plan.rs                 # Pure route/static-file resolution decisions [VERIFIED: 07-CONTEXT.md D-09]
tools/xtask/src/
├── package_manifest.rs            # Manifest v2 typed schema [VERIFIED: tools/xtask/src/main.rs]
├── package_artifacts.rs           # Checksums, offsets, tool versions, release metadata [VERIFIED: tools/xtask/src/main.rs]
└── release_validate.rs            # Release gate: manifest, provenance, license inventory [VERIFIED: 07-CONTEXT.md D-18/D-19]
docs/release/
└── ultra-205.md                   # Operator install/update/recovery guide [VERIFIED: 07-CONTEXT.md D-21]
docs/parity/evidence/
└── phase-07-ota-filesystem-release.md # Evidence record for package, OTA, recovery, rollback [VERIFIED: docs/adr/0012-parity-verification-evidence.md]
```

### Pattern 1: Manifest V2 Extends, It Does Not Break, Manifest V1 Consumers

**What:** Add manifest v2 fields while preserving the existing top-level `default_flash_image` field and relative-path behavior that `tools/flash` already consumes. [VERIFIED: tools/flash/src/main.rs; tools/xtask/src/main.rs; 07-CONTEXT.md D-02]

**When to use:** Use this for every Phase 7 package artifact so `just flash board=205` keeps working without requiring operators to know new artifact names. [VERIFIED: 07-CONTEXT.md D-06; tools/flash/src/main.rs]

**Example:**

```rust
// Source: local manifest v1 in tools/xtask + Phase 7 D-02.
// Keep `default_flash_image` stable for tools/flash compatibility.
#[derive(serde::Serialize, serde::Deserialize)]
struct PackageManifestV2 {
    schema_version: u32,
    board: String,
    release_name: String,
    firmware_commit: String,
    reference_commit: String,
    default_flash_image: String,
    artifacts: Vec<ReleaseArtifact>,
    tool_versions: ToolVersions,
    install_notes: Vec<String>,
    provenance: ReleaseProvenance,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct ReleaseArtifact {
    kind: ArtifactKind,
    path: String,
    sha256: String,
    offset: Option<String>,
    flash_size: Option<String>,
    content_type: Option<String>,
}
```

### Pattern 2: Pure Update Planner, Thin Firmware Effects

**What:** Put route access, AP-mode rejection, content-length checks, status labels, and response selection in a host-testable crate; keep ESP-IDF calls, streaming reads, partition erase/write, OTA activation, delays, and restart in `firmware/bitaxe`. [VERIFIED: 07-CONTEXT.md D-11; crates/bitaxe-api/src/route_shell.rs; firmware/bitaxe/src/http_api.rs]

**When to use:** Use this for `/api/system/OTA` and `/api/system/OTAWWW` so host tests cover most accept/reject behavior before hardware testing. [VERIFIED: .planning/REQUIREMENTS.md REL-02/REL-03; reference/esp-miner/main/http_server/http_server.c]

**Example:**

```rust
// Source: Phase 7 D-11/D-12 and existing route_shell access-gate pattern.
pub enum UpdateDecision {
    AcceptFirmwareOta,
    AcceptStaticWww,
    RejectApMode,
    RejectPublicOrigin,
    RejectTooLarge { max_bytes: u32 },
    RejectUnsupported { message: &'static str },
}

pub fn plan_update_request(input: UpdateRequest<'_>) -> UpdateDecision {
    if input.is_ap_mode {
        return UpdateDecision::RejectApMode;
    }

    if !input.access_allowed {
        return UpdateDecision::RejectPublicOrigin;
    }

    if let Some(content_length) = input.maybe_content_length {
        if content_length > input.max_partition_bytes {
            return UpdateDecision::RejectTooLarge {
                max_bytes: input.max_partition_bytes,
            };
        }
    }

    input.kind.accept_decision()
}
```

### Pattern 3: Partition Layout Is a Release Contract

**What:** Treat the partition CSV, generated binary partition table, `www.bin`, factory image offsets, and manifest offsets as one validated contract. [VERIFIED: reference/esp-miner/partitions.csv; reference/esp-miner/merge_bin.sh; 07-CONTEXT.md D-07]

**When to use:** Use this in `xtask package-firmware` and release validation so package generation fails if offsets, names, sizes, or checksums drift from the documented Ultra 205 contract. [VERIFIED: 07-CONTEXT.md D-18; tools/xtask/src/main.rs]

**Example:**

```rust
// Source: ESP-IDF partition table docs + esp-idf-part crate metadata.
fn validate_ultra205_partition_contract(table: &PartitionTable) -> anyhow::Result<()> {
    require_partition(table, "nvs", "data", "nvs", "0x9000", "0x6000")?;
    require_partition(table, "factory", "app", "factory", "0x10000", "4M")?;
    require_partition(table, "www", "data", "spiffs", "0x410000", "3M")?;
    require_partition(table, "ota_0", "app", "ota_0", "0x710000", "4M")?;
    require_partition(table, "ota_1", "app", "ota_1", "0xb10000", "4M")?;
    require_partition(table, "otadata", "data", "ota", "0xf10000", "8K")?;
    Ok(())
}
```

### Pattern 4: Evidence States Must Be Distinct

**What:** Represent package-built, host-tested, firmware-smoked, hardware-verified, interrupted-update-verified, and explicitly-gapped as different evidence states. [VERIFIED: docs/adr/0012-parity-verification-evidence.md; 07-CONTEXT.md D-22/D-23]

**When to use:** Use this in `docs/parity/checklist.md`, evidence docs, and release gate validation so a generated `www.bin` does not imply live OTAWWW safety. [VERIFIED: 07-CONTEXT.md D-16/D-22]

### Anti-Patterns to Avoid

- **Parallel release tooling:** Do not add a second package system beside `just package` -> Bazel -> `scripts/package-firmware.sh` -> `xtask`. [VERIFIED: 07-CONTEXT.md D-01]
- **Manifest v2 that breaks `tools/flash`:** Do not move or rename `default_flash_image` without updating and testing `tools/flash` compatibility. [VERIFIED: tools/flash/src/main.rs; 07-CONTEXT.md D-02]
- **Claiming static update parity from package evidence:** A generated `www.bin` is package evidence, not interrupted OTAWWW recovery evidence. [VERIFIED: 07-CONTEXT.md D-16/D-22]
- **Editing the reference tree:** Do not modify `reference/esp-miner`; derive fixtures and breadcrumbs outside it. [VERIFIED: AGENTS.md; docs/adr/0005-read-only-reference-implementation.md]
- **Raw flash arithmetic in request handlers:** Do not compute unsafe offsets in HTTP handlers; resolve named partitions and use ESP-IDF partition APIs. [CITED: ESP-IDF v5.5.4 partition docs; VERIFIED: reference/esp-miner/main/http_server/http_server.c]

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| ESP-IDF partition table parsing | Custom CSV parser with manual offset math | `esp-idf-part 0.6.0` in host tooling | Partition names, subtypes, offsets, binary generation, and alignment are easy to get subtly wrong. [VERIFIED: cargo info esp-idf-part; CITED: ESP-IDF v5.5.4 partition docs] |
| OTA slot selection and boot activation | Raw writes to OTA metadata | ESP-IDF OTA APIs through `esp-idf-sys`/`esp-idf-svc` | OTA data has power-failure semantics and rollback state that the bootloader interprets. [CITED: ESP-IDF v5.5.4 OTA docs] |
| SPIFFS image sizing/config | Ad hoc packed binary writer | ESP-IDF `spiffs_create_partition_image` or `spiffsgen.py` with matching build config | SPIFFS image parameters must match the runtime SPIFFS configuration. [CITED: ESP-IDF v5.5.4 SPIFFS docs] |
| Release checksums | Homegrown digest code | `sha2` | Existing package manifest code already uses SHA-256 from `sha2`. [VERIFIED: tools/xtask/src/main.rs; Cargo.lock] |
| JSON manifest handling | String concatenation or regex validation | `serde` / `serde_json` typed structs | Existing package/flash tools already deserialize manifest JSON. [VERIFIED: tools/xtask/src/main.rs; tools/flash/src/main.rs] |
| Rust crate license inventory | Manual transitive dependency list | `cargo-about` | Transitive Cargo dependency licensing is tool-solved, but non-Cargo assets still need explicit inventory. [VERIFIED: cargo info cargo-about; 07-CONTEXT.md D-19] |
| Firmware upload buffering | Full image in RAM | Stream `httpd_req_recv` chunks into ESP-IDF OTA or partition writes | Ultra 205 firmware/static images are multi-megabyte artifacts; buffering them in RAM is unnecessary and risky. [VERIFIED: reference/esp-miner/main/http_server/http_server.c; reference/esp-miner/partitions.csv] |

**Key insight:** Phase 7 is a release-safety phase, not just a packaging feature; the complex parts already have ESP-IDF/tooling semantics, and custom replacements would create parity and recovery risks. [VERIFIED: 07-CONTEXT.md; CITED: ESP-IDF v5.5.4 OTA/SPIFFS/partition docs]

## Common Pitfalls

### Pitfall 1: Treating OTA Upload Success as OTA Verification

**What goes wrong:** The firmware responds successfully to `/api/system/OTA`, but first boot validation and rollback behavior are not proven. [VERIFIED: 07-CONTEXT.md D-13; CITED: ESP-IDF v5.5.4 OTA docs]

**Why it happens:** ESP-IDF rollback marks a newly booted OTA image pending verification; the app must mark it valid after diagnostics or mark it invalid and reboot to roll back. [CITED: ESP-IDF v5.5.4 OTA docs]

**How to avoid:** Add boot-time validation logic, evidence logs for pending/valid/rollback states, and hardware tests that include a bad or intentionally failed validation path. [CITED: ESP-IDF v5.5.4 OTA docs; VERIFIED: .planning/REQUIREMENTS.md REL-08]

**Warning signs:** Evidence says only "upload returned 200" or "next partition selected" without boot logs and rollback state. [VERIFIED: docs/adr/0012-parity-verification-evidence.md]

### Pitfall 2: Breaking `just flash board=205` While Improving Release Packaging

**What goes wrong:** Manifest v2 renames or nests `default_flash_image`, so existing `tools/flash` cannot find the image. [VERIFIED: tools/flash/src/main.rs]

**Why it happens:** Release metadata expands faster than the compatibility surface is tested. [VERIFIED: 07-CONTEXT.md D-02]

**How to avoid:** Keep `default_flash_image` top-level, add manifest v1/v2 compatibility tests in `tools/flash`, and run `just package` plus a `tools/flash` manifest-read test. [VERIFIED: tools/flash/src/main.rs; tools/xtask/src/main.rs]

**Warning signs:** `just package` passes but `just flash board=205` fails before invoking `espflash`. [VERIFIED: tools/flash/src/main.rs]

### Pitfall 3: Factory Image Omits `www.bin` or OTA Data

**What goes wrong:** A merged factory image boots firmware but lacks the SPIFFS web partition or initialized OTA data expected by upstream-like recovery/update flows. [VERIFIED: scripts/package-firmware.sh; reference/esp-miner/merge_bin.sh]

**Why it happens:** Current package script merges only the app ELF through `espflash save-image`; upstream merge scripts explicitly place bootloader, partition table, app, `www.bin`, and OTA data at offsets. [VERIFIED: scripts/package-firmware.sh; reference/esp-miner/merge_bin.sh]

**How to avoid:** Generate/validate partition table and `www.bin`, pass explicit offsets to image generation or write-bin flows, and record artifact offsets in manifest v2. [VERIFIED: 07-CONTEXT.md D-04/D-07; CITED: espflash 4.0.1 README; CITED: esptool v4.8.1 basic commands]

**Warning signs:** Manifest lists a factory image but no `www` artifact, no `otadata`, or no offset table. [VERIFIED: 07-CONTEXT.md D-02/D-04]

### Pitfall 4: SPIFFS Capacity and Power-Loss Behavior Are Overlooked

**What goes wrong:** Static assets fit in source form but exceed practical SPIFFS capacity, or interrupted writes corrupt the `www` filesystem. [CITED: ESP-IDF v5.5.4 SPIFFS docs]

**Why it happens:** ESP-IDF documents SPIFFS as not real-time, lacking directories, practically limited to about 75 percent usable partition space, and vulnerable to corruption during power loss. [CITED: ESP-IDF v5.5.4 SPIFFS docs]

**How to avoid:** Track generated `www.bin` size, leave margin below the 3M reference partition, prefer recovery-page failover, and require interrupted-update evidence before claiming OTAWWW parity. [VERIFIED: reference/esp-miner/partitions.csv; 07-CONTEXT.md D-16; CITED: ESP-IDF v5.5.4 SPIFFS docs]

**Warning signs:** Release docs claim OTAWWW parity without interrupted-update evidence. [VERIFIED: 07-CONTEXT.md D-16/D-23]

### Pitfall 5: Route Order Makes Recovery or APIs Unreachable

**What goes wrong:** Static wildcard routes capture `/api/*` or `/recovery`, or filesystem failure serves broken paths instead of recovery behavior. [VERIFIED: reference/esp-miner/main/http_server/http_server.c]

**Why it happens:** ESP-IDF HTTP server URI registration order and wildcard matching must be coordinated with API routes and static serving. [VERIFIED: reference/esp-miner/main/http_server/http_server.c; CITED: ESP-IDF v5.5.4 HTTP server docs]

**How to avoid:** Register `/recovery` explicitly, keep `/api/*` behavior intact, and use a filesystem-unavailable static fallback that makes recovery explicit. [VERIFIED: reference/esp-miner/main/http_server/http_server.c; 07-CONTEXT.md D-08]

**Warning signs:** `/api/system/info` or `/api/system/OTA` returns static HTML, or missing static files return opaque 404s instead of the upstream-visible redirect/fallback. [VERIFIED: reference/esp-miner/main/http_server/http_server.c]

### Pitfall 6: Access Gate Parity Misses AP-Mode Rejection

**What goes wrong:** General private-network/origin checks pass but OTA routes accept uploads in AP/APSTA mode, unlike the reference behavior. [VERIFIED: reference/esp-miner/main/http_server/http_server.c; crates/bitaxe-api/src/route_shell.rs]

**Why it happens:** Existing route shell allows AP mode as an access decision, but upstream OTA and OTAWWW explicitly reject AP/APSTA update mode. [VERIFIED: crates/bitaxe-api/src/route_shell.rs; reference/esp-miner/main/http_server/http_server.c]

**How to avoid:** Add a distinct OTA/OTAWWW AP-mode decision after normal access-gate evaluation and cover it in host tests. [VERIFIED: 07-CONTEXT.md D-12; crates/bitaxe-api/src/route_shell.rs]

**Warning signs:** OTA tests only exercise public/private origin cases and never set AP-mode runtime state. [VERIFIED: crates/bitaxe-api/src/route_shell.rs]

### Pitfall 7: License Inventory Covers Cargo Only

**What goes wrong:** Release gate passes Rust dependency licenses but misses Bazel/rules, ESP-IDF components, flashing tools, and static assets. [VERIFIED: 07-CONTEXT.md D-19]

**Why it happens:** Cargo license tools see Cargo dependency graphs, not all firmware release inputs. [VERIFIED: cargo info cargo-about; 07-CONTEXT.md D-19]

**How to avoid:** Generate Cargo inventory with `cargo-about`, then add an explicit project release inventory for Bazel, ESP-IDF, esp-rs, espflash, static assets, generated artifacts, and provenance review. [VERIFIED: cargo info cargo-about; PROVENANCE.md; 07-CONTEXT.md D-19/D-20]

**Warning signs:** A release artifact contains `www.bin` but the license/provenance manifest has no static asset row. [VERIFIED: PROVENANCE.md; 07-CONTEXT.md D-19]

## Code Examples

Verified patterns from official and local sources. [VERIFIED: local code inventory; CITED: ESP-IDF v5.5.4 docs]

### Boot Validation Adapter Shape

```rust
// Source: ESP-IDF v5.5.4 OTA rollback docs.
// Keep diagnostics policy pure; keep ESP-IDF state mutation in firmware.
pub fn validate_boot_after_diagnostics(diagnostics_ok: bool) -> Result<(), BootValidationError> {
    let running = current_running_partition()?;
    let state = ota_state_for_partition(running)?;

    if state != OtaImageState::PendingVerify {
        return Ok(());
    }

    if diagnostics_ok {
        mark_running_app_valid()?;
        return Ok(());
    }

    mark_running_app_invalid_and_reboot()?;
    Ok(())
}
```

### Static File Resolution Shape

```rust
// Source: upstream static handler behavior and Phase 7 D-09.
pub fn resolve_static_request(path: &str, fs_available: bool) -> StaticDecision {
    if path == "/recovery" {
        return StaticDecision::EmbeddedRecoveryPage;
    }

    if !fs_available {
        return StaticDecision::RecoveryFallback;
    }

    let normalized = if path == "/" { "/index.html" } else { path };
    if has_file_with_suffix(normalized, ".gz") {
        return StaticDecision::ServeGzip {
            path: format!("{normalized}.gz"),
            cache_static_asset: !normalized.ends_with('/'),
        };
    }

    if has_file(normalized) {
        return StaticDecision::ServePlain {
            path: normalized.to_owned(),
            cache_static_asset: !normalized.ends_with('/'),
        };
    }

    StaticDecision::RedirectToRoot
}
```

### OTA Streaming Adapter Shape

```rust
// Source: upstream OTA streaming route and ESP-IDF OTA docs.
pub fn stream_firmware_ota<R: ChunkReader>(
    reader: &mut R,
    status: &mut UpdateStatus,
) -> Result<OtaApplyResult, OtaApplyError> {
    let partition = next_update_partition()?;
    let mut handle = begin_ota(partition)?;
    let mut bytes_written = 0_u32;

    while let Some(chunk) = reader.next_chunk()? {
        write_ota_chunk(&mut handle, chunk)?;
        bytes_written += chunk.len() as u32;
        status.record_progress(bytes_written);
    }

    finish_ota(handle)?;
    set_boot_partition(partition)?;
    status.record_finished();
    Ok(OtaApplyResult::RebootScheduled)
}
```

### Manifest Artifact Entry Shape

```json
{
  "schema_version": 2,
  "board": "205",
  "release_name": "ultra205-v1-rc",
  "default_flash_image": "bitaxe-ultra205.elf",
  "artifacts": [
    {
      "kind": "firmware_ota_image",
      "path": "bitaxe-ultra205.bin",
      "offset": "0x10000",
      "sha256": "..."
    },
    {
      "kind": "www_spiffs_image",
      "path": "www.bin",
      "offset": "0x410000",
      "sha256": "..."
    },
    {
      "kind": "factory_merged_image",
      "path": "bitaxe-ultra205-factory.bin",
      "offset": "0x0",
      "sha256": "..."
    }
  ]
}
```

Source: Phase 7 manifest decisions, current manifest v1 fields, and upstream offset contract. [VERIFIED: 07-CONTEXT.md D-02/D-04; tools/xtask/src/main.rs; reference/esp-miner/merge_bin.sh]

## State of the Art

| Old Approach | Current Approach | When Changed / Source | Impact |
|--------------|------------------|-----------------------|--------|
| Successful OTA upload treated as enough | Rollback-capable OTA with first-boot validation evidence | ESP-IDF v5.5.4 OTA rollback docs [CITED: ESP-IDF v5.5.4 OTA docs] | Phase 7 must prove boot validation or rollback behavior before REL-08 parity. [VERIFIED: .planning/REQUIREMENTS.md REL-08] |
| Static assets embedded only in release prose | `www.bin` as first-class manifest artifact with offset/checksum | Phase 7 D-02/D-03/D-06 [VERIFIED: 07-CONTEXT.md] | Owners and tools can inspect and upload upstream-compatible direct assets. [VERIFIED: 07-CONTEXT.md D-03] |
| Package manifest v1 contains only minimal artifacts | Manifest v2 includes app, factory, `www.bin`, update-only when present, checksums, offsets, source/ref commits, tools, install notes, provenance/license paths | Phase 7 D-02/D-18/D-19 [VERIFIED: 07-CONTEXT.md] | Release gate can validate artifacts instead of relying on prose release notes. [VERIFIED: 07-CONTEXT.md D-18] |
| Package-only proof for web assets | Separate package, live recovery, live OTAWWW, and interrupted-update evidence states | Phase 7 D-16/D-22/D-23 [VERIFIED: 07-CONTEXT.md] | Final V1 parity cannot outrun hardware/recovery evidence. [VERIFIED: 07-CONTEXT.md D-23] |

**Deprecated/outdated:**

- Do not use `cargo-espmonitor`; project stack says monitor functionality moved into `espflash`/`cargo-espflash` monitor commands. [VERIFIED: AGENTS.md technology stack block]
- Do not use PlatformIO as the production builder; project stack selects native ESP-IDF Rust through `esp-idf-sys`. [VERIFIED: AGENTS.md technology stack block]
- Do not choose ESP-IDF 6 as the first Phase 7 baseline; project stack pins the current production baseline to ESP-IDF `v5.5.4`. [VERIFIED: AGENTS.md technology stack block]

## Existing Repo Integration Points

| Area | Current State | Phase 7 Planning Implication |
|------|---------------|------------------------------|
| Human commands | `Justfile` exposes `build`, `test`, `package`, `flash`, `monitor`, `flash-monitor`, `verify-reference`, and `parity`. [VERIFIED: Justfile] | Keep all user-facing Phase 7 release flows behind existing `just` commands. [VERIFIED: 07-CONTEXT.md D-01/D-06] |
| Bazel package target | `firmware_image` currently emits `bitaxe-ultra205.elf`, `bitaxe-ultra205-factory.bin`, and `bitaxe-ultra205-package.json`. [VERIFIED: firmware/bitaxe/BUILD.bazel] | Add `www.bin`, partition artifacts, release/provenance outputs, and manifest v2 outputs without replacing the target. [VERIFIED: 07-CONTEXT.md D-01/D-02] |
| Package script | `scripts/package-firmware.sh` copies the ELF, calls `espflash save-image --chip esp32s3 --merge`, then runs `xtask package-firmware`. [VERIFIED: scripts/package-firmware.sh] | Expand explicit partition/static/image inputs; current factory image is not enough for complete reference-style factory/recovery packaging. [VERIFIED: reference/esp-miner/merge_bin.sh] |
| Manifest generator | `tools/xtask` writes schema version 1, checksums, commits, tool versions, default flash image, and artifact list. [VERIFIED: tools/xtask/src/main.rs] | Extend to schema version 2 and add compatibility tests. [VERIFIED: 07-CONTEXT.md D-02] |
| Flash tool | `tools/flash` reads only `default_flash_image`, builds package first, and invokes `espflash flash --chip esp32s3`. [VERIFIED: tools/flash/src/main.rs] | Manifest v2 must preserve this read path. [VERIFIED: 07-CONTEXT.md D-02] |
| HTTP API shell | `/api/system/OTA` and `/api/system/OTAWWW` are registered as fail-closed unsupported update handlers. [VERIFIED: firmware/bitaxe/src/http_api.rs; crates/bitaxe-api/src/route_shell.rs] | Replace or route these handlers through pure update planning plus firmware adapters. [VERIFIED: 07-CONTEXT.md D-11/D-12] |
| Boot logs | Firmware logs app identity, reset reason, running partition label, firmware commit, reference commit, and IDF version. [VERIFIED: firmware/bitaxe/src/main.rs] | Add OTA image state/validation logs for rollback evidence. [CITED: ESP-IDF v5.5.4 OTA docs; VERIFIED: .planning/REQUIREMENTS.md REL-08] |
| SDK config | Rust firmware SDK defaults include target/logging/HTTP WS but no custom partition/SPIFFS/rollback settings. [VERIFIED: firmware/bitaxe/sdkconfig.defaults] | Add explicit custom partition table, SPIFFS, rollback, and any HTTP/static limits needed for Phase 7. [VERIFIED: reference/esp-miner/sdkconfig.defaults; CITED: ESP-IDF v5.5.4 OTA/SPIFFS/partition docs] |
| Parity checklist | FS-001, OTA-001, OTA-002, REL-001, REL-002, and REL-003 are Phase 7 rows. [VERIFIED: docs/parity/checklist.md] | Update statuses only with matching package/live/hardware/gap evidence. [VERIFIED: 07-CONTEXT.md D-22] |

## Upstream Reference Behavior To Preserve

| Surface | Reference Behavior | Planning Notes |
|---------|--------------------|----------------|
| Partition layout | NVS at `0x9000`, PHY at `0xf000`, factory at `0x10000` size `4M`, `www` SPIFFS at `0x410000` size `3M`, `ota_0` at `0x710000` size `4M`, `ota_1` at `0xb10000` size `4M`, `otadata` at `0xf10000` size `8k`, coredump `64K`. [VERIFIED: reference/esp-miner/partitions.csv] | Preserve names, offsets, and intent unless manifest/docs/checklist/evidence explicitly explain a change. [VERIFIED: 07-CONTEXT.md D-07] |
| Merged image offsets | Upstream factory merge places bootloader at `0x0`, partition table at `0x8000`, app at `0x10000`, `www.bin` at `0x410000`, and OTA data at `0xf10000`. [VERIFIED: reference/esp-miner/merge_bin.sh] | Manifest v2 should record offsets and generated artifact provenance. [VERIFIED: 07-CONTEXT.md D-02/D-04] |
| SPIFFS mount | Reference registers SPIFFS with empty base path, partition label default, max files 5, and no format-on-mount-fail, then logs capacity and availability. [VERIFIED: reference/esp-miner/main/filesystem.c] | Rust firmware should make mount success/failure visible in logs/status and not silently serve broken static paths. [VERIFIED: 07-CONTEXT.md D-08] |
| Static serving | Reference maps `/` to `index.html`, prefers `.gz`, sets gzip and cache headers, redirects missing files to `/`, and exposes `/recovery`. [VERIFIED: reference/esp-miner/main/http_server/http_server.c] | Unit-test pure resolver behavior and smoke-test HTTP headers/paths on device. [VERIFIED: 07-CONTEXT.md D-09] |
| Recovery page | Reference embeds a page that uploads `www.bin` to `/api/system/OTAWWW` and warns not to restart during update. [VERIFIED: reference/esp-miner/main/http_server/recovery_page.html] | Rust page can be copied only with GPL/provenance handling or re-authored to match behavior without copying expression. [VERIFIED: PROVENANCE.md; docs/adr/0013-mit-first-with-gpl-guardrails.md] |
| Firmware OTA | Reference rejects AP/APSTA mode, streams binary chunks to the next OTA partition, reports progress/status, validates/activates the partition, returns success text, delays, and restarts. [VERIFIED: reference/esp-miner/main/http_server/http_server.c] | Rust implementation should add ESP-IDF rollback validation beyond the reference success path because Phase 7 requires rollback evidence. [VERIFIED: 07-CONTEXT.md D-13; CITED: ESP-IDF v5.5.4 OTA docs] |
| OTAWWW | Reference rejects AP/APSTA mode, locates `www` SPIFFS partition, rejects oversized uploads, erases partition chunks, writes upload chunks, reports progress/status, and returns success text. [VERIFIED: reference/esp-miner/main/http_server/http_server.c] | Whole-partition update is parity-compatible but must not be marked verified without interrupted-update evidence. [VERIFIED: 07-CONTEXT.md D-14/D-16] |
| Owner docs | Reference docs show OTA upload of `esp-miner.bin`, OTAWWW upload of `www.bin`, recovery page use, and factory flashing through owner tooling. [VERIFIED: reference/esp-miner/readme.md; reference/esp-miner/flashing.md] | Phase 7 docs need equivalent Rust firmware artifact names, commands, and safety/recovery caveats. [VERIFIED: 07-CONTEXT.md D-21] |

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|-------------|-----------|---------|----------|
| `cargo` | Rust tools/tests/package helpers | yes | `1.88.0-nightly` | None needed. [VERIFIED: `cargo --version`] |
| `rustc` | Rust builds/tests | yes | `1.88.0-nightly` | None needed. [VERIFIED: `rustc --version`] |
| Bazel | Canonical build graph | yes | `9.1.1` | None needed. [VERIFIED: `bazel --version`] |
| `just` | Human command surface | yes | `1.48.0` | None needed. [VERIFIED: `just --version`] |
| `espflash` | Flash, monitor, package image support | yes | `4.0.1` | None needed for current package/flash path. [VERIFIED: `espflash --version`; `espflash --help`] |
| Node/npm | Optional static asset build/provenance inspection | yes | Node `v24.13.0`, npm `11.6.2` | Avoid Angular rebuild unless needed; D-10 forbids UI rewrite. [VERIFIED: `node --version`; `npm --version`; 07-CONTEXT.md D-10] |
| `espup` | ESP Rust toolchain bootstrap | yes, older than stack recommendation | `0.15.1` | Upgrade only if toolchain bootstrap/reinstall becomes part of the plan. [VERIFIED: `espup --version`; AGENTS.md technology stack block] |
| `ldproxy` | ESP-IDF Rust linker proxy | yes | Version flag not usable locally | Existing ESP Rust builds can use installed linker; verify during firmware build. [VERIFIED: `command -v ldproxy`] |
| `esptool.py` | Exact upstream merge/update script compatibility | no | none | Use `espflash` for normal flows; install esptool only if exact upstream `merge_bin --format hex` output is required. [VERIFIED: `command -v esptool.py`; reference/esp-miner/merge_bin.sh] |
| `cargo-about` | Rust license inventory | no | none installed | Planner should add install step; manual `cargo metadata` fallback is weaker and not recommended for REL-05. [VERIFIED: `command -v cargo-about`; cargo info cargo-about] |
| `cargo-deny` | Optional license/advisory policy | no | none installed | Not required by locked decisions; do not add unless policy value is concrete. [VERIFIED: `command -v cargo-deny`; AGENTS.md dependency minimization rule] |

**Missing dependencies with no fallback:**

- `cargo-about` is missing and should be installed or otherwise supplied before claiming a complete Rust dependency license inventory. [VERIFIED: `command -v cargo-about`; .planning/REQUIREMENTS.md REL-05]

**Missing dependencies with fallback:**

- `esptool.py` is missing; current project flows can use `espflash`, but exact upstream `merge_bin --format hex` reproduction would require installing esptool or explicitly documenting a non-HEX alternative. [VERIFIED: `command -v esptool.py`; scripts/package-firmware.sh; reference/esp-miner/merge_bin.sh]

## Validation Architecture

This section is included because `workflow.nyquist_validation` is explicitly `true`. [VERIFIED: .planning/config.json]

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust unit tests through Cargo/Bazel, package validation through `xtask`, parity checks through `tools/parity`. [VERIFIED: Cargo.toml; MODULE.bazel; Justfile; tools/xtask/src/main.rs; tools/parity/src/main.rs] |
| Config file | `Cargo.toml`, `Cargo.lock`, `MODULE.bazel`, `Justfile`, and Bazel BUILD files. [VERIFIED: repo root files] |
| Quick run command | `cargo test -p bitaxe-api --all-features && cargo test -p xtask --all-features` [VERIFIED: Cargo.toml workspace] |
| Full suite command | `just test && just parity` [VERIFIED: Justfile] |

### Phase Requirements -> Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|--------------|
| REL-01 | Partition layout, SPIFFS image metadata, static route resolution, recovery fallback. [VERIFIED: .planning/REQUIREMENTS.md] | unit + package + hardware smoke | `cargo test -p bitaxe-api --all-features static && cargo test -p xtask --all-features partition` | no, Wave 0 gap for new static/partition tests. [VERIFIED: rg static/partition tests] |
| REL-02 | Firmware OTA accept/reject/apply/log/recover decisions. [VERIFIED: .planning/REQUIREMENTS.md] | unit + firmware smoke + hardware OTA | `cargo test -p bitaxe-api --all-features ota && just build` | no, Wave 0 gap for OTA planner tests and firmware adapter tests. [VERIFIED: firmware/bitaxe/src/http_api.rs] |
| REL-03 | OTAWWW implemented or explicit V1 gap. [VERIFIED: .planning/REQUIREMENTS.md] | unit + manual hardware interruption or gap validation | `cargo test -p bitaxe-api --all-features otawww && just parity` | no, Wave 0 gap for OTAWWW decision/gap tests. [VERIFIED: crates/bitaxe-api/src/route_shell.rs] |
| REL-04 | Manifest v2 artifacts, checksums, offsets, commits, metadata, install notes. [VERIFIED: .planning/REQUIREMENTS.md] | unit + package | `cargo test -p xtask --all-features package_manifest && just package` | partial: manifest v1 tests exist; v2 tests needed. [VERIFIED: tools/xtask/src/main.rs] |
| REL-05 | License inventory, provenance manifest, GPL review gate. [VERIFIED: .planning/REQUIREMENTS.md] | tool + docs/release gate | `cargo about generate about.hbs > target/license.html` plus project release validator | no, Wave 0 gap for cargo-about config and non-Cargo inventory validation. [VERIFIED: command -v cargo-about; PROVENANCE.md] |
| REL-06 | Package and flash flows need no manual artifact discovery. [VERIFIED: .planning/REQUIREMENTS.md] | package + flash-tool unit/smoke | `just package && cargo test -p bitaxe-flash --all-features` | partial: package/flash code exists; v2 compatibility tests needed. [VERIFIED: tools/flash/src/main.rs; tools/flash/Cargo.toml] |
| REL-07 | Operator docs cover connected Ultra 205 install/update/recovery. [VERIFIED: .planning/REQUIREMENTS.md] | docs review + command smoke | `just package && just parity` | no, Wave 0 gap for `docs/release/ultra-205.md`. [VERIFIED: find docs -maxdepth 3 -iname '*release*'] |
| REL-08 | Rollback, recovery, large erase, failed update, interrupted update evidence before release parity. [VERIFIED: .planning/REQUIREMENTS.md] | hardware evidence + parity gate | `just parity` plus manual Ultra 205 evidence capture | no, Wave 0 gap for evidence doc and parity rows. [VERIFIED: docs/parity/checklist.md] |

### Sampling Rate

- **Per task commit:** Run the relevant `cargo test -p ...` command for the touched pure/tool crate and `git diff --check`. [VERIFIED: AGENTS.md verification rules]
- **Per wave merge:** Run `just test && just package && just parity`; add `just build` when firmware files or SDK config change. [VERIFIED: Justfile; AGENTS.md verification rules]
- **Phase gate:** Full suite green, package artifacts generated, manifest v2 validated, license/provenance inventory present, operator docs reviewed, and hardware/evidence rows updated before `/gsd-verify-work`. [VERIFIED: 07-CONTEXT.md D-18/D-23; .planning/REQUIREMENTS.md REL-04/REL-05/REL-08]

### Wave 0 Gaps

- [ ] `crates/bitaxe-api/src/update_plan.rs` and tests for OTA/OTAWWW accept/reject/status decisions. [VERIFIED: 07-CONTEXT.md D-11; crates/bitaxe-api/src/route_shell.rs]
- [ ] `crates/bitaxe-api/src/static_plan.rs` and tests for `/`, `.gz`, cacheable assets, missing redirect, `/recovery`, and filesystem-unavailable fallback. [VERIFIED: 07-CONTEXT.md D-09; reference/esp-miner/main/http_server/http_server.c]
- [ ] `tools/xtask` manifest v2 tests for artifact kinds, offsets, checksums, source/reference commits, install notes, and backwards-compatible `default_flash_image`. [VERIFIED: tools/xtask/src/main.rs; 07-CONTEXT.md D-02]
- [ ] Partition table validation fixtures and either `esp-idf-part` integration tests or generated partition binary checks. [VERIFIED: reference/esp-miner/partitions.csv; cargo info esp-idf-part]
- [ ] `www.bin` generation fixture or build target with size/provenance checks. [VERIFIED: 07-CONTEXT.md D-06/D-19]
- [ ] Cargo license inventory config plus explicit non-Cargo inventory schema. [VERIFIED: 07-CONTEXT.md D-19; cargo info cargo-about]
- [ ] `docs/release/ultra-205.md` operator guide. [VERIFIED: 07-CONTEXT.md D-21]
- [ ] `docs/parity/evidence/phase-07-ota-filesystem-release.md` evidence record with separate package/live/hardware/gap conclusions. [VERIFIED: docs/adr/0012-parity-verification-evidence.md; 07-CONTEXT.md D-22]

## Security Domain

Security enforcement is enabled because `.planning/config.json` does not set `security_enforcement` to `false`. [VERIFIED: .planning/config.json]

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|------------------|
| V2 Authentication | no | No authenticated user system is in Phase 7 scope; rely on the existing private-network/origin gate and document that this is access restriction, not authentication. [VERIFIED: crates/bitaxe-api/src/route_shell.rs; 07-CONTEXT.md D-12] |
| V3 Session Management | no | No sessions are introduced in Phase 7. [VERIFIED: 07-CONTEXT.md; crates/bitaxe-api/src/route_shell.rs] |
| V4 Access Control | yes | Keep private-network/origin checks and add OTA-specific AP/APSTA rejection before accepting firmware/static uploads. [VERIFIED: crates/bitaxe-api/src/route_shell.rs; reference/esp-miner/main/http_server/http_server.c] |
| V5 Input Validation | yes | Validate route kind, origin, AP mode, content length, partition existence, partition size, artifact paths, manifest schema, offsets, checksums, and license/provenance rows. [VERIFIED: 07-CONTEXT.md; reference/esp-miner/main/http_server/http_server.c; tools/xtask/src/main.rs] |
| V6 Cryptography | yes | Use SHA-256 checksums for integrity metadata; do not represent checksums as authenticity/signature controls unless signing is explicitly added. [VERIFIED: tools/xtask/src/main.rs; .planning/REQUIREMENTS.md REL-04] |
| V10 Malicious Code | yes | Release artifacts, static assets, and upstream-derived materials need provenance, license inventory, source/reference commits, and GPL review before publication. [VERIFIED: PROVENANCE.md; docs/adr/0013-mit-first-with-gpl-guardrails.md; 07-CONTEXT.md D-19/D-20] |

### Known Threat Patterns for ESP-IDF OTA / Static Release Stack

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Browser-origin or public-network upload abuse | Spoofing / Elevation of privilege | Preserve private-network/origin gate and add OTA-specific AP-mode rejection. [VERIFIED: crates/bitaxe-api/src/route_shell.rs; reference/esp-miner/main/http_server/http_server.c] |
| Oversized `www.bin` corrupts adjacent flash | Tampering / Denial of service | Validate `Content-Length` and write only within the named `www` partition through ESP-IDF partition APIs. [VERIFIED: reference/esp-miner/main/http_server/http_server.c; CITED: ESP-IDF v5.5.4 partition docs] |
| Invalid firmware image becomes active | Tampering / Denial of service | Use `esp_ota_end`, `esp_ota_set_boot_partition`, rollback-capable boot validation, and evidence logs. [CITED: ESP-IDF v5.5.4 OTA docs] |
| Interrupted OTAWWW leaves static UI unusable | Denial of service | Keep `/recovery` embedded and available; require interruption evidence before verified parity or record REL-03 as an explicit V1 gap. [VERIFIED: 07-CONTEXT.md D-16; reference/esp-miner/main/http_server/recovery_page.html] |
| Static file path traversal | Tampering / Information disclosure | Normalize static paths, reject `..`/NUL/path escape cases, and serve only from mounted SPIFFS plus embedded recovery. [VERIFIED: 07-CONTEXT.md D-09; reference/esp-miner/main/http_server/http_server.c] |
| Release artifact provenance drift | Repudiation / Tampering | Record source commit, reference commit, tool versions, checksums, license inventory, and provenance manifest in package metadata. [VERIFIED: 07-CONTEXT.md D-02/D-19/D-20] |

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Phase 7 firmware plus OTA/SPIFFS code can still fit inside the reference 4M app slots. [ASSUMED] | Architecture Patterns / Upstream Reference Behavior | If wrong, planner must schedule partition-size changes with explicit manifest/docs/checklist/evidence updates. |
| A2 | A Rust-owned or reference-built static asset source can be selected without rewriting Angular AxeOS and without violating provenance policy. [ASSUMED] | Recommended Project Structure / Open Questions | If wrong, REL-01/REL-03 must be scoped to package/recovery fixtures or recorded as release gaps. |
| A3 | `esp-idf-part` can cover the host-side partition validation needs without needing custom parser extensions. [ASSUMED] | Standard Stack / Pattern 3 | If wrong, planner may need a small typed validation wrapper around generated ESP-IDF partition artifacts. |

## Open Questions (RESOLVED)

1. **What exact source should produce `www.bin` for V1?**
   - What we know: Phase 7 must not rewrite Angular AxeOS, and upstream builds `www.bin` from AxeOS assets into a SPIFFS partition. [VERIFIED: 07-CONTEXT.md D-10; reference/esp-miner/main/CMakeLists.txt]
   - Resolution: Use a Rust-owned minimal AxeOS-compatible static asset source for Phase 7 package and recovery validation, and record provenance explicitly in `docs/release/provenance-manifest.md`. Do not rewrite Angular AxeOS or treat reference-built GPL assets as MIT-owned release inputs. [RESOLVED: 07-03/07-04/07-05 plans]
   - Plan alignment: `07-04` creates the static/recovery asset surface, `07-05` packages `www.bin`, and `07-06` validates provenance/release-gate records. [RESOLVED: 07-04-PLAN.md; 07-05-PLAN.md; 07-06-PLAN.md]

2. **Will OTAWWW be implemented in Phase 7 or recorded as REL-03 gap?**
   - What we know: Full OTAWWW parity requires whole-`www` partition update behavior and interruption/recovery evidence; otherwise it must stay fail-closed with a documented V1 gap. [VERIFIED: 07-CONTEXT.md D-15/D-16]
   - Resolution: Record OTAWWW as an explicit REL-03 V1 parity gap in this plan set. Keep `/api/system/OTAWWW` fail-closed with public response `Wrong API input` and the exact operator copy required by `07-UI-SPEC.md`; do not implement whole-partition SPIFFS erase/write without scheduled interruption/recovery hardware evidence. [RESOLVED: 07-07-PLAN.md; 07-08-PLAN.md]
   - Plan alignment: `07-07` implements the fail-closed route and evidence entry; `07-08` documents the operator impact and follow-up; `07-09` keeps checklist status below verified. [RESOLVED: 07-07-PLAN.md; 07-08-PLAN.md; 07-09-PLAN.md]

3. **Does Phase 7 need exact upstream update-only HEX output?**
   - What we know: Upstream has `merge_bin.sh -u` using esptool `--format hex`, but current project packaging uses `espflash`, and `esptool.py` is not installed locally. [VERIFIED: reference/esp-miner/merge_bin.sh; scripts/package-firmware.sh; `command -v esptool.py`]
   - Resolution: Do not require exact upstream update-only HEX output for Phase 7. Keep loose app OTA image and `www.bin` as first-class manifest v2 outputs, keep the merged factory image for recovery/USB flash, and defer update-only HEX until a concrete operator workflow requires it. [RESOLVED: 07-05-PLAN.md]
   - Plan alignment: `07-05` deepens the manifest/package contract without adding esptool HEX as a required artifact. [RESOLVED: 07-05-PLAN.md]

## Sources

### Primary (HIGH confidence)

- `.planning/phases/07-ota-filesystem-and-release-packaging/07-CONTEXT.md` - locked Phase 7 decisions, discretion, deferred scope, canonical references, and implementation ideas. [VERIFIED: local file]
- `.planning/REQUIREMENTS.md` - REL-01 through REL-08. [VERIFIED: local file]
- `.planning/STATE.md` and `.planning/ROADMAP.md` - project history and Phase 7 scope. [VERIFIED: local files]
- `AGENTS.md`, `AGENTS.bright-builds.md`, `standards-overrides.md`, `standards/core/*`, `standards/languages/rust.md`, and `standards/languages/typescript-javascript.md` - project constraints and verification rules. [VERIFIED: local files]
- `Justfile`, `firmware/bitaxe/BUILD.bazel`, `scripts/package-firmware.sh`, `tools/xtask/src/main.rs`, `tools/flash/src/main.rs`, `firmware/bitaxe/src/http_api.rs`, `firmware/bitaxe/src/main.rs`, `crates/bitaxe-api/src/route_shell.rs`, `docs/parity/checklist.md` - current integration points. [VERIFIED: local files]
- `reference/esp-miner/partitions.csv`, `merge_bin.sh`, `merge_bin_update.sh`, `flashing.md`, `readme.md`, `main/CMakeLists.txt`, `main/filesystem.c`, `main/http_server/http_server.c`, `main/http_server/openapi.yaml`, `main/http_server/recovery_page.html`, and `main/http_server/axe-os/package.json` - upstream reference behavior. [VERIFIED: local files]
- ESP-IDF v5.5.4 OTA docs: `https://github.com/espressif/esp-idf/blob/v5.5.4/docs/en/api-reference/system/ota.rst` - OTA slots, OTA data, rollback validation. [CITED: official docs]
- ESP-IDF v5.5.4 SPIFFS docs: `https://github.com/espressif/esp-idf/blob/v5.5.4/docs/en/api-reference/storage/spiffs.rst` - SPIFFS image generation, runtime constraints, power-loss behavior. [CITED: official docs]
- ESP-IDF v5.5.4 partition table docs: `https://github.com/espressif/esp-idf/blob/v5.5.4/docs/en/api-guides/partition-tables.rst` - custom partition tables, offsets, app/data subtypes, alignment. [CITED: official docs]
- ESP-IDF v5.5.4 partition API docs: `https://github.com/espressif/esp-idf/blob/v5.5.4/docs/en/api-reference/storage/partition.rst` - partition find/read/write/erase APIs. [CITED: official docs]
- ESP-IDF v5.5.4 HTTP server docs: `https://github.com/espressif/esp-idf/blob/v5.5.4/docs/en/api-reference/protocols/esp_http_server.rst` - URI handler model and HTTP server configuration. [CITED: official docs]
- `espflash` 4.0.1 README: `https://github.com/esp-rs/espflash/blob/v4.0.1/espflash/README.md` - flash, monitor, save-image, write-bin, list-ports, erase commands. [CITED: official repo]
- esptool v4.8.1 basic commands: `https://github.com/espressif/esptool/blob/v4.8.1/docs/en/esptool/basic-commands.rst` - `merge_bin` behavior. [CITED: official docs]
- Cargo registry metadata for `esp-idf-part`, `cargo-about`, `cargo-license`, `spdx`, `esp-idf-svc`, and `esp-idf-sys`. [VERIFIED: cargo registry]

### Secondary (MEDIUM confidence)

- Local command availability probes for `cargo`, `rustc`, `bazel`, `just`, `espflash`, `node`, `npm`, `espup`, `ldproxy`, `esptool.py`, `cargo-about`, and `cargo-deny`. [VERIFIED: local commands]

### Tertiary (LOW confidence)

- None; assumptions are listed explicitly in the Assumptions Log. [VERIFIED: this research]

## Metadata

**Confidence breakdown:**

- Standard stack: HIGH - package versions and tool availability were verified through Cargo registry, lockfile, local commands, and project stack docs. [VERIFIED: Cargo.lock; cargo registry; local commands; AGENTS.md]
- Architecture: MEDIUM-HIGH - integration points and upstream behavior are verified; static asset source and OTAWWW evidence scope were resolved in the Phase 7 plan set. [VERIFIED: local code/reference inventory; 07-04-PLAN.md; 07-07-PLAN.md]
- Pitfalls: HIGH - risks are grounded in Phase 7 decisions, upstream behavior, ESP-IDF official docs, and current repo code. [VERIFIED: 07-CONTEXT.md; reference/esp-miner; CITED: ESP-IDF v5.5.4 docs]
- Validation: MEDIUM-HIGH - existing test infrastructure is present, but Phase 7-specific tests and evidence docs are Wave 0 gaps. [VERIFIED: Justfile; Cargo.toml; tools/xtask; docs/parity/checklist.md]

**Research date:** 2026-06-28 [VERIFIED: system date]
**Valid until:** 2026-07-28 for package/test architecture; re-check ESP-IDF/esp-rs/cargo-about versions before implementation if planning starts after that. [ASSUMED]

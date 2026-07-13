---
generated_by: gsd-discuss-phase
lifecycle_mode: yolo
phase_lifecycle_id: 7-2026-06-28T13-30-15
generated_at: 2026-06-28T13:30:15.161Z
---

# Phase 7: OTA, Filesystem, And Release Packaging - Context

**Gathered:** 2026-06-28
**Status:** Ready for planning
**Mode:** Yolo

<domain>
## Phase Boundary

Phase 7 delivers the Ultra 205 release and update surface: partition and SPIFFS filesystem behavior, static AxeOS assets, recovery page behavior, firmware OTA update flow, OTAWWW/static asset update handling, release artifacts, manifests, checksums, install/update/recovery docs, dependency license inventory, provenance review, and parity evidence.

This phase does not replace AxeOS with a new UI, does not expand V1 beyond Ultra 205/BM1366, and does not weaken earlier safety evidence gates. Firmware OTA and release workflows may move to `implemented` or `verified` only according to evidence. OTAWWW/static update behavior may be implemented with hardware and interruption evidence or explicitly recorded as a V1 parity gap with an owner, evidence, and release impact.

</domain>

<decisions>
## Implementation Decisions

### Release Artifact Packaging And Manifest Contract

- **D-01:** Extend the existing `just package` -> Bazel -> `scripts/package-firmware.sh` -> `tools/xtask` workflow instead of replacing it with a new packaging system.
- **D-02:** Introduce a package manifest v2 contract that preserves current `tools/flash` default-image compatibility while adding release-grade metadata for the Ultra 205 app image, factory image, `www.bin`, update-only image when present, checksums, offsets, source commit, reference commit, ESP-IDF/Rust/tool versions, release name, image metadata, and installation notes.
- **D-03:** Keep upstream-compatible loose binary assets as first-class manifest entries. Owners should still be able to identify and upload the firmware app image and `www.bin` directly where the upstream AxeOS flow expects those names.
- **D-04:** Keep the factory/recovery image as a merged multi-offset artifact with explicit offsets, while app OTA and OTAWWW assets remain individually addressable in the manifest.
- **D-05:** Do not make a versioned archive bundle the primary release contract in Phase 7. An archive or SBOM bundle may be added as an optional supplement only after the direct asset and manifest path is proven.

### Filesystem, Static Assets, And Recovery

- **D-06:** Model Phase 7 around the upstream-style `www` SPIFFS partition, generated `www.bin`, static serving from mounted SPIFFS, and embedded `/recovery` page. This is the default V1 parity target.
- **D-07:** Preserve the reference partition intent for Ultra 205: app factory and OTA slots, `www` data SPIFFS partition, `otadata`, NVS, PHY, and coredump areas. Any offset or size change must be explicit in the manifest, docs, checklist, and evidence.
- **D-08:** Mount filesystem behavior should be visible in logs and status. If SPIFFS is unavailable, the HTTP server should make recovery behavior explicit instead of silently serving broken static paths.
- **D-09:** Static serving should preserve upstream-visible behavior where practical: root directory `index.html`, gzipped file preference when a `.gz` variant exists, cache headers for non-directory static assets, fallback redirect to `/` for missing files, and `/recovery` availability.
- **D-10:** Do not rewrite Angular AxeOS in this phase. Use the existing AxeOS/static asset compatibility target from Phase 5, generated assets, fixtures, or reference-built assets as planning proves practical.

### Firmware OTA And OTAWWW Update Behavior

- **D-11:** Implement firmware OTA as the primary runtime update capability for Phase 7. Use a pure planner/test surface for accept/reject/status/log decisions and thin firmware adapters for ESP-IDF OTA effects.
- **D-12:** Firmware OTA must preserve upstream-visible route behavior for `/api/system/OTA`: private-network/origin gate, AP-mode rejection, binary upload streaming, progress/status updates, protocol/write/validation errors, successful text response, activation of the next app partition, and reboot scheduling.
- **D-13:** Prefer ESP-IDF rollback-capable OTA semantics for the Rust firmware. Plans should include boot validation or explicit rollback evidence, not just a successful upload response.
- **D-14:** Treat OTAWWW separately from firmware OTA. Direct whole-partition SPIFFS rewrite is the upstream-compatible route, but it has a weak interrupted-update recovery story.
- **D-15:** If planning can fit full OTAWWW parity with hardware/interruption evidence, implement `/api/system/OTAWWW` as an upstream-faithful whole-`www` partition update with size checks, chunked erase/write, progress/status, and recovery-page evidence.
- **D-16:** If full OTAWWW parity cannot be proven safely in Phase 7, keep OTAWWW fail-closed and record REL-03 as an explicit V1 parity gap with evidence, owner, release impact, and a follow-up path. Do not claim verified static-update parity from package-only evidence.
- **D-17:** Do not introduce an A/B static asset partition layout in Phase 7 unless planning proves the partition-layout divergence is necessary and worth the parity explanation burden.

### Release Verification, Licensing, And Operator Documentation

- **D-18:** Use a parity-integrated release gate. Release readiness should be backed by manifest checks, `docs/parity/checklist.md`, phase evidence docs, operator docs, and provenance/license review rather than prose-only release notes.
- **D-19:** Generate or update a dependency license inventory covering Rust crates, Bazel/rules dependencies, ESP-IDF Rust bindings, ESP-IDF components, flashing tools, and any web/static assets included in firmware images.
- **D-20:** Apply `PROVENANCE.md` and ADR-0013 before publishing firmware artifacts. Any intentionally ported GPL-covered source expression or included upstream-generated assets must be isolated, attributed, and reviewed instead of marked MIT-only by default.
- **D-21:** Release docs must cover `just build`, `just package`, `just flash board=205`, `just monitor`, `just flash-monitor`, firmware OTA, OTAWWW or its explicit gap, recovery page, large erase behavior, failed update behavior, interrupted update behavior, and safe rollback/recovery procedures for a developer with a connected Ultra 205.
- **D-22:** Parity checklist rows FS-001, OTA-001, OTA-002, REL-001, REL-002, REL-003, and any new release rows must distinguish package/workflow evidence from live firmware OTA, live recovery, and interrupted-update hardware evidence.
- **D-23:** Public or release-candidate artifacts may be produced with explicit gaps, but final V1 parity claims must not outrun hardware, rollback, recovery, license, and provenance evidence.

### the agent's Discretion

The agent may choose exact Rust module names, manifest schema field names, helper crate boundaries, package target names, fixture formats, evidence document names, and whether release validation lives in `tools/parity`, `tools/xtask`, or a focused helper. Those choices must preserve the repo's functional-core/imperative-shell boundary, keep upstream reference files read-only, keep `just` as the human command surface, keep Bazel as the canonical graph, use typed parsers instead of ad hoc string scanning for manifests where practical, and avoid adding dependencies unless the release/compliance value is concrete.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase Scope And Requirements

- `.planning/ROADMAP.md` - Phase 7 goal, dependency on Phase 6, success criteria, verification expectations, and research flags.
- `.planning/REQUIREMENTS.md` - REL-01 through REL-08 plus project-wide evidence and release requirements.
- `.planning/PROJECT.md` - Ultra 205 first target, ESP-IDF Rust stack, architecture constraints, safety constraints, and current state.
- `.planning/STATE.md` - Prior phase decisions, Phase 5 OTA/static deferrals, Phase 6 safety evidence boundaries, and current project status.
- `.planning/phases/01-foundation-and-gamma-601-boot-log/01-CONTEXT.md` - Package/flash/manifest, safe boot/log, reference guard, and parity evidence policy decisions.
- `.planning/phases/05-axeos-api-logs-and-telemetry/05-CONTEXT.md` - AxeOS API/static compatibility boundary and explicit deferral of OTA/OTAWWW, SPIFFS, recovery, and release packaging to Phase 7.
- `.planning/phases/06-safety-controllers-and-self-test/06-CONTEXT.md` - Hardware evidence and safety-gate rules that release claims must preserve.

### Existing Rust-Owned Surfaces

- `Justfile` - Human command surface for build, test, package, flash, monitor, flash-monitor, verify-reference, and parity.
- `firmware/bitaxe/BUILD.bazel` - Firmware and firmware_image Bazel targets and current package outputs.
- `firmware/bitaxe/sdkconfig.defaults` - Current ESP-IDF SDK defaults; Phase 7 must add any OTA, rollback, partition, or filesystem settings explicitly.
- `scripts/package-firmware.sh` - Current package wrapper around espflash save-image and `tools/xtask`.
- `tools/xtask/src/main.rs` - Current package manifest generation, checksums, reference guard, and package metadata.
- `tools/flash/src/main.rs` - Current flash/monitor workflow and manifest default-image reader.
- `firmware/bitaxe/src/http_api.rs` - Current ESP-IDF HTTP route shell, including fail-closed OTA and OTAWWW handlers.
- `crates/bitaxe-api/src/route_shell.rs` - Current route manifest and pure route/unsupported-update planning surface.
- `docs/parity/checklist.md` - Rows FS-001, OTA-001, OTA-002, REL-001, REL-002, REL-003 and release/evidence policy.
- `docs/parity/evidence/` - Phase evidence documents to extend with package, OTA, recovery, rollback, and release-readiness evidence.

### Upstream Reference Behavior

- `reference/esp-miner/partitions.csv` - Upstream partition layout for NVS, factory app, `www` SPIFFS, OTA slots, `otadata`, and coredump.
- `reference/esp-miner/merge_bin.sh` - Upstream factory, config, and update image merge behavior and offsets.
- `reference/esp-miner/merge_bin_update.sh` - Upstream update-only merge wrapper.
- `reference/esp-miner/flashing.md` - Upstream factory flashing instructions and owner workflow expectations.
- `reference/esp-miner/readme.md` - Upstream user-facing OTA, OTAWWW, and recovery documentation.
- `reference/esp-miner/main/CMakeLists.txt` - Upstream SPIFFS image creation and embedded recovery page build behavior.
- `reference/esp-miner/main/filesystem.c` - Upstream SPIFFS mount behavior and filesystem availability logging.
- `reference/esp-miner/main/http_server/http_server.c` - Upstream static file serving, recovery handler, OTA route, OTAWWW route, route registration, AP-mode rejection, progress/status strings, and recovery fallback behavior.
- `reference/esp-miner/main/http_server/openapi.yaml` - Upstream route contract for `/api/system/OTA` and `/api/system/OTAWWW`.
- `reference/esp-miner/main/http_server/recovery_page.html` - Upstream recovery page asset embedded into firmware.
- `reference/esp-miner/main/http_server/axe-os/` - Upstream AxeOS static source and update-page expectations.

### Architecture, Evidence, And Policy

- `docs/adr/0001-device-user-parity.md` - Observable behavior parity definition.
- `docs/adr/0003-esp-idf-rust-production-stack.md` - ESP-IDF Rust stack and adapter boundary.
- `docs/adr/0004-bazel-automation-with-just-wrapper.md` - Bazel/Just automation decision.
- `docs/adr/0005-read-only-reference-implementation.md` - Reference implementation policy.
- `docs/adr/0006-parity-checklist-as-audit-evidence.md` - Checklist as evidence policy.
- `docs/adr/0008-reference-breadcrumb-comments.md` - Breadcrumb placement policy.
- `docs/adr/0009-monorepo-package-layout.md` - Crate, firmware, tools, and script ownership.
- `docs/adr/0010-axeos-api-and-asset-compatibility.md` - API and static asset compatibility before UI rewrite.
- `docs/adr/0012-parity-verification-evidence.md` - Evidence requirements and hardware-control verification gate.
- `docs/adr/0013-mit-first-with-gpl-guardrails.md` - GPL guardrails for upstream-derived behavior and fixtures.
- `docs/adr/0014-pivot-to-ultra-205-bm1366-first-parity.md` - Ultra 205/BM1366 first parity target and deferred non-205 scope.
- `PROVENANCE.md` - Provenance, SPDX, upstream reference, fixture/source attribution, dependency license inventory, and firmware release review policy.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets

- `tools/xtask/src/main.rs` already builds a JSON package manifest with schema version, board, device model, ASIC, firmware commit, reference commit, ESP-IDF version, Rust target, tool versions, default flash image, artifact paths, offsets, and SHA-256 checksums.
- `scripts/package-firmware.sh` already copies the firmware ELF, creates a merged factory image with `espflash save-image --chip esp32s3 --merge`, invokes `xtask package-firmware`, and prints the generated manifest path.
- `tools/flash/src/main.rs` already builds the package target before flashing, reads `default_flash_image` from the manifest, prints the `espflash` command, records optional evidence logs, and rejects non-205 boards.
- `firmware/bitaxe/src/http_api.rs` already owns the ESP-IDF HTTP server shell and registers `/api/system/OTA` and `/api/system/OTAWWW` as fail-closed Phase 7 placeholders.
- `crates/bitaxe-api` already owns pure AxeOS route and unsupported-update response logic from Phase 5 and is the natural home for OTA/OTAWWW accept/reject/status planning that can be unit tested.
- `docs/parity/checklist.md` already has Phase 7 rows for filesystem behavior, OTA, OTAWWW, partition layout, SDK config parity, and release image behavior.

### Established Patterns

- Pure decisions live in crates and firmware owns ESP-IDF, HTTP request streaming, partition writes, OTA activation, task delays, reboot effects, and static filesystem effects.
- `just` remains the human command surface; Bazel remains the canonical automation graph.
- Reference files stay read-only. Use breadcrumbs and fixtures rather than editing `reference/esp-miner`.
- Evidence must distinguish package/workflow proof, host unit/golden proof, API/route proof, firmware smoke, hardware smoke, interrupted-update evidence, and accepted explicit gaps.
- Existing release/package logic already prefers machine-readable JSON manifests with reference guard enforcement; Phase 7 should deepen that contract instead of inventing parallel sidecars.

### Integration Points

- Extend `firmware/bitaxe/BUILD.bazel`, `scripts/package-firmware.sh`, and `tools/xtask` to produce and manifest app OTA, `www.bin`, factory image, update image if used, checksums, offsets, release notes, and license/provenance artifacts.
- Extend `tools/flash` only where needed to consume manifest v2 while preserving existing default image behavior for `just flash board=205`.
- Add firmware filesystem/static/recovery adapters around `esp_vfs_spiffs_register`, static file lookup, recovery-page serving, and status logging.
- Add firmware OTA and OTAWWW adapters in `firmware/bitaxe` while keeping public accept/reject/status decisions host-testable in `crates/bitaxe-api`.
- Update `docs/parity/checklist.md`, `docs/parity/evidence/`, and operator docs as each surface moves from deferred/not-started to implemented, verified, or explicitly gapped.

</code_context>

<specifics>
## Specific Ideas

- Manifest v2 should remain friendly to both `tools/flash` and human release inspection: default flash image, firmware OTA image, `www.bin`, factory image, update-only image if present, offsets, SHA-256 checksums, source commit, reference commit, tool versions, release name, release notes path, and license/provenance paths.
- Firmware OTA evidence should include accepted upload, rejected upload, protocol error, write error where injectable, validation or activation error where injectable, reboot scheduling, rollback or boot-validation path, and live Ultra 205 smoke before `verified`.
- OTAWWW evidence should include partition-not-found, too-large file, chunked erase/write behavior, successful update, recovery access after broken/static missing assets, and interrupted-update behavior if full OTAWWW parity is claimed.
- Static asset evidence should include `/`, `/recovery`, a representative gzipped asset, a representative missing asset redirect to `/`, cache header behavior for non-directory files, and API route coexistence.
- Release evidence should name board, port when hardware is used, firmware commit, reference commit, package manifest path, artifact checksums, commands run, observed update/recovery behavior, and explicit conclusions.
- External primary documentation to consult during research and planning includes ESP-IDF SPIFFS, ESP-IDF partition tables, ESP-IDF OTA/rollback, ESP-IDF HTTP server, and esptool merge/image documentation for the pinned ESP-IDF/Rust baseline.

</specifics>

<deferred>
## Deferred Ideas

- A/B static asset partitions may be a future recoverability improvement, but it is not the Phase 7 default because it diverges from the reference partition layout.
- A self-contained versioned release archive may be useful later, but direct upstream-compatible assets plus manifest v2 are the Phase 7 contract.
- Full Angular AxeOS replacement remains outside V1.
- Gamma 601/BM1370, non-205 boards, additional ASIC families, all-board factory images, Stratum v2, and BAP remain outside Phase 7 unless a later roadmap phase adds separate evidence.

</deferred>

***

*Phase: 07-ota-filesystem-and-release-packaging*
*Context gathered: 2026-06-28*

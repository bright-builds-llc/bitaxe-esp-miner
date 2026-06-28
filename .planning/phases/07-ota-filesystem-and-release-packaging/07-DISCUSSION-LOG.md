# Phase 7: OTA, Filesystem, And Release Packaging - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md - this log preserves the alternatives considered.

**Date:** 2026-06-28T13:30:15.161Z
**Phase:** 7-OTA, Filesystem, And Release Packaging
**Mode:** Yolo
**Areas discussed:** Release artifact packaging and manifest contract, Filesystem/static/recovery behavior, OTA firmware and OTAWWW update behavior, Release verification/license/provenance/operator docs gate

***

## Release Artifact Packaging And Manifest Contract

| Option | Description | Selected |
| --- | --- | --- |
| Manifest v2 sidecar over upstream-compatible loose assets | Preserve the current `just package` workflow and current `tools/flash` manifest reader while adding release-grade metadata for app OTA, `www.bin`, factory image, checksums, provenance, install notes, and release name. | yes |
| Upstream-style assets plus checksum/notes sidecars | Publish direct `esp-miner.bin`, `www.bin`, and factory binaries with separate checksum and notes files. | no |
| Versioned release bundle archive with embedded manifest | Publish one audit-focused archive that contains images, manifest, notes, source commits, reference commit, and license data. | no |

**User's choice:** Auto-selected recommended default: Manifest v2 sidecar over upstream-compatible loose assets.

**Notes:** This keeps existing Bazel/Just/package/flash behavior and deepens the machine-readable manifest instead of adding parallel release metadata. Direct upstream-compatible image names remain first-class manifest entries.

***

## Filesystem, SPIFFS, Static AxeOS Assets, And Recovery Page Behavior

| Option | Description | Selected |
| --- | --- | --- |
| Upstream-compatible `www` SPIFFS image plus embedded recovery page | Match the reference partition/static model: generated `www.bin`, mounted SPIFFS, static serving, embedded `/recovery`, and OTAWWW-compatible asset package. | yes |
| Release-package SPIFFS image now, keep runtime OTAWWW fail-closed | Prove `www.bin` and package layout now, but leave runtime static update unsupported. | no |
| Embedded minimal web/recovery assets in firmware, no SPIFFS `www.bin` | Keep a tiny firmware-embedded fallback and avoid the SPIFFS/static image surface. | no |

**User's choice:** Auto-selected recommended default: Upstream-compatible `www` SPIFFS image plus embedded recovery page.

**Notes:** This is the strongest fit for REL-01 and REL-03. It preserves the Phase 5 decision not to rewrite Angular while still targeting upstream-compatible static asset and recovery behavior.

***

## OTA Firmware And OTAWWW Update Behavior

| Option | Description | Selected |
| --- | --- | --- |
| Firmware OTA with rollback, OTAWWW explicit V1 gap | Implement recoverable firmware OTA first, with OTAWWW documented as a V1 gap if full static update evidence is not safe or feasible. | yes |
| Upstream-faithful OTA plus direct OTAWWW SPIFFS rewrite | Implement both firmware OTA and whole-partition OTAWWW rewrite for maximum route parity, including interruption evidence. | conditional |
| A/B static asset partitions behind upstream OTAWWW route facade | Improve static update recoverability with a staged partition design that diverges from the upstream layout. | no |

**User's choice:** Auto-selected recommended default: Firmware OTA with rollback as mandatory, with OTAWWW implemented only if planning can include safe evidence; otherwise record OTAWWW as an explicit V1 parity gap.

**Notes:** Firmware OTA has a stronger ESP-IDF rollback story than direct SPIFFS rewrite. The context keeps the upstream-faithful OTAWWW route as an accepted conditional target, but blocks verified claims without hardware/interruption evidence.

***

## Release Verification, License/Provenance, And Operator Docs Gate

| Option | Description | Selected |
| --- | --- | --- |
| Parity-integrated release gate | Gate release readiness through manifest validation, parity checklist rows, phase evidence docs, operator docs, dependency license inventory, and provenance review. | yes |
| Docs-only release packet | Rely on manually reviewed release notes, operator docs, and prose provenance. | no |
| Standards-heavy SBOM/provenance framework | Add full SBOM/provenance tooling such as SPDX/CycloneDX/SLSA-style artifacts immediately. | no |
| Hard final-parity release block | Prevent any release-candidate artifact until all V1 parity is final-verified. | no |

**User's choice:** Auto-selected recommended default: Parity-integrated release gate.

**Notes:** This matches `PROVENANCE.md`, ADR-0012, ADR-0013, and existing parity checklist behavior. Optional SBOM/provenance exports may be added if public distribution needs them, but they are not the core Phase 7 gate.

***

## the agent's Discretion

- Exact manifest v2 field names, module names, helper boundaries, fixture formats, and evidence document names.
- Whether release validation belongs in `tools/parity`, `tools/xtask`, or a focused helper, as long as the human command surface remains `just` and automation remains Bazel-backed.
- The exact static asset build source, provided it preserves upstream-compatible AxeOS behavior and read-only reference policy.

## Deferred Ideas

- A/B static asset partitions.
- Primary release archive bundle.
- Full Angular AxeOS replacement.
- Non-205 board release artifacts.

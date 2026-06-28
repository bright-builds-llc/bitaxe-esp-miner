# Phase 7 Release Provenance Manifest

This manifest records the provenance fields required before Ultra 205 release
artifacts can be described publicly. It follows PROVENANCE.md and ADR-0013:
original project work is MIT-first where possible, while upstream ESP-Miner
inputs remain GPL-risk-reviewed release material.

## source commit

- Field owner: release packaging workflow.
- Required value: short and full commit for the Rust firmware source tree.
- Evidence source: package manifest v2 and release gate output.
- Current state: pending package-generation evidence.

## reference commit

- Field owner: reference guard and package manifest workflow.
- Required value: pinned `reference/esp-miner` commit.
- Evidence source: `just verify-reference`, package manifest v2, and parity
  report output.
- Current state: pending release-gate evidence for this phase.

## static asset source

- Field owner: static/SPIFFS packaging workflow.
- Required value: source tree or fixture path used to generate `www.bin`.
- Review requirement: identify whether the asset source is independently
  authored, generated from upstream GPL-covered AxeOS source, or mixed.
- Current state: pending later Phase 7 static asset package plan.

## recovery page source

- Field owner: recovery/static firmware workflow.
- Required value: source path for the embedded `/recovery` page and the
  generated artifact or embedded bytes included in firmware.
- Review requirement: record whether the page is independently authored or
  upstream-derived, and keep any GPL-covered expression out of MIT-only claims.
- Current state: pending recovery/static implementation evidence.

## GPL review status

- Field owner: release reviewer.
- Required value: release decision for upstream-derived source expression,
  reference-built assets, linked components, and distributed firmware images.
- Review requirement: do not describe firmware images as MIT-only until this
  review records that claim explicitly.
- Current state: not complete for release publication.

## release artifact review

- Field owner: release gate.
- Required value: per-artifact path, checksum, source commit, reference commit,
  generation command, license posture, provenance note, and publication status.
- Review requirement: cover `esp-miner.bin`, `www.bin`, merged factory/recovery
  image, package manifest, `cargo-about.html`, license inventory, provenance
  manifest, and install notes.
- Current state: structure present; detailed rows are pending package outputs.

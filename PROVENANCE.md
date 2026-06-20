# Provenance And Licensing

This project is MIT-first for original project work, with GPL guardrails because upstream ESP-Miner is GPL-3.0 and is the behavioral reference implementation.

This document is an engineering policy, not legal advice.

## License Posture

- The root repository currently carries the MIT License.
- Original project scaffolding, docs, scripts, and independently authored Rust code should remain MIT-licensed where possible.
- Upstream ESP-Miner must be included as a pinned read-only submodule at `reference/esp-miner`.
- Upstream ESP-Miner is GPL-3.0 and must keep its own license and notices.
- Any file, crate, or artifact that intentionally ports or incorporates GPL-covered source expression should be marked GPL-3.0-compatible rather than MIT-only.
- Distributed firmware images are release artifacts that need explicit GPL-risk review before publication.

## Reference Implementation

`reference/esp-miner` is the authoritative behavioral reference for device-user parity. It may be used for:

- Behavioral research.
- Reference breadcrumbs.
- Fixture and golden-data derivation when licensing is acceptable.
- API schema comparison.
- Hardware behavior comparison.
- Parity checklist evidence.

It must not be used for:

- Normal project edits.
- Local patches hidden inside the submodule.
- Copy-pasting source expression into MIT-only files.
- Treating upstream C module boundaries as required Rust architecture.

Reference updates should be explicit submodule pointer updates called reference refreshes.

## Breadcrumbs

Rust-owned modules and docs should point to relevant upstream behavior with concise breadcrumbs:

```rust
// Reference: reference/esp-miner/components/asic/bm1370.c
// Parity: docs/parity/checklist.md#asic-bm1370-initialization
```

Breadcrumbs record provenance of behavior. They do not by themselves make the Rust source a copy of upstream code, but they do require careful review if the implementation closely follows source expression.

## SPDX Guidance

Default original files may use:

```text
SPDX-License-Identifier: MIT
```

Files that intentionally port or incorporate GPL-covered source expression should not be MIT-only. Use legal review to choose the exact SPDX expression. Until reviewed, prefer a conservative file-level note such as:

```text
SPDX-License-Identifier: GPL-3.0-only
```

Do not mix unclear licensing in the same file. If only one behavior or fixture is GPL-derived, isolate it in a clearly licensed file or crate.

## Dependency Review

Before release, generate a dependency license inventory for:

- Rust crates.
- Bazel rules and external repositories.
- ESP-IDF Rust bindings and ESP-IDF components.
- Flashing tools.
- Any web/static assets included in firmware images.

Release notes should include the dependency license inventory or a pointer to it.

## Firmware Release Review

Before publishing any firmware image:

- Confirm whether the image includes GPL-covered code, data, generated assets, or linked components.
- Confirm source availability obligations.
- Confirm installation information obligations for user devices.
- Confirm third-party dependency notices.
- Confirm whether the release artifact can be described as MIT-only, GPL-covered, or mixed.

If in doubt, treat the distributed image as GPL-risk-reviewed and do not call it MIT-only.

## Current Known Inputs

| Input | Location | License Posture | Notes |
| --- | --- | --- | --- |
| Project scaffolding | root repo | MIT | Original Bright Builds project files. |
| Upstream ESP-Miner | `reference/esp-miner` | GPL-3.0 | Read-only reference implementation. |
| Parity checklist | `docs/parity/checklist.md` | MIT | Original audit artifact with reference breadcrumbs. |
| Rust firmware | `firmware/bitaxe` | MIT by default | Reassess files that intentionally port GPL expression. |
| Pure Rust crates | `crates/*` | MIT by default | Reassess crates containing ported GPL expression or fixtures. |
| Release images | build artifacts | Needs review | Do not claim MIT-only without review. |

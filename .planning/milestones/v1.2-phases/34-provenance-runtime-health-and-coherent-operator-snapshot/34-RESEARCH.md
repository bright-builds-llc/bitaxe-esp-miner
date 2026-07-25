---
phase: 34
slug: provenance-runtime-health-and-coherent-operator-snapshot
status: complete
researched: "2026-07-15"
source: consolidated-codebase-audits
---

# Phase 34 Research: Provenance, Runtime Health, and Coherent Operator Snapshot

## Research Conclusion

Phase 34 should deepen the existing snapshot and packaging boundaries rather than introduce a parallel identity system. Plan 01 must replace the current split authority—12-character firmware stamp versus live-Git package lookup—with one validated stamp consumed by the ELF, ESP-IDF application descriptor, runtime surfaces, manifest, and admission gates.

The build label is presentation. Exact source proof remains the bare full commit plus package digest. Clean dev builds are therefore admissible; every dirty build fails before hardware interaction.

## Current Integration Points

| Area | Current source | Required change |
| --- | --- | --- |
| Git stamp | `scripts/source-commit-stamp.sh` | Replace short-hash output with stable workspace-status keys and scoped dirty/tag classification. |
| Bazel stamp | `scripts/BUILD.bazel` genrule | Replace ad-hoc local genrule with a Starlark rule consuming `ctx.info_file`. |
| Firmware action | `firmware/bitaxe/BUILD.bazel` | Consume identity outputs and explicit transitive Rust/root/build inputs. |
| Cargo build | `scripts/build-firmware.sh`, `firmware/bitaxe/build.rs` | Require one identity stamp, remove live-Git fallback, and pass compile-time identity fields. |
| ESP-IDF version | Generated ESP-IDF project | Add output-local sdkconfig/defaults with the derived label. |
| LCD | `crates/bitaxe-core/src/lib.rs`, `firmware/bitaxe/src/display_adapter.rs` | Render the complete 22-character maximum build label. |
| Runtime identity | `firmware/bitaxe/src/main.rs`, `runtime_snapshot.rs` | Retain full machine commit and project structured identity. |
| Public wire | `crates/bitaxe-api/src/wire.rs` and fixtures | Add source/channel/dirty/tag fields while keeping `version` as the label. |
| Package | `tools/xtask/src/package_manifest.rs`, `scripts/package-firmware.sh` | Read the same identity stamp, emit schema v3, and stop live-Git firmware identity lookup. |
| Admission | `tools/parity/src/release_gate.rs` and active evidence wrappers | Require v3 identity consistency and reject dirty packages before hardware. |

## Plan 01 Architecture

### Typed identity core

Use one dependency-free Rust type, shared by host tooling and firmware compile-time projection:

- Constructor inputs: full lowercase 40-character commit, dirty boolean, optional validated release tag.
- Derived values: first 12 commit characters, `release` or `dev`, and the exact label.
- Reject uppercase/nonhex/wrong-length commits, invalid tags, contradictory serialized fields, non-ASCII labels, and unknown schema versions.
- Do not accept short commit, channel, or label as independent constructor inputs.

A versioned stamp should serialize all fields so build, package, and tests can verify internal relationships. Shell code must never `eval` stamp content.

### Workspace status and dirty scope

The workspace-status script emits only:

```text
STABLE_BITAXE_SOURCE_COMMIT <40-hex>
STABLE_BITAXE_SOURCE_DIRTY true|false
STABLE_BITAXE_RELEASE_TAG <allowed-tag>|unavailable
```

It reads one checked-in Git pathspec contract and uses porcelain status with untracked files enabled. This covers staged, unstaged, deletion, rename, and untracked nonignored changes without treating planning/docs/reference/evidence edits as firmware dirtiness.

Tags pointing at HEAD are filtered by `^v[0-9]+\.[0-9]+(\.[0-9]+)?$`. Zero is dev, one is release, and more than one fails.

### Bazel correctness

`ctx.info_file` contains ordinary Bazel `BUILD_*` entries in addition to custom keys. The materializer must ignore non-Bitaxe keys but reject unknown, duplicate, missing, or malformed `STABLE_BITAXE_*` entries.

Stable status invalidates clean/dirty/tag/commit transitions but does not invalidate a second edit while the workspace stays dirty. The firmware action must therefore explicitly consume public source filegroups for every Rust dependency plus root Cargo/toolchain/Bazel inputs and identity/build/package scripts. This is preferable to a duplicate workspace-content digest.

### ESP-IDF boundary

Generate two outputs from the identity action: the versioned identity stamp and supplemental sdkconfig defaults containing:

```text
CONFIG_APP_PROJECT_VER_FROM_CONFIG=y
CONFIG_APP_PROJECT_VER="<build-label>"
```

Use an output-local `ESP_IDF_SDKCONFIG` and the tracked defaults followed by the generated defaults. The maximum label is 22 characters, below ESP-IDF's 31-character application version capacity.

### Runtime and API

- `firmware_commit=<40-hex>` stays the machine-readable retained marker.
- Add one redacted `runtime_build_identity` record with label, channel, dirty, and release tag/unavailable.
- `StartupDebugText` consumes the typed identity and renders the full label; it no longer truncates to 12 characters.
- Extend the platform snapshot structurally, then flatten into system-info as `version`, `sourceCommit`, `buildChannel`, `sourceDirty`, and `releaseTag`.
- Live WebSocket inherits the fields from the shared system-info wire type.
- Do not edit the pinned reference OpenAPI. Additive Rust wire/fixture tests own the project contract.

### Manifest and admission

Schema v3 keeps top-level `source_commit` as the full hash and adds:

```json
{
  "build_identity": {
    "label": "<derived-label>",
    "channel": "release-or-dev",
    "source_dirty": false,
    "release_tag": null
  }
}
```

Packaging reads the stamp used for the ELF. Active release/admission gates require v3 and validate the structured projection. Historical committed v2 evidence remains unchanged and readable where intentionally supported.

Admission must reject dirty identity before detector/port/flash/monitor work and compare only the full manifest commit, current full HEAD, embedded/logged commit, and package digest. A fresh scoped dirty check immediately before hardware prevents workspace changes after packaging from escaping the gate.

## Later Phase 34 Plans

Plan 01 supplies source/package identity fields used by later plans. Later work should then:

1. Introduce one boot-session and monotonic operator-snapshot revision across system-info, live WebSocket, retained logs, and evidence projections (OBS-06).
2. Add pinned reference, package, ESP-IDF/static assets, board/ASIC, partition, reset, uptime, and heap facts with explicit unavailable states (SYS-02 through SYS-05).
3. Project passive self-test and supervisor/checkpoint health without starting effects or claiming unproved task-watchdog participation (HLT-01 through HLT-04).

These plans remain software-only. Phase 35 owns the final correlated hardware run and promotion.

## Risks And Mitigations

| Risk | Mitigation |
| --- | --- |
| ELF and manifest describe different Git states | Make the same generated stamp a declared input to both actions; remove live-Git package identity lookup. |
| Dirty-to-dirty edit reuses cached firmware | Declare all transitive firmware/root inputs in Bazel independently of the dirty flag. |
| Stale sdkconfig overrides label | Generate and use an output-local sdkconfig with identity defaults last. |
| Human label used as proof | Keep full commit fields separate and forbid label parsing in admission/evidence tests. |
| Schema v3 breaks historical evidence | Preserve historical v2 files and isolate v2 parsing to historical readers. |
| API change breaks compatibility | Add fields only; retain `version` as a string and preserve unrelated shapes byte-for-byte. |
| Dirty package reaches hardware | Validate manifest identity and fresh workspace dirtiness before any detector/flash operation. |

## Validation Architecture

### Layers

| Layer | Target | Required behavior |
| --- | --- | --- |
| Pure Rust | Typed build identity | Four label states, tag/channel rules, malformed stamp rejection, 22-character bound. |
| Shell | Workspace status | Relevant staged/unstaged/untracked changes dirty; ignored/planning/docs changes clean; tag ambiguity fails. |
| Bazel | Identity/materialization and firmware graph | Stable-key parsing, ordinary `BUILD_*` tolerance, dirty-to-dirty source edit invalidation. |
| Firmware host tests | LCD, retained log, runtime snapshot | Full label fits, full machine commit preserved, structured log exact, heartbeat unchanged. |
| API tests | System-info/live WebSocket | Additive fields serialize with exact names/types and shared values. |
| Package tests | Manifest v3 | Same-stamp full commit and structured identity; malformed/dirty admission rejection; historical v2 unchanged. |
| Canonical build | ELF/package | App descriptor label, manifest identity, embedded full commit, and current scoped state agree. |

### Sampling

- After identity-core work: focused Cargo/Bazel unit tests.
- After workspace/Bazel wiring: shell behavior test plus identity target rebuild test.
- After runtime/API work: affected crate tests and exact heartbeat/source guards.
- After package/admission work: xtask/parity tests and package fixtures.
- At Plan 01 completion: mandatory Rust sequence, shell suites, `bazel test //...`, canonical build/package/reference checks, format/static analysis, manifest/app-descriptor inspection, and full diff review.

### Wave 0

No new test framework is required. Before implementation, add or plan focused test targets for the workspace-status script, typed identity, LCD/API projections, manifest v3, and dirty admission. All Plan 01 tasks must have an automated command and no three consecutive tasks may lack a feedback sample.

### Manual verification

No hardware or manual device step is permitted for Plan 01. ELF/application-descriptor and manifest inspection are automated host checks. Physical current-package qualification remains Phase 35.

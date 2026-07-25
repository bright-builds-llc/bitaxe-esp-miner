---
generated_by: codex
source: approved-user-plan
generated_at: "2026-07-15T03:25:00Z"
---

# Phase 34 PRD: Provenance, Runtime Health, and Coherent Operator Snapshot

## Phase Boundary

Plan the complete Phase 34 roadmap scope, but make build identity and LCD provenance Plan 01 in Wave 1. Plans for coherent snapshot revision, passive health, and remaining identity facts must follow in later waves. This execution pass is authorized for Plan 01 only; no Phase 34 hardware qualification or Phase 35 work is allowed.

## Locked Build Identity Decisions

- Introduce one validated identity with a bare 40-character lowercase `source_commit`, derived 12-character `short_commit`, derived `build_label`, `build_channel` (`release` or `dev`), `source_dirty`, and optional `release_tag`.
- Accept exactly one release tag at HEAD matching `vMAJOR.MINOR` or `vMAJOR.MINOR.PATCH`. No matching tag means `dev`; multiple matching tags or invalid Git state fail the canonical build.
- Derive labels exactly as: clean release `<hash>`, dirty release `<hash>-dirty`, clean dev `<hash>-dev`, dirty dev `<hash>-dirty-dev`. Dirty precedes dev. Dirty and channel are independent.
- Dirty detection includes staged, unstaged, deleted, renamed, and untracked nonignored firmware/package inputs. It excludes planning, docs, evidence, reference, scratch, ignored files, and unrelated host tooling.
- Store the dirty scope in one checked-in pathspec contract covering `firmware/**`, `crates/**`, `.cargo/**`, `tools/xtask/**`, root Cargo/Rust/Bazel/Just inputs, firmware/package Bazel rules, and firmware build/package/identity scripts.
- Dirty builds are never eligible for hardware admission. Clean dev and clean release builds may qualify by full commit and package digest.
- Keep semantic version separate from the human build label. The versioned provenance stamp contains semantic version, the canonical build identity, and the full pinned reference commit.

## Build And Cache Contract

- Replace the short commit genrule with a workspace-status command emitting exactly five primitive `STABLE_BITAXE_*` keys for source commit, dirty state, release tag, semantic version, and reference commit. A custom Starlark rule consumes `ctx.info_file` and invokes the shared Rust materializer to produce a strict versioned provenance stamp; Starlark does not duplicate label/channel derivation.
- Ignore Bazel's ordinary `BUILD_*` status keys, but reject missing, duplicate, malformed, or unknown `STABLE_BITAXE_*` keys.
- Add `.bazelrc` workspace-status wiring. Do not globally enable `--stamp`.
- Explicitly declare all firmware transitive Rust/root/build inputs in Bazel so dirty-to-dirty source edits invalidate the firmware action even while `source_dirty` remains true.
- The ELF and package manifest must consume the same identity stamp. Packaging must not re-query live Git for firmware identity.
- Canonical firmware Cargo builds require the identity stamp; missing or invalid identity fails with a `just build` instruction rather than silently falling back to Git.

## ESP-IDF And Runtime Surfaces

- Generate an output-local supplemental sdkconfig defaults file containing `CONFIG_APP_PROJECT_VER_FROM_CONFIG=y` and `CONFIG_APP_PROJECT_VER="<build-label>"`; use an output-local generated sdkconfig so stale local configuration cannot override identity.
- Keep the existing heartbeat, PATCH, restart, and unrelated public contracts byte-for-byte unchanged.
- LCD fourth line is exactly `fw <build_label>`. The 22-character maximum label must fit the existing 25-character line without truncation.
- Keep retained `firmware_commit=<full-40-character-hash>` as the machine marker. Add full pinned-reference and application-descriptor ELF-SHA machine markers plus `runtime_build_identity semantic_version=<version> label=<label> channel=<release|dev> source_dirty=<true|false> release_tag=<tag|unavailable> redacted=true`.
- `/api/system/info.version` becomes the human build label. Add `semanticVersion`, `sourceCommit`, `referenceCommit`, `appElfSha256`, `buildChannel`, `sourceDirty`, and nullable `releaseTag`. The live WebSocket projection receives the same additive fields through the shared system-info wire model.
- The running ESP-IDF application descriptor's lowercase 64-character ELF SHA-256 is the non-circular flashed-package identifier. Machine evidence compares it with host inspection while separately comparing the final package digest; it must never parse the LCD/API label.

## Package And Admission Contract

- Bump the active package manifest to schema v3 while preserving top-level `source_commit` as the full bare hash.
- Add semantic version, pinned reference commit, application-descriptor ELF SHA-256, and structured `build_identity` with `label`, `channel`, `source_dirty`, and nullable `release_tag`; validate every relationship to the versioned provenance stamp and top-level `source_commit`.
- Preserve committed historical v2 evidence unchanged and readable only where historical evidence intentionally requires it.
- Update active release/admission gates, the concrete `tools/flash` flash/flash-monitor entrypoints, and fixtures for v3. Perform manifest/dirty admission before USB port resolution or credential reads, reject explicit-image hardware runs without an admitted v3 manifest, and accept clean dev and clean release packages.

## Required Plan 01 Tests

- Four label states and fixed suffix ordering.
- Exact, unrelated, multiple, detached, and missing tag/Git cases.
- Relevant staged, unstaged, and untracked dirty cases; ignored and planning/docs-only clean cases.
- Stable-status parsing and dirty-to-dirty Bazel invalidation.
- Identity stamp consistency and malformed-input rejection.
- All LCD variants fit without truncation.
- Additive system-info and live-WebSocket serialization.
- Retained full commit plus structured identity log.
- Manifest v3/full-hash/structured identity validation and dirty pre-hardware rejection.
- Packaged ELF/ESP application descriptor contains the expected label.
- Historical evidence and public heartbeat/PATCH/restart shapes remain unchanged.

## Verification And Delivery

- Run shell behavior tests, affected Bazel tests, `bazel test //...`, `just build`, `just package`, `just verify-reference`, `git diff --check`, and shellcheck/shfmt for changed shell scripts.
- Before each commit, run `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, and `cargo test --all-features` in that order.
- Commit Phase 34 Plan 01 implementation and tests atomically. Do not push.
- Do not use hardware in Phase 34 Plan 01. Phase 35 alone may perform the final detector-gated current-package run that jointly closes CFG-12 and EVD-13.

## Remaining Phase 34 Requirements

The planner must create later-wave plans covering OBS-06, SYS-03, SYS-04, SYS-05, HLT-01, HLT-02, HLT-03, and HLT-04 without expanding Plan 01. Those plans must preserve the roadmap prohibitions on active watchdog intervention, hardware self-test effects, load/fault experiments, actuation, mining, archived Phase 28.1.1 work, credentials, direct UART/pins, OTA, other boards, or broad promotion.

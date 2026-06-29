---
phase: 09-flash-monitor-evidence-wrapper-hardening
reviewed: 2026-06-29T15:26:23Z
depth: standard
files_reviewed: 11
files_reviewed_list:
  - tools/flash/src/main.rs
  - BUILD.bazel
  - firmware/bitaxe/BUILD.bazel
  - firmware/bitaxe/build.rs
  - scripts/BUILD.bazel
  - scripts/build-firmware.sh
  - scripts/source-commit-stamp.sh
  - docs/parity/evidence/phase-09-flash-monitor-evidence-wrapper-hardening.md
  - docs/parity/evidence/phase-09-flash-monitor-evidence-wrapper-hardening/flash-command-evidence.json
  - docs/parity/evidence/phase-09-flash-monitor-evidence-wrapper-hardening/flash-monitor.log
  - .planning/phases/09-flash-monitor-evidence-wrapper-hardening/09-02-SUMMARY.md
findings:
  critical: 0
  warning: 0
  info: 0
  total: 0
status: clean
---

# Phase 09: Code Review Report

**Reviewed:** 2026-06-29T15:26:23Z
**Depth:** standard
**Files Reviewed:** 11
**Status:** clean

## Summary

Reviewed the requested Phase 09 final source, Bazel, shell, evidence, log, and summary files at standard depth, focusing on the wrapper trust gate, source commit stamping and package invalidation, evidence consistency, and parity scope.

Material guidance applied: `AGENTS.md` repo-local Ultra 205 evidence rules, `AGENTS.bright-builds.md`, `standards-overrides.md`, `standards/index.md`, `standards/core/architecture.md`, `standards/core/code-shape.md`, `standards/core/testing.md`, `standards/core/verification.md`, and `standards/languages/rust.md`.

All reviewed files meet quality standards. No issues found.

## Review Notes

- Wrapper trust gate is fail-closed for the Phase 09 serial-evidence contract. The capture accepts only trusted output with `completed` or `timed_out_after_trusted_output`; missing markers, stale commits, truncated commit markers, prefixed marker variants, failed monitor exits, and timeout-without-trust paths are rejected after writing diagnostic JSON.
- Trusted output now requires exact safe-state text, reset and ESP-IDF provenance, token-bounded SPIFFS and route-shell markers, and observed firmware/reference commit markers that are 12+ hex characters and match the expected source/reference commits.
- Source commit stamping is branch-agnostic enough for the repo's local Bazel build mode: `//scripts:source_commit_stamp` is driven by `.git/HEAD` and `.git/logs/HEAD`, `//firmware/bitaxe:firmware` passes that stamp into `BITAXE_SOURCE_COMMIT`, and `//firmware/bitaxe:firmware_image` includes the same stamp as an input so the package action reruns when local `HEAD` moves.
- Evidence claims match the generated JSON and serial log. The JSON records `capture_status=timed_out_after_trusted_output`, `trusted_output=true`, `firmware_commit=0a25ceeadc2788e8b93c4067603e71d7c067d372`, `observed_firmware_commit=0a25ceeadc27`, and matching reference commit values; the log contains the required boot, safe-state, SPIFFS, route-shell, reset, commit, and ESP-IDF markers.
- Parity scope is not overstated. The evidence ledger and summary limit Phase 09 proof to wrapper-owned serial flash-monitor evidence, and the checklist/release docs keep HTTP, static file serving, recovery, OTA, rollback, interrupted update, mining, voltage, fan, thermal, and power behavior outside this verified scope.

## Verification Notes

- `cargo test -p bitaxe-flash` passed: 29 tests, 0 failures.
- `bazel test //tools/flash:tests` passed with the cached test target.
- `bazel build //scripts:source_commit_stamp` passed; the generated stamp value `0a25ceeadc27` matched `git rev-parse --short=12 HEAD`.
- `bazel query --output=build 'set(//scripts:source_commit_stamp //firmware/bitaxe:firmware //firmware/bitaxe:firmware_image)'` confirmed the stamp target is an input to both firmware and package image actions.
- `just parity` passed with `validation_errors: none`.

***

_Reviewed: 2026-06-29T15:26:23Z_
_Reviewer: the agent (gsd-code-reviewer)_
_Depth: standard_

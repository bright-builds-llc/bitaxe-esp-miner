---
quick_id: 260628-kri
description: Add ESP-IDF contributor dependency workflow
created: 2026-06-28
status: complete
---

# Quick Task 260628-kri: Add ESP-IDF Contributor Dependency Workflow

## Goal

Make ESP-IDF a first-class contributor dependency through the existing pinned `esp-idf-sys` workflow, with a read-only doctor command, an explicit bootstrap command, onboarding docs, and repo-local guidance favoring ESP-IDF/esp-rs tooling.

## Plan

1. Add read-only ESP dependency checks in `scripts/esp-doctor.sh`.
2. Add explicit opt-in installer logic in `scripts/bootstrap-esp.sh`.
3. Add shell tests with fake `PATH`/`HOME` fixtures.
4. Wire `just doctor`, `just bootstrap-esp`, and Bazel-visible script targets.
5. Update build/package failure messages, onboarding docs, and repo-local agent guidance.
6. Verify with script tests, doctor, Rust checks, Bazel tests, and diff hygiene.

## Notes

- Do not change firmware runtime behavior, manifest schema, flashing API, or CI.
- Do not revert, stage, or commit existing Phase 7 dirty worktree changes.

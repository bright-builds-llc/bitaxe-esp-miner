---
status: completed
created: 2026-07-04
quick_id: 260704-dxg
title: Add Fresh Monitor Log DEVICE_URL Repo Rule
---

# Plan

## Scope

- Add a narrow repo-local exception allowing `DEVICE_URL` extraction from fresh monitor logs for local testing.
- Preserve detector gating, redaction, no network scanning, and no stale-log inference.
- Do not modify Phase 21 historical evidence.

## Tasks

- [x] Add the `AGENTS.md` rule under Autonomous Ultra 205 Hardware Verification.
- [x] Record quick-task artifacts and state tracking.
- [x] Run diff, wording, and Rust pre-commit verification.

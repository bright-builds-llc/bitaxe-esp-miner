---
status: completed
created: 2026-06-28
quick_id: 260628-lgg
title: Add Autonomous Ultra 205 Hardware Verification Rule
---

# Plan

## Scope

- Add standing repo guidance allowing autonomous Ultra 205 hardware verification when a single ESP32-S3 USB candidate is detected.
- Add a read-only detector command for agents and humans to find the eligible port.
- Update current Phase 07 hardware checkpoint language to use standing permission and auto-detection.

## Tasks

- [x] Add `scripts/detect-ultra205.sh` and `just detect-ultra205`.
- [x] Wire detector into Bazel scripts package and add shell tests for zero, multiple, success, and board-info-failure cases.
- [x] Add repo-local AGENTS guidance for autonomous Ultra 205 hardware use, evidence requirements, and phase-gated destructive/fault-injection limits.
- [x] Update Phase 07 checkpoint and threat model text from “ask for port/approval” to “auto-detect or record pending hardware evidence.”
- [x] Run focused verification and record results.

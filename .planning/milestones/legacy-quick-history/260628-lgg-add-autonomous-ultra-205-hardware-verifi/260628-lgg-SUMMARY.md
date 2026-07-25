---
status: completed
created: 2026-06-28
quick_id: 260628-lgg
title: Add Autonomous Ultra 205 Hardware Verification Rule
---

# Summary

Added a repo-local standing rule and read-only detector for autonomous Ultra 205 hardware verification.

## Changes

- Added `just detect-ultra205`, backed by `scripts/detect-ultra205.sh`.
- The detector runs `espflash list-ports --name-only`, requires exactly one likely ESP USB serial port, runs `espflash board-info --chip esp32s3 --port <port> --non-interactive`, and prints `port=<path>`.
- Added `scripts/detect-ultra205-test.sh` and Bazel `//scripts:detect_ultra205_test`.
- Added AGENTS guidance allowing autonomous Ultra 205 hardware verification after detector success, with strict stop conditions and evidence requirements.
- Updated Phase 07 Plan 09 hardware checkpoint and threat model text to use `just detect-ultra205` and standing permission.

## Verification

- `bash scripts/detect-ultra205-test.sh` passed.
- Documentation scan passed: `rg -n "Autonomous Ultra 205|detect-ultra205|board-info|phase-gated" AGENTS.md .planning/phases/07-ota-filesystem-and-release-packaging`.
- Real read-only detection passed and selected `port=/dev/cu.usbmodem1101`.
- `bazel test //scripts:detect_ultra205_test` passed.
- `just test` passed.
- `git diff --check` passed.

## Residual Risk

- The detector confirms a single ESP32-S3 USB candidate, not a physical enclosure label. Hardware evidence must still confirm board `205` through repo commands and captured firmware logs.

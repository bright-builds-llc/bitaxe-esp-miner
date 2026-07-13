---
phase: 15-bm1366-mining-evidence-completion
reviewed: 2026-07-01T05:33:51Z
depth: standard
files_reviewed: 33
files_reviewed_list:
  - crates/bitaxe-asic/src/bm1366/adapter_gate.rs
  - docs/parity/checklist.md
  - docs/parity/evidence/phase-15-bm1366-mining-evidence-completion.md
  - docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/README.md
  - docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/bounded-soak.md
  - docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/bounded-soak/allow-bounded-soak.json
  - docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/bounded-soak/bounded-soak.log
  - docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/bounded-soak/detect-ultra205.log
  - docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/chip-detect.md
  - docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/chip-detect/allow-chip-detect.json
  - docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/chip-detect/flash-command-evidence.json
  - docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/mining-smoke.md
  - docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/mining-smoke/allow-mining-smoke.json
  - docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/mining-smoke/detect-ultra205.log
  - docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/mining-smoke/mining-smoke.log
  - docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/redaction-review.md
  - docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/work-result.md
  - docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/work-result/allow-work-result.json
  - docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/work-result/flash-command-evidence.json
  - docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/work-result/flash-monitor.log
  - docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/work-result/package/bitaxe-ultra205-package.json
  - firmware/bitaxe/src/asic_adapter.rs
  - firmware/bitaxe/src/asic_adapter/status.rs
  - scripts/BUILD.bazel
  - scripts/phase15-bm1366-diagnostic-package-test.sh
  - scripts/phase15-bm1366-diagnostic-package.sh
  - scripts/phase15-controlled-mining-test.sh
  - scripts/phase15-controlled-mining.sh
  - scripts/phase15-websocket-capture.mjs
  - tools/flash/src/main.rs
  - tools/parity/BUILD.bazel
  - tools/parity/src/main.rs
  - tools/parity/src/mining_allow.rs
findings:
  critical: 0
  warning: 0
  info: 0
  total: 0
status: clean
---

# Phase 15: Code Review Report

**Reviewed:** 2026-07-01T05:33:51Z
**Depth:** standard
**Files Reviewed:** 33
**Status:** clean

## Summary

Reviewed the listed Phase 15 Rust firmware gates, shell wrappers, parity tooling, checklist updates, and checked-in evidence artifacts at standard depth. The review was informed by `AGENTS.md`, `AGENTS.bright-builds.md`, `standards-overrides.md`, and the loaded Bright Builds architecture, code-shape, verification, testing, and Rust standards.

The prior WebSocket helper warning is resolved. `scripts/phase15-websocket-capture.mjs` now initializes the device URL from CLI parsing only, does not read `process.env.DEVICE_URL`, and fails when `--device-url` is absent. `scripts/phase15-controlled-mining.sh` also keeps the live attempt gated on an explicit `--device-url` argument, so env-only `DEVICE_URL` no longer authorizes helper execution or live telemetry probes.

All reviewed files meet quality standards. No issues found.

## Review Notes

- No critical secret leak, direct unsafe raw hardware command, or evidence-promotion regression was found.
- No project-local skills were present under `.claude/skills/` or `.agents/skills/`.
- None of the sampled reviewed source files were ignored by git.
- Direct env-only helper probe failed as expected: `DEVICE_URL=... node scripts/phase15-websocket-capture.mjs --out /tmp/phase15-ws-env-only-review.log` exited with `DEVICE_URL is required`.
- Explicit helper probe wrote only redacted output: `websocket_capture_url=wss://[redacted]/api/ws/live` and `websocket_frame_status=pending - max frames zero`.
- Targeted verification passed: `bazel test //tools/parity:tests //scripts:phase15_controlled_mining_test //scripts:phase15_bm1366_diagnostic_package_test`.
- Parity validation passed: `just parity` completed with `validation_errors: none`.

_Reviewed: 2026-07-01T05:33:51Z_
_Reviewer: the agent (gsd-code-reviewer)_
_Depth: standard_

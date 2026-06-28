---
phase: 08
slug: parity-evidence-and-ultra-205-release-gate
status: draft
nyquist_compliant: true
wave_0_complete: false
created: 2026-06-28
---

# Phase 08 - Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Bazel `rust_test` plus focused Cargo tests for Rust crates and tools. |
| **Config file** | `tools/parity/BUILD.bazel`, root `Cargo.toml`, and package `Cargo.toml` files. |
| **Quick run command** | `bazel test //tools/parity:tests` |
| **Full suite command** | `just test && just parity && bazel run //tools/parity:report -- release-gate` |
| **Estimated runtime** | ~60-240 seconds for focused checks; full Rust pre-commit gate is longer. |

## Sampling Rate

- **After every task commit:** Run the narrow affected command, usually `bazel test //tools/parity:tests` for release/checklist guard logic or `just parity` for checklist/evidence-only changes.
- **After every plan wave:** Run `just parity`, `bazel run //tools/parity:report -- release-gate`, and affected crate tests.
- **Before `/gsd-verify-work`:** Run `just test`, `just package`, `just parity`, `bazel run //tools/parity:report -- release-gate`, and the Rust pre-commit gate from `AGENTS.md`.
- **Max feedback latency:** 1 plan task for automated checks; hardware evidence is gated by documented recovery procedures and may be manual/long-running.

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 08-01-01 | 01 | 1 | EVD-01, EVD-02, EVD-03, EVD-05 | T-08-01 | Invalid `verified` parity claims fail before release. | unit/workflow | `bazel test //tools/parity:tests && just parity` | yes | pending |
| 08-01-02 | 01 | 1 | EVD-04 | T-08-02 | Reference breadcrumbs exist at module/behavior boundaries without copying upstream expression. | audit/workflow | `rg -n "Reference breadcrumb|Reference breadcrumbs|reference/esp-miner" crates firmware tools` | yes | pending |
| 08-02-01 | 02 | 1 | REL-08, EVD-05 | T-08-03 | Hardware evidence starts only after `just detect-ultra205` succeeds for exactly one Ultra 205 ESP USB candidate. | workflow/hardware-smoke | `just detect-ultra205` | yes | pending |
| 08-02-02 | 02 | 1 | REL-08 | T-08-04 | Destructive or fault-injection evidence is not run without a documented recovery path and current package manifest. | docs/workflow | `rg -n "large erase|interrupted|recovery path|bitaxe-ultra205-factory.bin" docs/parity/evidence docs/release` | yes | pending |
| 08-03-01 | 03 | 2 | REL-08, EVD-05 | T-08-05 | Live HTTP/OTA/recovery rows are promoted only when evidence records device URL, commands, response status/body, board, port, source commit, reference commit, and package manifest. | hardware-smoke/workflow | `just parity && bazel run //tools/parity:report -- release-gate` | yes | pending |
| 08-04-01 | 04 | 2 | REL-08, EVD-01, EVD-02, EVD-03, EVD-04, EVD-05 | T-08-06 | Final release summary cites concrete evidence and keeps explicit gaps visible. | docs/workflow | `just test && just package && just parity && bazel run //tools/parity:report -- release-gate` | yes | pending |

*Status: pending, green, red, flaky*

## Wave 0 Requirements

- [ ] Add or confirm focused `tools/parity` tests for any new Phase 8 release-readiness guard before checklist promotion.
- [ ] Create `docs/parity/evidence/phase-08-ultra-205-release-gate.md` before live hardware actions so every run has a destination and expected evidence fields.
- [ ] Establish a `DEVICE_URL` discovery procedure or record the blocker before HTTP/OTA probes.

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Live HTTP/static/recovery smoke | REL-08, EVD-05 | Requires a reachable flashed Ultra 205 over the local network. | Run `just detect-ultra205`, `just package`, `just flash-monitor board=205 port=<port> evidence-dir=<path>`, establish `DEVICE_URL`, then capture `/`, `/assets/app.css.gz`, missing static redirect, `/recovery`, and API coexistence responses. |
| Valid firmware OTA and invalid OTA rejection | REL-08, EVD-05 | Requires device reboot and post-update identity/boot-validation evidence. | Use packaged `esp-miner.bin`, record checksum, upload to `/api/system/OTA`, capture public response, reboot logs, running partition, boot-validation status, and invalid image rejection. |
| Large erase and interrupted update recovery | REL-08 | Destructive/fault-injection behavior can affect a physical board. | Run only after a plan documents the recovery path, current package manifest, factory image checksum, exact commands, expected interruption point, post-recovery observations, and stop criteria. |

## Validation Sign-Off

- [x] All tasks have automated verification or a manual hardware gate.
- [x] Sampling continuity: no 3 consecutive tasks without automated verification.
- [x] Wave 0 covers missing evidence destination, guard tests, and `DEVICE_URL` procedure.
- [x] No watch-mode flags.
- [x] `nyquist_compliant: true` set in frontmatter.

**Approval:** approved 2026-06-28

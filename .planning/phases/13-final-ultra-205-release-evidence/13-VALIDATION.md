---
phase: 13
slug: final-ultra-205-release-evidence
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-06-30
---

# Phase 13 - Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

## Test Infrastructure

| Property | Value |
| --- | --- |
| **Framework** | Rust unit tests through Cargo and Bazel, repo shell scripts, release-gate validation, and detector-gated hardware/network evidence |
| **Config file** | `Cargo.toml`, `MODULE.bazel`, `Justfile`, `tools/parity/BUILD.bazel`, `tools/flash/BUILD.bazel` |
| **Quick run command** | `just parity` |
| **Full suite command** | `cargo fmt --all && cargo clippy --all-targets --all-features -- -D warnings && cargo build --all-targets --all-features && cargo test --all-features && just package && just parity` |
| **Estimated runtime** | Host checks take minutes; live hardware/network/destructive evidence depends on connected board and recovery prerequisites |

## Sampling Rate

- **After every task commit:** Run the task's targeted command, at minimum `just parity` for checklist/evidence changes or targeted Cargo/script tests for code changes.
- **After every plan wave:** Run `just parity` plus targeted host checks named by the completed plans.
- **Before `/gsd-verify-work`:** Full suite, manifest-backed release gate, and recorded hardware/network evidence status must be green or explicitly pending without overclaims.
- **Max feedback latency:** Keep host feedback under 5 minutes where possible; hardware/destructive checks may exceed this only when the plan records the expected duration and stop conditions.

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 13-01-01 | 01 | 1 | REL-04/EVD-05 | T-13-01 | Release artifacts must tie source/reference commits to manifest checksums. | package/release-gate | `just package && bazel run //tools/parity:report -- release-gate --manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json` | W0 | pending |
| 13-02-01 | 02 | 2 | FND-06/EVD-05 | T-13-02 | Hardware evidence must target exactly one detector-approved Ultra 205. | hardware-smoke | `just detect-ultra205` then `just flash-monitor board=205 port=<path> evidence-dir=docs/parity/evidence/phase-13-final-ultra-205-release-evidence` | W0 | pending |
| 13-03-01 | 03 | 3 | API-09/REL-01 | T-13-03 | Live HTTP evidence must use a reachable just-flashed device URL and no private data leakage. | live-http | plan-defined `DEVICE_URL` probe command | W0 | pending |
| 13-05-01 | 05 | 4 | REL-08 | T-13-05 | Recovery, failed-update, and destructive evidence must have current factory image recovery path, stop conditions, and explicit failed-update status before valid OTA. | hardware-regression | plan-defined recovery commands only | W0 | pending |
| 13-04-01 | 04 | 5 | REL-02/REL-08 | T-13-04 | OTA evidence must not run valid upload until recovery runbook and failed-update/recovery status exist, and must not treat upload success or invalid rejection as rollback proof without boot-validation logs. | live-ota | plan-defined OTA probe plus monitor evidence | W0 | pending |
| 13-06-01 | 06 | 6 | REL-03/EVD-05 | T-13-06 | OTAWWW remains a documented REL-03 gap unless interrupted-update hardware-regression exists. | parity/docs | `just parity` | W0 | pending |

*Status: pending, green, red, or flaky.*

## Wave 0 Requirements

- [ ] Confirm whether a repo-owned HTTP/OTA evidence helper is needed or whether existing plan commands can capture status codes, headers, response snippets, checksums, and conclusions.
- [ ] Define Phase 13 evidence directory and redaction review path.
- [ ] Confirm `DEVICE_URL` input mechanism before any live HTTP/OTA probe.
- [ ] Confirm destructive recovery runbooks name current factory image, stop conditions, failed-update evidence requirements, and recovery commands before any erase, rollback, valid OTA, failed-update, or interrupted-update action.

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
| --- | --- | --- | --- |
| Ultra 205 detector gate | FND-06/EVD-05 | Requires physical USB board. | Run `just detect-ultra205`; continue only with exactly one `port=<path>` and successful board-info. |
| Live HTTP/static/recovery smoke | API-09/REL-01 | Requires reachable device network endpoint. | Provide `DEVICE_URL`; probe `/`, `/assets/app.css.gz`, missing static path, `/recovery`, and route coexistence. |
| Firmware OTA success/rejection/boot validation | REL-02/REL-08 | Requires live upload and reboot observation. | Use manifest `esp-miner.bin`, capture valid and invalid upload behavior, monitor reboot, and record boot-validation/partition state. |
| Large erase, failed update, interrupted update, rollback | REL-08 | Destructive/fault-injection behavior. | Run only plan-approved commands with documented factory-image recovery path and stop conditions. |
| Redaction review | EVD-05 | Generated evidence can include private URLs or secrets. | Inspect all generated logs, JSON, and Markdown before commit and record the result. |

## Validation Sign-Off

- [ ] All tasks have automated or manual evidence commands.
- [ ] Sampling continuity: no three consecutive tasks without an automated host check or explicit hardware/manual evidence checkpoint.
- [ ] Wave 0 covers all missing evidence-helper and `DEVICE_URL` prerequisites.
- [ ] No watch-mode flags.
- [ ] Feedback latency is bounded or explicitly documented for hardware/destructive checks.
- [ ] Set `nyquist_compliant: true` after plans instantiate task IDs and verification commands.

**Approval:** pending

---
phase: 09
slug: flash-monitor-evidence-wrapper-hardening
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-06-29
---

# Phase 09 - Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

## Test Infrastructure

| Property | Value |
| --- | --- |
| **Framework** | Bazel `rust_test` target for `tools/flash`; Rust unit tests live in `tools/flash/src/main.rs`. |
| **Config file** | `tools/flash/BUILD.bazel`; package deps in `tools/flash/Cargo.toml`. |
| **Quick run command** | `bazel test //tools/flash:tests` |
| **Full suite command** | `just test`; before commit also run `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, and `cargo test --all-features`. |
| **Estimated runtime** | Quick: under 30 seconds after warm cache; full suite varies with Bazel/Cargo cache state. |

## Sampling Rate

- **After every task commit:** Run `bazel test //tools/flash:tests` for wrapper code changes.
- **After every documentation/evidence task:** Review the diff and run `just parity` when checklist or evidence semantics change.
- **After every plan wave:** Run `just test` when the wave changes Rust wrapper behavior or evidence semantics.
- **Before phase verification:** Run the Rust pre-commit sequence required by `AGENTS.md`, `just test`, `just parity`, and hardware evidence when `just detect-ultra205` succeeds.
- **Max feedback latency:** Keep quick wrapper feedback under 30 seconds after warm cache; full-suite latency is acceptable only at wave/phase boundaries.

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 09-01-01 | 01 | 1 | FND-08 | T-09-01 | Evidence-mode monitor command uses `espflash monitor --chip esp32s3 --port <port> --non-interactive` without changing ordinary monitor behavior. | unit | `bazel test //tools/flash:tests` | Yes | pending |
| 09-01-02 | 01 | 1 | EVD-05 | T-09-02 | Capture result records trusted status and fails closed on nonzero exit, timeout without trusted output, or log write failure. | unit | `bazel test //tools/flash:tests` | Yes | pending |
| 09-02-01 | 02 | 1 | EVD-05 | T-09-03 | `flash-command-evidence.json` records board, port, commits, manifest, exact commands, log path, capture mode/status, and conclusion. | unit | `bazel test //tools/flash:tests` | Yes | pending |
| 09-02-02 | 02 | 1 | REL-07 | T-09-04 | Failure guidance says evidence is not trusted and routes operators to `just detect-ultra205`, wrapper capture, and diagnostic `just monitor`. | unit/docs | `bazel test //tools/flash:tests` | Yes | pending |
| 09-03-01 | 03 | 2 | FND-07/EVD-05 | T-09-05 | Fresh Ultra 205 wrapper evidence is captured only after detector success, and docs/checklist do not overclaim HTTP/OTA/recovery parity. | hardware/docs | `just detect-ultra205`; `just flash-monitor board=205 port=<port> evidence-dir=<path>`; `just parity` | Yes | pending |

*Status: pending, green, red, flaky.*

## Wave 0 Requirements

Existing infrastructure covers all phase requirements. Add focused tests in `tools/flash/src/main.rs` before changing behavior:

- [ ] Evidence monitor command construction.
- [ ] Bounded capture statuses: success, timed out with trusted output, timed out without trusted output, nonzero exit, and log creation failure.
- [ ] Enriched `flash-command-evidence.json` fields.
- [ ] Failure guidance assertions for untrusted evidence and repo-owned recovery commands.

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
| --- | --- | --- | --- |
| Fresh Ultra 205 wrapper hardware evidence | EVD-05 | Requires connected board `205` and serial port access. | Run `just detect-ultra205`; continue only with exactly one detected port and successful board-info. Then run `just flash-monitor board=205 port=<port> evidence-dir=docs/parity/evidence/phase-09-flash-monitor-evidence-wrapper-hardening` and record commands, log, evidence JSON, observed boot lines, and conclusion. |

## Validation Sign-Off

- [ ] All tasks have automated verify or documented hardware/manual verification.
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify.
- [ ] Existing test infrastructure covers all missing references.
- [ ] No watch-mode flags.
- [ ] Feedback latency for quick checks is under 30 seconds after warm cache.
- [ ] `nyquist_compliant: true` set in frontmatter after execution evidence confirms the contract.

**Approval:** pending

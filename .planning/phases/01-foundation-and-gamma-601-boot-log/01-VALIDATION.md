---
phase: 01
slug: foundation-and-gamma-601-boot-log
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-06-21
---

# Phase 01 - Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | None exists yet. Phase 1 Wave 0 must add Cargo, Bazel, Just, Rust crates, firmware, scripts, and host-tool test targets. |
| **Config file** | None yet. Expected additions include `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `.cargo/config.toml`, `MODULE.bazel`, `.bazelversion`, `BUILD.bazel` files, `Justfile`, and tool/firmware configs. |
| **Quick run command** | `just verify-reference && just build && just test && just package && just parity` |
| **Full suite command** | Quick command plus `just flash-monitor board=601 port=<port>` when Gamma 601 hardware is connected. |
| **Estimated runtime** | Unknown until Phase 1 creates the toolchain and first firmware target. |

---

## Sampling Rate

- **After every task commit:** Run the narrow Bazel/Cargo/script check for the changed crate, tool, firmware target, or script.
- **After every plan wave:** Run `just verify-reference`, `just build`, `just test`, `just package`, and `just parity`.
- **Before `/gsd-verify-work`:** Full suite must be green where hardware is available; otherwise hardware smoke remains explicit missing evidence.
- **Max feedback latency:** Keep pure crate/tool checks under 60 seconds once Wave 0 infrastructure exists.

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 01-W0-reference | TBD | 0 | FND-01, FND-02 | T-01-reference | Reference tree is pinned, clean, read-only, and reported by commit. | workflow | `git submodule status --recursive && just verify-reference` | No - Wave 0 | pending |
| 01-W0-bazel | TBD | 0 | FND-03 | T-01-graph | Build/test/package/flash/parity flows are reachable through the Bazel graph. | workflow | `bazel query //... && just build && just test` | No - Wave 0 | pending |
| 01-W0-firmware-metadata | TBD | 0 | FND-04 | T-01-toolchain | ESP-IDF, Rust target, toolchain, and dependency versions are pinned. | build | `cargo metadata --format-version=1 && bazel build //firmware/bitaxe:firmware` | No - Wave 0 | pending |
| 01-W0-crates | TBD | 0 | FND-05 | T-01-boundary | Pure crates exist and stay independent of ESP-IDF/hardware side effects. | build + unit | `bazel test //crates/...` | No - Wave 0 | pending |
| 01-W0-boot-log | TBD | 0 | FND-06 | T-01-safe-mode | Firmware logs safe no-mining/no-control state before hardware effects exist. | hardware-smoke | `just flash-monitor board=601 port=<port>` | No - Wave 0 and hardware required | pending |
| 01-W0-just | TBD | 0 | FND-07 | T-01-command-surface | Human commands delegate to Bazel-visible targets or repo-owned scripts. | workflow | `just --list && just build && just test && just package && just parity` | No - Wave 0 | pending |
| 01-W0-flash | TBD | 0 | FND-08 | T-01-shell-injection | Port/image arguments are parsed as typed args and passed without shell concatenation. | unit + integration | `bazel test //tools/flash:tests` | No - Wave 0 | pending |
| 01-W0-package | TBD | 0 | FND-09 | T-01-provenance | Package manifest records paths, offsets, checksums, tool versions, firmware commit, and reference commit. | unit + workflow | `just package` | No - Wave 0 | pending |
| 01-W0-provenance | TBD | 0 | FND-10 | T-01-license | GPL-derived behavior and original MIT-first work remain explicit. | workflow | `just parity` | No - Wave 0 | pending |
| 01-W0-parity | TBD | 0 | FND-11 | T-01-false-parity | Parity report exposes statuses, gaps, implementation pointers, and reference breadcrumbs without treating implementation as verification. | unit + workflow | `bazel test //tools/parity:tests && just parity` | No - Wave 0 | pending |

*Status values: pending, green, red, flaky.*

---

## Wave 0 Requirements

- [ ] `reference/esp-miner` submodule at accepted upstream commit `c1915b0a63bfabebdb95a515cedfee05146c1d50`.
- [ ] `scripts/verify-reference-clean.sh` plus Bazel/Just wiring and tests.
- [ ] `MODULE.bazel`, `.bazelversion`, root `BUILD.bazel`, and package BUILD files.
- [ ] Root `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, and `.cargo/config.toml`.
- [ ] `firmware/bitaxe` ESP-IDF Rust app skeleton with safe boot/log behavior.
- [ ] Planned pure crates under `crates/`.
- [ ] `Justfile` with required Phase 1 commands.
- [ ] `tools/flash` CLI and tests.
- [ ] Package manifest generation and tests.
- [ ] `tools/parity` CLI and tests.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Gamma 601 flash-monitor smoke | FND-06, FND-08 | Requires connected Gamma 601 hardware and serial port. | Run `just flash-monitor board=601 port=<port>` and capture command, board, port, firmware commit, reference commit, timestamp, log path, observed boot/status output, and pass/fail conclusion. |
| Reference submodule pointer review | FND-01, FND-02, FND-10 | Human review confirms the pointer update is intentional and no files inside `reference/esp-miner` were modified. | Review `git diff --submodule` and `git status --short reference/esp-miner`; confirm only the parent submodule pointer changed. |

---

## Threat Model Inputs

| Threat | Severity | Required Planning Mitigation |
|--------|----------|------------------------------|
| T-01-reference: dirty or substituted reference tree | high | Verify submodule status, fail on dirty/missing reference, and print pinned commit. |
| T-01-shell-injection: untrusted `port` or image path reaches shell command strings | high | Use typed CLI parsing and `Command` argument vectors; do not assemble shell strings from user input. |
| T-01-provenance: package output lacks source/reference/tool identity | medium | Include firmware commit, reference commit, tool versions, offsets, paths, and checksums in the package manifest. |
| T-01-false-parity: implementation is treated as verification | high | `just parity` must flag pending evidence and safety-critical rows without hardware evidence. |
| T-01-safe-mode: boot logs imply mining or hardware control before gates exist | high | Firmware must log explicit safe no-mining/no-control state. |

---

## Validation Sign-Off

- [ ] All tasks have automated verify commands or Wave 0 dependencies.
- [ ] Sampling continuity: no three consecutive tasks without automated verify.
- [ ] Wave 0 covers all missing test/build infrastructure.
- [ ] No watch-mode flags in verification commands.
- [ ] Feedback latency is measurable after Wave 0 exists.
- [ ] `nyquist_compliant: true` is set only after Wave 0 infrastructure exists and plan tasks map all FND requirements to executable checks.

**Approval:** pending

---
phase: 16
slug: current-commit-release-evidence-completion
status: passed
nyquist_compliant: true
wave_0_complete: true
created: 2026-07-01
---

# Phase 16 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

***

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Bazel `sh_test` for shell helpers; Cargo/Rust unit tests for host tools; `just test` aggregates `bazel test //...`. |
| **Config file** | `MODULE.bazel`, `BUILD.bazel`, per-crate `BUILD.bazel`, and `Cargo.toml`. |
| **Quick run command** | `bazel test //scripts:phase16_http_static_smoke_test //scripts:phase16_recovery_regression_test` and `cargo test -p bitaxe-parity --all-features release_evidence`. |
| **Full suite command** | `cargo fmt --all && cargo clippy --all-targets --all-features -- -D warnings && cargo build --all-targets --all-features && cargo test --all-features && just test` |
| **Estimated runtime** | Quick suite ~60 seconds; full suite depends on firmware/package rebuild state. |

***

## Sampling Rate

- **After every task commit:** Run the targeted command named in that task's `<automated>` verification.
- **After every plan wave:** Run the relevant aggregate for touched surfaces; Rust/helper changes should run the full suite command above.
- **Before phase verification:** Run `just package`, manifest-backed release gate, applicable hardware/network/recovery commands, redaction review, `just parity`, `just verify-reference`, reference diff check, and lifecycle validation.
- **Max feedback latency:** No three consecutive implementation tasks may lack an automated check or a recorded hardware/manual blocker.

***

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 16-W0-01 | W0 | 0 | REL-04, EVD-05 | T-16-identity | Current git HEAD, package manifest `source_commit`, flash-monitor observed firmware commit, and evidence artifact paths are compared before docs/checklist promotion. | workflow/unit | `cargo test -p bitaxe-parity --all-features release_evidence` and `bazel test //tools/parity:tests --test_filter=release_evidence`. | yes | green |
| 16-W0-02 | W0 | 0 | API-09, REL-01, REL-02, REL-03 | T-16-url | HTTP evidence path blocks on missing explicit `DEVICE_URL`, sanitizes output, and probes `/api/system/OTA` plus existing static/recovery/OTAWWW routes. | shell unit | `bash -n scripts/phase16-http-static-smoke.sh` and `bazel test //scripts:phase16_http_static_smoke_test`. | yes | green |
| 16-W0-03 | W0 | 0 | REL-08, EVD-05 | T-16-recovery | Failed-update, interrupted-update, erase, rollback, and recovery operations all require detector, board-info, current manifest, current factory image, allow flags, aborts, and recovery steps before live action. | shell unit / typed allow | `bash -n scripts/phase16-recovery-regression.sh` and `bazel test //scripts:phase16_recovery_regression_test`. | yes | green |
| 16-W0-04 | W0 | 0 | REL-07, EVD-05 | T-16-redaction | Redaction review template covers API bodies, WebSocket frames, recovery logs, destructive logs, terminal snippets, private `DEVICE_URL`, pool/Wi-Fi credentials, tokens, NVS secrets, and absent artifacts. | document check | Artifact-by-artifact review in `docs/parity/evidence/phase-16-current-commit-release-evidence-completion/redaction-review.md`. | yes | green |
| 16-PKG-01 | package | 1 | REL-04, REL-07, EVD-05 | T-16-identity | `just package` and release gate pass for release-candidate commit before flash or live evidence. | workflow | `just package` and `bazel run //tools/parity:report -- release-gate --manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json`. | yes | green |
| 16-SER-01 | serial | 1 | FND-06, REL-01, EVD-05 | T-16-wrong-device | Detector finds exactly one Ultra 205 port and `flash-monitor` records trusted release-candidate serial boot evidence. | hardware-smoke | `just detect-ultra205` then `just flash-monitor board=205 port=/dev/cu.usbmodem1101 manifest=bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json evidence-dir=docs/parity/evidence/phase-16-current-commit-release-evidence-completion/serial-boot capture-timeout-seconds=35`. | yes | green |
| 16-HTTP-01 | live-http | 2 | API-09, REL-01, REL-02, REL-03, EVD-05 | T-16-url | Explicit `DEVICE_URL` probes capture live static, recovery, API coexistence, WebSocket no-upgrade, OTA route, and OTAWWW gap behavior or record blocked evidence. | hardware/network smoke | `scripts/phase16-http-static-smoke.sh --manifest docs/parity/evidence/phase-16-current-commit-release-evidence-completion/package-release-gate/bitaxe-ultra205-package.json --out-dir docs/parity/evidence/phase-16-current-commit-release-evidence-completion/http-static-recovery` recorded `DEVICE_URL status: blocked - missing DEVICE_URL`; no network scan ran. | hardware-gated | blocked |
| 16-OTA-01 | live-ota | 2 | REL-02, REL-08, EVD-05 | T-16-ota | Valid firmware OTA, invalid rejection, post-reboot identity, and boot-validation/rollback state are captured or blocked with explicit reason. | hardware/network smoke | `scripts/phase13-firmware-ota-smoke.sh --manifest docs/parity/evidence/phase-16-current-commit-release-evidence-completion/package-release-gate/bitaxe-ultra205-package.json --ota-image bazel-bin/firmware/bitaxe/esp-miner.bin --port /dev/cu.usbmodem1101 --out-dir docs/parity/evidence/phase-16-current-commit-release-evidence-completion/firmware-ota --monitor-seconds 45` blocked before upload because `DEVICE_URL` was missing. | hardware-gated | blocked |
| 16-REC-01 | recovery | 3 | REL-08, EVD-05 | T-16-recovery | Rollback, large erase, failed-update, and interrupted-update run only behind documented gates and restore safe state, or remain pending. | hardware-regression | `scripts/phase16-recovery-regression.sh --manifest docs/parity/evidence/phase-16-current-commit-release-evidence-completion/package-release-gate/bitaxe-ultra205-package.json --factory-image bazel-bin/firmware/bitaxe/bitaxe-ultra205-factory.bin --ota-image bazel-bin/firmware/bitaxe/esp-miner.bin --port /dev/cu.usbmodem1101 --out-dir docs/parity/evidence/phase-16-current-commit-release-evidence-completion/recovery-regression` recorded pending evidence because allow flags were omitted. | hardware-gated | skipped with reason |
| 16-DOC-01 | docs | 4 | REL-07, EVD-05 | T-16-overclaim | Release docs, parity checklist, requirements traceability, and milestone audit cite Phase 16 artifacts without promoting unsupported claims. | docs/parity | `just parity && just verify-reference && git diff -- reference/esp-miner --exit-code`. | yes | green |
| 16-FINAL-01 | final | 4 | FND-06, API-09, REL-01, REL-02, REL-03, REL-04, REL-07, REL-08, EVD-05 | T-16-final | Final verification status is `passed` only after package, release gate, redaction, parity, reference, relevant hardware/network/recovery checks, release-evidence validation, and lifecycle validation pass. | aggregate | `node "$HOME/.codex/get-shit-done/bin/gsd-tools.cjs" verify lifecycle 16 --expect-id 16-2026-07-01T12-36-46 --expect-mode yolo --require-plans --require-verification --raw`. | yes | green |

*Status: pending, green, red, blocked, or skipped with reason.*

***

## Wave 0 Requirements

- [x] Current-commit identity gate compares git HEAD, package manifest source commit, flash-monitor observed commit, and evidence paths before promotion; final closure may use the explicit post-source evidence-commit allowance only when all later changes are allowlisted evidence/docs/GSD paths.
- [x] HTTP/static evidence path covers `/api/system/OTA` in addition to existing `/`, `/assets/app.css.gz`, missing static, `/recovery`, API/WebSocket coexistence, and OTAWWW probes.
- [x] Recovery/destructive helper gate applies detector, board-info, current manifest, current factory image, allow flags, aborts, recovery steps, and safe-state checks before every failed-update, interrupted-update, rollback, and erase action.
- [x] Redaction review template exists before live artifacts are cited.

***

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Ultra 205 USB detector and board-info | FND-06, REL-08, EVD-05 | Requires connected physical Ultra 205. | Run `just detect-ultra205`; continue only if exactly one likely ESP USB serial port is found and board-info succeeds. |
| Live `DEVICE_URL` HTTP/static/recovery/OTA probes | API-09, REL-01, REL-02, REL-03, EVD-05 | Requires reachable just-flashed device URL and redaction-aware network evidence. | Provide explicit `DEVICE_URL`; run phase helper; otherwise record blocked evidence and do not promote live rows. |
| Firmware OTA, rollback, large erase, failed-update, interrupted-update | REL-02, REL-08, EVD-05 | Mutates connected hardware and may require recovery. | Run only through phase-documented allow gates, recovery steps, and safe-state checks; otherwise record pending evidence. |
| Redaction review of live logs and responses | REL-07, EVD-05 | Requires human review of generated evidence for secrets and private endpoints. | Review every cited artifact; record pass/fail and absent artifact boundaries before docs/checklist promotion. |

***

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or hardware/manual blocker handling.
- [x] Sampling continuity: no 3 consecutive tasks without automated verify or recorded blocker.
- [x] Wave 0 covers all missing references.
- [x] No watch-mode flags.
- [x] Feedback latency is bounded by targeted helper tests after each changed surface.
- [x] `nyquist_compliant: true` set in frontmatter after Wave 0 is implemented and mapped to concrete plan tasks.

**Approval:** passed

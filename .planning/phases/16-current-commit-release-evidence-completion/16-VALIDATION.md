---
phase: 16
slug: current-commit-release-evidence-completion
status: draft
nyquist_compliant: false
wave_0_complete: false
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
| **Quick run command** | `bazel test //scripts:phase13_http_static_smoke_test //scripts:phase13_firmware_ota_smoke_test //scripts:phase13_recovery_regression_test` |
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
| 16-W0-01 | W0 | 0 | REL-04, EVD-05 | T-16-identity | Current git HEAD, package manifest `source_commit`, flash-monitor observed firmware commit, and evidence artifact paths are compared before docs/checklist promotion. | workflow/unit | Planner must assign a concrete command after choosing shell or Rust implementation. | no W0 | pending |
| 16-W0-02 | W0 | 0 | API-09, REL-01, REL-02, REL-03 | T-16-url | HTTP evidence path blocks on missing explicit `DEVICE_URL`, sanitizes output, and probes `/api/system/OTA` plus existing static/recovery/OTAWWW routes. | shell unit | `bazel test //scripts:phase13_http_static_smoke_test` plus any new Phase 16 HTTP helper test. | partial W0 | pending |
| 16-W0-03 | W0 | 0 | REL-08, EVD-05 | T-16-recovery | Failed-update, interrupted-update, erase, rollback, and recovery operations all require detector, board-info, current manifest, current factory image, allow flags, aborts, and recovery steps before live action. | shell unit / typed allow | `bazel test //scripts:phase13_recovery_regression_test` plus any new Phase 16 gate test. | partial W0 | pending |
| 16-W0-04 | W0 | 0 | REL-07, EVD-05 | T-16-redaction | Redaction review template covers API bodies, WebSocket frames, recovery logs, destructive logs, terminal snippets, private `DEVICE_URL`, pool/Wi-Fi credentials, tokens, NVS secrets, and absent artifacts. | document check | `rg -n "DEVICE_URL|Wi-Fi|pool|token|NVS|WebSocket|absent artifacts|redaction" docs/parity/evidence/phase-16-current-commit-release-evidence-completion` | no W0 | pending |
| 16-PKG-01 | package | 1 | REL-04, REL-07, EVD-05 | T-16-identity | `just package` and release gate pass for current commit before flash or live evidence. | workflow | `just package && bazel run //tools/parity:report -- release-gate --manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json` | yes | pending |
| 16-SER-01 | serial | 1 | FND-06, REL-01, EVD-05 | T-16-wrong-device | Detector finds exactly one Ultra 205 port and `flash-monitor` records trusted current-commit serial boot evidence. | hardware-smoke | `just detect-ultra205` then `just flash-monitor board=205 port=<port> manifest=bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json evidence-dir=docs/parity/evidence/phase-16-current-commit-release-evidence-completion/serial-boot capture-timeout-seconds=35` | hardware-gated | pending |
| 16-HTTP-01 | live-http | 2 | API-09, REL-01, REL-02, REL-03, EVD-05 | T-16-url | Explicit `DEVICE_URL` probes capture live static, recovery, API coexistence, WebSocket no-upgrade, OTA route, and OTAWWW gap behavior or record blocked evidence. | hardware/network smoke | `scripts/phase13-http-static-smoke.sh --device-url "$DEVICE_URL" --manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --out-dir docs/parity/evidence/phase-16-current-commit-release-evidence-completion/http-static-recovery` plus Phase 16 OTA-route addition if implemented. | hardware-gated | pending |
| 16-OTA-01 | live-ota | 2 | REL-02, REL-08, EVD-05 | T-16-ota | Valid firmware OTA, invalid rejection, post-reboot identity, and boot-validation/rollback state are captured or blocked with explicit reason. | hardware/network smoke | `scripts/phase13-firmware-ota-smoke.sh --device-url "$DEVICE_URL" --manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --ota-image bazel-bin/firmware/bitaxe/esp-miner.bin --port <port> --out-dir docs/parity/evidence/phase-16-current-commit-release-evidence-completion/firmware-ota --monitor-seconds 45` | hardware-gated | pending |
| 16-REC-01 | recovery | 3 | REL-08, EVD-05 | T-16-recovery | Rollback, large erase, failed-update, and interrupted-update run only behind documented gates and restore safe state, or remain pending. | hardware-regression | `scripts/phase13-recovery-regression.sh --manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --factory-image bazel-bin/firmware/bitaxe/bitaxe-ultra205-factory.bin --ota-image bazel-bin/firmware/bitaxe/esp-miner.bin --port <port> --device-url "$DEVICE_URL" --out-dir docs/parity/evidence/phase-16-current-commit-release-evidence-completion/recovery-regression --allow-failed-update --allow-large-erase --allow-interrupted-ota` only after gates clear. | hardware-gated | pending |
| 16-DOC-01 | docs | 4 | REL-07, EVD-05 | T-16-overclaim | Release docs, parity checklist, requirements traceability, and milestone audit cite Phase 16 artifacts without promoting unsupported claims. | docs/parity | `just parity && just verify-reference && git diff -- reference/esp-miner --exit-code` | yes | pending |
| 16-FINAL-01 | final | 4 | FND-06, API-09, REL-01, REL-02, REL-03, REL-04, REL-07, REL-08, EVD-05 | T-16-final | Final verification status is `passed` only after package, release gate, redaction, parity, reference, relevant hardware/network/recovery checks, and lifecycle validation pass. | aggregate | `node "$HOME/.codex/get-shit-done/bin/gsd-tools.cjs" verify lifecycle 16 --expect-id 16-2026-07-01T12-36-46 --expect-mode yolo --require-plans --require-verification --raw` | yes | pending |

*Status: pending, green, red, blocked, or skipped with reason.*

***

## Wave 0 Requirements

- [ ] Current-commit identity gate compares git HEAD, package manifest source commit, flash-monitor observed commit, and evidence paths before promotion.
- [ ] HTTP/static evidence path covers `/api/system/OTA` in addition to existing `/`, `/assets/app.css.gz`, missing static, `/recovery`, API/WebSocket coexistence, and OTAWWW probes.
- [ ] Recovery/destructive helper gate applies detector, board-info, current manifest, current factory image, allow flags, aborts, recovery steps, and safe-state checks before every failed-update, interrupted-update, rollback, and erase action.
- [ ] Redaction review template exists before live artifacts are cited.

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

- [ ] All tasks have `<automated>` verify or hardware/manual blocker handling.
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify or recorded blocker.
- [ ] Wave 0 covers all missing references.
- [ ] No watch-mode flags.
- [ ] Feedback latency is bounded by targeted helper tests after each changed surface.
- [ ] `nyquist_compliant: true` set in frontmatter after Wave 0 is implemented and mapped to concrete plan tasks.

**Approval:** pending

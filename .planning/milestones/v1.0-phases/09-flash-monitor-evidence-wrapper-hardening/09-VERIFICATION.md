---
phase: 09-flash-monitor-evidence-wrapper-hardening
verified: 2026-06-29T15:33:53Z
status: passed
score: 9/9 must-haves verified
generated_by: gsd-verifier
lifecycle_mode: yolo
phase_lifecycle_id: "09-2026-06-29T13-16-47"
generated_at: 2026-06-29T15:33:53Z
lifecycle_validated: true
overrides_applied: 0
deferred:
  - truth: "Live HTTP/static/recovery/OTA/rollback release parity is not claimed by Phase 9 serial flash-monitor evidence."
    addressed_in: "Phase 13"
    evidence: "Phase 13 goal covers final package, flash, boot, HTTP, static, recovery, OTA, rollback, erase, failed-update, and interrupted-update evidence."
---

# Phase 9: Flash-Monitor Evidence Wrapper Hardening Verification Report

**Phase Goal:** A developer can capture Ultra 205 flash-monitor evidence through the repo wrapper without falling back to raw `espflash` commands.
**Verified:** 2026-06-29T15:33:53Z
**Status:** passed
**Re-verification:** No - initial verification

Material guidance applied: `AGENTS.md` repo-local Ultra 205 evidence rules, `AGENTS.bright-builds.md`, `standards/core/architecture.md`, `standards/core/code-shape.md`, `standards/core/testing.md`, `standards/core/verification.md`, `standards/languages/rust.md`, GSD verification overrides, and GSD gates.

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
| --- | --- | --- | --- |
| 1 | `tools/flash` supports a first-class noninteractive monitor/evidence path for `flash-monitor` that does not depend on an interactive input reader. | VERIFIED | `Justfile` routes `flash-monitor` through `bazel run //tools/flash:flash -- flash-monitor`; `prepare_evidence_monitor_command` emits `espflash monitor --chip esp32s3 --port <port> --non-interactive`; `prepare_monitor_command` remains `espflash monitor --port <port>`. |
| 2 | `just flash-monitor board=205 port=... evidence-dir=...` records board, port, source commit, reference commit, manifest, exact commands, monitor log, observed behavior, and conclusion. | VERIFIED | `flash-command-evidence.json` records board `205`, port `/dev/cu.usbmodem1101`, source commit `0a25ceeadc2788e8b93c4067603e71d7c067d372`, reference commit `c1915b0a63bfabebdb95a515cedfee05146c1d50`, manifest/image paths, flash/monitor commands, log path, observed commits, trusted status, and conclusion. |
| 3 | The wrapper prints clear recovery guidance for monitor startup failures and fails visibly when evidence capture cannot be trusted. | VERIFIED | `evidence_capture_failure_guidance` includes `just detect-ultra205`, wrapper `just flash-monitor board=205 port=<port> evidence-dir=<path>`, diagnostic-only `just monitor port=<port>`, and `evidence capture failed and is not trusted`; tests cover failed monitor guidance and untrusted timeout failure. |
| 4 | Fresh Ultra 205 wrapper evidence replaces the fallback-only evidence path without changing `reference/esp-miner`. | VERIFIED | Phase 9 ledger says raw fallback was not used; JSON/log are wrapper-produced; verifier ran `git diff -- reference/esp-miner --exit-code` with exit 0. |
| 5 | Evidence-mode capture is bounded and cannot hang indefinitely. | VERIFIED | `DEFAULT_MONITOR_CAPTURE_TIMEOUT_SECONDS` is 25; `execute_capturing` uses `spawn`, `try_wait`, deadline polling, `kill`, and `wait`; timeout is represented as `CaptureProcessStatus::TimedOut`. |
| 6 | A passing evidence record requires trusted Ultra 205 serial boot markers and matching source/reference identity. | VERIFIED | Trust logic requires exact boot/safe-state messages, tokenized SPIFFS/route markers, reset and ESP-IDF provenance, and 12+ hex observed commit prefixes matching expected commits; regression tests cover stale, truncated, and prefixed markers. |
| 7 | Fresh evidence was captured through `just flash-monitor ... evidence-dir=...`, not raw `espflash monitor` fallback. | VERIFIED | Ledger records `just detect-ultra205`, selected port `/dev/cu.usbmodem1101`, exact wrapper command, `raw espflash fallback: not used`, and generated JSON/log artifacts. |
| 8 | WF-005 cites wrapper-produced Phase 9 evidence while HTTP/static/recovery/OTA/rollback rows remain below verified. | VERIFIED | `WF-005` cites the Phase 9 ledger, JSON, and log; `FS-001`, `OTA-001`, `REL-001`, `REL-002`, and `REL-003` remain `implemented`; `OTA-002` remains `deferred | deferred`; release docs explicitly state serial evidence does not verify HTTP/static/recovery/OTA/rollback. |
| 9 | Release documentation tells operators how to recover from monitor startup failures without treating partial/raw logs as trusted proof. | VERIFIED | `docs/release/ultra-205.md` documents the wrapper command, `capture-timeout-seconds=25`, trusted statuses, fail-closed wording, recovery commands, diagnostic-only `just monitor`, and says raw `espflash monitor` output must not be trusted Phase 9 proof. |

**Score:** 9/9 truths verified

### Deferred Items

Items not yet met but explicitly addressed in later milestone phases.

| # | Item | Addressed In | Evidence |
| --- | --- | --- | --- |
| 1 | Live HTTP/static/recovery/OTA/rollback release parity remains outside Phase 9 serial evidence. | Phase 13 | Roadmap Phase 13 covers final package, flash, boot, HTTP, static, recovery, OTA, rollback, erase, failed-update, and interrupted-update evidence. |

### Required Artifacts

| Artifact | Expected | Status | Details |
| --- | --- | --- | --- |
| `tools/flash/src/main.rs` | Noninteractive evidence monitor path, bounded capture, marker classifier, enriched JSON, recovery guidance, tests | VERIFIED | Exists, substantive, wired through `run_flash_monitor`; unit tests cover command construction, trusted markers, timeouts, stale/truncated/prefixed commits, and recovery guidance. |
| `BUILD.bazel` | Source commit stamp git-state inputs | VERIFIED | Exports `.git/HEAD` and `.git/logs/HEAD` for source identity invalidation. |
| `firmware/bitaxe/BUILD.bazel` | Firmware and package source stamp wiring | VERIFIED | `firmware` consumes `//scripts:source_commit_stamp`; `firmware_image` includes the stamp input. |
| `firmware/bitaxe/build.rs` | Firmware commit injection from `BITAXE_SOURCE_COMMIT` | VERIFIED | Reads `BITAXE_SOURCE_COMMIT` first, falls back to `git rev-parse --short=12 HEAD`, and sets `BITAXE_FIRMWARE_COMMIT`. |
| `scripts/BUILD.bazel` | Source stamp target | VERIFIED | Defines `source_commit_stamp_tool` and `source_commit_stamp` genrule. |
| `scripts/build-firmware.sh` | Exports stamped source commit into firmware build | VERIFIED | Reads source commit file, rejects missing/empty stamp, exports `BITAXE_SOURCE_COMMIT`. |
| `scripts/source-commit-stamp.sh` | Branch-agnostic 12-character source stamp | VERIFIED | Uses `git rev-parse --short=12 HEAD` and writes the stamp. |
| `docs/parity/evidence/phase-09-flash-monitor-evidence-wrapper-hardening.md` | Human-readable evidence ledger | VERIFIED | Contains required sections, detector gate, exact wrapper command, artifact paths, marker table, scope boundary, secret review, and final conclusion. |
| `docs/parity/evidence/phase-09-flash-monitor-evidence-wrapper-hardening/flash-command-evidence.json` | Canonical machine-readable evidence | VERIFIED | JSON parsed and direct assertion passed for expected source/reference commits, command shape, capture status, observed commits, and conclusion. |
| `docs/parity/evidence/phase-09-flash-monitor-evidence-wrapper-hardening/flash-monitor.log` | Wrapper-owned noninteractive serial capture | VERIFIED | Contains trusted Ultra 205 boot, safe-state, OTA validation, SPIFFS, route-shell, reset, firmware/reference commit, and ESP-IDF markers. |
| `docs/parity/checklist.md` | WF-005 citation without release-row overclaim | VERIFIED | `just parity` passed with `validation_errors: none`; release-sensitive rows remain below verified. |
| `docs/release/ultra-205.md` | Operator recovery docs for trusted wrapper evidence | VERIFIED | Documents fail-closed statuses, recovery commands, diagnostic-only monitor use, and no raw monitor proof. |

### Key Link Verification

| From | To | Via | Status | Details |
| --- | --- | --- | --- | --- |
| `Justfile` | `tools/flash/src/main.rs` | `just flash-monitor` route | WIRED | `flash-monitor *args:` runs `bazel run //tools/flash:flash -- flash-monitor {{ args }}`. |
| `tools/flash/src/main.rs` | `espflash monitor` | Evidence command builder | WIRED | `prepare_evidence_monitor_command` returns an argument vector containing `monitor`, `--chip`, `esp32s3`, `--port`, selected port, `--non-interactive`. |
| `tools/flash/src/main.rs` | `flash-command-evidence.json` | `EvidenceRecord` serialization | WIRED | `write_flash_monitor_evidence_if_requested` calls `write_evidence_record`; record includes capture status, commands, observed commits, and conclusion. |
| `scripts/detect-ultra205.sh` | Phase 9 ledger | Detector gate | WIRED | Ledger records `just detect-ultra205`, selected `port=/dev/cu.usbmodem1101`, and board-info summary before flash-monitor evidence. |
| `docs/parity/checklist.md` | Phase 9 evidence files | WF-005 citation | WIRED | WF-005 cites the ledger, `flash-command-evidence.json`, and `flash-monitor.log`. |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
| --- | --- | --- | --- | --- |
| `tools/flash/src/main.rs` | `capture_outcome` / `EvidenceRecord` | `execute_capturing` writes `flash-monitor.log`, then `read_to_string` and `monitor_capture_outcome` classify it against expected commits | Yes | FLOWING |
| `firmware/bitaxe/build.rs` | `BITAXE_FIRMWARE_COMMIT` | `scripts/source-commit-stamp.sh` -> `scripts/build-firmware.sh` -> env var -> firmware log marker | Yes | FLOWING |
| `flash-command-evidence.json` | `observed_firmware_commit`, `observed_reference_commit`, `trusted_output` | Parsed from committed `flash-monitor.log` and compared to expected commits | Yes | FLOWING |
| `docs/parity/checklist.md` | WF-005 evidence citation | Static citation to committed Phase 9 evidence files, validated by `just parity` | Yes | FLOWING |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
| --- | --- | --- | --- |
| Hardware detector gate was run before evidence capture | `just detect-ultra205` | Recorded in ledger as exactly one port, `/dev/cu.usbmodem1101`, with ESP32-S3 board-info success | PASS |
| Wrapper flash-monitor capture produced trusted artifacts | `just flash-monitor board=205 port=/dev/cu.usbmodem1101 evidence-dir=docs/parity/evidence/phase-09-flash-monitor-evidence-wrapper-hardening` | Recorded JSON/log show wrapper-owned noninteractive capture with `trusted_output=true` | PASS |
| Evidence JSON/log match requested final commits and trusted markers | Phase 09 evidence assertion | Direct JSON/log assertion passed; `phase09_evidence_assertion=pass` | PASS |
| Parity checklist validates and avoids overclaiming verified release rows | `just parity` | Passed; `validation_errors: none` | PASS |
| Reference implementation unchanged | `git diff -- reference/esp-miner --exit-code` | Exit 0 | PASS |
| Full Bazel test surface | `just test` | Passed; 13/13 Bazel tests pass | PASS |
| Rust formatting | `cargo fmt --all -- --check` | Passed; non-mutating verifier equivalent of recorded `cargo fmt --all` | PASS |
| Rust lint | `cargo clippy --all-targets --all-features -- -D warnings` | Passed | PASS |
| Rust build | `cargo build --all-targets --all-features` | Passed | PASS |
| Rust tests | `cargo test --all-features` | Passed | PASS |
| Flash wrapper tests | `bazel test //tools/flash:tests` and `cargo test -p bitaxe-flash` | Bazel target passed; Cargo package tests passed 29/29 | PASS |
| Phase 09 code review | Read `09-REVIEW.md` | Clean review: 0 critical, 0 warning, 0 info findings | PASS |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
| --- | --- | --- | --- | --- |
| FND-07 | 09-01, 09-02 | Human commands route through Bazel or repo-owned scripts represented in the automation graph. | SATISFIED | `Justfile` exposes `build`, `test`, `package`, `flash`, `monitor`, `flash-monitor`, `verify-reference`, and `parity`; `just test` and `just parity` passed. |
| FND-08 | 09-01, 09-02 | USB flashing ergonomics support board 205, port handling, build-before-flash, and underlying command output. | SATISFIED | `tools/flash` resolves/validates board and port, builds package before flash, emits flash/monitor commands, and supports noninteractive evidence capture. |
| REL-07 | 09-02 | Build, flash, monitor, OTA, and recovery documentation is sufficient for safe Ultra 205 operation. | SATISFIED FOR PHASE 9 SCOPE | `docs/release/ultra-205.md` now documents trusted wrapper evidence and recovery. Full live HTTP/static/recovery/OTA/rollback proof remains explicitly deferred and not overstated. |
| EVD-05 | 09-01, 09-02 | Verification layers include unit tests, golden/API/hardware evidence where appropriate. | SATISFIED | Unit tests cover wrapper trust behavior; committed hardware-smoke JSON/log evidence exists; `just parity`, `just test`, and full Cargo checks passed. |

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
| --- | --- | --- | --- | --- |
| `docs/release/ultra-205.md` | 159 | "not available in this release candidate" | Info | Intentional release-boundary language for OTAWWW; not a stub because the row remains deferred and docs warn against overclaiming. |

No blocker anti-patterns were found in the modified source, scripts, docs, JSON evidence, or log artifacts. The only dirty worktree item before writing this report was `.planning/config.json` toggling `_auto_chain_active`; it is outside the phase source/evidence surfaces and was left untouched.

### Human Verification Required

None. The relevant hardware behavior for this phase is represented by committed wrapper-generated JSON/log evidence and was programmatically checked. No visual UI, live HTTP, OTA, recovery, rollback, or destructive/fault-injection behavior is claimed by Phase 9.

### Gaps Summary

No goal-blocking gaps found. Phase 9 delivered a wrapper-owned, bounded, fail-closed noninteractive flash-monitor evidence path; committed fresh Ultra 205 JSON/log evidence for source commit `0a25ceeadc2788e8b93c4067603e71d7c067d372`; preserved `reference/esp-miner`; and updated parity/release docs without promoting deferred HTTP/static/recovery/OTA/rollback claims.

_Verified: 2026-06-29T15:33:53Z_
_Verifier: the agent (gsd-verifier)_

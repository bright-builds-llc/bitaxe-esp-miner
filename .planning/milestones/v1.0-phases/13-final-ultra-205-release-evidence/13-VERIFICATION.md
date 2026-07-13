---
phase: 13-final-ultra-205-release-evidence
verified: 2026-06-30T19:23:48Z
status: passed
score: "4/4 must-haves verified"
generated_by: gsd-verifier
lifecycle_mode: yolo
phase_lifecycle_id: 13-2026-06-30T14-53-46
generated_at: 2026-06-30T19:23:48Z
lifecycle_validated: true
overrides_applied: 0
current_head: c68144bdd465d9281f31979a9a252fdb6c544e35
live_hardware_source_commit: 190849539700b8f9a7909fd2b6ebd84142557968
reference_commit: c1915b0a63bfabebdb95a515cedfee05146c1d50
---

# Phase 13: Final Ultra 205 Release Evidence Verification Report

**Phase Goal:** The final V1 source commit has package, flash, boot, HTTP, static, recovery, OTA, rollback, erase, failed-update, and interrupted-update evidence for Ultra 205 release parity.
**Verified:** 2026-06-30T19:23:48Z
**Status:** passed, with live HTTP/OTA/recovery/destructive evidence explicitly blocked or pending and not promoted to verified.

## Goal Achievement

Phase 13 achieved the release-evidence boundary required by the plan and this verification: package/release-gate and serial boot evidence exist, live `DEVICE_URL`-dependent evidence is explicitly blocked, destructive and fault-injection evidence remains pending behind documented allow gates, and release/checklist rows stay below `verified` where evidence is missing. This report does not claim full live release parity.

### Observable Truths

| # | Truth | Status | Evidence |
| --- | --- | --- | --- |
| 1 | Package identity, factory image, source commit, reference commit, install notes, license/provenance artifacts, and release gate are recorded before flash. | VERIFIED | `docs/parity/evidence/phase-13-final-ultra-205-release-evidence/package-release-gate.md` records `just package` and `release_gate: passed`; current verifier rerun of `just package` and release gate also passed with manifest source commit `c68144bdd465d9281f31979a9a252fdb6c544e35`. |
| 2 | Ultra 205 hardware flash and serial boot evidence captures identity, partition/image state, PSRAM/SPIFFS state, route registration, and safe startup. | VERIFIED | `serial-boot.md` records `serial_boot_status: passed` for board `205` on `/dev/cu.usbmodem1101`, firmware commit `190849539700`, reference commit `c1915b0...`, ESP-IDF `v5.5.4`, SPIFFS mounted, route shell started, and mining/work/control disabled. |
| 3 | Live HTTP/static/recovery/firmware OTA evidence is not overclaimed when `DEVICE_URL` is missing. | VERIFIED | `http-static-recovery.md` has `http_static_status: blocked` and `DEVICE_URL status: blocked - missing DEVICE_URL`; `firmware-ota.md` has `firmware_ota_status: blocked - DEVICE_URL unavailable`; checklist rows `API-004`, `API-007`, `API-008`, `FS-001`, `OTA-001`, `REL-001`, and `REL-002` remain below `verified`. |
| 4 | Recovery, rollback, erase, failed-update, and interrupted-update evidence is either captured or explicitly pending behind documented gates, without destructive/fault-injection overclaiming. | VERIFIED | `recovery-regression.md` records rollback/boot-validation pending, failed update pending without allow flag, large erase pending without allow flag, and interrupted update pending without allow flag; `recovery-runbook.md` and `scripts/phase13-recovery-regression.sh` require explicit allow flags, fresh detector confirmation, board-info, post-restore markers, and post-action smoke checks. |

**Score:** 4/4 must-haves verified.

## Required Artifacts

| Artifact | Expected | Status | Details |
| --- | --- | --- | --- |
| `docs/parity/evidence/phase-13-final-ultra-205-release-evidence.md` | Final Phase 13 ledger | VERIFIED | Summarizes package, serial, blocked live HTTP/OTA, pending recovery/destructive evidence, and final command set. |
| `docs/parity/evidence/phase-13-final-ultra-205-release-evidence/package-release-gate.md` | Package and release gate evidence | VERIFIED | Records package manifest, images, provenance/license docs, source/reference commits, and release gate pass. |
| `docs/parity/evidence/phase-13-final-ultra-205-release-evidence/serial-boot.md` | Live Ultra 205 flash/boot evidence | VERIFIED | Captures detector, board-info, flash-monitor command, boot markers, route shell, SPIFFS, and safe state. |
| `docs/parity/evidence/phase-13-final-ultra-205-release-evidence/http-static-recovery.md` | HTTP/static/recovery evidence boundary | VERIFIED | Explicitly blocked for missing `DEVICE_URL`; no live route success is claimed. |
| `docs/parity/evidence/phase-13-final-ultra-205-release-evidence/firmware-ota.md` | OTA evidence boundary | VERIFIED | Manifest/image validation is recorded; live valid/invalid OTA is blocked for missing `DEVICE_URL`. |
| `docs/parity/evidence/phase-13-final-ultra-205-release-evidence/recovery-regression.md` | Recovery/destructive evidence boundary | VERIFIED | Destructive and fault-injection paths remain pending without allow flags. |
| `docs/parity/checklist.md` | Evidence-aware parity status | VERIFIED | Missing live evidence rows remain `implemented` or `deferred`, not `verified`; `OTA-002` is deferred for the OTAWWW gap. |
| `docs/release/ultra-205.md` | Release operator guide | VERIFIED | Documents package/serial evidence, missing `DEVICE_URL`, below-verified checklist rows, and destructive gate requirements. |
| `scripts/phase13-*.sh` | Evidence capture helpers and tests | VERIFIED | Scripts are substantive, syntax-clean, tested, and enforce explicit URLs, allow flags, redaction, and post-action markers. |

## Key Link Verification

| From | To | Via | Status | Details |
| --- | --- | --- | --- | --- |
| Package manifest | Release gate | `tools/parity:report -- release-gate --manifest ...` | WIRED | Current verifier rerun passed; manifest includes image/provenance paths and reference commit. |
| Hardware detector | Flash/monitor evidence | `just flash-monitor board=205 port=... evidence-dir=...` | WIRED | Serial evidence records detector output, board-info, selected port, flash command, and captured log markers. |
| HTTP/static helper | `DEVICE_URL` live route probes | `scripts/phase13-http-static-smoke.sh` | WIRED, blocked by input | Helper refuses missing/invalid URL, disables scanning, and only probes explicit `DEVICE_URL`. |
| OTA helper | Firmware image, invalid rejection, post-OTA monitor | `scripts/phase13-firmware-ota-smoke.sh` | WIRED, blocked by input | Helper validates manifest checksum/name, requires explicit URL and port, and requires post-OTA serial markers before pass. |
| Recovery helper | Failed update, large erase, interrupted update | `scripts/phase13-recovery-regression.sh` | WIRED, pending by gate | Helper defaults destructive/fault actions to pending and requires explicit allow flags plus detector/board-info/post-smoke checks. |
| Checklist/release docs | Evidence files | Breadcrumbs and row notes | WIRED | Rows cite Phase 13 evidence and keep missing live evidence below `verified`. |

## Data-Flow And Guard Trace

| Artifact | Data/Decision | Source | Produces Real Evidence | Status |
| --- | --- | --- | --- | --- |
| `phase13-http-static-smoke.sh` | `DEVICE_URL` route probes | Explicit env/arg only | No scan or inference; missing URL records blocked | VERIFIED |
| `phase13-firmware-ota-smoke.sh` | OTA image and post-OTA status | Package manifest, curl response, monitor capture | Blocks on missing URL, bad checksum, accepted invalid image, or missing boot markers | VERIFIED |
| `phase13-recovery-regression.sh` | Destructive/fault-injection actions | Explicit allow flags, detector, board-info, smoke helpers | Defaults to pending; requires fresh port proof and post-action evidence | VERIFIED |
| `docs/parity/checklist.md` | Verification status promotion | Evidence breadcrumbs | Keeps live HTTP/OTA/recovery rows below `verified` where evidence is absent | VERIFIED |

## Behavioral Spot-Checks

| Behavior | Command | Result | Status |
| --- | --- | --- | --- |
| Phase 13 script syntax | `bash -n scripts/phase13-*.sh` | No syntax errors | PASS |
| Phase 13 script tests including monitor | `bazel test //scripts:phase13_http_static_smoke_test //scripts:phase13_monitor_capture_test //scripts:phase13_firmware_ota_smoke_test //scripts:phase13_recovery_regression_test` | 4 tests passed | PASS |
| Schema drift | `node .../gsd-tools.cjs verify schema-drift 13 --raw` | `drift_detected:false`, `blocking:false` | PASS |
| Rust format | `cargo fmt --all -- --check` | Clean; final ledger also records `cargo fmt --all` passed | PASS |
| Rust lint | `cargo clippy --all-targets --all-features -- -D warnings` | Finished successfully | PASS |
| Rust build | `cargo build --all-targets --all-features` | Finished successfully | PASS |
| Rust tests | `cargo test --all-features` | 370 unit tests passed; doc-tests had 0 tests | PASS |
| Package build | `just package` | Produced ELF, OTA app image, SPIFFS image, factory image, otadata, and package manifest | PASS |
| Release gate | `bazel run //tools/parity:report -- release-gate --manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json` | `release_gate: passed` | PASS |
| Parity report | `just parity` | `validation_errors: none` | PASS |
| Full Bazel tests | `just test` | 17 Bazel tests passed | PASS |
| Reference cleanliness | `just verify-reference` | `reference clean: c1915b0a63bfabebdb95a515cedfee05146c1d50` | PASS |
| Reference diff | `git diff -- reference/esp-miner --exit-code` | No diff | PASS |
| Code review | `13-REVIEW.md` and `13-REVIEW-FIX.md` | Review clean after 14 fixes; no remaining findings | PASS |

## Requirements Coverage

| Requirement | Status | Evidence |
| --- | --- | --- |
| `FND-06` | SATISFIED | Serial boot evidence captures safe startup, app identity, route shell, SPIFFS, ESP-IDF version, and disabled mining/control state. |
| `API-09` | SATISFIED WITH BOUNDARY | HTTP/static/recovery helpers and docs exist; live `DEVICE_URL` evidence is blocked and matching checklist rows are not verified. |
| `REL-01` | SATISFIED WITH BOUNDARY | Release docs and package manifest exist; live OTA/recovery evidence remains below verified. |
| `REL-02` | SATISFIED WITH BOUNDARY | Runbooks/helpers define valid OTA, rollback, and recovery checks; live OTA run is blocked by missing `DEVICE_URL`. |
| `REL-03` | SATISFIED WITH BOUNDARY | Recovery/destructive scripts and runbook exist; failed-update, large erase, rollback, and interrupted update evidence remains pending. |
| `REL-04` | SATISFIED | Package manifest, install notes, provenance, license docs, image outputs, and release gate are recorded and rechecked. |
| `REL-08` | SATISFIED WITH BOUNDARY | Release guide and checklist explicitly preserve gaps and pending live evidence instead of promoting release parity. |
| `EVD-05` | SATISFIED | Evidence stack includes package gate, serial hardware capture, blocked/pending ledgers, redaction review, tests, and clean review. |

## Anti-Patterns Found

No blocker anti-patterns were found. The only placeholder-like matches in Phase 13 scripts are `mktemp` template suffixes in tests, not implementation stubs. Redaction review reports no secrets, private endpoints, pool credentials, Wi-Fi credentials, or NVS secret values in Phase 13 evidence.

## No-Overclaim Audit

The missing live evidence is explicit and release-sensitive rows stay below verified:

- `DEVICE_URL status: blocked - missing DEVICE_URL`
- `http_static_status: blocked`
- `firmware_ota_status: blocked - DEVICE_URL unavailable`
- `failed_update_status: pending - allow flag not provided`
- `large_erase_status: pending - allow flag not provided`
- `interrupted_update_status: pending - allow flag not provided`
- rollback and boot-validation recovery status remain pending until OTA evidence runs
- checklist rows `API-004`, `API-007`, `API-008`, `FS-001`, `OTA-001`, `REL-001`, `REL-002`, and `REL-003` are not promoted to verified; `OTA-002` remains deferred

## Residual Risk

Live HTTP/static/recovery/OTA, rollback, large erase recovery, failed-update recovery, and interrupted-update recovery are still not verified on a reachable device URL. Hardware serial evidence is tied to the live-flashed package commit `190849539700b8f9a7909fd2b6ebd84142557968`; current HEAD `c68144bdd465d9281f31979a9a252fdb6c544e35` is verified by host package, release gate, parity, tests, and reference-clean checks, but is not overclaimed as live-flashed hardware evidence.

## Gaps Summary

No blocking verification gaps remain for the evidence-boundary goal. Full live release parity remains intentionally unclaimed until `DEVICE_URL` and the documented destructive/fault-injection gates are available and rerun.

_Verified: 2026-06-30T19:23:48Z_
_Verifier: the agent (gsd-verifier)_

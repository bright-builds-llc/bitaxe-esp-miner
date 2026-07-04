---
phase: 20-active-safety-hardware-telemetry-evidence
verified: 2026-07-04T00:08:36Z
status: passed
score: "26/26 must-haves verified"
generated_by: gsd-verifier
lifecycle_mode: yolo
phase_lifecycle_id: 20-2026-07-03T20-48-00
generated_at: 2026-07-04T00:08:36Z
lifecycle_validated: true
overrides_applied: 0
---

# Phase 20: Active Safety Hardware Telemetry Evidence Verification Report

**Phase Goal:** Active Ultra 205 safety-control behavior and live telemetry have hardware-regression evidence, or remain explicitly below verified with bounded recovery instructions and no overclaim.
**Verified:** 2026-07-04T00:08:36Z
**Status:** passed
**Re-verification:** No - previous report had no structured `gaps:` section, so this is an initial goal-backward verification after code review fixes.

## Goal Achievement

Phase 20 achieves the roadmap goal as an evidence-governance closure. It does not claim active voltage, fan, self-test, load, fault-stimulus, runtime display/input, or live safety telemetry freshness parity. Those surfaces are explicitly below verified or blocked where the required safe route, explicit target, or hardware-regression evidence does not exist.

## Observable Truths

| # | Truth | Status | Evidence |
| --- | --- | --- | --- |
| R1 | The phase plan documents recovery steps, explicit allow gates, stop conditions, redaction rules, and post-action safe-state checks before active safety hardware verification runs. | VERIFIED | `20-CONTEXT.md` D-01 through D-16 define the gates; `evidence-contract.md` lists metadata, aborts, recovery, safe-state markers, and evidence classes for every pack. |
| R2 | Active voltage/power, fan/thermal, self-test, watchdog/load, display/input, and failure-path evidence records board 205, selected port, commits, package manifest, commands, logs, observations, and conclusions, or stays below verified without overclaim. | VERIFIED | `summary.md` final matrix covers all packs; `safe-baseline.md` records board, port, source/reference commits, package identity, detector, board-info, flash-monitor, and safe-state markers; active packs record precise blocked/below-verified conclusions. |
| R3 | Live API and WebSocket telemetry evidence is correlated with hardware observations or explicitly blocked below verified. | VERIFIED | `live-api-websocket-telemetry.md` records missing explicit target, `network_scan: disabled`, bounded `/api/ws/live` settings, safe-baseline correlation context, and no freshness/cadence claim. |
| R4 | `just parity` continues to reject safety-critical verified rows without valid hardware-smoke or hardware-regression evidence. | VERIFIED | `just parity` passed with `validation_errors: none`; checked-in allow manifests validate; checklist rows keep active control/failure-path surfaces below verified without `hardware-regression`. |
| 1 | Failure-path probes are first-class `failure-paths` safety allow surface coverage. | VERIFIED | `tools/parity/src/safety_allow.rs` includes `failure-paths`, per-surface claim tiers, required fault-stimulus inputs, and tests for valid/rejected/deferred failure-path manifests. |
| 2 | The Phase 20 evidence tree exists before citations are added. | VERIFIED | Evidence tree contains 35 files, including contract, package gate, safe baseline, active packs, live telemetry pack, redaction review, and final summary. |
| 3 | Committed evidence has a redaction contract covering serial logs, manifests, API/WS artifacts, detector output, board-info output, command output, URLs, IP/MAC/SSID values, credentials, tokens, NVS secrets, and terminal secrets. | VERIFIED | `redaction-review.md` lists every required surface and reports `redaction_status: passed`, `raw_artifacts_committed: no`, and no raw private values. |
| 4 | Current-source package identity is captured before hardware or live telemetry evidence is trusted. | VERIFIED | `package-release-gate.md` and `safe-baseline.md` agree on package-time source commit `c11fba2622a389af533774447956b95f254c0280` and reference commit `c1915b0a63bfabebdb95a515cedfee05146c1d50`. Current HEAD is later, which is documented as expected after evidence commits. |
| 5 | Ultra 205 hardware work starts with `just detect-ultra205` and board-info, or records precise blocked evidence. | VERIFIED | `safe-baseline.md` has `detector_status: passed`, `board_info_status: passed`, `board: 205`; detector log records the ESP32-S3 board-info gate. |
| 6 | Safe-baseline evidence records safe-state markers before active/live packs consume serial logs. | VERIFIED | `safe-baseline.md` and `flash-monitor.log` record `safe_state: mining=disabled`, `asic_work_submission=disabled`, and `hardware_control=disabled`. |
| 7 | No network target is inferred from redacted serial evidence or scans. | VERIFIED | `target-lock.json` has `status: blocked` and `network_scan: disabled`; live telemetry pack records no `curl`, socket, ARP, mDNS, router, or inferred target use. |
| 8 | Power/current/voltage read-only telemetry and active voltage-control claims are separated. | VERIFIED | `active-power-voltage.md` separates read-only `power-telemetry` hardware-smoke wrapper boundary from `voltage-control` deferred unsupported-pending boundary. |
| 9 | Thermal/fan read-only telemetry, PID unit coverage, fan RPM, fan duty effects, and fault non-claims are separated. | VERIFIED | `active-thermal-fan.md` separates read-only thermal/fan wrapper boundary, pure PID unit evidence, and deferred fan-duty/fault behavior. |
| 10 | Unsupported active voltage or fan-duty routes produce blocked evidence instead of verified claims. | VERIFIED | `power-voltage.log` and `thermal-fan.log` record `claim_tier: unsupported-pending`, `evidence_class: deferred`, and no production-safe bounded route. |
| 11 | Power/thermal evidence is tied to the Phase 20 package, detector, board-info, commits, and safe-baseline serial log. | VERIFIED | Active pack ledgers cite source/reference commits, package manifest, allow manifests, wrapper commands, and safe-baseline evidence. |
| 12 | Self-test, watchdog/load, runtime display/input, and failure paths are independently promotable or blocked. | VERIFIED | Separate ledgers exist for `self-test-watchdog-load`, `runtime-display-input`, and `failure-paths`, each with claim rows and non-claims. |
| 13 | Failure-path evidence has a phase-owned wrapper and tests before any fault-stimulus claim is attempted. | VERIFIED | `scripts/phase20-failure-paths.sh` rejects `--stimulus`; `scripts/phase20-failure-paths-test.sh` and Bazel target pass. |
| 14 | No self-test, load, runtime display/input, or failure-path claim is verified without safe route, bounded inputs, aborts, recovery, and final safe-state marker. | VERIFIED | Ledgers record `pending`, `blocked`, `not_run`, `required-before-promotion`, and `below_verified` statuses instead of verified claims. |
| 15 | Live `/api/system/info` safety fields are captured only from explicit origin target or recorded as blocked. | VERIFIED | `live-telemetry.log` records `DEVICE_URL status: blocked - missing DEVICE_URL`; no target discovery was used. |
| 16 | Live `/api/ws/live` frames are bounded by duration and max frame count only from explicit origin target or blocked. | VERIFIED | `api-ws-live.txt` records `/api/ws/live`, `duration_ms=10000`, `max_frames=5`, and `websocket_target_status=blocked - missing DEVICE_URL`. |
| 17 | API/WebSocket telemetry evidence is correlated before freshness claims are made. | VERIFIED | `live-api-websocket-telemetry.md` records `telemetry_correlation_status: blocked` and only cites safe-baseline context, without freshness/cadence claims. |
| 18 | Route presence, no-upgrade responses, and stale cached bodies are not treated as live telemetry proof. | VERIFIED | `non_claims` explicitly excludes those as proof in live telemetry ledger and final summary. |
| 19 | Phase 20 has a final exact-claim summary covering every required surface. | VERIFIED | `summary.md` has `phase20_status: complete` and rows for all eight packs. |
| 20 | Redaction review passes before checklist or requirements citations. | VERIFIED | `redaction-review.md` reports `redaction_status: passed`; checklist and requirements cite final Phase 20 evidence. |
| 21 | Checklist updates preserve row IDs and promote only exact subclaims supported by matching evidence class. | VERIFIED | `docs/parity/checklist.md` preserves row IDs and keeps active safety-control/failure-path/live telemetry subclaims below verified unless evidence class supports them. |
| 22 | Final tests, parity, reference, lifecycle, and relevant checks pass before verification reports passed. | VERIFIED | Spot-checks passed in this verifier run; orchestrator also reported full `cargo fmt`, clippy, build, tests, `just test`, `just parity`, `just verify-reference`, Bazel script tests, lifecycle, and schema-drift checks. |

**Score:** 26/26 truths verified

## Deferred Items

No verification gaps were deferred by Step 9b. Residual below-verified items are expected Phase 20 non-claims, not failed must-haves. Phase 21 explicitly covers live mining and soak evidence; active voltage/fan/fault/self-test/load/runtime-display gaps remain future evidence work before those subclaims can be promoted.

## Required Artifacts

| Artifact | Expected | Status | Details |
| --- | --- | --- | --- |
| `tools/parity/src/safety_allow.rs` | First-class `failure-paths`, surface-specific claim validation, active-tier evidence guard | VERIFIED | Contains `failure-paths`, `allowed_claim_tiers_for_surface`, failure-path required inputs, live target validation, and 16 passing safety_allow tests. |
| `scripts/phase20-failure-paths.sh` | Phase-owned no-stimulus failure-path wrapper | VERIFIED | Uses `set -euo pipefail`, validates safety allow manifest, rejects `--stimulus`, and records blocked evidence. |
| `evidence-contract.md` | Pack contract for Phase 20 | VERIFIED | Defines all eight packs, evidence classes, metadata, non-claims, and citation rules. |
| `redaction-review.md` | Final redaction result | VERIFIED | `redaction_status: passed`, `raw_artifacts_committed: no`; strict private IP/MAC scan returned no matches. |
| `summary.md` | Final evidence ledger | VERIFIED | `phase20_status: complete`; matrix covers all required packs, requirements, supported subclaims, below-verified subclaims, and non-claims. |
| `package-release-gate.md` and package manifest | Package/release identity | VERIFIED | Release gate passed; package-time source/reference commits recorded and shared by target lock and safe baseline. |
| `safe-baseline.md` and logs | Detector-gated board 205 hardware smoke | VERIFIED | Detector, board-info, flash-monitor, package identity, and safe-state markers passed. |
| `target-lock.json` | No-scan target provenance | VERIFIED | JSON parses; `network_scan == "disabled"` and `status == "blocked"`. |
| Active pack ledgers | Power/voltage, thermal/fan, self-test/watchdog/load, display/input, failure-path boundaries | VERIFIED | Files are substantive, wired to wrapper outputs, and keep unsupported surfaces below verified. |
| Live telemetry pack | Explicit-target or blocked HTTP/WebSocket evidence | VERIFIED | Missing target is recorded, WebSocket settings are bounded, and no network discovery path is used. |
| `docs/parity/checklist.md` | Conservative Phase 20 citations | VERIFIED | Phase 20 citations exist; safety-critical row statuses remain conservative. |
| `.planning/REQUIREMENTS.md` | Requirements traceability | VERIFIED | Phase 20 final evidence note covers SAFE-01 through SAFE-09 and EVD-05. |
| `.planning/phases/20-active-safety-hardware-telemetry-evidence/20-VALIDATION.md` | Validation closure | VERIFIED | `status: pass`, rows 20-W0-01 through 20-W0-06 are pass with evidence paths. |

Note: `gsd-tools verify artifacts` reports Plan 20-01 invalid for the scaffold-only markers `redaction_status: pending` and `phase20_status: draft`. This is intentional supersession by Plan 20-06: the same files now contain final `redaction_status: passed` and `phase20_status: complete`, and Plan 20-06 artifact verification passes.

## Key Link Verification

| From | To | Via | Status | Details |
| --- | --- | --- | --- | --- |
| `evidence-contract.md` | `tools/parity/src/safety_allow.rs` | `failure-paths` surface | VERIFIED | Key-link verifier passed for 20-01. |
| `20-VALIDATION.md` | `tools/parity/src/safety_allow.rs` | `20-W0-01` validator coverage | VERIFIED | Validation row cites safety allow tests and contract. |
| `safe-baseline.md` | `scripts/detect-ultra205.sh` | detector gate | VERIFIED | Detector/board-info status present. |
| `safe-baseline.md` | `tools/flash` | `just flash-monitor board=205 ...` | VERIFIED | Flash-monitor command evidence and log present. |
| Phase 14 wrappers | Active/read-only evidence ledgers | allow-manifest gated output | VERIFIED | Wrapper logs record `safety_allow_status: passed` and pack statuses. |
| `scripts/phase20-failure-paths.sh` | `tools/parity/src/safety_allow.rs` | surface `failure-paths` validation | VERIFIED | Wrapper calls `safety-allow`; checked-in manifest validates. |
| `scripts/phase17-websocket-capture.mjs` | `api-ws-live.txt` | bounded capture output | VERIFIED | Artifact records path, duration, max frames, and blocked target status. |
| `docs/parity/checklist.md` | `summary.md` | Phase 20 evidence citation | VERIFIED | 20-06 key-link verifier passed. |
| `.planning/REQUIREMENTS.md` | `summary.md` | Phase 20 final evidence note | VERIFIED | Requirements traceability note present. |

## Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
| --- | --- | --- | --- | --- |
| `tools/parity/src/safety_allow.rs` | `surface`, `claim_tier`, `evidence_class`, `allowed_inputs`, `allowed_command` | Checked-in allow manifests plus package manifest JSON | Yes - validated against real manifests and package commits | VERIFIED |
| `summary.md` | Pack status matrix | Surface ledgers, logs, target lock, redaction review | Yes - cites concrete files and line-level status markers | VERIFIED |
| `safe-baseline.md` | Detector/package/safe-state status | `detect-ultra205.log`, `flash-command-evidence.json`, `flash-monitor.log`, package manifest | Yes - hardware smoke evidence exists | VERIFIED |
| Active pack ledgers | Status and non-claims | Wrapper logs and allow manifests | Yes - logs record passed safety allow validation plus blocked/deferred boundaries | VERIFIED |
| `live-api-websocket-telemetry.md` | HTTP/WS target and correlation status | `target-lock.json`, `live-telemetry.log`, `api-ws-live.txt`, safe baseline | Yes - produces blocked target evidence; no live frames claimed | VERIFIED |

## Behavioral Spot-Checks

| Behavior | Command | Result | Status |
| --- | --- | --- | --- |
| Failure-path wrapper and parity tests | `bazel test //tools/parity:tests //scripts:phase20_failure_paths_test` | 2 cached tests passed | PASS |
| Safety allow validation logic | `cargo test -p bitaxe-parity safety_allow` | 16 tests passed | PASS |
| WebSocket capture CLI contract | `node scripts/phase17-websocket-capture.mjs --help` | Usage printed successfully | PASS |
| Checklist no-overclaim validation | `just parity` | `validation_errors: none` | PASS |
| Reference remains read-only | `just verify-reference`; `git diff -- reference/esp-miner --exit-code` | Reference clean at `c1915b0a63bfabebdb95a515cedfee05146c1d50`; no diff | PASS |
| Lifecycle provenance | `gsd-tools verify lifecycle 20 --require-plans --require-verification --raw` | `valid` | PASS |
| Schema drift | `gsd-tools verify schema-drift 20` | `drift_detected: false` | PASS |
| Target lock safety | `jq -e '.network_scan == "disabled" and .status == "blocked"' target-lock.json` | `true` | PASS |
| Checked-in allow manifests | Loop over all 8 Phase 20 `allow*.json` files through `bazel-bin/tools/parity/report safety-allow` | All reported `safety_allow_status: passed` | PASS |
| Redaction strict value scan | Private IP/MAC regex over evidence tree | No matches | PASS |

The orchestrator also reported recent successful `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, `cargo test --all-features`, `just test`, `just parity`, `just verify-reference`, scoped Bazel tests, lifecycle validation, and schema-drift validation.

## Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
| --- | --- | --- | --- | --- |
| SAFE-01 | 20-02, 20-03, 20-05, 20-06 | Voltage/power fail-closed surfaces | SATISFIED | Safe-baseline passed; active voltage and unsafe recovery remain below verified in `active-power-voltage.md`; checklist does not overclaim. |
| SAFE-02 | 20-02, 20-03, 20-05, 20-06 | Thermal/fan readings, duty, RPM, failure reporting | SATISFIED | Thermal/fan ledger separates read-only boundary, PID unit evidence, and deferred fan/fault behavior. |
| SAFE-03 | 20-03, 20-06 | PID/thermal-control unit coverage before effects | SATISFIED | `active-thermal-fan.md` keeps PID as unit-only evidence and avoids hardware-effect promotion. |
| SAFE-04 | 20-01, 20-03, 20-04, 20-05, 20-06 | Fault paths safe states and user-visible status | SATISFIED | `failure-paths` surface exists; wrapper and ledger record blocked no-stimulus status and required future fields. |
| SAFE-05 | 20-04, 20-06 | Self-test lifecycle behavior | SATISFIED | `self-test-watchdog-load.md` records watchdog breadcrumbs and keeps hardware submodes/pass/fail/cancel below verified. |
| SAFE-06 | 20-04, 20-06 | Display/input preserved or deferred | SATISFIED | `runtime-display-input.md` records startup display breadcrumb and runtime input/display gap. |
| SAFE-07 | 20-02, 20-03, 20-05, 20-06 | Power/current/voltage/fan/temperature telemetry captured where exposed | SATISFIED | Safe-baseline exists; active telemetry/live correlation is blocked below verified without target or fresh sensor evidence. |
| SAFE-08 | 20-01 through 20-06 | Safety-critical verified rows require hardware evidence | SATISFIED | `just parity` passes; active claims require `hardware-regression`; checklist rows remain conservative. |
| SAFE-09 | 20-02, 20-04, 20-05, 20-06 | Watchdog responsiveness under load | SATISFIED | Startup/yield breadcrumbs recorded; bounded load and watchdog recovery remain below verified; Phase 21 covers live mining/soak. |
| EVD-05 | 20-01 through 20-06 | Verification layers include tests and appropriate hardware/regression evidence | SATISFIED | Unit/workflow tests, hardware-smoke safe baseline, blocked evidence, redaction, parity, reference, lifecycle, and schema checks all present. |

All requirement IDs listed by the user and PLAN frontmatter are accounted for in `.planning/REQUIREMENTS.md`. No additional Phase 20 requirement IDs were found orphaned outside the plans.

## Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
| --- | --- | --- | --- | --- |
| None blocking | N/A | N/A | N/A | Stub scan found only intentional evidence-boundary text such as missing `DEVICE_URL`, `pending`, `blocked`, and `below_verified`; these are the phase's conservative non-claim mechanism. |
| `package-release-gate/bitaxe-ultra205-package.json` | 15-16 | Dotted version strings matched broad IP-like regex | Info | Non-secret `cargo 1.88.0.0` and `rustc 1.88.0.0` tool versions; strict private IP/MAC scan returned no matches. |

## Human Verification Required

None for Phase 20 closure. The phase verifies committed evidence, validator behavior, and conservative traceability. Live telemetry freshness, active voltage/fan/fault/self-test/load behavior, runtime physical input/display behavior, and production mining/soak remain future below-verified work rather than human verification needed for this phase.

## Gaps Summary

No blocking gaps found. Phase 20 achieved its goal by closing the audit gap with exact evidence ledgers, a passed redaction review, safety allow validator coverage, conservative checklist/requirements traceability, and verified below-verified boundaries for unavailable active/live surfaces.

_Verified: 2026-07-04T00:08:36Z_
_Verifier: the agent (gsd-verifier)_

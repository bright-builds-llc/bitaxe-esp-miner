---
phase: 21-live-mining-and-soak-evidence
status: blocked
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 21-2026-07-04T01-35-47
generated_at: 2026-07-04T06:37:28Z
lifecycle_validated: true
requirements:
  - ASIC-07
  - STR-06
  - STR-07
  - SAFE-09
  - EVD-05
---

# Phase 21 Verification

final_phase_status: blocked
phase21_evidence_closure: blocked_or_below_verified
controlled_runtime_harness_status: ready
controlled_runtime_harness_observation_status: not observed in live smoke or soak
redaction_review: passed
reference_clean: passed
just_test: passed
just_parity: passed
just_verify_reference: passed
lifecycle_validation: passed
source_commit_after_traceability: 9eb35b1
reference_commit: c1915b0
summary_artifact: .planning/phases/21-live-mining-and-soak-evidence/21-08-SUMMARY.md
commit_push_gate: blocked until 21-VERIFICATION.md status passed and lifecycle validation succeeds

Phase 21 software verification, parity checks, reference cleanliness, and
redaction review are green. The phase evidence closure is still blocked because
the live mining smoke ledger records `live_mining_smoke_status: blocked`,
`blocker: missing_live_prerequisites`, `share_outcome: not-run`, and
`hardware_command_status: not-run`; the bounded soak ledger records
`bounded_soak_status: blocked`, `live_smoke_prerequisite: failed`, and
`hardware_command_status: not-run`; watchdog observations remain blocked
because bounded soak did not run.

## Final Verification Commands

| Command | Result |
| --- | --- |
| `bash -n scripts/phase21-live-mining-evidence.sh scripts/phase21-live-mining-evidence-test.sh scripts/phase21-live-mining-package.sh scripts/phase21-live-mining-package-test.sh` | passed |
| `bazel test //scripts:phase21_live_mining_evidence_test //scripts:phase21_live_mining_package_test` | passed |
| `cargo test -p bitaxe-parity --all-features mining_allow` | passed |
| `cargo test -p bitaxe-asic --all-features adapter_gate` | passed |
| `cargo test -p bitaxe-asic --all-features work` | passed |
| `cargo test -p bitaxe-asic --all-features result` | passed |
| `cargo test -p bitaxe-stratum --all-features controlled_runtime` | passed |
| `cargo test -p bitaxe-stratum --all-features mining_loop` | passed |
| `cargo test -p bitaxe-stratum --all-features fake_pool` | passed |
| `cargo test -p bitaxe-stratum --all-features queue` | passed |
| `cargo test -p bitaxe-api --all-features mining` | passed |
| `cargo test -p bitaxe-api --all-features telemetry` | passed |
| `node --check scripts/phase17-websocket-capture.mjs` | passed |
| `cargo fmt --all` | passed |
| `cargo clippy --all-targets --all-features -- -D warnings` | passed |
| `cargo build --all-targets --all-features` | passed |
| `cargo test --all-features` | passed |
| `just test` | passed |
| `just parity` | passed |
| `just verify-reference` | passed |
| `git diff -- reference/esp-miner --exit-code` | passed |
| `git diff --check` | passed |

Detailed command output was captured locally under
`target/phase21-08-final-verification/final-verification.log`, which is ignored
and not evidence for commit.

## Evidence Closure

| Artifact | Status | Closure impact |
| --- | --- | --- |
| `docs/parity/evidence/phase-21-live-mining-and-soak-evidence/summary.md` | blocked | Final ledger records `phase21_status: blocked` and `phase21_evidence_closure: blocked_or_below_verified`. |
| `docs/parity/evidence/phase-21-live-mining-and-soak-evidence/redaction-review.md` | passed | Redaction passed for committed artifacts only; this does not promote blocked evidence. |
| `docs/parity/evidence/phase-21-live-mining-and-soak-evidence/live-mining-smoke.md` | blocked | Missing live prerequisites, no live pool command, no controlled package boot, no pool input bridge, and no share outcome. |
| `docs/parity/evidence/phase-21-live-mining-and-soak-evidence/live-api-websocket-telemetry.md` | blocked | No explicit target; no HTTP or WebSocket runtime correlation was attempted. |
| `docs/parity/evidence/phase-21-live-mining-and-soak-evidence/bounded-soak.md` | blocked | Soak prerequisite failed because live smoke was blocked; no soak hardware command ran. |
| `docs/parity/evidence/phase-21-live-mining-and-soak-evidence/bounded-soak/watchdog-observations.md` | blocked | Startup watchdog breadcrumbs are not bounded soak responsiveness proof. |

rejected_completion_markers: missing_live_prerequisites, controlled_runtime_harness_not_observed, blocked live smoke, blocked soak, share_outcome not-run, missing controlled package boot, missing pool-input bridge

## hardware_command_inventory

| Plan | Command actually used | Result |
| --- | --- | --- |
| 21-03 | `just detect-ultra205` | passed for exactly one Ultra 205 candidate; committed detector evidence is redacted. |
| 21-03 | `just flash-monitor board=205 port=/dev/cu.usbmodem1101 evidence-dir=docs/parity/evidence/phase-21-live-mining-and-soak-evidence/preflight/safe-baseline capture-timeout-seconds=45 redact-evidence=true wifi-credentials=<local ignored credential file>` | passed; safe-baseline evidence committed redacted. |
| 21-04 | `just detect-ultra205` | passed before chip-detect diagnostic hardware use. |
| 21-04 | `scripts/phase15-bm1366-diagnostic-package.sh --mode chip-detect --out-dir docs/parity/evidence/phase-21-live-mining-and-soak-evidence/bm1366-init-work-result/chip-detect` | passed; package and allow manifest created. |
| 21-04 | `bazel run //tools/parity:report -- mining-allow --manifest docs/parity/evidence/phase-21-live-mining-and-soak-evidence/bm1366-init-work-result/chip-detect/allow-chip-detect.json --surface bm1366-chip-detect --allowed-command <manifest allowed_command>` | passed. |
| 21-04 | `bazel run //tools/flash:flash -- flash-monitor --board 205 --port /dev/cu.usbmodem1101 --manifest docs/parity/evidence/phase-21-live-mining-and-soak-evidence/bm1366-init-work-result/chip-detect/package/bitaxe-ultra205-package.json --evidence-dir docs/parity/evidence/phase-21-live-mining-and-soak-evidence/bm1366-init-work-result/chip-detect --capture-timeout-seconds 45 --redact-evidence` | passed; diagnostic chip-detect evidence committed redacted. |
| 21-05 | `just detect-ultra205` | passed before work-result diagnostic hardware use. |
| 21-05 | `scripts/phase15-bm1366-diagnostic-package.sh --mode work-result --out-dir docs/parity/evidence/phase-21-live-mining-and-soak-evidence/bm1366-init-work-result/work-result` | passed; package and allow manifest created. |
| 21-05 | `bazel run //tools/parity:report -- mining-allow --manifest docs/parity/evidence/phase-21-live-mining-and-soak-evidence/bm1366-init-work-result/work-result/allow-work-result.json --surface bm1366-work-result --allowed-command <manifest allowed_command>` | passed. |
| 21-05 | `bazel run //tools/flash:flash -- flash-monitor --board 205 --port /dev/cu.usbmodem1101 --manifest docs/parity/evidence/phase-21-live-mining-and-soak-evidence/bm1366-init-work-result/work-result/package/bitaxe-ultra205-package.json --evidence-dir docs/parity/evidence/phase-21-live-mining-and-soak-evidence/bm1366-init-work-result/work-result --capture-timeout-seconds 35 --redact-evidence` | passed; diagnostic work-result evidence committed redacted. |
| 21-06 | `just detect-ultra205` | passed; no live pool smoke command followed because live prerequisites were missing. |
| 21-07 | no fresh hardware command | blocked by failed live smoke prerequisite; prior detector log copied into bounded-soak pack. |
| 21-08 | no fresh hardware command | final closure was software-only and evidence-review only. |

## network_command_inventory

none - explicit DEVICE_URL absent

No `curl`, runtime `node scripts/phase17-websocket-capture.mjs`, live HTTP
capture, WebSocket connection, pool PATCH, pool credential use, or target URL
command was run for Phase 21 final closure. The only Phase 21 WebSocket command
in final verification was `node --check scripts/phase17-websocket-capture.mjs`,
which is syntax validation and does not open a network connection.

## Blockers

- Live smoke remains blocked by missing live prerequisites.
- No controlled package boot or pool input bridge evidence exists for actual live mining closure.
- `share_outcome` remains `not-run`; no accepted or rejected share behavior was observed.
- Bounded soak did not run because the live smoke prerequisite failed.
- Watchdog responsiveness under bounded mining or soak load remains blocked.
- Runtime statistics, telemetry freshness, API/WebSocket mining-load correlation, ASIC frequency transition, voltage, and fan claims remain below verified.

## Redaction And Reference

redaction_review: passed
raw_artifacts_committed: no
reference_clean: passed
network_scan: disabled

The final redaction review covers committed evidence only. It allows labels,
schema field names, redacted placeholders, command examples, USB port identity,
package/tool metadata, and explicit non-claims. It does not allow raw device
URLs, pool credentials, worker secrets, Wi-Fi credentials, API tokens, NVS
secret values, private endpoints, or unredacted target data.

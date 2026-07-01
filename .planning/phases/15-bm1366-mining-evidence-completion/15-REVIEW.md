---
phase: 15-bm1366-mining-evidence-completion
reviewed: 2026-07-01T05:03:49Z
depth: standard
files_reviewed: 33
files_reviewed_list:
  - crates/bitaxe-asic/src/bm1366/adapter_gate.rs
  - docs/parity/checklist.md
  - docs/parity/evidence/phase-15-bm1366-mining-evidence-completion.md
  - docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/README.md
  - docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/bounded-soak.md
  - docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/bounded-soak/allow-bounded-soak.json
  - docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/bounded-soak/bounded-soak.log
  - docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/bounded-soak/detect-ultra205.log
  - docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/chip-detect.md
  - docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/chip-detect/allow-chip-detect.json
  - docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/chip-detect/flash-command-evidence.json
  - docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/mining-smoke.md
  - docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/mining-smoke/allow-mining-smoke.json
  - docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/mining-smoke/detect-ultra205.log
  - docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/mining-smoke/mining-smoke.log
  - docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/redaction-review.md
  - docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/work-result.md
  - docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/work-result/allow-work-result.json
  - docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/work-result/flash-command-evidence.json
  - docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/work-result/flash-monitor.log
  - docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/work-result/package/bitaxe-ultra205-package.json
  - firmware/bitaxe/src/asic_adapter.rs
  - firmware/bitaxe/src/asic_adapter/status.rs
  - scripts/BUILD.bazel
  - scripts/phase15-bm1366-diagnostic-package-test.sh
  - scripts/phase15-bm1366-diagnostic-package.sh
  - scripts/phase15-controlled-mining-test.sh
  - scripts/phase15-controlled-mining.sh
  - scripts/phase15-websocket-capture.mjs
  - tools/flash/src/main.rs
  - tools/parity/BUILD.bazel
  - tools/parity/src/main.rs
  - tools/parity/src/mining_allow.rs
findings:
  critical: 0
  warning: 7
  info: 0
  total: 7
status: issues_found
---

# Phase 15: Code Review Report

**Reviewed:** 2026-07-01T05:03:49Z
**Depth:** standard
**Files Reviewed:** 33
**Status:** issues_found

## Summary

Reviewed the Phase 15 Rust firmware gates, shell wrappers, parity tooling, checklist updates, and checked-in evidence artifacts. The review used repo-local guidance from `AGENTS.md`, `AGENTS.bright-builds.md`, `standards-overrides.md`, and the loaded Bright Builds architecture, code-shape, verification, testing, and Rust standards. No project-local skills were present under `.claude/skills/` or `.agents/skills/`, and none of the reviewed files were ignored by git.

No critical secret leak or direct unsafe raw hardware command was found in the reviewed files. The issues are warning-level because they can let Phase 15 evidence or future parity validation overstate what was actually gated, captured, or allowed.

## Warnings

### WR-01: Detector Logs Do Not Substantiate Board-Info Claims

**File:** `docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/mining-smoke.md:64`
**Issue:** The mining-smoke evidence says the detector output selected board `205` and recorded successful ESP32-S3 board-info output, but the cited checked-in detector log contains only `port=/dev/cu.usbmodem1101` at `docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/mining-smoke/detect-ultra205.log:1`. The same pattern appears for bounded-soak at `docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/bounded-soak.md:59` versus `bounded-soak/detect-ultra205.log:1`, while the final ledger claims the board-info gate passed at `docs/parity/evidence/phase-15-bm1366-mining-evidence-completion.md:32`. This is an evidence overclaim: the committed artifact does not prove the required board-info gate.
**Fix:** Replace the one-line detector logs with the full `just detect-ultra205` transcript including the `espflash board-info --chip esp32s3 --port ... --non-interactive` output, or downgrade the Markdown/ledger claims to say only that the selected port was recorded and board-info evidence is pending.

### WR-02: Redaction Review Is Marked Passed With Unchecked Checklist Items

**File:** `docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/redaction-review.md:11`
**Issue:** The review declares cited artifacts passed redaction review, but every checklist item under `## Review Checklist` remains unchecked at lines 20-48. For privacy-sensitive evidence, that creates an ambiguous audit state: readers cannot tell whether the checklist was actually completed or the pass status was copied forward without marking the reviewed controls.
**Fix:** Mark each completed review item with `[x]`, or split absent artifact classes into explicit `N/A - no artifact generated` entries. Keep the `passed` status only after the checklist state matches the narrative and table.

### WR-03: Environment Variables Can Trigger Live Probes Outside The Manifested Command

**File:** `scripts/phase15-controlled-mining.sh:13`
**Issue:** `device_url` defaults from `DEVICE_URL`, but `command_arg_list` only includes `--device-url` when the CLI flag was provided at lines 160-162. As a result, a manifest whose `allowed_command` and `allowed_inputs.device_url` say the device URL is missing, such as `docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/mining-smoke/allow-mining-smoke.json:15`, can still enter `record_live_attempt` when `DEVICE_URL` and pool env vars are present because `live_prerequisites_missing` checks environment-derived state at lines 296-300 and the live branch runs at lines 442-447. That bypasses the procedure-scoped allow contract and can generate live API/WebSocket probes under a controlled-no-share manifest.
**Fix:** Do not read `DEVICE_URL` implicitly for evidence-producing runs. Require `--device-url` for live attempts, include it in the allowed command, and fail or record pending unless the validated manifest claim tier and `allowed_inputs.device_url` explicitly authorize a live URL.

### WR-04: Mining Allow Validation Does Not Bind Surfaces To Claim Tiers Or Require The Command Filter

**File:** `tools/parity/src/mining_allow.rs:220`
**Issue:** The mining allow validator checks that the surface is in the allowed surface list and the claim tier is in the allowed tier list, but it does not validate legal surface/tier pairings. It also only checks that `allowed_command` is non-empty at lines 242-245, while exact command matching is optional through `maybe_allowed_command` at lines 377-384. A manifest can therefore pass direct validation with a mismatched surface/tier or a dangerous command string whenever the caller omits `--allowed-command`.
**Fix:** Add a surface-to-claim-tier matrix, for example chip-detect only permits `diagnostic-chip-detect`, work-result only permits `diagnostic-work-result`, mining-smoke permits controlled/live smoke tiers, bounded-soak permits bounded or unsupported-pending tiers, and parity-redaction only permits `parity-redaction`. Also make the allowed-command filter mandatory for the CLI path or reject non-wrapper commands and prohibited tokens directly in `validate_required_procedure_scope`.

### WR-05: STR-008 Validation Can Accept A Controlled No-Share Overclaim

**File:** `tools/parity/src/main.rs:621`
**Issue:** `has_mining_smoke_or_soak_details` treats `controlled no-share condition` as sufficient share outcome metadata for a verified `STR-008` row when board, port, commits, redaction, and conclusion are present. The positive test at lines 1222-1237 confirms this behavior. Phase 15 evidence explicitly says the current controlled no-share state is due to missing live prerequisites and keeps broad live smoke/soak below verified, so this validator can approve the exact overclaim the phase documents are trying to prevent.
**Fix:** Require `STR-008` verified notes to contain either accepted/rejected live share evidence or an explicitly approved bounded controlled-no-share soak with no blocker terms such as `missing live prerequisites`, `pending`, `blocked`, `not run`, or `below verified`. Apply blocker-term rejection to ASIC/mining verified rows, not only release/OTA rows.

### WR-06: Frequency Transition Can Be Promoted Without A Hardware-Control Guard

**File:** `tools/parity/src/main.rs:554`
**Issue:** The safety-critical detector does not include `frequency`, and `is_live_asic_or_mining_row` omits `ASIC-007` at lines 610-615. The checklist row at `docs/parity/checklist.md:65` correctly says Phase 15 has no bounded frequency-transition hardware-regression artifact, but the parity validator would not block a future `ASIC-007` verified row backed only by unit/workflow evidence. Frequency transition is a hardware-control surface and should not be promotable without bounded hardware evidence.
**Fix:** Add `frequency`/`frequency transition` to the safety-critical or active hardware-control validation path, include `ASIC-007` in the ASIC/mining guard, and require `hardware-regression` or an explicit bounded frequency-transition hardware artifact before verified status.

### WR-07: API Probe Failure Can Abort Instead Of Recording Pending Evidence

**File:** `scripts/phase15-controlled-mining.sh:339`
**Issue:** `probe_api_status` captures curl's status under `set +e`, but then unconditionally redacts `$body_file.tmp` and `$error_file.tmp` at lines 345-346. If `curl` is missing or fails before creating either temp file, `set -e` will abort the wrapper instead of writing the controlled pending markers that downstream evidence expects.
**Fix:** Pre-create the temp files before invoking curl and guard the redaction step:

```bash
: >"$body_file.tmp"
: >"$error_file.tmp"
status="$("$curl_bin" ... --output "$body_file.tmp" ... 2>"$error_file.tmp")"
redact_text <"$body_file.tmp" >"$body_file"
redact_text <"$error_file.tmp" >"$error_file"
```

If curl cannot be executed, log `api_telemetry_status=pending - curl failed` and continue to the final pending conclusion.

## Info

No info-level findings.

## Review Notes

The quick-pattern scan produced only false positives from `mktemp` templates containing `XXXXXX`. No ignored files were included in the review scope.

_Reviewed: 2026-07-01T05:03:49Z_
_Reviewer: the agent (gsd-code-reviewer)_
_Depth: standard_

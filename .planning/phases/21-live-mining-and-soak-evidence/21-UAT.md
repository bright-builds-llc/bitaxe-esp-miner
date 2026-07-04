---
status: complete
phase: 21-live-mining-and-soak-evidence
source:
  - .planning/phases/21-live-mining-and-soak-evidence/21-01-SUMMARY.md
  - .planning/phases/21-live-mining-and-soak-evidence/21-02-SUMMARY.md
  - .planning/phases/21-live-mining-and-soak-evidence/21-03-SUMMARY.md
  - .planning/phases/21-live-mining-and-soak-evidence/21-04-SUMMARY.md
  - .planning/phases/21-live-mining-and-soak-evidence/21-05-SUMMARY.md
  - .planning/phases/21-live-mining-and-soak-evidence/21-06-SUMMARY.md
  - .planning/phases/21-live-mining-and-soak-evidence/21-07-SUMMARY.md
  - .planning/phases/21-live-mining-and-soak-evidence/21-08-SUMMARY.md
started: 2026-07-04T16:00:47Z
updated: 2026-07-04T17:13:08Z
---

## Current Test

[testing complete]

## Tests

### 1. Evidence Wrapper And Readiness Gates

expected: Phase 21 live mining commands are routed through a repo-owned wrapper that blocks by default, records missing live prerequisites, disables network scanning, and keeps raw runtime inputs out of committed evidence.
result: pass
verified_by: agent
evidence: "`scripts/phase21-live-mining-evidence.sh` contains `missing_live_prerequisites`, `network_scan: disabled - DEVICE_URL must be explicit`, JSON credential loading, and redaction filters. `scripts/phase21-live-mining-evidence-test.sh` covers blocked missing-prerequisite behavior."
notes: Existing committed Phase 21 evidence remains historical and blocked until a later rerun produces fresh evidence.

### 2. Controlled Runtime Enablement Is Opt-In

expected: Controlled live mining support exists only behind explicit enablement artifacts and does not turn readiness evidence into live mining proof.
result: pass
verified_by: agent
evidence: "`.planning/phases/21-live-mining-and-soak-evidence/21-02-SUMMARY.md` and `docs/parity/evidence/phase-21-live-mining-and-soak-evidence/live-mining-enablement.md` document the controlled runtime package, compile-time harness, safe-stop plan, and ready ledger as prerequisites rather than proof."
notes: This passes as a static evidence and claim-boundary check only.

### 3. Preflight Hardware Evidence Exists With Redaction Boundaries

expected: Phase 21 has detector-gated Ultra 205 preflight evidence, safe-baseline boot evidence, package identity, and explicit no-mining safe-state claims.
result: pass
verified_by: agent
evidence: "`.planning/phases/21-live-mining-and-soak-evidence/21-03-SUMMARY.md` reports a detector-gated board-info pass, package-backed safe baseline run, and safe-state markers. Committed artifacts use redacted evidence paths and do not promote live mining claims."
notes: Raw port, URL, IP, MAC, Wi-Fi, or pool values are intentionally not repeated in this UAT file.

### 4. BM1366 Diagnostics Are Fail-Closed

expected: BM1366 diagnostic evidence is recorded as lower-tier hardware evidence and does not claim full ASIC initialization, accepted shares, or production mining.
result: pass
verified_by: agent
evidence: "`.planning/phases/21-live-mining-and-soak-evidence/21-04-SUMMARY.md`, `.planning/phases/21-live-mining-and-soak-evidence/21-05-SUMMARY.md`, and `docs/parity/evidence/phase-21-live-mining-and-soak-evidence/bm1366-init-work-result.md` describe fail-closed chip-detect and work-result diagnostics."
notes: The work-result diagnostic timed out safely and remains below live mining verification.

### 5. Historical Live Smoke And Soak Evidence Remains Blocked

expected: Existing Phase 21 live mining smoke, API/WebSocket telemetry, bounded soak, watchdog, and share evidence are not treated as verified when prerequisites were missing.
result: pass
verified_by: agent
evidence: "`.planning/phases/21-live-mining-and-soak-evidence/21-06-SUMMARY.md`, `.planning/phases/21-live-mining-and-soak-evidence/21-07-SUMMARY.md`, `.planning/phases/21-live-mining-and-soak-evidence/21-08-SUMMARY.md`, `docs/parity/evidence/phase-21-live-mining-and-soak-evidence/live-mining-smoke.md`, `docs/parity/evidence/phase-21-live-mining-and-soak-evidence/bounded-soak.md`, and `docs/parity/evidence/phase-21-live-mining-and-soak-evidence/summary.md` record blocked or below-verified conclusions."
notes: A new live rerun can supersede this only with fresh prerequisites and redacted evidence.

### 6. Local JSON Pool Credentials Are Ready Without Disclosure

expected: The owner-supplied local JSON pool credential file exists, validates through the repo helper, and is treated only as sensitive local runtime input.
result: pass
verified_by: agent
evidence: "`test -s pool-credentials.json` succeeded. `node scripts/phase21-pool-credentials-json.mjs pool-credentials.json >/dev/null` exited successfully without printing raw values."
notes: The JSON credential values were not read into this file, printed, summarized, or committed.

### 7. Fresh Ultra 205 Detector Gate Currently Passes

expected: Before any autonomous hardware rerun, `just detect-ultra205` must pass freshly with exactly one Ultra 205 candidate and board-info success.
result: pass
verified_by: agent
evidence: "`just detect-ultra205` returned exit status 0 before hardware use in this UAT attempt. Ignored local artifacts under `target/phase21-uat-complete-20260704T165435Z` record `port_count=1` and a redacted board-info command for the single Ultra 205 candidate; raw detector output was not copied into committed evidence."
notes: This permits using the detected port for repo-owned commands, but it does not supply a `DEVICE_URL`.

### 8. Fresh Same-Session DEVICE_URL And Live Rerun

expected: A fresh repo-owned monitor or flash-monitor run from the same current session provides exactly one origin-only `http://...` or `https://...` `DEVICE_URL` candidate, after which the Phase 21 live mining wrapper is rerun with `--pool-credentials pool-credentials.json`. The rerun records only redacted categories such as local owner supplied pool config, never raw pool values, address, worker, endpoint, password, IPs, MACs, Wi-Fi values, or target URL.
result: pass
verified_by: agent
evidence: "target/phase21-uat-retry-20260704T170549Z contains ignored local runtime artifacts only: fresh `just detect-ultra205` returned exit status 0 with `port_count=1`; fresh controlled package generation recorded package and harness ready; the same-session flash-monitor serial log had `monitor_origin_candidate_count=1` and `device_url_marker_count=1`; `node scripts/phase21-pool-credentials-json.mjs pool-credentials.json >/dev/null` succeeded; the local live-smoke wrapper returned `wrapper_exit_status=0`, `mining_allow_status_passed=true`, `pool_input_bridge_status_applied=true`, `pool_settings_consumed_by_runtime_true=true`, `controlled_mining_status_live-prerequisites-present=true`, and `api_telemetry_status_http_status_200_curl_0=true`."
notes: Raw DEVICE_URL, pool values, worker/address, password, Wi-Fi values, IPs, MACs, and endpoint values were not copied into this tracked UAT file. The wrapper conclusion remained `pending_share_review`, so this UAT pass does not promote share, soak, or watchdog evidence beyond the wrapper/live-probe boundary.

## Summary

total: 8
passed: 8
issues: 0
pending: 0
skipped: 0
blocked: 0

## Gaps

[]

## Residual Risks

- Existing committed Phase 21 evidence remains historically blocked until promoted by a deliberate redacted evidence update.
- This UAT confirms the fresh detector, same-session monitor-derived target, JSON pool credential bridge, settings consumption marker, and live wrapper/API probe path. It does not claim accepted/rejected share evidence, bounded soak, or watchdog responsiveness evidence.

---
status: partial
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
updated: 2026-07-04T16:22:01Z
---

## Current Test

[testing paused - 1 item outstanding]

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
evidence: "A fresh temp-file capture of `just detect-ultra205` returned `detect_status=0`, `port_count=1`, and `board_info_marker_count=1`; the raw detector output was not printed and the temp file was deleted."
notes: This permits using the detected port for repo-owned commands, but it does not supply a `DEVICE_URL`.

### 8. Fresh Same-Session DEVICE_URL And Live Rerun

expected: A fresh repo-owned monitor or flash-monitor run from the same current session provides exactly one origin-only `http://...` or `https://...` `DEVICE_URL` candidate, after which the Phase 21 live mining wrapper is rerun with `--pool-credentials pool-credentials.json`. The rerun records only redacted categories such as local owner supplied pool config, never raw pool values, address, worker, endpoint, password, IPs, MACs, Wi-Fi values, or target URL.
result: blocked
blocked_by: physical-device
reason: "Agent verified fresh detector status, current controlled live-mining package generation, trusted controlled boot, exactly one origin-only same-session DEVICE_URL candidate, and valid local JSON pool credentials. The live wrapper was not run because the required pool-input bridge was not proven: PATCH /api/system timed out with curl status 28 and HTTP status 000, redacted device logs lacked settings-patch and controlled-runtime lifecycle markers, and running the wrapper would bypass the Phase 21 gate."
evidence: "target/phase21-uat-live contains ignored local runtime artifacts only: detect_status=0, port_count=1, current package manifest source matched HEAD, trusted_output=true, capture_status=timed_out_after_trusted_output, device_url_candidate_count=1, pool_credentials_valid=yes, api_reachability_status=reachable, pool_input_bridge_status=blocked, pool_patch_curl_status=28, pool_patch_http_status=000, and no redacted settings-patch/runtime markers in fetched device logs."
notes: Raw DEVICE_URL, pool values, worker/address, password, Wi-Fi values, IPs, MACs, and endpoint values were not copied into this tracked UAT file.

## Summary

total: 8
passed: 7
issues: 0
pending: 0
skipped: 0
blocked: 1

## Gaps

[]

## Residual Risks

- Existing committed Phase 21 evidence remains blocked until a fresh rerun produces redacted live mining and soak artifacts.
- The current live UAT attempt stopped at the pool-input bridge gate; pool reachability, share observation, and watchdog behavior remain unverified.

---
status: diagnosed
trigger: "Diagnose Phase 17 UAT gap root cause only: expected live HTTP/static/API evidence from a just-flashed device, but UAT found missing DEVICE_URL-driven artifacts and blocked ledgers."
created: 2026-07-03T02:10:55Z
updated: 2026-07-03T02:22:01Z
---

## Current Focus
<!-- OVERWRITE on each update - reflects NOW -->

hypothesis: Confirmed - Phase 17 completed the planned no-scan blocked path because no explicit origin-only DEVICE_URL was available, but the UAT truth required live target-lock and per-route artifacts.
test: Diagnosis complete; no fix applied by request.
expecting: Future gap closure must supply an explicit reachable DEVICE_URL and treat missing target as unresolved for live-evidence UAT, not as live-evidence completion.
next_action: return root cause report

## Symptoms
<!-- Written during gathering, then IMMUTABLE -->

expected: summary.md, http-static-api.md, and the http-static-api/ artifacts show an explicit reachable DEVICE_URL, a sanitized target-lock.json, and live HTTP status/response summaries for /, /assets/app.css.gz, representative missing static behavior, /recovery, /api/system/info, /api/ws, /api/ws/live, and OTA route coexistence from the just-flashed device.
actual: Agent-performed artifact check found no target-lock.json, no per-route HTTP header/body/curl-error artifacts, and ledgers explicitly recording http_static_api_status: blocked because DEVICE_URL was missing.
errors: None reported beyond missing DEVICE_URL/evidence artifacts.
reproduction: Test 3 in Phase 17 UAT.
started: Discovered during UAT.

## Eliminated
<!-- APPEND only - prevents re-investigating -->

## Evidence
<!-- APPEND only - facts discovered -->

- timestamp: 2026-07-03T02:10:55Z
  checked: git merge-base HEAD d2f5cdf45dc933cb5f6cc4ec729108fc26d75254
  found: merge-base was d2f5cdf45dc933cb5f6cc4ec729108fc26d75254
  implication: Investigation can proceed from the expected base ancestry without rewriting history.
- timestamp: 2026-07-03T02:13:22Z
  checked: Required Phase 17 UAT, planning, summary, verification, evidence, helper script, and debugger references.
  found: UAT Test 3 expects reachable DEVICE_URL live HTTP/static/API artifacts; Plan 17-03 and evidence summaries explicitly record missing DEVICE_URL, absent target-lock.json, no live probes, and absent per-route artifacts. The helper exits before route probes when device_url is empty.
  implication: Primary failure path is at target input/provenance before curl route capture, not at individual HTTP route response validation.
- timestamp: 2026-07-03T02:16:14Z
  checked: Knowledge base and Phase 17 artifact search for DEVICE_URL/target-lock provenance.
  found: No debug knowledge base exists. The Phase 17 evidence directory has no target-lock.json and no per-route HTTP artifacts; only http-static-api/http-static-api.log exists for HTTP route evidence. Search results show repeated explicit missing-DEVICE_URL/absent-target-lock statements and the executed HTTP helper command lacks --device-url.
  implication: There is no evidence that an explicit target existed and was mishandled; the helper ran without the required target input.
- timestamp: 2026-07-03T02:18:45Z
  checked: Plan 17-03 instructions versus Phase 17 roadmap/UAT expectations.
  found: Roadmap success criteria require an explicit reachable DEVICE_URL and live HTTP/API/WebSocket capture. Plan 17-03's task and success criteria permit completion with "or write blocked evidence" / "or precise no-scan blocked evidence" when DEVICE_URL is absent, and its automated verification only greps for blocked ledger fields rather than requiring target-lock.json or per-route artifacts.
  implication: The implementation followed the plan, but the plan's completion gate was weaker than the phase/UAT truth; a blocked prerequisite was accepted as plan completion.
- timestamp: 2026-07-03T02:22:01Z
  checked: Helper transcript, serial evidence notes, README target gate, and context decisions.
  found: The helper transcript records DEVICE_URL blocked/missing, target_status blocked, and no curl route probes. Serial evidence explicitly says no DEVICE_URL was used, inferred, or recorded. README/context require explicit origin-only DEVICE_URL and forbid scanning/inference; blocked artifacts may only prove the blocker and must keep rows below verified.
  implication: Root cause is not route handler behavior or artifact redaction; it is missing explicit target input combined with a plan completion gate that accepted blocked evidence for a live-evidence phase.

## Resolution
<!-- OVERWRITE as understanding evolves -->

root_cause: Phase 17's live HTTP/static/API evidence gap was caused by a target-input and gating mismatch. The required explicit origin-only DEVICE_URL was not provided, so scripts/phase17-live-http-api-smoke.sh intentionally took its no-scan blocked branch and exited before writing target-lock.json or any per-route headers/body/curl-error artifacts. Plan 17-03 allowed that blocked path to complete the plan, but UAT Test 3 and the roadmap/verification truth required live artifacts from a reachable just-flashed device.
fix: Not applied per find_root_cause_only request. Gap closure should rerun the Phase 17 HTTP/WebSocket evidence flow with an explicit reachable DEVICE_URL and require target-lock/per-route artifacts for live-evidence UAT completion.
verification: Confirmed by comparing UAT/roadmap expectations, Plan 17-03 blocked-path acceptance, script target gating, helper transcript, file inventory, and Phase 17 summaries.
files_changed: []

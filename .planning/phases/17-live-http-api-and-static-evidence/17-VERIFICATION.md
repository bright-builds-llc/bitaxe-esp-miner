---
phase: 17
phase_name: live-http-api-and-static-evidence
verified: 2026-07-02T03:50:19Z
status: gaps_found
score: "1/4 must-haves verified"
generated_by: gsd-verifier
lifecycle_mode: interactive
phase_lifecycle_id: "17-2026-07-02T01-09-48"
generated_at: 2026-07-02T03:50:19Z
lifecycle_validated: true
overrides_applied: 0
gaps:
  - truth: "Ultra 205 detector output, board 205, selected port, source commit, reference commit, package manifest, and explicit reachable DEVICE_URL are recorded without network scanning or secrets."
    status: partial
    reason: "Detector, board, port, package, flash identity, source commit, reference commit, and no-scan/redaction evidence are recorded, but the evidence explicitly says DEVICE_URL was missing and no target-lock.json exists."
    artifacts:
      - path: "docs/parity/evidence/phase-17-live-http-api-and-static-evidence/summary.md"
        issue: "Records live HTTP/static/API and WebSocket probes blocked because no explicit origin-only DEVICE_URL or explicit-input target-lock.json was available."
      - path: "docs/parity/evidence/phase-17-live-http-api-and-static-evidence/http-static-api.md"
        issue: "Records device_url_status: blocked - missing explicit origin-only DEVICE_URL and target_lock_status: absent - not cited."
      - path: "docs/parity/evidence/phase-17-live-http-api-and-static-evidence/websocket.md"
        issue: "Records device_url_status: blocked and no explicit-input target lock."
    missing:
      - "Provide an explicit origin-only DEVICE_URL for the just-flashed Ultra 205 and rerun the Phase 17 HTTP helper."
      - "Create a sanitized target-lock.json from explicit input that records reachable target provenance without raw secrets."
  - truth: "Live evidence captures /, /assets/app.css.gz, representative missing static behavior, /recovery, API route coexistence, /api/ws, and /api/ws/live from the just-flashed device."
    status: failed
    reason: "No live route probes or WebSocket captures ran. All required route, header, body, curl-error, and frame artifacts are absent or explicitly blocked."
    artifacts:
      - path: "docs/parity/evidence/phase-17-live-http-api-and-static-evidence/http-static-api.md"
        issue: "All D-08 HTTP/static/API routes are blocked - missing DEVICE_URL with artifacts absent - not cited."
      - path: "docs/parity/evidence/phase-17-live-http-api-and-static-evidence/websocket.md"
        issue: "Both /api/ws/live and /api/ws frame artifacts are absent - not cited."
      - path: "docs/parity/evidence/phase-17-live-http-api-and-static-evidence/http-static-api/"
        issue: "Only http-static-api.log exists; no per-route headers, bodies, or curl-error artifacts exist."
    missing:
      - "Capture live HTTP status and redacted response summaries for /, /assets/app.css.gz, representative missing static behavior, /recovery, /api/system/info, unknown API behavior, /api/ws, and /api/ws/live."
      - "Capture bounded WebSocket evidence for /api/ws/live and /api/ws, or record route-specific no-upgrade and frame statuses from an explicit target run."
  - truth: "Evidence records exact commands, HTTP status and response summaries, relevant device logs, observed behavior, conclusion, and redaction review."
    status: partial
    reason: "Exact package, detector, flash-monitor, blocked-helper commands, serial logs, conclusions, and redaction review exist, but HTTP status/response summaries and WebSocket frame summaries from live device captures do not exist."
    artifacts:
      - path: "docs/parity/evidence/phase-17-live-http-api-and-static-evidence/summary.md"
        issue: "Contains exact commands and conclusions but records live route artifacts and WebSocket frame artifacts as absent - not cited."
      - path: "docs/parity/evidence/phase-17-live-http-api-and-static-evidence/http-static-api/http-static-api.log"
        issue: "Contains a blocked missing-DEVICE_URL transcript rather than live HTTP status or response summaries."
      - path: "docs/parity/evidence/phase-17-live-http-api-and-static-evidence/websocket/websocket-capture.log"
        issue: "Contains a not-run missing-target decision rather than live WebSocket capture output."
    missing:
      - "Record HTTP status, selected headers, redacted body summaries, curl errors, and conclusion from a reachable DEVICE_URL run."
      - "Record WebSocket open/frame/timeout statuses and redacted frame snippets from a reachable DEVICE_URL run."
---

# Phase 17: Live HTTP API And Static Evidence Verification Report

**Phase Goal:** The just-flashed Ultra 205 exposes live administration surfaces at an explicit `DEVICE_URL` with static asset, recovery page, API route, and WebSocket evidence.
**Verified:** 2026-07-02T03:50:19Z
**Status:** gaps_found
**Re-verification:** No - initial verification

## Evidence Reviewed

Loaded repo guidance from `AGENTS.md`, `AGENTS.bright-builds.md`, `standards/core/verification.md`, `standards/core/testing.md`, `standards/core/operability.md`, and `standards/languages/rust.md`. No project skill indexes were present under `.claude/skills` or `.agents/skills`.

Primary phase artifacts reviewed:

| Artifact | Result |
| --- | --- |
| `.planning/ROADMAP.md` Phase 17 | Requires explicit reachable `DEVICE_URL` live HTTP/static/recovery/API/WebSocket evidence. |
| `.planning/REQUIREMENTS.md` | `API-09`, `REL-01`, `REL-07`, and `EVD-05` trace Phase 17 while preserving below-verified live rows. |
| `17-01` through `17-05` plans and summaries | All loaded; plans allow blocked evidence when `DEVICE_URL` is unavailable, but do not override the roadmap goal. |
| `summary.md` | Records package/serial/redaction passed, but live HTTP and WebSocket blocked by missing `DEVICE_URL`. |
| `http-static-api.md` | Records `http_static_api_status: blocked`, `device_url_status: blocked`, and every route artifact `absent - not cited`. |
| `websocket.md` | Records `websocket_status: blocked`, no `/api/ws/live` frame artifact, and no `/api/ws` frame artifact. |
| `package-release-gate.md` and `serial-boot.md` | Package, detector, board `205`, selected port, source/reference commits, and flash-monitor identity are recorded. |
| `redaction-review.md` | `redaction_status: passed`; absent live artifacts are explicitly marked `absent - not cited`. |
| `docs/release/ultra-205.md` and `docs/parity/checklist.md` | Updated conservatively and keep affected live rows below `verified`. |

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
| --- | --- | --- | --- |
| 1 | Ultra 205 detector output, board `205`, selected port, source commit, reference commit, package manifest, and explicit reachable `DEVICE_URL` are recorded without network scanning or secrets. | PARTIAL | Package and serial identity pass, but `summary.md:12-14`, `http-static-api.md:3-7`, and `websocket.md:3-8` record missing `DEVICE_URL`; `target-lock.json` is absent. |
| 2 | Live evidence captures `/`, `/assets/app.css.gz`, representative missing static behavior, `/recovery`, API route coexistence, `/api/ws`, and `/api/ws/live` from the just-flashed device. | FAILED | `summary.md:63-70` marks target lock, HTTP headers/bodies/errors, and WebSocket frame outputs absent; `summary.md:98-107` marks every route blocked. |
| 3 | Evidence records exact commands, HTTP status and response summaries, relevant device logs, observed behavior, conclusion, and redaction review. | PARTIAL | Exact commands, logs, conclusions, and redaction review exist, but live HTTP status/response summaries and WebSocket frame summaries are absent (`summary.md:87-121`). |
| 4 | Release docs, parity checklist, and requirements traceability are updated without marking rows `verified` unless evidence criteria are met. | VERIFIED | `docs/release/ultra-205.md:30-59`, `docs/parity/checklist.md:90-94` and `137-142`, and `.planning/REQUIREMENTS.md:156-168` preserve blocked/below-verified status. |

**Score:** 1/4 truths verified

## Required Artifacts

| Artifact | Expected | Status | Details |
| --- | --- | --- | --- |
| `package-release-gate/bitaxe-ultra205-package.json` | Current package manifest | VERIFIED | Present; manifest source/reference commits match flash JSON. |
| `serial-boot/flash-command-evidence.json` | Wrapper-owned flash-monitor identity | VERIFIED | Present with board `205`, selected port, `trusted_output: true`, and matching commits. |
| `target-lock.json` | Sanitized explicit target lock from reachable `DEVICE_URL` | MISSING | Not created because no explicit origin-only `DEVICE_URL` was accepted. |
| `http-static-api.md` | Live HTTP/static/API route summary | HOLLOW | Present and substantive, but it records blocked/no-target evidence instead of live captures. |
| `http-static-api/*.headers.txt`, `*.body.txt`, `*.curl-error.txt` | Per-route HTTP artifacts | MISSING | No per-route files exist; only `http-static-api.log` exists. |
| `websocket/api-ws-live.txt` | Bounded `/api/ws/live` frame capture | MISSING | Artifact absent - not cited. |
| `websocket/api-ws.txt` | Bounded `/api/ws` raw-log frame capture | MISSING | Artifact absent - not cited. |
| `redaction-review.md` | Final redaction review | VERIFIED | `redaction_status: passed`; absent artifacts are tracked. |
| `docs/release/ultra-205.md`, `docs/parity/checklist.md`, `.planning/REQUIREMENTS.md` | Conservative traceability updates | VERIFIED | Updated without overclaiming live rows. |

## Key Link Verification

| From | To | Via | Status | Details |
| --- | --- | --- | --- | --- |
| Package manifest | Flash evidence JSON | Matching source/reference commits | WIRED | `identity_flow: passed`; board `205` and `trusted_output: true`. |
| HTTP helper | `target-lock.json` | Explicit `DEVICE_URL` target lock | NOT WIRED | Helper recorded missing target; `target-lock.json` absent. |
| HTTP evidence | Per-route artifacts | Live route probes | NOT WIRED | No live route probes ran; headers/bodies/errors absent. |
| WebSocket helper | `websocket/api-ws-live.txt` and `api-ws.txt` | Bounded frame capture | NOT WIRED | Capture commands documented but not run because no explicit target source existed. |
| Summary/redaction | Release docs/checklist/requirements | Exact artifact citations only | WIRED | `gsd-tools verify key-links` for plan `17-05` returned 3/3 verified. |

## Data-Flow Trace

| Artifact | Data Variable | Source | Produces Real Data | Status |
| --- | --- | --- | --- | --- |
| `serial-boot/flash-command-evidence.json` | source/reference identity | `bitaxe-ultra205-package.json` and wrapper serial output | Yes | FLOWING |
| `http-static-api.md` | live route statuses | `scripts/phase17-live-http-api-smoke.sh` | No live data; blocked before curl | HOLLOW |
| `websocket.md` | WebSocket frame statuses | `scripts/phase17-websocket-capture.mjs` | No live data; commands not run | HOLLOW |
| `docs/parity/checklist.md` | evidence status rows | Phase 17 summary ledger | Yes, conservative blocked status | VERIFIED |

## Behavioral Spot-Checks

| Behavior | Command | Result | Status |
| --- | --- | --- | --- |
| GSD lifecycle provenance | `gsd-tools verify lifecycle 17 --require-plans --raw` | `valid` | PASS |
| Helper syntax | `bash -n scripts/phase17-live-http-api-smoke.sh` and `node --check scripts/phase17-websocket-capture.mjs` | Passed | PASS |
| Helper tests | `bazel test //scripts:phase17_live_http_api_smoke_test` | Passed, cached | PASS |
| Package-to-flash identity | Node JSON comparison | `identity_flow: passed` | PASS |
| Parity checklist validity | `just parity` | `validation_errors: none` | PASS |
| Reference cleanliness | `just verify-reference` | `reference clean: c1915b0a63bfabebdb95a515cedfee05146c1d50` | PASS |
| Live `DEVICE_URL` route behavior | HTTP/WebSocket helpers with explicit target | Not run; no explicit `DEVICE_URL` was available in evidence | FAIL |

## Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
| --- | --- | --- | --- | --- |
| API-09 | 17-01, 17-03, 17-04, 17-05 | Static AxeOS assets and recovery page behavior remain compatible enough for administration. | PARTIAL | Implementation and docs are present, but live static/recovery/API/WebSocket captures are blocked. |
| REL-01 | 17-02, 17-03, 17-05 | Partition, filesystem, SPIFFS/static, and recovery assets support user-facing flows. | PARTIAL | Package and SPIFFS mount evidence pass; live `/`, `/assets/app.css.gz`, missing static, and `/recovery` are blocked. |
| REL-07 | 17-02, 17-05 | Build, flash, monitor, OTA, and recovery docs are sufficient and safe. | VERIFIED | Release docs cite exact artifacts and blocked states without overclaiming. |
| EVD-05 | 17-01 through 17-05 | Verification layers include tests, API compare, hardware smoke, and hardware evidence where appropriate. | PARTIAL | Helper tests, package, serial, redaction, parity, and reference checks pass; required live HTTP/WebSocket evidence layer is absent. |

No orphaned Phase 17 requirements were found beyond `API-09`, `REL-01`, `REL-07`, and `EVD-05`.

## Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
| --- | --- | --- | --- | --- |
| `docs/release/ultra-205.md` | 290 | `not available` | Info | Intentional operator copy for unsupported OTAWWW behavior, not a placeholder. |
| `scripts/phase17-websocket-capture.mjs` | 59 | `console.log` | Info | CLI output path, not an empty handler or stub. |

No blocker anti-patterns were found in the reviewed helper, evidence, release, checklist, or requirements files. The blocking issue is missing live evidence, not a code stub.

## Human Verification Required

No separate human verification item is useful until the gap is closed. The next verification step is concrete and evidence-based: provide an explicit origin-only `DEVICE_URL` for the just-flashed board and rerun the existing Phase 17 HTTP and WebSocket capture helpers.

## Gaps Summary

Phase 17 executed the safe blocked path correctly: it did not scan, infer, or leak a target, and it did not overclaim docs/checklist status. That is good evidence hygiene, but it is not goal achievement for this phase. The roadmap goal and success criteria require live administration surfaces at an explicit reachable `DEVICE_URL`, including static asset, recovery page, API route, `/api/ws`, and `/api/ws/live` evidence. Those captures are absent.

Concrete next action: rerun the Phase 17 evidence flow with an explicit origin-only `DEVICE_URL`, generate `target-lock.json`, capture the required HTTP status/header/body/error artifacts and bounded WebSocket outputs, update the summary/redaction/checklist/docs from those artifacts, then re-run verification.

_Verified: 2026-07-02T03:50:19Z_
_Verifier: the agent (gsd-verifier)_

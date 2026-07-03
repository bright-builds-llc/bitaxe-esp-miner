---
phase: "17-live-http-api-and-static-evidence"
plan: "06"
type: "execution-summary"
subsystem: "live HTTP evidence"
tags:
  - phase-17
  - evidence
  - http
  - static
  - parity
lifecycle_mode: "interactive"
phase_lifecycle_id: "17-2026-07-02T01-09-48"
generated_by: "gsd-execute-plan"
generated_at: "2026-07-03T06:51:40Z"
requirements_completed:
  - API-09
  - REL-01
  - REL-07
  - EVD-05
dependency_graph:
  requires:
    - "17-01"
    - "17-05"
  provides:
    - "Current package and flash identity refresh"
    - "Trusted USB-derived sanitized target lock"
    - "Live HTTP/static/API route artifacts"
    - "HTTP evidence redaction review updates"
  affects:
    - "docs/parity/evidence/phase-17-live-http-api-and-static-evidence"
    - "scripts/phase17-live-http-api-smoke.sh"
    - "scripts/phase17-live-http-api-smoke-test.sh"
tech_stack:
  added: []
  patterns:
    - "Trusted flash-monitor device_url source"
    - "Origin-only target validation"
    - "Redacted per-route HTTP evidence artifacts"
key_files:
  created:
    - "docs/parity/evidence/phase-17-live-http-api-and-static-evidence/target-lock.json"
    - "docs/parity/evidence/phase-17-live-http-api-and-static-evidence/http-static-api/*.headers.txt"
    - "docs/parity/evidence/phase-17-live-http-api-and-static-evidence/http-static-api/*.body.txt"
    - "docs/parity/evidence/phase-17-live-http-api-and-static-evidence/http-static-api/*.curl-error.txt"
    - ".planning/phases/17-live-http-api-and-static-evidence/17-06-SUMMARY.md"
  modified:
    - "docs/parity/evidence/phase-17-live-http-api-and-static-evidence/package-release-gate.md"
    - "docs/parity/evidence/phase-17-live-http-api-and-static-evidence/serial-boot.md"
    - "docs/parity/evidence/phase-17-live-http-api-and-static-evidence/http-static-api.md"
    - "docs/parity/evidence/phase-17-live-http-api-and-static-evidence/redaction-review.md"
    - "scripts/phase17-live-http-api-smoke.sh"
    - "scripts/phase17-live-http-api-smoke-test.sh"
decisions:
  - "Use the local developer-raw USB flash-monitor evidence only as an in-memory trusted device_url source for live HTTP probing."
  - "Commit only sanitized target and route artifacts; keep raw developer evidence under target/ and outside git."
  - "Treat WebSocket upgrade paths in this plan as HTTP route-coexistence checks only; frame proof belongs to plan 17-07."
metrics:
  completed_date: "2026-07-03"
  tasks_completed: 2
  files_changed: 43
  task_commits:
    - "958f800"
---

# Phase 17 Plan 06: Live HTTP Static API Evidence Summary

Plan 17-06 refreshed current package and flash identity evidence, accepted the trusted USB flash-monitor `device_url` source from local developer-raw evidence, wrote a sanitized target lock, and captured the live HTTP/static/API route artifacts for the just-flashed Ultra 205.

## Scope

The evidence chain was refreshed from the current package output and a new wrapper-owned flash-monitor run for board `205` on `/dev/cu.usbmodem1101`. The committed serial evidence is commit-redacted and records `redaction_mode: commit-redacted` plus `commit_ready: true`.

Live route probing used the helper's trusted USB flash-monitor source path, not network scanning, mDNS, ARP, router state, or inferred target discovery. The resulting committed `target-lock.json` records sanitized provenance with `device_url_source: usb_flash_monitor_log` and `network_scan: disabled`.

## Completed Tasks

| Task | Name | Commit | Files |
| --- | --- | --- | --- |
| 1 | Refresh package, release-gate, detector, and flash identity evidence | `958f800` | `package-release-gate.md`, `package-release-gate/*`, `serial-boot.md`, `serial-boot/*` |
| 2 | Capture live HTTP/static/API artifacts | `958f800` | `target-lock.json`, `http-static-api.md`, `http-static-api/*`, `redaction-review.md`, HTTP helper scripts |

## Evidence Closure

The HTTP ledger is `docs/parity/evidence/phase-17-live-http-api-and-static-evidence/http-static-api.md`.

It records these exact states:

| Surface | Final status |
| --- | --- |
| Package and release gate | passed |
| Detector and flash-monitor identity | passed |
| Target lock | present, sanitized, network scan disabled |
| `/` | passed |
| `/assets/app.css.gz` | passed |
| missing static redirect | passed |
| `/recovery` | passed |
| `/api/system/info` | passed |
| unknown `/api/*` | passed |
| `/api/ws` HTTP no-upgrade route check | passed as route-coexistence only |
| `/api/ws/live` HTTP no-upgrade route check | passed as route-coexistence only |
| `POST /api/system/OTA` empty request | passed as route-presence/validation-path only |
| `POST /api/system/OTAWWW` empty request | passed as fail-closed gap response only |

## Verification

Syntax and helper regression checks passed:

```bash
bash -n scripts/phase17-live-http-api-smoke.sh scripts/phase17-live-http-api-smoke-test.sh
bazel test //scripts:phase17_live_http_api_smoke_test
```

Live evidence checks passed against the generated artifacts:

```bash
node -e 'const fs=require("fs"); const manifest=JSON.parse(fs.readFileSync("docs/parity/evidence/phase-17-live-http-api-and-static-evidence/package-release-gate/bitaxe-ultra205-package.json","utf8")); const flash=JSON.parse(fs.readFileSync("target/phase17-gap-current-dev-raw/flash-command-evidence.json","utf8")); if (flash.command_kind !== "flash-monitor") throw new Error("wrong command"); if (flash.board !== "205") throw new Error("wrong board"); if (flash.trusted_output !== true) throw new Error("untrusted"); if (manifest.source_commit !== flash.firmware_commit) throw new Error("source mismatch"); if (manifest.reference_commit !== flash.reference_commit) throw new Error("reference mismatch"); console.log("phase17_gap_identity_check: passed");'
scripts/phase17-live-http-api-smoke.sh --use-flash-log-device-url --manifest docs/parity/evidence/phase-17-live-http-api-and-static-evidence/package-release-gate/bitaxe-ultra205-package.json --flash-evidence-json target/phase17-gap-current-dev-raw/flash-command-evidence.json --out-dir docs/parity/evidence/phase-17-live-http-api-and-static-evidence/http-static-api --target-lock-out docs/parity/evidence/phase-17-live-http-api-and-static-evidence/target-lock.json
```

Repository verification passed before the task commit:

```bash
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo build --all-targets --all-features
cargo test --all-features
just parity
just verify-reference
git diff --check
```

`just parity` reported `validation_errors: none`. `just verify-reference` reported `reference clean: c1915b0a63bfabebdb95a515cedfee05146c1d50`.

## Deviations from Plan

The written plan originally described an operator-supplied explicit `DEVICE_URL`. Execution used the newer trusted USB source added for Phase 17: `--use-flash-log-device-url` with local developer-raw flash evidence. This still preserved the no-scan boundary and wrote only a sanitized target lock to committed evidence.

The helper also needed a small compatibility fix: `target-lock.json` now falls back from `selected_port` to `port` when reading flash evidence. A regression test covers this shape.

## Auth Gates

None.

## Known Stubs

None.

## Threat Surface Scan

The helper reads a trusted local flash-monitor evidence file, validates the flash evidence contract, extracts exactly one origin-only `device_url` in memory, and redacts the committed target lock and route artifacts. It does not scan networks or persist raw target values.

## Deferred Issues

Valid OTA upload, invalid image rejection, reboot identity, selected partition, rollback, boot validation, whole-`www` OTAWWW update behavior, production mining, pool behavior, active safety telemetry, and soak evidence remain outside Phase 17.

## Self-Check: PASSED

Checked expected artifact existence, verified commit `958f800` exists in git history, and confirmed this summary uses standalone `---` only for the opening and closing frontmatter delimiters.

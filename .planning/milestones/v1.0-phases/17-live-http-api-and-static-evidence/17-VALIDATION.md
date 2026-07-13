---
phase: 17
slug: live-http-api-and-static-evidence
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-07-02
---

# Phase 17 - Validation Strategy

Per-phase validation contract for feedback sampling during execution.

***

## Test Infrastructure

| Property | Value |
| --- | --- |
| Framework | Bazel `sh_test` for shell helpers, Node syntax/runtime checks for WebSocket capture helpers, Rust/Cargo tests for any host-tool or parity changes, and guarded hardware/network evidence commands. |
| Config file | `scripts/BUILD.bazel`, `Cargo.toml`, per-crate `BUILD.bazel`, `Justfile`, and `.planning/phases/17-live-http-api-and-static-evidence/17-RESEARCH.md`. |
| Quick run command | `bash -n scripts/phase17-live-http-api-smoke.sh && node --check scripts/phase17-websocket-capture.mjs && bazel test //scripts:phase17_live_http_api_smoke_test` after Wave 0 creates those files. |
| Full suite command | `just test && just parity && just verify-reference` plus the documented Phase 17 hardware/network evidence commands when `DEVICE_URL` is explicitly provided. |
| Estimated runtime | Quick helper feedback should stay under 60 seconds once implemented; full suite and live hardware/network evidence are environment-dependent. |

***

## Sampling Rate

- After every task commit: run the narrow automated command named in that task's acceptance criteria.
- After every plan wave: run the helper tests for changed scripts and `just parity` when checklist, requirements, release docs, or evidence ledgers change.
- Before phase verification: run `just package`, `just detect-ultra205`, the explicit `DEVICE_URL` HTTP/static/API/WebSocket evidence path, redaction review, `just parity`, `just verify-reference`, and lifecycle validation.
- Max feedback latency: no three consecutive implementation tasks may proceed without automated feedback or a recorded hardware/network blocker.

***

## Per-Requirement Verification Map

| Requirement | Behavior | Test Type | Automated Command | Manual Or Hardware Evidence | Status |
| --- | --- | --- | --- | --- | --- |
| API-09 | Explicit target validation, bounded route coverage, static/recovery markers, API coexistence, and split WebSocket proof are captured without scanning or overclaiming. | shell helper tests plus live hardware/network smoke | `bash -n scripts/phase17-live-http-api-smoke.sh && bazel test //scripts:phase17_live_http_api_smoke_test` | With a just-flashed Ultra 205 and explicit origin-only `DEVICE_URL`, capture `/`, `/assets/app.css.gz`, missing static redirect, `/recovery`, `/api/system/info`, unknown `/api/*`, `/api/ws`, `/api/ws/live`, `POST /api/system/OTA`, and `POST /api/system/OTAWWW`. | pending |
| REL-01 | Live HTTP/static/recovery evidence is tied to current package, flash, board, port, source commit, reference commit, and package manifest identity. | workflow plus hardware smoke | `just package && just detect-ultra205` | Run `just flash-monitor board=205 port=<port> manifest=<manifest> evidence-dir=<dir>` before live probes, then cite that package/flash identity in the Phase 17 ledger. | pending |
| REL-07 | Release docs describe exact commands, artifacts, redaction status, recovery boundaries, and non-claims without promoting unsupported OTA or WebSocket behavior. | docs/parity review | `just parity && just verify-reference` | Review `docs/release/ultra-205.md`, the parity checklist, requirements traceability, and Phase 17 evidence ledger for exact claim boundaries. | pending |
| EVD-05 | Evidence layers include helper tests, package/flash identity, live HTTP/static/API/WebSocket artifacts, detector output, redaction review, parity, and reference cleanliness. | aggregate workflow | `just test && just parity && just verify-reference` | Commit redacted micro-artifacts under `docs/parity/evidence/phase-17-live-http-api-and-static-evidence/` or record explicit blocked/pending evidence when `DEVICE_URL` or frame data is unavailable. | pending |

***

## Wave 0 Requirements

- [ ] `scripts/phase17-live-http-api-smoke.sh` exists and enforces origin-only `DEVICE_URL`, package/flash identity checks, stale-evidence blocking, D-08 route coverage, route-specific markers, and redacted artifacts.
- [ ] `scripts/phase17-live-http-api-smoke-test.sh` covers missing target, invalid scheme, userinfo/path/query/fragment rejection, successful fake route coverage, no-upgrade WebSocket non-overclaim, stale package/flash blocking, and redaction.
- [ ] `scripts/phase17-websocket-capture.mjs` exists with an allowlisted path set, bounded duration, bounded frame count, redacted output, and separate handling for `/api/ws/live` versus `/api/ws`.
- [ ] `scripts/BUILD.bazel` exposes the Phase 17 helper tests.
- [ ] `docs/parity/evidence/phase-17-live-http-api-and-static-evidence/redaction-review.md` exists before live artifacts are cited.

***

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
| --- | --- | --- | --- |
| Ultra 205 USB detector and board-info | REL-01, EVD-05 | Requires connected physical Ultra 205 over USB. | Run `just detect-ultra205`; continue only when exactly one likely ESP USB serial port is found and board-info succeeds for board `205`. |
| Explicit live `DEVICE_URL` route probes | API-09, REL-01, EVD-05 | Requires an operator-provided reachable origin for the just-flashed device; scanning is forbidden. | Provide `--device-url` or `DEVICE_URL` as an origin-only URL, run the Phase 17 route helper, and record blocked evidence instead of probing when the target is missing, invalid, unreachable, or wrong-device. |
| WebSocket frame capture | API-09, EVD-05 | Requires live upgrade behavior and may time out without raw log frames. | Capture `/api/ws/live` connect or cadence frames and `/api/ws` log frames with bounded duration and redaction; if `/api/ws` opens but no frame arrives, record open/timeout and keep raw-log frame evidence pending. |
| Redaction review of live artifacts | REL-07, EVD-05 | Requires reviewing generated logs, headers, body snippets, and frame snippets for secrets and private endpoints. | Review every cited artifact and mark absent artifacts explicitly before docs/checklist promotion. |

***

## Security Threat Coverage

| Threat Ref | Surface | Secure Behavior | Validation |
| --- | --- | --- | --- |
| T-17-target | Live target selection | Helpers accept only explicit origin-only `DEVICE_URL` input and never infer, scan, or reuse private observations as discovery. | Helper tests for invalid targets plus live blocked evidence when no explicit target exists. |
| T-17-identity | Package and flashed-device identity | Live claims require current source commit, reference commit, package manifest, board `205`, selected port, detector output, and flash/serial evidence. | Package/flash identity checks before HTTP/WebSocket probes and ledger citations. |
| T-17-redaction | Evidence artifacts | Evidence must not expose Wi-Fi credentials, pool credentials, API tokens, private endpoints, MACs, NVS secrets, or unredacted device URLs. | Redacted snippets, redaction scan/review, and artifact matrix before docs/checklist promotion. |
| T-17-overclaim | OTA and WebSocket route evidence | Route presence, no-upgrade coexistence, WebSocket frame evidence, valid OTA, invalid OTA, reboot, rollback, and boot-validation claims remain separate. | Checklist/docs review and helper output categories that keep unsupported claims pending. |

***

## Validation Sign-Off

- [ ] All tasks have automated verification or a documented hardware/network/manual blocker.
- [ ] Sampling continuity: no three consecutive implementation tasks without automated feedback or explicit blocker evidence.
- [ ] Wave 0 covers all missing helper, test, Bazel, and redaction-review files before live claims are promoted.
- [ ] No watch-mode flags in verification commands.
- [ ] Feedback latency is bounded by targeted helper tests after each changed surface.
- [ ] `nyquist_compliant: true` is set only after plans map every Phase 17 requirement to executable checks and Wave 0 gaps are addressed or assigned.

Approval: pending

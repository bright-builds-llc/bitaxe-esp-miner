# Phase 17: Live HTTP API And Static Evidence - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution
> agents. Decisions are captured in CONTEXT.md; this log preserves the
> alternatives considered.

**Date:** 2026-07-02T01:09:48.162Z
**Phase:** 17-live-http-api-and-static-evidence
**Mode:** Interactive fallback with advisor recommendations
**Areas discussed:** Target gate and current-device identity, live probe set and
pass criteria, WebSocket proof depth, evidence artifacts and redaction,
checklist and docs promotion boundary

The Codex interactive question tool was unavailable in this session. Per the
`gsd-discuss-phase` fallback, the workflow selected the recommended defaults and
used advisor research for all core gray areas.

## Target Gate And Current-Device Identity

| Option | Description | Selected |
| --- | --- | --- |
| One-shot explicit `DEVICE_URL` gate | Matches Phase 16 and keeps the endpoint out of persisted files, but stale environment values can target the wrong device. | No |
| Phase target-lock manifest | Binds normalized explicit target input to detector port, board-info, package manifest, flash evidence, and commits. | Yes |
| Operator-confirmed external source label | Allows bench provenance from serial, AP UI, router UI, or notes, but remains manual and not discovery. | Partial |
| Commit-bound live identity gate | Adds device sanity checks through live API/log markers, but increases redaction and false-negative risk. | Partial |

**Selected answer:** Use explicit `--device-url`, explicit `DEVICE_URL`, or a
phase target-lock manifest created from explicit input. Operator observations
can be provenance only. Require detector, board-info, current package/flash
identity, bounded reachability, and `/api/system/info` sanity before promotion.

**Notes:** Missing, invalid, unreachable, or wrong-device targets produce
blocked evidence. No scans or inferred targets.

## Live Probe Set And Pass Criteria

| Option | Description | Selected |
| --- | --- | --- |
| Reuse Phase 16 route gate unchanged | Fastest path and already tested, but CSS headers and WebSocket depth are not strong enough for every claim. | No |
| Phase 17 hardened route gate | Same bounded route set with explicit status, header, body, and route-presence expectations. | Yes |
| Route gate plus WebSocket frame capture | Adds real frame proof where WebSocket claims require it. | Yes |
| Manifest-derived full route sweep | Broad route drift check, but risks side effects and scope creep. | No |

**Selected answer:** Harden the Phase 16 helper route set for Phase 17. Probe
`/`, `/assets/app.css.gz`, a missing static route, `/recovery`,
`/api/system/info`, an unknown `/api/*`, `/api/ws`, `/api/ws/live`,
`POST /api/system/OTA` as route-presence only, and `POST /api/system/OTAWWW`
as fail-closed gap response.

**Notes:** Keep valid OTA, rollback, erase, failed-update, interrupted-update,
and whole-OTAWWW behavior out of Phase 17.

## WebSocket Proof Depth

| Option | Description | Selected |
| --- | --- | --- |
| No-upgrade route proof only | Proves route precedence and static fallback avoidance, but not WebSocket framing. | No |
| WebSocket upgrade/frame capture only | Proves frame behavior, but does not separately prove route precedence. | No |
| Both no-upgrade route proof and bounded WebSocket capture | Separates route coexistence from frame-level proof and prevents overclaiming. | Yes |

**Selected answer:** Require both no-upgrade route proof and bounded real
WebSocket capture before verified WebSocket claims.

**Notes:** `/api/ws/live` can pass with a redacted connect or cadence frame.
`/api/ws` requires a redacted log frame or remains pending if no log frame is
observed before timeout.

## Evidence Artifacts And Redaction

| Option | Description | Selected |
| --- | --- | --- |
| Commit redacted summaries only | Lowest leak surface, but weak replay value. | No |
| Commit redacted per-route artifacts plus summary | Closest to Phase 16 and gives auditable proof after sanitization. | Yes |
| Split public ledger plus quarantined raw pack | Keeps raw captures out of git, but future reviewers may not have replayable evidence. | Optional |
| Typed allowlist extraction | Stores only status, whitelisted fields, metadata, and sanitized snippets. | Yes |

**Selected answer:** Commit redacted micro-artifacts plus a summary ledger.
Use typed allowlisting for API responses and WebSocket frames where practical.
Mark absent artifacts as `absent - not cited`.

**Notes:** Redaction must cover `DEVICE_URL`, private endpoints, IP addresses,
MAC addresses, Wi-Fi credentials, pool credentials, worker secrets, API tokens,
NVS secret values, and local terminal secrets. Avoid standalone body `---`
separators in parsed Markdown artifacts.

## Checklist And Docs Promotion Boundary

| Option | Description | Selected |
| --- | --- | --- |
| Guard-aligned live-surface promotion | Promote only live HTTP/API/static/WebSocket claims backed by artifacts that satisfy existing guards. | Yes |
| Evidence-only docs update | Safest when captures are incomplete, but leaves completed live evidence understated. | Conditional |
| Strict recovery-coupled promotion | Treats `/recovery` page load as insufficient for any recovery-related row, even when FS-001 terms are met. | No |

**Selected answer:** Promote only exact live-surface claims supported by Phase 17
artifacts. Keep OTA, rollback, erase, failed-update, interrupted-update, and
OTAWWW whole-update rows below verified.

**Notes:** `FS-001` requires live static, `/assets/app.css.gz`, missing static
redirect, `/recovery`, hardware evidence, and no blocker terms. WebSocket rows
need real frame evidence, not just no-upgrade route responses.

## the agent's Discretion

- Exact Phase 17 helper names and whether they wrap or copy Phase 16 helpers.
- Target-lock manifest schema and stale-manifest rules.
- Timeout values and fake-device test fixture details.
- Whether bounded WebSocket capture uses Node, Rust host tooling, or another
  repo-owned client.

## Deferred Ideas

- Valid firmware OTA, invalid OTA rejection, rollback, and boot validation.
- Recovery fault injection, failed update, large erase, interrupted update, and
  whole-`www` OTAWWW behavior.
- Active safety telemetry and live production mining/soak evidence.
- Network discovery, mDNS/router scraping, non-205 boards, Angular UI rewrite,
  Stratum v2, BAP, and production mining tuning.

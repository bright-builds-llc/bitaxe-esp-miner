# Phase 9: Flash-Monitor Evidence Wrapper Hardening - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md - this log preserves the alternatives considered.

**Date:** 2026-06-29T13:20:44.365Z
**Phase:** 09-flash-monitor-evidence-wrapper-hardening
**Mode:** Yolo
**Areas discussed:** Noninteractive Monitor Capture, Evidence Record Contract, Monitor Startup Failure Guidance, Hardware Verification And Checklist Promotion Boundary

## Noninteractive Monitor Capture

| Option | Description | Selected |
| --- | --- | --- |
| Evidence-only `espflash monitor --chip esp32s3 --port <port> --non-interactive` with wrapper-owned capture window | Matches the Phase 8 fallback, uses espflash's noninteractive monitor mode, and preserves normal interactive monitor behavior. | Yes |
| Single espflash invocation with flash/write-bin `--monitor --non-interactive` | Lets espflash own the transition from flash to monitor, but mixes flash and monitor error attribution. | No |
| PTY-backed interactive monitor capture | Preserves keyboard reset behavior but keeps the failure-prone input-reader dependency in the evidence path. | No |
| Custom serial capture backend after espflash flash | Fully programmatic capture, but reimplements behavior that espflash already owns. | No |

**User's choice:** Yolo selected the recommended evidence-only noninteractive espflash monitor path.
**Notes:** This directly addresses the Phase 8 `Failed to initialize input reader` failure while keeping raw espflash fallback out of the accepted evidence path.

## Evidence Record Contract

| Option | Description | Selected |
| --- | --- | --- |
| Enrich existing `flash-command-evidence.json` | Machine-readable, testable, compatible with existing package manifest and parity tooling patterns. | Yes |
| Add wrapper-written Markdown ledger | Easy to review but weaker as canonical machine evidence and prone to field drift. | No |
| Enriched JSON plus generated Markdown summary from the same record | Good long-term shape when human-readable generated evidence is worthwhile, but more implementation surface. | Possible follow-up |
| Keep JSON minimal and document manual evidence | Small code change, but repeats the Phase 8 manual transcription burden. | No |

**User's choice:** Yolo selected enriched JSON as the source of truth, with generated Markdown allowed only if it comes from the same structured record.
**Notes:** Required fields include board, port, commits, manifest, exact commands, log path, capture status, and conclusion.

## Monitor Startup Failure Guidance

| Option | Description | Selected |
| --- | --- | --- |
| Evidence mode defaults to noninteractive monitor | Makes trusted evidence capture repo-owned and avoids interactive input-reader startup failures. | Yes |
| Explicit `monitor-mode=interactive|noninteractive` plus recovery banner | Transparent but adds surface area and user mode confusion. | No |
| Explicit automatic retry after input-reader failure | Low friction but risks hiding degraded or partial evidence unless carefully labeled. | No |
| PTY/manual monitor guidance only | Useful as diagnostics, but not sufficient as the Phase 9 evidence fix. | Secondary only |

**User's choice:** Yolo selected fail-closed noninteractive evidence mode with manual/PTTY monitor only as diagnostic guidance.
**Notes:** Recovery guidance should point to `just detect-ultra205` and rerunning the wrapper command, not raw fallback commands as the primary path.

## Hardware Verification And Checklist Promotion Boundary

| Option | Description | Selected |
| --- | --- | --- |
| Evidence-only Phase 9 addendum | Lowest overclaim risk but may leave checklist and release docs stale. | No |
| Refresh workflow evidence only | Replaces raw-monitor fallback as proof for wrapper capture while keeping HTTP/OTA/recovery unpromoted. | Yes |
| Add dedicated checklist row for wrapper evidence capture | Clear but may duplicate `WF-005` and cause checklist churn. | No |

**User's choice:** Yolo selected refreshing workflow evidence and release caveats after fresh wrapper-based Ultra 205 evidence.
**Notes:** Phase 9 evidence must not promote HTTP/static/recovery/OTA/rollback release rows to verified.

## the agent's Discretion

- Exact CLI flag names and capture timeout defaults.
- Exact enriched JSON schema field names.
- Whether to generate a Markdown summary from the evidence JSON.
- Exact helper boundaries in `tools/flash/src/main.rs`.

## Deferred Ideas

- Custom serial backend if espflash noninteractive monitor cannot capture reliable logs.
- New dedicated checklist row if `WF-005` cannot express wrapper evidence clearly.
- Live HTTP/static/recovery/OTA/rollback evidence for later release evidence phases.

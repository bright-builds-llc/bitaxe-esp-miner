# Phase 15: BM1366 Mining Evidence Completion - Discussion Log

> Audit trail only. Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md - this log preserves the alternatives considered.

**Date:** 2026-07-01T02:14:24.605Z
**Phase:** 15-bm1366-mining-evidence-completion
**Mode:** Yolo
**Areas discussed:** Trusted BM1366 initialization evidence, Diagnostic work/result evidence, Controlled mining smoke/soak, Checklist promotion and redaction

## Trusted BM1366 Initialization Evidence

| Option | Description | Selected |
| --- | --- | --- |
| Packaged diagnostic factory image through the existing wrapper | Fixes Phase 12 root cause, keeps SPIFFS/package/trusted boot markers, preserves compile-time chip-detect acknowledgement, and avoids production mining routes. | yes |
| Runtime allow-manifest BM1366 probe in the standard package | Uses release-like package identity and can extend to work/result or mining smoke, but creates a larger firmware surface. | fallback |
| Diagnostic-only trust profile for ELF captures | Smallest change, but weakens wrapper trust semantics and repeats the Phase 12 audit concern. | no |
| Evidence-only deferral with trusted before/after safe boot | Operationally safest, but does not close the trusted BM1366 initialization gap. | fallback only |

**Captured decision:** Use a package-backed diagnostic path as the preferred fix for Phase 12's untrusted chip-detect capture. Do not lower wrapper trust requirements to promote verified claims.

## Diagnostic BM1366 Work-Send/Result-Receive Evidence

| Option | Description | Selected |
| --- | --- | --- |
| Typed firmware diagnostic over USB console | Avoids credentials and raw ASIC serial writes while exercising repo-owned BM1366 init/send/result APIs on real Ultra 205 hardware. | yes |
| Diagnostic HTTP/admin endpoint in a gated firmware build | Offers structured JSON evidence but requires network setup and stronger exposure controls. | fallback |
| Local Stratum harness with controlled mining smoke | Exercises mining-loop integration, but belongs after typed diagnostic evidence passes. | later tier |
| Host-only typed replay/golden tests | Deterministic regression guard but not hardware evidence. | support only |

**Captured decision:** Use a typed firmware diagnostic over USB console as the primary work-send/result-receive evidence path. Keep host-only tests as supporting regression coverage, not live proof.

## Controlled Mining Smoke Or Bounded Soak

| Option | Description | Selected |
| --- | --- | --- |
| Local deterministic Stratum harness / controlled no-share | Avoids secrets, can prove lifecycle and controlled no-share/work pipeline behavior, and fits blocked-prerequisite cases. | yes when live prerequisites are missing |
| Live pool micro-smoke | Proves real pool lifecycle and authentic share behavior but needs disposable credentials, network, DEVICE_URL, safety gates, and redaction. | yes only when all gates pass |
| Bounded live soak | Strongest stability evidence, but inappropriate before live smoke and safety/stop conditions pass. | later tier |
| Layered evidence ladder with blocked/pending fallback | Preserves exact-claim promotion and prevents assuming hardware/pool/DEVICE_URL availability. | yes |

**Captured decision:** Use a layered evidence ladder. Controlled no-share evidence is acceptable when labelled precisely; live pool micro-smoke and soak are allowed only behind explicit safety, recovery, telemetry, and redaction gates.

## Checklist Promotion, Parity Guard, And Redaction Boundaries

| Option | Description | Selected |
| --- | --- | --- |
| Evidence-tiered exact-claim promotion | Matches Phase 12/14 patterns and supports partial progress tied to detector-gated artifacts and redaction review. | yes |
| Evidence-only ledger with no checklist promotion | Lowest overclaim risk and appropriate when evidence is partial or redaction is uncertain. | fallback |
| Tool-enforced promotion guard | Makes `just parity` enforce prerequisites before verified claims, useful when rows are promoted. | conditional |

**Captured decision:** Use evidence-tiered exact-claim promotion by default, extend parity guards only when necessary for verified promotions, and fall back to evidence-only notes when proof is incomplete.

## the agent's Discretion

- Exact plan count and wave split.
- Diagnostic package target or wrapper implementation details.
- Evidence schema and marker naming.
- Whether additional parity checks live in `tools/parity`, scripts, or a small host tool.

## Deferred Ideas

- Phase 16 owns same-commit release HTTP/static/recovery/OTA and destructive recovery evidence.
- Production mining tuning, unbounded stress, Stratum v2, BAP, non-205 boards, and broad display/input parity remain out of scope.

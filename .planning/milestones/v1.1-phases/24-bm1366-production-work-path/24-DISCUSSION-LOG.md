# Phase 24: BM1366 Production Work Path - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md -- this log preserves the alternatives considered.

**Date:** 2026-07-05T00:27:27.205Z
**Phase:** 24-BM1366 Production Work Path
**Mode:** Yolo
**Areas discussed:** Production vs Diagnostic ASIC Modes, Pool-Derived Work Dispatch, Result Correlation and Share Claim Gate, Fail-Closed Errors and Redaction, Verification and Evidence Semantics

## Production vs Diagnostic ASIC Modes

| Option | Description | Selected |
|--------|-------------|----------|
| Hard type-level split | Keep diagnostic chip/work modes distinct from trusted production initialization/work/result states. | Yes |
| Reuse diagnostic mode with flags | Extend diagnostic work paths into production behavior through configuration flags. | No |
| Shell-owned distinction | Let firmware scripts/logs distinguish diagnostic from production behavior. | No |

**User's choice:** Auto-selected hard type-level split.
**Notes:** Prior Phases 3, 12, 15, 21, 22, and 23 require exact non-claim boundaries and typed fail-closed behavior. Diagnostic evidence remains useful readiness evidence but cannot imply production work parity.

## Pool-Derived Work Dispatch

| Option | Description | Selected |
|--------|-------------|----------|
| Active pool-job registry | Bind BM1366 work to active Stratum job, extranonce, difficulty/target, session generation, and invalidation state. | Yes |
| Fire-and-forget work dispatch | Send BM1366 work without a durable active-work correlation record. | No |
| Firmware-only tracking | Track sent jobs only in the ESP-IDF shell. | No |

**User's choice:** Auto-selected active pool-job registry.
**Notes:** `ASIC-10` requires pool-derived work, job/extranonce/difficulty context, and stale-work invalidation on clean-jobs or reconnect. The pure core should own these invariants so tests can prove behavior without hardware.

## Result Correlation and Share Claim Gate

| Option | Description | Selected |
|--------|-------------|----------|
| Correlate before claim | Map every nonce/result observation to active, non-stale work before preparing any submit intent or share claim. | Yes |
| Submit best-effort observations | Treat parsed BM1366 results as submit-worthy unless later pool response rejects them. | No |
| Defer all correlation to Phase 25 | Keep Phase 24 limited to dispatch and leave result mapping to the live runtime phase. | No |

**User's choice:** Auto-selected correlate before claim.
**Notes:** `ASIC-11` makes correlation the key Phase 24 behavior. Phase 25 owns live socket response classification, but Phase 24 should produce the redaction-safe submit-intent gate that Phase 25 can consume.

## Fail-Closed Errors and Redaction

| Option | Description | Selected |
|--------|-------------|----------|
| Typed redaction-safe blocker reasons | Represent init, UART, reset, timeout, malformed result, stale work, and correlation failures as stable safe-to-commit reasons. | Yes |
| Raw diagnostic artifacts | Preserve raw frames and payloads in evidence for later troubleshooting. | No |
| Generic failure status | Collapse production failures into a small number of public "not ready" states. | No |

**User's choice:** Auto-selected typed redaction-safe blocker reasons.
**Notes:** Phase 22 established stable blocker semantics and Phase 23 established committed evidence redaction. Phase 24 should not leak raw BM1366 frames, Stratum targets, extranonces, share payloads, pool values, device URLs, IPs, MACs, Wi-Fi values, tokens, or NVS secrets.

## Verification and Evidence Semantics

| Option | Description | Selected |
|--------|-------------|----------|
| Exact-claim test and evidence ladder | Use pure tests for work/correlation invariants and detector-gated evidence only for hardware-sensitive promotion. | Yes |
| Implementation-only promotion | Mark Phase 24 ASIC rows verified once code and unit tests exist. | No |
| Hardware-first broad promotion | Run one live hardware path and promote all related ASIC/mining claims together. | No |

**User's choice:** Auto-selected exact-claim test and evidence ladder.
**Notes:** `ASIC-09` through `ASIC-12` can advance only to the level supported by deterministic tests and any detector-gated, redaction-reviewed evidence actually produced. Phase 25 and Phase 26 non-claims stay visible.

## Claude's Discretion

Claude may choose exact type, module, fixture, plan, and artifact names while preserving the locked decisions in `24-CONTEXT.md`.

## Deferred Ideas

- Real Stratum v1 socket lifecycle, deterministic fake-pool production tests, live accepted/rejected pool response classification, watchdog under bounded production load, and bounded safe-stop runtime proof remain Phase 25 scope.
- API/WebSocket/statistics/scoreboard promotion and final v1.1 parity closure remain Phase 26 scope.
- Non-205 boards, non-BM1366 ASIC families, full active safety closure, OTA/recovery, display/input, BAP, Stratum v2, and unbounded stress mining remain future scope.

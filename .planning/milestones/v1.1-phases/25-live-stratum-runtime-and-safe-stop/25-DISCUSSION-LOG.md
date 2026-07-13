# Phase 25: Live Stratum Runtime And Safe Stop - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in `25-CONTEXT.md`; this log preserves the yolo recommendation pass and alternatives considered.

**Date:** 2026-07-05T01:55:45.817Z
**Phase:** 25-live-stratum-runtime-and-safe-stop
**Mode:** Yolo
**Areas discussed:** Real Stratum runtime boundary, submit response classification, deterministic fake-pool coverage, bounded safe stop, watchdog responsiveness, evidence and allow-manifest integration

## Real Stratum Runtime Boundary

| Option | Description | Selected |
| --- | --- | --- |
| Firmware socket adapter around pure Stratum core | ESP-IDF sockets and task yielding in firmware; parsing/state/classification in `crates/bitaxe-stratum`. | yes |
| Put socket and protocol behavior directly in firmware shell | Faster initial wiring but mixes I/O and business state. | no |
| Keep controlled transcript runtime for now | Preserves existing evidence mode but cannot satisfy STR-08. | no |

**Chosen answer:** Firmware socket adapter around the existing pure Stratum core.
**Notes:** This carries forward the functional core / imperative shell split from Phase 24 and keeps raw protocol handling out of firmware logs.

## Submit Response Classification

| Option | Description | Selected |
| --- | --- | --- |
| Classify only responses tied to live ASIC-derived submit intents | Accepted/rejected claims require a live submit intent and observed pool response. | yes |
| Treat any `mining.submit` response as share evidence | Simpler but risks stale, synthetic, or unrelated response overclaims. | no |
| Defer all submit classification to Phase 26 | Avoids Phase 25 complexity but fails STR-09 ownership. | no |

**Chosen answer:** Classify accepted/rejected/blocked outcomes only when tied to a live ASIC-derived submit intent from the active production work registry.
**Notes:** Fake-pool and implementation evidence remain below live accepted/rejected proof.

## Deterministic Fake-Pool Coverage

| Option | Description | Selected |
| --- | --- | --- |
| Extend the existing fake-pool harness | Reuses `crates/bitaxe-stratum/src/v1/fake_pool.rs` for subscribe, authorize, notify, clean-jobs, submit, reconnect, fallback, and errors. | yes |
| Create a second production-only simulator | Could isolate Phase 25 tests but duplicates protocol behavior. | no |
| Rely on live pool testing only | Produces useful evidence but leaves deterministic STR-11 behavior undercovered. | no |

**Chosen answer:** Extend the existing deterministic fake-pool/fixture harness.
**Notes:** Tests should cover accepted, rejected, blocked, timeout, reconnect, clean-jobs invalidation, and stale work rejection paths.

## Bounded Safe Stop

| Option | Description | Selected |
| --- | --- | --- |
| Define explicit runtime postconditions | Socket stopped, queues drained/invalidated, mining disabled, hardware control disabled, submission blocked, post-stop state refreshed. | yes |
| Stop only the socket loop | Narrower but leaves stale active work or API state risk. | no |
| Treat safe stop as evidence-script cleanup only | Useful for operator runs but insufficient for firmware runtime correctness. | no |

**Chosen answer:** Define safe stop as explicit runtime postconditions shared by normal stop, reconnect/fallback exhaustion, prerequisite failure, cancellation, and verification cleanup.
**Notes:** Committed evidence must use redaction-safe status labels.

## Watchdog Responsiveness

| Option | Description | Selected |
| --- | --- | --- |
| Use explicit checkpoints around blocking-prone work | Socket, ASIC, API/WebSocket, and evidence-capture loops remain watchdog-friendly. | yes |
| Rely on existing task scheduling only | Less code but weak evidence for SAFE-13. | no |
| Push watchdog proof entirely to hardware testing | Hardware evidence is valuable but pure budget tests should catch regressions earlier. | no |

**Chosen answer:** Combine pure watchdog budget/checkpoint tests with redacted runtime markers and hardware/workflow evidence when available.
**Notes:** If no detector-gated proof exists, SAFE-13 must remain below hardware-verified status.

## Evidence And Allow-Manifest Integration

| Option | Description | Selected |
| --- | --- | --- |
| Add a deliberate Phase 25 live Stratum evidence tier | Keeps detector, redaction, safe-state, and verified-row guards intact. | yes |
| Bypass allow-manifest restrictions for live Stratum commands | Faster hardware iteration but violates evidence governance. | no |
| Keep Phase 23 slots blocked until Phase 26 | Too conservative for Phase 25 safe-stop/share-outcome ownership. | no |

**Chosen answer:** Update Phase 23 evidence-root and parity/allow tooling deliberately for Phase 25 artifacts before promoting hardware evidence.
**Notes:** Any blocked detection, prerequisite, credential, socket, or share outcome path should be recorded as an exact blocker, not inferred as success.

## Claude's Discretion

- Exact module names, adapter trait names, timeout budgets, retry limits, fake-pool fixture format, evidence filenames, redaction labels, and plan count.
- Choices must preserve functional core / imperative shell structure, typed fail-closed behavior, Ultra 205 detector gating, repo-owned verification, and conservative parity semantics.

## Deferred Ideas

- Phase 26 owns full API, WebSocket, statistics, scoreboard, and final parity projection from v1.1 runtime events.
- Future milestones own full active voltage/fan/thermal fault-stimulus, self-test hardware closure, OTA/recovery destructive or fault-injection evidence, runtime display/input, BAP, Stratum v2, non-205 boards, other ASIC families, and unbounded stress mining.

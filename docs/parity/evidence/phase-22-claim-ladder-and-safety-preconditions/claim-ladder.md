# Phase 22 Claim Ladder

This ladder defines the exact evidence claims an Ultra 205 operator may make while moving from v1.0 controlled no-share mining evidence toward v1.1 trusted production mining. Lower tiers can support readiness, implementation confidence, or deferred tracking, but they do not prove higher-tier live production mining outcomes.

## Allowed Claims

### `version_1_0_controlled_no_share`

- **Allowed claim:** Phase 21 closed with `approved_controlled_no_share_soak` evidence for the Ultra 205 controlled no-share path.
- **Evidence basis:** `docs/parity/evidence/phase-21-live-mining-and-soak-evidence/summary.md` and `.planning/phases/21-live-mining-and-soak-evidence/21-VERIFICATION.md`.
- **Blocked claim:** This tier does not prove accepted shares, rejected shares, unbounded production mining, successful live ASIC nonce/result parsing, or full production mining.
- **Evidence required for promotion:** Later evidence must include detector-gated, redacted live production mining artifacts that observe parsed pool responses to live ASIC-derived work.

### `version_1_1_prerequisite_readiness`

- **Allowed claim:** The firmware and tooling can define or enforce prerequisite readiness for Ultra 205 production mining before BM1366 work dispatch.
- **Evidence basis:** Phase 22 typed safety precondition tests, blocker reason evidence, parity checks, and exact checklist language.
- **Blocked claim:** This tier does not prove accepted shares, rejected shares, unbounded production mining, full active safety closure, active voltage/fan/fault hardware behavior, thermal fault stimulus, or self-test hardware closure.
- **Evidence required for promotion:** A later phase must collect fresh or explicitly bounded power, thermal, fan, voltage, and safety observations in a detector-gated production mining session.

### `version_1_1_live_socket_runtime`

- **Allowed claim:** The Ultra 205 production runtime has live Stratum v1 socket lifecycle evidence for connect, subscribe, authorize, notify, submit attempts, reconnect handling, safe stop, and watchdog responsiveness.
- **Evidence basis:** Deferred Phase 25 live socket/runtime evidence under the v1.1 evidence root.
- **Blocked claim:** Socket/runtime evidence alone does not prove accepted shares, rejected shares, live ASIC-derived share outcomes, unbounded production mining, full active safety closure, non-205 board support, Stratum v2, OTA/recovery trust, runtime display/input parity, or BAP behavior.
- **Evidence required for promotion:** A later phase must correlate live socket events with trusted BM1366 production work and parsed pool responses.

### `version_1_1_live_asic_share_outcome`

- **Allowed claim:** A detector-gated Ultra 205 production mining session observed at least one accepted share or rejected share response to a live ASIC-derived `mining.submit`, or explicitly recorded that safe prerequisites blocked the attempt.
- **Evidence basis:** Deferred Phase 25 share-outcome artifacts with redacted pool, ASIC, API, WebSocket, safe-stop, and conclusion records.
- **Blocked claim:** This tier still does not prove non-205 board support, Stratum v2, OTA/recovery trust, runtime display/input parity, BAP behavior, full active safety closure, destructive recovery behavior, or unbounded production stress mining.
- **Evidence required for promotion:** Later milestones must add their own hardware-regression or protocol evidence for each deferred surface.

## Blocked Claims

Phase 21 `approved_controlled_no_share_soak` supports controlled no-share closure only.

That closure must not be cited as proof of accepted shares, rejected shares, unbounded production mining, full active safety closure, non-205 board support, Stratum v2, OTA/recovery trust, runtime display/input parity, or BAP behavior.

The following claims remain blocked unless a later phase creates detector-gated, redacted evidence for that exact surface:

- accepted shares
- rejected shares
- unbounded production mining
- full active safety closure
- non-205 board support
- Stratum v2
- OTA/recovery trust
- runtime display/input parity
- BAP behavior

## Explicit Non-Claims

### `explicit_deferred_non_claim`

- **Allowed claim:** The deferred surface is named, tracked, and intentionally not promoted by this evidence set.
- **Explicit non-claim:** Phase 22 does not add an end-to-end v1.1 evidence root, live socket procedure, BM1366 production dispatch procedure, accepted/rejected share proof, full active voltage/fan/thermal/self-test/fault-stimulus closure, non-205 board support, OTA/recovery trust, runtime display/input parity, BAP behavior, Stratum v2, or unbounded stress mining.
- **Evidence required for promotion:** The owning future phase must provide exact artifacts, redaction review, parity checklist updates, and verification commands for the specific surface being promoted.

## Promotion Rules

1. A lower tier may support readiness for a later phase, but it must not be promoted as a higher-tier claim without new evidence for that tier.
2. `approved_controlled_no_share_soak` may support `version_1_0_controlled_no_share` only. Live share behavior requires its own future evidence.
3. A claim may advance to `version_1_1_prerequisite_readiness` only when the evidence names fresh or explicitly bounded prerequisite observations and preserves blocker reasons for stale, unavailable, unsafe, ambiguous, or undocumented inputs.
4. A claim may advance to `version_1_1_live_socket_runtime` only when a redacted live Stratum v1 socket/runtime procedure and artifacts exist.
5. A claim may advance to `version_1_1_live_asic_share_outcome` only when live ASIC-derived work is correlated with a parsed pool response or a documented safe-prerequisite blocker.
6. Deferred surfaces remain `explicit_deferred_non_claim` until their own phase provides exact evidence and parity validation.

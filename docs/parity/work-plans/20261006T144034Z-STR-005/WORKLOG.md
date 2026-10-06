# Parity work log

## 2026-10-06T14:40:34Z | Evidence validation

- Source commit: `ad4d7e04` (plan); final candidate firmware `2bff1004`,
  ELF `9783dc74dfb369e633f9f3295021c34eefbd36fb50859a0d58a235447189b33a`,
  Gate `86fc62d7`, reference `c1915b0a`.
- Actions:
  - Re-verified the sealed inventory and the result and seal digests of
    psram-default-install attempt-002, heartbeat010, restart007,
    share-current-001, restart008, restart009 and renew-current-002; every
    result is complete.
  - `just str005-lineage show`: the head re-derives to
    `psram-default-install` on `2bff1004`/`9783dc74`, latest Start
    renew-current-002.
  - `just stratum-v2-noise-serial review` on install attempt-002: `passed`,
    `complete`, `hardware_qualified`, result `45db405c…`, seal `3998bc37…`,
    including its public projection.
  - The five published native audits
    (`docs/parity/evidence/str005-candidate-native/`) and the four sealed
    preflight audits of renew-current-002 are all bound to the candidate ELF;
    the retained ELF hashes to `9783dc74…`.
  - Confirmed the completed integration review: independently reviewed
    cumulative coverage, final-candidate compatibility, no open blocker, and
    recorded owner decisions (ADR-0032; `hardware-regression` qualifies).
- Verification: ordered Cargo checks pass (2,664 tests, 3 existing ignores);
  Bright Builds, `just verify-redaction` and `just parity` pass.
- Evidence: see `RESULT.md`.
- Outcome: eligible for `verified`.
- Blocker or next safe action: none; transition STR-005.

## 2026-10-06T16:30:00Z | Transition and validator correction

- Source commit: `3f4845a0` (RESULT.md).
- Actions: `transition-item` moved STR-005 to `verified` with
  `unit,golden,workflow,hardware-regression`; `sync-progress` appended
  progress and updated the README to 91 of 95.
- Verification: `just parity` then failed with "deferred or non-205
  verified rows cannot reuse Ultra 205 evidence". The guard dated from
  June 2026, when STR-005 was `deferred` and only Stratum v1 was in scope;
  its test pinned the case of Stratum v2 verified on reused Ultra 205
  Stratum v1 evidence. STR-005 left the hard-coded deferred list and gained
  its own rule: Ultra 205 evidence is admitted only through its Stratum V2
  integration review, and never when the row cites reused Stratum v1
  evidence. The original test still rejects its v1 case; two new tests
  cover acceptance through the review and rejection of v1 reuse beside it.
  `just parity` then reported `validation_errors: none`.
- Outcome: verified.

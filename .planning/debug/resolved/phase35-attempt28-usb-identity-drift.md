---
status: resolved
trigger: "Attempt 28 reached restart and sealed with typed device-session category usb_identity_drift."
created: 2026-07-23T02:48:42Z
updated: 2026-07-23T03:00:35Z
---

## Current Focus

hypothesis: Confirmed. Recovery scans construct separate candidates from `IOCalloutDevice` and `IODialinDevice` properties even though macOS exposes them as aliases of one serial service and physical identity. Initial selection filters the admitted callout path, while recovery filters only physical identity and therefore sees two matches.
test: Add a sanitized ioreg fixture containing paired callout/dial-in aliases for one physical device and prove the adapter retains only the canonical callout candidate.
expecting: The paired-alias regression produces one candidate after the fix while preserving the existing nested-property and physical/enumeration separation tests.
next_action: Commit the verified alias repair, run exact-head Phase 35 preflight, then execute fresh Attempt 29 under the standing progress-gated authority.

## Symptoms

expected: After the fully transmitted restart request, recovery finds at most one node matching the original physical identity and continues to the HTTP Boot B quorum.
actual: Three initial samples matched the same device and armed the reader; pre/post serial bytes were received and one restart request was fully sent and answered. The first recovery sample classified multiple physical matches, and no Boot B HTTP observation was admitted.
errors: Public primary category is `usb_identity_drift`; the seal records `restoration_action_failed`, cleanup completed, and a fresh protected GET independently confirms the original setting is already restored.
reproduction: Attempt 28 is sealed and must not be reused. The candidate parser deterministically emits both aliases from one sanitized serial-service fixture.
started: Attempt 28 at exact source `7e9be48adcdb64a072f08b41dc4849b073c5ab15` after the full software gate and exact-head preflight passed.

## Eliminated

- Attempt 27 parser defect: all three initial samples matched the same physical device and the reader armed.
- Observer silence: pre- and post-restart serial delivery both succeeded.
- Restart transmission ambiguity: exactly one request was fully written and its response was received.
- Actual current setting drift: a fresh protected typed GET classifies ready and its private value digest matches the original, not the mutated value.
- Cleanup failure: cleanup completed with no secondary category.

## Evidence

- timestamp: 2026-07-23T02:48:42Z
  checked: Attempt 28's redacted non-promotion seal, public device-session projection, and category-only private event reduction.
  found: Initial samples are unique-same and accessible with zero holders; recovery reports one multiple match; request count is one and complete; serial delivery is correlated; cleanup is complete.
  implication: Candidate aliasing after restart, not identity loss or request failure, is the authoritative boundary.
- timestamp: 2026-07-23T02:48:42Z
  checked: A fresh protected HTTP read after the sealed run.
  found: The typed HTTP boundary is ready and the current private value digest matches the original setting.
  implication: No additional recovery mutation is required despite the conservative restoration secondary category.
- timestamp: 2026-07-23T03:00:35Z
  checked: Sanitized paired callout/dial-in regression, focused Cargo test, uncached device-session and full Phase 35 supervisor Bazel suites, mandatory Rust sequence, redaction, reference, parity, lifecycle, and diff integrity.
  found: Candidate parsing retains exactly one canonical callout node; both uncached Bazel suites pass, and every complete software gate exits successfully.
  implication: The recovery alias defect is repaired without weakening physical identity or enumeration checks.

## Resolution

root_cause: Candidate construction treated both `IOCalloutDevice` and `IODialinDevice` properties as distinct serial nodes. Initial qualification filtered the admitted callout path, but physical-identity recovery retained both aliases and classified one device as multiple matches.
fix: Canonicalized macOS serial candidates to `IOCalloutDevice` and added a sanitized paired-alias regression that requires exactly one candidate while retaining independent physical and enumeration identities.
verification: The focused regression passes; uncached `//tools/device-session:tests` and `//scripts:phase35_correlated_evidence_test` pass; the mandatory Rust sequence, redaction, reference, parity, exact Phase 35 lifecycle, and diff checks all pass. No hardware or device-network effect occurred during the repair.
files_changed:

- .planning/debug/phase35-attempt28-usb-identity-drift.md
- tools/device-session/src/macos.rs

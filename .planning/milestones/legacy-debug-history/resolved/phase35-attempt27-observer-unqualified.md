---
status: resolved
trigger: "Attempt 27 stopped before restart with typed device-session category observer_unqualified."
created: 2026-07-23T02:26:51Z
updated: 2026-07-23T02:34:03Z
---

## Current Focus

hypothesis: Confirmed. The macOS ioreg parser accepts only property lines whose trimmed first byte is a quote, but real tree output prefixes nested properties with branch characters. The canonical shell parser accepts those lines and reproduces the detector-bound physical identity, while the Rust parser drops the target candidate.
test: Add a sanitized nested-tree regression containing branch-prefixed USB properties and the admitted callout node, then prove the parser returns one same-device candidate before any reader or HTTP work.
expecting: Attempt 27's captured ioreg structure fails before the repair and the sanitized regression passes afterward without recording raw USB values or paths.
next_action: Commit the verified parser repair, run exact-head Phase 35 preflight, then execute fresh Attempt 28 under the standing progress-gated authority.

## Symptoms

expected: Three initial macOS samples identify the admitted callout node as the same physical device, prove accessibility and no holders, then arm the receive-only reader.
actual: Thirty-three bounded initial samples reported no matching candidate; the reader was never armed and no restart request was attempted.
errors: Public terminal category is `observer_unqualified`; restoration and cleanup have no secondary failures.
reproduction: Attempt 27 is sealed and must not be reused. Its protected ioreg structure contains one exact callout occurrence, and the canonical shell identity parser matches the detector-bound digest.
started: Attempt 27 at exact source `120e09dd117faaaa3bfdc056ebe6ea640e9b99c7` after the full software gate and exact-head preflight passed.

## Eliminated

- Missing admitted node: the protected diagnostic found one exact callout occurrence.
- Physical-device drift: the canonical shell parser recomputed the detector-bound digest.
- Holder conflict or serial-read failure: candidate selection failed before accessibility, holder, open, or read work.
- Restart ambiguity: request attempt count remained zero.
- Restoration or cleanup failure: both secondary categories are `none`, and cleanup completed.

## Evidence

- timestamp: 2026-07-23T02:26:51Z
  checked: Attempt 27's redacted non-promotion seal and `esp-device-session-v1` projection.
  found: Flash boundary remained ready, device-session category is `observer_unqualified`, all 33 initial samples classified the physical match as none, the reader was not armed, request attempts are zero, duration is bounded, cleanup completed, and the root is non-reusable.
  implication: The failure is before application effect and is safe to reproduce in the host parser.
- timestamp: 2026-07-23T02:26:51Z
  checked: One separately protected, read-only ioreg capture against the admitted callout node, reduced to counts and digest-equality facts.
  found: The callout occurs exactly once, the canonical shell parser finds the USB ancestors and matches the detector-bound physical digest, and nested property lines include tree branch prefixes before their quoted keys.
  implication: The Rust parser's start-of-line quote assumption is the deterministic source defect.
- timestamp: 2026-07-23T02:34:03Z
  checked: Sanitized branch-prefixed nested-ioreg regression, focused Cargo test, uncached device-session and full Phase 35 supervisor Bazel suites, mandatory Rust sequence, redaction, reference, parity, lifecycle, and diff integrity.
  found: The regression now returns exactly one candidate; both uncached Bazel suites pass, and every complete software gate exits successfully.
  implication: The targeted parser defect is repaired without weakening identity, request, evidence, or hardware boundaries.

## Resolution

root_cause: The macOS adapter trimmed only whitespace before requiring a quoted ioreg property key. Real nested tree output prefixes those properties with branch characters, so all USB ancestor fields were discarded and the admitted callout candidate was never constructed.
fix: Restrict property-prefix trimming to the native tree's spaces and branch characters before parsing the quoted key, and add a sanitized nested USB-device/interface/serial regression with branch-prefixed properties.
verification: The focused regression passes; uncached `//tools/device-session:tests` and `//scripts:phase35_correlated_evidence_test` pass; the mandatory Rust sequence, redaction, reference, parity, exact Phase 35 lifecycle, and diff checks all pass. No hardware or device-network effect occurred during the repair.
files_changed:

- .planning/debug/phase35-attempt27-observer-unqualified.md
- tools/device-session/src/macos.rs

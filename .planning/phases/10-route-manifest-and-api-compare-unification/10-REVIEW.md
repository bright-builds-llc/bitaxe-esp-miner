---
phase: 10-route-manifest-and-api-compare-unification
reviewed: 2026-06-29T17:24:28Z
depth: standard
files_reviewed: 4
files_reviewed_list:
  - crates/bitaxe-api/src/lib.rs
  - crates/bitaxe-api/src/route_shell.rs
  - firmware/bitaxe/src/http_api.rs
  - tools/parity/src/api_compare.rs
findings:
  critical: 0
  warning: 1
  info: 0
  total: 1
status: issues_found
---

# Phase 10: Code Review Report

**Reviewed:** 2026-06-29T17:24:28Z
**Depth:** standard
**Files Reviewed:** 4
**Status:** issues_found

## Summary

Reviewed the Phase 10 route manifest/reporting and API compare changes in the four scoped Rust files. The route manifest exports and firmware reporting are narrowly scoped, and the Phase 7 route-kind policy checks are covered by focused unit tests. The review was informed by `AGENTS.md`, `AGENTS.bright-builds.md`, `standards/core/architecture.md`, `standards/core/code-shape.md`, `standards/core/verification.md`, `standards/core/testing.md`, and `standards/languages/rust.md`.

One warning was found in the new release-sensitive evidence policy: unknown evidence labels currently satisfy the guard as if they were strong evidence.

## Warnings

### WR-01: Unknown evidence labels can bypass weak-evidence overclaim rejection

**File:** `tools/parity/src/api_compare.rs:589-593`

**Issue:** `validate_verified_claim_policy` rejects a release-sensitive `verified_claim` only when every evidence label is listed in `WEAK_VERIFIED_EVIDENCE_LABELS`. Because `Iterator::all` returns `false` as soon as it sees an unrecognized label, a typo or bogus label such as `hardwar-smoke` prevents the warning from firing even though no real hardware or release-gate evidence exists. That weakens the Phase 10 guarantee that `/api/system/OTA`, `/api/system/OTAWWW`, `/recovery`, and `/*` cannot be marked verified from non-live evidence.

**Fix:** Treat evidence labels as an allowlisted enum/policy. Reject unknown labels and require at least one recognized strong label for release-sensitive verified claims.

```rust
const STRONG_VERIFIED_EVIDENCE_LABELS: &[&str] =
    &["hardware-smoke", "hardware-regression", "release-gate"];

fn is_known_verified_evidence_label(evidence: &str) -> bool {
    WEAK_VERIFIED_EVIDENCE_LABELS.contains(&evidence)
        || STRONG_VERIFIED_EVIDENCE_LABELS.contains(&evidence)
}

let has_unknown_evidence = claim
    .evidence
    .iter()
    .any(|evidence| !is_known_verified_evidence_label(evidence.as_str()));
let has_strong_evidence = claim
    .evidence
    .iter()
    .any(|evidence| STRONG_VERIFIED_EVIDENCE_LABELS.contains(&evidence.as_str()));

if has_unknown_evidence || !has_strong_evidence {
    validation_errors.push(format!(
        "release-sensitive route {} has insufficient verified evidence: evidence={}",
        route_key(&call.method, &call.path),
        claim.evidence.join(", ")
    ));
}
```

Add a regression test where `verified_claim.evidence` is `["hardwar-smoke"]` or another unknown label and assert that API compare fails for the release-sensitive route.

***

_Reviewed: 2026-06-29T17:24:28Z_
_Reviewer: the agent (gsd-code-reviewer)_
_Depth: standard_

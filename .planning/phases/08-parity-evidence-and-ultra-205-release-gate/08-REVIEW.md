---
phase: 08-parity-evidence-and-ultra-205-release-gate
reviewed: 2026-06-29T00:16:47Z
depth: standard
files_reviewed: 9
files_reviewed_list:
  - docs/parity/checklist.md
  - docs/parity/evidence/phase-08-ultra-205-release-gate.md
  - docs/parity/evidence/phase-08-ultra-205-release-gate/flash-monitor-noninteractive.log
  - docs/parity/evidence/phase-08-ultra-205-release-summary.md
  - docs/release/license-inventory.md
  - docs/release/provenance-manifest.md
  - docs/release/ultra-205.md
  - tools/parity/src/main.rs
  - tools/parity/src/release_gate.rs
findings:
  critical: 0
  warning: 2
  info: 0
  total: 2
status: issues_found
---

# Phase 8: Code Review Report

**Reviewed:** 2026-06-29T00:16:47Z
**Depth:** standard
**Files Reviewed:** 9
**Status:** issues_found

## Summary

Reviewed the Phase 8 parity checklist, release evidence, release docs, serial log, and `tools/parity` release-gate code. The checked-in docs keep the live HTTP/OTA/recovery surfaces below `verified` while `DEVICE_URL` is blocked, and the evidence/log scan did not find committed private URLs, Wi-Fi credentials, pool credentials, private endpoints, or NVS secret values.

Material guidance used: `AGENTS.md`, `AGENTS.bright-builds.md`, `standards/core/architecture.md`, `standards/core/code-shape.md`, `standards/core/verification.md`, `standards/core/testing.md`, and `standards/languages/rust.md`.

Verification run during review:

- `cargo test -p bitaxe-parity --all-features` passed, 37 tests.
- `just parity` passed with `validation_errors: none`.
- `bazel run //tools/parity:report -- release-gate --manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json` passed.

## Warnings

### WR-01: Verified-row guards accept negated live-evidence text

**File:** `tools/parity/src/main.rs:588`

**Issue:** `validate_release_image_verified_row` and the shared required-term check accept required words anywhere in the row text. That lets blocker language satisfy release evidence requirements. I reproduced this with a `verified` `REL-003` row whose notes said `rollback not run`, `recovery not run`, `large erase not run`, `failed update not run`, and `interrupted-update not run`; `--fail-on-invalid-verified` still printed `validation_errors: none`. This can create a false positive if a future checklist edit promotes a release row while live evidence is still blocked.

**Fix:** Reject blocker terms before accepting required release terms, or require structured positive evidence tokens/phrases. Add regression tests for `REL-003`, `OTA-001`, `OTA-002`, and `FS-001` rows where required terms appear only in `not run`, `blocked`, `pending`, or `no reachable DEVICE_URL` text.

```rust
fn row_contains_live_evidence_blocker(row: &ChecklistRow) -> bool {
    let haystack = row_haystack(row);
    ["not run", "blocked", "pending", "no reachable device_url", "unverified"]
        .iter()
        .any(|term| haystack.contains(term))
}
```

### WR-02: Manifest gate accepts wrong-board metadata

**File:** `tools/parity/src/release_gate.rs:248`

**Issue:** Manifest validation only checks that `image_metadata.board`, `device_model`, and `asic` are non-empty. I reproduced `release_gate: passed` with a manifest named `bitaxe-ultra205-package.json` that contained `board = 601`, `device_model = Gamma 601`, and `asic = BM1370`, as long as the artifact path substrings and SHA-256 shapes were present. This weakens the Ultra 205 release gate because a wrong-board manifest can satisfy the manifest-backed release check.

**Fix:** Validate exact Ultra 205 metadata and artifact contracts when `--manifest` is supplied: `board == "205"`, `device_model == "Ultra 205"`, `asic == "BM1366"`, expected `release_name`, expected `default_flash_image`, exact license/provenance paths, and required artifact kind/path/offset tuples. Add a regression test that a Gamma/BM1370 manifest is rejected.

```rust
fn validate_manifest_exact_string(
    errors: &mut Vec<String>,
    manifest_path: &Utf8PathBuf,
    manifest: &Value,
    pointer: &str,
    label: &str,
    expected: &str,
) {
    if manifest.pointer(pointer).and_then(Value::as_str) == Some(expected) {
        return;
    }

    errors.push(format!(
        "package manifest `{manifest_path}` field `{label}` must be `{expected}`"
    ));
}
```

_Reviewed: 2026-06-29T00:16:47Z_
_Reviewer: the agent (gsd-code-reviewer)_
_Depth: standard_

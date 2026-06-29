---
phase: 11-safety-controller-hardware-regression-evidence
reviewed: 2026-06-29T21:34:48Z
depth: standard
files_reviewed: 1
files_reviewed_list:
  - tools/parity/src/main.rs
findings:
  critical: 0
  warning: 0
  info: 0
  total: 0
status: clean
---

# Phase 11: Code Review Report

**Reviewed:** 2026-06-29T21:34:48Z
**Depth:** standard
**Files Reviewed:** 1
**Status:** clean

## Summary

Reviewed `tools/parity/src/main.rs` for the Phase 11 active safety-control evidence validation change. The validator correctly scopes the new `hardware-regression` requirement to `verified` active-control rows, preserves read-only `hardware-smoke` behavior for non-active rows such as `PWR-006`, and the current checklist passes `just parity`.

Material context loaded: `AGENTS.md`, `AGENTS.bright-builds.md`, `standards-overrides.md`, `standards/index.md`, `standards/core/architecture.md`, `standards/core/code-shape.md`, `standards/core/testing.md`, `standards/core/verification.md`, and `standards/languages/rust.md`. No project skill directories were present under `.claude/skills/` or `.agents/skills/`.

## Warnings

### WR-01: Active-Control ID Coverage Is Partial

**File:** `tools/parity/src/main.rs:974`
**Issue:** `is_active_safety_control` classifies eight rows as active safety-control rows (`PWR-001`, `PWR-002`, `PWR-003`, `PWR-005`, `THR-001`, `THR-002`, `SELF-001`, and `UI-003`), but the new active-control test only exercises the rejection path for `PWR-003`, `THR-002`, and `SELF-001`, plus the non-active `PWR-006` allow path. That leaves five active row IDs and the positive `hardware-regression` allow path unguarded by tests. Because this list is the safety evidence gate, a future typo or removal could silently relax a row while the targeted tests still pass.

**Fix:**
```rust
#[test]
fn active_safety_control_requires_hardware_regression_for_all_active_rows() {
    // Arrange
    let active_ids = [
        "PWR-001", "PWR-002", "PWR-003", "PWR-005", "THR-001", "THR-002", "SELF-001",
        "UI-003",
    ];

    for id in active_ids {
        let checklist = format!(
            r#"
| ID | Surface | Reference Breadcrumb | Rust-Owned Target | Status | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| {id} | Active safety-control row | reference path | firmware/bitaxe | verified | hardware-smoke | Active hardware-control behavior. |
"#
        );
        let rows = parse_checklist(&checklist).expect("checklist should parse");

        // Act
        let errors = validate_rows(&rows);

        // Assert
        assert_validation_error_contains(&errors, id, "requires hardware-regression evidence");
    }
}

#[test]
fn active_safety_control_allows_hardware_regression_evidence() {
    // Arrange
    let checklist = r#"
| ID | Surface | Reference Breadcrumb | Rust-Owned Target | Status | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| PWR-003 | Core voltage control | reference path | firmware/bitaxe | verified | hardware-regression | Active voltage regression passed. |
"#;
    let rows = parse_checklist(checklist).expect("checklist should parse");

    // Act
    let errors = validate_rows(&rows);

    // Assert
    assert!(errors.is_empty());
}
```

**Resolution:** Fixed. The active-control rejection test now iterates over all eight active row IDs, and a positive test proves `hardware-regression` evidence is accepted for an active row.

## Verification

- Green: `cargo test -p bitaxe-parity --all-features active_safety_control`
- Green: `cargo test -p bitaxe-parity --all-features safety_critical`
- Green: `just parity`
- Green after fix: `cargo test -p bitaxe-parity --all-features active_safety_control`
- Green after fix: `cargo test -p bitaxe-parity --all-features safety_critical`
- Green after fix: `just parity`

## Final Status

All review findings were addressed; no remaining issues.

_Reviewed: 2026-06-29T21:34:48Z_
_Reviewer: the agent (gsd-code-reviewer)_
_Depth: standard_

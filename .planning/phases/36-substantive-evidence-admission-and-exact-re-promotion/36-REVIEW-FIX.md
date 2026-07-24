---
phase: 36
fixed_at: 2026-07-24T17:51:36Z
review_path: .planning/phases/36-substantive-evidence-admission-and-exact-re-promotion/36-REVIEW.md
iteration: 4
findings_in_scope: 1
fixed: 1
skipped: 0
status: all_fixed
---

# Phase 36: Code Review Fix Report

**Fixed at:** 2026-07-24T17:51:36Z
**Source review:** `.planning/phases/36-substantive-evidence-admission-and-exact-re-promotion/36-REVIEW.md`
**Iteration:** 4

**Summary:**

- Findings in scope: 1
- Fixed: 1
- Skipped: 0
- Phase 35 admitted evidence bytes, canonical Phase 36 evidence, and the parity checklist remained unchanged.
- The subsequent final review passed at exact commit `f1cb6101f2c384acaffe0b8523097433ff0f04cc`; this fix report retains its own `all_fixed` status, fix commit, iteration, and verification facts.

## Fixed Issues

### WR-01: Runtime-identity replay validator is omitted from the evidence evaluator identity

**Status:** Fixed; independently verified by the final passed review at `f1cb6101f2c384acaffe0b8523097433ff0f04cc`.
**Files modified:** `tools/device-session/BUILD.bazel`, `tools/parity/BUILD.bazel`, `tools/parity/fixtures/phase36/envelope-only.json`, `tools/parity/src/phase36_evidence.rs`, `tools/parity/src/phase36_evidence/tests.rs`
**Commit:** `f54b69c3`
**Applied fix:** The Phase 36 evaluator inventory now includes `tools/device-session/src/model.rs`, which owns `SessionRequest::schema_is_valid`, `SessionState::apply`, replay transitions, and terminal projections. A narrow device-session filegroup exposes that source to the parity binary as declared Bazel compile data. Evaluator identity encoding is now versioned and unambiguous: each inventory entry binds a big-endian 64-bit path length, path bytes, source length, and source bytes. The regression asserts the exact inventory, locates both runtime validators, and proves hostname-validator source drift, `SessionState::apply` drift, and inventory-path drift each rotate both evaluator and successor-contract identities. Only the envelope-only evaluator and contract fixture values were rotated.

## Verification

- Red regression signal: the extended inventory test initially failed to compile because the path-aware inventory digest and contract-from-evaluator helpers did not exist; it passed after implementation.
- `cargo test -p bitaxe-parity phase36_evaluator_inventory_binds_every_material_owned_validator --all-features` passed.
- `cargo test -p bitaxe-parity phase36 --all-features` passed 76/76.
- The final `bazel test --nocache_test_results //tools/parity:phase36_evidence_tests //scripts:phase36_evidence_test //tools/parity:phase36_promotion_tests //tools/parity:operator_evidence_generation_tests //scripts:package_firmware_test --test_output=errors` passed 5/5, including declared compile-data enforcement and the real CLI exact `category=protected_input_missing` regression.
- `just parity` reported `validation_errors: none`; `just verify-reference` reported clean reference `c1915b0a63bfabebdb95a515cedfee05146c1d50`; `just verify-redaction` passed.
- The exact mandatory Rust sequence passed in order from fresh isolated target `/var/folders/b6/j7bsvp3j6jzbl8r28p9wqtnh0000gn/T/phase36-iteration4-target.D3lrR6`: `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, and `cargo test --all-features`.
- Recomputed evaluator digest `a07293e163637be694c1e08698d1836011618fe932052fcd334d43e4d9033ddb` and successor contract digest `5e4db14d72eb419dbffe317bbae4a2629b55b394126c7de1b2c65835ab335af7` matched the checked-in envelope-only fixture.
- `git diff --check` passed. Phase 35 evidence, canonical Phase 36 evidence, and `docs/parity/checklist.md` had no diff.

## Skipped Issues

None.

***

_Fixed: 2026-07-24T17:51:36Z_
_Fixer: the agent (gsd-code-fixer)_
_Iteration: 4_

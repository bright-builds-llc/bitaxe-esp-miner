---
phase: 36-substantive-evidence-admission-and-exact-re-promotion
reviewed: 2026-07-24T18:01:45Z
reviewed_commit: f1cb6101f2c384acaffe0b8523097433ff0f04cc
depth: standard
files_reviewed: 53
files_reviewed_list:
  - BUILD.bazel
  - docs/parity/checklist.md
  - docs/parity/evidence/phase-36-substantive-evidence-admission-and-exact-re-promotion/checklist.md
  - docs/parity/evidence/phase-36-substantive-evidence-admission-and-exact-re-promotion/decision-matrix.json
  - docs/parity/evidence/phase-36-substantive-evidence-admission-and-exact-re-promotion/manifest.json
  - docs/parity/evidence/phase-36-substantive-evidence-admission-and-exact-re-promotion/typed-fact-projection.json
  - docs/parity/evidence/phase-36-substantive-evidence-admission-and-exact-re-promotion/verdict.json
  - firmware/bitaxe/BUILD.bazel
  - scripts/BUILD.bazel
  - scripts/build-firmware.sh
  - scripts/package-firmware-test.sh
  - scripts/package-firmware.sh
  - scripts/phase35-promotion-contract-test.sh
  - scripts/phase36-evidence-test.sh
  - tools/device-session/BUILD.bazel
  - tools/device-session/src/model.rs
  - tools/parity/BUILD.bazel
  - tools/parity/Cargo.toml
  - tools/parity/fixtures/phase36/envelope-only.json
  - tools/parity/fixtures/phase36/independent-effects-eligible.json
  - tools/parity/fixtures/phase36/mutation-catalog.json
  - tools/parity/fixtures/phase36/runtime-identity-package.json
  - tools/parity/fixtures/phase36/substance-eligible.json
  - tools/parity/src/main.rs
  - tools/parity/src/operator_evidence.rs
  - tools/parity/src/operator_evidence/generation.rs
  - tools/parity/src/operator_evidence/generation/phase36.rs
  - tools/parity/src/operator_evidence/generation/phase36/transaction.rs
  - tools/parity/src/operator_evidence/generation/tests.rs
  - tools/parity/src/operator_evidence/generation/tests/phase36.rs
  - tools/parity/src/operator_evidence/generation/tests/support.rs
  - tools/parity/src/phase35_evidence/contract.rs
  - tools/parity/src/phase35_evidence/tests.rs
  - tools/parity/src/phase36_evidence.rs
  - tools/parity/src/phase36_evidence/contract.rs
  - tools/parity/src/phase36_evidence/effects.rs
  - tools/parity/src/phase36_evidence/runtime_identity.rs
  - tools/parity/src/phase36_evidence/runtime_identity/ledger.rs
  - tools/parity/src/phase36_evidence/substance.rs
  - tools/parity/src/phase36_evidence/substance/types.rs
  - tools/parity/src/phase36_evidence/tests.rs
  - tools/parity/src/phase36_evidence/tests/authority.rs
  - tools/parity/src/phase36_evidence/tests/effects.rs
  - tools/parity/src/phase36_evidence/tests/mutations.rs
  - tools/parity/src/phase36_evidence/tests/runtime_identity.rs
  - tools/parity/src/phase36_evidence/tests/substance.rs
  - tools/parity/src/phase36_offline.rs
  - tools/parity/src/phase36_promotion.rs
  - tools/parity/src/phase36_promotion/checklist.rs
  - tools/parity/src/phase36_promotion/evaluator.rs
  - tools/parity/src/phase36_promotion/tests.rs
  - tools/parity/src/phase36_promotion/types.rs
  - tools/parity/src/protected_input.rs
findings:
  critical: 0
  high: 0
  warning: 0
  info: 0
  total: 0
unresolved:
  critical: 0
  high: 0
  warning: 0
status: passed
---

# Phase 36: Code Review Report

**Reviewed:** 2026-07-24T18:01:45Z
**Reviewed commit:** `f1cb6101f2c384acaffe0b8523097433ff0f04cc`
**Depth:** standard
**Files reviewed:** 53
**Status:** passed

## Summary

All reviewed Phase 36 source, fixtures, build metadata, canonical generation, and validation evidence meet the checkpoint's correctness, security, and maintainability gates. No unresolved critical, high, warning, or informational findings remain.

The v4 warning is closed. `tools/device-session/src/model.rs` is now a declared Bazel compile-data input and an exact member of the evidence evaluator inventory. Evaluator identity uses versioned, length-delimited path/source framing, and regressions prove that both `SessionState::apply` source drift and inventory path drift rotate the evaluator and successor-contract digests. The checked-in envelope identity was rotated accordingly.

The earlier repairs also remain closed: production authority does not accept caller-authored companion digests; immutable Phase 35 authority is pinned and authenticated; protected reads are descriptor-relative and revalidated; invalid companions cannot publish; generation/checklist publication is crash-recoverable and rollback-tested; temporary staging and replacement creation is collision-safe; Bazel declares the explicit firmware ELF, SDK config, bootloader, partition table, and OTA data inputs; and the evaluator inventory covers every material repository-owned validator in the reviewed admission path.

No hardware, USB, serial, network, credential, direct-UART/pin, or archived Phase 28.1.1 path was invoked or expanded. Canonical Phase 36 evidence and the checklist were not changed by the v5 repair.

## Verification Performed

- Confirmed exact source commit `f1cb6101f2c384acaffe0b8523097433ff0f04cc`.
- `cargo test -p bitaxe-parity phase36 --all-features` passed 76/76.
- Fresh uncached Bazel tests passed 5/5 across `//tools/parity:phase36_evidence_tests`, `//scripts:phase36_evidence_test`, `//tools/parity:phase36_promotion_tests`, `//tools/parity:operator_evidence_generation_tests`, and `//scripts:package_firmware_test`.
- Shell syntax checks passed for the Phase 36, Phase 35 promotion, firmware build, firmware package, and package test scripts.
- A fresh CLI run from a temporary directory without Git metadata failed closed on missing public artifacts with exit 1, zero stdout, and only `category=offline_public_inputs_invalid`.
- `git diff --check` passed.
- Phase 35 evidence is unchanged since admission and retains aggregate tree hash `117501de868e6390511c862105be785f87e61c2954a6fec3a0cf8beec093b9bd`, root digest `0401e7b485df2d1ccfc67e63845f98b6217816a184901bf0595d03af3219757d`, generation digest `cb76a76075ffe43bf2f4f7aff2e9224f70cbb65f35bcb59405896319a62792d9`, admitted digest `2f0cb112f67192fd7920c03e97cd01c373b26ab2dfe725edc1a114276ed0b7b0`, matrix digest `cf53f575a05f2aa84235548511743759c4928086d4a34e65d316c3a1b63832fe`, and projection digest `3e0eab6db782745eaa6bd250690ea21c65c39cd3a216acda17e9ef374d2e68b0`.
- Canonical Phase 36 generation files and `docs/parity/checklist.md` remain byte-unchanged since commit `d89a051f`.
- Existing dirty paths were preserved; only this review artifact was overwritten.

***

_Reviewed: 2026-07-24T18:01:45Z_
_Reviewer: the agent (gsd-code-reviewer)_
_Depth: standard_

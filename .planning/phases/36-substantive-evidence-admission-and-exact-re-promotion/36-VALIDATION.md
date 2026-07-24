---
phase: "36"
slug: substantive-evidence-admission-and-exact-re-promotion
status: draft
nyquist_compliant: true
wave_0_complete: false
created: "2026-07-23"
---

# Phase 36 — Validation Strategy

> Per-phase validation contract for substantive evidence admission and exact re-promotion.

## Test Infrastructure

| Property | Value |
| --- | --- |
| **Framework** | Rust unit tests, Bazel integration tests, repository shell fixtures |
| **Config file** | `Cargo.toml`, `tools/parity/BUILD.bazel`, `scripts/BUILD.bazel` |
| **Quick run command** | `cargo test -p bitaxe-parity phase36` |
| **Full suite command** | `just test && just parity && just verify-reference && just verify-redaction` |
| **Estimated runtime** | ~180 seconds without hardware |

## Sampling Rate

- **After every task commit:** Run the narrow Phase 36 Rust or Bazel target named by that task.
- **After every plan wave:** Run `just test && just parity && just verify-reference && just verify-redaction`.
- **Before phase verification:** Run the full Rust pre-commit sequence and every Phase 36 integration target.
- **Max feedback latency:** 180 seconds for task-local checks.

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 36-01-01 | 01 | 1 | EVD-11, EVD-12, EVD-14 | T36-01, T36-02 | Fixtures use closed fake values and immutable explicit roots | integration | `bazel test //tools/parity:phase36_evidence_tests` | ✅ | ✅ passed: 1/1 target (fresh 2026-07-24) |
| 36-01-02 | 01 | 1 | EVD-14 | T36-01 | Protected input never appears in terminal or public output | real-process | `bazel test //scripts:phase36_evidence_test` | ✅ | ✅ passed: 1/1 target (fresh 2026-07-24) |
| 36-02-01 | 02 | 2 | EVD-12, EVD-15 | T36-03 | Sensor/health substance and provenance joins fail closed | unit | `cargo test -p bitaxe-parity phase36_substance` | ✅ | ✅ passed: 14/14 tests (fresh 2026-07-24) |
| 36-02-02 | 02 | 2 | SYS-02, EVD-11 | T36-03 | Runtime identity is replayed from observations, not package fields | unit | `cargo test -p bitaxe-parity phase36_runtime_identity` | ✅ | ✅ passed: 11/11 tests (fresh 2026-07-24) |
| 36-02-03 | 02 | 2 | EVD-14 | T36-03, T36-04 | Legacy effect proof is complete or typed insufficient | unit | `cargo test -p bitaxe-parity phase36_effects` | ✅ | ✅ passed: 12/12 tests (fresh 2026-07-24) |
| 36-03-01 | 03 | 3 | EVD-15 | T36-05 | Claim decisions and checklist correction publish atomically | integration | `bazel test //tools/parity:phase36_promotion_tests` | ✅ | ✅ passed: 1/1 target (fresh 2026-07-24) |
| 36-03-02 | 03 | 3 | SYS-02, EVD-11, EVD-12, EVD-14, EVD-15 | T36-06 | Attempt 31 classification cannot trigger hardware | real-process | `bazel test //scripts:phase36_evidence_test` | ✅ | ✅ passed: 1/1 target (fresh 2026-07-24) |
| 36-03-03 | 03 | 3 | SYS-02, EVD-11, EVD-12, EVD-14, EVD-15 | T36-03, T36-06 | Attempt 31 yields exact decisions or aggregate typed insufficiency without hardware | real-process | `bazel test //scripts:phase36_evidence_test //tools/parity:phase36_promotion_tests` | ✅ | ✅ passed: 2/2 targets (fresh 2026-07-24) |
| 36-04-01 | 04 | 4 | SYS-02, EVD-11, EVD-12, EVD-14, EVD-15 | all | Reconciliation occurs only after clean verification | repository | `just test && just parity && just verify-reference && just verify-redaction` | ✅ | ✅ passed at `cc4784ca74cb343a38c8fe0f255b1189b831410f`: 73/73; no parity errors; reference clean; redaction passed (fresh 2026-07-24) |
| 36-04-02 | 04 | 4 | SYS-02, EVD-11, EVD-12, EVD-14, EVD-15 | all | Independent verifier produces a passed lifecycle-bound artifact before reconciliation | lifecycle | `node "$HOME/.codex/get-shit-done/bin/gsd-tools.cjs" verify lifecycle 36 --require-plans --require-verification --raw` | ❌ W0 | ⬜ pending |
| 36-04-03 | 04 | 4 | SYS-02, EVD-11, EVD-12, EVD-14, EVD-15 | T36-05, T36-06 | Canonical planning truth changes only after independent passed verification | repository | `just parity && just verify-reference && just verify-redaction` | ✅ | ⬜ pending |

## Plan 04 Task 1 Fresh Verification Evidence

Fresh verification on 2026-07-24 used source commit `cc4784ca74cb343a38c8fe0f255b1189b831410f`. No canonical planning-truth reconciliation was performed.

| Gate | Observed result |
| --- | --- |
| Mandatory Rust sequence | `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, and `cargo test --all-features` passed in the required order with fresh isolated `CARGO_TARGET_DIR=/tmp/bitaxe-phase36-iteration3-gate.3upJeU`; the parity crate ran 383 tests. |
| Focused Phase 36 Cargo suites | `cargo test -p bitaxe-parity phase36 --all-features` passed 76/76; `phase36_substance` passed 14/14; `phase36_runtime_identity` passed 11/11; `phase36_effects` passed 12/12. |
| Shell syntax and exact category | `bash -n scripts/phase36-evidence-test.sh scripts/phase35-promotion-contract-test.sh scripts/build-firmware.sh scripts/package-firmware.sh scripts/package-firmware-test.sh` passed; the declared `//scripts:phase36_evidence_test` fixture passed its exact `category=protected_input_missing` assertion. |
| Uncached Phase 36 Bazel concurrency | `bazel test --nocache_test_results //tools/parity:phase36_evidence_tests //tools/parity:phase36_promotion_tests //scripts:phase36_evidence_test` passed 3/3. |
| Firmware package regression | `bazel test --nocache_test_results //scripts:package_firmware_test` passed 1/1, and `bazel build //firmware/bitaxe:firmware_image` produced the ELF, OTA image, SPIFFS image, OTA data, factory image, and package manifest from declared ESP-IDF inputs. |
| Canonical repository suite | `just test` passed 73/73. |
| Parity, reference, and privacy | `just parity` reported `validation_errors: none`; `just verify-reference` reported clean reference `c1915b0a63bfabebdb95a515cedfee05146c1d50`; `just verify-redaction` passed. |
| Lifecycle and diff hygiene | Lifecycle verification with `--require-plans --raw` printed `valid`; `git diff --check` passed. Lifecycle verification with `--require-verification --raw` printed `invalid` because the independent security and verification artifacts do not yet exist, so Task 36-04-02 remains pending. |
| Phase 35 immutability | The Phase 35 generation aggregate remained `117501de868e6390511c862105be785f87e61c2954a6fec3a0cf8beec093b9bd`; its manifest, admitted document, matrix, and projection retained hashes `cb76a760…`, `2f0cb112…`, `cf53f575…`, and `3e0eab6d…`, and root digest `0401e7b4…` remained unchanged. |
| Checklist and scope inspection | Compared with the pre-Phase 36 checklist, exactly the four V12 rows changed. No unrelated row, hardware path, credential input, target discovery path, direct UART/pin path, archived Phase 28.1.1 lineage, network surface, or unapproved effect broker was added. |

Four Task 1/review gate failures were fixed before the final fresh run: commit `ff46c450` gives concurrent generation tests collision-resistant workspace identities, commit `857e2a95` makes Bazel declare and pass the exact ESP-IDF packaging inputs instead of relying on a source-tree cache, commit `37b7bc47` authenticates immutable evidence artifacts at the authority boundary, and commit `cc4784ca` binds transitive evaluator validators into provenance.

## Wave 0 Requirements

- [ ] Phase 36 fixture factory and one-field mutation matrix.
- [ ] Fake protected-root process harness with permission and redaction assertions.
- [ ] Successor publisher rollback/failure-injection fixtures.
- [ ] Named Cargo/Bazel targets referenced in the verification map.

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
| --- | --- | --- | --- |
| Any new hardware evidence attempt | EVD-11, EVD-12, EVD-14 | Not authorized by the base phase; depends on typed immutable-artifact insufficiency and later explicit plan authority | Stop after the insufficiency result. Add a later Phase 36 plan containing the detector, recovery, effects, evidence, timeout, privacy, and progress-gated contract before hardware use. |

## Validation Sign-Off

- [x] All planned tasks have automated verification or Wave 0 dependencies.
- [x] Sampling continuity avoids three consecutive tasks without automated verification.
- [x] Wave 0 names all missing test surfaces.
- [x] No watch-mode flags.
- [x] Feedback latency target is under 180 seconds.
- [x] `nyquist_compliant: true` is set in frontmatter.

**Approval:** pending plan checker and phase execution

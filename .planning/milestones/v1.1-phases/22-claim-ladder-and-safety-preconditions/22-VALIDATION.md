---
phase: 22
slug: claim-ladder-and-safety-preconditions
status: complete
nyquist_compliant: true
wave_0_complete: true
created: 2026-07-04
---

# Phase 22 - Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

## Test Infrastructure

| Property | Value |
| --- | --- |
| **Framework** | Bazel `rust_test` targets for Rust crates/tools plus repo `just` aggregate commands |
| **Config file** | `BUILD.bazel` files per crate/tool and root `Justfile` |
| **Quick run command** | `bazel test //crates/bitaxe-safety:tests //crates/bitaxe-stratum:tests //crates/bitaxe-api:tests //tools/parity:tests` |
| **Full suite command** | `just test` |
| **Evidence gates** | `just parity` and `just verify-reference` |
| **Estimated runtime** | Targeted Bazel tests should stay under a few minutes; full `just test` may take longer depending on cache state |

## Sampling Rate

- **After every task commit:** Run the narrow Bazel target for the changed crate/tool, plus `just parity` when docs/checklist/evidence surfaces change.
- **After every plan wave:** Run `bazel test //crates/bitaxe-safety:tests //crates/bitaxe-stratum:tests //crates/bitaxe-api:tests //tools/parity:tests`, `just parity`, and `just verify-reference`.
- **Before verification:** Run `just test`, `just parity`, `just verify-reference`, lifecycle validation, and any detector-gated hardware command actually used.
- **Max feedback latency:** Prefer targeted commands under 5 minutes; document cache or hardware blockers instead of silently skipping checks.

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 22-01-01 | 01 | 1 | EVD-06 | T-22-01 | Claim ladder cannot promote Phase 21 controlled no-share evidence to accepted/rejected production-share proof | parity/doc guard | `bazel test //tools/parity:tests` and `just parity` | Claim ladder docs and tests committed in 22-01 | passed |
| 22-01-02 | 01 | 1 | EVD-06 | T-22-02 | Parity claim-ladder guard requires all tier ids and rejects controlled no-share overclaims in tested Markdown | parity/unit guard | `bazel test //tools/parity:tests` | `tools/parity/src/claim_ladder.rs` committed in 22-01 | passed |
| 22-02-01 | 02 | 1 | SAFE-10 | T-22-02 | Production mining readiness requires fresh or explicitly bounded power, thermal, fan, voltage, and safety observations before BM1366 work dispatch | unit | `bazel test //crates/bitaxe-safety:tests //crates/bitaxe-stratum:tests` | `crates/bitaxe-safety/src/mining_preconditions.rs` committed in 22-02 | passed |
| 22-02-02 | 02 | 1 | SAFE-11 | T-22-03 | Missing, stale, unavailable, unsafe, ambiguous, or undocumented prerequisites fail closed with stable user-visible blocker reasons | unit + projection guard | `bazel test //crates/bitaxe-safety:tests //crates/bitaxe-stratum:tests //crates/bitaxe-api:tests` | Blocked-reason propagation committed in 22-02 | passed |
| 22-03-01 | 03 | 2 | EVD-06, SAFE-10, SAFE-11 | T-22-04 | Evidence docs and checklist preserve exact non-claims and cite only supported Phase 22 artifacts | workflow | `just parity` and `just verify-reference` | Evidence docs and checklist rows committed in 22-03 | passed |
| 22-03-02 | 03 | 2 | EVD-06, SAFE-10, SAFE-11 | T-22-08, T-22-11 | Checklist and validation status stay conservative, lifecycle-checked, and avoid promoting SAFE-10/SAFE-11 beyond implemented without detector-gated hardware evidence | workflow + lifecycle | `bazel test //tools/parity:tests //crates/bitaxe-safety:tests //crates/bitaxe-stratum:tests //crates/bitaxe-api:tests && just parity && just verify-reference && node "$HOME/.cursor/get-shit-done/bin/gsd-tools.cjs" verify lifecycle 22 --expect-id 22-2026-07-04T20-10-36 --expect-mode yolo --require-plans` | Full Phase 22 verification gate passed after checklist and validation updates | passed |

## Wave 0 Requirements

- [x] `crates/bitaxe-safety/src/mining_preconditions.rs` or equivalent typed prerequisite module if existing modules cannot represent Phase 22 semantics clearly.
- [x] Tests covering every required blocker class: missing, stale, unavailable, unsafe, ambiguous, and undocumented.
- [x] `tools/parity` tests for claim ladder tier and non-claim language, including Phase 21 controlled no-share non-promotion.
- [x] Phase 22 evidence docs: `claim-ladder.md`, `safety-preconditions.md`, `blocker-reasons.md`, `summary.md`, and `redaction-review.md`.

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
| --- | --- | --- | --- |
| New hardware evidence, if any is attempted | SAFE-10, SAFE-11 | Hardware use requires detector gate and evidence capture; Phase 22 can pass without new hardware claims if pure/static artifacts prove its scope | Run `just detect-ultra205` first; proceed only if exactly one Ultra 205 port and board-info pass, then record board `205`, port, source/reference commits, commands, logs, behavior, conclusion, and redaction status |

## Validation Sign-Off

- [x] All tasks have automated verification or documented manual-only gates.
- [x] Sampling continuity: no 3 consecutive tasks without automated verification.
- [x] Wave 0 covers all missing test references.
- [x] No watch-mode flags.
- [x] Feedback latency remains bounded or blockers are recorded.
- [x] `nyquist_compliant: true` set in frontmatter after plan coverage is complete.

**Approval:** passed

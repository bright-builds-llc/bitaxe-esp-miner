---
phase: "34"
slug: provenance-runtime-health-and-coherent-operator-snapshot
status: draft
nyquist_compliant: false
wave_0_complete: false
created: "2026-07-15"
---

# Phase 34 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

## Test Infrastructure

| Property | Value |
| --- | --- |
| **Framework** | Rust unit/integration tests, Bash behavior tests, Bazel test graph |
| **Config file** | `Cargo.toml`, `MODULE.bazel`, package-local `BUILD.bazel` files |
| **Quick run command** | `cargo test -p bitaxe-core -p bitaxe-api` plus the affected focused Bazel target |
| **Full suite command** | `bazel test //...` |
| **Estimated runtime** | ~180 seconds excluding firmware cross-build/package |

## Sampling Rate

- **After every task commit:** Run the task's focused Cargo, Bash, or Bazel behavior test.
- **After every plan wave:** Run the affected Bazel targets and `git diff --check`.
- **Before phase verification:** The mandatory Rust sequence, `bazel test //...`, `just build`, `just package`, and `just verify-reference` must be green.
- **Max feedback latency:** 180 seconds for host checks; canonical ESP-IDF build/package may exceed this bound and runs at plan completion.

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 34-01-01 | 01 | 1 | SYS-01, SYS-02 | T-34-01, T-34-02 | One validated identity stamp; relevant dirty state fails admission and dirty-to-dirty edits rebuild. | unit/shell/Bazel | focused identity crate and workspace-status tests | ❌ W0 | ⬜ pending |
| 34-01-02 | 01 | 1 | SYS-01 | T-34-03 | LCD/API/log surfaces project one identity while machine proof retains the full commit. | unit/integration | affected `bitaxe-core` and `bitaxe-api` tests | ❌ W0 | ⬜ pending |
| 34-01-03 | 01 | 1 | SYS-02 | T-34-04 | Manifest v3 consumes the ELF stamp and rejects dirty/inconsistent identity before hardware. | integration/Bazel | affected xtask/parity tests plus canonical package inspection | ❌ W0 | ⬜ pending |
| 34-02-01 | 02 | 2 | OBS-06 | T-34-05 | All operator projections bind to one boot session and monotonic revision. | unit/integration | focused snapshot/projection tests | ✅ | ⬜ pending |
| 34-03-01 | 03 | 3 | SYS-03, SYS-04, SYS-05 | T-34-06 | Platform facts are embedded/runtime-derived or explicitly unavailable, never fixture-substituted. | unit/integration/build | focused platform identity and build inspection tests | ✅ | ⬜ pending |
| 34-04-01 | 04 | 4 | HLT-01, HLT-02, HLT-03, HLT-04 | T-34-07 | Passive state/health projection performs no self-test, watchdog, load, fault, or hardware effect. | unit/source guard | focused health/checkpoint and forbidden-effect tests | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

## Wave 0 Requirements

- [ ] Typed build-identity unit tests and Bazel target.
- [ ] Workspace-status shell behavior test with temporary Git repositories.
- [ ] LCD/system-info/live-WebSocket identity projection tests.
- [ ] Manifest v3 and dirty-admission tests.
- [ ] Cache invalidation regression proving a second dirty source edit changes the firmware action inputs.

Existing Cargo/Bazel/shell infrastructure covers later Phase 34 plans.

## Manual-Only Verifications

None for Plan 01. The application descriptor, embedded commit, manifest, and dirty admission are inspected with host tooling. Physical qualification is deferred to Phase 35.

## Validation Sign-Off

- [ ] All tasks have `<automated>` verification or Wave 0 dependencies.
- [ ] Sampling continuity: no three consecutive tasks without automated verification.
- [ ] Wave 0 covers all missing references.
- [ ] No watch-mode flags.
- [ ] Feedback latency is under 180 seconds for host checks.
- [ ] `nyquist_compliant: true` is set in frontmatter after all planned tests exist.

**Approval:** pending

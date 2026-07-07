# Phase 28.1 Deferred Items

Out-of-scope discoveries logged during execution. Not fixed per scope-boundary rule.

## Pre-existing firmware `#[cfg(test)]` bit-rot (found during 28.1-02 Task 2)

- `firmware/bitaxe/src/live_stratum_runtime.rs` test module contains pre-existing
  assertions such as `assert!(consumed)` where `consumed` is a `SocketLineOutcome`
  enum (present at HEAD before this plan, e.g., commit `cd09f88` line ~1688).
  These tests never compile anywhere: `cargo test -p bitaxe-firmware` fails at the
  esp-idf-sys host-target gate, `#[cfg(test)]` is inactive under the xtensa
  `just build`, and `firmware/bitaxe/BUILD.bazel` has no `rust_test` target.
- Impact: none on shipped firmware; the tests are dead text. Risk is confusion and
  false confidence if anyone ever wires a firmware test target without fixing them.
- Suggested disposition: address when/if a firmware host-test harness lands, or
  prune the never-compiled tests in a dedicated chore (candidate for Plan 28.1-04
  cleanup or a later refactor of the 2000-line `live_stratum_runtime.rs`).

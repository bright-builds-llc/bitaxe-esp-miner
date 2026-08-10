# Parity work log

## 2026-08-10T03:43:00Z | Implementation and focused verification

- Source commit: `9841d61d686876511ad24ee84b7d7cde776b0cf3` (plan checkpoint)
- Actions: Generalized the serialized settings transaction from hostname-only
  persistence to every typed write emitted by the validated upstream settings
  planner. Added private exact reread reconciliation, non-secret snapshot
  publication, persistence-only effect classification, an exhaustive
  reference-derived REST fixture, and firmware source-ownership checks.
- Verification: `cargo test -p bitaxe-api --all-features` passed 264 tests;
  `cargo clippy -p bitaxe-api --all-targets --all-features -- -D warnings`
  passed; `bazel test //crates/bitaxe-api:tests
  //firmware/bitaxe:settings_source_ownership_tests --test_output=errors`
  passed; `bazel build //firmware/bitaxe:firmware` produced
  `bazel-bin/firmware/bitaxe/bitaxe-firmware.elf`.
- Evidence: `crates/bitaxe-api/fixtures/api/settings-patch-cases.json` proves
  all 42 upstream REST fields produce the exact 44 typed NVS writes, including
  frequency and fan compatibility mirrors. Pure transaction tests prove
  validation, serialization, failure ordering, commit, private reconciliation,
  non-secret publication, and deferred hostname effect ordering. Firmware
  ownership tests bind the production route and all four NVS adapter types.
- Outcome: Focused software verification passed. The complete mandatory gate
  remains pending before the implementation checkpoint.
- Blocker or next safe action: A direct host-target
  `cargo check -p bitaxe-firmware --all-features` reached `esp-idf-sys` and
  rejected unsupported target `aarch64-apple-darwin`; the repo-owned Bazel
  ESP32-S3 build passed and is the canonical firmware proof. Run the complete
  pre-commit gate next. No hardware access occurred.

## 2026-08-10T04:02:00Z | Bright Builds structure correction

- Source commit: `9841d61d686876511ad24ee84b7d7cde776b0cf3` (plan checkpoint)
- Actions: The first complete pre-commit run passed all Rust checks but Bright
  Builds rejected `crates/bitaxe-api/src/settings/tests.rs` at 694 physical
  lines. Moved two persistence scenarios into the child module
  `settings/tests/persistence_more_tests.rs` and added it to the Bazel target.
- Verification: The focused Cargo and Bazel API suites still pass 264 tests;
  `bun scripts/bright-builds-check.ts all` now reports zero findings; and
  `git diff --check` passes.
- Evidence: The split changes module structure only; the exhaustive transaction
  and serialization scenarios retain their original assertions.
- Outcome: Structural finding resolved without changing behavior.
- Blocker or next safe action: Rerun the complete ordered pre-commit gate after
  the final source layout, then create the implementation checkpoint. No
  hardware access occurred.

# Parity work log

## 2026-08-03T00:18:34Z | static-version root-cause investigation

- Source commit: `6c377f4d7439d4cedb1162899f93647577c1a478`
- Actions: Selected the first deterministic candidate, traced upstream version
  initialization and the Rust package/runtime path, and defined one bounded
  exact-package hardware contract.
- Verification: Clean synchronized `main`; pinned reference
  `c1915b0a63bfabebdb95a515cedfee05146c1d50`; `next-item` returned no open plan
  and `SYS-004` first at `implemented`. Upstream `SYSTEM_init_versions` reads
  `/version.txt`; Rust package input contains no `version.txt`, while
  `platform_identity.rs` returns the generic `assets/release.json` name.
- Evidence: `reference/esp-miner/main/system.c`,
  `reference/esp-miner/main/http_server/system_api_json.c`,
  `firmware/bitaxe/src/platform_identity.rs`,
  `firmware/bitaxe/static/www/assets/release.json`, and
  `scripts/package-firmware.sh`.
- Outcome: The remaining static-semantics gap is an implementation defect, not
  merely missing evidence. The targeted fix must make the installed SPIFFS
  payload own the reported version before consuming the one hardware attempt.
- Blocker or next safe action: Run all plan pre-commit gates, commit and push
  this immutable plan/task contract, then implement the software fix without
  exercising hardware.

## 2026-08-03T00:36:00Z | targeted package/runtime repair

- Source commit: implementation worktree based on
  `033002d10dd77ca4be76c70203aa79156724a51a`.
- Actions: Replaced the embedded fallback UI-name projection with a mounted,
  package-owned version marker; added strict pure parsing; staged the marker
  without mutating checked-in assets; and added a typed exact-package private
  capture classifier with a closed public schema.
- Verification: Focused Cargo API and parity tests passed; the package shell
  regression proved byte-exact `version.txt` content; focused Clippy passed;
  and Bazel `//tools/parity:tests` plus `//scripts:package_firmware_test`
  passed. One initial Bazel invocation named the package test target
  incorrectly and performed no tests; the corrected target passed.
- Evidence: `crates/bitaxe-api/src/platform_identity.rs`,
  `firmware/bitaxe/src/platform_identity.rs`, `scripts/package-firmware.sh`,
  `scripts/package-firmware-test.sh`, and
  `tools/parity/src/sys004_version_evidence.rs`.
- Outcome: The root defect is repaired at the package/runtime ownership
  boundary, and stale or mismatched live/static versions now fail closed.
- Blocker or next safe action: Run every mandatory repository gate, review the
  complete diff, commit and push the clean software source, then create the
  exact package and consume the single detector-gated hardware attempt.

## 2026-08-03T00:50:00Z | software verification gate

- Source commit: implementation worktree based on
  `033002d10dd77ca4be76c70203aa79156724a51a`.
- Actions: Ran the exact ordered Rust pre-commit sequence, the complete Bazel
  test graph, Bright Builds checks, parity validation/progress, redaction,
  reference cleanliness, and whitespace validation.
- Verification: Formatting, Clippy with warnings denied, all-target/all-feature
  build, all-feature Cargo tests, Bright Builds with zero findings, and all 82
  Bazel tests passed. Parity remains 34/94 verified (36.2%), redaction passed,
  and the pinned reference is clean. The first parity report completed its
  report but its process ended with transient host `os error 35`; an unchanged
  immediate rerun completed with `validation_errors: none`, followed by every
  remaining gate passing.
- Evidence: Local command results plus the focused entries above; no generated
  package from the dirty verification worktree is admissible for hardware.
- Outcome: The targeted software repair is ready for a clean source commit.
- Blocker or next safe action: Review and commit the exact diff, push `main`,
  then build a fresh clean package before detector or hardware use.

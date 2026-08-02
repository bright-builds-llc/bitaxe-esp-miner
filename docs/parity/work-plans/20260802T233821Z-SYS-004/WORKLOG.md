# Parity work log

## 2026-08-02T23:43:11Z | version-reporting root-cause audit

- Source commit: `0fabeb6a1277ceaaa8659b44a8d633a0a72d1ed8`
- Actions: Traced upstream `SYSTEM_init_versions` and system-info JSON through
  the current Rust build provenance, platform identity, runtime snapshot, and
  wire serialization. Upstream obtains firmware `version` from the application
  descriptor, `axeOSVersion` from SPIFFS `/version.txt`, and `idfVersion` from
  `esp_get_idf_version()`. Rust projects the compatible three fields and also
  exposes semantic version, source/reference commits, application ELF digest,
  build timestamp, channel, dirty state, and release tag.
- Verification: `cargo test -p bitaxe-api build_identity --all-features` (8
  passed); `cargo test -p bitaxe-api
  system_info_wire_serializes_upstream_field_names_and_encodings
  --all-features` (1 passed); `cargo test -p xtask package_manifest
  --all-features` (8 passed); `cargo test -p bitaxe-api
  runtime_boot_attestation --all-features` (11 passed); `bazel test
  //crates/bitaxe-api:tests //tools/xtask:tests` (2 targets passed); and `bazel
  run //tools/parity:report -- api-compare` (schema 99, captured response 47,
  static route 36, no validation errors).
- Evidence: `reference/esp-miner/main/system.c:149-190` and
  `reference/esp-miner/main/http_server/system_api_json.c:113-115` define the
  upstream contract. `crates/bitaxe-api/src/build_identity.rs`,
  `firmware/bitaxe/build.rs`, and `firmware/bitaxe/src/main.rs` enforce and
  embed canonical build identity. `firmware/bitaxe/src/platform_identity.rs`
  obtains ESP-IDF and static-asset facts;
  `firmware/bitaxe/src/runtime_snapshot.rs:457-476` projects them; and
  `crates/bitaxe-api/src/wire.rs:157-178` retains the compatible field names.
  The redacted Phase 17 system-info capture observes all three upstream fields
  on an Ultra 205, and Phase 21 corroborates the live system-info surface under
  a controlled no-share run. The exact-package V12 identity result additionally
  proves current firmware/reference/ELF/ESP-IDF attestation, but does not prove
  the complete current API response.
- Outcome: The root cause of the stale `SYS-004` status is evidence-accounting
  drift: the row note predates the implemented API projection and canonical
  provenance path. No missing software implementation or broad code change was
  found. The targeted correction is therefore a one-row transition to
  `implemented` with accurate evidence and non-claims, not a firmware rewrite.
- Blocker or next safe action: `verified` still requires a later bounded run
  that binds an exact-current package to a live `/api/system/info` response and
  explicitly resolves whether the Rust static-asset identity is semantically
  equivalent to upstream `/version.txt`. Historical output and boot identity
  alone are insufficient.

## 2026-08-02T23:43:11Z | evidence pre-commit gate

- Source commit: `0fabeb6a1277ceaaa8659b44a8d633a0a72d1ed8`
- Actions: Ran the complete required Rust, managed-rule, Bazel, parity,
  redaction, reference-integrity, and whitespace gates against the plan and
  audited implementation.
- Verification: `cargo fmt --all`; `cargo clippy --all-targets --all-features
  -- -D warnings`; `cargo build --all-targets --all-features`; `cargo test
  --all-features`; `bun scripts/bright-builds-check.ts all` (zero findings);
  `just test` (82 of 82 tests passed, including firmware build/package); `just
  parity` (no validation errors); `just parity-progress` (34/94, 36.2%); `just
  verify-redaction`; `just verify-reference`; and `git diff --check`.
- Evidence: This worklog and the exact implementation/test paths named above.
- Outcome: All plan and focused evidence checks passed. The checklist remains
  unchanged until this evidence is committed and its full commit is supplied
  to the transition ledger.
- Blocker or next safe action: Commit this worklog and task progress, capture
  that commit as `SOURCE_COMMIT`, then transition only `SYS-004` to
  `implemented` and synchronize parity progress.

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

## 2026-08-02T23:46:34Z | guarded transition and progress synchronization

- Source commit: `3df7672f427fc682d2661144d1da4ddfabfc3570`
- Actions: Transitioned only `SYS-004` from `in-progress` to `implemented`
  through the parity ledger, replaced the stale implementation note with the
  exact evidence and non-claims, and immediately synchronized parity progress
  from the committed evidence tree.
- Verification: Transition
  `20260802T234800Z-SYS-004` produced checklist digest
  `2bf35e1dfebf70b807d89030526302f39585eab10f56be0f1727d2c00f6c586d`;
  `sync-progress` appended a hash-chained record for source commit
  `3df7672f427fc682d2661144d1da4ddfabfc3570`; progress remains 34 verified of
  94 active rows (36.2%).
- Evidence:
  `docs/parity/checklist-transitions/20260802T234800Z-SYS-004.json`, the
  `SYS-004` checklist row, and the final record in
  `docs/parity/progress.jsonl`.
- Outcome: The targeted evidence-accounting fix is applied without changing
  any other parity row or overclaiming hardware verification.
- Blocker or next safe action: Run the complete finalization gate, review the
  exact diff, commit it, verify upstream synchronization, and push without
  force. The separate live API/static-asset semantics gate remains open.

## 2026-08-02T23:49:47Z | transition target-format correction

- Source commit: `3df7672f427fc682d2661144d1da4ddfabfc3570`
- Actions: The first final `just parity` run rejected the generated
  Rust-owned-target cell because the transition argument lacked Markdown code
  spans. Before any finalization commit, removed that invalid uncommitted
  receipt and progress record, restored the exact predecessor row, regenerated
  the same one-row transition with code-spanned paths, and synchronized
  progress again from the unchanged evidence commit.
- Verification: The failure was exactly `SYS-004: invalid Rust-owned target:
  Rust-Owned Target must contain at least one code span`. The corrected
  transition `20260802T234800Z-SYS-004` produced checklist digest
  `2590ff82638157130b094949bfc9f921de145ac6bb9bd0a25376865831d94f52`;
  `sync-progress` appended the replacement hash-chained record and again
  reported 34/94 verified (36.2%).
- Evidence: The corrected transition receipt, checklist row, and final progress
  record all contain the same source commit, plan, and selected row; only the
  malformed target formatting changed.
- Outcome: The targeted integration defect is corrected without changing
  status, evidence scope, notes, source binding, or any other row.
- Blocker or next safe action: Re-run the complete finalization gate and commit
  only after every validator accepts the corrected transition.

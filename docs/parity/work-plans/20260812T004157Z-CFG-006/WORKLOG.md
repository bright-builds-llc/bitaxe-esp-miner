# Parity work log

## 2026-08-12T00:41:57Z | selection and audit plan

- Source commit: `8203a96fe59baafe5f86ab8c11a61074c45fd19b`.
- Actions: Selected canonical candidate `CFG-006` with no skips; audited the
  prior implementation, result, golden fixture, public Rust matrix, board
  catalog, and all 21 pinned reference seed files; isolated the missing direct
  reference-to-Rust comparison.
- Verification: Clean synchronized `main`, no open parity plan, exact reference
  commit, complete seed inventory, and row-local non-hardware scope confirmed.
- Evidence: Immutable plan and resumed active task record.
- Outcome: Source-backed validation plan ready for the planning-commit gate.
- Blocker or next safe action: Run the mandatory planning gate, commit and push
  this plan, then edit implementation files.

## 2026-08-12T00:52:00Z | direct reference validation

- Source commit: `ebd77310`.
- Actions: Added a pure typed CSV parser and closed matrix comparator to the
  parity report, bound the report to the public `bitaxe-config` matrix, and
  added nine regressions covering accepted input, missing and extra sources,
  missing and duplicate fields, encoding drift, noncanonical integers, value
  drift, and duplicate matrix identities.
- Verification: Focused Cargo tests, strict parity Clippy, Bazel parity tests,
  and the real `just parity` boundary passed. The full ordered Rust sequence,
  Bright Builds, all 37 Bazel test targets, parity/progress, redaction,
  reference cleanliness, and diff checks also passed.
- Evidence: `just parity` inventoried all checked-out `config-*.cvs` sources,
  compared them directly to `board_profile_defaults()`, and reported
  `validation_errors: none` at pinned reference commit
  `c1915b0a63bfabebdb95a515cedfee05146c1d50`.
- Outcome: The declarative matrix now has independent direct-reference,
  golden, and catalog evidence; no runtime or hardware path changed.
- Blocker or next safe action: Commit and push the source work, record its full
  commit as `SOURCE_COMMIT`, then prepare the verified result and transition.

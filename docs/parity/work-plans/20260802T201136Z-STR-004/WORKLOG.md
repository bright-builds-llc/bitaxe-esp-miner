# Parity work log

## 2026-08-02T20:15:20Z | implementation attempt 1

- Source commit: `c0b15ceb`
- Actions: Audited the pinned coinbase decoder, its unit vectors, the existing
  Rust hashing helpers, and the separate `STR-012` ownership boundary for
  payout-address codecs.
- Verification: The plan commit passed formatting, strict Clippy, the
  all-target/all-feature Cargo build and tests, managed Bright Builds checks,
  the full Bazel test graph, redaction verification, parity validation, and
  parity-progress validation.
- Evidence: The immutable audit plan is commit `c0b15ceb`; implementation
  evidence is pending.
- Outcome: Proceed with a bounds-checked transaction decoder and executable
  pinned vectors, without representing address strings or user-payout matches.
- Blocker or next safe action: Implement the software-only decoder and focused
  regression coverage, then run every scoped and mandatory gate.

## 2026-08-02T20:20:59Z | implementation verification complete

- Source commit: `c0b15cebd55cf80f4e060b9eda2deaac95f7083d`
- Actions: Added a typed coinbase transaction decoder, exact CompactSize and
  split-transaction parsing, network difficulty, BIP-34 height, printable pool
  tag extraction, bounded output retention and totals, standard script-shape
  classification, and BIP-54/BIP-110 decisions. Added independently
  constructed pinned fixture data and eleven focused tests.
- Verification: `jq empty`, focused Cargo tests, strict crate Clippy, the
  Stratum Bazel target, reference cleanliness, formatting, workspace strict
  Clippy, all-target/all-feature Cargo build and tests, managed Bright Builds
  checks, redaction verification, the full Bazel test graph, `just parity`, and
  `just parity-progress` all passed. The pre-transition baseline is 30 of 94
  active rows verified (31.9%).
- Evidence: The fixture covers all four CompactSize widths, every upstream
  script shape without address rendering, exact decoded fields and totals,
  output retention, disabled projections, truncation and split rejection, and
  the BIP signal boundaries.
- Outcome: Source changes are ready for the required implementation commit
  before any checklist transition.
- Blocker or next safe action: Review and commit the source/evidence diff, bind
  `RESULT.md` to that immutable commit, then transition only `STR-004`.

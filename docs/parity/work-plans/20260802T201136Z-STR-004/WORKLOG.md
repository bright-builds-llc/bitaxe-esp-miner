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

## 2026-08-02T20:23:48Z | checklist transition complete

- Source commit: `b55228706d28f9b34d71a092656ef3ca6f3f649a`
- Actions: Bound `RESULT.md` to the immutable implementation commit,
  transitioned only `STR-004` from `implemented` to `verified`, and synchronized
  the deterministic progress ledger.
- Verification: Transition receipt
  `20260802T202626Z-STR-004` binds the predecessor and result checklist digests,
  plan and result hashes, pinned reference commit, exact Rust-owned targets,
  and unchanged `unit,golden` evidence requirement. Progress synchronization
  reports 31 of 94 active rows verified (33.0%).
- Evidence: `docs/parity/checklist-transitions/20260802T202626Z-STR-004.json`
  and the hash-chained `docs/parity/progress.jsonl` entry.
- Outcome: `STR-004` is verified for its deterministic software surface.
- Blocker or next safe action: Archive the completed task, rerun the mandatory
  finalization gate, commit the transition artifacts, then rebase safely and
  push without force.

## 2026-08-02T20:26:26Z | finalization attempt 1 corrected

- Source commit: `b55228706d28f9b34d71a092656ef3ca6f3f649a`
- Actions: Ran the post-transition gate. Parity validation found that the first
  generated checklist projection omitted Markdown code spans around the
  Rust-owned paths. Removed the uncommitted invalid projection, receipt, and
  derived progress record together; then reran the transition and progress
  synchronization through the repository tools with correctly formatted
  targets.
- Verification: The replacement hash-chained receipt is
  `20260802T202626Z-STR-004`; progress again reports 31 of 94 active rows
  verified (33.0%). The failed projection was never committed.
- Evidence: The finalization gate passed redaction, formatting, strict Clippy,
  all-target/all-feature Cargo build and tests, managed Bright Builds checks,
  and all 82 Bazel tests before the parity validator exposed the formatting
  defect.
- Outcome: The authoritative checklist is validly projected through the
  replacement receipt; no row scope or evidence claim changed.
- Blocker or next safe action: Rerun the complete finalization gate from the
  start and require every command, including parity validation, to pass.

## 2026-08-02T20:28:18Z | finalization verification complete

- Source commit: `b55228706d28f9b34d71a092656ef3ca6f3f649a`
- Actions: Re-ran the post-transition checks with fail-fast shell semantics,
  then retried only the parity report after a transient host resource error.
- Verification: Redaction, formatting, strict Clippy, the all-target/all-feature
  Cargo build and tests, managed Bright Builds checks, all 82 Bazel tests,
  parity validation, and progress validation passed. The final progress output
  is 31 of 94 active rows verified (33.0%).
- Evidence: Exit status zero from every mandatory finalization command; the
  parity retry used bounded local stdout only and did not alter repository
  state.
- Outcome: The implementation, result binding, transition ledger, progress
  history, README projection, and archived task are ready for final review and
  commit.
- Blocker or next safe action: Review the complete diff, commit finalization,
  fetch and rebase only if conflict-free, then push without force.

# Parity work log

## 2026-08-20T17:34:36Z | durable scoreboard projection correction

- Source commit: `4594760b08e606959d952a1fc7803095967e5bf2`.
- Actions: added an exact IEEE-754 ties-to-even one-decimal projection matching
  pinned Rust `{:.1}` and upstream C `%.1f`; retained raw and durable scoreboard
  digests; changed restart persistence to compare the pre-restart durable digest
  with the raw post-restart digest while preserving exact same-boot repeats,
  count, order, and every non-difficulty field.
- Regressions: the full real-child workflow now proves full-precision runtime
  difficulty becomes only its durable one-decimal value after restart. Pure
  cases cover midpoint rounding, wrong durable difficulty, changed nonce, and
  reordered entries; the real-child workflow also withholds projection on
  post-restart repeat drift and count drift.
- Source binding: the unchanged 31-path inventory now requires the Rust
  `"{:.1}"` persistence format plus upstream `%.1f` write and `%lf` reload
  fragments.
- Verification infrastructure: added `.bazelignore` entries for repo-local
  generated/ignored trees. This prevents `bazel test //...` from traversing
  protected `scratch/` evidence and the multi-megabyte `target/debug/deps`
  directory index; the exact `just test` command then passed all 48 targets in
  66 seconds on its first clean run and 27 seconds in the final cached run.
- Verification: `cargo fmt --all`; isolated-target `cargo clippy --all-targets
  --all-features -- -D warnings`, `cargo build --all-targets --all-features`,
  and `cargo test --all-features` including all doctests; Bright Builds; focused
  automation; all 48 Bazel tests; firmware build/package; redaction; reference;
  parity; progress; selector; and diff checks passed.
- Outcome: software correction complete. No credentials, protected attempt
  artifacts, detector, device, USB, external network runtime, flash, monitor,
  mining, share submission, restart, evidence projection/promotion, recovery,
  or attempt-006 was used.
- Blocker or next safe action: close without parity transition. A separately
  authorized immutable evidence plan may re-evaluate the corrected verifier
  against eligible live evidence or authorize a new bounded attempt.

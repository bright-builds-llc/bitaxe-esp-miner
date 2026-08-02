# Parity work result

- Parity row: `STAT-004`
- Final status: `verified`
- Implementation commit: `8a89a7e50db2abeaba3f6cd5173c7536c0b72d9c`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`

## Evidence and verification

The executable fixture
`crates/bitaxe-stratum/fixtures/v1/work-queue-cases.json` binds `STAT-004` to
the pinned `work_queue.c` and `work_queue.h` reference sources. Ten focused
queue tests cover the exact twelve-item capacity, initial empty state, FIFO
ordering across storage reuse, unchanged contents after a full-boundary signal,
unchanged state after an empty-boundary signal, and deterministic destruction
of every queued item on clear. Existing mining-queue tests continue to cover
clean-jobs queue and valid-job invalidation as supporting integration evidence.

The following commands passed against the implementation commit or its
reviewed pre-commit worktree:

- `cargo test -p bitaxe-stratum work_queue --all-features`
- `bazel test //crates/bitaxe-stratum:tests`
- `bazel run //scripts:verify_reference_clean`
- `jq empty crates/bitaxe-stratum/fixtures/v1/work-queue-cases.json`
- `cargo fmt --all`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo build --all-targets --all-features`
- `cargo test --all-features`
- `bun scripts/bright-builds-check.ts all`
- `just verify-redaction`
- `just test`
- `just parity`
- `just parity-progress`
- `git diff --check`

Reference cleanliness reported
`c1915b0a63bfabebdb95a515cedfee05146c1d50`, all 82 Bazel tests passed, parity
reported no validation errors, and the pre-transition baseline remained 31 of
94 active rows verified (33.0%).

## Conclusion

The `unit,golden` evidence proves the complete `STAT-004` queue data-structure
contract claimed by the checklist: exact bounded capacity, FIFO storage and
reuse, clear/drop behavior, and fail-closed backpressure signals that preserve
state for the imperative shell. The fixture owns explicit reference provenance
and is executed by both Cargo and Bazel tests. This is sufficient to promote
`STAT-004` from `implemented` to `verified` without a hardware claim.

## Non-claims and residual risks

Condition-variable blocking, timed-wait clock behavior, pthread or FreeRTOS
wakeup ordering, and task scheduling remain owned by unverified `SYS-005`.
Clean-jobs generation, live Stratum sockets, pool reconnects, credentials,
ASIC dispatch, result correlation, share acceptance or rejection, mining
timing, and hardware behavior remain owned by their separate checklist rows.
The Rust queue uses explicit `QueueFull` and `QueueEmpty` signals so its
imperative shell can provide backpressure; this result does not claim those
signals are a literal pthread implementation.

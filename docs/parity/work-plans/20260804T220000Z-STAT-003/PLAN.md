# Parity work plan

- Run ID: `20260804T220000Z-STAT-003`
- Parity row: `STAT-003`
- Initial status: `in-progress`
- Source commit: `2247df1b6421b5482e0caa35afb21e5b195eb04c`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-stat003-scoreboard`

## Selection

The deterministic selector reported no open plan. Earlier `implemented` rows
remain gated by existing purpose-bound hardware, networking, mining,
safety-control, release, unavailable-board, or unavailable-accessory evidence.
CFG-001 in particular has no authorized retry after its bounded soak lineage
closed at `stop_repeated_boundary`. The display/input rows need physical user
evidence, and ASIC-009, ASIC-010, and BAP-001 need unavailable hardware.

STAT-003 is the next software-actionable row. Its HTTP wire mapping exists, but
the runtime projection always returns an empty array. The production work
registry already correlates current-generation BM1366 nonce observations to the
exact pool job and reconstructs the nonce difficulty; the missing behavior is a
typed valid-nonce receipt, exact top-20 retention, indexed NVS durability, and
one API-visible owner.

## Scope and non-scope

Implement the pinned scoreboard contract as a pure bounded owner. Admit every
valid current-generation nonce observation after exact difficulty computation,
including below-pool-target and duplicate candidates, because upstream records
the scoreboard independently of share-submission outcome. Reject invalid,
stale, uncorrelated, or target-computation-failed observations. Retain at most
20 entries ordered by descending difficulty, preserve stable order for equal
difficulty, and ignore a full-board candidate no better than the last entry.

Encode and parse the exact upstream indexed persistence shape
`difficulty;job_id;extranonce2;ntime;nonce;version_bits`, with one-decimal
difficulty persistence and bounded 31-byte job/extranonce fields. Load
`scoreboard_01` through `scoreboard_20` at boot, stop at the first missing or
empty slot, skip malformed nonempty slots with a category-only warning, and
publish an insertion only after the changed suffix commits and reloads exactly.
The production session emits one redacted scoreboard-record effect; the
firmware adapter owns NVS and the API reads a clone without mutation.

This work does not authorize a device flash, monitor, credentials, network
access, mining campaign, pool connection, voltage/frequency/fan/power effect,
OTA, recovery, direct UART, or pin interaction. Live nonce observations,
hardware-derived difficulty, device persistence, API/browser behavior, mining,
and share outcomes remain below verified.

## Implementation

- [ ] Add the pure top-20 scoreboard, exact stable insertion and full-board
      rules, bounded persistence codec, mutation receipt, and focused tests.
- [ ] Carry a redacted valid-nonce scoreboard candidate through production
      correlation and one typed firmware effect without weakening submission
      deduplication, target validation, generation checks, or debug redaction.
- [ ] Add the transactional indexed-NVS firmware owner, boot initialization,
      read-only HTTP projection, source-ownership regressions, and real firmware
      build coverage.
- [ ] Run every mandatory gate and create a commit-bound result before
      transitioning only STAT-003 to `implemented`.

## Verification and promotion

Focused tests must cover empty load, persistence round-trip, malformed and
oversized fields, insertion ordering, equal-difficulty stability, below-target
and duplicate valid nonce admission, full-board ignore/eviction, changed-suffix
receipts, failed persistence withholding publication, and request-time
immutability. Production tests must prove one redacted effect after valid nonce
correlation and no effect for stale/uncorrelated/invalid observations.

Run `cargo fmt --all`, strict all-target/all-feature Clippy, all-target/all-
feature build, all-feature tests, Bright Builds checks, `just test`, `just
parity`, `just parity-progress`, redaction, reference cleanliness, and diff
checks. Transition only STAT-003 from `in-progress` to `implemented` with
`unit,workflow,api-compare` evidence when all gates pass. Do not claim live
mining, accepted or rejected pool responses, device persistence, UI behavior,
or verified status.

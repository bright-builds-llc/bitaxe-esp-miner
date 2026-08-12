# Parity work plan

- Run ID: `20260812T091446Z-ASIC-003`
- Parity row: `ASIC-003`
- Initial status: `implemented`
- Source commit: `29fba85041c0e4338a512acb3d251ae9363e066f`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-asic003-sealed-work-send-promotion`

## Selection

The canonical selector returned no open plan and listed `ASIC-003` first,
followed by `ASIC-004`, `ASIC-005`, and the remaining unfinished rows.
`ASIC-003` is actionable without skipping another candidate. Its current row
already has unit, golden, and diagnostic hardware-smoke evidence, while the
committed `ASIC-002` projection now supplies the missing exact-package live
boundary: initialized BM1366 work necessarily produced a qualified correlated
nonce and an accepted submit response in the sealed campaign.

Preflight found `main` clean, checked out, tracking `origin/main`, and exactly
synchronized after fetch. The read-only reference is clean at the commit
above. The accepted hardware source is clean commit
`3e0966a140edbff1a14d2a48ca63d140649762c0`. The three byte-level BM1366 work
and command modules are byte-identical from that commit through current HEAD;
the current adapter and worker retain the same bounded production dispatch and
UART-write spans even though unrelated result-poll and monitoring code changed
elsewhere in those files.

## Scope and non-scope

This run will derive one redacted `bitaxe-asic-work-send-evidence-v1`
projection from the already committed and independently validated
`bitaxe-asic-initialization-evidence-v1` artifact. The new projector must bind
that artifact's digest and trusted exact-package campaign facts, prove the
source commit identities, require exact compatibility of
`crates/bitaxe-asic/src/bm1366/work.rs`,
`crates/bitaxe-asic/src/bm1366/production.rs`, and
`crates/bitaxe-asic/src/bm1366/command.rs`, and compare bounded semantic source
spans for production worker dispatch and adapter UART write between the
hardware commit and current source. Current behavior tests must prove the
fixed 82-byte payload, 88-byte job frame, job-ID packing/advance behavior,
typed `WriteFrame` action, production-ready gate, worker dispatch, and
fail-closed UART error handling.

No protected campaign input needs to be reopened. No detector, flash, reset,
USB session, credential read, serial or network request, mining lease,
fan/voltage/power/ASIC actuation, recovery, direct UART, pin manipulation, or
other hardware effect is permitted. The public projection must not contain raw
work bytes, headers, nonces, targets, difficulty, pool or Wi-Fi values,
endpoints, ports, users, workers, owner addresses, credentials, USB/network
identifiers, device paths, local paths, secrets, or secret-derived hashes.

This row does not claim result parsing or result correlation semantics,
frequency transition, serial transport under arbitrary load, Stratum socket
behavior, target validation, submission policy, default-profile soak,
voltage/fan/power/thermal behavior, other ASICs or boards, updates, recovery,
profitability, or release readiness.

## Implementation

- [ ] Add a Rust-owned closed evidence contract and independent validator for
      the exact BM1366 production-work-send projection.
- [ ] Add a thin host projector that validates the committed ASIC-002 source
      projection, source identities, byte-identical work modules, bounded
      dispatch/UART-write spans, and clean relevant worktree before publishing
      only closed facts.
- [ ] Add behavior-focused regressions for malformed/incomplete source
      evidence, digest drift, commit/path/span drift, dirty relevant paths,
      validator failure, sensitive output, and a real child-process/file seam.
- [ ] Produce the checklist's required public evidence without interacting
      with hardware or protected campaign inputs.

## Verification and promotion

Focused verification will run the new Rust contract tests, host projection
tests, real-child integration, BM1366 work/production tests, adapter and worker
dispatch tests, and direct validation of
`docs/parity/evidence/asic003-work-send/asic-work-send-projection.json`. The
mandatory ordered repository gate is:

1. `cargo fmt --all`
2. `cargo clippy --all-targets --all-features -- -D warnings`
3. `cargo build --all-targets --all-features`
4. `cargo test --all-features`
5. `bun scripts/bright-builds-check.ts all`
6. `just test`
7. `just parity`
8. `just parity-progress`

Also require the real ESP32-S3 firmware image, generated automation contracts,
`just verify-redaction`, `just verify-reference`, exact reference cleanliness,
task uniqueness, immutable-plan digest, public-sensitive-value scan, source-
projection validation/digest binding, source compatibility, and
`git diff --check`.

Promote only `ASIC-003` from `implemented` to `verified` with
`unit,golden,workflow,hardware-smoke,hardware-regression` if the closed proof
shows the exact admitted Ultra 205 package completed mining-ready
initialization, dispatched live production work through the retained UART,
received a qualified correlated result that led to an accepted submit
response, remained safe, stopped cleanly, and the exact byte-level work stack
plus bounded production dispatch/write spans remain compatible with current
source. Any malformed, incomplete, digest-mismatched, source-drifted, dirty,
validator-rejected, or sensitive input withholds evidence and leaves the row
`implemented`; there is no hardware retry or effect path in this plan.

# Parity work plan

- Run ID: `20260812T093928Z-ASIC-004`
- Parity row: `ASIC-004`
- Initial status: `implemented`
- Source commit: `12f30445a0fdd70b92a0e3b067c06c7af34fe2fa`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-asic004-sealed-result-parsing-promotion`

## Selection

The canonical selector returned no open plan and listed `ASIC-004` first,
followed by `ASIC-005`, `ASIC-007`, and the remaining unfinished rows.
`ASIC-004` is actionable without skipping another candidate. Its current row
has unit, golden, and diagnostic hardware-smoke evidence. The committed
`ASIC-003` projection now closes the live boundary by proving that exact-
package BM1366 work produced a qualified correlated result and accepted submit
response through the production path.

Preflight found `main` clean, checked out, tracking `origin/main`, and exactly
synchronized after fetch. The read-only reference is clean at the commit
above. The accepted hardware source is clean commit
`3e0966a140edbff1a14d2a48ca63d140649762c0`. Since that attempt, result-parser
changes add closed soft-discard classifications; the accepted nonce decoding,
frame validation, adapter classification, worker emission, and production
correlation spans remain semantically compatible and can be compared as
unique bounded source spans.

## Scope and non-scope

This run will derive one redacted `bitaxe-asic-result-parsing-evidence-v1`
projection from the committed and independently validated ASIC-003 work-send
artifact. The projector must bind that artifact's digest and trusted live
qualified-result facts, prove source identities, require the unchanged
transcript module, and compare unique bounded source spans for strict frame
validation, nonce decoding, adapter classification, worker nonce emission,
and production correlation between the accepted hardware commit and current
source. Current tests must prove the 11-byte result frame, preamble and CRC
checks, job-ID lookup, little-endian submit nonce, core/address validation,
version-bit recovery, known register parsing, all closed discard categories,
and soft-discard continuation.

No protected campaign input needs to be reopened. No detector, flash, reset,
USB session, credential read, serial or network request, mining lease,
fan/voltage/power/ASIC actuation, recovery, direct UART, pin manipulation, or
other hardware effect is permitted. The public projection must not contain raw
frames, nonces, targets, difficulty, pool or Wi-Fi values, endpoints, ports,
users, workers, owner addresses, credentials, USB/network identifiers, device
paths, local paths, secrets, or secret-derived hashes.

This row does not claim work encoding, arbitrary-load serial transport,
frequency transitions, Stratum socket behavior, target-validation or submit
policy, default-profile soak, voltage/fan/power/thermal behavior, other ASICs
or boards, updates, recovery, profitability, or release readiness.

## Implementation

- [ ] Add a Rust-owned closed evidence contract and independent validator for
      the exact BM1366 production-result parsing projection.
- [ ] Add a thin host projector that validates the committed ASIC-003 source
      projection, source identities, unchanged transcript module, bounded
      parser/adapter/worker/correlation spans, and clean relevant worktree
      before publishing only closed facts.
- [ ] Add behavior-focused regressions for malformed/incomplete source
      evidence, digest drift, commit/path/span drift, dirty relevant paths,
      validator failure, sensitive output, and a real child-process/file seam.
- [ ] Produce the checklist's required public evidence without interacting
      with hardware or protected campaign inputs.

## Verification and promotion

Focused verification will run the new Rust contract tests, host projection
tests, real-child integration, BM1366 result/transcript tests, adapter and
worker result tests, production-correlation tests, and direct validation of
`docs/parity/evidence/asic004-result-parsing/asic-result-parsing-projection.json`.
The mandatory ordered repository gate is:

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

Promote only `ASIC-004` from `implemented` to `verified` with
`unit,golden,workflow,hardware-smoke,hardware-regression` if the closed proof
shows the exact admitted Ultra 205 package produced a live BM1366 result that
passed strict parsing and correlation before an accepted submit response, and
the accepted decoding path remains compatible with current source while
current malformed inputs fail closed as typed soft discards. Any malformed,
incomplete, digest-mismatched, source-drifted, dirty, validator-rejected, or
sensitive input withholds evidence and leaves the row `implemented`; there is
no hardware retry or effect path in this plan.

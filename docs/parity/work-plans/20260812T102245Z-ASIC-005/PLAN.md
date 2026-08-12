# Parity work plan

- Run ID: `20260812T102245Z-ASIC-005`
- Parity row: `ASIC-005`
- Initial status: `implemented`
- Source commit: `b25fe27f071c2cc1fbb5d52cce2b205295a62f5b`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-asic005-serial-transport-promotion`

## Selection

The canonical selector returned no open plan and listed `ASIC-005` first,
followed by `ASIC-007`, `STR-001`, and the remaining unfinished rows.
`ASIC-005` is actionable without skipping another candidate. The accepted
Ultra 205 hardware session already produced the two independently validated
public projections required to close this boundary: `ASIC-003` proves a live
production work frame traversed the retained UART path before a qualified
result and accepted submit response, and `ASIC-004` proves the corresponding
live result traversed the retained UART path and strict parser.

Preflight found `main` clean, checked out, tracking `origin/main`, and exactly
synchronized after fetch. The read-only reference is clean at the commit
above. The accepted hardware source is clean commit
`3e0966a140edbff1a14d2a48ca63d140649762c0`; both the complete UART transport
module and its adapter surface are byte-identical between that commit and the
plan source. Current production TX and RX spans remain compatible with the
accepted live observations.

## Scope and non-scope

This run will derive one redacted `bitaxe-asic-serial-transport-evidence-v1`
projection from the committed and independently validated ASIC-003 work-send
and ASIC-004 result-parsing projections. The projector must bind both source
artifact digests, trusted runtime facts, source identities, the unchanged UART
module and adapter, and unique bounded production TX/RX spans. Current source
and tests must prove 115200-baud 8N1 initialization without flow control,
Ultra 205 TX/RX pin bindings, bounded TX completion, exact full-frame writes,
an absolute RX deadline, partial-read accumulation, idle-timeout handling,
partial-frame rejection with RX cleanup, and the production work/result
callers that use those primitives.

No protected campaign input needs to be reopened. No detector, flash, reset,
USB session, credential read, serial or network request, mining lease,
fan/voltage/power/ASIC actuation, recovery, direct UART, pin manipulation, or
other hardware effect is permitted. The public projection must not contain
raw frames or traces, nonces, targets, difficulty, pool or Wi-Fi values,
endpoints, ports, users, workers, owner addresses, credentials, USB/network
identifiers, device paths, local paths, secrets, or secret-derived hashes.

This row does not claim arbitrary baud-rate or board support, direct external
UART use, frequency-transition correctness, Stratum socket behavior,
target-validation or submit policy, default-profile soak, voltage/fan/power/
thermal behavior, other ASICs or boards, updates, recovery, profitability, or
release readiness.

## Implementation

- [ ] Add a Rust-owned closed evidence contract and independent validator for
      the exact BM1366 serial-transport projection.
- [ ] Add a thin host projector that validates both committed source
      projections, source identities, unchanged UART ownership, bounded
      production TX/RX spans, and clean relevant paths before publishing only
      closed transport facts.
- [ ] Add behavior-focused regressions for malformed/incomplete source
      evidence, digest drift, commit/path/span drift, dirty relevant paths,
      validator failure, sensitive output, and a real child-process/file seam.
- [ ] Preserve generated-contract file-size compliance by simplifying the
      synchronized Rust/TypeScript representation rather than adding an
      exception.
- [ ] Produce the checklist's required public evidence without interacting
      with hardware or protected campaign inputs.

## Verification and promotion

Focused verification will run the new Rust contract tests, host projection
tests, real-child integration, UART transport tests, production work/result
tests, and direct validation of
`docs/parity/evidence/asic005-serial-transport/asic-serial-transport-projection.json`.
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
task uniqueness, immutable-plan digest, public-sensitive-value scan, both
source-projection validators and digest bindings, source compatibility, and
`git diff --check`.

Promote only `ASIC-005` from `implemented` to `verified` with
`unit,golden,workflow,hardware-smoke,hardware-regression` if the closed proof
shows that the exact admitted Ultra 205 package transmitted production work
and received its live qualified result through the same unchanged bounded UART
transport before an accepted submit response, while current tests prove the
transport fails closed on partial writes and partial frames. Any malformed,
incomplete, digest-mismatched, source-drifted, dirty, validator-rejected, or
sensitive input withholds evidence and leaves the row `implemented`; there is
no hardware retry or effect path in this plan.

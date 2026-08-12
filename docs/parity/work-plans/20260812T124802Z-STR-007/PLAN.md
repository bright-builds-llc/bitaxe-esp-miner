# Parity work plan

- Run ID: `20260812T124802Z-STR-007`
- Parity row: `STR-007`
- Initial status: `implemented`
- Source commit: `e4f460ec8f25e1600a946c5ad2654753d8e1c42b`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-str007-mining-criteria-promotion`

## Selection

The clean synchronized selector reports no open plan and selects `STR-007`
first, followed by `API-009` and the remaining unfinished rows. No row is
skipped. This row is the bounded mining smoke and soak criteria surface. It is
distinct from the active stricter default-profile continuity task, whose
attempt-004 ended at `stop_repeated_boundary`, consumed its hardware authority,
and does not authorize attempt-005 or parity promotion.

Committed public evidence already supplies the exact bounded proof needed by
this row: Phase 21 records a detector-gated controlled no-share smoke and
300-second approved bounded controlled no-share soak with trusted package boot,
pool lifecycle, subscribe/authorize/notify, typed work dispatch, watchdog
checkpoints, redacted HTTP/WebSocket telemetry, safe stop, and passed redaction.
The verified `STR-006` projection separately binds a later accepted conservative
Ultra 205 production lifecycle and current coordinator compatibility. Current
typed campaign source and tests enforce an exact 600-second smoke/soak duration,
upstream-default soak profile, active-duration accounting, accepted-share and
network-correlation gates, full-duration rejection, terminal safe stop,
cleanup, private evidence, and redaction.

## Scope and non-scope

Create one redacted `bitaxe-mining-criteria-evidence-v1` projection that binds
the exact Phase 21 summary, smoke, and soak document digests; independently
validates their closed public facts; binds the exact verified STR-006 projection
and validator; and proves current criteria from clean source with unique,
ordered semantic spans and focused tests. Publication must use a mode-0600
candidate, independent Rust validation, atomic rename, final mode 0644, and a
closed public denylist.

No protected campaign artifact may be opened, copied, summarized beyond its
already committed task record, or promoted. No detector, package build, flash,
reset, USB/network session, credential input, mining, pool contact,
fan/voltage/power/ASIC actuation, recovery, direct UART, pins, or other hardware
effect is permitted. Attempt-005 is not authorized. The result must not claim
that the unresolved attempt-004 continuity ceiling passed.

This row does not claim accepted or rejected share behavior during a soak,
successful current 600-second default-profile continuity, the active parent
soak task, uninterrupted HTTP/WebSocket or watchdog continuity, automatic
controls, arbitrary pools, profitability, unbounded mining, TLS, Stratum v2,
other boards, updates, recovery, or release readiness.

## Implementation

- [ ] Add a Rust-owned closed evidence contract and independent validator for
      the bounded mining criteria projection.
- [ ] Add a host projector that validates the exact public Phase 21 documents,
      the verified STR-006 projection, current source identity and cleanliness,
      and the unique current smoke/soak admission, active-duration, terminal,
      safe-stop, evidence-sealing, and redaction spans.
- [ ] Add behavior-focused regressions for malformed or digest-drifted public
      evidence, incomplete historical facts, source/span/dirty-path drift,
      incomplete current criteria, validator failure, sensitive output, and a
      real child-process/file seam.
- [ ] Publish the projection only after every closed gate passes.

## Verification and promotion

Run focused contract, projector, real-child, production-session lifecycle, and
campaign tests, then the mandatory ordered repository gate:

1. `cargo fmt --all`
2. `cargo clippy --all-targets --all-features -- -D warnings`
3. `cargo build --all-targets --all-features`
4. `cargo test --all-features`
5. `bun scripts/bright-builds-check.ts all`
6. `just test`
7. `just parity`
8. `just parity-progress`

Also require generated-contract verification, independent validation of both
closed projections, `just verify-redaction`, `just verify-reference`, exact
reference cleanliness, task uniqueness, immutable-plan and admitted-input
digests, final mode, sensitive-value scan, source cleanliness, and
`git diff --check`.

Promote only `STR-007` from `implemented` to `verified` with
`workflow,hardware-smoke,soak` when the closed projection proves the committed
bounded smoke/soak evidence and the current fail-closed criteria without
reinterpreting or reopening the terminal attempt-004 continuity task. Any
failure withholds evidence and leaves the row implemented; there is no retry,
hardware fallback, or expanded authority in this plan.

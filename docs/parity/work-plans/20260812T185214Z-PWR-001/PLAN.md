# Parity work plan

- Run ID: `20260812T185214Z-PWR-001`
- Parity row: `PWR-001`
- Initial status: `implemented`
- Source commit: `ade14353358c72809cd32e69341b3a676a22b02e`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-pwr001-asic-reset-evidence-audit`

## Selection

The clean synchronized selector ranks `API-009` first and `PWR-001` second.
`API-009` is temporarily unavailable for a safe fresh attempt because its next
ordinal requires an operator who explicitly reports being present and watching
before the package, detector, and campaign start; no such occurrence is
present in this invocation. Repeating the campaign without that pre-effect
condition would consume another ordinal while making the required physical
IDENTIFY observation impossible. This is not a terminal skip and does not
weaken or infer the missing observation.

`PWR-001` is the first currently actionable candidate. The sealed Ultra 205
accepted-share attempt already exercised the production nine-step preparation
transaction. Its validated `ASIC-002` projection proves exactly 18 accepted
started/completed preparation events, no invalid events, terminal
`retain_production_uart/completed`, exactly one BM1366 detected, trusted exact-
package runtime identity, live accepted work, fresh safety, safe stop, cleanup,
and no later hardware rerun. The deterministic preparation source places
`ResetAndDetectExactlyOneChip` before mining-ready initialization, and the
unchanged adapter emits the active-low 100 ms/100 ms reset pulse before chip
detection. The reset-owning paths are byte-identical between accepted-attempt
commit `3e0966a140edbff1a14d2a48ca63d140649762c0` and this plan's source
commit. This is row-specific hardware-regression evidence available for a
typed audit without touching the device again.

The active lesson set remains above its deterministic loading budget with the
unchanged 2026-08-03 audit baseline and no new audit trigger. The complete
safety, authorization, evidence, retry, redaction, physical-observation,
earliest-failure, real-process, ESP-IDF, and host-stall lesson blocks remain
loaded; the previously disclosed unrelated omitted set remains unchanged.

## Scope and non-scope

Add a typed `bitaxe-asic-reset-evidence-v1` functional-core contract and a
narrow automation projector that derives only closed PWR-001 facts from the
existing validated `bitaxe-asic-initialization-evidence-v1` projection,
committed task/plan identity, the pinned reference, and exact Git path
compatibility. The projection must bind the source projection digest and
commits, prove active-low GPIO reset semantics with 100 ms low and 100 ms high
durations, prove the reset-and-detect preparation boundary completed before
exactly-one-chip and accepted-work observations, and retain fail-closed
hold-low, safe-stop, cleanup, no-rerun, and redaction facts.

The projector must independently validate the source projection through the
Rust contract validator, reject incomplete or mutated source evidence, reject
any change to reset-owning paths since the accepted attempt, write through a
private candidate, validate the candidate, and publish atomically only after
the complete quorum passes. Public output may contain only schemas, commits,
digests, fixed counts/durations, closed categories, and booleans.

Repair the parity selector's newly reproduced cross-row reconciliation bug as
a prerequisite: discard a fully terminal-closed explicit continuation lineage
before checking whether remaining open plans span multiple rows. Preserve
strict failure for genuinely simultaneous open rows and for unlinked same-row
plans. Add a focused regression for a closed API-009 lineage followed by this
open PWR-001 plan.

No detector, flash, reset, USB session, serial monitor, network request,
credential access, mining, GPIO effect, direct UART, pin manipulation, fault
injection, voltage/fan/power change, or other hardware interaction is in
scope. This plan does not measure electrical waveforms or independently prove
ESP-IDF scheduling accuracy; it proves the observable production reset
transaction through the exact admitted firmware, downstream chip response,
accepted work, and unchanged reset implementation. Other power, voltage,
thermal, sensor, ASIC, mining, or board rows remain out of scope.

## Implementation

- [ ] Add the minimum typed reset-evidence contract, generated bindings,
      projector command, validator route, and human command surface.
- [ ] Fix terminal-lineage reconciliation in the selector and add the exact
      closed-previous-row/open-current-row regression without weakening real
      multi-row conflict detection.
- [ ] Add behavior-focused Rust and TypeScript regressions for complete proof,
      mutated/incomplete source evidence, source-path drift, workflow identity,
      publication withholding, and sensitive-output absence.
- [ ] Produce and independently validate one redacted public PWR-001 evidence
      projection from the already sealed attempt; do not rerun hardware.

## Verification and promotion

Focused tests must prove that only a validator-accepted ASIC initialization
projection with its exact SHA-256, trusted package/runtime identity, complete
nine-step preparation, exactly one detected chip, accepted live work, safe
stop, cleanup, no hardware rerun, and unchanged reset-owning paths can produce
the reset projection. They must prove active-low 100 ms/100 ms semantics,
fail-closed hold-low ownership, source/reference/task/workflow binding,
candidate cleanup, final-evidence withholding on every failed quorum member,
and absence of hostnames, origins, ports, USB/network identifiers,
credentials, raw traces, or private paths from public output.
The selector regression must prove the latest terminal closure retires its
explicit same-row lineage before cross-row conflict detection, while two
genuinely open rows and unlinked same-row plans still fail closed.

Run, in order:

1. `cargo fmt --all`
2. `cargo clippy --all-targets --all-features -- -D warnings`
3. `cargo build --all-targets --all-features`
4. `cargo test --all-features`
5. `bun scripts/bright-builds-check.ts all`
6. `just test`
7. `just parity`
8. `just parity-progress`

Also require focused automation tests, exact generated contracts, independent
Rust validation of both source and final projections, `just verify-redaction`,
`just verify-reference`, selector and unique-task binding, immutable-plan
digest, reference cleanliness, sensitive-output review, `git diff --check`,
and full diff review. Commit and push this immutable plan/task checkpoint
before implementation.

Promote `PWR-001` from `implemented` to `verified` only if the final closed
projection passes every validator and repository gate and establishes that
the exact admitted Ultra 205 firmware completed the upstream-aligned active-
low reset pulse, obtained the downstream exactly-one-chip response, advanced
to accepted work, then completed fail-closed safe stop and cleanup. Otherwise
retain `implemented`, withhold final evidence, record the exact blocker, and
stop this row without a hardware attempt.

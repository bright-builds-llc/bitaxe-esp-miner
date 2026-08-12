# Parity work plan

- Run ID: `20260812T203223Z-PWR-003`
- Parity row: `PWR-003`
- Initial status: `implemented`
- Source commit: `96d18ba5ec7d4e33c7806a5c4cfac54869934f41`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-pwr003-core-voltage-control-audit`

## Selection

The clean synchronized selector ranks `API-009` first and `PWR-003` second.
`API-009` is temporarily unavailable because its closed attempt-006 requires a
new explicit operator report of being present and watching before any package,
detector, or campaign work. No such fresh pre-effect occurrence exists in this
invocation, so starting that campaign would violate its immutable continuation
contract.

`PWR-003` is the first currently actionable row. The independently validated
`bitaxe-asic-power-initialization-evidence-v1` projection already binds one
accepted Ultra 205 hardware transaction to exact-package and trusted-runtime
identity, fresh safety, an issued conservative 1100 mV core-voltage command, a
complete 500 ms stabilization boundary, active-low ASIC enable, successful
BM1366 initialization and accepted downstream work, safe stop, cleanup, and no
hardware rerun. A source-bound typed audit can close the narrower core-voltage
control row without another device effect.

The active lesson set remains above its deterministic loading budget with the
unchanged 2026-08-03 audit baseline and no new audit trigger. The complete
safety, authorization, evidence, retry, redaction, physical-observation,
earliest-failure, real-process, ESP-IDF, and host-stall lesson blocks remain
loaded. The disclosed unrelated omitted blocks are the caption-VTT, small-row
deduplication, GSD body-separator, and manual-removal lessons.

## Scope and non-scope

Add a typed `bitaxe-core-voltage-control-evidence-v1` functional-core contract
and narrow automation projector. Derive only closed PWR-003 facts from the
independently validated PWR-002 projection, its exact SHA-256 and commits, the
pinned reference, this immutable plan, the active task, and exact source
compatibility from the accepted hardware commit to the current commit.

The projection must bind board 205 and the conservative 1100 mV command to the
Ultra 205 DS4432U I2C address, output-zero register, and upstream-derived
register code; prove that the safety owner routes the typed command through one
DS4432U write; prove the fixed 500 ms stabilization boundary before ASIC
enable; and preserve the pinned upstream zero-voltage behavior in which the
DS4432U is not written and the active-low ASIC-enable boundary removes VCORE.
It must also bind trusted package/runtime identity, fresh safety, successful
downstream initialization and accepted work, confirmed safe stop, cleanup, no
hardware rerun, and redaction.

The projector must independently validate its source projection, reject any
incomplete or mutated quorum member, reject ambiguous or drifted source
semantics, write through a private candidate, validate the candidate with the
Rust-owned contract, and publish atomically only after every check passes.
Public output may contain only schemas, commits, digests, fixed safe constants,
closed categories, counts, and booleans.

No detector, package build, flash, reset, USB session, serial monitor, network
request, credential access, mining rerun, GPIO or I2C effect, direct UART, pin
manipulation, fault injection, voltage/fan/power change, or other hardware
interaction is in scope. This plan does not claim measured analog voltage,
setpoint accuracy, rail timing or waveform, arbitrary targets, dynamic voltage
changes, over/under-voltage fault recovery, INA260 correlation, or behavior on
another board.

## Implementation

- [ ] Add the minimum typed core-voltage-control evidence contract,
      independent validator, generated binding, projector command, and human
      command surface.
- [ ] Add behavior-focused Rust and TypeScript regressions for the complete
      quorum, exact DS4432U constants and write route, stabilization and
      active-low disable semantics, source compatibility, publication
      withholding, candidate cleanup, workflow identity, and sensitive-output
      absence.
- [ ] Produce and independently validate one redacted public PWR-003 evidence
      projection from the accepted PWR-002 projection; do not rerun hardware.

## Verification and promotion

Focused tests must prove that only a validator-accepted PWR-002 source
projection with its exact digest, trusted package/runtime identity, issued
1100 mV command, successful downstream work, safe stop, cleanup, and no
hardware rerun can produce PWR-003 evidence. They must bind the exact current
DS4432U address/register/code and single-write route, fixed stabilization,
active-low zero-voltage disable semantics, source/reference/task/plan/workflow
identity, source-drift rejection, candidate cleanup, evidence withholding on
every failed quorum member, and absence of hostnames, origins, ports,
USB/network identifiers, credentials, raw traces, or private paths.

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
`just verify-reference`, unique task/plan binding, immutable-plan digest,
reference cleanliness, sensitive-output review, `git diff --check`, and full
diff review. Commit and push this immutable plan/task checkpoint before
implementation.

Promote `PWR-003` from `implemented` to `verified` only if the final closed
projection passes every validator and repository gate and establishes that the
exact admitted Ultra 205 firmware issued the source-bound DS4432U 1100 mV
command, completed stabilization and successful downstream work, and returned
through its active-low safe-stop boundary. Otherwise retain `implemented`,
withhold final evidence, record the exact blocker, and stop this row without a
hardware attempt.

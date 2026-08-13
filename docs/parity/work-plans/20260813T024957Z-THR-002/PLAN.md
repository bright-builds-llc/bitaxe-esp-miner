# Parity work plan

- Run ID: `20260813T024957Z-THR-002`
- Parity row: `THR-002`
- Initial status: `implemented`
- Source commit: `b7a74b5642f2bcbcb709c99ec69d62390d407e9c`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-thr002-fan-evidence-reconciliation`

## Selection

The clean synchronized selector reports no open plan and ranks `API-009`,
`THR-001`, then `THR-002`. `API-009` is sealed at its repeated-boundary stop
and prohibits another attempt without genuinely new software evidence.
`THR-001` consumed its read-only attempt and now requires a distinct bounded
overheat or fault hardware-regression contract; inventing that stimulus during
automatic continuation would be unsafe and outside its task.

`THR-002` is the first actionable row. Its checklist note predates the accepted
PWR-002 hardware-regression evidence and still says production fan writes are
suppressed. The committed, independently validated
`bitaxe-asic-power-initialization-evidence-v1` projection now proves a typed
100% EMC2101 fan command on the admitted Ultra 205 package, fan-before-voltage
ordering, and a fresh nonzero post-command fan RPM before the successful ASIC
initialization and accepted submit. A deeper source audit also found that the
pure controller decisions are not yet scheduled by the production firmware.
THR-002 is therefore actionable by wiring those decisions through the existing
typed queue into the already qualified sole I2C owner, then composing focused
workflow proof of that new orchestration with the sealed physical fan-response
evidence. This avoids both an unsafe evidence-only promotion and an unnecessary
repeat of the already proven EMC2101 hardware effect.

The active lesson set remains above its deterministic loading budget. Its
2026-08-03 audit baseline still matches the active inputs, so no new audit is
triggered. Safety, authorization, evidence, retry, redaction, source-boundary,
and real-process lessons materially inform this plan. The unrelated caption,
small-table deletion, GSD separator, and manual-removal blocks do not affect
this software-only reconciliation. Repo-local guidance, the Bright Builds
sidecar, architecture, code-shape, verification, testing, Rust, and TypeScript
standards were also reviewed; there is no active local override.

## Scope and non-scope

Add a production fan-controller runtime that schedules the existing pure
`FanControlDecision` at the upstream 100 ms cadence, reads only typed current
settings, mining state, and producer-owned thermal observations, preserves PID
state across iterations, suppresses unchanged duties, and submits only typed
fan commands through the existing bounded queue. The runtime must never own or
bypass raw I2C, and its retry behavior must remain bounded and observable.

Validate the existing PWR-002 projection through its Rust-owned contract;
confirm its exact committed digest, source/reference identity, immutable result
lineage, final mode, redaction, and unchanged production EMC2101 actuation
boundary; then add a row-specific `RESULT.md` explaining the composed workflow
and physical fan-response proof. Transition only THR-002 if every closed fact
and every mandatory gate still passes.

Production changes are limited to the high-level controller plan/runtime,
startup wiring, and focused ownership/orchestration tests. The raw EMC2101 and
I2C boundaries, reference source, evidence schema, projector, generated
contract, package, and hardware artifacts remain unchanged. No detector,
package build, flash, reset, USB or serial access, network request, credentials,
mining, voltage, fan, power, GPIO, I2C, direct UART, pins, fault injection, or
other hardware interaction is authorized or required. The completed PWR-002
campaign is not rerun or broadened.

This plan does not claim automatic dynamic fan behavior on hardware, a fan
speed curve, closed-loop thermal response, analog RPM accuracy, arbitrary duty
levels, fan-stall or fan-write fault stimulus, recovery from an injected fault,
extended soak, another board, or another thermal controller.

## Implementation

- [x] Audit the THR-002 row, pinned reference fan-controller surface, current
      controller and EMC2101 owners, and accepted PWR-002 projection/result.
- [ ] Add the production fan-controller planner and bounded 100 ms runtime,
      preserving single-owner I2C and typed actuation boundaries.
- [ ] Add focused tests for mode priority, settings validation, PID-state
      retention, unchanged-duty suppression, bounded retry, startup wiring,
      and the prohibition on raw actuator ownership.
- [ ] Independently validate the committed PWR-002 projection, unchanged
      physical actuation boundary, and all exact identities needed by THR-002.
- [ ] Add a row-specific `RESULT.md` containing only closed composed facts,
      conclusion, and explicit non-claims.
- [ ] Produce the checklist's required `unit,workflow,hardware-regression`
      evidence by referencing the existing immutable projection and result.
- [ ] Transition only THR-002, synchronize deterministic progress, complete
      the task review, and archive the task atomically.

## Verification and promotion

Focused verification must include the fan-controller core and production-shell
tests; startup and source-ownership assertions; the Rust ASIC power-
initialization evidence validator and contract tests; the TypeScript projector
regressions; exact projection, source-plan, and source-result digests; final
file mode; source/reference ancestry; unchanged production fan-write and
tachometer ownership; pinned-reference cleanliness; repository redaction; and
sensitive-output review.

Run the mandatory sequence in order: `cargo fmt --all`, `cargo clippy
--all-targets --all-features -- -D warnings`, `cargo build --all-targets
--all-features`, `cargo test --all-features`, `bun
scripts/bright-builds-check.ts all`, `just test`, `just parity`, and `just
parity-progress`. Also require immutable-plan digest, unique task binding,
`git diff --check`, and full diff review before each commit boundary required
by the parity workflow.

Promote THR-002 only if schema
`bitaxe-asic-power-initialization-evidence-v1` validates at committed SHA-256
`0668c274d09b3e39d7d5edfea4b2e66c97248ff77de9192981f3af00e547ddfe`
and binds board 205, exact source/reference identity, trusted package/runtime,
fresh safety before effects, a typed 100% fan command, fan-before-voltage
ordering, a fresh nonzero post-command RPM, successful initialized work, an
accepted submit, safe stop, cleanup, no hardware rerun, and passed redaction.
The production scheduler, mode selection, PID-state retention, unchanged-duty
suppression, bounded retry, typed queue, pure controller, duty-conversion,
tachometer, and fan-fault behavior must also remain covered by passing unit and
workflow tests. Any validation,
identity, source-compatibility, privacy, or repository-gate failure keeps
THR-002 at `implemented`, changes no checklist field, records a truthful
closure, and stops without hardware recovery or retry.

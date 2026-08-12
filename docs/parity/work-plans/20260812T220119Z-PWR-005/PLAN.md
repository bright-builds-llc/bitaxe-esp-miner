# Parity work plan

- Run ID: `20260812T220119Z-PWR-005`
- Parity row: `PWR-005`
- Initial status: `implemented`
- Source commit: `ec98c10322e8786e281047932b98b20bb5b38309`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-pwr005-ds4432u-evidence-reconciliation`

## Selection

The clean synchronized selector reports no open plan and ranks `API-009`
first, followed by `PWR-005`. `API-009` is temporarily unavailable because
its latest closure requires a fresh pre-effect occurrence in which the
operator explicitly reports being present, watching the display, and ready to
answer both live prompts. This automatic continuation has no such occurrence,
so it may not consume another hardware ordinal.

`PWR-005` is the first actionable row. Its checklist note predates the accepted
PWR-003 campaign and still says firmware does not write DS4432U hardware. The
committed, independently validated
`bitaxe-core-voltage-control-evidence-v1` projection now proves the exact
opposite for the admitted Ultra 205 package: one typed DS4432U output-zero
write at address `0x48`, register `0xf8`, code `0xe1`, followed by successful
BM1366 initialization, accepted work, and a confirmed safe stop. Reusing those
closed row-independent facts can reconcile PWR-005 without a duplicate schema,
projector, or hardware effect.

The active lesson set remains above its deterministic loading budget. Its
2026-08-03 audit baseline still matches the active inputs, so no new audit is
triggered. Safety, authorization, evidence, retry, redaction, source-boundary,
and real-process lessons materially inform this plan. The unrelated caption,
small-table deletion, GSD separator, and manual-removal blocks do not affect
this software-only reconciliation. Repo-local guidance, the Bright Builds
sidecar, architecture, code-shape, verification, testing, Rust, and TypeScript
standards were also reviewed; there is no active local override.

## Scope and non-scope

Validate the existing PWR-003 projection through its Rust-owned contract;
confirm its exact committed digest, source/reference identity, immutable result
lineage, final mode, redaction, and current production ownership; and add a
row-specific `RESULT.md` explaining why the overlapping DS4432U facts satisfy
PWR-005. Transition only PWR-005 if every existing closed fact still passes.

No production source, reference source, evidence schema, projector, generated
contract, package, or raw hardware artifact changes. No detector, package
build, flash, reset, USB or serial access, network request, credentials,
mining, voltage, fan, power, GPIO, I2C, direct UART, pins, fault injection, or
other hardware interaction is authorized or required. The completed PWR-003
campaign is not rerun or broadened.

This plan does not claim analog voltage measurement, voltage accuracy, rail
waveform or timing, DS4432U reads, output-one behavior, arbitrary or dynamic
setpoints, fault injection, INA260 correlation, non-conservative profiles,
another board, or another ASIC family.

## Implementation

- [x] Audit the PWR-005 row, pinned reference DS4432U surface, current typed
      write owner, and accepted PWR-003 projection/result lineage.
- [ ] Independently validate the committed PWR-003 projection and all exact
      identities needed by the PWR-005 claim.
- [ ] Add a row-specific `RESULT.md` containing only closed facts, conclusion,
      and explicit non-claims; add code only if validation exposes a real gap.
- [ ] Produce the checklist's required `unit,workflow,hardware-regression`
      evidence by referencing the existing immutable projection and result.
- [ ] Transition only PWR-005, synchronize deterministic progress, complete
      the task review, and archive the task atomically.

## Verification and promotion

Focused verification must include the Rust core-voltage evidence validator,
the focused Rust contract tests, the TypeScript projector regressions, exact
projection/result digests, final file mode, source/reference ancestry,
production DS4432U address/register/write ownership, pinned-reference
cleanliness, repository redaction, and sensitive-output review.

Run the mandatory sequence in order: `cargo fmt --all`, `cargo clippy
--all-targets --all-features -- -D warnings`, `cargo build --all-targets
--all-features`, `cargo test --all-features`, `bun
scripts/bright-builds-check.ts all`, `just test`, `just parity`, and `just
parity-progress`. Also require immutable-plan digest, unique task binding,
`git diff --check`, and full diff review before each commit boundary required
by the parity workflow.

Promote PWR-005 only if schema `bitaxe-core-voltage-control-evidence-v1`
validates at committed SHA-256
`11dd1abbf6fda86d203fdcff49b420ab5139e1d29c35f4d17000c61c3112ae68`
and binds board 205, exact source/reference identity, trusted package/runtime,
the source-compatible typed DS4432U route, address `0x48`, register `0xf8`,
code `0xe1`, exactly one write, successful initialized work, an accepted
submit, safe stop, cleanup, no hardware rerun, and passed redaction. Any
validation, identity, source-compatibility, privacy, or repository-gate failure
keeps PWR-005 at `implemented`, changes no checklist field, records a truthful
closure, and stops without hardware recovery or retry.

# Parity work plan

- Run ID: `20260804T192918Z-API-010`
- Parity row: `API-010`
- Initial status: `implemented`
- Source commit: `6d42e35271d973ba1425521c152b799d24575519`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-api010-baseline-epoch-admission`

## Selection

The deterministic selector resumed the open `API-010` plan at
`docs/parity/work-plans/20260804T185605Z-API-010/PLAN.md`; it reported no new
candidates. That plan and its completed implementation remain immutable. The
sole `attempt-002` capture stopped before any route or restart action because
the exact-package flash-monitor transcript classified as
`baseline_multiple_sessions`, so this follow-up owns only the newly isolated
host-orchestration defect.

Source inspection establishes the production shape without reading the private
hardware trace: `flash-monitor` performs the factory image write and then a
credential-seed write before opening its receive-only reader. Each write can
produce a boot epoch, and bytes from the intermediate boot can remain queued on
the device node. The final reader can therefore receive a well-ordered stale
prefix followed by the current ready epoch. Treating the whole transcript as a
single epoch correctly fails, but does not admit the independently complete
terminal epoch.

## Scope and non-scope

Add a distinct terminal-baseline classifier for production flash-monitor
transcripts. It must parse every boot-identity marker, require a chronological
chain of stable per-epoch identities with boot ordinals advancing exactly by
one, select the final epoch at its first identity marker, and require that
terminal slice to independently pass the existing strict baseline identity,
monotonic-uptime, bound-origin, and passive-safe-state rules. Interleaved or
reappearing sessions, ordinal gaps/regressions, inconsistent same-session
markers, malformed markers, mixed origins, stale-only tails, and incomplete
terminal epochs fail closed.

Use the classifier's admitted origin and identity directly in the settings,
theme, and operator-snapshot orchestration seams so they do not reparse the
mixed whole transcript. Preserve the existing whole-trace `baseline` mode and
fixture-oriented interfaces unchanged.

This is synthetic software work only. Do not read, print, summarize, copy, or
retain the private attempt trace. Do not flash, monitor, contact HTTP services,
mutate settings or theme state, restart or recover the device, use credentials,
mine, change hardware controls, perform OTA, use direct UART or pins, or perform
any physical electrical action. No new hardware attempt or parity promotion is
authorized by this plan.

## Implementation

- [ ] Add a typed terminal-baseline functional core beside the existing strict
      Phase 33 classifiers, with a dedicated CLI mode and redaction-safe closed
      failure categories.
- [ ] Route the settings, theme, and operator-snapshot production baseline
      seams through the terminal mode and consume its admitted origin rather
      than scanning the mixed transcript independently.
- [ ] Convert the production-shaped multi-epoch case into a regression and add
      unit cases for one ready epoch, ordered stale prefixes, interleaving,
      ordinal gaps/regressions, malformed markers, mixed origins, missing safe
      state, and incomplete terminal epochs.
- [ ] Add a real-child-process orchestration test whose classifier observes the
      actual production-shaped trace rather than manufacturing a passing JSON
      projection in-process.
- [ ] Record diagnosis, implementation, verification, and conservative closure
      checkpoints in `WORKLOG.md`; do not create `RESULT.md` because this task
      cannot produce hardware evidence or promote the row.

## Verification and promotion

Run focused Rust and TypeScript/Bazel targets first, then the mandatory ordered
sequence: `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D
warnings`, `cargo build --all-targets --all-features`, `cargo test
--all-features`, `bun scripts/bright-builds-check.ts all`, `just test`, `just
parity`, and `just parity-progress`. Also run semantic redaction, pinned-reference
cleanliness, immutable-plan, sensitive-output, and diff checks.

Acceptance requires the synthetic production-shaped stale-prefix transcript to
admit only its independently complete terminal epoch while every ambiguous or
incomplete shape fails closed. Public projections and errors must not expose
origins, hostnames, ports, USB identities, network identifiers, credentials, or
raw traces. Keep `API-010` at `implemented`; after a clean pushed software fix,
record that any live retry requires a separate active task with a fresh exact
command, evidence, privacy, recovery, retry, and stop contract.

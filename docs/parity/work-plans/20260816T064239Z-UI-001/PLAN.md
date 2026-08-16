# Parity work plan

- Run ID: `20260816T064239Z-UI-001`
- Parity row: `UI-001`
- Initial status: `implemented`
- Source commit: `ef01046b52c41418417d121ca4c8f439c174c54e`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-ui001-display-behavior`

## Selection

The clean synchronized `main` branch equals `origin/main`, the pinned reference
is clean at the recorded commit, and
`bazel run //tools/parity:report -- next-item --format json` reported no open
plan. Its candidate order begins `UI-001`, `UI-002`, `UI-003`, `SELF-001`,
`BAP-002`, `STAT-001`, `STAT-002`, `STAT-003`, `REL-003`, `SAFE-10`,
`SAFE-11`, `CFG-07`, `ASIC-09`, `ASIC-10`, `ASIC-11`, `ASIC-12`, `STR-08`,
`STR-09`, `SAFE-12`, `SAFE-13`, `ASIC-009`, `ASIC-010`, `BAP-001`.
No row is skipped because UI-001 is first and is actionable.

UI-001's implementation and unit/workflow evidence are complete. Its remaining
row-owned gap is a physical panel observation. The committed
`bitaxe-display-uat-evidence-v1` projection from API-009 attempt-005 already
records one detector-admitted board 205, exact build and USB admission, one
machine-confirmed IDENTIFY render and natural clear, and independent operator
confirmation that both physical pixel states were observed. Its SHA-256 is
`a863fc0034f105c85ae3007cd45a532035bfd6e061dbbf1a915282a5cfa3314f`.
It binds the committed API-009 attempt-046 programmatic projection whose
SHA-256 is
`216420e0a9d93cbbacced7415be0a234ed13c0d895dcb20eb1ff295ff434a8a3`.
The display core, settings projection, adapter, and startup paths are unchanged
from that package source; the only change to the broader runtime owner is an
unrelated thermal-fault replay request. A source-bound UI-001 projection can
therefore close the gap without another physical effect.

Active lessons total 29,963 bytes, above both bounded-load limits. Complete
safety, privacy, authorization, evidence, hardware-retry, exact-boundary,
diagnostic-completeness, and current-task blocks totaling 22,325 bytes were
loaded. The unread global blocks are
`lesson-use-source-vtt-for-caption-fixes`,
`lesson-zsh-lowercase-path-mutates-path`,
`lesson-macos-host-stalls-separate-policy-from-cache`, and
`lesson-claim-visible-chrome-tab-before-handoff`; the unread repository blocks
are `lesson-gsd-frontmatter-body-separators`,
`lesson-manual-removal-needs-owner-observation`,
`lesson-physical-usb-identity-excludes-enumeration-fields`,
`lesson-cold-boot-proof-needs-an-independent-observer`, and
`lesson-consume-qualified-transport-capabilities`. The 2026-08-03 audit
baseline already consumed the hard-limit crossing; only five new lessons have
accumulated, fewer than 90 days have elapsed, and no append is proposed, so no
new lesson-audit trigger exists.

## Scope and non-scope

Add a closed Rust evidence contract, an independently tested TypeScript
projector, a validator binary, Bazel/CLI wiring, and one public aggregate
projection for UI-001. The projector must admit only the two exact committed
public inputs above; verify their digests, schemas, board, package/reference
identity, single IDENTIFY request, machine and operator render/clear quorum,
safe stop, cleanup, and redaction; bind this immutable plan and the exact active
task block; compare the display implementation at captured source
`522d5abda3af659a45691c2d4a7c03712573fb80` with the pushed projector source;
and admit pinned reference display semantics from `main/display.c` and
`main/screen.c`.

Unchanged display-owned files must be byte-identical across the captured and
current source commits. The broader `operator_sensor_runtime.rs` path may use
semantic-fragment comparison at both commits only because its sole intervening
change is independently visible and unrelated to display behavior. Fragment
checks must be unique and tests must reject missing, duplicate, drifted,
undeclared, stale-plan, stale-task, bad-digest, incomplete-quorum, and wrong-
identity inputs. The output may contain only shareable facts and public
provenance and must state `hardware_rerun_used: false`.

This plan authorizes only committed public evidence, repository source and Git
history, deterministic tests, documentation, checklist tooling, and local
builds. It prohibits reading protected attempt artifacts or credentials and
prohibits detector, USB/serial, device/network/HTTP, display effects, human
checkpoints, mining, settings mutation, restart, OTA, recovery, hardware
control, external UART/BAP, and all pin/pad/header/GPIO/probe/jumper/solder/
signal work. It does not claim physical pixel geometry, brightness, every
rotation or inversion on hardware, timeout duration/current draw, physical
button behavior, UI-002 content parity, mining, other boards, soak, update,
recovery, or release readiness.

The local guidance in `AGENTS.md`, managed `AGENTS.bright-builds.md`, empty
effective overrides, and the architecture, code-shape, verification, testing,
Rust, and TypeScript standards materially require a typed functional core,
thin orchestration, complete boundary parsing, focused Arrange/Act/Assert
tests, exact source/runfiles ownership, and the ordered pre-commit gates.

## Implementation

- [ ] Add `bitaxe-display-behavior-evidence-v1` as a closed Rust contract with
      an independent validator and generated TypeScript type.
- [ ] Add a source-bound projector and focused tests for both accepted evidence
      and every identity, digest, quorum, source, task, plan, and publication
      rejection boundary.
- [ ] Wire the projector through the automation CLI, Bazel/runfiles graph, and
      human command surface; generate and independently validate only
      `docs/parity/evidence/ui001-display-behavior/display-behavior-projection.json`.
- [ ] Produce a commit-bound `RESULT.md`, transition only UI-001 when the exact
      quorum passes, synchronize deterministic progress, and archive only its
      completed active task record.

## Verification and promotion

Before the implementation commit, run focused Rust contract tests, focused
automation projector tests, the validator through Bazel, source/runfiles
ownership checks, and `just verify-redaction`. Run the mandatory ordered gates:
`cargo fmt --all`,
`cargo clippy --all-targets --all-features -- -D warnings`,
`cargo build --all-targets --all-features`, `cargo test --all-features`,
`bun scripts/bright-builds-check.ts all`, `just test`, `just parity`, and
`just parity-progress`. Also run `just verify-reference`, `just package`, plan
digest, source-commit, selector, output-schema, permissions, sensitive-value,
and `git diff --check` checks.

Commit and push the projector implementation before generating evidence. Bind
the projection to that full pushed commit, then commit and push the projection,
`RESULT.md`, and worklog as `SOURCE_COMMIT`. Promote UI-001 from `implemented`
to `verified` with `unit,workflow,hardware-smoke` only if the independent
validator accepts the exact public UAT, programmatic, source, reference, plan,
task, cleanup, and redaction quorum. After transition, immediately run
`sync-progress --source-commit "$SOURCE_COMMIT" --selected-row UI-001 --plan
docs/parity/work-plans/20260816T064239Z-UI-001/PLAN.md`, archive the completed
UI-001 task atomically, rerun every ordered gate, review the complete diff,
commit, fetch, and push without force. Any failed boundary withholds the
projection and verified claim, leaves UI-001 `implemented`, records a truthful
closure, and performs no hardware rerun.

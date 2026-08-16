# Parity work plan

- Run ID: `20260816T073911Z-UI-002`
- Parity row: `UI-002`
- Initial status: `implemented`
- Source commit: `7375f9f091c9c750b257f7ab3b0476b15843705f`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-ui002-screen-flow`

## Selection

The clean synchronized deterministic selector reported no open plan and ranked
UI-002 first, followed by UI-003 and the remaining unfinished rows. No earlier
candidate was skipped. UI-002 is actionable because its pure and firmware
screen-flow implementation is complete, the exact committed API-009 physical
display UAT proves one IDENTIFY overlay rendered and naturally cleared through
the production screen owner, and the captured firmware's core screen-flow,
frame, runtime projection, and display-adapter sources are byte-identical to
current source. The retained runtime owner changed later only to add evidence
receipts and must be admitted through exact unique semantics at both commits.

The pinned reference owns a 500 ms update timer, priority screens, two bounded
intro screens, a four-page carousel, identify and notification overlays,
new-block statistics pinning, and button-owned manual advancement. Current
Rust has deterministic tests for those decisions and a private four-line panel
frame. This plan may join that software proof to the already accepted physical
IDENTIFY observation; it may not broaden the UAT into physical proof of every
page, animation, bitmap, timing interval, input path, or pixel detail.

## Scope and non-scope

Add a closed `bitaxe-screen-flow-evidence-v1` aggregate projection and an
independent Rust validator. The projector must accept only the exact committed
API-009 display-UAT and command-effects projections, this immutable plan, the
one exact active task block, clean synchronized current source, captured source
`522d5abda3af659a45691c2d4a7c03712573fb80`, and pinned reference
`c1915b0a63bfabebdb95a515cedfee05146c1d50`. It must prove the public hardware
quorum, exact package/reference identity, safe stop, cleanup, disabled mining
and hardware control, source compatibility, and reference semantics before
publishing one atomic aggregate-only document at
`docs/parity/evidence/ui002-screen-flow/screen-flow-projection.json`.

Bind the six priority pages, two one-shot intro pages, four carousel pages,
500 ms evaluation cadence, 3,000 ms intro dwell, 10,000 ms carousel dwell,
IDENTIFY override, accepted/rejected/work/paused notifications, new-block
statistics pin, bounded private four-line frames, side-effect-free snapshot
projection, retained screen owner, change-only rendering, priority-to-power
visibility, and display-failure isolation. Reject missing, duplicate, drifted,
dirty, unrelated, or caller-authored source/reference semantics. Declare every
validator and source input in Bazel/runfiles and cover the actual invocation
and independent-validator process boundaries.

This is a software-only evidence continuation. It authorizes committed public
evidence, repository source and Git history, deterministic tests, documentation,
checklist tooling, local builds, and one clean-source projector/validator
transaction. Do not read credentials or protected attempt artifacts. Do not
access the detector, USB/serial, device/network/HTTP, physical display, browser,
operator checkpoint, mining, pool, settings mutation, restart, OTA, recovery,
hardware control, external UART/BAP, or any pin, pad, header, GPIO, probe,
jumper, solder, or injected-signal interface. No hardware attempt or human
checkpoint is authorized or required.

Physical verification of every priority/intro/carousel page, exact page dwell,
notification combination, new-block screen, private values, input behavior,
LVGL animation/bitmap/QR fidelity, pixel geometry, brightness, other boards,
mining, soak, updates, recovery, and release readiness remain non-claims.

## Implementation

- [ ] Add the closed Rust screen-flow evidence contract, independent validator,
      generated TypeScript binding, and focused acceptance/rejection tests.
- [ ] Add a functional-core TypeScript projector with exact public-evidence,
      task/plan, source-history, reference-semantic, cleanliness, and atomic
      publication boundaries.
- [ ] Add the typed CLI, invocation/failure handling, `just` surface, explicit
      Bazel targets/runfiles, and real-process boundary regressions.
- [ ] Run every focused and mandatory gate, commit and push the implementation,
      then generate and independently validate exactly one final projection
      from that clean synchronized source.

## Verification and promotion

Focused tests must accept the exact closed quorum and reject incomplete UAT,
wrong binding digests, altered task or plan, missing/duplicate/drifted current
or captured source semantics, reference drift, dirty source, candidate survival,
wrong file mode, validator failure, launch failure, and unsupported arguments.
The output must be schema-valid, mode `0644`, free of private values and raw
frames, state `hardware_rerun_used: false`, and contain no voltage, millivolt,
credential, origin, address, port, hostname, USB/process identity, or protected
path data.

Run, in order, `cargo fmt --all`, `cargo clippy --all-targets --all-features --
-D warnings`, `cargo build --all-targets --all-features`, and `cargo test
--all-features`; then run `bun scripts/bright-builds-check.ts all`, `just test`,
`just parity`, `just parity-progress`, `just verify-redaction`, `just
verify-reference`, `just package`, the focused projector and independent
validator targets, immutable-plan/task/digest/mode/candidate checks, the
deterministic selector, and `git diff --check`.

Commit and push the plan/task checkpoint before implementation. Commit and push
the complete implementation before projection; that clean pushed commit is the
projector's current source. Commit the accepted projection and result before
using that full commit as `SOURCE_COMMIT` for the single UI-002 transition and
immediate progress synchronization. Promote only UI-002 from `implemented` to
`verified` with `unit,workflow,hardware-smoke` evidence when the complete
independently validated quorum passes. Otherwise withhold the candidate, keep
UI-002 `implemented`, create a truthful closure, leave the task active with its
blocker and next safe action, and do not synchronize unchanged progress.

# Parity work plan

- Run ID: `20260816T102741Z-UI-003`
- Parity row: `UI-003`
- Initial status: `implemented`
- Source commit: `f713c086c86dae43ae4e4c5c57728a12f99e2417`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-ui003-boot-button`

## Selection

The worktree is clean, `main` equals `origin/main`, the pinned reference is
clean at the recorded commit, and the deterministic selector reported no open
plan. It ranked UI-003 first, followed by SELF-001, BAP-002, and the remaining
unfinished rows. No candidate is skipped.

UI-003 remains `implemented` because attempt-001 stopped before its physical
checkpoint and produced no public projection. That failure supplied verified
new information: arbitrary receive chunks were parsed as complete lines, so a
split runtime-attestation marker was classified as malformed. Pushed commit
`f713c086` now retains a bounded partial line and its focused unit and integrated
tests prove fragmented markers reach the checkpoint without a parse failure.
A fresh attempt ordinal is therefore eligible under the active task's standing
authorization and hardware-retry lesson.

The pinned `reference/esp-miner/main/input.c` uses active-low GPIO0 with an
internal pull-up, emits an LVGL short-click event, and uses a 2,000 ms long-press
threshold. Rust owns a 10 ms sampler, 30 ms debounce, exact 2,000 ms long-press
boundary, retained GPIO0 pull-up owner, and the production
`input_event=short_click effect=screen_advance` route. The only remaining
row-owned gap is one trustworthy post-checkpoint physical short click on the
exact current package.

The combined active lesson inputs exceed both startup budgets. All seven global
lessons and fifteen complete repository blocks covering ESP reset behavior,
USB ownership and identity, native observation, boot replay, direct-UART
authorization, protected evidence roots, earliest failure, hardware retries,
qualified transports, flash-versus-monitor proof, standing authorization,
physical checkpoints, live checkpoint wording, and preflight exits were read.
Thirteen lower-priority repository blocks were not read: GSD frontmatter,
generic ESP-IDF ownership/redaction, opaque handoff, cross-process tests,
heartbeat/silent transport, manual removal, cold-boot observer, ESP-IDF main
task capacity, HTTP liveness, redact-after-classification, evaluator identity,
telemetry operating ranges, and completed legacy wire units. The 2026-08-03
audit baseline exists; six new lessons are below the ten-lesson trigger, fewer
than 90 days have elapsed, and this plan appends no lesson, so no audit runs.

## Scope and non-scope

Rebind the existing typed, transcript-free input UAT from consumed
`attempt-001` to fresh `attempt-002` and this immutable plan. Preserve its exact
package admission, repository USB supervision, one factory flash, repeated
same-session runtime attestation, source/reference semantics admission, durable
operator checkpoint, one post-checkpoint short-click marker, cleanup, aggregate
projection, and independent validator contracts.

Before hardware, replace the coarse runtime-attestation failure with its closed
redaction-safe status label and regression-test fragmented, malformed, identity,
and readiness boundaries. Do not retain source serial text or publish device,
USB, port, network, process, protected-path, credential, or private values.

This plan authorizes local source and task edits, tests, builds, ordinary git
operations, one `just package`, one `just detect-ultra205`, one exact-package
factory flash and receive-only observation, and one human press-and-release of
the provided BOOT button lasting less than two seconds after the live checkpoint
is visibly active. The press may wake or advance the display and must not change
persistent configuration.

Long press, configuration-AP toggling, self-test, credentials, HTTP or other
network access, mining, pool traffic, voltage, frequency, fan, thermal, power,
ASIC control, OTA, erase, rollback, recovery writes, fault injection, external
UART/BAP, physical power manipulation, and contact with pins, pads, headers,
GPIO, probes, jumpers, solder, or electrical signals are prohibited.

The local `AGENTS.md` task and hardware guidance, managed
`AGENTS.bright-builds.md`, empty effective overrides, and the architecture,
code-shape, testing, verification, and Rust standards require a typed core,
thin effect shell, focused Arrange/Act/Assert tests, exact evidence boundaries,
redaction, clean pushed source before effects, and the ordered Rust gates.

## Implementation

- [ ] Rebind the UAT's exact plan and private root contracts to attempt-002
      without changing the fixed public projection path or input semantics.
- [ ] Preserve bounded incremental line framing and expose the first closed
      runtime-attestation status without source text or private values.
- [ ] Add focused pure and integrated regressions for the attempt-002 path,
      fragmented markers, closed failure detail, interruption, and projection
      withholding.
- [ ] Commit and push the complete implementation, build its exact package,
      run only attempt-002, and promote only if the complete projection passes.

## Authorized live command and recovery

After the implementation is clean, fully verified, committed, and pushed, run
these commands in order:

1. `just package`
2. `just detect-ultra205`
3. `just input-uat --board 205 --port <detector-port> --manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --private-root scratch/ui003-input/attempt-002 --plan docs/parity/work-plans/20260816T102741Z-UI-003/PLAN.md --projection docs/parity/evidence/ui003-input/input-uat-projection.json`

Record the detector command, its one-device board-info success, selected port,
source/reference commits, package manifest and artifacts, and exact UAT command
in private or redacted evidence. Never commit the raw port or USB identity. The
workflow must emit the live checkpoint only after the receive-only reader is
admitted and two monotonic exact-package runtime attestations plus source and
reference input semantics are trusted. Only then may the operator briefly press
and release BOOT once. There is no human-response deadline.

`Ctrl-C` or refusal releases USB ownership and writes no positive projection.
Cleanup must always release the repository lease; cleanup failure remains
secondary to the earliest operation failure. Because this safe input workflow
does not alter configuration, its normal recovery is USB/process cleanup and a
normal running device. No recovery flash or additional device effect is
authorized.

`attempt-002` is the sole effectful attempt. Do not retry unchanged. A later
ordinal requires verified new information and another immutable plan. Stop on
detector ambiguity/failure, non-205 identity, package/source/reference drift,
flash failure, runtime-attestation or source-semantics failure, missing
checkpoint, interruption/refusal, missing/duplicate/unexpected/long-press input
marker, cleanup failure, projection/validator/redaction failure, or successful
verified projection.

## Verification and promotion

Before the implementation commit, run focused input UAT tests plus the ordered
mandatory sequence: `cargo fmt --all`,
`cargo clippy --all-targets --all-features -- -D warnings`,
`cargo build --all-targets --all-features`, `cargo test --all-features`,
`bun scripts/bright-builds-check.ts all`, `just test`, `just parity`, and
`just parity-progress`. Also run `just verify-redaction`,
`just verify-reference`, `just package`, the independent input-evidence
validator build, immutable-plan and projection-absence checks, and
`git diff --check`.

Commit and push this plan/task checkpoint before implementation. Commit and
push the complete implementation before hardware; rebuild so the package binds
that exact clean pushed commit. Promotion requires a validator-accepted
`bitaxe-input-uat-evidence-v1` projection proving board 205, exact package and
plan identity, GPIO0 active-low pull-up, 10/30/2,000 ms timing, trusted repeated
runtime attestation, admitted source/reference semantics, exactly one physical
short click routed to screen advance after the checkpoint, no long press,
complete cleanup, disabled mining/control, no retained transcript, and passed
redaction.

On success, commit the projection, `RESULT.md`, worklog, and completion review
as `SOURCE_COMMIT`; transition only UI-003 to `verified` with
`unit,workflow,hardware-smoke`; immediately synchronize progress; archive only
its completed task record; rerun every ordered gate; review, fetch, and push
without force. Any failed or declined boundary withholds the projection and
verified claim, leaves UI-003 `implemented`, records a truthful closure, and
stops without an unchanged hardware retry.

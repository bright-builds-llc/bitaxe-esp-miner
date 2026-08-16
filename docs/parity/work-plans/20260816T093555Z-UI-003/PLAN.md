# Parity work plan

- Run ID: `20260816T093555Z-UI-003`
- Parity row: `UI-003`
- Initial status: `implemented`
- Source commit: `415f845a79443bd02c3e93e188b31c07f49fb37d`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-ui003-boot-button`

## Selection

The clean synchronized `main` branch equals `origin/main`, the pinned reference
is clean at the recorded commit, and the deterministic selector reported no
open plan. It ranked `UI-003` first, followed by `SELF-001`, `BAP-002`, and the
remaining unfinished rows. No row is skipped.

UI-003's pure active-low debounce classifier, exact 2,000 ms long-press
boundary, retained GPIO0 pull-up owner, and short/long routing are already
implemented with unit and workflow evidence. No committed evidence contains a
physical `input_event=short_click effect=screen_advance` observation. The
remaining row-owned gap is therefore one trustworthy physical input event on
an exact current package, not another software relabel.

The pinned `reference/esp-miner/main/input.c` owns the active-low BOOT input,
pull-up, LVGL short-click delivery, and 2,000 ms long-press threshold. Current
Rust owns the corresponding 10 ms sampler, 30 ms debounce, 2,000 ms threshold,
one-shot classifier, and short-click screen-advance marker. A single brief
physical BOOT press can verify the representative safe input path. Live long
press is deliberately excluded because it changes configuration-AP state;
self-test effects and exact LVGL timing remain nonclaims.

Active lessons exceed the deterministic full-load budget. All global lessons
and 17 complete repository blocks covering safety, authorization, privacy,
evidence, hardware retries, USB ownership, physical observation, operator
checkpoints, telemetry ranges, and current-task concerns were loaded. The
additional relevant cross-process, earliest-failure, and flash-versus-monitor
blocks were also loaded before this plan. Eight disclosed lower-priority blocks
remain unread: GSD frontmatter, generic boot replay, heartbeat/silent transport,
manual removal, ESP-IDF capacity, HTTP liveness, qualified transport
capabilities, and completed legacy wire units. The 2026-08-03 audit baseline
exists; six new lessons are below the ten-lesson trigger, less than 90 days
elapsed, and this plan does not append a lesson, so no new audit is triggered.

## Scope and non-scope

Add one integrated `input-uat` Rust workflow to the existing flash tool. It
must admit the exact clean package, acquire the repository USB supervisor,
flash that package once, consume serial data without retaining a raw transcript,
and establish exact build/reference identity, safe-state startup, and active
input ownership before publishing a durable operator checkpoint. Only a later
production marker for exactly one short click routed to screen advance may
complete the machine observation. Boundary-only and pre-checkpoint input lines
must be ignored. The first typed failure must survive cleanup.

The workflow owns a fresh ignored mode-`0700` private attempt root and mode-
`0600` private files. It may publish one aggregate-only, independently
validated `bitaxe-input-uat-evidence-v1` projection at
`docs/parity/evidence/ui003-input/input-uat-projection.json`. The projection may
contain only public source/reference/package digests, board and fixed input
contract values, bounded counts/categories, safe booleans, plan identity,
cleanup, and redaction status. It must not retain or publish serial text, USB
identity, port, network identity, credentials, process identity, protected
paths, or device-private values.

This plan authorizes local builds and tests, one `just package`, one
`just detect-ultra205`, one exact-package factory flash, receive-only serial
observation, and one human press-and-release of the provided BOOT button lasting
less than two seconds after the live checkpoint says it is ready. The press may
advance or wake the display but changes no persistent configuration. It does
not authorize a long press, configuration-AP toggle, self-test effect, mining,
pool or Wi-Fi credentials, HTTP/network access, voltage, frequency, fan,
thermal, power or ASIC control, OTA, erase, rollback, fault injection,
external UART/BAP, or manipulation of pins, pads, headers, probes, jumpers,
solder, or electrical signals.

The local guidance in `AGENTS.md`, managed `AGENTS.bright-builds.md`, empty
effective overrides, and the architecture, code-shape, verification, testing,
and Rust standards require a typed functional core, thin effect shell, exact
package admission, real USB/process boundary tests where feasible, focused
Arrange/Act/Assert coverage, redaction, and ordered pre-commit gates.

## Implementation

- [ ] Add a pure input-UAT observation reducer that requires exact startup,
      input-owner, post-checkpoint short-click, one-shot, interruption, and
      earliest-failure semantics without accumulating a serial transcript.
- [ ] Add the integrated flash/observe shell, fresh protected evidence root,
      durable self-describing checkpoint, cleanup handling, aggregate public
      projection, closed Rust evidence contract, independent validator, CLI,
      `just`, Bazel, and generated-type wiring.
- [ ] Add focused pure, fake-environment, CLI, contract, redaction, permission,
      source-ownership, and production invocation tests.
- [ ] Commit and push the complete implementation, then run at most one live
      attempt and promote only UI-003 if its exact projection validates.

## Authorized live command and recovery

After the implementation is clean, fully verified, committed, and pushed, run:

1. `just package`
2. `just detect-ultra205`
3. `just input-uat --board 205 --port <detector-port> --manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --private-root scratch/ui003-input/attempt-001 --plan docs/parity/work-plans/20260816T093555Z-UI-003/PLAN.md --projection docs/parity/evidence/ui003-input/input-uat-projection.json`

The workflow must display the live checkpoint only after the serial reader is
admitted and the exact startup/input contract has been observed. The operator
then briefly presses and releases BOOT once. There is no human-response
deadline. `Ctrl-C` or refusal releases USB ownership, records no positive
projection, and is an accepted stop. Cleanup must always release the repository
lease; cleanup failure is secondary to any earlier typed failure.

`attempt-001` is the only authorized effectful attempt. An unchanged failure
must not be retried. A new attempt requires a verified implementation or
environment change and a new immutable plan. Stop on zero or multiple likely
ports, board-info failure, non-205 identity, package/source/reference mismatch,
flash failure, missing exact startup/input ownership, interrupted or declined
checkpoint, missing/duplicate post-checkpoint marker, identity drift, cleanup
failure, projection/validator/redaction failure, or successful verified
projection.

## Verification and promotion

Before the implementation commit, run focused tests and the ordered mandatory
sequence: `cargo fmt --all`,
`cargo clippy --all-targets --all-features -- -D warnings`,
`cargo build --all-targets --all-features`, `cargo test --all-features`,
`bun scripts/bright-builds-check.ts all`, `just test`, `just parity`, and
`just parity-progress`. Also run `just verify-redaction`,
`just verify-reference`, `just package`, the independent input-evidence
validator, immutable-plan/task/source checks, selector, mode/candidate checks,
sensitive-value scans, and `git diff --check`.

Commit and push this plan/task checkpoint before implementation. Commit and
push the complete implementation before the live command; the exact clean
pushed commit and its package are the live source. If the independently
validated projection proves the complete quorum, commit and push the projection,
`RESULT.md`, and worklog as `SOURCE_COMMIT`, transition only UI-003 from
`implemented` to `verified` with `unit,workflow,hardware-smoke`, immediately
synchronize progress, archive only its completed task record, rerun all ordered
gates, review the complete diff, fetch, and push without force. Any failed or
declined boundary withholds the projection and verified claim, leaves UI-003
`implemented`, records a truthful closure, and stops without an unchanged
hardware retry.

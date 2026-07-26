# Tasks

This is the repository's sole active work tracker. Use one stable, timestamped
task block per unit of work. Update only that block as work progresses, record
the verification performed, and finish with a concise completion review.

Historical plans, milestones, debug sessions, and task records under
`.planning/milestones/` are evidence and context only. They do not authorize
new work.

## Active

### task-durable-ultra205-device-sessions | 2026-07-25 | Make USB flash cycles self-cleaning

- [x] Implement one typed macOS device-session supervisor for detector,
  bootloader, flash, receive-only monitor, re-enumeration, and cleanup stages.
- [x] Add host-wide same-device locking, private crash journals, isolated child
  process groups, bounded signal/timeout cleanup, and earliest-failure
  preservation.
- [x] Route `just detect-ultra205`, `just flash`, `just monitor`, and
  `just flash-monitor` through the supervisor without breaking their existing
  arguments.
- [x] Add `just verify-flash-durability` plus pure, fresh-process, CLI,
  runfiles, redaction, and entrypoint contract regressions.
- [x] Update active hardware/session guidance and run all required Rust,
  Bazel, repository, redaction, and reference checks.
- [ ] Run the task-gated 20-cycle connected Ultra 205 durability soak only
  after the software gates pass.

Hardware contract:

- Permitted commands:
  1. `just detect-ultra205`
  2. `just verify-flash-durability board=205 cycles=20 port=<detector-port> manifest=bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json wifi-credentials=wifi-credentials.json protected-root=scratch/flash-durability/attempt-001`
- Objective: prove twenty consecutive detector/flash/receive-only-monitor/
  cleanup/immediate-reflash permutations finish with the admitted physical
  Ultra 205 stably enumerated, accessible, and holder-free without unplugging
  USB or barrel power.
- Evidence: the supervisor exclusively creates
  `scratch/flash-durability/attempt-001` beneath a mode-0700 ignored parent.
  Detailed device, process, command, and serial material is
  `ProtectedOperational` in mode-0600 files. Console and completion review use
  only closed categories, booleans, counts, bounded durations, and safe source
  or package provenance. Nothing from this run is promoted or committed.
- Preconditions: exact current `HEAD` package and manifest pass admission;
  `just detect-ultra205` admits exactly one board 205; the ignored local
  `wifi-credentials.json` exists and is passed without reading or printing its
  contents; all software verification gates pass.
- Allowed effects: exact admitted factory-image writes, existing optional NVS
  seed writes, `usb-reset`/`hard-reset`, receive-only native USB observation,
  same-physical-device re-acquisition, and termination of only
  supervisor-proven repository-owned child process groups.
- Prohibited effects: erase-flash, arbitrary raw writes, watchdog-reset,
  voltage/fan/mining stress, network discovery, foreign-process termination,
  direct UART, pins, pads, headers, GPIO, probes, jumpers, soldering, injected
  signals, and evidence promotion.
- Recovery/restoration: after every stage, terminate and reap owned process
  groups, release serial descriptors, and require three stable same-device,
  accessible, holder-free samples. The final successful cycle leaves the exact
  admitted package and local Wi-Fi seed installed. A genuinely absent
  transport, identity drift, foreign holder, or unproved cleanup stops without
  physical intervention.
- Retry bound: one automatic retry is allowed only for a typed
  software-transport failure after cleanup proves the same physical device
  changed state while the operation and immutable package remain unchanged.
  The same authoritative signature recurring after that remediation selects
  `stop_repeated_boundary`; no other retry is allowed.
- Accepted terminal outcomes: `complete`, `stop_repeated_boundary`,
  `stop_hardware_blocker`, `stop_authority_boundary`, or
  `stop_impossible_contract`. The harness stops on the first non-ready cycle
  and preserves its earliest typed category, including
  `concurrent_repo_session`, `foreign_holder`, `transport_absent`,
  `identity_drift`, `bootloader_connect_failed`,
  `flash_failed_before_transfer`, `flash_failed_after_transfer`,
  `monitor_failed`, `cleanup_failed`, `recovery_not_observed`, or
  `repeated_boundary`.
- Timeouts: individual flash/monitor operations receive at least 360 seconds
  and their invoking process receives at least 420 seconds. Early hard errors
  may stop immediately; ordinary silence is not failure before the bound.

Verification: `cargo fmt --all`,
`cargo clippy --all-targets --all-features -- -D warnings`,
`cargo build --all-targets --all-features`, and
`cargo test --all-features` passed in order. Focused uncached Bazel tests for
the device-session, flash CLI, and entrypoint contract passed. All 76 Bazel
tests that can run against a modified source tree passed. After committing and
building the exact-`HEAD` firmware package, the remaining
`//scripts:phase36_substantive_evidence_test` passed. Redaction verification,
reference cleanliness, shell formatting, ShellCheck, and diff checks passed.

Completion review: Software implementation and clean-commit package
qualification complete; connected qualification is pending.

Hardware attempt 001 review: Stopped at the first reported boundary as
required. All five detect -> flash cycles completed, producing ten
`usb_session: ready` boundaries without unplugging USB or power. Cycle 6's
receive-only monitor completed supervisor cleanup, left zero repository-owned
flash processes and zero serial holders, and ended its protected mode-0600 log
with one `usb_session: ready` marker. The harness nevertheless reported
`cleanup_failed` because serial bytes did not end in a newline and the marker
was therefore not a standalone anchored line. The protected root remained
mode 0700. This is a software acceptance-harness false negative, not evidence
of a device cleanup failure; the 20-cycle qualification remains incomplete.
Do not rerun the unchanged boundary. Fix output framing, create a new exact
clean-`HEAD` package, and add a separately gated attempt before resuming.

Attempt 001 identity: board `205`; selected port `/dev/cu.usbmodem1101`;
source commit `c68ea40bfa933d2eb028c4bc618a969f49484def`; reference commit
`c1915b0a63bfabebdb95a515cedfee05146c1d50`; package manifest
`bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json`; protected logs
`scratch/flash-durability/attempt-001`. Exact commands were
`just detect-ultra205` followed by the permitted
`just verify-flash-durability board=205 cycles=20
port=/dev/cu.usbmodem1101
manifest=bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json
wifi-credentials=wifi-credentials.json
protected-root=scratch/flash-durability/attempt-001`. The detector and all
completed boundaries returned the closed `ready` category; detailed
`board-info`, flash, and serial output remains protected and uncommitted.

Hardware attempt 002 contract:

- Permitted commands:
  1. `just detect-ultra205`
  2. `just verify-flash-durability board=205 cycles=20 port=/dev/cu.usbmodem1101 manifest=bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json wifi-credentials=wifi-credentials.json protected-root=scratch/flash-durability/attempt-002`
- Preconditions: the receive-only output-framing regression and terminal-log
  contract tests pass; all software gates above pass; the fix and this contract
  are committed; the exact clean-`HEAD` package is rebuilt and admitted;
  detection returns exactly `/dev/cu.usbmodem1101`; the ignored credential
  input is present without being read or printed; and the protected root does
  not exist.
- Objective and effects: restart all four five-cycle sequences from cycle 1
  using one immutable package and the same physical board 205. The objective,
  allowed/prohibited effects, 360/420-second timeouts, one state-changing retry
  bound, cleanup/restoration procedure, closed failure vocabulary, and terminal
  stop categories are exactly those in the Hardware contract above.
- Evidence: only Attempt 002 may write
  `scratch/flash-durability/attempt-002`. Its root must be mode 0700 and regular
  files mode 0600. Attempt 001 remains immutable, ignored, and unpromoted.
  Success requires 20 completed cycles and 40 operation logs whose final
  logical line is exactly `usb_session: ready`, plus zero repository-owned
  processes/descriptors and zero serial holders without unplugging USB or
  power.
- Recovery: stop on the first failed boundary and preserve its earliest typed
  category. Do not rerun an unchanged failed boundary. Leave the exact package
  installed after success; after failure, perform only the existing bounded
  supervisor cleanup and read-only holder/accessibility audit.

### task-ci-reference-submodule-checkout | 2026-07-25 18:23 | Repair evidence-redaction CI checkout

- [x] Reproduce the failed GitHub Actions boundary and identify the earliest
  typed failure.
- [x] Configure the evidence-redaction workflow to check out the pinned
  reference submodule recursively.
- [x] Add a contract regression for the workflow and submodule declaration.
- [x] Run required Rust, focused Bazel, redaction, and full repository checks.
- [x] Commit, push, and verify the replacement GitHub Actions run.

Verification: Rust format, clippy, build, and tests passed. The workflow
contract and redaction tests passed without Bazel cache, all 74 tests that do
not require a clean source tree passed, and parity, reference-cleanliness,
redaction, shell formatting, ShellCheck, and diff checks passed. GitHub Actions
run `30179451307` passed on the real Ubuntu runner after recursively checking
out the pinned reference submodule.

Completion review: Complete. The failure occurred before the redaction
validator because Bazel workspace status could not read reference Git metadata
in the CI checkout. Recursive submodule checkout restored that prerequisite,
the contract regression prevents silent removal, and the unchanged redaction
validator passed in CI.

## Backlog

### task-private-first-remaining-evidence-pipelines | 2026-07-20 | Migrate remaining evidence workflows

- [ ] Inventory active evidence-producing workflows outside the archived
  Phase 35 workflow against `docs/parity/evidence-policy.md`.
- [ ] Migrate any active workflow that still performs in-place or post-write
  sanitization to private-first capture with a distinct shareable projection.
- [ ] Add focused regressions and route every admitted projection through
  `just verify-redaction`.
- [ ] Run the repository verification required for every changed language and
  workflow surface.

Verification: Pending.

Completion review: Pending. This task does not authorize hardware, credentials,
network access, device mutation, evidence promotion, or push operations.

### task-cross-platform-device-session-adapters | 2026-07-22 | Qualify Linux and Windows ESP device sessions

- [ ] Implement Linux physical/enumeration identity, exclusive ownership,
  receive-only observation, and bounded reacquisition behind the canonical
  device-session contract.
- [ ] Implement the corresponding Windows adapter without weakening
  exclusive ownership, request-once, or private-artifact guarantees.
- [ ] Add platform-native real-process tests.
- [ ] Keep unsupported platforms fail-closed until each exact adapter and its
  separately authorized hardware evidence qualify.

Verification: Pending.

Completion review: Pending. macOS remains the only production adapter. This
task does not itself authorize hardware, credentials, network discovery, direct
UART or pin work, evidence promotion, or push operations.

## Effectful Hardware Task Gate

Standing permission for safe USB interaction remains subject to `AGENTS.md` and
`docs/hardware/hardware-attempt-policy.md`. Before any effectful hardware run,
move or add one task block under `Active` that explicitly records:

- the exact permitted repo-owned command and objective;
- the evidence destination, privacy class, and redaction policy;
- recovery, restoration, and cleanup procedures;
- retry bounds, including the unchanged-boundary stop rule; and
- accepted terminal categories and stop conditions.

If any field is missing, hardware work is not authorized. A task entry never
expands the direct-UART, pin-manipulation, privacy, safety, or archived-lineage
boundaries in `AGENTS.md`.

## Accepted Debt and Constraints

- Milestone v1.2 is administratively closed with gaps and is not a release.
- Phase 36 stopped after 8 of 10 plans. Plans 36-07 and 36-04 did not complete.
- SYS-02, EVD-11, EVD-12, and EVD-14 remain blocked. EVD-15 is satisfied by
  exact preservation, typed demotion, and explicit non-claims.
- The sole final Phase 36 hardware attempt sealed `sealed_non_promotion`,
  produced no candidate, and left device restoration unresolved.
- Do not repeat the unchanged hardware attempt. A future attempt requires new
  diagnostic information, a targeted regression-backed fix or objectively
  verified non-invasive remediation, and a complete task-scoped hardware
  contract under the gate above.
- Administrative closure, software verification, or task completion alone is
  never hardware or parity evidence.

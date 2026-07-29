# Tasks

This is the repository's sole active work tracker. Use one stable, timestamped
task block per unit of work. Update only that block as work progresses, record
the verification performed, and finish with a concise completion review.

Task blocks under `## Future — Explicit Only` remain incomplete but are
excluded from automatic task selection, including repeated top-task loops.
Only a current request that names the exact stable task ID makes one future
task a selection candidate; that opt-in does not bypass dependencies,
environment, authorization, verification, or safety gates. When only future
tasks remain, the automatic queue is exhausted, but the tracker is not fully
complete.

Historical plans, milestones, debug sessions, and task records under
`.planning/milestones/` are evidence and context only. They do not authorize
new work.

## Active

### task-add-future-explicit-only-disposition | 2026-07-28 19:32 | Add explicit-only future tasks

- [x] Define the `## Future — Explicit Only` scheduling contract in the active
      tracker and repo-local task guidance.
- [x] Move `task-cross-platform-device-session-adapters` into that section
      without changing its task body or completion state.
- [x] Update the local `work-top-task` skill and UI metadata so automatic
      selection skips future work unless the current request names its exact
      stable task ID.
- [x] Validate automatic-queue outcomes, the skill package, and all required
      repository verification gates.

Verification:

- Static selection-contract checks covered active-plus-future selection,
  future-only automatic-queue exhaustion, exact-ID opt-in, rejection of
  title-only or broad opt-in, and preservation of authorization, prerequisite,
  verification, and safety gates.
- The local skill passed `quick_validate.py` through `uv run --with pyyaml`;
  its `agents/openai.yaml` parsed successfully with the required display name,
  description length, and `$work-top-task` default prompt.
- A byte-for-byte comparison against `HEAD` confirmed the
  `task-cross-platform-device-session-adapters` body was unchanged, and tracker
  inspection found it only beneath `## Future — Explicit Only`.
- `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`,
  `cargo build --all-targets --all-features`, and
  `cargo test --all-features` passed in order.
- `bun scripts/bright-builds-check.ts all` reported zero findings;
  `just test` passed all 76 Bazel tests; `just parity` reported
  `validation_errors: none`; `just verify-reference` reported clean reference
  commit `c1915b0a63bfabebdb95a515cedfee05146c1d50`; and
  `just verify-redaction` passed.
- Bare `mdformat --check` retained the same nonzero result as `HEAD` for both
  repository Markdown files, with no new Markdown-baseline regression;
  `git diff --check` passed.

Completion review: The future disposition is now canonical repository guidance
and the local `work-top-task` selection contract. The future cross-platform
adapter task remains incomplete and unchanged; when it is the only remaining
work, automatic selection reports queue exhaustion without treating it as a
blocker or mutating the repository. No hardware, credentials, network
discovery, evidence generation or promotion, direct UART or pin work, or
cross-platform adapter implementation was performed. Residual risk: the local
skill is intentionally outside repository Git and must be distributed
separately from this repository change.

### task-normalize-optional-function-names | 2026-07-27 20:06 | Normalize audit item 3 optional function names

- [x] Inventory every active repository-owned Rust function whose successful
      result may be absent, including multiline and `Result<Option<_>>`
      signatures.
- [x] Rename each candidate with a leading `maybe_` and update all in-repo
      callers without compatibility aliases.
- [x] Preserve externally required trait names, definite aggregate returns,
      wire formats, evidence schemas, and historical artifacts.
- [x] Verify the final exception inventory, expected Phase 36 evaluator
      identity rotation, and complete software test surface.

Verification policy:

- This task is a source-level naming refactor only. Do not change runtime
  behavior, schemas, CLI/protocol vocabulary, parity statuses, evidence,
  reference contents, or evaluator inventory membership.
- The pre-change Phase 36 evaluator identity is
  `fb45f3578257cb37a4a73572a7ea0643a93e11dae8762966bb71a6b036cb296c`.
  Source-bound identity rotation is expected when inventoried Rust sources are
  renamed; do not hard-code or promote the replacement identity.
- Do not detect, flash, monitor, read credentials, discover network targets,
  generate evidence, or otherwise interact with hardware.

Verification:

- The refreshed inventory accounts for 112 renamed function definitions under
  103 distinct prior spellings. The approved 105-candidate estimate was
  conservative: semantic expansion covered repeated cfg/test definitions and
  additional multiline `Result<Option<_>>` signatures, while the final scan
  also found and renamed `json_string_value_bounds`.
- The only direct absence-returning definition without `maybe_` is the
  externally imposed `std::error::Error::source`. Definite aggregate helpers
  `runtime_projection_for_api_views`, `validate_load_address`,
  `release_evidence_validation_paths`, and `execute_operation` retained their
  names; `project_observation` likewise returns a definite observation despite
  accepting an optional projection callback.
- Focused tests passed for every changed host package, including all 381
  `bitaxe-parity` tests and the active Phase 34 source guards.
  `cargo check --all-targets --all-features` also passed. Directly selecting
  `bitaxe-firmware` for a macOS-host test is unsupported by `esp-idf-sys`; the
  canonical `just test` path instead compiled the ESP32-S3 firmware target
  successfully.
- The required ordered Rust sequence passed: `cargo fmt --all`,
  `cargo clippy --all-targets --all-features -- -D warnings`,
  `cargo build --all-targets --all-features`, and
  `cargo test --all-features`.
- `bun scripts/bright-builds-check.ts all` passed with 556 files scanned,
  eight existing exceptions, and zero findings; `just test` passed all 72
  tests.
- `just parity` reported `validation_errors: none`; `just verify-reference`
  confirmed pinned commit `c1915b0a63bfabebdb95a515cedfee05146c1d50` is
  clean; `just verify-redaction` and `git diff --check` passed.
- The Phase 36 evaluator identity rotated from
  `fb45f3578257cb37a4a73572a7ea0643a93e11dae8762966bb71a6b036cb296c`
  to
  `7fd0ad4fba61f712e19ed3652c14be40395e5d3be8059bafd289e5f911197c21`;
  its inventory-binding test passed and no membership was changed or digest
  hard-coded.
- Final scope review found no changes to the checklist, committed evidence,
  planning archives, or reference tree. Rust schema and behavior tests passed
  unchanged, and no hardware, credentials, network discovery, evidence
  generation, or parity promotion occurred.

Completion review:

- Completed the direct source-level rename across the active Rust workspace
  and updated all typed callers plus active source-string guards without
  compatibility aliases.
- Residual risk is limited to supported external Rust callers outside this
  repository, for which no publishing or compatibility contract was found.
  This wave intentionally did not expand into an unrelated repository-wide
  rename of untouched optional locals, parameters, or fields.

### task-split-phase34-package-admission-guard | 2026-07-27 19:21 | Split audit item 2 source guard

- [x] Split the Phase 34 package and hardware admission source guard into
      focused one-concern unit tests.
- [x] Preserve every required marker, prohibited marker, and ordering
      relationship from the original source guard.
- [x] Keep explicit Arrange, Act, and Assert sections with small shared
      assertion helpers.
- [x] Run focused parity tests and the complete required software verification
      sequence.

Verification policy:

- This task is a test-structure-only Bright Builds audit remediation. Do not
  change production APIs, behavior, evidence schemas, checklist rows, parity
  status, historical evidence, reference contents, or evaluator inventories.
- Do not detect, flash, monitor, read credentials, discover network targets,
  generate evidence, or otherwise interact with hardware.

Verification:

- `cargo test -p bitaxe-parity phase34_package --all-features` passed all 10
  focused guards; an assertion-literal inventory comparison retained every
  policy marker and ordering relationship from the original test.
- `bazel test //tools/parity:tests` passed.
- The required ordered Rust sequence passed: `cargo fmt --all`,
  `cargo clippy --all-targets --all-features -- -D warnings`,
  `cargo build --all-targets --all-features`, and
  `cargo test --all-features`.
- `bun scripts/bright-builds-check.ts all` passed with 556 files scanned,
  eight existing exceptions, and zero findings; `just test` passed all 72
  tests.
- `just parity` reported `validation_errors: none`; `just verify-reference`
  confirmed pinned commit `c1915b0a63bfabebdb95a515cedfee05146c1d50` is
  clean; `just verify-redaction` and `git diff --check` passed.
- Final scope review found changes only in this task block and the Phase 34
  source guard. The checklist, evidence, planning archives, reference tree,
  and Phase 36 evaluator inventory are unchanged.

Completion review:

- Replaced the 245-line multi-concern test with 10 focused Arrange/Act/Assert
  tests and four small marker helpers without weakening source-guard policy or
  changing production behavior.
- Residual risk: these remain software-only source guards. No hardware or
  evidence verification was performed or authorized.

### task-type-http-exchange-observation | 2026-07-27 19:03 | Make HTTP exchange states unrepresentable

- [x] Replace the mutable flat HTTP exchange observation with closed typed
      transport, request, and response states.
- [x] Migrate the Phase 35 and device-session consumers without changing their
      evidence schemas, field meanings, or failure categories.
- [x] Add focused state-invariant and consumer-projection regression tests.
- [x] Update Bazel source ownership and run the complete required software
      verification sequence.

Verification policy:

- This task is software-only. Do not detect, flash, monitor, read credentials,
  discover network targets, create evidence, or promote parity claims.
- Preserve historical evidence, the reference tree, Phase 35/36 schemas,
  statuses, non-claims, and evaluator identities byte-for-byte.

Verification:

- The ordered `cargo fmt --all`,
  `cargo clippy --all-targets --all-features -- -D warnings`,
  `cargo build --all-targets --all-features`, and
  `cargo test --all-features` sequence passed.
- Focused HTTP transport, device-session, and parity tests passed with 7, 50,
  and 372 tests respectively, including exact flat projection, malformed
  response, partial response, request completion, and TLS failure coverage.
- `bun scripts/bright-builds-check.ts all` scanned 555 files with zero findings
  and the existing eight justified exceptions. `just test` passed all 72 Bazel
  tests.
- `just parity` reported `validation_errors: none`; `just verify-reference`
  reported the pinned reference clean at
  `c1915b0a63bfabebdb95a515cedfee05146c1d50`; `just verify-redaction` passed;
  and `git diff --check` passed.
- The final path audit confirmed no changes under historical evidence,
  `.planning/`, the parity checklist, or the reference tree, and no Phase 36
  evaluator source-inventory membership change.

Completion review: HTTP exchange construction now terminates in one closed
typed state with nonzero successful-stage facts and private parsed-response
construction. Phase 35 and device-session retain their exact existing flat
schemas through boundary-only projections.

Residual risks: This was software-only verification with loopback HTTP peers.
No hardware, credentials, live device target, evidence promotion, schema
revision, or evaluator identity change was used or claimed.

### task-eliminate-oversized-file-debt | 2026-07-27 14:08 | Eliminate oversized-file debt

- [x] Add exactly eight file-length exceptions for one generated Cargo license
      report and seven terminal-archive-protected files.
- [x] Refactor all oversized active core crates and firmware adapter files while
      preserving public Rust interfaces and firmware behavior.
- [x] Refactor all oversized parity modules while preserving evaluator,
      promotion, evidence-integrity, and caller-visible contracts.
- [x] Refactor all oversized host tools and Phase 17, 19, and 35 shell
      automation while preserving CLI arguments, schemas, redaction, runfiles,
      and failure categories.
- [x] Update Bazel source sets, source guards, and exact evaluator inventories
      for every extracted production child.
- [x] Run focused suites and the full required verification sequence, confirm
      zero findings with exactly eight exceptions, and review the final diff.

Verification policy:

- This task is software-only. Do not detect, flash, monitor, read credentials,
  discover network targets, create evidence, or promote parity claims.
- Preserve public Rust interfaces, CLI arguments, output schemas, evidence
  paths, redaction policy, hardware safety gates, and source-tree/Bazel-runfiles
  behavior.
- Preserve historical evidence byte-for-byte. Source-derived Phase 36
  evaluator and successor-contract digest rotation from explicit new source
  membership is expected, but no historical artifact may be regenerated or
  rewritten.
- Add no exception for active repository-owned code and make no
  `standards-overrides.md` change.

Verification:

- The ordered `cargo fmt --all`,
  `cargo clippy --all-targets --all-features -- -D warnings`,
  `cargo build --all-targets --all-features`, and
  `cargo test --all-features` sequence passed.
- Both extracted Python helpers passed `python3 -m py_compile`.
- `bun scripts/bright-builds-check.ts all` scanned 542 tracked files and
  reported zero findings with exactly eight justified exceptions.
- `just test` passed all 72 Bazel tests, including the ESP32-S3 firmware build
  and package graph. `just parity` reported `validation_errors: none`.
- `just verify-reference` reported the pinned reference clean at
  `c1915b0a63bfabebdb95a515cedfee05146c1d50`, and
  `just verify-redaction` passed.
- The current Phase 36 evaluator identity is
  `bef761f5f6580b462c131a9f381b61a9933a6775b0987b6af6788e0d05f9b294`;
  exact-membership and drift tests passed after adding the extracted owned
  evaluator sources. Historical evidence files remained byte-unchanged.
- `git diff --check`, staged source membership, Bazel data/runfiles, and the
  final source inventory were reviewed.

Completion review: All 42 active repository-owned oversized files were split
behind stable Rust facades or stable shell entrypoints. No active owned source
is excepted, all tracked files satisfy the 628-line hard maximum, and the eight
exceptions are limited to one generated license report and seven locally
immutable terminal-archive files.

Residual risks: The change is intentionally structural and was verified through
host, cross-compiled firmware, runfiles, parity, reference, and redaction gates,
but no hardware, credentials, live network target, or runtime evidence was
used. Existing hardware and parity claims remain unchanged.

### task-audit-integrity-state-provenance | 2026-07-26 21:24 | Repair audit items 1-3

- [x] Publish a hash-bound documentation-only successor to the authenticated
      Phase 36 checklist without changing historical evidence, statuses, or
      evidence classifications.
- [x] Validate every Rust-owned checklist target against the current workspace
      and refresh the 17 stale implementation-pointer rows.
- [x] Replace contradictory USB-process and monitor-capture state bags with
      closed typed states while preserving the existing flat evidence JSON.
- [x] Expose semantic version, short source commit, build timestamp, and the
      public source link in the fallback and recovery interfaces.
- [x] Add focused checklist, state-transition, API, and UI regressions.
- [x] Run the required Rust checks in order, focused Bazel tests, full
      repository tests, packaging, parity, reference, redaction, source scans,
      and final diff review.

Verification policy:

- Preserve
  `docs/parity/evidence/phase-36-substantive-evidence-admission-and-exact-re-promotion/`
  byte-for-byte.
- Do not change checklist status or evidence columns and do not promote any
  hardware, mining, OTA, recovery, or release claim.
- Do not detect, flash, monitor, access credentials, discover a network target,
  or otherwise interact with hardware.

Verification:

- The documentation successor
  `2026-07-26-source-pointer-refresh` binds the unchanged Phase 36 checklist
  digest `dd38e01ad40b07833fcffa9eb8c5f251d93feaf6f6a12c766881096c3245b497`
  and publishes checklist digest
  `1f8eb1423c404c9084e113c7a5689e2884d3c9377d84a4aa9c48e831d57af95b`.
- The ordered `cargo fmt --all`, Clippy, all-target build, and all-feature test
  sequence passed.
- Focused parity, flash, device-session, API, static-provenance, and packaging
  Bazel tests passed; the final `just test` passed all 72 repository tests.
- `just verify-production-session`, `just package`, `just parity`,
  `just verify-reference`, `just verify-redaction`, missing-target scans, and
  `git diff --check` passed.
- All five authenticated Phase 36 artifact digests remained byte-identical.

Completion review: Audit items 1-3 are repaired without parity promotion or
evidence-schema changes. The active checklist is now a validated, hash-bound
documentation successor; process and capture decisions are typed; and both
fallback interfaces expose safe firmware provenance.

Residual risks: No hardware, credentials, live network, OTA, recovery upload,
or mining behavior was exercised. Existing hardware and parity non-claims
remain unchanged.

### task-lcd-build-time-uptime | 2026-07-26 01:18 | Alternate LCD build time and uptime

- [x] Embed the canonical firmware build UTC timestamp from Bazel volatile
  workspace status.
- [x] Replace the static third LCD row with build time and uptime alternating
  every five seconds while preserving the other rows and display geometry.
- [x] Keep runtime LCD refresh and read-only sensor acquisition under one
  bounded I2C owner, with display failures isolated from sensor operation.
- [x] Add focused build-provenance, display-model, geometry, scheduling, and
  cache-contract regressions.
- [x] Update runtime display markers and the parity checklist without promoting
  full display/input parity.
- [x] Run the required Rust, Bazel, package, reference, and diff verification.
- [x] Attempt one detector-gated connected Ultra 205 display smoke and stop
  without flashing when detector admission fails.

Hardware contract:

- Permitted commands:
  1. `just detect-ultra205`
  2. `just flash-monitor board=205 port=<detector-port> manifest=bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json evidence-dir=scratch/lcd-build-time-uptime/attempt-001 capture-timeout-seconds=360 redact-evidence=true`
- Objective: verify the exact packaged firmware keeps the existing four-line
  128x32 layout while row three starts with the logged UTC build time, changes
  to increasing uptime after five seconds, and continues alternating.
- Evidence: `scratch/lcd-build-time-uptime/attempt-001` is ignored,
  non-promoted local evidence. The supervisor owns its evidence child under a
  mode-0700 parent; detailed serial, USB, process, and command material remains
  mode-0600 `ProtectedOperational`. Console and completion review use only
  closed categories, bounded counts/durations, and safe build provenance.
- Preconditions: all software gates pass; `just package` produces an exact
  current-HEAD manifest; detector admission finds exactly one board 205; no
  credential file is read or supplied.
- Allowed effects: write and verify the exact admitted factory image, perform
  the existing repo-owned reset/re-enumeration sequence, receive-only runtime
  observation, same-device re-acquisition, and cleanup of supervisor-proven
  repository child process groups.
- Prohibited effects: erase-flash, arbitrary raw writes, NVS seeding,
  credentials, network discovery, watchdog reset, voltage/fan/mining stress,
  foreign-process termination, direct UART, pins, pads, headers, GPIO, probes,
  jumpers, soldering, injected signals, and evidence promotion.
- Recovery/restoration: the device-session supervisor must terminate and reap
  owned process groups, release serial descriptors, and prove the admitted
  physical device accessible and holder-free. Success leaves the exact
  admitted package installed; identity drift, absence, a foreign holder, or
  unproved cleanup stops without physical intervention.
- Retry bound: no unchanged retry. A new ordinal is allowed only after a
  targeted regression-backed fix or an authorized non-invasive remediation
  objectively changes the failed boundary; one recurrence of the same
  authoritative signature selects `stop_repeated_boundary`.
- Accepted terminal outcomes: `complete`, `stop_repeated_boundary`,
  `stop_hardware_blocker`, `stop_authority_boundary`, or
  `stop_impossible_contract`. Existing closed flash/session categories provide
  the authoritative boundary signature, with the earliest failure preserved.
- Timeouts: capture is at least 360 seconds and the invoking wall clock exceeds
  420 seconds. Ordinary silence is not failure before the full bound.

Verification: `cargo fmt --all`,
`cargo clippy --all-targets --all-features -- -D warnings`,
`cargo build --all-targets --all-features`, and
`cargo test --all-features` passed in order. Focused Rust and Bazel tests,
the ESP32-S3 firmware cross-build, `just package`, `just verify-reference`,
ShellCheck, shell-format checks, and diff checks passed. After commit
`4701f51d7872`, the exact clean-`HEAD` package was materialized and all 78
`just test` targets passed, including the clean-source Phase 36 preflight.
The active parity checklist remains byte-identical to its authenticated
Phase 36 mirror; changing its notes directly would rewrite historical
evidence, so a later formal evidence generation must carry that documentation
update.

Completion review: The firmware implementation and software verification are
complete. Visual display verification remains pending because detector
admission failed before any flash. Full upstream display carousel/input parity
remains out of scope and below verified.

Hardware attempt 001 review: `just detect-ultra205` found one candidate at
`/dev/cu.usbmodem1101` and completed supervised cleanup with
`usb_session: ready`, but the required ESP32-S3 board-info probe returned
`bootloader_connect_failed`. The attempt selected the accepted
`stop_hardware_blocker` outcome with no unchanged retry and performed no flash.
Source commit `4701f51d78722f5399bb1ed2b1a24d18a0e8c798`, reference
commit `c1915b0a63bfabebdb95a515cedfee05146c1d50`, and exact package manifest
`bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json` were in scope.

Documentation closure review (2026-07-28): Later task-gated work objectively
changed the original bootloader boundary, proved exact-package runtime
attestation, and recorded operator LCD UAT for the current eight-second build
window. The hash-bound documentation-only successor
`2026-07-28-runtime-display-documentation` updates IO-001 and UI-001 through
UI-003 with the current runtime marker, display cadence, full-frame transfer,
and bounded I2C ownership while preserving every status and evidence column.
The authenticated Phase 36 generation and both predecessor revisions remain
byte-identical. The ordered Rust format, Clippy, all-target build, and
all-feature test sequence passed; focused parity and revision-contract Bazel
tests, the full `just test` graph, `just package`, `just parity`,
`just verify-reference`, `just verify-redaction`, Bright Builds checks,
ShellCheck, shell formatting, JSON parsing, and diff checks passed. No new
hardware attempt, credential access, network discovery, evidence promotion, or
historical artifact rewrite occurred. Whole-file `mdformat --check` remains
non-green for `TASKS.md` and the generated checklist exactly as it does at
`HEAD`; this task did not rewrite either append-only ledger for unrelated
formatting.

Residual risk: This closure documents the bounded Ultra 205 debug display only.
Full upstream display carousel, LVGL screen/task flow, physical input, active
DS4432U writes, and complete shared-I2C parity remain below verified.

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
- [x] Run the task-gated 20-cycle connected Ultra 205 durability soak only
  after the software gates pass.
- [x] Replace the split post-flash recovery observation with one continuous
  60-second bound backed by a pure reducer and bounded private diagnostics.
- [x] Verify the Attempt 002 delayed-recovery regression, the focused
  supervisor surface, and all required repository gates.
- [x] Run Attempt 003 exactly once from cycle 1 and record its typed result.

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

Completion review: Software implementation, clean-commit package
qualification, and the connected 20-cycle durability qualification are
complete. The supported repository flash and receive-only monitor workflows
finished all admitted permutations without USB or power intervention.

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

Hardware attempt 002 review: Incomplete; stopped without rerun at the first
failed boundary as required. The five detect -> flash cycles and the first
four complete receive-only-monitor -> flash cycles passed. The fifth
receive-only monitor also completed with a final standalone
`usb_session: ready`, proving the Attempt 001 framing defect fixed across all
five routine monitor captures. Its immediate reflash, operation boundary 20
(`cycle-10-flash`), stopped with earliest typed category
`recovery_not_observed`. Nineteen of the required forty operation logs ended
ready, so the twenty-cycle qualification did not complete.

Attempt 002 identity: board `205`; selected port `/dev/cu.usbmodem1101`;
source commit `1997c3145d2da7d115ed47f678a4b36d3622ec71`; reference commit
`c1915b0a63bfabebdb95a515cedfee05146c1d50`; package manifest
`bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json`; protected logs
`scratch/flash-durability/attempt-002`. The exact contract commands above were
used from cycle 1 without USB or power unplugging. The post-failure read-only
audit found zero repository-owned flash/reader processes, zero serial holders,
and an accessible transport. The protected root is mode 0700, all regular
files are mode 0600, and no symlinks exist. Detailed logs remain local,
ignored, immutable, and unpromoted. Do not rerun this unchanged boundary;
investigate the unobserved post-flash recovery under a new task-gated attempt.

Attempt 003 diagnosis and hardware contract:

- Diagnosis: Attempt 002's factory-image `espflash` child completed
  successfully, the first 30-second post-flash recovery observation expired,
  and the NVS child was not launched. The independent cleanup observation then
  admitted the same accessible, holder-free device. The targeted repository
  defect is therefore the split 30 + 30-second recovery classification, not a
  flash write or verification failure.
- Permitted commands:
  1. `just detect-ultra205`
  2. `just verify-flash-durability board=205 cycles=20 port=/dev/cu.usbmodem1101 manifest=bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json wifi-credentials=wifi-credentials.json protected-root=scratch/flash-durability/attempt-003`
- Preconditions: a virtual-time regression proves that recovery after second
  30 but before second 60 is rejected by the old policy and accepted by the
  new policy; bounded recovery diagnostics and their redaction contracts pass;
  all required software gates pass; the fix and this contract are committed;
  the exact clean-`HEAD` package is rebuilt and admitted; detection returns
  exactly `/dev/cu.usbmodem1101`; the ignored credential input exists without
  being read or printed; and the protected root does not exist.
- Recovery policy: every successful factory or NVS `espflash` child is invoked
  once and receives one continuous 60-second same-device recovery observation.
  Initial admission, monitor re-acquisition, and final cleanup retain their
  existing 30-second bounds and three stable 150 ms samples. A successful
  flash is never repeated solely because recovery is delayed.
- Evidence: only Attempt 003 may write
  `scratch/flash-durability/attempt-003`. Its root must be mode 0700 and regular
  files mode 0600. Public diagnostics contain only the closed category plus
  bounded recovery phase, deadline, booleans, stable-sample count,
  enumeration-change observation, and final state. Attempts 001 and 002 remain
  immutable, ignored, and unpromoted.
- Objective and effects: restart all four five-cycle sequences from cycle 1
  using one immutable package and the same physical board 205. The original
  Hardware contract's objective, allowed and prohibited effects, package/NVS
  semantics, one state-changing retry bound, and closed failure vocabulary
  remain unchanged. Each flash/monitor operation receives 360 seconds and its
  caller receives at least 480 seconds.
- Recovery and stop rule: stop on the first failed boundary, retain the
  earliest authoritative signature, perform only bounded supervisor cleanup
  and a read-only holder/accessibility audit, and do not rerun. Recurrence of
  the post-flash 60-second `recovery_not_observed` signature selects
  `stop_repeated_boundary`; a distinct signature returns to diagnosis without
  authorizing another attempt.
- Acceptance: all 20 cycles and 40 operation logs end with a final standalone
  `usb_session: ready`; the same physical device and exact package are used;
  zero repository-owned processes/descriptors and zero serial holders remain;
  protected modes are correct; and no USB or power unplugging occurs. On
  success, record completion, commit, and push all local commits. On failure,
  record and commit locally, keep detailed logs private, and do not push.

Hardware attempt 003 review: Complete. The exact permitted commands were run
once from cycle 1 against board `205` on `/dev/cu.usbmodem1101`, using source
commit `06885ab4449a9efb7f27b47a2aae0224a7bd14c3`, reference commit
`c1915b0a63bfabebdb95a515cedfee05146c1d50`, and the exact admitted manifest at
`bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json`. The harness returned
`durability_result=ready cycles=20`. All four five-cycle sequences completed:
detect -> flash, receive-only monitor -> flash, flash-monitor -> immediate
reflash, and SIGINT after reader admission -> cleanup -> immediate reflash.
All 40 operation logs ended with a final standalone `usb_session: ready`.

The post-run read-only audit found the same package provenance, an accessible
transport, zero repository-owned processes, zero serial holders, 40 operation
logs, and 40 terminal readiness records. The protected root
`scratch/flash-durability/attempt-003` is ignored and mode 0700; all 41 regular
files are mode 0600; no symlinks exist. The same supervisor-bound physical
device remained admitted throughout, and no USB or barrel-power unplugging was
needed. Detailed evidence remains private, ignored, and unpromoted.

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

### task-reflash-transport-recovery | 2026-07-26 | Diagnose and restore Ultra 205 reflashing

- [x] Establish a deterministic red-capable reproduction for the current
  `bootloader_connect_failed` detector boundary.
- [x] Inspect the protected device-session evidence and minimize the failure to
  one transport/reset/ownership boundary.
- [x] Rank and test three or more falsifiable hypotheses one variable at a
  time.
- [x] Add a regression before any software fix, then verify the original
  connected-device reproduction.
- [x] Rebuild an exact clean-`HEAD` package and run one detector-gated
  flash/monitor only after detector admission succeeds.
- [x] Record the root cause, verification evidence, and residual recovery risk.

Hardware contract:

- Permitted commands:
  1. `just detect-ultra205`
  2. `lsof /dev/cu.usbmodem1101`
  3. `ioreg -p IOUSB -l -w 0`
  4. `just package`
  5. `just flash-monitor board=205 port=/dev/cu.usbmodem1101 manifest=bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json evidence-dir=scratch/reflash-transport-recovery/attempt-001 capture-timeout-seconds=360 redact-evidence=true`
- Objective: restore the repository-owned bootloader connection and prove one
  exact-package reflash followed by receive-only runtime observation.
- Evidence: detailed USB, process, command, and serial material remains ignored
  `ProtectedOperational` under `scratch/device-sessions` and
  `scratch/reflash-transport-recovery`. Committed notes contain only closed
  categories, bounded counts/durations, and safe build provenance.
- Preconditions: use only the provided USB and barrel-power connections; the
  exact package must match a clean current `HEAD`; detector admission must find
  exactly one board 205; no credential file is read or supplied.
- Allowed effects: repository-supervised USB reset/hard-reset, read-only OS USB
  and holder inspection, exact admitted factory-image write, same-device
  re-acquisition, receive-only observation, and cleanup of supervisor-proven
  repository child process groups.
- Prohibited effects: erase-flash, arbitrary raw writes, NVS seeding,
  credentials, network discovery, watchdog reset, voltage/fan/mining stress,
  foreign-process termination, direct UART, pins, pads, headers, GPIO, probes,
  jumpers, soldering, injected signals, and evidence promotion.
- Recovery/restoration: every device session must release serial descriptors,
  reap its owned process group, and prove the same device accessible and
  holder-free. A failure stops without physical intervention.
- Retry bound: the existing detector failure is attempt zero. One detector
  retry is allowed only after a regression-backed software fix or an
  authorized non-invasive remediation objectively changes the boundary. No
  unchanged retry is allowed; recurrence selects `stop_repeated_boundary`.
- Accepted terminal outcomes: `complete`, `stop_repeated_boundary`,
  `stop_hardware_blocker`, `stop_authority_boundary`, or
  `stop_impossible_contract`, preserving the earliest closed category.
- Timeouts: any flash/monitor capture is at least 360 seconds with an invoking
  wall clock above 420 seconds. Ordinary silence is not failure before the
  bound; a hard transport error may stop earlier.

Diagnosis checkpoint: the failing detector session found one accessible,
holder-free ESP32-S3 USB Serial/JTAG transport, but `espflash board-info`
could not synchronize with its bootloader and supervised recovery observed no
enumeration change. The same tool and repository code had connected
successfully in preceding sessions, and the pending LCD firmware had never
been flashed. Alternative `espflash` default-reset selection resolves to the
same USB Serial/JTAG reset strategy for this device. A connector-only power
cycle changed the hardware boundary, after which the single permitted detector
retry passed immediately. The device-session error now gives that bounded
recovery procedure only for the same-device, holder-free, unchanged-enumeration
bootloader failure.

Verification: The focused Cargo regression and Bazel device-session tests
passed with the recovery guidance at commit
`697d77e027057f2ff0101e8edeb2360ad615e870`. That commit's exact-package
flash completed, but its 360-second receive-only capture correctly remained
untrusted because it contained no usable runtime proof. The targeted
runtime-attestation fix at `5b7f755d8417c1ab2ceeb75d529c96efaf6d28f3`
subsequently qualified that late-attach boundary. The later refactor smoke then
admitted one board 205, built and flashed the exact clean-`HEAD` package for
`3318a9e06d4177afb9f4bd97f32b487eb28e85f0`, observed trusted repeated runtime
attestations for the exact source, reference, and app identity during the full
360-second receive-only capture, and ended with `usb_session: ready` without a
retry. Those existing authoritative records satisfy this task's pending
detector-gated exact-package reflash objective; no unchanged hardware attempt
was run for this administrative closure.

For this closure update, the mandatory `cargo fmt --all`, Clippy with all
targets/features and denied warnings, all-target/all-feature build, and
all-feature test sequence passed. The full Bazel graph passed all 72 tests and
rebuilt the ESP32-S3 release firmware image. The managed Bright Builds checks,
reference-cleanliness, parity, redaction, and diff checks passed with no
findings or validation errors. The whole-file `mdformat --check TASKS.md`
failure is unchanged from `HEAD`; the new completion prose passes the
repository-compatible formatter in isolation, and no unrelated tracker
formatting was rewritten.

Completion review: Complete. The diagnosed root-cause boundary was retained
USB/bootloader reset state on an accessible, holder-free, same-device
transport after the expected reset produced no enumeration change. A
connector-only power cycle objectively changed that boundary and the single
permitted detector retry passed. The repository now reports that bounded
non-invasive recovery only for the matching closed failure signature and never
retries the write without an eligible state change. The later exact-package
hardware smoke proves bootloader access, factory-image write, receive-only
runtime observation, and clean session restoration after the recovery.

Residual risks: Host evidence does not distinguish the precise controller,
firmware, cable, or silicon mechanism that retained the failed reset state, so
the root cause remains bounded to the observed USB/bootloader transport
category. A recurrence still requires fresh objective boundary-change evidence
and must stop on the same post-remediation signature. Hardware proof is bound
to the cited exact package; later source changes require their own task-gated
hardware evidence before making new current-`HEAD` hardware claims.

### task-lcd-ghosting-uptime-window | 2026-07-26 | Clear row-three remnants and favor uptime

- [x] Add a capturing fake-I2C regression proving a runtime uptime frame
  transfers the complete 512-byte SSD1306 framebuffer with cleared suffix
  pixels.
- [x] Show build time for the first 3 seconds of each 60-second monotonic
  uptime cycle and uptime for the remaining 57 seconds.
- [x] Preserve the four-line layout, one-second display cadence, 500 ms sensor
  cadence, sole I2C ownership, redraw-on-change behavior, and fail-once display
  disablement.
- [x] Run focused renderer/core tests, mandatory Rust checks, full Bazel tests,
  packaging, reference-cleanliness, and diff checks.
- [x] Build an exact clean-`HEAD` package and run one detector-gated
  flash/monitor with private redacted evidence.
- [x] Record visual verification or leave it explicitly pending without
  weakening the independent serial-evidence trust policy.

Hardware contract:

- Permitted commands:
  1. `just detect-ultra205`
  2. `just package`
  3. `just flash-monitor board=205 port=/dev/cu.usbmodem1101 manifest=bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json evidence-dir=scratch/lcd-ghosting-uptime-window/attempt-001 capture-timeout-seconds=360 redact-evidence=true`
- Objective: write the exact admitted package and observe the runtime long
  enough to verify the three-second build-time window and clean transition to
  increasing uptime without disturbing the other rows.
- Evidence: detailed USB, command, and serial material remains ignored
  `ProtectedOperational` under `scratch/device-sessions` and
  `scratch/lcd-ghosting-uptime-window`. Committed notes contain only closed
  categories, bounded counts/durations, and safe build provenance.
- Preconditions: use only the provided USB and barrel-power connections; the
  package must match a clean current `HEAD`; detector admission must find
  exactly one board 205; no credential file is read or supplied.
- Allowed effects: repository-supervised USB reset/hard-reset, exact admitted
  factory-image write, same-device re-acquisition, receive-only observation,
  and cleanup of supervisor-proven repository child process groups.
- Prohibited effects: erase-flash, arbitrary raw writes, NVS seeding,
  credentials, network discovery, watchdog reset, voltage/fan/mining stress,
  foreign-process termination, direct UART, pins, pads, headers, GPIO, probes,
  jumpers, soldering, injected signals, and evidence promotion.
- Recovery/restoration: every device session must release serial descriptors,
  reap its owned process group, and prove the same device accessible and
  holder-free. A hard failure stops without an unchanged retry.
- Retry bound: one detector-gated flash/monitor attempt is allowed after the
  software fix passes all host verification. Another attempt requires a
  regression-backed software fix or an authorized non-invasive remediation
  that objectively changes the failed boundary.
- Accepted terminal outcomes: `complete`, `stop_repeated_boundary`,
  `stop_hardware_blocker`, `stop_authority_boundary`, or
  `stop_impossible_contract`, preserving the earliest closed category.
- Timeouts: the flash/monitor capture is at least 360 seconds with an invoking
  wall clock above 420 seconds. Ordinary silence is not failure before the
  bound; a hard transport error may stop earlier.
- Serial evidence policy: the known late-attach missing-boot-marker result is
  independent of LCD visual behavior. Do not weaken marker validation or
  reinterpret late runtime output as original boot-marker capture.

Verification:

- Regression-first focused tests failed before the implementation at the
  intended boundaries: row three still showed build time at 3,000 ms, and the
  shorter uptime frame transmitted only 456 of 512 framebuffer bytes.
- `bazel test //crates/bitaxe-core:tests //firmware/bitaxe:display_adapter_tests`
  passes after the fix, including schedule boundaries through 63,000 ms and
  cleared row-three suffix pixels in a complete framebuffer transfer.
- `cargo fmt --all`, Clippy with warnings denied, all-target/all-feature Cargo
  build, and all-feature Cargo tests pass in the required order.
- The first dirty-tree `just test` run passed 78 of 79 targets. The sole
  `//scripts:phase36_substantive_evidence_test` failure was traced to its
  intentional clean-source/exact-package preflight. After rebuilding the
  canonical package for the implementation commit, `just test` passed all 79
  tests, including the Phase 36 substantive evidence test.
- `just package`, `just verify-reference`, `just parity`, and `git diff
  --check` pass. The admitted manifest identified clean source commit
  `70ebf803d5a939496eb1780a9167459dc7a2bcfc`, reference commit
  `c1915b0a63bfabebdb95a515cedfee05146c1d50`, board 205, and no dirty source.
- `just detect-ultra205` admitted exactly one Ultra 205 at the contracted port.
  The single permitted flash/monitor command wrote the admitted factory image,
  reacquired the same USB device, captured for 360 seconds, released the
  session holder, and ended with `usb_session: ready`.
- Private redacted evidence is retained only under ignored
  `scratch/lcd-ghosting-uptime-window/attempt-001`. The wrapper classified the
  serial evidence `timed_out_without_trusted_output` because the late-attached
  capture omitted trusted boot markers. It was not retried or promoted.
- Diagnostic-only late runtime output contained 176 increasing uptime samples
  from 2,011 through 350,341 ms and no runtime display-disable marker. This
  does not replace the missing trusted boot evidence and is not hardware proof
  of the LCD pixels.

Completion review: The functional schedule and the SSD1306 full-frame
clear/draw/flush regression are complete, and the exact implementation package
was written to the connected board. Visual confirmation of a clean transition,
the three-second minute-boundary build window, and unchanged surrounding rows
passed operator UAT on 2026-07-26. The independent late-monitor boot-marker
issue remained unchanged and out of scope; no marker validation was weakened.

### task-eight-second-build-window-runtime-attestation | 2026-07-26 10:35 | Extend LCD build window and distinguish post-flash proof

- [x] Show build time during uptime milliseconds `0..=7_999` of every
  60-second cycle and uptime for the remaining 52 seconds.
- [x] Add a versioned, redaction-safe runtime boot attestation in the pure API
  model and replay it from the boot-lifetime firmware owner after startup
  readiness.
- [x] Accept either the unchanged original boot transcript or two consistent,
  monotonic exact-package runtime attestations without conflating the two trust
  bases.
- [x] Record flash effect, boot transcript, runtime attestation, and overall
  trust as separate additive evidence fields. An untrusted monitor remains
  nonzero but must state that flashing completed and must not recommend an
  automatic reflash.
- [x] Preserve the native receive-only reader, hard-reset lifecycle, private
  evidence policy, full framebuffer redraw, display/I2C cadences, and archived
  Phase 28.1.1 closure.
- [x] Update ADR/runbook guidance and append the durable separated-outcome
  lesson.
- [x] Run regression-first focused tests, mandatory Rust checks, full Bazel
  tests, packaging, reference/parity/redaction verification, and diff checks.
- [x] Build an exact clean-`HEAD` package and run one detector-gated
  flash/monitor with private redacted evidence.
- [x] Record hardware trust basis and operator LCD UAT, or leave either
  explicitly pending without weakening the evidence contract.

Hardware contract:

- Permitted commands:
  1. `just detect-ultra205`
  2. `just package`
  3. `just flash-monitor board=205 port=/dev/cu.usbmodem1101 manifest=bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json evidence-dir=scratch/lcd-eight-second-attestation/attempt-001 capture-timeout-seconds=360 redact-evidence=true`
- Objective: write the exact admitted package, verify either original
  boot-transcript trust or the distinct runtime-attestation trust basis, and
  observe the eight-second LCD build window without disturbing other rows.
- Evidence: detailed USB, command, and serial material remains ignored
  `ProtectedOperational` under `scratch/device-sessions` and
  `scratch/lcd-eight-second-attestation`. Committed notes contain only typed
  categories, safe provenance, counts, and bounded durations.
- Preconditions: use only the provided USB and barrel-power connections; the
  package must match a clean current `HEAD`; detector admission must find
  exactly one board 205; no credential file is read or supplied.
- Allowed effects: repository-supervised USB reset/hard-reset, exact admitted
  factory-image write, same-device re-acquisition, receive-only observation,
  and cleanup of supervisor-proven repository child process groups.
- Prohibited effects: erase-flash, arbitrary raw writes, NVS seeding,
  credentials, network discovery, watchdog reset, voltage/fan/mining stress,
  foreign-process termination, direct UART, pins, pads, headers, GPIO, probes,
  jumpers, soldering, injected signals, archived diagnostic invocation, and
  evidence promotion.
- Recovery/restoration: every device session must release serial descriptors,
  reap its owned process group, and prove the same device accessible and
  holder-free. A hard failure stops without an unchanged retry.
- Retry bound: one detector-gated flash/monitor attempt is allowed only after
  the software fix passes all host verification. Another attempt requires a
  regression-backed software fix or authorized non-invasive remediation that
  objectively changes the failed boundary.
- Accepted terminal outcomes: `complete`, `stop_repeated_boundary`,
  `stop_hardware_blocker`, `stop_authority_boundary`, or
  `stop_impossible_contract`, preserving the earliest typed failure.
- Timeouts: flash/monitor capture is at least 360 seconds with an invoking wall
  clock above 420 seconds. Ordinary silence is not failure before the bound; a
  hard transport error may stop earlier.
- Trust policy: an original boot transcript and a replayed runtime attestation
  remain distinct. Runtime trust requires at least two same-session,
  same-ordinal attestations with identical static facts, increasing uptime,
  and exact source, reference, and app-ELF identity matches.

Verification:

- Regression-first proof recorded the intended pre-fix failures: row three
  switched to uptime at 7,999 ms, the late-attached repeated exact-package log
  was rejected for missing startup markers, and untrusted-monitor output did
  not separate completed flashing from monitor failure.
- Focused API/core/renderer/flash tests pass, including malformed, stale
  package, wrong reference/digest, mixed session/ordinal, non-monotonic,
  single-sample, incomplete-readiness, full-framebuffer, both-trust-path, and
  terminal-write-failure cases.
- `cargo fmt --all`, Clippy with all targets/features and denied warnings,
  all-target/all-feature build, and all-feature tests pass in the mandatory
  order.
- The full 79-target `just test` graph passes from clean implementation commit
  `5b7f755d8417c1ab2ceeb75d529c96efaf6d28f3`, including the ESP32-S3 release
  firmware compile, package graph, and the clean-HEAD-only Phase 36 process
  gate. The gate first rejected the intentionally dirty worktree, then rejected
  the stale dirty-build manifest until `just package` refreshed it; it passed
  once both source and package identities were clean and exact.
- `just verify-reference`, `just parity`, and `just verify-redaction` pass.
- `just package` produced a schema-v3 manifest with
  `source_dirty=false` and exact source commit `5b7f755d8417c1ab2ceeb75d529c96efaf6d28f3`.
- `just detect-ultra205` admitted exactly one board 205. The single authorized
  360-second flash/monitor attempt completed with
  `flash_status=completed`, `monitor_evidence_status=trusted`,
  `boot_transcript_status=missing`,
  `runtime_attestation_status=trusted`,
  `trust_basis=runtime_attestation`, and
  `capture_status=timed_out_after_trusted_output`. The redacted capture
  contained 35 attestations from one session and one ordinal, matched the exact
  source/reference/app-ELF package identity, ended with `usb_session: ready`,
  and performed no retry.

Completion review:

- Row three now shows build time for uptime milliseconds `0..=7_999` of every
  minute and uptime for the remaining 52 seconds while preserving complete
  framebuffer clearing and the existing display/I2C cadence.
- Firmware emits the versioned ready-state attestation immediately after OTA,
  SPIFFS, and route-shell readiness and every ten seconds thereafter. The flash
  tool keeps original boot transcript and replay attestation as separate trust
  bases and no longer describes missing monitor evidence as a failed write or
  recommends an automatic reflash.
- The real late-attach hardware boundary passed through runtime-attestation
  trust against the exact clean package while correctly retaining
  `boot_transcript_status=missing`.
- Operator UAT on 2026-07-26 confirmed that the build timestamp remains visible
  for eight seconds. The prior clean-transition and unchanged-row UAT remains
  green, closing the final LCD timing observation without changing the parity
  claim.

### task-production-mining-session | 2026-07-26 | Build the production mining owner and retire obsolete phase runtimes

- [x] Record the GSD-era removal rule, Production Mining Session definition,
      and sole-ownership ADR.
- [x] Add the pure production-session lifecycle, recovery policy, typed
      interfaces, and deterministic tests to `bitaxe-stratum`.
- [x] Add the thin ESP owner, lazy pool settings seam, operator-intent
      projection, boot preference, and category-only notifications.
- [x] Remove the obsolete Phase 21, Phase 25, Phase 27, and dependent Phase 28
      executable paths without compatibility aliases.
- [x] Add focused source-contract and production-session verification wiring.
- [x] Run Rust format, Clippy, build, and tests in the required order, followed
      by focused verification, repository tests, packaging, parity,
      reference-integrity, redaction, source scans, and diff review.

Verification policy:

- Software fakes may prove protocol, lifecycle, recovery, and projection
  behavior. They do not promote hardware or parity evidence.
- Do not flash hardware, connect to a real pool, mine, or actuate hardware for
  this task. The ordinary ESP implementation remains actuation-unqualified and
  fail-closed before pool-secret access or external effects.

Completion review: Complete. The pure session and fail-closed ESP owner now own
operator intent, readiness, recovery, lifecycle projection, and ordered safe
stop. The focused verifier, ESP32-S3 firmware build, host format/Clippy/build/
tests, package, parity, reference-integrity, redaction, source scan, and diff
checks pass. The repository-wide Bazel suite passes 72 of 73 tests;
the remaining Phase 36 publication test intentionally reports
`source_tree_not_clean` until this implementation is committed. Its evidence
guard was preserved rather than weakened.

Residual risks: Real networking and ASIC actuation remain fail-closed and
outside this task. Hardware qualification, live pool recovery, and successful
share submission still require separately authorized evidence before any parity
promotion.

### task-software-complete-production-mining-engine | 2026-07-26 | Deepen the Production Mining Session

- [x] Replace the recursive lifecycle-action shell with one event/effect
      Production Mining Session that owns recovery, V1 protocol state, framing,
      work correlation, submit classification, and ordered safe stop.
- [x] Add deterministic pool, clock, ASIC, settings, and projection adapters
      that prove the complete admitted software lifecycle.
- [x] Keep the ordinary ESP adapter actuation-unqualified and fail closed
      before pool-secret, network, socket-write, or ASIC effects.
- [x] Remove the superseded mining-loop and direct fake/live-runtime paths,
      including active build, test, marker, and documentation wiring.
- [x] Update the ownership ADR, Production Mining Session context, focused
      verifier, and current implementation pointers without promoting parity.
- [x] Run the required Rust checks in order, focused production-session
      verification, repository tests, packaging, parity, reference-integrity,
      redaction, source scans, and diff review.
- [x] Retire the legacy Phase 36 substantive-evidence test and build wiring,
      then rerun the repository test suite before publication.

Verification policy:

- Deterministic accepted and rejected shares are software evidence only.
- Do not read real pool credentials, connect to a pool, flash hardware, mine,
  actuate hardware, or promote parity during this task.

Completion review: Complete. The Production Mining Session now owns one
iterative event/effect lifecycle across admission, recovery, V1 framing and
negotiation, centrally allocated generations, work/nonce/submit correlation,
telemetry snapshots, fallback probing, and ordered safe stop. The deterministic
adapter proves accepted and rejected software outcomes; the ordinary ESP
adapter remains actuation-unqualified and the focused source contract proves
that it contains no pool-secret, TCP/socket-write, or ASIC-effect path.

Verification passed for the ordered Cargo format, Clippy, all-target build, and
all-feature test sequence; `just verify-production-session`; `just package`;
`just parity`; `just verify-reference`; `just verify-redaction`; active-source
scans; `git diff --check`; and final diff review. `just test` passed 71 of 72
targets before the obsolete Phase 36 substantive-evidence process test and its
Bazel wiring were retired; the final repository run passed all 71 remaining
tests. The production Phase 36 implementation, authenticated parity checklist,
historical evidence, and `.planning/` archives were preserved. The focused
source contract now guards against restoring the retired test or active build
references, and current software verification is recorded separately.

Residual risks: Real pool secrets, TCP/TLS, socket writes, ASIC effects, device
flashing, mining, and hardware evidence remain unqualified and untested.
Deterministic share outcomes do not promote parity or release readiness.

### task-item4-refactor-hardware-smoke | 2026-07-27 10:48 | Flash and observe the refactored firmware

- [x] Confirm the repository is clean at refactor commit
      `e33eec2b4fff4161d5bf0f8c55ec3d02e426b8a1`.
- [x] Run `just detect-ultra205` and admit exactly one ESP32-S3 Ultra 205.
- [x] Build the exact clean-HEAD package and retain its manifest identity.
- [x] Flash the admitted factory image and capture at least 360 seconds of
      receive-only runtime output.
- [x] Review the redacted evidence for boot, runtime, and cleanup results
      without promoting parity or making new hardware claims.

Hardware contract:

- Permitted commands:
  1. `just detect-ultra205`
  2. `just package`
  3. `just flash-monitor board=205 port=/dev/cu.usbmodem1101 manifest=bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json evidence-dir=scratch/item4-refactor-hardware-smoke/attempt-001 capture-timeout-seconds=360 redact-evidence=true`
- Objective: prove the exact packaged firmware after the flash, parity, NVS,
  and HTTP module extractions can be admitted, written to the detected Ultra
  205, boot, remain observable for the bounded capture, and restore the USB
  session cleanly. This is a regression smoke, not parity promotion.
- Evidence: `scratch/item4-refactor-hardware-smoke/attempt-001` is ignored,
  local, non-promoted evidence. The supervisor owns its mode-0700 private
  parent; serial, USB, command, and process artifacts remain mode-0600
  `ProtectedOperational`. Only redacted output and closed result categories may
  be summarized.
- Preconditions: the read-only detector must admit exactly one board 205;
  `just package` must produce an exact clean-HEAD manifest; reference integrity
  must remain clean; no Wi-Fi or pool credential file may be read or supplied.
  The successful 2026-07-27 detector run is new diagnostic information relative
  to the earlier `bootloader_connect_failed` boundary.
- Allowed effects: write and verify only the exact admitted factory image,
  perform the repo-owned reset and re-enumeration sequence, observe serial
  output receive-only, reacquire the same physical device, and clean up only
  supervisor-proven repository child process groups.
- Prohibited effects: erase-flash, arbitrary raw writes, NVS seeding,
  credentials, network discovery, OTA, recovery upload, mining, voltage/fan or
  thermal stress, foreign-process termination, direct UART, pins, pads,
  headers, GPIO, probes, jumpers, soldering, or injected signals.
- Recovery/restoration: the device-session supervisor must terminate and reap
  owned process groups, release serial descriptors, and prove the admitted
  device accessible and holder-free. Success leaves the admitted package
  installed. Identity drift, absence, a foreign holder, or unproved cleanup
  stops without physical intervention.
- Retry bound: no unchanged retry. A new attempt ordinal is allowed only after
  a regression-backed software fix or an authorized non-invasive remediation
  objectively changes the failed boundary. Recurrence of the same signature
  selects `stop_repeated_boundary`.
- Accepted terminal outcomes: `complete`, `stop_repeated_boundary`,
  `stop_hardware_blocker`, `stop_authority_boundary`, or
  `stop_impossible_contract`. The earliest authoritative failure is preserved.
- Timeouts: monitor capture is at least 360 seconds and the invoking wall-clock
  allowance exceeds 420 seconds. Ordinary silence is not failure before the
  full bound.

Verification: `just detect-ultra205`, the ordered Cargo format, Clippy,
all-target build, and all-feature test sequence, `just verify-reference`,
`just package`, and the exact authorized `just flash-monitor` command passed.
The evidence files are mode 0600 under a mode-0700 ignored local directory.

Completion review: Complete. The exact clean package for source commit
`3318a9e06d4177afb9f4bd97f32b487eb28e85f0` and reference commit
`c1915b0a63bfabebdb95a515cedfee05146c1d50` was flashed to the single detected
board 205. The 360-second receive-only capture ended
`timed_out_after_trusted_output` with `flash_status: completed`,
`runtime_attestation_status: trusted`, exact observed commit agreement, and
`usb_session: ready`. The original boot transcript was not captured, so the
trusted repeated runtime attestation is the bounded success basis. The
redacted log contained no panic, abort, or Guru Meditation markers. No retry,
credentials, network discovery, direct UART, pin manipulation, stress
operation, or parity promotion occurred.

### task-fix-provenance-link-and-pure-core-coverage | 2026-07-27 16:30 | Fix provenance link and pure-core coverage

- [x] Link the visible fallback and recovery source commit to the exact public
      GitHub commit after validating the full hash.
- [x] Preserve explicit `Unavailable` behavior and safe external-link
      attributes for missing, malformed, or unavailable provenance.
- [x] Add focused Stratum runtime, parser, and client-message unit coverage for
      every branch identified by the Bright Builds audit.
- [x] Add focused OpenAPI comparison and Phase 36 effect-result unit coverage
      for every branch identified by the Bright Builds audit.
- [x] Prove at least 95% line coverage in each of the four audited low-coverage
      modules and execute the uncovered `Pong` and `SendVersion` branches.
- [x] Run the full ordered verification sequence and confirm historical
      evidence, parity statuses, schemas, and public interfaces remain
      unchanged.

Verification policy:

- This task is software-only. Do not detect, flash, monitor, read credentials,
  discover devices or network targets, generate evidence, or promote parity
  claims.
- Preserve public Rust APIs, firmware endpoints, JSON schemas, CLI arguments,
  evidence policy, redaction behavior, and hardware safety gates.
- A source-derived Phase 36 evaluator digest rotation is expected because its
  inventoried implementation file contains the expanded unit suite. Do not
  regenerate, rewrite, or promote historical evidence or checklist revisions.
- The scoped acceptance threshold is at least 95% line coverage for
  `live_runtime.rs`, `messages/server.rs`, `api_compare/openapi.rs`, and
  `hardware_process/effect_result.rs`; no permanent global coverage gate is
  introduced.

Verification:

- The static provenance Bazel test passed for clean, dirty, malformed,
  unavailable, and throwing fallback and embedded-recovery inputs.
- `cargo +stable llvm-cov` measured `live_runtime.rs`,
  `messages/server.rs`, and `messages/client.rs` at 100% line coverage,
  `api_compare/openapi.rs` at 100%, and
  `hardware_process/effect_result.rs` at 98.84%.
- The Stratum crate passed 178 tests, including separate `Pong` and
  `SendVersion` cases. The focused OpenAPI and effect-result suites each passed
  12 tests.
- The ordered `cargo fmt --all`,
  `cargo clippy --all-targets --all-features -- -D warnings`,
  `cargo build --all-targets --all-features`, and
  `cargo test --all-features` sequence passed.
- `bun scripts/bright-builds-check.ts all` reported zero findings with exactly
  eight existing exceptions. `just test` passed all 72 Bazel tests,
  `just parity` reported `validation_errors: none`, `just verify-reference`
  reported the pinned reference clean, and `just verify-redaction` passed.
- The source-derived Phase 36 evaluator identity rotated to
  `fb45f3578257cb37a4a73572a7ea0643a93e11dae8762966bb71a6b036cb296c`;
  evaluator inventory and drift tests passed, and no historical evidence,
  checklist revision, or parity status changed.

Completion review: Complete. Both user-visible provenance surfaces now link a
validated short commit label to the exact public GitHub commit and fail closed
to an href-less `Unavailable` state. Focused pure-core suites close every
audited coverage gap without changing production interfaces or behavior.

Residual risks: Coverage is host-side software evidence only. No firmware
hardware path, credentials, device, network target, or parity evidence was
used, and no hardware or parity claim changed.

## Backlog

### task-private-first-remaining-evidence-pipelines | 2026-07-20 | Migrate remaining evidence workflows

- [x] Inventory active evidence-producing workflows outside the archived
      Phase 35 workflow against `docs/parity/evidence-policy.md`.
- [x] Migrate any active workflow that still performs in-place or post-write
  sanitization to private-first capture with a distinct shareable projection.
- [x] Add focused regressions and route every admitted projection through
  `just verify-redaction`.
- [x] Run the repository verification required for every changed language and
      workflow surface.

Working plan and bounded inventory (2026-07-27):

- Audit scope is the current human command surface in `justfile`: the
  flash/monitor/finalization family, the private-only flash-durability and
  device-session diagnostics, Phase 23 operator evidence, Phase 33 settings
  durability, and Phase 36 candidate capture/classification. Phase 35 is
  explicitly excluded by this task; archived `.planning/` workflows and
  non-routed legacy phase binaries are historical rather than active command
  surfaces.
- The flash family already sanitizes independent child streams before their
  first write and uses digest-bound distinct finalization when a private
  classifier is required. Flash durability and device-session diagnostics
  retain only mode-`0600` `ProtectedOperational` files below ignored
  mode-`0700` roots. Phase 36 already separates private capture, candidate,
  seal, and classification artifacts. No active workflow performs in-place or
  post-write sanitization.
- Phase 23 and Phase 33 already derive distinct shareable projections without
  copying private artifacts, but they do not route an untracked or unstaged
  candidate projection through the canonical repository redaction adapter.
  Add a fail-closed explicit-projection mode to `just verify-redaction`, route
  those two generators through it, and add fresh-process regressions for safe
  acceptance, non-echoing rejection, and exactly-once generator validation.
- Verify the focused shell targets, shell formatting and ShellCheck, the
  managed Bright Builds checks, the complete repository suite, parity,
  reference integrity, redaction, and the final scoped diff.

Verification:

- Direct fresh-process tests passed for `verify-redaction`, Phase 23 operator
  evidence, and Phase 33 settings durability. They cover safe untracked
  projection acceptance, non-echoing sensitive-value rejection, symlink and
  outside-workspace rejection, exactly-once generator validation, and
  fail-closed generator outcomes.
- Focused Bazel tests passed for
  `//scripts:verify_redaction_test`,
  `//scripts:phase23_redacted_operator_evidence_test`, and
  `//scripts:phase33_confirmed_settings_durability_test`.
- `shfmt -d` and ShellCheck passed for every changed shell file.
- The required ordered Rust sequence passed: `cargo fmt --all`,
  `cargo clippy --all-targets --all-features -- -D warnings`,
  `cargo build --all-targets --all-features`, and
  `cargo test --all-features`.
- `bun scripts/bright-builds-check.ts all` scanned 558 files with the eight
  existing justified exceptions and zero findings. `just test` passed all 72
  Bazel tests, including the ESP32-S3 firmware compile and package graph.
- `just parity` reported `validation_errors: none`; `just verify-reference`
  confirmed pinned commit `c1915b0a63bfabebdb95a515cedfee05146c1d50`
  clean; `just verify-redaction` and `git diff --check` passed.

Completion review:

- The canonical redaction adapter can now validate one or more explicit
  untracked or unstaged projection files/directories without echoing matched
  content, following symlinks, or accepting paths outside the workspace.
- Phase 23 and Phase 33 invoke that adapter exactly once after their typed
  projection is complete and before reporting success. Phase 33's duplicate
  ad hoc post-write regex was removed so the canonical policy is the sole
  projection admission check.
- The bounded active-workflow audit found no in-place/post-write sanitizer to
  migrate: existing private classifiers already consume immutable
  secret-sanitized inputs before distinct projection, while private-only
  diagnostics remain protected and unadmitted.
- Residual risk: non-routed legacy phase binaries remain historical surfaces
  outside the active `justfile` command inventory. Any future reactivation must
  first pass the same private-first audit. No hardware, credentials, network
  discovery, evidence generation or promotion, historical artifact rewrite,
  or parity-status change occurred.

The task's original backlog scope did not itself authorize push operations; the
current explicit `work-top-task` invocation supplied that authorization. All
other original prohibitions remained in force.

## Future — Explicit Only

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

### task-refactor-oversized-active-modules | 2026-07-27 12:00 | Refactor audit item 4

- [x] Split the flash CLI into responsibility-aligned modules while preserving
      its command, evidence, supervision, and redaction contracts.
- [x] Split the parity CLI into command, environment, checklist, rendering, and
      validation modules without changing report or promotion behavior.
- [x] Split the NVS model into typed schema, migration, loading, and test
      modules while preserving its public exports and exact schema order.
- [x] Split the firmware HTTP adapter into route, access, response, deferred
      effect, and WebSocket modules without changing its public route behavior.
- [x] Update Bazel ownership and source guards without weakening their
      supervision, ordering, or evidence assertions.
- [x] Publish a documentation-only hash-bound checklist successor for CFG-004,
      OTA-001, OTA-002, REL-003, and EVD-08 without changing status or evidence.
- [x] Verify focused suites, the required Rust sequence, firmware packaging,
      production-session, repository tests, parity, reference integrity,
      redaction, source sizes, historical evidence digests, and the final diff.

Verification policy:

- This is structure-only software work. Do not detect, flash, monitor, discover,
  read credentials, exercise hardware, or promote parity.
- Treat roughly 628 file lines and 161 function lines as refactor triggers, not
  hard caps. Record any deliberate exception in this block.

Completion review: The four entrypoints are now 106, 222, 24, and 142 lines,
and every newly extracted module is below 628 lines. The extracted production
functions are below 161 lines. Focused Bazel contracts, all 72 repository test
targets, the required Cargo format/Clippy/build/test sequence, firmware package,
production-session verification, parity, reference integrity, and redaction
all pass. The new five-row checklist authority is chained to the prior
17-row documentation revision; Phase 36 artifact digests remain unchanged.

Residual risks: Smaller pre-existing outliers outside the four-module scope
remain candidates for later bounded refactors. No hardware was exercised, and
no hardware or parity classification was promoted.

### task-close-bright-builds-audit-item-1 | 2026-07-27 20:58 | Refactor oversized active functions

- [x] Refactor all five active Rust functions above the roughly 161-line
      code-shape trigger without changing public interfaces or behavior.
- [x] Preserve device-session event ordering, request-once semantics, terminal
      classification, timing, cleanup, artifacts, and redaction.
- [x] Preserve firmware startup ordering, build identity, readiness, and Phase
      34 source authority while extracting private startup orchestration.
- [x] Preserve Phase 36 authenticated fixture bytes, digests, roles,
      permissions, validation order, and cleanup.
- [x] Preserve the CLI reboot-boundary matrix and operator snapshot chronology
      regression, including the existing source-guard markers.
- [x] Run focused suites, the required Rust sequence, firmware build,
      repository tests, parity, reference integrity, redaction, function/file
      length checks, and final diff review.

Verification policy:

- This is structure-only software work. Do not detect, flash, monitor, discover,
  read credentials, exercise hardware, modify historical evidence, or promote
  parity.
- Keep each affected function at or below 161 lines and each touched or new
  source file below 628 lines.

Completion review: The five audited functions are now 26, 3, 28, 9, and 10
lines. A lexical repository scan covered 366 active Rust files and 4,392
functions with zero functions above 161 lines; every touched or new source file
is below 628 lines. Device-session's 50 focused tests, parity's 381 focused
tests, the ESP32-S3 firmware build, the required Cargo format/Clippy/build/test
sequence, all 72 Bazel test targets, Bright Builds checks, parity validation,
reference integrity, redaction, and final diff checks pass.

Residual risks: This structure-only change intentionally did not exercise
hardware. Firmware startup behavior is supported by compilation, source guards,
and repository tests rather than new hardware evidence. The eight existing
documented file-length exceptions remain unchanged; this task added none and
did not modify historical evidence or parity status.

## Architecture Conformance Backlog

### task-repair-production-session-source-contract | 2026-07-27 21:44 | Repair the production-session architecture guard

- [x] Replace the stale `adapter.execute(effect)` source assertion with the
      current `adapter.maybe_execute(effect)` contract.
- [x] Preserve the exactly-two-adapters assertion and every existing
      ordinary-adapter forbidden-I/O check.
- [x] Add or update focused regressions so the guard fails when the current
      event/effect interpreter seam, adapter count, or forbidden-I/O boundary
      drifts.
- [x] Run the source guard, focused core tests, `just
      verify-production-session`, and the canonical firmware build.

Dependencies: None. Complete this task before relying on the production-session
source contract as verification for later architecture work.

Verification:

- The pre-change `just verify-production-session` failure was reproduced after
  its three focused core targets passed:
  `owner contract missing: adapter.execute(effect)`.
- The new focused Bazel shell test passes the current contract and proves
  failures for a missing `adapter.maybe_execute(effect)` seam, a third adapter,
  and forbidden `std::net::TcpStream` ownership. It also passed in Bazel's
  `rg`-free sandbox through the production guard's `grep` fallback.
- `shfmt -d`, ShellCheck, the source guard, and
  `just verify-production-session` passed. The latter ran the focused guard
  regression, all three production-session core targets, and the canonical
  ESP32-S3 firmware build.
- The required ordered Rust sequence passed: `cargo fmt --all`,
  `cargo clippy --all-targets --all-features -- -D warnings`,
  `cargo build --all-targets --all-features`, and
  `cargo test --all-features`.
- `bun scripts/bright-builds-check.ts all` scanned 558 files with eight
  existing exceptions and zero findings; `just test` passed all 73 Bazel test
  targets, including the new focused guard regression.
- `just parity` reported `validation_errors: none`; `just verify-reference`
  confirmed pinned commit `c1915b0a63bfabebdb95a515cedfee05146c1d50`
  clean; `just verify-redaction` and `git diff --check` passed.

Completion review:

- Repaired the architecture source guard at the renamed production adapter
  boundary and kept the interpreter seam, exact adapter inventory, and
  ordinary-adapter I/O prohibition under one production/tested check.
- Residual risk is limited to the source guard's intentional textual contract;
  typed production-session behavior remains covered by the existing core
  tests. No runtime behavior, hardware, credentials, network discovery,
  evidence, parity status, reference contents, or evaluator inventory changed.
  The current explicit `work-top-task` invocation supplies commit and push
  authorization for this task only.

### task-remove-core-clock-ownership | 2026-07-27 21:44 | Make production-session timing deterministic

- [x] Replace `Instant` ownership in `BridgeOrchestrator` and
      `ProductionMiningSession` with caller-supplied monotonic milliseconds.
- [x] Keep `ProductionSessionEvent::now_ms` as the sole production-session time
      input and remove all real-clock reads from the reusable Stratum core.
- [x] Preserve dispatch-before-poll priority, regeneration cadence, timeout
      telemetry, invalidation, and fail-closed session behavior.
- [x] Add deterministic tests for dispatch priority, pre-threshold and
      at-threshold regeneration, timeout behavior, clock regression, and
      saturating timestamp arithmetic.
- [x] Run focused Stratum and API tests, the repaired production-session source
      contract, and the relevant complete software verification surface.

Dependencies: Complete the production-session architecture-guard repair first.

Verification:

- The pre-change inventory found `Instant` ownership in
  `BridgeOrchestrator::maybe_last_dispatch_at`,
  `ProductionMiningSession::bridge_epoch`, and the epoch-plus-duration
  conversion in `drive_bridge`. The reusable engine now stores and compares
  only caller-supplied `u64` millisecond values with `saturating_sub`.
- `cargo test -p bitaxe-stratum --all-features` passed all 182 tests. Focused
  regressions preserve dispatch-before-poll priority, pre-threshold polling,
  at-threshold regeneration, non-terminal timeout cadence, regressed-clock
  behavior, and exact `u64::MAX` regeneration without overflow.
- The production-session source guard now rejects `Instant` or `SystemTime`
  ownership in the reusable engine. Its direct regression, `shfmt -d`,
  ShellCheck, and the complete source contract passed.
- The required ordered Rust sequence passed: `cargo fmt --all`,
  `cargo clippy --all-targets --all-features -- -D warnings`,
  `cargo build --all-targets --all-features`, and
  `cargo test --all-features`.
- Focused Bazel Stratum, API, and source-guard tests passed.
  `just verify-production-session` passed its Stratum, API, config, and guard
  targets plus the canonical ESP32-S3 firmware build.
- `bun scripts/bright-builds-check.ts all` scanned 561 files with the eight
  existing justified exceptions and zero findings. `just test` passed all 73
  Bazel test targets, including firmware compilation and packaging.
- `just parity` reported `validation_errors: none`; `just verify-reference`
  confirmed pinned commit `c1915b0a63bfabebdb95a515cedfee05146c1d50`
  clean; `just verify-redaction` and `git diff --check` passed.
- Final scope review found no changes to hardware/evidence artifacts, the
  parity checklist, planning archives, reference contents, schemas, or parity
  status.

Completion review:

- The Production Mining Session and bridge now share one deterministic
  caller-supplied monotonic-millisecond domain. Clock regression saturates
  elapsed time to zero, while maximum timestamps preserve exact cadence
  without epoch arithmetic or overflow.
- Residual risk is limited to the firmware shell supplying monotonic event
  timestamps; this software-only task compiled that shell but did not exercise
  hardware, credentials, live networking, or mining actuation. The current
  explicit `work-top-task` invocation supplies commit and push authorization
  for this task only.

### task-relocate-reusable-crate-concurrency-shells | 2026-07-27 21:44 | Move concurrency ownership into firmware adapters

- [x] Keep pure snapshot, sequence, effect, transition, and error models in
      reusable crates while moving process-lifetime synchronization ownership
      into firmware adapters.
- [x] Move the `Mutex`-owned confirmed-settings cell into the firmware settings
      adapter without changing storage-confirmation or poison-handling behavior.
- [x] Move operator-snapshot publication locking into the firmware runtime
      adapter while preserving completion order, retention-before-issuance,
      reentrancy rejection, and earliest-failure classification.
- [x] Move deferred-effect channels, leases, worker lifecycle, and
      response-before-effect coordination into the firmware HTTP shell without
      compatibility aliases.
- [x] Move concurrency-focused tests to host-testable shell targets, update
      affected source guards, and record expected source-derived evaluator
      identity rotation without hard-coding or promoting the new identity.
- [x] Run focused config, API, firmware-shell, and parity tests plus the
      repository's complete required software verification sequence.

Dependencies: Complete the production-session architecture-guard repair first.
Coordinate source-guard updates with the host-tool core/shell separation task
if both touch parity architecture assertions.

Verification:

- `cargo fmt --all`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo build --all-targets --all-features`
- `cargo test --all-features`
- `bazel test //firmware/bitaxe:settings_snapshot_store_tests
  //firmware/bitaxe:deferred_effect_queue_tests
  //firmware/bitaxe:operator_snapshot_publication_shell_tests
  //firmware/bitaxe:retained_pair_production_tests
  //crates/bitaxe-config:tests //crates/bitaxe-api:tests
  //tools/parity:tests`
- `just verify-production-session`
- `bun scripts/bright-builds-check.ts all`
- `just test` (76 Bazel tests passed, including firmware image packaging)
- `just parity` (`validation_errors: none`)
- `just verify-reference`
- `just verify-redaction`

Completion review:

- Reusable API and config crates now retain pure snapshot, sequence, effect,
  transition, read-health, and failure models. Confirmed-settings storage,
  ordered operator-snapshot publication, and deferred-effect worker ownership
  live in firmware adapters with dedicated host-testable shell targets.
- Existing poison recovery, completion ordering, retention-before-issuance,
  revision consumption, reentrancy rejection, earliest-failure
  classification, and response-before-effect behavior remain covered.
- Phase 34 source guards now fail if the reusable models reacquire
  synchronization or if the firmware shells lose their required ownership
  primitives. Because evaluator identity is source-derived, this source move
  is expected to rotate it; no identity was hard-coded and no evidence or
  parity status was generated or promoted.
- Residual risk is limited to device-runtime scheduling differences: the
  ESP32-S3 release firmware compiled and packaged, but this software-only task
  did not detect, flash, monitor, or otherwise interact with hardware,
  credentials, live networking, or mining actuation. The explicit
  `work-top-task` invocation supplies commit and push authorization for this
  task only.

### task-centralize-ipv4-access-classification | 2026-07-27 21:44 | Move peer-address policy into the pure API core

- [x] Move peer-address byte-order normalization and RFC1918 classification
      from the ESP-IDF HTTP adapter into `bitaxe-api::route_shell`.
- [x] Expose one typed peer-address normalization function for the firmware
      adapter and remove the duplicated firmware classifier.
- [x] Keep raw socket access, ESP-IDF calls, logging, and request handling in
      the firmware shell.
- [x] Add pure unit tests for network-order private addresses, host-order
      fallback, unspecified addresses, public addresses, and all three RFC1918
      ranges.
- [x] Run focused API route-shell tests, firmware compilation, access source
      guards, and the relevant complete software verification surface.

Dependencies: None.

Working plan (2026-07-27):

- Move raw peer-address normalization and RFC1918 classification into the pure
  route shell behind one typed result that preserves fallback provenance.
- Keep socket acquisition and fallback logging in the ESP-IDF adapter, then
  remove its private normalization and classification helpers.
- Add focused pure normalization tests and strengthen the active access source
  guard before running the required focused and complete software gates.

Verification:

- All 214 focused `bitaxe-api` tests passed, including network-order private,
  host-order fallback, unspecified, public, and RFC1918 boundary cases. The
  focused parity architecture guard passed, and the focused Bazel API/parity
  targets passed.
- The required ordered Rust sequence passed using a clean isolated Cargo
  target: `cargo fmt --all`,
  `cargo clippy --all-targets --all-features -- -D warnings`,
  `cargo build --all-targets --all-features`, and
  `cargo test --all-features`.
- `just build` compiled the ESP32-S3 release firmware against ESP-IDF `v5.5.4`.
  The complete isolated `bazel test //...` graph passed all 76 tests and
  rebuilt the firmware image and package.
- `bun scripts/bright-builds-check.ts all` scanned 565 files with the eight
  existing justified exceptions and zero findings. `just parity` reported
  `validation_errors: none`; `just verify-reference` confirmed pinned commit
  `c1915b0a63bfabebdb95a515cedfee05146c1d50` clean; and
  `just verify-redaction` passed.
- `git diff --check`, source-ownership scans, the final ten-file scoped diff,
  and touched-file sizes passed review. A pre-existing ignored macOS Cargo
  cache stalled the first focused Cargo run and the first full Bazel workspace
  load; it was recoverably quarantined outside the workspace before both
  surfaces passed against clean isolated build state.

Completion review:

- Peer IPv4 normalization and RFC1918 classification now live in the pure API
  route shell behind `PeerIpv4Normalization`, which preserves whether the
  selected address came from the ordinary network-order path or the bounded
  host-order fallback.
- The firmware adapter owns only ESP-IDF socket access, unavailable-peer
  handling, fallback diagnostics, and request assembly. A focused source guard
  rejects policy or byte-order logic returning to that effectful shell.
- Residual risk is limited to the existing host-order compatibility fallback:
  the supported ESP32-S3 and host verification targets are little-endian, and
  no new runtime hardware evidence was required or claimed. No hardware,
  credentials, network discovery, evidence, parity status, historical
  artifact, schema, or reference content changed. The current explicit
  `work-top-task` invocation supplies commit and push authorization for this
  task only.

### task-separate-host-tool-cores-and-shells | 2026-07-27 21:44 | Separate deterministic host-tool logic from effects

- [x] Inventory active production host-tool modules that combine deterministic
      decisions with filesystem, process, network, clock, or terminal effects
      and record the exact bounded candidate set in this task block before
      editing.
- [x] Start with the known flash-model temporary-file materialization and Phase
      36 classification/filesystem loading mix, keeping pure models and
      classifiers independent from effectful adapters.
- [x] Preserve every existing CLI, schema, evidence, permission, redaction,
      terminal-category, package-admission, and evaluator-identity contract.
- [x] Keep already-separated device-session model/live modules separated and
      add focused source or dependency guards that prevent the identified
      effect imports from returning to pure modules.
- [x] Run focused flash, parity, device-session, HTTP-transport, and xtask tests
      as applicable, followed by the repository's complete required software
      verification sequence.

Dependencies: Coordinate source-guard and evaluator-inventory edits with the
reusable-crate concurrency-shell relocation task; neither task may weaken the
other's assertions.

Working inventory (2026-07-28):

- The bounded audit covered active production Rust modules below
  `tools/{flash,parity,device-session,http-transport,xtask}/src`. Modules
  explicitly named or structured as `main`, `commands`, `environment`,
  `filesystem`, `live`, `hardware`, transport, evidence, or package adapters
  remain imperative shells; their deterministic collaborators are already
  separated or their remaining decisions are adapter-local.
- Exact candidate 1 is `tools/flash/src/model.rs`: it mixes reusable command,
  image, and capture models with `NamedTempFile` materialization and
  `TempDir` ownership. Move those resource owners to the existing flash shell
  boundaries while keeping the pure model free of filesystem and `tempfile`
  dependencies.
- Exact candidate 2 is
  `tools/parity/src/phase36_evidence/classification.rs`: it mixes the pure
  envelope classifier with protected-root opening, immutable artifact
  authentication, and unchanged-file verification. Move loading and
  authentication orchestration to a dedicated adapter while keeping the
  classifier source-bound in the evaluator inventory.
- `tools/device-session/src/model.rs` and `model/state.rs` remain the qualified
  pure replay/state core; `live.rs` and its platform modules remain the
  effectful adapters. This task will add guards for the two identified
  candidates without changing that existing split.

Verification:

- All 167 focused flash tests passed. All 11 focused Phase 34 package-admission
  source guards passed, including private durable snapshot ownership and the
  new pure-model effect exclusion. The focused Phase 36 evaluator inventory
  guard passed with the pure classifier and loading adapter independently
  checked.
- `bazel test //tools/flash:tests //tools/parity:tests` passed. The required
  ordered `cargo fmt --all`,
  `cargo clippy --all-targets --all-features -- -D warnings`,
  `cargo build --all-targets --all-features`, and
  `cargo test --all-features` sequence passed, covering device-session,
  HTTP-transport, and xtask as well as the changed packages.
- `bun scripts/bright-builds-check.ts all` scanned 567 tracked files and
  reported zero findings with the eight existing justified exceptions.
  `just test` passed all 76 Bazel test targets, including the ESP32-S3 firmware
  build and package graph.
- `just parity` reported `validation_errors: none`; one aggregate invocation
  encountered transient host resource pressure after the test graph completed,
  and the isolated retry passed without repository changes.
  `just verify-reference` confirmed pinned commit
  `c1915b0a63bfabebdb95a515cedfee05146c1d50` clean, and
  `just verify-redaction` passed.
- The source-derived Phase 36 evaluator identity is
  `5f933db0fadcc959f3a39fc9608d8c9e3a84c3c4c6adb073f25d135e16cae865`.
  Exact inventory membership plus classifier and loading-source drift tests
  passed; no identity was hard-coded and no historical evidence or parity
  status changed. `git diff --check` passed.

Completion review:

- `tools/flash/src/model.rs` now contains reusable data models only. Private
  execution-snapshot ownership, prepared-package resource ownership, and NVS
  temporary-directory ownership live in the corresponding flash shell modules.
- Phase 36 envelope classification is independent of protected filesystem
  access. A dedicated loading adapter owns protected-root admission, immutable
  artifact authentication, and unchanged-file verification, and remains bound
  into the evaluator identity and Bazel runfiles graph.
- Existing CLI, schema, permission, redaction, terminal-category,
  package-admission, evidence, and device-session model/live contracts remain
  unchanged. Residual risk is limited to runtime scheduling not exercised by
  this software-only refactor; no hardware, credentials, network discovery,
  evidence generation or promotion, parity-status change, or historical
  artifact rewrite occurred. The current explicit `work-top-task` invocation
  supplies commit and push authorization for this task only.

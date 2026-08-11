# Archived Tasks

This append-only file preserves finalized task records moved from `TASKS.md`.
It is historical evidence, not an active tracker or an automatic task-selection
source. Use targeted stable-ID lookup only when historical context is required.
Follow-up work must use a new active task ID that references the archived task.

## Finalized

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
### task-archive-finalized-work-immediately | 2026-07-28 19:50 | Archive finalized tasks immediately

- [x] Create the append-only historical task archive and migrate every existing
      finalized task block without changing its contents.
- [x] Define immediate finalization, archive immutability, stable-ID uniqueness,
      and linked follow-up rules in the active tracker and repository guidance.
- [x] Update `work-top-task` and its UI metadata to archive finalized records
      universally without loading archives during automatic selection.
- [x] Repair active documentation breadcrumbs and validate migration,
      selection, verification, and Git delivery contracts.

Verification:

- A structural migration check found all 24 pre-existing finalized IDs exactly
  once in `TASKS.archive.md`, found none in `TASKS.md`, and compared every
  archived block byte-for-byte with its `HEAD` source block.
- Stable IDs were unique across both files, the archive contained no unchecked
  work, the implementation and future tasks remained active during
  verification, both active policy sections remained present, and the active
  tracker shrank from 104,782 bytes to 4,773 bytes.
- The skill passed `quick_validate.py` through `uv run --with pyyaml`; its
  `agents/openai.yaml` parsed with the required display name, description
  length, `$work-top-task` prompt, and archive wording. Static contract checks
  covered archive-path resolution, automatic-selection exclusion, exact-ID
  ineligibility, linked follow-up IDs, unresolved-task retention, ambiguous
  boundary failure, and future-only queue exhaustion.
- `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`,
  `cargo build --all-targets --all-features`, and
  `cargo test --all-features` passed in order.
- `bun scripts/bright-builds-check.ts all` reported zero findings;
  `just test` passed all 76 Bazel tests; `just parity` reported
  `validation_errors: none`; `just verify-reference` reported clean reference
  commit `c1915b0a63bfabebdb95a515cedfee05146c1d50`; and
  `just verify-redaction` passed.
- `mdformat --check` retained the existing nonzero `AGENTS.md` baseline,
  improved `TASKS.md` to clean, kept
  `docs/hardware/esp-device-session.md` clean, and matched the archive's
  inherited task-content baseline; `git diff --check` passed.

Completion review: Finalized task history now lives in one append-only archive,
while the active tracker retains only unresolved work and active policy. The
global `work-top-task` skill resolves a repository override or adjacent
`.archive` path, never reads archives for automatic selection, archives only
the selected verified final record, and requires a new linked ID for follow-up.
The historical durability breadcrumb now targets the archived record without
authorizing reuse.

Residual risks: The archive intentionally grows without rotation, relying on
targeted stable-ID lookup to avoid context cost. The global skill remains
outside repository Git and must be distributed separately. No hardware,
credentials, network discovery, evidence generation or promotion, direct UART
or pin work, or future cross-platform adapter work was performed.

### task-production-mining-hardware-lifecycle | 2026-07-28 | Deepen the production mining hardware lifecycle

- [x] Add typed hardware preparation, readiness, safe-stop, and bounded
      campaign-lease states to the single deep Production Mining Session
      interface.
- [x] Replace externally supplied `production_asic_ready` truth with
      session-owned hardware state; keep the ordinary ESP and deterministic
      adapters as the two adapters at this seam.
- [x] Add validated `MiningHardwareProfile` and one-shot
      `MiningCampaignLease` types, including `FirstSubmitResponse` and
      `ActiveDuration` stop conditions.
- [x] Gate pool-secret reads on operator intent, network readiness, Stratum V1,
      fresh safety observations, a valid campaign lease, qualified actuation,
      and successful hardware preparation.
- [x] Extend `ProductionSessionEvent` and `ProductionSessionEffect` with
      prepare, prepared/failed, safe-stop, and stop-confirmation behavior, and
      carry pool generation plus valid-job context through ASIC effects.
- [x] Enforce and test safe-stop order: block submissions, invalidate work and
      generations, stop ASIC interaction, close transports, perform hardware
      safe-stop, then publish the terminal snapshot.
- [x] Update ADR-0016 and ADR-0017 without restoring any retired phase runtime
      or introducing another mining owner.

Dependencies: None.

Verification:

- The required ordered `cargo fmt --all`,
  `cargo clippy --all-targets --all-features -- -D warnings`,
  `cargo build --all-targets --all-features`, and
  `cargo test --all-features` sequence passed on the final source.
- All 25 focused Production Mining Session tests passed. They cover every
  readiness gate, hardware preparation success and failure, lazy
  pool-configuration access, profile and lease validation, active-duration
  timing, first accepted/rejected response stop, generation/job context, and
  terminal publication only after hardware stop confirmation.
- `just verify-production-session` passed its focused Bazel tests, source
  guards, and ESP32-S3 firmware build. `just test` passed all 76 Bazel test
  targets, and `just package` produced the complete Ultra 205 firmware-image
  artifact set.
- `bun scripts/bright-builds-check.ts all` scanned 567 tracked files and
  reported zero findings with the eight existing justified exceptions.
  `just parity` reported `validation_errors: none`; one aggregate invocation
  encountered transient host resource pressure after report generation, and
  the isolated read-only retry passed without repository changes.
- `just verify-reference` confirmed pinned reference commit
  `c1915b0a63bfabebdb95a515cedfee05146c1d50` clean.
  `just verify-redaction`, the production-session source scans, and
  `git diff --check` passed. Final diff review found no retired runtime, new
  mining owner, credential, network, hardware, evidence, or parity-promotion
  path.

Completion review:

- The single Production Mining Session now owns typed preparing, ready,
  safe-stopping, stopped, armed, active, and consumed campaign state. A
  validated monotonic lease cannot be reused, `FirstSubmitResponse` is bounded
  from preparation, and `ActiveDuration` begins only after authorization makes
  mining active.
- Pool configuration remains lazy until every external readiness gate and the
  matching hardware-prepared event succeed. ASIC dispatch and poll effects
  carry the active pool generation and valid-job set.
- Terminal stop retains the correlated accepted/rejected counters, blocks and
  invalidates submissions, stops ASIC interaction, closes owned transports,
  requests hardware safe-stop, and withholds terminal publication until the
  matching confirmation. Partial preparation and unavailable/exhausted pools
  take the same fail-closed path.
- The ordinary ESP adapter still has no actuation authority or campaign lease;
  the deterministic adapter remains the only other adapter. Residual risk is
  the intentionally unimplemented qualified hardware and live-I/O adapters
  owned by dependency-gated follow-up tasks. No pool credentials, network
  connection, hardware actuation, flash, evidence promotion, parity change,
  direct UART, or pin manipulation occurred.

### task-ultra205-safety-observation-completeness | 2026-07-28 | Complete fail-closed Ultra 205 sensor truth

- [x] Read EMC2101 internal temperature as the Ultra 205 VR-temperature
      observation while preserving external ASIC temperature and tachometer
      acquisition under the sole shared-I2C owner.
- [x] Require fresh INA260 voltage, current, and power; ASIC temperature; VR
      temperature; and fan RPM before hardware preparation or work dispatch.
- [x] Preserve the existing fail-closed limits: 4.5-5.5 V input, 15 W maximum,
      ASIC temperature below 75 C, one-second freshness, and explicit
      unavailable, stale, invalid, and read-failed states.
- [x] Project the new VR-temperature truth through runtime, HTTP, and WebSocket
      views without fabricating unavailable samples or weakening source
      ownership.
- [x] Add acquisition, decoding, staleness, projection, failure-retention, and
      source-ownership tests.

Dependencies: None.

Verification:

- The required ordered `cargo fmt --all`,
  `cargo clippy --all-targets --all-features -- -D warnings`,
  `cargo build --all-targets --all-features`, and
  `cargo test --all-features` checks passed. The full test rerun completed
  cleanly after a transient macOS policy-service startup delay.
- Focused Cargo tests passed 217 API and 62 safety tests. Focused Bazel tests
  passed for the API, safety core, EMC2101 adapter acquisition, and sensor
  source-ownership targets.
- `just verify-production-session` passed its focused tests, source guard, and
  ESP32-S3 build. `just test` passed all 78 Bazel test targets, and
  `just package` produced the complete Ultra 205 firmware-image artifact set.
- `bun scripts/bright-builds-check.ts all` scanned 568 tracked files and
  reported zero findings with eight existing justified exceptions.
  `just parity` reported `validation_errors: none`; `just verify-reference`
  confirmed pinned reference commit
  `c1915b0a63bfabebdb95a515cedfee05146c1d50` clean; and
  `just verify-redaction` passed.
- Source scans confirmed that the sole operator sensor producer makes each
  acquisition call exactly once, raw sensor-bus capability stays inside the
  safety facade, and the diff adds no actuation, socket, or credential path.
  `git diff --check` and final diff review passed.

Completion review:

- The sole shared-I2C owner now acquires EMC2101 register `0x00` as an
  independently stamped VR-temperature fact while preserving separate external
  ASIC-temperature and tachometer truth. Read, decode, and retention failures
  remain source-local and fail closed.
- Mining readiness, hardware preparation, and ASIC dispatch now require all
  six observations to be fresh at the current monotonic time. The gate
  preserves 4.5-5.5 V, 15 W maximum, ASIC temperature below 75 C, finite
  numeric values, and an exact one-second sample-age boundary.
- Runtime, HTTP, and WebSocket projections carry the VR fact and its
  unavailable, stale, fault, or fresh state without advancing producer
  metadata. The simplification pass retained one pure readiness predicate and
  one ordinary firmware adapter recheck rather than duplicating policy.
- Residual risk is intentionally deferred hardware validation: this task
  performed no fan or voltage write, ASIC action, credential read, network
  connection, device run, evidence promotion, direct UART, or pin
  manipulation. The ordinary adapter remains actuation-unqualified until the
  dependency-gated mining adapter task completes.

### task-ultra205-mining-actuation-adapter | 2026-07-28 | Implement the qualified Ultra 205 mining adapter

- [x] Add a typed command channel to the sole shared-I2C owner for EMC2101 fan
      and DS4432U voltage effects without exposing internal I2C seams through
      the Production Mining Session interface.
- [x] Retain Ultra 205 GPIO10 ASIC-enable, reset, and UART ownership in the
      ordinary ESP adapter; keep every non-205 target fail-closed.
- [x] Define validated `conservative` (400 MHz, 1100 mV, 100% fan) and
      `upstream-default` (485 MHz, 1200 mV, 100% fan) profiles.
- [x] Implement preparation ordering: fresh observations, 100% fan and RPM
      proof, voltage, stabilization, ASIC enable/reset, exactly-one-chip
      detection, upstream-aligned mining-ready initialization with frequency
      ramp, and production UART retention.
- [x] Implement a distinct safe-shutdown plan: stop dispatch, frequency-down
      and nonce reset, hold reset low, core voltage off, ASIC enable off, and
      fan 100% until a fresh temperature is at or below 45 C before reducing
      to the paused 30% duty.
- [x] Roll every partial-preparation failure through the same safe-stop
      implementation while preserving the earliest typed failure.
- [x] Add golden frame/order tests, shared-I2C command tests, failure-injection
      rollback tests, and source guards proving raw device primitives remain
      inside firmware adapters.

Dependencies: Complete
`task-production-mining-hardware-lifecycle` and
`task-ultra205-safety-observation-completeness`.

Verification:

- The required ordered `cargo fmt --all`,
  `cargo clippy --all-targets --all-features -- -D warnings`,
  `cargo build --all-targets --all-features`, and
  `cargo test --all-features` checks passed.
- Focused Bazel tests passed for the ASIC, Stratum, EMC2101 acquisition,
  DS4432U actuation, sensor source-ownership, and mining-actuation targets.
- `just verify-production-session` passed its focused tests, source guard, and
  ESP32-S3 release build. `just test` passed all 80 Bazel test targets, and
  `just package` produced the complete Ultra 205 firmware-image artifact set.
- `bun scripts/bright-builds-check.ts all` reported zero findings.
  `just parity` reported `validation_errors: none`; `just verify-reference`
  confirmed pinned reference commit
  `c1915b0a63bfabebdb95a515cedfee05146c1d50` clean; and staged redaction
  verification passed.
- Source scans confirmed that raw shared-I2C primitives stay inside the
  firmware safety adapter, production admission remains
  `actuation_qualified: false` with no campaign lease, and no hardware command
  ran. `git diff --check` and final staged-diff review passed.

Completion review:

- The ordinary Ultra 205 adapter now owns a bounded typed fan/voltage command
  channel, active-low GPIO10 ASIC enable, reset, and retained UART. Non-205
  targets and unavailable peripherals remain fail closed without publishing a
  production actuation handle.
- Closed conservative and upstream-default profiles drive an exact ordered
  preparation plan with fresh safety and post-command RPM proof, voltage
  stabilization, one-chip detection, and a quarter-MHz mining-ready ramp.
- Every partial-preparation failure invokes the same idempotent safe-stop
  sequence. Safe-stop attempts every cleanup action, preserves the earliest
  typed failure, and requires fresh cooling proof at or below 45 C before
  reducing the fan from 100% to the paused 30% duty.
- The simplification pass retained policy and ordering in a pure actuation
  core while keeping ESP-IDF I2C, GPIO, timing, and UART effects in the
  firmware adapter.
- This establishes software status only. No hardware actuation, credential
  read, network connection, flash, evidence promotion, direct UART, or pin
  manipulation occurred. Production admission remains disabled until the
  dependency-gated live-I/O task supplies and validates the campaign lease;
  hardware verification remains residual risk.

### task-production-mining-live-io | 2026-07-28 | Qualify live Stratum and bounded campaign I/O

- [x] Change the firmware owner inbox to accept category wakeups and typed
      transport/ASIC events while leaving all lifecycle policy inside the
      Production Mining Session.
- [x] Implement bounded per-pool TCP workers behind the existing
      primary/fallback interface so the mining owner never blocks on socket
      connect, read, or write.
- [x] Add a dedicated lazy NVS pool reader with redacted debug behavior and no
      secret-bearing logs, reads, or projections before the session requests
      pool configuration.
- [x] Carry pool generation and valid job IDs through ASIC dispatch and poll
      effects so invalidated or stale results cannot produce a submission.
- [x] Replace the source guard's hard-coded `actuation_qualified: false`
      assertion with guards proving raw sockets, secrets, clocks, and device
      primitives remain outside the deep session owner.
- [x] Add the repo-owned `just mining-campaign` command with typed
      `observation`, `live-share`, and `soak` stages by extending existing
      package admission, device-session supervision, NVS injection, redaction,
      and evidence sealing.
- [x] Make the command persist `mineonboot=false` before activation and install
      a one-shot device-local lease: `live-share` stops after the first
      accepted/rejected submit response or 600 seconds, and `soak` stops after
      600 active seconds.
- [x] Add real-process loopback transport tests, deterministic accepted and
      rejected session tests, partial-frame and recovery tests, credential/NVS
      redaction tests, lease-timeout tests, and reboot-remains-paused tests.

Dependencies: Complete
`task-production-mining-hardware-lifecycle` and
`task-ultra205-mining-actuation-adapter`.

Verification:

- The required ordered `cargo fmt --all`,
  `cargo clippy --all-targets --all-features -- -D warnings`,
  `cargo build --all-targets --all-features`, and
  `cargo test --all-features` checks passed after the final source change.
- Focused Bazel tests passed for the production Stratum core, real-process TCP
  transport, campaign status, firmware source ownership, flash campaign
  supervisor, device-session lifecycle, and production-session source guard.
- `just verify-production-session` passed its focused tests, deep ownership
  guard, and ESP32-S3 release build. `just test` passed all 82 Bazel test
  targets, and `just package` produced the complete Ultra 205 image and package
  artifact set.
- `bun scripts/bright-builds-check.ts all` reported zero findings.
  `just parity` reported `validation_errors: none`; `just verify-reference`
  confirmed pinned reference commit
  `c1915b0a63bfabebdb95a515cedfee05146c1d50` clean; and staged
  `just verify-redaction` passed.
- Secret-pattern and ignored-result scans found no new credential values or
  swallowed failures in the production paths. `git diff --check` and the
  final staged-diff review passed.

Completion review:

- The production owner now receives category wakeups plus typed, generation-
  and epoch-stamped transport/ASIC feedback. Independent bounded primary and
  fallback TCP workers perform connect/read/write work without blocking the
  owner, and a non-lossy close signal remains effective when a worker queue is
  saturated.
- Pool credentials stay behind a dedicated lazy NVS reader and redacted
  boundary types. Stale transport epochs, ASIC generations, job IDs, timeout
  feedback, and nonce results are rejected before they can mutate the current
  session or submit a share.
- `just mining-campaign` admits only board 205 with exact stage/profile/duration
  combinations, creates one private evidence attempt beneath a pre-admitted
  parent, seeds one combined private NVS image with `mineonboot=false`, consumes
  a one-shot live lease before actuation, preserves the earliest typed failure,
  and seals only closed redacted evidence categories.
- Observation never reads pool credentials or actuates hardware and runs the
  full 360-second bound on success. Live-share terminates on one correlated
  accepted/rejected response or 600 seconds, soak requires 600 active seconds,
  and both require confirmed safe-stop before terminal success. A consumed
  lease returns operator intent to paused and cannot replay after reboot.
- The simplification pass kept lifecycle policy, deadlines, generations, and
  share classification in the pure Production Mining Session; raw TCP, NVS,
  clocks, ESP device primitives, and campaign evidence effects remain in thin
  boundary modules.
- This completion is software evidence only. No owner-supplied pool or Wi-Fi
  credential file was read, no external pool connection or hardware command
  ran, and no parity claim was promoted. The only socket activity was the
  isolated host loopback test. Detector-gated Ultra 205 observation, live-share,
  and soak evidence remain the residual hardware validation.

### task-ultra205-mining-observation-baseline | 2026-07-28 | Re-establish a known-safe mining observation baseline

Status: Complete — `attempt-005` accepted the corrected Ultra 205 observation
contract from the exact clean-HEAD package.

- [x] Reproduce the zero-marker `marker_invalid` boundary with non-UTF-8
      non-candidate bytes surrounding valid runtime attestations and campaign
      markers.
- [x] Replace whole-stream UTF-8 conversion with incremental LF framing,
      candidate-only decoding, independent runtime-attestation assessment, and
      earliest-failure preservation.
- [x] Add sealed `mining-campaign-result-v2` and private
      `mining-campaign-serial-diagnostics-v1` evidence with bounded typed
      events, aggregate counts, and no raw serial or candidate content.
- [x] Reproduce the `attempt-002` false-to-true boot-preference loss at the
      pure reload boundary and make persistence load both upstream and
      project-owned settings schemas.
- [x] Add a closed per-source freshness projection so a five-of-six safety
      result names the unavailable observation without sensor values or raw
      serial.
- [x] Prove from sealed hardware evidence and the pinned board-205 reference
      that EMC2101 internal temperature is the supported ASIC-temperature
      source and the DS4432U path has no VR-temperature sensor.
- [x] Pass every required software, package, parity, reference, and redaction
      gate for the parser and diagnostics change before committing it.
- [x] Build and admit the exact current-HEAD package after its software
      dependencies complete.
- [x] Detect exactly one Ultra 205 and run the single authorized observation
      attempt.
- [x] Prove exact source/package runtime attestation, all five supported
      Ultra 205 safety observations fresh, VR temperature explicitly
      unsupported, `mineonboot=false`, no campaign lease, no pool-secret read,
      and no fan, voltage, or ASIC actuation.
- [x] Seal the private result with one accepted terminal outcome and preserve
      exact non-claims for mining, shares, soak, and parity promotion.

Dependencies: Complete
`task-ultra205-safety-observation-completeness` and
`task-production-mining-live-io`.

Hardware contract:

- Permitted commands:
  1. `just detect-ultra205`
  2. `just package`
  3. `just mining-campaign stage=observation board=205 port=<detector-port> manifest=bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json wifi-credentials=wifi-credentials.json evidence-dir=scratch/ultra205-mining-observation-baseline/attempt-002 duration-seconds=360 redact-evidence=true`
  4. `just mining-campaign stage=observation board=205 port=<detector-port> manifest=bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json wifi-credentials=wifi-credentials.json evidence-dir=scratch/ultra205-mining-observation-baseline/attempt-003 duration-seconds=360 redact-evidence=true`
  5. `just mining-campaign stage=observation board=205 port=<detector-port> manifest=bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json wifi-credentials=wifi-credentials.json evidence-dir=scratch/ultra205-mining-observation-baseline/attempt-004 duration-seconds=360 redact-evidence=true`
  6. `just mining-campaign stage=observation board=205 port=<detector-port> manifest=bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json wifi-credentials=wifi-credentials.json evidence-dir=scratch/ultra205-mining-observation-baseline/attempt-005 duration-seconds=360 redact-evidence=true`
- Objective: establish a fresh current-architecture, exact-package,
  observe-only baseline on the single detected board 205 without reopening or
  retrying the terminal Phase 36 lineage.
- Evidence: preserve the sealed ignored `attempt-001` root unchanged. The new
  ignored `scratch/ultra205-mining-observation-baseline/attempt-002` root is
  private, non-promoted `ProtectedOperational` evidence. Its parent is mode
  0700 and artifacts are mode 0600. Only redacted closed categories, bounded
  counts, durations, and safe build provenance may be summarized. The new
  typed diagnostic trace must never contain raw serial bytes, candidate
  payloads, excerpts, identifiers, credentials, endpoints, or secret-derived
  hashes.
- Diagnostic contract: `campaign-diagnostics.private.json` is mode 0600 and
  records only aggregate byte, line, candidate, accepted-marker, encoding,
  JSON, schema, and trailing-partial counts plus the first and last 32 typed
  events. `campaign-result.json` uses `mining-campaign-result-v2`, binds the
  diagnostic artifact digest, keeps `marker_invalid` compatible, records one
  closed `serial_outcome_detail`, and records runtime attestation independently.
- Preconditions: all dependency and software gates pass; the source tree and
  reference are clean; `just package` freezes an exact current-HEAD manifest;
  the detector admits exactly one ESP32-S3 board 205; the local Wi-Fi
  credential file is present but never printed; no pool credential is read or
  supplied. The completed current-architecture dependencies are required new
  diagnostic information relative to the archived Phase 36 boundary.
- Allowed effects: write and verify only the exact admitted factory image,
  inject the local Wi-Fi configuration through the existing private NVS path,
  persist `mineonboot=false`, perform repo-owned reset/re-enumeration,
  receive-only serial observation, use a fresh same-session origin-only device
  URL when uniquely available, and clean up supervisor-owned child processes.
- Prohibited effects: pool configuration, pool network connections, mining
  lease creation, fan/voltage/ASIC actuation, erase-flash, arbitrary raw
  writes, OTA, recovery upload, network discovery, foreign-process
  termination, evidence promotion, direct UART, pins, pads, headers, GPIO,
  probes, jumpers, soldering, or injected signals.
- Recovery/restoration: terminate and reap supervisor-owned process groups,
  release serial descriptors, persist `mineonboot=false`, leave no campaign
  lease, and prove the admitted device accessible and holder-free. Success
  leaves the exact admitted package installed. Identity drift, device absence,
  a foreign holder, or unproved cleanup stops without physical intervention.
- Retry bound: the user authorized sequential hardware retries after changes
  until the task completes. `attempt-003` is authorized only after a
  deterministic regression proves and a clean committed/pushed fix repairs the
  `mineonboot=false` state boundary exposed by `attempt-002`. Never run an
  unchanged retry. `attempt-004` is authorized only after the v2 per-source
  freshness diagnostic is regression-backed, fully verified, committed,
  pushed, and rebuilt from clean exact HEAD. `attempt-005` is authorized only
  after the board-205 temperature source and capability correction is
  regression-backed, fully verified, committed, pushed, and rebuilt from clean
  exact HEAD. After any later failure, diagnose its closed boundary,
  verify one targeted fix or objective non-invasive boundary change, and amend
  this contract with the exact next ordinal and command before hardware use.
  A recurrence of the same authoritative boundary signature after its targeted
  verified fix selects `stop_repeated_boundary`. Observation completion also
  authorizes the existing conservative live-share task under its own
  change-gated retry contract.
- Accepted terminal outcomes: `complete`, `stop_repeated_boundary`,
  `stop_hardware_blocker`, `stop_authority_boundary`, or
  `stop_impossible_contract`. Preserve the earliest typed failure.

Verification: In progress under the newly authorized parser diagnosis and
`attempt-002` contract. The deterministic pre-fix regression sealed
`marker_invalid` with zero markers when otherwise valid observation input was
surrounded by non-UTF-8 noise. The byte-safe implementation passes the focused
194-test `bitaxe-flash` suite, the ordered Rust format/Clippy/build/test gates,
`just verify-production-session`, all 82 Bazel tests, `just package`, Bright
Builds checks, parity with no validation errors, reference cleanliness, and
redaction verification. All prior software, package, parity, reference,
redaction, and clean exact-HEAD gates passed at
`a6cc0a20`. The initial detector run preserved `recovery_not_observed` at final
cleanup: the same device was accessible and holder-free, but the 30-second
window admitted only two of three required stable samples. The targeted
60-second final-cleanup fix and its slow-sampler regression test passed and
were committed before the second detector run. That detector admitted exactly
one device and completed cleanup. The 360-second campaign then sealed
`marker_invalid` with zero accepted markers, runtime identity not trusted,
package admission true, and USB cleanup ready. Private attempt evidence
permissions and the result seal passed.

`attempt-002` admitted the clean exact-HEAD package at `44a85c4d` and exactly
one Ultra 205, then stopped on the first accepted observation marker before the
360-second window completed. The sealed v2 result records
`mineonboot_enabled`, package admission true, runtime attestation missing, one
accepted marker, `mineonboot=true`, safety stale with five of six fresh
observations, no pool read, no actuation, and USB cleanup ready. The sealed
serial diagnostic records clean framing, no invalid bytes or malformed
candidates, and `serial_outcome_detail=clean`. This disproves a repeated parser
failure for the new attempt, does not identify the exact historical byte-level
trigger, and selects the no-retry stop required by the authorized contract.
The host stop predicate also needed a follow-up regression-backed correction
so observation contract failures retain the full diagnostic window; that
software correction does not authorize another hardware ordinal.

For `attempt-003`, the exact host regression
`cargo test -p bitaxe-config persistence_reload_preserves_project_boot_preference`
failed before the fix because stored `mineonboot=0` reloaded as no typed value,
which made firmware callers use their fail-safe `true` fallback. The same test
passes after `load_values` chains the deliberately separate project-owned
schema, and all 48 `bitaxe-config` plus all 195 focused `bitaxe-flash` tests
pass. The ordered Rust format, warnings-denied Clippy, all-target build, and
all-feature test gates pass; `just verify-production-session`, all 82 Bazel
tests, `just package`, Bright Builds checks, parity validation with no errors,
reference cleanliness, and redaction verification also pass. Clean-HEAD
commit `5cd7ff02`, push, exact-HEAD rebuild, and `attempt-003` hardware
verification also pass.

`attempt-003` used the clean exact-HEAD package at `5cd7ff02` and completed the
full 360-second window. Its sealed result and bound diagnostics pass mode,
digest, and result-seal checks. They record exact-package runtime identity
trusted, `mineonboot=false`, 719 accepted markers, clean serial framing, no pool
read, no actuation, safe-stop not required, and USB cleanup ready. The distinct
terminal boundary is `safety_stale`: every marker reports exactly five of six
fresh observations. The aggregate marker does not identify the missing source,
so the firmware/host marker contract is being advanced to
`mining-campaign-status-v2` with six closed Boolean freshness fields and a
count-consistency check. The firmware marker test failed red against v1, then
passed with the v2 projection. All 196 focused `bitaxe-flash` tests, the
ordered Rust format, warnings-denied Clippy, all-target build, all-feature
tests, production-session verification, all 82 Bazel tests, packaging, Bright
Builds checks, parity validation with no errors, reference cleanliness, and
redaction verification pass for the v2 diagnostic change.

`attempt-004` used clean exact-HEAD commit `2d6a8e73` and completed the full
360-second window. Its sealed result and bound diagnostics pass mode, digest,
and result-seal checks. All 719 accepted markers agree that only
`chip_temp_celsius` is stale while power, bus voltage, current, VR temperature,
and fan tach are fresh. Package identity and runtime attestation are trusted,
`mineonboot=false`, serial framing is clean, no pool was read, no actuation
occurred, and USB cleanup is ready. Comparison with pinned board-205 reference
configuration and thermal/power selection proves the Rust mapping was
backwards: Ultra 205 sets `emc_internal_temp=true`, so EMC2101 internal
temperature is ASIC temperature, while its DS4432U power path exposes no
VR-temperature source. The pre-fix readiness regression rejects an otherwise
safe Ultra 205 when VR temperature is explicitly unavailable, and the pre-fix
adapter regression fails because the board-specific internal-temperature
acquisition does not exist. Both regressions pass after mapping ASIC
temperature to EMC2101 internal, representing the unsupported VR source
without a fabricated stamp, requiring the other five independent facts, and
binding that exact requirement set in `mining-campaign-status-v3`. The host
rejects a contradictory requirement set or freshness count.

All 63 `bitaxe-safety`, 217 `bitaxe-api`, and 197 `bitaxe-flash` tests pass.
The ordered Rust format, warnings-denied Clippy, all-target build, all-feature
tests, production-session verification, all 82 Bazel tests, packaging, Bright
Builds checks, parity validation with no errors, reference cleanliness,
redaction verification, and diff checks also pass. A parity mutation test
exposed a same-process temporary-path collision at nanosecond clock
resolution; its atomic fixture suffix regression passes 20 consecutive runs
and the full parity suite passes afterward.

`attempt-005` used clean exact-HEAD commit `97385614` and completed the full
360-second window. Its sealed result is accepted as `observation_complete` with
trusted package and runtime identity, 719 accepted markers, clean serial
framing, `mineonboot=false`, and the exact Ultra 205 requirement mask. Power,
bus voltage, current, internal chip temperature, and fan tach are fresh; VR
temperature is false in both the freshness and requirement masks because the
board has no such sensor. No pool was read, no campaign lease or hardware
actuation occurred, safe-stop is not required, USB cleanup is ready, all
artifacts have the required private modes, both bound artifact digests match,
and the result seal verifies.

Completion review: The deterministic host parser, persistence reload, and
board-capability bugs exposed by the observation lineage are fixed with red
regressions and clean-HEAD hardware proof. The accepted baseline authorizes
the separate conservative live-share task under its own effects, safety,
evidence, and retry contract. It makes no mining, share, soak, release,
parity-promotion, profitability, or long-duration stability claim. No pool
credential was supplied to any observation attempt, and no direct UART,
pin work, raw serial persistence, network discovery, or evidence promotion
occurred.

### task-ultra205-live-pool-share | 2026-07-28 | Prove one real BM1366 pool submission

Status: Complete — `attempt-005` accepted the conservative live-share campaign
after the bounded chip-detection and typed preparation-diagnostic fix. It
proved owner-pool work, BM1366 nonce correlation, a real rejected submit
response, fresh supported safety, trusted exact-package identity, confirmed
safe stop, and USB cleanup.

- [x] Freeze and admit the exact current-HEAD package, single detected board
      205, ignored local Wi-Fi credentials, and exactly one ignored local pool
      credential file.
- [x] Run the `conservative` profile at 400 MHz, 1100 mV, and 100% fan until
      the first accepted/rejected submit response or the 600-second lease
      expires.
- [x] Prove pool authorization, notify-derived work, BM1366 dispatch, parsed
      nonce, matching generation/job correlation, share submission, and an
      accepted or rejected pool response.
- [x] Confirm device-local safe-stop, lease removal, persisted
      `mineonboot=false`, retained owner-supplied pool configuration, and the
      new firmware paused after the attempt.
- [x] Seal one private, redacted result without automatic parity promotion.

Dependencies: Complete tasks
`task-production-mining-hardware-lifecycle`,
`task-ultra205-safety-observation-completeness`,
`task-ultra205-mining-actuation-adapter`,
`task-production-mining-live-io`, and
`task-ultra205-mining-observation-baseline`.

Hardware contract:

- Permitted commands:
  1. `just detect-ultra205`
  2. `just package`
  3. `just mining-campaign stage=live-share profile=conservative board=205 port=<detector-port> manifest=bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json wifi-credentials=wifi-credentials.json pool-credentials=<single-ignored-local-pool-file> evidence-dir=scratch/ultra205-live-pool-share/attempt-001 duration-seconds=600 redact-evidence=true`
  4. `just mining-campaign stage=live-share profile=conservative board=205 port=<detector-port> manifest=bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json wifi-credentials=wifi-credentials.json pool-credentials=<single-ignored-local-pool-file> evidence-dir=scratch/ultra205-live-pool-share/attempt-002 duration-seconds=600 redact-evidence=true`
  5. `just mining-campaign stage=live-share profile=conservative board=205 port=<detector-port> manifest=bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json wifi-credentials=wifi-credentials.json pool-credentials=<single-ignored-local-pool-file> evidence-dir=scratch/ultra205-live-pool-share/attempt-003 duration-seconds=600 redact-evidence=true`
  6. `just mining-campaign stage=live-share profile=conservative board=205 port=<detector-port> manifest=bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json wifi-credentials=wifi-credentials.json pool-credentials=<single-ignored-local-pool-file> evidence-dir=scratch/ultra205-live-pool-share/attempt-004 duration-seconds=600 redact-evidence=true`
  7. `just mining-campaign stage=live-share profile=conservative board=205 port=<detector-port> manifest=bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json wifi-credentials=wifi-credentials.json pool-credentials=<single-ignored-local-pool-file> evidence-dir=scratch/ultra205-live-pool-share/attempt-005 duration-seconds=600 redact-evidence=true`
- Objective: obtain one real BM1366 nonce correlated to owner-pool work and
  one accepted or rejected Stratum V1 submit response under the conservative
  profile, then prove safe stop.
- Evidence: each ignored `scratch/ultra205-live-pool-share/attempt-<ordinal>`
  root is private, non-promoted `ProtectedOperational` evidence with mode-0700
  parent and mode-0600 artifacts. Committed or summarized output may record
  `pool_config: local-owner-supplied`, closed result categories, bounded
  counts, durations, and safe provenance only. It must never contain raw pool
  URL, port, user, worker, owner address, password, endpoint, token, NVS secret,
  Wi-Fi value, device URL, IP, MAC, or unredacted logs.
- Preconditions: all five dependencies and software gates pass; exact
  current-HEAD package identity is frozen; detector admission finds exactly
  one board 205; the prior observation baseline completed; exactly one ignored
  pool credential file and the ignored Wi-Fi credential file exist; the pool
  declares Stratum V1 over ordinary TCP; TLS is rejected as out of scope.
- Allowed effects: private NVS injection of Wi-Fi and owner pool settings,
  persistence of `mineonboot=false`, installation of one conservative
  campaign lease, exact package flash, repo-owned USB reset/re-enumeration,
  fan 100%, DS4432U 1100 mV, ASIC enable/reset, BM1366 initialization and
  work/result traffic, Stratum V1 TCP connection and submission, fresh-session
  HTTP/WebSocket observation, and bounded device-local safe-stop.
- Safety and stop limits: observations must remain fresh; input must remain
  4.5-5.5 V; power must not exceed 15 W; ASIC temperature must remain below
  75 C; fan RPM must remain fresh and nonzero after the qualified 100% command.
  Any sensor, watchdog, actuation, generation, lease, transport, or evidence
  fault blocks submissions and begins safe-stop immediately.
- Prohibited effects: TLS, Stratum V2, automatic fan mode, unbounded mining,
  non-205 hardware, erase-flash, arbitrary raw writes, OTA, recovery upload,
  network discovery, foreign-process termination, raw secret output, parity
  promotion, direct UART, pins, pads, headers, GPIO, probes, jumpers,
  soldering, injected signals, voltage/fan stress, or fault injection.
- Recovery/restoration: preserve the earliest typed failure; block and
  invalidate submissions; close owned pool transports; frequency-down and
  reset the ASIC; set core voltage off and ASIC enable off; keep fan at 100%
  until fresh temperature is at or below 45 C, then set 30%; clear the lease;
  persist `mineonboot=false`; retain pool settings; and release owned USB and
  process resources. If device-local stop cannot be confirmed, one
  predeclared exact-baseline reflash is allowed only after same-device
  re-admission; otherwise stop.
- Retry bound: one fresh attempt only. No timeout, no-share, rejected-share,
  transport, ASIC, or safety result authorizes an unchanged retry. The user
  authorized sequential retries after changes until this task completes. A
  later ordinal requires a targeted regression-backed fix or authorized
  non-invasive remediation with objective boundary-change proof plus a task
  amendment naming its exact command and evidence root; one post-fix recurrence
  of the same authoritative boundary signature selects
  `stop_repeated_boundary`. `attempt-002` changes the authoritative evidence
  boundary: the firmware marker and sealed result must preserve the earliest
  closed hardware-preparation phase, step, adapter category, and any secondary
  rollback failure so the misleading `pool_configuration_missing` precedence
  cannot recur. `attempt-003` changes the hardware boundary by restoring the
  pinned EMC2101 tach-input initialization before direct fan mode and duty. It
  is not an unchanged retry. `attempt-004` changes the host evidence boundary:
  after one complete validated live terminal marker, already-buffered suffix
  bytes are outside the campaign and are counted without being parsed as a new
  marker. Malformed or truncated candidates before that boundary and every
  observation-campaign trailing candidate remain fail-closed. `attempt-005`
  changes the firmware preparation boundary: chip-detect draining now has both
  a five-second wall-clock ceiling and a 64-frame ceiling, and every closed
  preparation step emits privacy-safe `started`, `completed`, or `failed`
  progress sealed into the private diagnostic artifact. The historical
  byte-level trigger remains uncertain because raw serial was intentionally
  ephemeral; the new bounds eliminate the identified non-return class without
  overstating that uncertainty.
- Accepted terminal outcomes: `complete` only when every success and safe-stop
  criterion passes; otherwise `stop_repeated_boundary`,
  `stop_hardware_blocker`, `stop_authority_boundary`, or
  `stop_impossible_contract`.

Verification: `attempt-001` used exact clean-HEAD commit `0e84acc5`, one
detected Ultra 205, and exactly one ignored owner pool input. It stopped before
active mining and sealed package/runtime identity trusted, clean serial
framing, five supported observations fresh, `mineonboot=false`, zero active
milliseconds, no submit response, confirmed safe-stop, USB cleanup ready, and
no parity promotion. Its seven markers remain `pool_config=not_read`, move
directly from the pre-session projection to consumed safe-stop, and prove an
earlier preparation failure. Because marker v3 carries no typed preparation
failure, the host incorrectly selected `pool_configuration_missing`. Run the
new red diagnostic-precedence regression, all required software gates, the
exact permitted hardware commands, private-artifact permission checks,
redaction and secret denylist verification, lease/safe-stop validation, sealed
result validation, and final diff review. `attempt-002` used exact clean-HEAD
commit `28d89759`, one detected Ultra 205, and exactly one ignored owner pool
input. It sealed `hardware_preparation` at
`require_fresh_nonzero_fan_rpm` with `fan_rpm_proof_timed_out`, one valid
marker, clean serial framing, five supported observations fresh,
`mineonboot=false`, zero active milliseconds, no pool read or submit, confirmed
safe-stop, and USB cleanup ready. The focused EMC2101 regression failed before
the fix because no configuration-register write existed, then passed after the
adapter wrote `0x03=0x04` before fan mode and duty.
The first `attempt-003` preflight detector invocation stopped before campaign
creation with `recovery_not_observed` at `post_probe`, with
`stable_samples_max=1`. Its same-session mode-0600 final-cleanup trace then
proved `stable_samples_max=3`, same device seen, accessible, holder-free, and
no identity drift or enumeration change. The ignored preflight root is
mode-0700 and retains the mode-0600 protected console record; no mining ordinal
was consumed.
The authorized re-detection then passed, and `attempt-003` used exact
clean-HEAD commit `47edbc90`, one detected Ultra 205, and exactly one ignored
owner pool input. It recorded 2,189 active milliseconds, owner pool
configuration loaded, a real rejected submit response, trusted runtime
attestation and package identity, five supported observations fresh,
`mineonboot=false`, confirmed safe stop, and USB cleanup ready. Fifty-three
markers were accepted. A 101-byte candidate began only after the complete
terminal marker; the old finish path classified that post-terminal suffix as
`marker_truncated` and incorrectly selected `marker_invalid`. The new red
regression reproduces this exact ordering and proves the host accepts the
terminal result while counting, not parsing, post-terminal bytes.
`attempt-004` used exact clean-HEAD commit `c5596a8a`, one detected Ultra 205,
and exactly one ignored owner pool input. It retained seven valid live-share
markers, all with the admitted lease, conservative profile, fresh supported
observations, `mineonboot=false`, zero active milliseconds, no pool read, no
submit response, and no campaign failure. Marker publication then stopped
while periodic trusted runtime attestations continued through the 600-second
capture. The result sealed clean serial framing, trusted identity, USB cleanup
ready, and `pool_configuration_missing`. Because the last marker preceded
preparation, that terminal category is a downstream symptom rather than the
authoritative root boundary. The red chip-detect budget regression failed
before the new ceiling existed, and the typed preparation-progress regression
failed before those events were recognized and retained.
`attempt-005` used exact clean-HEAD commit `9861f4c4`, one detected Ultra 205,
ignored local Wi-Fi input, and exactly one ignored owner pool input. Its sealed
v2 result is accepted as `submit_response_observed`: 77 markers, 4,713 active
milliseconds, owner pool configuration loaded, a real rejected submit
response, all five supported Ultra 205 safety observations fresh,
`mineonboot=false`, trusted package and runtime identity, confirmed safe stop,
and USB cleanup ready. Serial framing was clean. All 18 typed preparation
events were valid and the last closed boundary was
`retain_production_uart/completed`; no preparation, marker, encoding, JSON,
schema, or trailing-partial failure occurred. The private root is mode 0700,
all four artifacts are mode 0600, the result and both bound artifact digests
verify, the credential and identifier denylist is clear, and parity promotion
is false.

Completion review: The bounded preparation fix eliminates the identified
continuous chip-detect drain class and the accepted clean-HEAD retry proves a
real end-to-end submitted-share path with safe shutdown. A rejected response
is the contract's valid proof outcome; it does not claim pool acceptance,
profitability, unbounded stability, default-profile safety, release readiness,
or parity promotion. The exact historical byte-level trigger remains unknown
because raw serial was intentionally ephemeral. No direct UART, pin work,
network discovery, raw serial persistence, secret retention, or evidence
promotion occurred.

### task-ultra205-accepted-pool-share | 2026-07-31 | Obtain one accepted owner-pool share

Status: Complete — the deterministic diagnosis and software fix are verified.
`attempt-001` then proved local below-target filtering but ended on
an unclassified early safe stop. `attempt-002` classified the stop as an ASIC
bridge failure. `attempt-003` crossed that boundary and isolated the remaining
failure to the poll side; code review found an untracked in-flight poll path
capable of filling the worker queue. `attempt-004` crossed that boundary and
isolated a silent synchronous hardware-preparation stall after the fan-full
step started. `attempt-005` crossed that boundary, sustained mining, and then
isolated a false-stale observation race caused by waking the owner before
releasing the observation store, requiring an ordering fix and post-fix
`attempt-006`. That attempt sustained mining until the host's wall-clock
capture cut a periodic marker before the device's active-time lease expired,
requiring bounded terminal-capture grace. Clean-HEAD `attempt-007` crossed the
final boundary and sealed one pool-accepted correlated share with safe stop.

- [x] Reproduce the rejected-share path deterministically from a known BM1366
      nonce, reconstructed header, and pool difficulty.
- [x] Prove the rejection cause against the read-only reference behavior and
      preserve only closed privacy-safe diagnostics.
- [x] Fix the production correlation/submission boundary so only a share that
      satisfies the active pool target can be submitted.
- [x] Verify the exact clean-HEAD package, detect exactly one Ultra 205, and
      run one bounded conservative accepted-share attempt.
- [x] Seal an accepted share, fresh safety, trusted identity, confirmed safe
      stop, lease cleanup, `mineonboot=false`, and USB cleanup without parity
      promotion or secret retention.

Dependencies: Complete archived task `task-ultra205-live-pool-share` and its
accepted `submit_response_observed` proof at clean commit `9861f4c4`.

Hardware contract:

- Permitted commands:
  1. `just detect-ultra205`
  2. `just package`
  3. `just mining-campaign stage=live-share profile=conservative board=205 port=<detector-port> manifest=bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json wifi-credentials=wifi-credentials.json pool-credentials=<single-ignored-local-pool-file> evidence-dir=scratch/ultra205-accepted-pool-share/attempt-001 duration-seconds=600 redact-evidence=true`
  4. After the diagnostic change is committed/pushed and rebuilt from exact
     clean HEAD, the same command once with
     `evidence-dir=scratch/ultra205-accepted-pool-share/attempt-002`.
  5. After the ASIC result-loop fix is committed/pushed and rebuilt from exact
     clean HEAD, the same command once with
     `evidence-dir=scratch/ultra205-accepted-pool-share/attempt-003`.
  6. After the poll in-flight invariant and queue diagnostics are
     committed/pushed and rebuilt from exact clean HEAD, the same command once
     with `evidence-dir=scratch/ultra205-accepted-pool-share/attempt-004`.
  7. After the fan preparation path is proven non-blocking, incomplete
     preparation is classified by the host, and the fix is committed/pushed
     and rebuilt from exact clean HEAD, the same command once with
     `evidence-dir=scratch/ultra205-accepted-pool-share/attempt-005`.
  8. After observation publication is proven to release storage before waking
     the mining owner, the false-stale read is removed, and the fix is
     committed/pushed and rebuilt from exact clean HEAD, the same command once
     with `evidence-dir=scratch/ultra205-accepted-pool-share/attempt-006`.
  9. After the host capture budget is proven to include preparation and
     safe-stop grace without extending the 600-second device mining lease, the
     fix is committed/pushed and rebuilt from exact clean HEAD, the same
     command once with
     `evidence-dir=scratch/ultra205-accepted-pool-share/attempt-007`.
- Objective: obtain one pool-accepted Stratum V1 share derived from current
  owner-pool work and a correlated BM1366 nonce, then prove safe stop.
- Evidence: the ignored
  `scratch/ultra205-accepted-pool-share/attempt-001` root is private,
  non-promoted `ProtectedOperational` evidence with mode-0700 parent and
  mode-0600 artifacts. Persist only closed categories, bounded counts and
  durations, safe provenance, and digests. Never persist or summarize raw
  serial, pool responses, submit payloads, targets, difficulty values,
  endpoints, ports, users, workers, owner addresses, passwords, Wi-Fi values,
  device paths, IPs, MACs, tokens, NVS secrets, or secret-derived hashes.
- Preconditions: the deterministic regression fails before and passes after
  the fix; all required software gates pass; the fix is committed and pushed;
  the package is rebuilt from clean exact HEAD; exactly one board 205 is
  admitted; and exactly one ignored local pool input plus ignored Wi-Fi input
  exists without being read into output or evidence.
- Allowed effects: private NVS injection of Wi-Fi and owner pool settings,
  persistence of `mineonboot=false`, one conservative campaign lease, exact
  package flash, repo-owned USB reset/re-enumeration, fan 100%, DS4432U 1100
  mV, ASIC enable/reset, BM1366 initialization and work/result traffic,
  Stratum V1 TCP connection and qualified-share submission, fresh-session
  HTTP/WebSocket observation, and bounded device-local safe stop.
- Safety and stop limits: observations must remain fresh; input must remain
  4.5-5.5 V; power must not exceed 15 W; ASIC temperature must remain below
  75 C; fan RPM must remain fresh and nonzero after the qualified 100% command.
  Any sensor, watchdog, actuation, validity, generation, lease, transport, or
  evidence fault blocks submission and begins safe stop immediately.
- Prohibited effects: submission of locally known below-target work, TLS,
  Stratum V2, automatic fan mode, mining beyond 600 seconds, non-205 hardware,
  erase-flash, arbitrary raw writes, OTA, recovery upload, network discovery,
  foreign-process termination, raw secret output, parity promotion, direct
  UART, pins, pads, headers, GPIO, probes, jumpers, soldering, injected
  signals, voltage/fan stress, or fault injection.
- Recovery/restoration: preserve the earliest typed failure; block and
  invalidate submissions; close owned pool transports; frequency-down and
  reset the ASIC; set core voltage off and ASIC enable off; keep fan at 100%
  until fresh temperature is at or below 45 C, then set 30%; clear the lease;
  persist `mineonboot=false`; retain pool settings; and release owned USB and
  process resources. If device-local stop cannot be confirmed, one exact
  baseline reflash is allowed only after same-device re-admission; otherwise
  stop.
- Retry bound: `attempt-001` is sealed and immutable; it may not be repeated.
  The owner-authorized non-invasive remediation added a closed terminal reason
  and aligned status freshness with the authoritative runtime clock, providing
  objective change proof for `attempt-002`. That attempt selected a new ASIC
  failure boundary. The pinned upstream result loop treats invalid receives as
  a dropped iteration and continues, while the Rust poll path terminalized a
  complete malformed frame. A deterministic soft-discard regression plus that
  boundary-changing fix authorized `attempt-003`. That attempt produced the
  new refined signature `production_asic_poll_unavailable`; four valid
  candidates preceded it and the closed malformed-discard count remained
  zero. The owner could enqueue another poll on unrelated wakeups while one
  was in flight, allowing bounded worker-queue backpressure to surface as the
  same terminal poll category. A deterministic one-poll-in-flight regression,
  the invariant fix, and distinct queue-full/worker-disconnected categories
  authorize exactly one `attempt-004`. That attempt emitted the typed
  `set_fan_duty_to_100_percent` started boundary but neither completion nor
  failure while runtime attestations remained live for the full capture. A
  deterministic deferred-reply fan-enqueue regression, non-blocking observation
  access, and host classification of incomplete preparation authorize exactly
  one `attempt-005`. Any later ordinal requires another new deterministic
  regression and verified boundary change. `attempt-005` completed every
  preparation boundary and sustained active mining, but the newly introduced
  non-blocking observation read could synthesize stale truth when the producer
  woke the owner while still holding the observation mutex. A deterministic
  release-before-wakeup regression and removal of that false-stale path
  authorize exactly one `attempt-006`. That attempt reached 584 seconds active
  before the 600-second host wall-clock capture ended inside a periodic marker;
  preparation time made the host deadline earlier than the device's
  active-time lease deadline. A deterministic 780-second host-capture/600-second
  device-lease regression and bounded 180-second terminal grace authorize
  exactly one `attempt-007`. A recurrence of the same refined authoritative
  signature selects `stop_repeated_boundary`.
- Accepted terminal outcome: `complete` only for `submit_response_observed`
  with `submit_outcome=accepted`, trusted exact-package identity, clean serial
  diagnostics, fresh supported safety, `mineonboot=false`, confirmed safe
  stop, lease removal, USB cleanup ready, valid artifact seals and modes, and
  no parity promotion. A rejected response or any other failure stops without
  retry and returns to diagnosis under the retry-bound rule.

Verification: The pre-fix regression reproduced submission of a known
below-pool-target reference nonce, and the BM1366 parser regression reproduced
the nonce byte swap. The pinned upstream vectors prove the corrected header
reconstruction and difficulty calculation, while the production wire-frame
test proves the submit nonce is preserved. The fix also keeps distinct
candidates from one job eligible and ignores exact duplicates. `cargo fmt
--all`, warnings-as-errors Clippy, all-target/all-feature build, all-feature
tests, focused ASIC/Stratum/flash tests, production-session verification,
`just test`, `just package`, Bright Builds checks, parity, reference
cleanliness, redaction, and diff checks pass. Clean commit `db1974ac`
`attempt-001` admitted the exact package and
one board, preserved trusted runtime identity and clean serial framing, and
sealed fresh supported observations, `mineonboot=false`, confirmed safe stop,
and ready USB cleanup. It correctly counted one below-pool-target ASIC
candidate and submitted none, but consumed after `5,835` active milliseconds
with `submit_response_missing`. Because v5 did not retain the session blocker
and status freshness used a different clock origin than the authoritative
safety gate, the exact early-stop cause remains unclassified. Ephemeral raw
detector and console logs were deleted after extracting these closed facts;
the sealed private attempt remains ignored and non-promoted.
Clean commit `04de47c9` `attempt-002` reproduced one below-pool-target
candidate and no submit, then sealed `terminal_reason=production_asic_unavailable`
after `5,644` active milliseconds with the same trusted identity, clean serial
framing, fresh supported observations, safe stop, and cleanup guarantees. This
rules out the host parser, pool response, and safety gate as the immediate
stop. The result-loop fix now soft-discards malformed complete frames like the
pinned upstream loop and adds closed version-mask/dispatch/poll terminal
subtypes for any remaining ASIC failure. Ephemeral attempt-002 detector and
console logs were deleted; sealed private evidence remains ignored and
non-promoted.
Clean commit `7d871de8` `attempt-003` retained trusted identity, clean serial
framing, fresh supported observations, `mineonboot=false`, confirmed safe
stop, and ready USB cleanup. It classified four below-pool-target candidates,
zero qualified candidates, and no submit before
`terminal_reason=production_asic_poll_unavailable` after `5,765` active
milliseconds. No malformed-frame discard occurred, disproving that trigger
for this attempt. The shell/core review then identified untracked concurrent
poll requests as a concrete worker-queue backpressure path. Ephemeral detector
and console logs were deleted after extracting only these closed facts; the
sealed private attempt remains ignored and non-promoted.
Clean commit `a80060fe` `attempt-004` retained trusted identity, clean serial
framing, fresh supported observations, `mineonboot=false`, ready USB cleanup,
and valid private artifact modes and seals. It never reached active mining or
read the local pool input. Three typed preparation events ended at
`step=set_fan_duty_to_100_percent,outcome=started`; no completion or failure
followed even though runtime attestations continued through the full bounded
capture. This disproves the poll invariant as the immediate attempt-004
boundary and isolates a synchronous fan-preparation stall. The host's
`pool_configuration_missing` result and retained `network_unavailable` marker
were pre-preparation state, not evidence of a new network failure. Ephemeral
detector and console logs were deleted after extracting only these closed
facts; the sealed private attempt remains ignored and non-promoted.
Clean commit `ec23da41` `attempt-005` crossed the preparation and ASIC failure
boundaries: all nine preparation steps completed, active mining continued for
`439,041` milliseconds, and 60 valid candidates were classified below the
active pool target with zero duplicates, qualified candidates, or submissions.
The attempt then safe-stopped on `terminal_reason=safety_prerequisites_stale`
with trusted identity, clean serial framing, local owner pool input admitted,
fresh supported terminal observations, `mineonboot=false`, confirmed safe
stop, ready USB cleanup, and valid private artifact modes and seals. The
producer called the owner wakeup while still holding the observation-store
mutex, and the new non-blocking reader converted that transient contention
into an empty stale snapshot. This identifies a deterministic false-stale race
rather than a physical safety observation failure. The sealed private attempt
remains ignored and non-promoted; no raw serial or credential material was
retained.
Clean commit `196a9846` `attempt-006` completed all preparation boundaries and
mined for `584,185` active milliseconds with trusted identity, clean complete
serial lines before the final partial, fresh supported observations,
`mineonboot=false`, and ready USB cleanup. It classified 91 valid candidates
below the active pool target, with zero duplicates, qualified candidates, or
submissions. The 600-second host wall-clock capture then ended inside one
otherwise valid periodic marker, producing
`serial_outcome_detail=marker_truncated` before the device's separately counted
600-second active lease could terminate and publish safe-stop proof. This is a
host observation-budget bug, not malformed firmware output or a share
rejection. The device lease remains capped at 600 active seconds; only the host
capture receives bounded terminal grace. The sealed private attempt remains
ignored and non-promoted; no raw serial or credential material was retained.

Clean commit `3e0966a1` `attempt-007` admitted the exact package and one Ultra
205, completed every preparation boundary, and mined for `215,958` active
milliseconds. It classified 71 valid candidates below the active pool target,
then correlated one qualified candidate with current work, submitted it once,
and received a real accepted pool response. The sealed v2 result is accepted
as `submit_response_observed` with clean serial diagnostics, 1,780 accepted
markers, trusted runtime identity, all five supported observations fresh,
`mineonboot=false`, confirmed safe stop, lease cleanup, and USB cleanup ready.
The private root is mode 0700, all four artifacts are mode 0600, the result
seal and both bound private-artifact digests verify, redaction is enabled, and
parity promotion is false. No raw serial, pool response, credential, endpoint,
target, difficulty, device identifier, or secret-derived value was retained.

Completion review: The historical rejection was caused by incorrect BM1366
nonce byte order and missing local target qualification at the submission
boundary. The corrected path reconstructs and validates candidates against
current work, filters below-target and duplicate results locally, and preserves
the submit nonce on the wire. The accepted clean-HEAD hardware result proves
that bounded conservative path end to end, including pool acceptance and safe
shutdown. It does not prove profitability, default-profile safety, unbounded
stability, release readiness, or parity promotion.

### task-ultra205-job-transition-soak | 2026-07-31 | Prove a 30-minute new-block transition

- [x] Add a closed `job-transition` campaign at the conservative Ultra 205
      profile with an exact 1,800-active-second lease and 1,980-second host
      observation budget.
- [x] Replace cumulative campaign serial capture with bounded chunk-fed
      analysis and aggregate observations that cannot retain raw serial.
- [x] Prove a changed previous-block notify invalidates old work, advances the
      generation, dispatches replacement work, and correlates a replacement
      result without submitting stale work.
- [x] Run `attempt-001` once from clean pushed commit `e732ca4b`; preserve its
      fail-closed result and do not open the conditional retry gate.
- [x] Land and verify the two regression fixes exposed by `attempt-001`:
      incremental typed runtime-attestation classification and in-flight
      transition lineage across same-block clean generation refreshes.
- [x] Run exactly one newly authorized post-fix `attempt-002` from the clean,
      pushed tracker-amendment HEAD; never run `attempt-003` under this task.
- [x] Seal one full-duration hardware attempt with continuous fresh safety,
      trusted identity, safe stop, lease cleanup, `mineonboot=false`, and USB
      cleanup using the newly authorized post-fix ordinal below.

Dependencies: Complete archived `task-ultra205-accepted-pool-share` with its
accepted clean-HEAD owner-pool share and confirmed safe stop.

Hardware contract:

- Permitted repo-owned commands:
  1. `just package`
  2. `just detect-ultra205`
  3. `just mining-campaign stage=job-transition profile=conservative board=205 port=<detector-port> manifest=bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json wifi-credentials=wifi-credentials.json pool-credentials=<single-ignored-local-pool-file> evidence-dir=scratch/ultra205-job-transition-soak/attempt-001 duration-seconds=1800 redact-evidence=true`
  4. After the 2026-07-31 post-fix authorization, the same command once with
     `evidence-dir=scratch/ultra205-job-transition-soak/attempt-002`; this is
     the final attempt authorized by this task.
- Objective: prove at least one in-session Bitcoin previous-block transition
  from a clean pool notify through old-generation invalidation, replacement
  BM1366 dispatch, and a correlated replacement-generation nonce while mining
  for the full 1,800 active seconds.
- Evidence: each ignored attempt root is mode 0700 with mode-0600
  `ProtectedOperational` artifacts. Persist only closed states, counts,
  bounded durations, safe provenance, and digests. Never persist raw serial,
  block hashes, job IDs, pool messages, submit payloads, targets, difficulty,
  credentials, endpoints, workers, owner addresses, device identifiers,
  network values, tokens, NVS secrets, or secret-derived hashes. Evidence is
  private, redacted, sealed, and never automatically promoted.
- Preconditions: deterministic regressions fail before and pass after the
  implementation; all required software gates pass; changes are committed and
  pushed; the exact package is rebuilt from clean HEAD; exactly one board 205
  is admitted; and ignored Wi-Fi plus exactly one ignored pool input exist
  without their contents being printed or retained.
- Allowed effects: private NVS injection of Wi-Fi and owner pool settings,
  persistence of `mineonboot=false`, one conservative 400 MHz / 1100 mV /
  100% fan campaign lease, exact package flash, repo-owned USB reset and
  re-enumeration, BM1366 initialization/work/result traffic, Stratum V1 pool
  traffic and locally qualified submissions, bounded public Bitcoin-tip reads
  for the conditional retry gate, and device-local safe stop.
- Safety and stop limits: all five supported Ultra 205 safety observations
  must remain fresh; input must remain 4.5-5.5 V; power must not exceed 15 W;
  ASIC temperature must remain below 75 C; fan RPM must remain fresh and
  nonzero after the 100% command. Any safety, watchdog, transport, parser,
  protocol-consistency, generation, dispatch, correlation, actuation, lease,
  evidence, or cleanup fault blocks submissions and begins safe stop.
- New-block acceptance: require at least one `clean_jobs=true` notify with a
  changed previous-block value, matching new-block generation invalidation,
  replacement dispatch, and a correlated result under that replacement
  generation. Require no active-marker gap greater than 5,000 ms, zero rejected
  shares, and zero stale-generation submissions. An accepted share is optional;
  a valid below-target replacement nonce satisfies result correlation.
- Prohibited effects: mining beyond 1,800 active seconds, upstream-default
  actuation, TLS, Stratum V2, automatic fan mode, non-205 hardware,
  erase-flash, arbitrary raw writes, OTA, recovery upload, local network
  discovery, foreign-process termination, raw secret output, raw serial
  persistence, parity promotion, direct UART, pins, pads, headers, GPIO,
  probes, jumpers, soldering, injected signals, stress, or fault injection.
- Recovery/restoration: preserve the earliest typed failure; block and
  invalidate submissions; close owned pool transports; frequency-down and
  reset the ASIC; set core voltage and ASIC enable off; keep fan at 100% until
  fresh temperature is at or below 45 C, then set 30%; clear the lease;
  persist `mineonboot=false`; retain pool settings; and release USB/process
  resources. If safe stop cannot be confirmed, one exact baseline reflash is
  allowed only after same-device re-admission; otherwise stop.
- Retry bound: `attempt-001` sealed `job_transition_evidence_incomplete` and
  did not open its original conditional retry gate. The user subsequently
  authorized exactly one post-fix `attempt-002` after the two boundary defects
  were reproduced, fixed, and fully software-verified in pushed commit
  `5d530464`. Rebuild from the clean pushed tracker-amendment HEAD, re-detect
  exactly one board, and run that ordinal once. Any safety, identity, parser,
  protocol, generation, dispatch, correlation, rejection, transport, evidence,
  lease, safe-stop, or cleanup failure stops without retry. If no transition is
  observed, stop inconclusive without a public-tip wait. Never run
  `attempt-003` under this task.
- Accepted terminal outcomes: `complete` only for full-duration
  `job_transition_complete` plus every identity, safety, transition, rejection,
  safe-stop, seal, mode, and cleanup requirement. The only non-failure
  conditional outcome is `job_transition_not_observed`; every other category
  stops without retry and returns to diagnosis.

Verification: Software gates passed on 2026-07-31 before hardware: the exact
Rust pre-commit sequence, focused Stratum/campaign/device-session regressions,
`just verify-production-session`, `just test`, `just package`, Bright Builds
checks, parity, reference cleanliness, and redaction. `attempt-001` then ran
for 1,800,133 active ms and sealed `job_transition_evidence_incomplete` with
five previous-block changes, five matching generation advances, five
replacement dispatches, zero credited post-transition results, zero rejected
shares, zero stale-generation submissions, zero reconnects, a 519 ms maximum
active-marker gap, fresh required safety, `mineonboot=false`, confirmed safe
stop, and USB cleanup ready. Its private artifacts are mode 0600 under a mode
0700 ignored root and their result-bound digests verify. The conditional retry
gate did not open. Red regressions reproduced two host-accounting defects:
partial retention of the final runtime attestation at the old text-byte cap,
and loss of an in-flight transition lineage after a same-block clean generation
refresh. Both red regressions pass after the fixes, as do the exact Rust
sequence, focused API/Stratum/campaign suites, production-session verification,
all 82 Bazel test targets, package build, Bright Builds checks, parity,
reference cleanliness, redaction, artifact mode/seal/digest checks, and the
private evidence denylist. The newly authorized, no-retry post-fix
`attempt-002` then ran from clean pushed commit `cea568dd` for 1,800,120 active
ms and safely sealed `job_transition_evidence_incomplete`. It observed one
changed-previous-block notify, one matching new-block generation, and 761
replacement-work dispatches, but zero post-transition correlated results and
therefore zero completed transitions. The run otherwise recorded 90
below-pool-target results, zero rejected shares, zero stale-generation results
or submissions, zero reconnects, a 532 ms maximum active-marker gap, trusted
runtime identity and attestation, fresh required safety, `mineonboot=false`,
confirmed safe stop, and USB cleanup ready. All four artifacts have the
required owner-only modes, both private-artifact digests and the result seal
verify, and the private evidence denylist passes. One trailing partial serial
candidate was conservatively classified as `marker_truncated`; the accepted
terminal marker and trusted attestation remained intact, so this was not the
campaign failure. The authorized retry budget is exhausted and `attempt-003`
is prohibited.

Completion review: Incomplete. Two bounded full-duration attempts safely
reached lease expiry and proved changed-block detection, generation advance,
and replacement dispatch, but neither proved a correlated result under the
replacement generation. The remaining blocker is the post-transition ASIC
result-correlation link; further hardware execution requires a new task and
fresh authorization after additional deterministic diagnosis. This task does
not prove new-block transition completion, profitability, upstream-default
stability, unbounded mining, automatic fan control, release readiness, or
parity promotion.

Supersession review: Superseded on 2026-07-31 by
`task-ultra205-job-transition-poll-liveness`. The historical attempts and
conclusions above remain unchanged; the successor owns the deterministic
poll-liveness proof, typed diagnostics, and any newly authorized hardware
validation under a distinct evidence root.

### task-ultra205-job-transition-poll-liveness | 2026-07-31 | Restore post-transition ASIC polling

- [x] Capture a deterministic red Production Mining Session regression proving
      that a stale old-generation poll completion suppresses replacement-
      generation polling after a clean changed-block notify.
- [x] Fix successful generation advances by invalidating only stale bridge
      orchestration state before replacement work is queued, without disarming
      the ASIC listener or allowing an old completion to mutate a newer poll.
- [x] Add typed, privacy-safe ASIC bridge and BM1366 parser diagnostics,
      version the campaign status/result/observation contracts, and seal a
      bounded private mining-diagnostics artifact.
- [x] Document behavioral parity with the pinned upstream independent job
      dispatch and continuous ASIC result tasks without copying GPL expression.
- [x] Complete the required ordered software verification, commit and push the
      verified fix, then rebuild the exact clean-HEAD package.
- [x] Run one authorized 1,800-active-second conservative real-pool attempt,
      with one conditional no-transition retry only under the gate below.

Dependencies: Supersedes archived `task-ultra205-job-transition-soak`, whose
two safe full-duration attempts proved changed-block detection, generation
advance, and replacement dispatch but exhausted their retry budget without a
post-transition correlated result.

Hardware contract:

- Permitted repo-owned commands:
  1. `just package`
  2. `just detect-ultra205`
  3. `just mining-campaign stage=job-transition profile=conservative board=205 port=<detector-port> manifest=bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json wifi-credentials=wifi-credentials.json pool-credentials=<single-ignored-local-pool-file> evidence-dir=scratch/ultra205-job-transition-poll-liveness/attempt-001 duration-seconds=1800 redact-evidence=true`
  4. Only if attempt-001 seals `job_transition_not_observed` and every other
     gate passes: read the public Bitcoin tip height until it strictly advances,
     persist only `public_tip_advanced=true`, rebuild the same clean-HEAD
     package, re-detect the board, and run the same command once with
     `evidence-dir=scratch/ultra205-job-transition-poll-liveness/attempt-002`.
- Objective: prove at least one clean changed-block notify advances the work
  generation, invalidates the old bridge poll state, dispatches replacement
  BM1366 work, rearms and completes replacement-generation polling, and
  decodes and correlates at least one replacement-generation nonce.
- Evidence: each ignored attempt root is mode 0700 with mode-0600
  `ProtectedOperational` artifacts. Persist only closed states, monotonic
  counts/durations, bounded typed traces, safe provenance, and digests. Never
  persist raw serial or UART bytes, block hashes, job IDs, generations as raw
  values, nonces, pool messages, submit payloads, targets, difficulty,
  credentials, endpoints, workers, owner addresses, device identifiers,
  network values, tokens, NVS secrets, or secret-derived hashes. Evidence is
  private, redacted, sealed, and never automatically promoted.
- Preconditions: the red poll-starvation regression fails on the historical
  implementation and passes after the fix; all required software gates pass;
  changes are committed and pushed; the exact package is rebuilt from clean
  HEAD; exactly one Ultra 205 is admitted; and ignored Wi-Fi plus exactly one
  ignored pool input exist without their contents being printed or retained.
- Allowed effects: private NVS injection of Wi-Fi and owner pool settings,
  persistence of `mineonboot=false`, one conservative 400 MHz / 1100 mV /
  100% fan campaign lease per admitted attempt, exact package flash,
  repo-owned USB reset and re-enumeration, BM1366 initialization/work/result
  traffic, Stratum V1 pool traffic and locally qualified submissions, the
  bounded public tip-height retry gate, and device-local safe stop.
- Safety and stop limits: all five supported Ultra 205 safety observations
  must remain fresh; input must remain 4.5-5.5 V; power must not exceed 15 W;
  ASIC temperature must remain below 75 C; fan RPM must remain fresh and
  nonzero after the 100% command. Any safety, watchdog, transport, parser,
  protocol-consistency, generation, dispatch, correlation, actuation, lease,
  evidence, or cleanup fault blocks submissions and begins safe stop.
- Acceptance: require the full 1,800 active seconds, trusted exact-package
  identity, no active-marker gap above 5,000 ms, at least one clean changed-
  block notify and generation advance, replacement dispatch followed by a
  post-transition poll request and completion, at least one decoded and
  correlated replacement-generation nonce, zero rejected shares, stale
  submissions, reconnects, or unresolved in-flight poll at safe stop,
  `mineonboot=false`, confirmed safe stop, lease removal, sealed diagnostics,
  and USB cleanup ready.
- Prohibited effects: mining beyond the exact lease, upstream-default
  actuation, TLS, Stratum V2, automatic fan mode, non-205 hardware, erase-
  flash, arbitrary raw writes, OTA, recovery upload, network discovery,
  foreign-process termination, raw secret output, raw serial persistence,
  parity promotion, fresh upstream firmware flashing, direct UART, pins,
  pads, headers, GPIO, probes, jumpers, soldering, injected signals, stress,
  or fault injection.
- Recovery/restoration: preserve the earliest typed failure; block and
  invalidate submissions; close owned pool transports; frequency-down and
  reset the ASIC; set core voltage and ASIC enable off; keep fan at 100% until
  fresh temperature is at or below 45 C, then set 30%; clear the lease;
  persist `mineonboot=false`; retain pool settings; and release USB/process
  resources. If safe stop cannot be confirmed, one exact baseline reflash is
  allowed only after same-device re-admission; otherwise stop.
- Retry bound: attempt-002 is permitted only when attempt-001 seals
  `job_transition_not_observed` and every other gate passes, and only after a
  public tip-height advance. Parser, correlation, safety, transport, evidence,
  rejection, or `job_transition_evidence_incomplete` outcomes stop without
  retry. A second clean no-transition attempt ends inconclusive. No further
  ordinal is authorized.
- Accepted terminal outcomes: `complete` only for the full acceptance
  contract; `job_transition_not_observed` may open the single conditional
  retry; otherwise classify with the typed diagnostics and stop.

Verification: The pre-fix regression failed at
`stale_old_generation_poll_completion_rearms_replacement_generation_poll`
because no replacement-generation `PollAsic` effect followed the stale
old-generation completion. It passes after the bridge invalidation fix, as do
the companion stale-nonce, same-block clean, repeated-clean, parser subtype,
strict v8 marker, bounded first/last trace, v4/v3 evidence, private-artifact
digest, permission, and privacy tests. The original nine job-transition
regressions and all new liveness/diagnostic cases pass. The ordered Rust gates
(`cargo fmt --all`, warnings-denied Clippy, all-target and all-feature build,
and all-feature tests), focused ASIC/Stratum/flash suites,
`just verify-production-session`, all 82 `just test` Bazel targets,
`just package`, Bright Builds checks, parity, reference cleanliness, redaction,
artifact mode/seal/digest tests, privacy denylisting, `git diff --check`, and
final diff review all pass. The implementation was committed and pushed as
`34b551be`, the exact clean-HEAD package recorded `source_dirty=false`, and
`attempt-001` then sealed accepted `job_transition_complete` after 1,800,164
active ms. It observed 59 pool notifies, nine clean notifies, two changed-block
generations, 251 replacement dispatches, 1,957 post-transition poll requests,
1,726 post-transition completions, and 227 decoded and correlated replacement-
generation nonces. It completed both transitions, accepted 52 shares, rejected
zero, submitted zero stale-generation shares, reconnected zero times, recorded
zero parser discards and zero blocked correlations, and ended with the poll
state invalidated rather than in flight. Maximum active-marker gap was 335 ms;
all five required safety observations remained fresh; `mineonboot=false`, the
lease was consumed, safe stop was confirmed, and USB cleanup was ready. The
mode-0700 ignored root contains exactly five mode-0600 artifacts; all three
artifact digests, the result seal, strict schemas, and privacy denylist verify.
The conditional retry gate did not open and `attempt-002` was not created.

Completion review: Complete. The deterministic red test and accepted real-pool
campaign confirm the root cause: a clean generation advance previously left an
old poll marked in flight, suppressing replacement-generation polling while
redispatch continued. Invalidating that bridge state restores continuous
polling without letting stale completions mutate newer work. This proves one
bounded conservative 30-minute session across two real block transitions; it
does not prove profitability, upstream-default stability, unbounded mining,
automatic fan control, release readiness, direct electrical access, or parity
promotion.

### task-campaign-websocket-connection-stability | 2026-08-01 | Stabilize campaign WebSocket observation

- [x] Exercise the attempt-003 signature with one real loopback TCP connection
      across 109 representative idle observation intervals and record that the
      exact reconnect count did not reproduce on the pre-fix macOS path.
- [x] Verify Tungstenite's transport contract that `WouldBlock` permits reuse
      while `TimedOut` and other I/O errors are fatal.
- [x] Implement a bounded, non-busy persistent read strategy that preserves the
      existing 64-KiB message cap, plain-`ws://` restriction, omitted `Origin`
      header, and privacy boundary.
- [x] Prove idle connections remain open, genuine peer closes permit a fresh
      connection with 1/2/4/5-second bounded backoff, and sockets, threads, and
      related resources are released on terminal paths.
- [x] Re-run the campaign transport, continuity, evidence, and redaction suites
      without weakening window completeness or earliest-failure precedence.

Dependencies: The software-only startup-state recovery recorded in
`task-ultra205-default-profile-soak` was complete before this task began. This
task's software prerequisite for drafting a future soak contract is complete.

Authorization boundary: software and real loopback TCP tests only. No hardware,
package flashing, Wi-Fi or pool credentials, device discovery, raw device
origin, attempt-004, or other network target was used.

Verification:

- The red classifier regression failed because production treated both
  `WouldBlock` and `TimedOut` as reusable, contrary to Tungstenite's connection
  reuse contract. After the change, only `WouldBlock` is retried; `TimedOut`,
  capacity, protocol, and other I/O errors are fatal.
- A real loopback connection survived 109 bounded idle reads and then received
  text without a second handshake. That scenario also passed before the fix on
  this host, so it guards persistence but does not independently reproduce or
  prove the exact attempt-003 reconnect cause.
- Real TCP tests cover omitted `Origin`, nonblocking ping/pong flushing, the
  exact 64-KiB admissible boundary, oversized-message rejection, peer close,
  a fresh subsequent connection, and joined server threads. A deterministic
  control-flush test forces one temporary `WouldBlock` before success, and
  elapsed-time bounds cover the non-busy local observation deadline.
- Deterministic observer tests prove delays of 1, 2, 4, then capped 5 seconds
  and reset to 1 second after a successful connection. Idle
  `WebSocketRead::Timeout` continues to leave the current socket installed.
- Focused `bitaxe-http-transport` and campaign-network suites passed. In order,
  `cargo fmt --all`, warnings-denied Clippy, the all-target/all-feature Cargo
  build, all-feature Cargo tests, all 82 Bazel tests through `just test`, the
  managed Bright Builds checks with zero findings, and
  `just verify-redaction` passed.

Completion review: Complete for the authorized software scope. The client now
uses blocking handshake deadlines followed by a timeout-free nonblocking
socket, retries only `WouldBlock` in 25-ms bounded sleeps, and returns a local
timeout without discarding an idle connection. Control-frame flushes use the
same bound, while genuine close or fatal error releases the socket and enters
the explicit reconnect policy. The public `PlainWebSocket` API, campaign
continuity contract, evidence schema, privacy limits, CLI, and dependencies are
unchanged. Residual risk: attempt-003 retained only aggregate evidence, and the
109-reconnect signature did not reproduce under loopback before the fix, so a
future separately authorized soak must prove a maximum 5,000-ms WebSocket gap
and no recurrence before the broader task can complete.

### task-fix-campaign-websocket-control-frames | 2026-08-01 | Fix campaign control frames and truncated markers

- [x] Enable ESP-IDF WebSocket control-frame dispatch and fully consume bounded
      Ping, Pong, and Close frames before returning to the HTTP server parser.
- [x] Preserve plain `ws://`, omitted `Origin`, the 64-KiB data-message cap,
      connection leases, queued-send ownership, and bounded reconnect policy.
- [x] Add closed aggregate reconnect-cause counts without identifiers, URLs,
      frame bodies, raw errors, or credentials.
- [x] Classify newline-terminated EOF JSON as serial truncation and recover only
      through the existing bounded accepted-marker continuity contract; keep
      syntactically malformed JSON immediately terminal.
- [x] Add red-first source-boundary, transport, continuity, evidence, sealing,
      and privacy regressions, then run every required repository gate.

Dependencies: Root-cause investigation of attempt-004 proved that firmware
sent empty Ping frames every 500 ms while its ESP-IDF URI registration left
control-frame dispatch disabled. The same investigation proved that the first
campaign marker was transport-truncated rather than serializer-invalid.

Authorization boundary: software and loopback tests only. No package build for
hardware use, device detection, credentials, network target, firmware flash,
hardware action, attempt-005, or parity promotion was used or authorized.

Verification:

- The firmware source-contract regression first failed because the WebSocket
  URI did not request ESP-IDF control-frame dispatch. The fixed handler parses
  and drains the remaining bounded frame bytes, answers Ping with Pong, handles
  Pong and Close explicitly, and caps control payloads at the RFC limit of 125
  bytes. The canonical ESP32-S3 release firmware compiled against pinned
  ESP-IDF 5.5.4 with `just build`.
- The serial regression first failed with `marker_invalid` when an 851-byte,
  newline-terminated prefix preceded a complete valid marker. It now records
  one aggregate truncation and accepts the later marker through the unchanged
  continuity contract. Non-EOF malformed JSON remains immediately terminal,
  and a stream with no complete accepted marker remains `marker_missing`.
- Fatal host WebSocket reads now map to closed `io`, `protocol`, `capacity`, or
  `other` categories. Connect failures and peer closes remain separate. The
  private `mining-campaign-network-continuity-v3` artifact exposes only counts;
  tests deny raw errors, identifiers, URLs, bodies, frames, and credentials.
- Focused suites passed: 23 campaign-serial tests, 22 campaign-network tests,
  and 9 real WebSocket transport tests. In order, `cargo fmt --all`,
  warnings-denied Clippy, the all-target/all-feature Cargo build, all-feature
  Cargo tests, all 82 Bazel tests through `just test`, the managed Bright Builds
  checks with zero findings, and `just verify-redaction` passed. The complete
  diff passed `git diff --check`.

Completion review: Complete for the authorized software scope. The fault was
at the ESP-IDF control-frame ownership boundary, not in Tungstenite's idle-read
policy: with dispatch disabled, ESP-IDF consumed the first control-frame byte
before the application handler attempted its own receive, which could leave the
stream parser misaligned. Firmware now owns and drains every dispatched control
frame coherently. A newline-terminated JSON EOF is treated as transport
truncation rather than serializer corruption, but earns no continuity credit;
only a later complete accepted marker can recover the observation. Residual
risk: no hardware run was authorized, so the fixes are software-verified but
not yet confirmed by a fresh soak. The existing broader soak task remains
active and no attempt-005 or parity claim is authorized by this completion.

### task-comprehensive-reference-parity-checklist | 2026-08-02 | Make the reference-derived parity checklist comprehensive

- [x] Audit the pinned, read-only `reference/esp-miner` tree and inventory every
      device-user-observable behavior and firmware capability, including board
      configuration, boot/runtime services, NVS and settings, ASIC families,
      Stratum and mining, HTTP/OpenAPI/WebSocket/UI surfaces, networking,
      logging and telemetry, power/thermal/fan control, display and input,
      self-test, filesystem, OTA/recovery, packaging, flashing, and release
      behavior.
- [x] Reconcile that inventory into the existing canonical
      `docs/parity/checklist.md` instead of creating a competing checklist.
      Give every independently verifiable surface a stable ID, exact reference
      path plus symbol/route/key breadcrumb, Rust-owned implementation pointer
      or explicit gap, board/ASIC scope, status, evidence type and pointer, and
      precise non-claims.
- [x] Add a deterministic coverage artifact or parity-tool check that proves
      every inventoried reference surface is represented by exactly one
      checklist row or an explicit, reasoned deferral, and fails when a tracked
      surface disappears, is duplicated, or lacks its required metadata.
- [x] Record the pinned reference commit and provenance boundaries without
      modifying `reference/esp-miner` or copying GPL-covered source expression
      into MIT-first Rust files. Preserve existing evidence-backed statuses;
      do not promote a row from implementation or documentation alone.
- [x] Keep safety-critical and hardware-control rows below `verified` unless
      their exact claims have the required named-board hardware evidence, and
      keep non-205 behavior explicitly scoped until separately evidenced.

Dependencies: None. Existing checklist rows, revision records, parity tooling,
and evidence were inputs to reconcile, not completeness proof by themselves.

Verification:

- `just verify-reference` passed and confirmed pinned reference commit
  `c1915b0a63bfabebdb95a515cedfee05146c1d50` remained clean.
- `cargo test -p bitaxe-parity` passed all 398 parity tests, including six
  focused inventory tests for valid, missing, duplicate, commit-drift,
  missing-anchor, and missing-metadata cases. `bazel test
  //tools/parity:tests` also passed.
- `just parity` passed with 99 rows, 99 unique inventory mappings across 12
  audited domains, and `validation_errors: none`. The comprehensive revision
  guard proved no predecessor row was removed or changed status/evidence.
- In required order, `cargo fmt --all`, warnings-denied Clippy, the
  all-target/all-feature Cargo build, and all-feature Cargo tests passed.
  `bun scripts/bright-builds-check.ts all` passed with zero findings after the
  new revision authority was split below the file-length limit.
- `just verify-redaction` and `git diff --check` passed. Final review found no
  unsupported promotion, duplicate ownership, wildcard locator, missing exact
  source anchor, broken new local link, reference-tree modification, or
  sensitive value.

Completion review: Complete. The canonical checklist now contains 99
independently tracked surfaces, adding explicit conservative rows for the full
board-profile matrix, station/SoftAP/DNS/scan/IPv6 networking, address codecs,
HTTP command effects, theme persistence, and AxeOS operator workflows. The
hash-bound companion inventory records exact source locators and anchors,
scope, provenance, and non-claims, while `just parity` fails closed on missing
or duplicate mappings, malformed metadata, reference drift, nonexistent paths,
or missing anchors. Existing evidence-backed statuses and evidence types were
preserved byte-for-byte by the comprehensive revision guard; no hardware,
credentials, network discovery, direct UART, pin action, or reference edit was
performed. Residual risk: inventory completeness is deterministic against the
currently pinned reference commit, not behavioral verification. Any future
reference update must deliberately refresh the inventory and checklist, and
all hardware-bound, non-205, Stratum v2, UI/BAP, OTA/recovery, and active safety
gaps retain their conservative statuses until claim-specific evidence exists.

### task-advance-parity-skill | 2026-08-02 | Add the audited advance-parity workflow

- [x] Add an explicitly invoked repository skill that selects or resumes one
      actionable parity row and persists its plan before implementation.
- [x] Extend the parity CLI with deterministic candidate ranking, progress
      calculation, one-row transitions, progress history, and README syncing.
- [x] Generalize checklist revision validation without weakening existing
      comprehensive-inventory or evidence guards.
- [x] Add the JSONL baseline, README parity block, Bazel/just wiring, and
      behavior-focused regression tests.
- [x] Validate the skill, run every required Rust/Bazel/Bright Builds/parity
      gate, review the final diff, archive this task, commit, and push.

Verification:

- The skill-creator validator passed for `.agents/skills/advance-parity`, and
  the generated OpenAI metadata disables implicit invocation.
- `cargo fmt --all`, warnings-denied Clippy, the all-target/all-feature build,
  and the all-feature test suite passed. The build and test commands used a
  fresh isolated target after the shared macOS Cargo cache stalled at link time.
- `bun scripts/bright-builds-check.ts all` passed with zero findings, and
  `just test` passed all 82 Bazel tests. The stale Phase 35 shell contract now
  validates the hash-bound August 2 comprehensive checklist revision that was
  already authoritative at the starting commit.
- `just parity` passed with `validation_errors: none`; `just parity-progress
  --format json` reported 27 verified of 94 active rows and 2,872 basis points,
  rendered as 28.7%. Re-running progress synchronization deduplicated the
  unchanged checklist digest.
- Selection-only `next-item --format json` found no open plan and ranked
  `CFG-001` first, followed by the remaining `implemented`, `in-progress`, and
  `not-started` rows in checklist order. `git diff --check` passed.

Completion review: Complete. The explicit-only `advance-parity` skill now
requires a clean synchronized branch, commits an immutable plan before work,
resumes unfinished attempts, respects task and hardware authorization gates,
uses conservative status transitions, and pushes truthful verified or partial
checkpoints. The existing Rust parser owns selection, progress arithmetic,
one-row mutation, transition receipts, JSONL hash-chain validation, and README
synchronization, avoiding a competing checklist parser. The initial history
snapshot and README expose the unchanged 27/94 (28.7%) baseline; this task did
not promote a parity row. Residual risk: the selection path and mutation logic
are software-tested, but a real skill invocation has not yet exercised the
first transition receipt or hardware-gated plan. Those paths deliberately fail
closed and remain subject to the repository's existing evidence and recovery
contracts.

### task-parity-asic-006-crc-verification | 2026-08-02 | Verify the complete reference CRC contract

- [x] Persist and commit the immutable `ASIC-006` work plan before source edits.
- [x] Implement the missing zero-initialized CRC16 behavior without copying the
      upstream GPL lookup table into MIT-first Rust source.
- [x] Add deterministic reference vectors for all CRC variants and BM1366 frame
      placement, and retain receive-residue coverage at the parser boundary.
- [x] Produce the worklog and terminal result, transition only `ASIC-006` to
      `verified`, synchronize progress, archive this task, and prepare the
      audited commits for the required final push.

Plan: `docs/parity/work-plans/20260802T181828Z-ASIC-006/PLAN.md`.

Authorization: software-only. No hardware, flash, credentials, network
discovery, direct UART, pins, safety actuation, or reference-tree edits.

Verification: Focused Cargo/Bazel CRC tests and every repository gate passed on
the implementation worktree. The terminal result binds implementation commit
`268a118b565579674695bba523b7c970c7db734a`; the transition receipt binds the
predecessor/result checklist digests and plan/result hashes; progress sync
reports 28/94 verified and 2,979 basis points, rendered as 29.8%. Final gates
are run again after this archival and before the final commit.

Completion review: Complete. All three CRC entry points in the pinned reference
are implemented through a compact bitwise CCITT core, independently fixed
boundary and canonical vectors cover the algorithm variants, exact BM1366
frame bytes prove CRC16-FALSE coverage and byte order, and existing parser tests
retain receive-side zero-residue enforcement. `ASIC-006` alone transitioned
from `implemented` to `verified`; the hash-chained progress history and README
now report 28/94 active items verified (29.8%). Residual risk: no live ASIC or
serial behavior is claimed here; those hardware-bound behaviors remain under
their separate conservative checklist rows. The workflow's final push follows
the finalization commit and therefore is intentionally not represented as
historical evidence inside this archived pre-push task record.

### task-parity-str-002-message-verification | 2026-08-02 | Verify Stratum v1 message parity

- [x] Select `STR-002` as the first actionable deterministic parity candidate
      and commit its immutable audit plan before implementation.
- [x] Make the pinned reference-derived protocol fixture executable as golden
      coverage for every message family owned by `STR-002`.
- [x] Enforce the reference's 32-Merkle-branch parser boundary and add focused
      behavior tests.
- [x] Run the focused and mandatory repository verification commands, record
      the result, transition only `STR-002` when justified, and synchronize
      deterministic progress.

Plan: `docs/parity/work-plans/20260802T184857Z-STR-002/PLAN.md`.

Scope and safety: software-only JSON-RPC parsing, serialization, fixtures, and
tests. No network, credentials, hardware, flashing, mining, transport effects,
direct UART, pin manipulation, or reference-tree edits were authorized or used.

Verification: The focused 55-test Stratum message suite, Bazel Stratum tests,
reference-clean guard, JSON validation, mandatory Rust format/lint/build/test
sequence, managed Bright Builds checks, redaction check, all 82 Bazel tests,
parity validation, progress validation, and diff checks passed. The terminal
result binds implementation commit
`f7b750843d9e6cf094713391b432b2224f895354`; the transition receipt binds the
predecessor/result checklist digests and plan/result hashes. Progress sync
reports 29/94 verified and 3,085 basis points, rendered as 30.9%. Final gates
are run again after this archival and before the final commit.

Completion review: Complete. All 23 pinned synthetic message shapes are
executable through the owned Rust parser or serializer, and exact tests prove
acceptance at 32 Merkle branches and rejection at 33. `STR-002` alone
transitioned from `implemented` to `verified` with `unit,golden` evidence; the
hash-chained progress history and README now report 29/94 active items verified
(30.9%). Residual risk: live sockets, networking, TLS, credentials, pool timing,
retry/reconnect lifecycle, ASIC behavior, share outcomes, mining, and hardware
remain explicit non-claims owned by separate checklist rows. The workflow's
final push follows the finalization commit and is intentionally not represented
as historical evidence inside this archived pre-push task record.

### task-parity-str-003-mining-job-verification | 2026-08-02 | Verify Stratum mining-job construction

- [x] Select `STR-003` as the first actionable parity row and commit the audit
      plan before implementation.
- [x] Convert the pinned reference-derived mining-job fixture into executable
      golden coverage for the full typed Ultra 205 construction boundary.
- [x] Add behavior-focused construction and rejection coverage exposed by the
      fixture audit.
- [x] Run focused and mandatory verification, record evidence, and transition
      only the checklist fields justified by the result.
- [x] Synchronize deterministic parity progress, archive this task after the
      row reaches `verified`, and prepare every audited commit for push.

Plan: `docs/parity/work-plans/20260802T195138Z-STR-003/PLAN.md`.

Authorization and safety: software-only and effect-free. No credentials,
hardware, flashing, pool/network connections, destructive actions, direct
UART, pin manipulation, or reference-tree edits were used.

Verification: Five exact extranonce vectors and one complete pinned upstream
mining-job vector execute through the Rust work builder. Focused Cargo and
Bazel Stratum tests, reference cleanliness, JSON validation, the mandatory Rust
format/lint/build/test sequence, managed Bright Builds checks, redaction, all
82 Bazel tests, parity validation, progress validation, and diff checks passed.
The terminal result binds implementation commit
`242a51ebaa61a6451b11f1122ff159b26a274b5e`; the transition receipt binds the
predecessor/result checklist digests and plan/result hashes. Progress sync
reports 30/94 verified and 3,191 basis points, rendered as 31.9%. Final gates
are run again after this archival and before the final commit.

Completion review: Complete. Exact assertions now cover extranonce2 encoding,
coinbase hashing, Merkle folding, every typed BM1366 work field, retained job
identity and pool context, plus malformed-branch rejection. `STR-003` alone
transitioned from `implemented` to `verified` with `unit,golden` evidence; the
hash-chained progress history and README report 30/94 active items verified
(31.9%). Residual risk: live sockets, networking, TLS, credentials, reconnect
behavior, ASIC dispatch, nonce validation, share outcomes, production mining,
other ASIC families, and hardware remain explicit non-claims owned by separate
checklist rows. The workflow's final push follows the finalization commit and
is intentionally not represented as historical evidence inside this archived
pre-push task record.

### task-parity-str-004-coinbase-decoder-verification | 2026-08-02 | Verify deterministic coinbase decoding parity

- [x] Implement typed, bounds-checked coinbase transaction decoding for the
      deterministic `STR-004` surface.
- [x] Add pinned golden vectors and focused malformed/truncation coverage.
- [x] Preserve payout-address codecs, user-payout matching, live mining, share,
      ASIC, networking, credential, and hardware behavior as explicit
      non-claims.
- [x] Run focused checks plus every mandatory Rust, Bright Builds, Bazel,
      redaction, parity, and progress gate.
- [x] Transition only `STR-004` when its exact `unit,golden` evidence passes,
      synchronize progress, and archive this task in the finalization commit.

Plan: `docs/parity/work-plans/20260802T201136Z-STR-004/PLAN.md`

Authorization and safety: software-only and effect-free. No credentials,
hardware, flashing, pool/network connections, destructive actions, direct
UART, pin manipulation, or reference-tree edits were used.

Verification: Eleven focused decoder tests, the Stratum Bazel target, fixture
JSON parsing, reference cleanliness, formatting, strict Clippy, the
all-target/all-feature Cargo build and tests, managed Bright Builds checks,
redaction verification, the full Bazel test graph, parity validation, and the
30-of-94 pre-transition progress baseline passed. The terminal result binds
implementation commit `b55228706d28f9b34d71a092656ef3ca6f3f649a`;
transition receipt `20260802T202626Z-STR-004` binds the predecessor/result
checklist digests, plan/result hashes, pinned reference commit, exact Rust-owned
targets, and unchanged `unit,golden` evidence. Progress sync reports 31/94
verified (33.0%). Final gates are run again after this archival and before the
final commit.

Completion review: Complete. The typed decoder now covers all CompactSize
widths, BIP-34 height and printable pool-tag extraction, compact-target network
difficulty, bounded output retention with complete totals, every reference
script shape, and exact BIP-54/BIP-110 decisions, with malformed and truncated
input rejected. `STR-004` alone transitioned from `implemented` to `verified`.
Residual risk: address encoding and payout matching remain owned by unverified
`STR-012`; live sockets, networking, TLS, credentials, ASIC dispatch, shares,
production mining, timing, and hardware remain explicit non-claims. The
workflow's final push follows the finalization commit and is intentionally not
represented as historical evidence inside this archived pre-push task record.

### task-parity-stat-004-work-queue-verification | 2026-08-02 | Verify deterministic work-queue parity

- [x] Add pinned executable golden cases for the exact `STAT-004` queue
      data-structure contract.
- [x] Cover capacity, FIFO wrap-around, full/empty boundary preservation, and
      clear/drop ownership with focused Arrange/Act/Assert tests.
- [x] Preserve task blocking/timing, live Stratum, ASIC, share, credential,
      network, and hardware behavior as explicit non-claims.
- [x] Run focused checks plus every mandatory Rust, Bright Builds, Bazel,
      redaction, parity, and progress gate.
- [x] Transition only `STAT-004` if its exact `unit,golden` evidence passes,
      synchronize progress, and archive this task in the finalization commit.

Plan: `docs/parity/work-plans/20260802T214207Z-STAT-004/PLAN.md`

Authorization and safety: software-only and effect-free. No credentials,
hardware, flashing, pool/network connections, destructive actions, direct
UART, pin manipulation, or reference-tree edits were used.

Verification: Ten focused queue tests, the Stratum Bazel target, fixture JSON
parsing, reference cleanliness, formatting, strict Clippy, the
all-target/all-feature Cargo build and tests, managed Bright Builds checks,
redaction verification, all 82 Bazel tests, parity validation, and the 31-of-94
pre-transition progress baseline passed. The terminal result binds
implementation commit `8a89a7e50db2abeaba3f6cd5173c7536c0b72d9c`;
transition receipt `20260802T214944Z-STAT-004` binds the predecessor/result
checklist digests, plan/result hashes, pinned reference commit, exact Rust-owned
targets, and `unit,golden` evidence. Progress sync reports 32/94 verified
(34.0%). Final gates are run again after this archival and before the final
commit.

Completion review: Complete. The executable fixture and focused regressions
prove capacity twelve, initial empty state, FIFO ordering across storage reuse,
unchanged state at full and empty boundaries, and deterministic drop-on-clear.
`STAT-004` alone transitioned from `implemented` to `verified`. Residual risk:
condition-variable waits, timeout-clock behavior, thread/FreeRTOS wakeup
ordering, and task scheduling remain owned by `SYS-005`; live Stratum, pool,
credential, ASIC, share, mining, and hardware behavior remain explicit
non-claims. The workflow's final push follows the finalization commit and is
intentionally not represented as historical evidence inside this archived
pre-push task record.

### task-parity-ota-001-timeout-root-cause | 2026-08-02 | Diagnose and fix the OTA valid-upload timeout boundary

- [x] Build a deterministic, fast reproduction of the exact valid-upload
      timeout with HTTP `000`, zero response bytes, and curl status `28`.
- [x] Test ranked falsifiable hypotheses against the protected attempt facts
      without printing or committing operational values.
- [x] Add a regression at the real Phase 13 orchestration seam and implement
      the narrowest root-cause fix.
- [x] Run focused OTA checks plus all repository-required Rust, Bright Builds,
      Bazel, parity, progress, redaction, reference-cleanliness, and diff gates.

Scope and authorization: this task was software-only. It inspected the existing
ignored `target/advance-parity-ota001/` artifacts through redacted classifiers
without printing, copying, committing, or summarizing raw device, network,
serial, HTTP, Wi-Fi, or USB values. It did not renew the consumed OTA attempt or
perform another flash, upload, reboot, rollback, recovery, OTAWWW, mining,
hardware-control, direct-UART, or pin-manipulation action. `OTA-001` remains
`implemented`; no checklist transition or progress sync was performed.

Verification: The deterministic valid-upload reproducer failed at the former
30-second boundary with curl status 28, HTTP `000`, and zero response bytes on
three consecutive pre-fix runs. Moving only that deadline beyond the simulated
minimum made the same loop pass. A separate red/green contract proved the
prearmed capture now selects the qualified OS-native runtime reader. Shell
syntax, both focused OTA Bazel tests, formatting, strict Clippy, all-target and
all-feature Cargo build and tests, managed Bright Builds checks, all 82 Bazel
tests, parity validation, the unchanged 32-of-94 progress report, redaction,
reference cleanliness, and diff checks passed.

Completion review: Complete. The proven immediate cause was the host helper's
fixed 30-second valid-upload deadline: it exactly produced the observed curl
status 28, HTTP `000`, and empty response, and the current image is about 28.5%
larger than the earlier image that completed within that same budget. The
secondary evidence defect was the helper's implicit `espflash` reader despite
the repository's qualified OS-native runtime-reader requirement, so absent
post-upload reboot markers could not establish firmware failure. The targeted
fix gives valid uploads a bounded 120-second default, preserves the invalid
request's 30-second bound, gives post-upload observation a 360-second default,
keeps the prearmed capture alive for the sum of both budgets, and explicitly
uses the OS-native reader. Residual risk: because the original hardware attempt
was consumed and this task did not authorize another, device-side behavior
beyond the former client deadline remains unobserved and real-hardware OTA
confirmation is still required before `OTA-001` can transition.

### task-parity-ota-001-reboot-evidence | 2026-08-02 | Close firmware OTA reboot evidence

- [x] Fix the OTA smoke helper's post-response monitor-attachment race and add
      deterministic monitor-order/cleanup regressions.
- [x] Pass focused checks plus every mandatory Rust, Bright Builds, Bazel,
      redaction, parity, and progress gate.
- [x] Commit the software implementation before the hardware attempt, then
      build/package/flash that exact clean commit.
- [x] Run the exact one-attempt hardware contract from
      `docs/parity/work-plans/20260802T215555Z-OTA-001/PLAN.md` and retain raw
      device/network/serial/HTTP evidence only under ignored
      `target/advance-parity-ota001/`.
- [x] Transition only `OTA-001` if current package admission, invalid rejection,
      valid upload, reboot identity, safe-state, boot validation, cleanup, and
      privacy all pass; otherwise record the exact terminal stop category.

Plan: `docs/parity/work-plans/20260802T215555Z-OTA-001/PLAN.md`

Authorization and safety: the user explicitly authorized hardware interactions
for this continuing goal. The only effectful commands permitted are the exact
detector, package, flash-monitor, one OTA smoke invocation, cleanup detector,
and conditional single recovery flash/check recorded in the plan. No erase,
rollback fault injection, interrupted update, OTAWWW, mining, pool access,
voltage/fan/power actuation, direct UART, or pin manipulation is authorized.
The authorization expires when this one OTA attempt reaches a terminal outcome.

Evidence and privacy: raw flash evidence, device URL, network values, IP/MAC
values, serial logs, and HTTP artifacts stay in ignored
`target/advance-parity-ota001/`. Committed evidence may contain only redacted
category labels, public repo paths, source/reference commits, artifact digests,
HTTP status/body markers, and conclusions. Recovery is limited to one current-
package wrapper flash only if the cleanup detector fails; no second OTA attempt
is allowed. Stop on any detector, target-lock, manifest, identity, marker,
privacy, or cleanup failure using the plan's exact terminal categories.

Verification: `stop_hardware_blocker`. Implementation commit
`afb73fba3b34f4b43250d503d574c92c258f9606` passed the complete software gate,
packaged with the same source identity and pinned reference, and produced the
manifest-admitted `esp-miner.bin` digest
`0dab8e06f08f566a898c8f4b07f315a8a7e8e2d2fd961deb3ad9c2177bdaad7c`.
The preflight detector and wrapper flash-monitor passed with trusted output.
The one OTA invocation captured invalid-image HTTP 500 rejection and proved
monitor readiness before the valid upload, but that upload ended with curl
status 28 and HTTP 000 after 30 seconds with zero response bytes. Firmware,
reference, boot-validation, and safe-state reboot markers were therefore absent.
The cleanup detector passed on the same board/port; no recovery flash ran. Raw
evidence remains ignored and the redacted evidence boundary passed.

Completion review: Closed at `stop_hardware_blocker`. At that time, `OTA-001`
remained `implemented`; no checklist transition or progress sync occurred. The
exact one-attempt authorization was consumed, and no retry was permitted under
this task. The helper closed the original monitor-attachment race, but the
device/network session did not return a valid-upload response inside the fixed
30-second HTTP window. Rollback, destructive/fault-injection recovery,
selected-partition, OTAWWW, mining, and hardware-control behavior remained
non-claims. This terminal record is archived only because the user later issued
fresh authorization and `task-parity-ota-001-bounded-retry` independently
resolved the blocker without altering this attempt's outcome.

### task-parity-ota-001-bounded-retry | 2026-08-02 | Retry OTA reboot evidence after timeout fix

- [x] Commit the fresh authorization and detector-only Phase A contract before
      any hardware interaction.
- [x] Run exactly one `just detect-ultra205`; require one Ultra 205 and bind its
      qualified port into a committed Phase B contract before flashing.
- [x] Build/package and wrapper flash-monitor the exact clean Phase B commit,
      using the ignored local Wi-Fi credential input without reading it.
- [x] Admit exactly one same-session origin and run exactly one bounded
      invalid-plus-valid OTA invocation with the fixed timeout and qualified
      OS-native post-OTA reader.
- [x] Run the cleanup detector and only the contract's single conditional
      recovery flash/check if cleanup fails.
- [x] Record a redacted terminal result, run repository gates, and transition
      only `OTA-001` if every promotion criterion passes.

Plan: `docs/parity/work-plans/20260802T223139Z-OTA-001-RETRY/PLAN.md`

Authorization and safety: the user explicitly authorized one new hardware
attempt on 2026-08-02. Phase A authorized only the exact read-only detector
command recorded in the plan. Flash, OTA, and recovery remained prohibited
until the freshly selected port and exact Phase B commands were committed. The
attempt budget was exactly one invalid-plus-valid OTA invocation, with no OTA
retry. No erase, rollback fault injection, interrupted update, OTAWWW, mining,
pool access, voltage/fan/power actuation, direct UART, or pin manipulation was
authorized or performed.

Evidence and privacy: raw USB, serial, network, HTTP, origin, IP/MAC, and Wi-Fi
material remains in the plan's ignored `target/` roots. The credential file was
passed only to the repo-owned flash wrapper and was not read, printed,
summarized, or committed. Committed output is limited to public source/reference
identities, artifact digests, HTTP status/body categories, redacted marker
categories, and conclusions.

Recovery and stop policy: the cleanup detector passed on the same qualified
target, so the conditional recovery flash was prohibited and did not run. The
single attempt budget is consumed; no second OTA invocation is permitted.

Verification: Implementation/package commit
`2541818aa23120dd85c711386efadb69a1415ad3` passed source/reference/digest
admission. The attempt captured invalid HTTP 500 plus `Write Error`, valid HTTP
200 plus `Firmware update complete, rebooting now!`, zero curl statuses, exact
post-reboot identities, fail-closed safe state,
`ota_boot_validation=complete`, `ota_boot_validation=marked_valid`, qualified
OS-native passive capture, and successful cleanup. Focused OTA tests, formatting,
strict Clippy, all-target/all-feature Cargo build and tests, managed Bright
Builds checks, all 82 Bazel tests, parity validation, redaction, reference
cleanliness, and diff checks passed. Transition receipt
`20260802T230503Z-OTA-001` binds the plan, result, predecessor/result checklist
digests, reference commit, targets, status, and evidence. Progress synchronization
reports 33 of 94 active rows verified (35.1%).

Completion review: Complete. `OTA-001` alone transitioned from `implemented` to
`verified` for current-package firmware OTA observable behavior. The successful
attempt consumed the fresh authorization and needed no recovery. Selected
partition internals, rollback, destructive or interrupted-update recovery,
OTAWWW, network longevity, mining, pool access, active voltage/fan/power
behavior, other boards, direct UART, and pin manipulation remain explicit
non-claims.

### task-parity-transition-notes-projection | 2026-08-02 | Bind verified notes into transitions

- [x] Reproduce the validator failure where a verified transition preserves
      stale blocker notes.
- [x] Extend transition receipts with an optional hash-bound before/after notes
      projection while preserving every existing receipt.
- [x] Add focused tests for note projection and legacy receipt compatibility.
- [x] Roll back only the uncommitted failed OTA transition artifacts, rerun the
      corrected `OTA-001` transition, and synchronize progress.
- [x] Pass every mandatory Rust, Bright Builds, Bazel, parity, progress,
      redaction, reference-cleanliness, and diff gate.

Scope: software-only finalization repair. The hardware attempt was complete and
its budget consumed; this task performed no detector, flash, monitor, HTTP, OTA,
recovery, credential, network, direct-UART, or pin action.

Verification: The original transition projection deterministically failed the
verified-row blocker-note guard. Seven focused transition tests pass, including
hash-bound note replacement, incomplete-binding rejection, and validation of
the legacy receipt shape. Strict focused Clippy passes. The corrected receipt
`20260802T230503Z-OTA-001` contains exact before/after notes, passes the required
OTA evidence vocabulary, and preserves every prior receipt. Formatting, strict
workspace Clippy, all-target/all-feature Cargo build and tests, managed Bright
Builds checks, all 82 Bazel tests, parity validation, progress, redaction,
reference cleanliness, and diff checks pass.

Completion review: Complete. Transition receipts now optionally project notes
in the same predecessor/result hash chain as target, status, and evidence while
legacy receipts deserialize and validate unchanged. Empty, multiline,
pipe-containing, mismatched, and one-sided notes fail closed. This allowed the
`OTA-001` transition to replace its obsolete blocker note atomically rather than
publishing a contradictory verified row. No other checklist row changed.

### task-parity-v12-package-identity-205 | 2026-08-02 | Verify exact package and runtime identity on Ultra 205

- [x] Confirm the immutable bounded OTA evidence binds one clean package source
      commit, the pinned reference, the admitted package artifact, and exact
      post-reboot runtime identities on board 205.
- [x] Run focused package-manifest and runtime-attestation regressions plus all
      mandatory repository gates.
- [x] Commit a row-specific result before transitioning only
      `V12-PACKAGE-IDENTITY-205` to `verified`.
- [x] Correct the stale Phase 35 contract to validate its immutable Phase 36
      successor snapshot rather than pinning the mutable current checklist.
- [x] Synchronize parity progress, record the completion review, and archive
      this task in the same finalization commit.

Plan:
`docs/parity/work-plans/20260802T231836Z-V12-PACKAGE-IDENTITY-205/PLAN.md`

Authorization and evidence boundary: this task was evidence-only. It read only
committed redacted result and receipt artifacts and ran software verification.
It performed no detector, flash, reset, OTA, HTTP, serial, mining, network,
credential, voltage, fan, power, direct-UART, or pin effect. Raw hardware
evidence remained private under ignored roots and was not copied.

Verification: Evidence/source commit
`3bc773550fab128d4323779f48a86a311389e03d` passed 11 focused runtime boot
attestation tests, 8 focused package-manifest tests, and the Bazel `bitaxe-api`
and `xtask` suites. The immutable result binds clean package commit
`2541818aa23120dd85c711386efadb69a1415ad3`, pinned reference
`c1915b0a63bfabebdb95a515cedfee05146c1d50`, package digests, one
detector-admitted Ultra 205, wrapper flash admission, and qualified passive
post-reboot observation of both exact runtime identities. The first final Bazel
run passed 81 of 82 targets and exposed one stale test assumption: the Phase 35
contract read the mutable current checklist while asserting the immutable Phase
36 successor state. The targeted snapshot-path correction passes its focused
Bazel regression. Transition receipt
`20260802T232729Z-V12-PACKAGE-IDENTITY-205` binds the selected row and
replacement notes; synchronized progress is 34 of 94 active rows verified
(36.2%). This record is committed only after the complete ordered Rust, Bright
Builds, Bazel, parity, progress, redaction, reference, and diff gates pass.

Completion review: Complete. Only `V12-PACKAGE-IDENTITY-205` transitioned from
`implemented` to `verified`, closing the stale
`runtime_identity_observation_insufficient` correction with exact current-package
hardware evidence. The historical contract now checks its digest-bound snapshot
instead of freezing the mutable tracker. Hostname, operator-snapshot,
runtime-health, partition, rollback, OTAWWW, network, mining, voltage/fan/power,
other-board, and release claims remain explicit non-claims. No new hardware
attempt was necessary.

### task-remove-legacy-phase-checks | 2026-08-02 | Remove four deletion-ready phase checks

- [x] Remove the isolated Phase 15 BM1366 diagnostic package binary/test and
      the self-contained Phase 13 HTTP/recovery plus Phase 29 redaction
      binary/test clusters.
- [x] Remove every corresponding Bazel target and confirm no active caller or
      build reference survives.
- [x] Preserve active Phase 14/20 safety policy, Phase 15 mining-allow,
      Phase 17/23/33/35/36 evidence, and Phase 28/30 safety guard surfaces.
- [x] Run focused graph checks and the complete mandatory repository gates.

Plan: Delete only the four `sh_test` targets, their four orphaned `sh_binary`
targets, and the eight owned scripts proven to have no current caller. Keep
historical parity evidence immutable and make no checklist status change.

Verification: Exact-name scans found no surviving active source reference.
The Bazel graph contains none of the eight removed targets and retains sampled
Phase 14/20 safety, Phase 15 mining-allow, Phase 17/23/33/35/36 evidence, and
Phase 28/30 guard targets. Query-visible individual test rules decreased from
82 to 78. The ordered Rust format, warning-denied Clippy, all-target/all-feature
build, and all-feature test sequence passed. Bright Builds reported zero
findings; all 79 Bazel-reported test targets passed; parity validation reported
no errors; progress remained 34/94 (36.2%); and redaction, reference integrity,
plus diff checks passed.

Completion review: Complete. Four obsolete phase checks, four orphaned binary
targets, and eight owned scripts were removed together, deleting 2,352 lines
without moving their complexity to another caller. Active safety, evidence,
mining-allow, and preserved UART/no-promotion implementation remained intact.
Historical parity evidence was not edited, no checklist or progress artifact
changed, and no hardware interaction occurred.

### task-typed-automation-cutover | 2026-08-03 | Replace active ad-hoc automation with typed TypeScript

- [x] Establish the hermetic Node/TypeScript Bazel toolchain and Rust-owned
      command/result contract generator.
- [x] Replace or delete every reachable non-preserved Bash, MJS, and Python
      automation surface without compatibility aliases.
- [x] Move active operator workflows behind one semantic
      `bitaxe-automation` CLI with canonical `--flag value` inputs and typed
      JSON result envelopes.
- [x] Replace stringly process and safety-policy seams with generated command
      builders, structured workflow identity, and one audited process adapter.
- [x] Add the semantic evidence schemas and an auditable migration ledger;
      preserve prior verified claims only where equivalence is proved.
- [x] Run the ordered Rust, Bazel, Bright Builds, parity, redaction, reference,
      source-policy, and regression gates.
- [x] Commit and push the atomic cutover, then execute exactly one task-gated
      safe-package hardware smoke and record the redacted conclusion.

Scope contract: the active migration candidates are every repo-owned `.sh`,
`.mjs`, and active helper `.py` under `scripts/` except the upstream-managed
Bright Builds updater, all terminal Phase 28.1.1 descendant artifacts, and the
dormant late-attach/external-UART implementation and tests required to preserve
that terminal history. The cutover must leave no active compatibility wrapper,
phase-numbered command alias, `key=value`/underscore CLI alias, or raw
`node:child_process` call outside the one process adapter.

Hardware contract:

- Preconditions: the cutover commit is clean and pushed; `test -s
  wifi-credentials.json` succeeds without reading or printing the file; the
  exact package is freshly built with `bazel build
  //firmware/bitaxe:firmware_image`; `scratch/automation-refactor/attempt-001`
  and the shareable projection are absent; and `just detect-ultra205` admits
  exactly one board `205` through successful ESP32-S3 board-info.
- Private evidence: `scratch/automation-refactor` is mode `0700`, the fresh
  `attempt-001` child is created mode `0700`, and all private detector,
  command, serial, HTTP, and process artifacts are mode `0600`. Raw port,
  origin, network, Wi-Fi, device identity, and process values remain private.
- Sole launch command: `just capture-version-evidence --private-root
  scratch/automation-refactor/attempt-001 --package-manifest
  bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials
  wifi-credentials.json --port <detector-port> --projection
  docs/parity/evidence/automation-refactor/attempt-001/version-evidence.json
  --capture-timeout-seconds 120`.
- Allowed effect: one exact-package flash followed by bounded passive boot and
  same-session origin-only `/api/system/info` observation. Mining and hardware
  control must remain disabled. The only shareable output is the redacted
  `bitaxe-version-evidence-v1` projection.
- Recovery: only after the sole launch confirms that a flash effect occurred
  and then fails, at most one recovery flash of the same manifest is allowed:
  `just flash --board 205 --port <detector-port> --manifest
  bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials
  wifi-credentials.json`. Recovery is not a retry and creates no parity claim.
- Bounds and stop categories: exactly one capture launch, zero automatic or
  manual relaunches, and at most one same-package recovery after confirmed
  effect. Stop immediately on `hardware_blocked`, `authorization_blocked`,
  `contract_mismatch`, `evidence_invalid`, `process_failed`, or `timeout`.
  Any failed launch leaves the task incomplete and requires a new task-scoped
  contract before another attempt.
- Forbidden: pool input, mining, voltage/frequency/fan actuation, erase, OTA,
  fault injection, direct UART, pin work, network discovery, stale origin use,
  or evidence containing protected operational values.

Verification: `cargo fmt --all`; warnings-as-errors Clippy; all-target/all-feature
Cargo build; full all-feature Cargo tests; `bazel build //...`; all 28 Bazel
tests; Bright Builds; parity and progress consistency; semantic redaction;
pinned-reference integrity; contract drift, negative compile, process-adapter,
repository-guard, and forbidden-surface scans all pass. Hardware acceptance is
pending a fresh successful attempt.

Attempt 001 closure: detector admission passed and the sole launch flashed the
exact pushed package. Private evidence classified the flash and runtime
attestation as trusted and observed the exact source/reference identities with
mining, work submission, and hardware control disabled. The shareable
projection was not produced because the TypeScript validator required the
earlier boot-only `safe_state:` line even though the late-attached trusted
runtime attestation carried the same closed safety facts. No recovery was
needed, no retry occurred, and Attempt 001 is closed.

Fresh Attempt 002 contract:

- Preconditions: commit and push the targeted safe-state parser fix and its
  regression test; rerun every ordered software gate; build that exact clean
  package; require `scratch/automation-refactor/attempt-002` and
  `docs/parity/evidence/automation-refactor/attempt-002/version-evidence.json`
  to be absent; check the ignored Wi-Fi file with `test -s` only; and freshly
  admit exactly one Ultra 205 with `just detect-ultra205`.
- Private evidence: the existing mode-`0700`
  `scratch/automation-refactor` parent receives a fresh mode-`0700`
  `attempt-002` child and mode-`0600` artifacts. Protected values remain local.
- Sole launch command: `just capture-version-evidence --private-root
  scratch/automation-refactor/attempt-002 --package-manifest
  bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials
  wifi-credentials.json --port <detector-port> --projection
  docs/parity/evidence/automation-refactor/attempt-002/version-evidence.json
  --capture-timeout-seconds 120`.
- Allowed effect and evidence: one exact-package safe flash, bounded passive
  boot/runtime attestation, and same-session origin-only `/api/system/info`
  observation. Mining, work submission, and hardware control remain disabled;
  only the validated redacted `bitaxe-version-evidence-v1` projection may be
  shared.
- Recovery, bounds, stops, and forbidden actions are identical to Attempt 001:
  zero relaunches, at most one same-package recovery after confirmed effect,
  and immediate stop on any closed failure category. A failed Attempt 002
  requires another targeted diagnosis and a new contract.

Completion review: The active ad-hoc automation surface was replaced by the
strict Bazel-built TypeScript module and Rust-owned generated contracts; 101
legacy script entrypoints and their Phase 36 compatibility graph were removed.
Attempt 001 exposed and reproduced the late-attach safe-state parser defect
without retrying. Attempt 002 then flashed exact clean pushed commit
`9d7a891d14cfa634fffc1aa50fb9ddd5f7d4da17` once and produced the Rust-validated,
redaction-passed `bitaxe-version-evidence-v1` projection with boot and same-origin
API observation while mining and hardware control stayed disabled. The final
redaction false positive for the schema-owned boolean
`same_origin_api_observed` is regression-tested. Residual risks are the explicit
macOS-only production session, inert preserved Phase 28/UART history, and the
three parity rows deliberately downgraded until fresh semantic evidence exists.

### task-evidence-redaction-ci-repair | 2026-08-03 | Restore semantic redaction CI

- [x] Make the host-only Bazel workspace-status command select the stable Rust
      toolchain explicitly instead of inheriting the firmware `esp` override.
- [x] Update the GitHub workflow to invoke bare semantic
      `just verify-redaction` without deleted revision flags or aliases.
- [x] Add regression coverage for the deployed workflow/config contract and
      continued rejection of the removed flags.
- [x] Reproduce both original boundaries, run every mandatory software gate,
      and require both GitHub workflows to pass on the exact pushed commit.

Verification: The missing-default-toolchain reproduction passes with
`RUSTUP_TOOLCHAIN=bitaxe-intentionally-missing`, while the removed revision
flags still fail with exit code 2. The focused automation test, ordered Cargo
format/Clippy/all-target build/all-feature tests, `bazel build //...`, all 28
Bazel tests, Bright Builds checks, parity validation, progress consistency,
semantic redaction, reference integrity, actionlint, source scans, and diff
checks pass locally. On exact pushed commit
`8ab59a6f7228bf18a2c46536df8f889d307c98ef`, Bright Builds run `30858292767`
and Evidence redaction run `30858292790` both completed successfully.

Completion review: Complete. The software-only redaction job now selects the
stable host Rust toolchain explicitly and invokes only the canonical semantic
CLI. The ESP firmware override remains unchanged, deleted flags remain
unsupported, and no hardware, evidence, parity status, or compatibility layer
changed. Residual risk is limited to the GitHub-hosted runner continuing to
provide its documented stable Rust installation.

### task-standing-task-authorization | 2026-08-03 | Remove repeated task and attempt confirmation gates

- [x] Record the user correction as a durable lesson and run the triggered
      lesson-ledger audit.
- [x] Give active and future repository tasks standing execution authorization,
      including autonomous selection of fresh hardware-attempt ordinals after
      verified progress.
- [x] Keep detector, exact-command, safety, privacy, evidence, recovery, retry,
      terminal-stop, direct-UART/pin, and destructive/fault-injection gates
      intact.
- [x] Make future tasks ordinary automatic queue candidates instead of
      requiring an exact-ID user request.
- [x] Verify, commit, push, and archive this governance task before the next
      effectful hardware attempt.

Plan: update only repo-owned guidance, the hardware-attempt policy, the local
parity skill, tracker semantics, and the current SYS-004 continuation contract.
Managed Bright Builds files remain unchanged. Then run the complete repository
verification required for the changed policy surfaces, publish the checkpoint,
and resume parity work without another authorization prompt.

Authorization decision: the user's 2026-08-03 correction grants standing
authorization to work every repository task. Ordinary task selection, future
task promotion, USB interaction, and fresh progress-backed attempt ordinals no
longer require per-task or per-attempt confirmation. Materially different
direct-UART/pin manipulation and ad hoc destructive or fault-injection actions
retain their specific safety gates.

Verification: `cargo fmt --all`, warning-denied all-target/all-feature Clippy,
the all-target/all-feature Cargo build, all-feature Cargo tests, all 28 Bazel
tests, Bright Builds with zero findings, parity validation, progress
consistency, semantic redaction, pinned-reference integrity, and diff checks
passed in order. The lesson audit retained all 29 active lessons without unsafe
consolidation or archival.

Completion review: Complete. Active and future tasks now carry standing
execution authorization, and progress-backed fresh hardware ordinals no longer
manufacture user-confirmation blockers. The exact task contract remains the
effect boundary; no unchanged blind retry, weakened evidence, direct-UART/pin
permission, or ad hoc destructive/fault-injection authority was introduced.
The open SYS-004 task now contains its complete standing-authorized Attempt-006
contract. No hardware effect occurred during this governance change.

### task-parity-sys004-version-reporting | 2026-08-02 | Reconcile implemented version reporting

- [x] Audit the upstream firmware, AxeOS, and ESP-IDF version surface against
      the current canonical Rust build/platform/API projection.
- [x] Run focused build-identity, system-info wire, package-manifest, and
      runtime-attestation regressions plus every mandatory repository gate.
- [x] Commit the row evidence before transitioning only `SYS-004` from
      `in-progress` to `implemented`.
- [x] Synchronize parity progress and retain this task with the exact later
      live-evidence gate required for `verified`.
- [x] Correct `next-item` so a non-verified plan closes after its checklist
      status advances beyond the plan's recorded initial status.
- [x] Prove an unchanged-status plan still resumes while the completed
      `SYS-004` implementation plan yields the next candidate queue.
- [x] Run all mandatory repository gates, keep the checklist and progress
      history unchanged, and push the audited continuation.
- [x] Generate the canonical build label as package-owned SPIFFS
      `version.txt` and read that installed file for `axeOSVersion`.
- [x] Add a typed exact-package live version projection with focused
      regression coverage and a closed commit-safe schema.
- [x] Commit and push the software fix, build the exact package, and perform at
      most one detector-gated Ultra 205 verification attempt.
- [x] Verify or conservatively retain only `SYS-004`, synchronize progress only
      if its checklist fields change, and push the audited result.
- [x] Extend the typed version-evidence workflow with exact-package HTTP and
      identical same-boot/revision WebSocket comparisons while keeping raw
      device responses private.
- [x] Run every mandatory software gate, commit and push the workflow extension,
      then execute the detector-gated Attempt-006 contract exactly once.
- [x] Verify or conservatively retain only `SYS-004`, synchronize progress only
      if its checklist fields change, archive this task only after verified
      completion, and continue automatic parity selection.

Plan: `docs/parity/work-plans/20260802T233821Z-SYS-004/PLAN.md`

Verification plan:
`docs/parity/work-plans/20260803T001834Z-SYS-004/PLAN.md`

Initial-plan authorization boundary: software-only evidence reconciliation. No detector,
hardware, credential, network, flash, monitor, HTTP, WebSocket, OTA, mining,
safety-control, direct-UART, or pin action is authorized.

Verification: The focused build-identity (8), system-info wire (1), package
manifest (8), and runtime-attestation (11) tests passed; the focused Bazel
targets passed; API compare checked 99 schema, 47 captured-response, and 36
static-route facts without validation errors; and the complete Rust, Bright
Builds, Bazel, parity, progress, redaction, reference-integrity, and whitespace
gates passed before the evidence commit.

Completion review: The stale checklist state was caused by evidence-accounting
drift, not a missing firmware path. The existing canonical version projection
passed focused and repository-wide checks, its evidence was committed before
the guarded one-row transition, and progress was hash-chain synchronized.
`SYS-004` is now accurately `implemented`. Residual risk is explicit:
exact-current-package live API version evidence and a decision on static-asset
version semantics remain required before `verified`, so this task stays active
and unarchived.

Continuation: The next `advance-parity` preflight incorrectly reopened this
completed implementation plan because `next-item` treats every plan without a
`RESULT.md` as open. `RESULT.md` is reserved for verified completion, so that
rule deadlocks every intentionally conservative transition to `implemented`.
This software-only continuation will make plan openness depend on whether the
authoritative checklist status still equals the plan's recorded initial status.
It does not alter `SYS-004` evidence, authorize hardware, or weaken the later
verification gate.

Continuation verification: Focused isolated-target Cargo tests passed all seven
`parity_work` cases, including unchanged-status resumption, non-verified status
advance closure, and regression rejection. The Bazel parity test target passed.
Running the newly built CLI against the real repository returned
`maybe_open_plan: null` and restored the ordered candidate queue with `SYS-004`
first at `implemented`; the checklist and progress history remain unchanged.

Continuation completion review: Commit
`2ba235e661a628a66be035f329e75d1de82da80f` contains the targeted selector fix.
All Rust, Bazel, Bright Builds, parity, redaction, and reference-integrity gates
passed. The ordinary macOS Cargo target again stalled while launching the newly
linked parity test binary, so the complete Cargo suite was rerun successfully
from the clean isolated target that had already crossed that host boundary.
No checklist transition, progress-history append, `RESULT.md`, or hardware
interaction occurred. Residual risk is limited to future plan metadata/schema
evolution; missing, invalid, and regressed states fail closed.

Verification continuation: Source inspection resolved the static-semantics
question as a concrete defect. Upstream reports the version stored in the
flashed static filesystem and compares it with the application version. Rust
currently reports the generic checked-in fallback UI name, and its SPIFFS
package contains no `version.txt`. The new immutable verification plan permits
the minimum package/runtime repair, a typed version-only evidence projection,
and one exact-package Phase 36 broker attempt after the software source is
clean, fully verified, committed, and pushed.

Hardware contract: only the five exact commands and paths listed in the
verification plan are permitted. They allow package creation, two detector
admissions (one explicit and one broker-owned), one exact-package factory
flash with local Wi-Fi NVS input, passive receive-only serial capture,
read-only same-origin HTTP/WebSocket observation, typed same-package recovery
if required, cleanup, and a redaction-safe version projection. The private
root is `scratch/sys004-version-reporting/attempt-001`; it must be mode `0700`
with mode-`0600` artifacts. The Wi-Fi file contents, USB identity, device
origin, IP/MAC/SSID, raw response, and operational paths are never printed,
summarized, or committed.

Safety, recovery, and stop contract: board 205 only; 360-second capture and
420-second effect wall-clock bounds; safe boot with mining, work submission,
and hardware control disabled; no pool input or mining/control action; and no
manual reset, erase, raw write, OTA, discovery, fault injection, direct UART,
or pin access. The broker preserves the earliest typed failure, permits only
its same-package typed recovery after a confirmed flash effect, and proves
cleanup. One attempt only, with no unchanged retry. The accepted terminal
outcomes are `complete`, `stop_repeated_boundary`, `stop_hardware_blocker`,
`stop_authority_boundary`, or `stop_impossible_contract`.

Implementation continuation: The package now copies the checked-in static
tree into an ephemeral staging directory, writes the exact canonical build
label to `version.txt`, and feeds only that staged tree to ESP-IDF
`spiffsgen.py`. The firmware reads `/www/version.txt` after the existing SPIFFS
mount and classifies absent, malformed, or noncanonical content as explicitly
unavailable. The new `project-sys004-version-evidence` classifier requires the
mode-`0600` Phase 36 handle, capture, and eligible seal; revalidates exact
manifest, package, source, reference, ELF, capability, board, ASIC, and
same-device joins; requires identical HTTP/WebSocket JSON; and emits only the
closed commit-safe version/provenance projection. Focused API/parser,
packager, source-boundary, projection, stale-version, and private-mode tests
pass. The remaining unchecked work is the mandatory full gate, clean software
commit/push, and single hardware attempt.

Attempt-001 outcome: clean source `0a4475f232cc7d944e69c6425955994bbfc12a9e`
was packaged and the standalone detector passed. The broker admitted the exact
package, but its internal board-205 detector failed before credential access or
flash, then cleanup passed. The sealed categorical record reports
`detector_failed`, `recovery_disposition: not_authorized`, no secondary
failure, no candidate, and no private capture. Root cause is deterministic:
the canonical `tools/flash detect` output uses `port: <value>`, while the
broker accepts only the nonexistent `port=<value>` spelling. This is a host
parser defect, not device evidence, and the attempt changed no device state.

Attempt-002 authorization: add a pure detector-output parser that accepts
exactly one canonical `port: ` line and rejects missing, duplicate, legacy
`port=`, empty, and invalid UTF-8 inputs. Run all mandatory gates, commit and
push the fix, and create a new clean exact package before the retry. Exactly
one retry is permitted at
`scratch/sys004-version-reporting/attempt-002` using the same 360-second
capture, 420-second effect, safety, privacy, recovery, cleanup, and stop
contract as attempt 001. The only permitted retry workflow is:

1. `just package`
2. `just detect-ultra205`
3. `just phase36-substantive-evidence mode=preflight board=205 private-parent=scratch/sys004-version-reporting/attempt-002 attempt-handle-file=scratch/sys004-version-reporting/attempt-002/handle.json candidate-output=scratch/sys004-version-reporting/attempt-002/candidate.json capture-timeout-seconds=360 package-manifest=bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json`
4. `just phase36-substantive-evidence mode=hardware board=205 private-parent=scratch/sys004-version-reporting/attempt-002 attempt-handle-file=scratch/sys004-version-reporting/attempt-002/handle.json candidate-output=scratch/sys004-version-reporting/attempt-002/candidate.json capture-timeout-seconds=360 wifi-credentials=wifi-credentials.json`
5. `bazel run //tools/parity:report -- project-sys004-version-evidence --private-parent scratch/sys004-version-reporting/attempt-002 --attempt-handle-file scratch/sys004-version-reporting/attempt-002/handle.json --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --output docs/parity/evidence/sys004-version-reporting/version-projection.json`

No unchanged attempt, alternate command, or later ordinal is authorized. A
retry may proceed only after the parser regression, all gates, and clean push
prove the objectively changed boundary.

Attempt-002 outcome: clean source `9f4d56700c42a318e1aef61ee99bffcaf06e4231`
was packaged; the standalone detector and exact-package preflight passed. The
broker again stopped before credential access or flash with the same sealed
categorical detector failure, then cleanup passed with no candidate or private
capture. The first repair proved the stdout grammar but exposed the remaining
invocation cause: the broker starts nested `just detect-ultra205` without
setting its working directory to Bazel's `BUILD_WORKSPACE_DIRECTORY`, so the
process cannot reliably resolve the repository Justfile from the runfiles
working directory. The targeted follow-up sets only that command working
directory and has a pure command-construction regression. A third attempt is
not authorized. `SYS-004` must remain `implemented` with exact-package live
version evidence pending until a future explicitly authorized ordinal can
exercise the corrected broker.

Attempt-003 authorization: on 2026-08-03 the user explicitly authorized
`SYS-004` attempt 003 after the clean pushed
`f369dbde0cc689b6dc8cd4c76b9fd4fe45d5ad71` workspace-directory repair. The
attempt may proceed only after this contract passes every mandatory gate and
is committed and pushed. The exact permitted workflow is:

1. `just package`
2. `just detect-ultra205`
3. `just phase36-substantive-evidence mode=preflight board=205 private-parent=scratch/sys004-version-reporting/attempt-003 attempt-handle-file=scratch/sys004-version-reporting/attempt-003/handle.json candidate-output=scratch/sys004-version-reporting/attempt-003/candidate.json capture-timeout-seconds=360 package-manifest=bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json`
4. `just phase36-substantive-evidence mode=hardware board=205 private-parent=scratch/sys004-version-reporting/attempt-003 attempt-handle-file=scratch/sys004-version-reporting/attempt-003/handle.json candidate-output=scratch/sys004-version-reporting/attempt-003/candidate.json capture-timeout-seconds=360 wifi-credentials=wifi-credentials.json`
5. `bazel run //tools/parity:report -- project-sys004-version-evidence --private-parent scratch/sys004-version-reporting/attempt-003 --attempt-handle-file scratch/sys004-version-reporting/attempt-003/handle.json --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --output docs/parity/evidence/sys004-version-reporting/version-projection.json`

Attempt-003 evidence and privacy contract: the ignored private parent must be
mode `0700`, all private artifacts must be mode `0600`, and the broker-owned
child must not exist before launch. Wi-Fi contents, USB identity, device
origin, IP/MAC/SSID, raw HTTP/WebSocket responses, and operational paths must
never be printed, summarized, or committed. Only the typed version projection
may enter `docs/parity/evidence/` after its private classifier and redaction
checks pass.

Attempt-003 effects and safety contract: exactly one detector-admitted Ultra
205 may receive the exact clean package, private Wi-Fi NVS input, the
broker-owned factory flash, qualified passive serial capture, and read-only
same-origin HTTP/WebSocket observation. Mining, pool input, work submission,
voltage, frequency, fan or other hardware control, OTA, OTAWWW, rollback,
erase-flash, arbitrary raw writes, discovery, fault injection, manual reset,
non-205 hardware, direct UART, and pin/pad/header/GPIO/probe/jumper/solder or
injected-signal access remain prohibited. The capture bound is 360 seconds and
the effect wall-clock bound is 420 seconds.

Attempt-003 recovery and stop contract: preserve the earliest typed failure;
permit only the broker's same-package typed recovery after a confirmed flash
effect; always prove cleanup; and do not infer recovery from elapsed time. This
is one fresh ordinal with no unchanged retry, alternate command, attempt 004,
or broader authority. A recurrence of the corrected detector boundary selects
`stop_repeated_boundary`. Accepted outcomes are `complete`,
`stop_repeated_boundary`, `stop_hardware_blocker`, `stop_authority_boundary`,
or `stop_impossible_contract`.

Attempt-003 promotion contract: `SYS-004` may reach `verified` only if the
detector, exact source/reference/manifest/ELF joins, safe boot, complete HTTP
response, identical same-boot WebSocket projection, manifest-equal `version`
and `axeOSVersion`, manifest-equal provenance and ESP-IDF fields, private
seals, commit-safe projection, cleanup, and every repository gate pass. Any
missing or contradictory fact retains only `SYS-004` at `implemented`, with no
progress append or `RESULT.md`.

Attempt-003 outcome: `stop_hardware_blocker` at a new pre-transfer
`flash_failed` boundary; no retry was run. Clean pushed source
`3793e6dcad0a814a4d5ebd94f75e2dd29eb76362` produced the exact package and
build label `3793e6dcad0a-dev`; standalone detection, exact-package preflight,
and the broker-owned detector all passed. The flash result is
`failed_no_device_effect`; cleanup completed; recovery was not authorized; and
no serial, HTTP, WebSocket, private capture, or candidate exists. The typed
projection rejected the absent private boundary. The mode-`0700` parent and all
eight mode-`0600` artifacts pass the private-mode contract.

Attempt-003 root cause: Phase 36 preflight deliberately resolves the factory
artifact through `realpath`, while `tools/flash` deliberately requires an
explicit image path to lexically equal the factory path resolved beside the
manifest. The handle's canonical execroot path and the manifest's `bazel-bin`
path identify the same file but are different strings, so the redundant
`--image` override is rejected before transfer or evidence-directory creation.
This exactly explains the detector-complete, no-stage, no-device-effect seal.
The targeted fix removes only that redundant explicit image override from the
Phase 36 adapter: the already admitted v3 manifest remains the sole flash-image
selector, while the broker independently retains and verifies the canonical
factory path and digest. A fresh-process fake-flash regression must prove the
adapter forwards the manifest and omits the image override without any device
effect. Attempt 004 remains unauthorized.

Attempt-003 software repair: the Phase 36 hardware-effect adapter now forwards
only the admitted v3 manifest to `tools/flash`; it no longer supplies the
redundant canonical `--image` spelling that the manifest boundary intentionally
rejects. The broker still verifies the canonical factory path, digest, package
identity, and exact source/reference joins before this adapter runs. A real
fresh-process fake-flash regression records the child argument vector and
proves exactly one manifest selector, no image override, a completed typed
effect result, and mode-`0600` output without touching hardware.

Attempt-003 completion review: the root-cause fix is minimal and preserves the
stricter flash-tool admission rule. The ordered Rust sequence, all 83 Bazel
tests, Bright Builds with zero findings, parity validation, unchanged 34/94
progress, redaction, reference integrity, and diff checks pass. `SYS-004`
remains conservatively `implemented`; the checklist and progress history are
unchanged, no `RESULT.md` exists, and this unresolved task remains active and
unarchived. Exact-current-package live HTTP/WebSocket equality is still
unproved. Attempt 003 is consumed and attempt 004 is not authorized, so the
next admissible hardware action requires a future explicit ordinal.

Attempt-003 software-only hardening continuation: on 2026-08-02 the user asked
for any further targeted fixes needed to get past the earlier Attempt-003
problems. This continuation authorizes deterministic fake-process replay and
software repair only. It does not authorize Attempt 004, USB detection,
credential use, flash, monitor, HTTP/WebSocket device access, recovery, direct
UART, pins, or any other hardware effect.

- [x] Prove a fast red-capable real-process replay of the exact redundant-image
      path-spelling failure and the current manifest-only green behavior.
- [x] Exercise the complete preflight-to-effect process seam with fakes and
      minimize any newly reproduced failure.
- [x] Implement only a reproduced root-cause fix with regression coverage; do
      not change hardware authority, promotion status, or evidence claims.
- [x] Run every mandatory repository gate and record the conservative outcome.

Attempt-003 hardening outcome: temporarily restoring the redundant
`--image` argument made the real adapter process test fail with
`redundant image override crossed the Phase 36 boundary`; restoring the
manifest-only adapter made the same test pass. A new fresh-process regression
now creates clean source and reference repositories, admits an exact v3
package through real preflight, transfers its canonical artifact identities
through the attempt handle and hardware wrapper, and invokes the real effect
adapter behind fake broker and flash boundaries. It proves exactly one
manifest selector, no image override, a completed typed result, a sealed
non-promotion outcome, and mode-`0600` private outputs without USB, credentials,
device access, or any other hardware effect.

No further production defect reproduced: runfiles resolution, canonical path
transfer, broker-side argument transfer, and private modes all passed. The
targeted fix is therefore the missing end-to-end regression seam, not another
runtime workaround. The ordered Rust format, clippy, build, and test sequence;
all 80 Bazel tests; Bright Builds; parity validation; unchanged 34/94 progress;
redaction; reference integrity; and diff checks pass. One first full-suite run
observed an unrelated archived Phase 28 socket-startup `ECONNREFUSED`; three
isolated reruns and the complete rerun passed, and the protected archived
source was not changed. `SYS-004` remains `implemented`, the checklist and
progress history remain unchanged, and Attempt 004 remains unauthorized.

Attempt-004 authorization: on 2026-08-03 the user explicitly authorized one
new `SYS-004` ordinal with a complete task-scoped hardware contract. The clean
pushed `3c471b28219df2554e2e5f1b575f8b5708c51d9d` source contains the
manifest-only adapter repair and a fresh-process preflight-to-effect regression.
Attempt 004 may proceed only after this contract passes every mandatory gate,
is committed and pushed, and a fresh exact-current-HEAD package is built. The
only permitted workflow is:

1. `just package`
2. `umask 077; mkdir -p scratch/sys004-version-reporting/attempt-004 && chmod 700 scratch/sys004-version-reporting/attempt-004 && just detect-ultra205 >scratch/sys004-version-reporting/attempt-004/standalone-detector.stdout 2>scratch/sys004-version-reporting/attempt-004/standalone-detector.stderr`
3. `just phase36-substantive-evidence mode=preflight board=205 private-parent=scratch/sys004-version-reporting/attempt-004 attempt-handle-file=scratch/sys004-version-reporting/attempt-004/handle.json candidate-output=scratch/sys004-version-reporting/attempt-004/candidate.json capture-timeout-seconds=360 package-manifest=bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json`
4. `just phase36-substantive-evidence mode=hardware board=205 private-parent=scratch/sys004-version-reporting/attempt-004 attempt-handle-file=scratch/sys004-version-reporting/attempt-004/handle.json candidate-output=scratch/sys004-version-reporting/attempt-004/candidate.json capture-timeout-seconds=360 wifi-credentials=wifi-credentials.json`
5. `bazel run //tools/parity:report -- project-sys004-version-evidence --private-parent scratch/sys004-version-reporting/attempt-004 --attempt-handle-file scratch/sys004-version-reporting/attempt-004/handle.json --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --output docs/parity/evidence/sys004-version-reporting/version-projection.json`

Attempt-004 evidence contract: the ignored private parent must be absent before
the standalone detector; command 2 creates it mode `0700` under `umask 077` and
captures the detector's distinct stdout and stderr as mode-`0600` regular
private artifacts. Preflight may reuse only that validated parent. The
broker-owned attempt child must be absent immediately before hardware launch.
Handle stdout and stderr remain distinct protected siblings.
Wi-Fi contents, USB identity or path, device origin, IP/MAC/SSID/hostname, raw
serial, HTTP or WebSocket material, process data, unredacted commands, and
operational paths must not reach inherited output, Git, or the typed projection.
Only closed shareable facts and public provenance may be summarized. The root
is sealed once and cannot be reused, rewritten, spliced, promoted directly, or
treated as deleted by cleanup.

Attempt-004 effects and safety contract: exactly one standalone-detector and
broker-detector admitted Ultra 205 may receive the exact clean v3 package,
private Wi-Fi NVS input, one broker-owned factory flash, qualified receive-only
serial observation, and read-only same-origin HTTP/WebSocket observation. Boot
must remain fail-closed with mining, ASIC work submission, and hardware control
disabled. Capture is bounded to 360 seconds and each device effect to 420
seconds. Pool input, mining, ASIC work, voltage, frequency, fan actuation,
thermal or power control, OTA, OTAWWW, rollback, erase-flash, arbitrary raw
writes, discovery, fault injection, manual reset, non-205 hardware, direct
UART, and pins, pads, headers, GPIO, probes, jumpers, soldering, or injected
signals remain prohibited.

Attempt-004 recovery, retry, and stop contract: preserve the earliest typed
failure through sealing; permit only broker-owned same-package typed recovery
after a confirmed flash effect; always close owned processes, serial holders,
and USB resources; and record cleanup separately without overwriting the first
failure. Exactly one Attempt-004 hardware launch is authorized. No unchanged
retry, alternate command, attempt 005, or broader diagnostic action is allowed.
Recurrence of the repaired manifest/image boundary selects
`stop_repeated_boundary`. Accepted outcomes are `complete`,
`stop_repeated_boundary`, `stop_hardware_blocker`, `stop_authority_boundary`,
or `stop_impossible_contract`.

Attempt-004 promotion contract: `SYS-004` may reach `verified` only when the
detector, exact source/reference/manifest/ELF/package joins, safe boot, complete
HTTP response, identical same-boot/revision WebSocket projection,
manifest-equal `version` and `axeOSVersion`, manifest-equal extended provenance,
manifest-equal ESP-IDF version, private modes and seals, commit-safe version
projection, cleanup, redaction, and every repository gate pass. Any missing,
contradictory, or unclassified fact leaves only `SYS-004` at `implemented`,
with no progress append, `RESULT.md`, task archival, or evidence promotion.

Attempt-004 software-only correction: the one authorized hardware launch is
consumed and no retry or new ordinal is authorized. Its closed seal records
`flash_failed`, `sealed_non_promotion`, successful cleanup, and no promotable
capture. The durable USB lifecycle's closed recovery fields prove that the
factory transfer completed before a legacy Phase 35 readiness boundary failed;
the wrapper nevertheless wrote `failed_no_device_effect`. The SYS-004
projector then reported the legitimately absent non-promotion capture as
`sys004_private_boundary_invalid` before reading the seal. This continuation
permits only the targeted software correction and fake/process regression work
below; it does not permit credentials, detector, USB, flash, serial, HTTP,
WebSocket, recovery, evidence replay, or sealed-root mutation.

- [x] Remove the Phase 36 effect adapter's obsolete dependence on Phase 35
      stage metrics and readiness environment.
- [x] Have the durable USB lifecycle expose a closed device-effect state and
      make `tools/flash` write the one typed Phase 36 effect result after
      operation plus cleanup complete.
- [x] Regress completed, partial, no-effect, parser, and cleanup-failure
      process boundaries without hardware.
- [x] Make SYS-004 projection classify a sealed non-promotion attempt before
      requiring eligible-only private capture artifacts.
- [x] Run every mandatory gate, preserve `SYS-004` as `implemented`, and push
      the truthful correction checkpoint without projecting Attempt 004.

Attempt-004 correction outcome: Phase 36 now derives its typed flash result
from the durable USB session's monotonic `none` / `confirmed_partial` /
`completed` effect state after operation and cleanup, with no Phase 35 stage
root, metrics, or readiness dependency. Fresh-process tests prove one manifest
selector, no image override, no legacy stage environment, and a mode-`0600`
typed result. Pure and broker tests preserve parser, no-effect, partial,
completed, cleanup-failure, identity, and recovery boundaries. SYS-004
projection now rejects a valid non-promotion seal as
`sys004_attempt_not_eligible` before looking for eligible-only capture files.
The ordered Rust checks, all 80 Bazel tests, Bright Builds, parity validation,
unchanged 34/94 progress, redaction, reference integrity, and diff checks pass.
`SYS-004` remains `implemented`; a new explicitly authorized ordinal with a
fresh complete hardware contract is required for live version promotion.

Attempt-005 authorization: on 2026-08-03 the user gave fresh explicit
authorization for one new `SYS-004` ordinal. The clean pushed
`d73d87064c44151b5b69ff6cac4b7066660b5f34` source contains the targeted
Attempt-004 corrections: Phase 36 owns its typed flash-effect result through
the durable USB lifecycle, and SYS-004 classifies a sealed non-promotion before
requiring eligible-only capture. Attempt 005 may proceed only after this
contract passes every mandatory gate, is committed and pushed, and a fresh
exact-current-HEAD package is built. The only permitted workflow is:

1. `just package`
2. `umask 077; mkdir -p scratch/sys004-version-reporting/attempt-005 && chmod 700 scratch/sys004-version-reporting/attempt-005 && just detect-ultra205 >scratch/sys004-version-reporting/attempt-005/standalone-detector.stdout 2>scratch/sys004-version-reporting/attempt-005/standalone-detector.stderr`
3. `just phase36-substantive-evidence mode=preflight board=205 private-parent=scratch/sys004-version-reporting/attempt-005 attempt-handle-file=scratch/sys004-version-reporting/attempt-005/handle.json candidate-output=scratch/sys004-version-reporting/attempt-005/candidate.json capture-timeout-seconds=360 package-manifest=bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json`
4. `just phase36-substantive-evidence mode=hardware board=205 private-parent=scratch/sys004-version-reporting/attempt-005 attempt-handle-file=scratch/sys004-version-reporting/attempt-005/handle.json candidate-output=scratch/sys004-version-reporting/attempt-005/candidate.json capture-timeout-seconds=360 wifi-credentials=wifi-credentials.json`
5. `bazel run //tools/parity:report -- project-sys004-version-evidence --private-parent scratch/sys004-version-reporting/attempt-005 --attempt-handle-file scratch/sys004-version-reporting/attempt-005/handle.json --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --output docs/parity/evidence/sys004-version-reporting/version-projection.json`

Attempt-005 evidence and privacy contract: the ignored private parent must be
absent before command 2. Command 2 creates it under `umask 077`, fixes it to
mode `0700`, and captures detector stdout and stderr as distinct mode-`0600`
regular private artifacts. Preflight may reuse only that validated parent. The
broker-owned attempt child must be absent immediately before command 4, and
the one opaque handle remains a mode-`0600` sibling outside that child. Wi-Fi
contents, USB identity or path, device origin, IP/MAC/SSID/hostname, raw serial,
HTTP or WebSocket material, process data, unredacted commands, and operational
paths must not reach inherited output, Git, or the typed projection. Only
closed shareable facts and public provenance may be summarized. The attempt
root and child are sealed once and cannot be reused, rewritten, spliced,
promoted directly, or treated as deleted by cleanup.

Attempt-005 effects and safety contract: exactly one standalone-detector and
broker-detector admitted Ultra 205 may receive the exact clean v3 package,
private Wi-Fi NVS input, one broker-owned factory flash, qualified receive-only
serial observation, and read-only same-origin HTTP/WebSocket observation. Boot
must remain fail-closed with mining, ASIC work submission, and hardware control
disabled. Capture is bounded to 360 seconds and each device effect to 420
seconds. Pool input, mining, ASIC work, voltage, frequency, fan actuation,
thermal or power control, OTA, OTAWWW, rollback, erase-flash, arbitrary raw
writes, discovery, fault injection, manual reset, non-205 hardware, direct
UART, and pins, pads, headers, GPIO, probes, jumpers, soldering, or injected
signals remain prohibited.

Attempt-005 recovery, retry, and stop contract: preserve the earliest typed
failure through sealing; record any cleanup failure separately; and permit
only broker-owned same-package typed recovery after a confirmed flash effect.
All owned processes, serial holders, and USB resources must be closed, and
recovery must be proved rather than inferred from elapsed time. Exactly one
Attempt-005 hardware launch is authorized. There is no unchanged retry,
alternate command, Attempt 006, sealed-root replay, or broader diagnostic
authority. Recurrence of the corrected Phase 36 effect-result or SYS-004 seal
classification boundary selects `stop_repeated_boundary`. Accepted outcomes
are `complete`, `stop_repeated_boundary`, `stop_hardware_blocker`,
`stop_authority_boundary`, or `stop_impossible_contract`.

Attempt-005 promotion contract: `SYS-004` may reach `verified` only when the
standalone and broker detectors, exact source/reference/manifest/ELF/package
joins, safe boot, complete HTTP response, identical same-boot/revision
WebSocket projection, manifest-equal `version` and `axeOSVersion`,
manifest-equal extended provenance, manifest-equal ESP-IDF version, private
modes and seals, commit-safe version projection, cleanup, redaction, and every
repository gate pass. Any missing, contradictory, or unclassified fact leaves
only `SYS-004` at `implemented`, with no checklist transition, progress append,
`RESULT.md`, task archival, or evidence promotion.

Attempt-005 software-only correction: the single hardware launch is consumed
and no retry or new ordinal is authorized. The exact package flash completed;
the passive serial operation then failed with `capture_failed`; broker-owned
same-package recovery and cleanup completed; and the attempt sealed as a
non-promotion with no private capture, candidate, projection, or checklist
claim. Source plus the absence of a monitor USB-session trace prove that the
Phase 36 adapter invoked `tools/flash monitor` with `--evidence-mode dual`,
which that CLI rejects before session admission. The contracted projector then
resolved relative private paths before workspace detection and returned
`sys004_private_boundary_invalid` rather than the authenticated seal's
non-eligibility category. This continuation permits only the two targeted
software corrections and fake/filesystem regression work below. It does not
permit credentials, detector, USB, flash, serial, HTTP, WebSocket, recovery,
sealed-root replay or mutation, projection retry, or any other hardware or
network effect.

- [x] Make the passive Phase 36 adapter privately capture the supported
      receive-only `monitor` output without passing unsupported evidence flags,
      and distinguish its private stdout and stderr boundaries.
- [x] Add a fresh-process regression that rejects reintroduced monitor evidence
      flags and proves exactly one protected classifier input plus a typed
      completed result.
- [x] Anchor SYS-004 projector inputs and output to the detected Bazel workspace
      before private admission, with a relative-path non-promotion regression.
- [x] Run every mandatory gate, preserve `SYS-004` as `implemented`, and push a
      truthful correction checkpoint without touching Attempt 005 again.

Attempt-005 correction outcome: the passive adapter now invokes the supported
receive-only `monitor` surface without evidence-only flags, redirects its raw
stdout and diagnostic stderr into separate mode-`0600` files under one
mode-`0700` child, and derives the one trusted origin only from that protected
classifier input. A fresh-process fake rejects every formerly unsupported flag
and proves the typed completed result. The SYS-004 command now anchors all four
relative paths to the detected Bazel workspace before private admission; its
filesystem regression reaches a valid non-promotion seal and returns
`sys004_attempt_not_eligible` without creating output. The ordered Rust checks,
all 80 Bazel tests, Bright Builds, parity validation, unchanged 34/94 progress,
redaction, reference integrity, and diff checks pass. Attempt 005 remains sealed
and untouched after its one projection call. `SYS-004` remains `implemented`;
a fresh progress-backed ordinal and complete task contract are required for
live version promotion, but no separate user confirmation is required.

Standing-authorization continuation: fresh progress-backed ordinals are now
ordinary task execution, so Attempt 006 proceeds without a separate user
confirmation. The Attempt-005 monitor and workspace defects have targeted
real-process regressions, and the later typed automation cutover replaced the
obsolete Phase 36 effect surface. Before hardware, extend the current typed
`capture-version-evidence` workflow so one private exact-package session proves
the manifest-equal `version`, installed `axeOSVersion`, `idfVersion`, extended
provenance, and later same-boot WebSocket projection while emitting only a
closed commit-safe result.

Attempt-006 contract:

1. `just package`
2. `umask 077; mkdir -p scratch/sys004-version-reporting/attempt-006-detector && chmod 700 scratch/sys004-version-reporting/attempt-006-detector && just detect-ultra205 >scratch/sys004-version-reporting/attempt-006-detector/stdout 2>scratch/sys004-version-reporting/attempt-006-detector/stderr`
3. `just capture-version-evidence --private-root scratch/sys004-version-reporting/attempt-006 --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --port <detector-port> --projection docs/parity/evidence/sys004-version-reporting/version-projection.json --capture-timeout-seconds 360`
4. Recovery only after command 3 confirms a flash effect and then fails:
   `just flash --board 205 --port <detector-port> --manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json`

Attempt-006 evidence and safety contract: both ignored attempt roots must be
absent before use and created under `umask 077` with mode `0700`; every private
artifact is mode `0600`. Raw credentials, USB/device identity, origin, network
values, serial, API/WebSocket bodies, process data, and operational paths remain
private. Only the typed redaction-passed projection may be committed. Exactly
one detector-admitted Ultra 205 may receive the clean pushed exact package,
private Wi-Fi NVS input, one factory flash, passive serial observation, and
same-origin read-only HTTP/WebSocket observation. Mining, work submission,
voltage, frequency, fan or other hardware control, pool input, OTA, rollback,
erase, discovery, fault injection, direct UART, and pin/pad/header/GPIO/probe/
jumper/solder or injected-signal work remain prohibited.

Attempt-006 recovery and stop contract: preserve the earliest typed failure;
allow at most the one same-package recovery above after a confirmed flash
effect; prove process, serial, and USB cleanup; and never reuse or rewrite a
sealed root. There is no unchanged retry. A new distinct diagnosed boundary
may receive a targeted regression-backed fix and fresh standing-authorized
ordinal; recurrence after its targeted fix selects `stop_repeated_boundary`.
Accepted outcomes are `complete`, `stop_repeated_boundary`,
`stop_hardware_blocker`, `stop_authority_boundary`, or
`stop_impossible_contract`.

Attempt-006 promotion contract: only `SYS-004` may reach `verified`, and only
after exact source/reference/manifest/package identity, safe boot, complete
same-origin HTTP, later same-boot/revision WebSocket, manifest-equal version and
provenance fields, private modes, cleanup, semantic redaction, and every
repository gate pass. Any missing or contradictory fact leaves `SYS-004` at
`implemented` without `RESULT.md` or progress synchronization.

Attempt-006 software checkpoint: the typed workflow now reads the complete
package identity, captures one same-origin system-info response followed by one
live WebSocket frame, requires identical boot session, revision, and version
projection, and emits only six closed comparison booleans. Raw HTTP/WebSocket
bodies remain mode-`0600` private artifacts. Focused Rust/TypeScript tests and
the ordered Rust, Bright Builds, 28-test Bazel, parity, progress, redaction,
reference-integrity, and diff gates pass. One combined-gate parity invocation
hit a transient host `Resource temporarily unavailable` after all tests passed;
the isolated unchanged parity command immediately passed with no validation
errors.

Attempt-006 outcome: `continue_after_verified_fix` with no device effect. The
detector and package passed, but the capture command was rejected during typed
argument parsing because the operator handoff searched for the legacy shell
form `port=` while the current Rust detector emits `port: `. The private attempt
root and public projection were never created, so no flash, credential read,
serial, HTTP, WebSocket, or recovery action occurred. The immutable detector
root remains private. The targeted fix moves this handoff into the typed
automation command, requires a mode-`0600` detector output with exactly one
canonical admitted port, and has a real-file regression that rejects the
obsolete delimiter.

Attempt-007 contract: standing authorization selects this fresh ordinal after
the verified handoff fix. The objective, privacy, permitted/prohibited effects,
recovery bounds, stop conditions, and promotion gate are identical to Attempt
006, except the repo-owned capture command now resolves the protected detector
result itself. The exact commands are:

1. `just package`
2. `umask 077; mkdir -p scratch/sys004-version-reporting/attempt-007-detector && chmod 700 scratch/sys004-version-reporting/attempt-007-detector && just detect-ultra205 >scratch/sys004-version-reporting/attempt-007-detector/stdout 2>scratch/sys004-version-reporting/attempt-007-detector/stderr`
3. `just capture-version-evidence --private-root scratch/sys004-version-reporting/attempt-007 --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --detector-output scratch/sys004-version-reporting/attempt-007-detector/stdout --projection docs/parity/evidence/sys004-version-reporting/version-projection.json --capture-timeout-seconds 360`
4. Recovery only after command 3 confirms a flash effect and then fails:
   `bash -c 'set -euo pipefail; attempt_port="$(sed -n "s/^port: //p" scratch/sys004-version-reporting/attempt-007-detector/stdout)"; [[ -n "$attempt_port" ]]; just flash --board 205 --port "$attempt_port" --manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json'`

Attempt-007 outcome: `continue_after_verified_fix`. The exact-package flash,
safe boot, HTTP response, WebSocket response, and cleanup completed, so the
optional same-package recovery was unnecessary and was not run. All manifest
comparisons passed: build label, installed static-asset version, semantic/source/
reference/ELF/channel/dirty/release provenance, and ESP-IDF version. The
WebSocket carried the same boot and identical version fields, but its operator
snapshot revision was newer than the immediately preceding HTTP response. The
host validator incorrectly required exact revision equality across sequential
observations. Private roots remain mode `0700`, raw responses mode `0600`, and
no public projection was emitted.

Attempt-008 contract: standing authorization selects this fresh ordinal after
the targeted revision-ordering fix. The validator must accept equal-or-later
positive WebSocket revision for the same boot while still requiring identical
version fields. The objective, privacy, effects, recovery, stop, and promotion
contracts remain otherwise identical to Attempt 007. The exact commands are:

1. `just package`
2. `umask 077; mkdir -p scratch/sys004-version-reporting/attempt-008-detector && chmod 700 scratch/sys004-version-reporting/attempt-008-detector && just detect-ultra205 >scratch/sys004-version-reporting/attempt-008-detector/stdout 2>scratch/sys004-version-reporting/attempt-008-detector/stderr`
3. `just capture-version-evidence --private-root scratch/sys004-version-reporting/attempt-008 --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --detector-output scratch/sys004-version-reporting/attempt-008-detector/stdout --projection docs/parity/evidence/sys004-version-reporting/version-projection.json --capture-timeout-seconds 360`
4. Recovery only after command 3 confirms a flash effect and then fails:
   `bash -c 'set -euo pipefail; attempt_port="$(sed -n "s/^port: //p" scratch/sys004-version-reporting/attempt-008-detector/stdout)"; [[ -n "$attempt_port" ]]; just flash --board 205 --port "$attempt_port" --manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json'`

Attempt-008 outcome: `complete`. The clean pushed package commit
`66cf184943d7f3a5aedfc99e692a9f500707de9e` passed detector admission, exact
factory flash, safe boot, passive serial capture, same-origin HTTP, same-boot
equal-or-later-revision WebSocket comparison, private-mode checks, cleanup, and
the closed Rust evidence validator. All six public version comparisons are
true. Recovery was unnecessary and did not run.

Final verification: the ordered Rust format, warning-denied Clippy, all-target
build, and all-feature tests passed; Bright Builds reported zero findings; all
28 Bazel tests passed; parity validation, 32/94 progress consistency, semantic
redaction, pinned-reference integrity, and diff checks passed. Transition
receipt `20260803T231314Z-SYS-004` changed only `SYS-004` from `implemented` to
`verified` with `unit,workflow,api-compare,hardware-smoke` evidence.

Completion review: Complete. Exact-current-package hardware evidence now binds
the canonical firmware version, installed AxeOS static-asset version, ESP-IDF
version, and extended provenance across HTTP and WebSocket on the Ultra 205.
Raw device and credential material remains private. Broader snapshot, health,
network, mining, safety-control, OTA/recovery, non-205, direct-UART, and pin
claims remain separate tasks and were not promoted.

### task-parity-v12-package-identity-typed-evidence | 2026-08-03 | Re-verify exact package identity from typed hardware evidence

- [x] Confirm the committed `bitaxe-version-evidence-v1` projection binds one
      clean package source commit, the pinned reference, the package manifest,
      a safe Ultra 205 boot, and matching HTTP/WebSocket runtime provenance.
- [x] Run focused typed-evidence, package-manifest, and runtime-attestation
      regressions plus all mandatory repository gates.
- [x] Record a new row-specific plan and result without copying private device,
      network, serial, or credential material.
- [x] Transition only `V12-PACKAGE-IDENTITY-205`, synchronize parity progress,
      and archive this task in the same finalization commit.

Plan:
`docs/parity/work-plans/20260803T231848Z-V12-PACKAGE-IDENTITY-205/PLAN.md`

Authorization and evidence boundary: standing repository-task authorization
applied. This task was evidence-only: it read committed redacted projections
and ran software verification. It performed no detector, flash, reset, monitor,
HTTP, WebSocket, OTA, mining, network, credential, voltage, fan, power,
direct-UART, or pin effect. Private attempt artifacts remained ignored and were
not copied.

Verification: The Rust typed-evidence validator accepted the committed
`bitaxe-version-evidence-v1` projection. Eleven focused runtime boot-attestation
tests, eight package-manifest tests, and the focused Bazel contract, API, and
xtask suites passed. The ordered Rust format, warning-denied Clippy, all-target
build, and all-feature test sequence passed; Bright Builds reported zero
findings; all 28 Bazel tests passed. Parity validation, 33/94 progress
consistency, semantic redaction, pinned-reference integrity, and diff checks
passed. A first verbose parity invocation encountered host stdout backpressure
(`os error 35`); an unchanged rerun captured the report to a temporary log and
passed with `validation_errors: none`. Transition receipt
`20260803T232021Z-V12-PACKAGE-IDENTITY-205` changed only the selected row from
`implemented` to `verified` with `workflow,hardware-smoke` evidence.

Completion review: Complete. The typed migration gap is closed by immutable
exact-package evidence from source commit
`66cf184943d7f3a5aedfc99e692a9f500707de9e`, the pinned reference, one package
manifest, safe Ultra 205 boot, same-origin HTTP provenance, and matching later
same-boot WebSocket provenance. Configuration, network longevity, mining,
safety-control, partitions, recovery, other-board, direct-UART, pin, and release
claims remain separate and were not promoted.

### task-parity-rel09-typed-operator-evidence | 2026-08-03 | Re-verify the typed detector-gated operator workflow

- [x] Project the completed SYS-004 exact-package run into the current release
      operator-evidence schema using closed, redacted facts only.
- [x] Validate the new root through the typed `capture-operator-evidence`
      command and run focused plus mandatory repository gates.
- [x] Record a row-specific result and transition only `REL-09` when the typed
      root, redaction, and detector-gated provenance all pass.
- [x] Synchronize progress and archive this task in the finalization commit.

Plan:
`docs/parity/work-plans/20260803T232442Z-REL-09/PLAN.md`

Authorization and evidence boundary: standing repository-task authorization
applied. This task reprojected immutable committed evidence and ran software
validators only. It performed no new detector, flash, reset, monitor, HTTP,
WebSocket, OTA, mining, network, credential, voltage, fan, power, direct-UART,
or pin effect. It did not copy private attempt artifacts or raw runtime values.

Verification: The current typed `capture-operator-evidence` consumer accepted
the exact release inventory and returned a successful automation result. The
focused automation and parity suites passed. The ordered Rust format,
warning-denied Clippy, all-target build, and all-feature tests passed; Bright
Builds reported zero findings; all 28 Bazel tests passed. Parity validation,
34/94 progress consistency, semantic redaction, pinned-reference integrity,
and diff checks passed. Transition receipt `20260803T232637Z-REL-09` changed
only `REL-09` from `implemented` to `verified` with `workflow` evidence.

Completion review: Complete. A fresh-schema release evidence root now binds the
canonical typed detector-output path, one admitted Ultra 205, exact package,
safe boot, same-origin API, later same-boot WebSocket observation, cleanup, and
redaction from source commit
`66cf184943d7f3a5aedfc99e692a9f500707de9e`. Share and production safe-stop
slots remain deferred. Credentials during mining, settings durability,
ASIC/Stratum, safety controls, recovery, other-board, direct-UART, pin, and
release claims remain separate and were not promoted.

### task-parity-v12-hostname-device-session-retry | 2026-08-03 | Replace the broken hostname restart observer

- [x] Add a private typed reboot intent and a live device-session adapter that
      derives and binds the admitted Ultra 205 physical identity in-process.
- [x] Replace the settings workflow's invented monitor artifact and fixed
      readiness delay with the typed reboot transaction and closed projection.
- [x] Add unit and real-process regressions for the confirmed missing-artifact
      defect, typed failures, restoration, recovery, and privacy boundaries.
- [x] Run all mandatory software gates, commit and push the clean fix, then run
      exactly one detector-gated `attempt-003` hardware capture.
- [x] Transition only `V12-HOSTNAME-205` if typed restart, persistence, cleanup,
      restoration, and redaction all pass; otherwise record the terminal result.

Plan:
`docs/parity/work-plans/20260803T232954Z-V12-HOSTNAME-205/PLAN.md`

Hardware contract: software diagnosis proved that `tools/flash monitor` returns
serial bytes on stdout and intentionally creates no `flash-monitor.log`, while
the settings workflow waited for that nonexistent file. After the typed
device-session replacement passes all software gates on a clean pushed commit,
standing task authorization permits exactly one new-information retry. Run
`just package`; capture `just detect-ultra205` privately beneath mode-`0700`
`scratch/v12-hostname-typed/attempt-003-detector`; then run
`just verify-settings-durability --mode capture --private-root
scratch/v12-hostname-typed/attempt-003 --package-manifest
bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials
wifi-credentials.json --detector-output
scratch/v12-hostname-typed/attempt-003-detector/stdout --projection
docs/parity/evidence/v12-hostname-205/durability-projection.json
--capture-timeout-seconds 360`. The workflow may perform the exact package
flash, one safe hostname PATCH, one typed normal application restart, private
readback, restoration, and at most one recovery-only exact-package flash when
restoration cannot otherwise complete. Stop without retry on any non-ready
device-session category, identity drift, unsafe state, response or
postcondition failure, restoration failure, privacy failure, or cleanup
failure. Direct UART, pins, mining, voltage/fan/power effects, OTA, erase, raw
writes, discovery, and fault injection remain prohibited.

Verification: Implementation commit
`cb0fe1f78ad8dd82ec815069739572053fa54c22` passed focused device-session and
automation tests plus the complete Rust, Bright Builds, Bazel, parity,
progress, redaction, reference-cleanliness, and diff gates before it was pushed.
The first combined parity invocation encountered one transient host
`Resource temporarily unavailable` error; isolated and final full-sequence
reruns passed with no validation findings. The one detector-gated
`attempt-003` then emitted a redacted v2 projection proving same-device
admission, reader-before-request ordering, one normal restart, exact build
recovery, changed boot session, ordinal `N+1`, persisted hostname digest,
cleanup, and confirmed restoration. Transition receipt
`20260804T043800Z-V12-HOSTNAME-205` changed only the selected row from
`implemented` to `verified` with `workflow,hardware-smoke` evidence. Progress
synchronized to 35 of 94 active rows verified (37.2%).

Completion review: Complete. The host orchestration root cause is fixed by the
typed device-session reboot transaction, and real Ultra 205 evidence now proves
the exact hostname-durability claim. The original private hostname was restored
and confirmed, no recovery flash ran, and the single attempt was consumed.
Broader configuration, network longevity, mining, ASIC, safety-control, OTA,
recovery, other-board, direct-UART, pin, and release claims remain separate and
were not promoted.

### task-parity-v12-hostname-typed-capture | 2026-08-03 | Capture fresh typed hostname durability evidence

- [x] Extend `verify-settings-durability` with one semantic capture mode that
      owns exact-package flash, hostname PATCH/readback, normal restart,
      post-restart readback, and restoration of the original hostname.
- [x] Add private-first artifact handling, a closed public projection, strict
      detector-output admission, bounded recovery, and regression coverage.
- [x] Run the bounded detector-gated Ultra 205 attempts and leave
      `V12-HOSTNAME-205` implemented because persistence evidence did not pass.
- [x] Run all mandatory gates and record the terminal blocker without a
      checklist transition, progress synchronization, or task archival.

Plan:
`docs/parity/work-plans/20260803T232954Z-V12-HOSTNAME-205/PLAN.md`

Hardware contract: standing repository-task authorization selects one bounded
attempt after implementation and all software gates pass on a clean pushed
commit. Exact commands are `just package`; one private mode-`0700` detector
capture running `just detect-ultra205`; then `just verify-settings-durability
--mode capture --private-root scratch/v12-hostname-typed/attempt-001
--package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json
--wifi-credentials wifi-credentials.json --detector-output <private-detector>
--projection docs/parity/evidence/v12-hostname-205/durability-projection.json
--capture-timeout-seconds 360`. The typed workflow may flash the exact package,
read the current hostname privately, PATCH one non-secret test hostname, issue
one normal application restart, observe the next boot, prove persistence, PATCH
the original hostname, and prove restoration. It may perform one recovery-only
exact-package flash after a confirmed flash or hostname effect if restoration
cannot otherwise complete. It must keep mining and hardware control disabled,
must not read credential contents, and must never publish hostnames, origins,
network identifiers, USB paths, or raw traces. Stop without retry on ambiguous
detection, identity drift, unsafe state, PATCH/readback/restart mismatch,
restoration failure, privacy failure, or repeated unchanged boundary. Direct
UART, pins, mining, voltage/fan/power effects, OTA, erase, raw writes, discovery,
and fault injection are prohibited.

Verification: The typed capture and invocation regressions pass both success
and post-restart mismatch/restoration paths. The ordered Rust gates, Bright
Builds checks, all 28 Bazel tests, parity/progress, redaction, reference, and
diff checks pass. Attempt 001 passed package build and private detector
admission, completed exact-package flash, safe initial capture, hostname PATCH,
and normal restart, then failed closed because the monitor was launched after
USB restart and produced no post-restart artifact. The public projection was
withheld. Recovery PATCH and private readback confirmed the original hostname
was restored; no recovery flash ran. Attempt 002 used the regression-backed
pre-acquired passive monitor from clean pushed source commit `ca3eeb1c`, passed
fresh detector admission and the same exact-package flash, PATCH, immediate
readback, and restart boundaries, then exhausted the post-restart capture
without producing an artifact. It failed closed with `process_failed`; the
public projection remained absent. A value-free private comparison proved the
recovery readback matched the original hostname, and no recovery flash ran.
The later `task-parity-v12-hostname-device-session-retry` fixed the confirmed
host-orchestration defect and verified the row with typed attempt-003 evidence.

Completion review: Superseded by
`task-parity-v12-hostname-device-session-retry`, which is archived with the
successful implementation and hardware result. The two failed attempts and
their consumed authorization remain historical facts; they did not themselves
prove hostname durability. The replacement task verified `V12-HOSTNAME-205`,
so this record no longer represents an active terminal blocker.

### task-parity-v12-operator-snapshot-typed-capture | 2026-08-04 | Capture substantive two-epoch operator snapshots

- [x] Add a typed, private-first operator-snapshot capture that joins one HTTP
      snapshot, one later same-boot WebSocket snapshot, and the exact retained
      log marker in each of two boot epochs.
- [x] Reuse the live device-session reboot transaction to prove one normal
      restart on the same physical Ultra 205 with exact build identity and
      boot ordinal `N+1`.
- [x] Add closed projection validation, redaction checks, behavior-focused
      unit tests, and a real-child-process regression at the orchestration
      boundary.
- [x] Run every mandatory software gate on a clean pushed implementation, then
      execute exactly one detector-gated hardware capture.
- [x] Transition only `V12-OPERATOR-SNAPSHOT-205` if both substantive epoch
      joins, restart identity, cleanup, safe state, and redaction pass.

Plan:
`docs/parity/work-plans/20260804T122408Z-V12-OPERATOR-SNAPSHOT-205/PLAN.md`

Hardware contract: after the implementation and all software gates pass on a
clean pushed commit, standing task authorization permits `just package`; one
private mode-`0700` detector capture running `just detect-ultra205`; and one
`just capture-operator-snapshot-evidence --private-root
scratch/v12-operator-snapshot/attempt-001 --package-manifest
bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials
wifi-credentials.json --detector-output <private-detector> --projection
docs/parity/evidence/v12-operator-snapshot-205/snapshot-projection.json
--capture-timeout-seconds 360`. The workflow may flash the exact package,
observe private serial/API/WebSocket/retained-log documents, issue exactly one
normal application restart, reacquire only the admitted physical USB device,
and perform one recovery-only exact-package flash if the initial flash or
restart effect occurred but safe exact-build recovery cannot otherwise be
confirmed. It must leave settings unchanged, keep mining and hardware control
disabled, and publish no origins, network or USB identifiers, hostnames,
credentials, raw documents, or traces.

Stop without retry on ambiguous detection, physical-identity drift, missing or
contradictory epoch joins, unsafe state, restart mismatch, build mismatch,
cleanup failure, privacy failure, or recovery failure. Direct UART, pins,
mining, voltage/fan/power control, OTA, erase, arbitrary writes, discovery,
fault injection, and any second restart are prohibited. Exactly one fresh
attempt is authorized; a later ordinal requires verified new information under
the repository hardware-attempt policy. Accepted terminal outcomes are
`complete`, `stop_repeated_boundary`, `stop_hardware_blocker`,
`stop_authority_boundary`, and `stop_impossible_contract`.

Verification: Complete. Software gates passed on clean pushed implementation
commit `409864d0`: `cargo fmt --all`, strict Clippy, all-target/all-feature
Cargo build and tests, managed Bright Builds checks with zero findings, all 28
Bazel test targets, parity with no validation errors, progress, semantic
redaction, reference cleanliness, and diff checks. One private detector
admitted exactly one Ultra 205. The single `attempt-001` capture passed both
substantive epoch joins, the typed same-device restart, exact package recovery,
ordinal `N+1`, safe state, cleanup, modes, Rust contract validation, and
redaction. The committed projection and full result are recorded under the
plan and evidence paths above.

Completion review: Completed and promoted only
`V12-OPERATOR-SNAPSHOT-205` from `implemented` to `verified` under transition
`20260804T124921Z-V12-OPERATOR-SNAPSHOT-205`. No recovery flash or retry ran.
Runtime health, settings, networking, mining, ASIC, safety-control, OTA,
recovery, other-board, and release claims remain separate. The private
hardware root remains ignored and contains no committed sensitive values.

### task-parity-v12-runtime-health-typed-capture | 2026-08-04 | Capture substantive passive runtime health

- [x] Add a typed, private-first runtime-health capture joining exact-package
      HTTP, later same-boot WebSocket, and retained-log projections.
- [x] Prove available healthy supervisor checkpoints and fresh participating
      task-watchdog feeds without invoking self-test, mining, or controls.
- [x] Add closed evidence validation, failure/recovery precedence, redaction,
      behavior tests, and a real-child-process regression.
- [x] Run all mandatory gates on a clean pushed implementation, then execute
      exactly one detector-gated passive hardware capture.
- [x] Transition only `V12-RUNTIME-HEALTH-205` if the substantive live join,
      exact package, safe state, cleanup, and privacy contract all pass.

Plan:
`docs/parity/work-plans/20260804T125402Z-V12-RUNTIME-HEALTH-205/PLAN.md`

Hardware contract: after the implementation and all software gates pass on a
clean pushed commit, standing task authorization permits `just package`; one
private mode-`0700` detector capture running `just detect-ultra205`; and one
`just capture-runtime-health-evidence --private-root
scratch/v12-runtime-health/attempt-001 --package-manifest
bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials
wifi-credentials.json --detector-output <private-detector> --projection
docs/parity/evidence/v12-runtime-health-205/runtime-health-projection.json
--capture-timeout-seconds 360`. The workflow may flash the exact package and
observe private serial/API/WebSocket/retained-log documents. It may perform one
recovery-only exact-package flash if the initial flash effect occurred but
safe exact-build recovery cannot otherwise be confirmed. It must leave
settings unchanged, keep mining and hardware control disabled, and publish no
origins, network or USB identifiers, hostnames, credentials, raw documents,
traces, or unbounded timing values.

Stop without retry on ambiguous detection, identity drift, missing or
contradictory health joins, stale/unhealthy/unavailable supervisor or watchdog
truth, unsafe state, build mismatch, cleanup failure, privacy failure, or
recovery failure. Direct UART, pins, restart, self-test, mining,
voltage/fan/power control, OTA, erase, arbitrary writes, discovery, and fault
injection are prohibited. Exactly one fresh attempt is authorized; a later
ordinal requires verified new information under the repository
hardware-attempt policy. Accepted terminal outcomes are `complete`,
`stop_repeated_boundary`, `stop_hardware_blocker`, `stop_authority_boundary`,
and `stop_impossible_contract`.

Verification: Complete. The implementation and finalization gates passed
`cargo fmt --all`, strict Clippy, all-target/all-feature Cargo build and tests,
Bright Builds checks with zero findings, all 28 Bazel test targets, parity with
no validation errors, progress, redaction, reference cleanliness, and diff
checks. One private detector admitted exactly one Ultra 205. The sole
`attempt-001` proved the exact package, same-boot HTTP and later WebSocket
runtime health, both exact retained tuples, healthy supervisor checkpoints,
fresh participating watchdog feeds, disabled mining and hardware control,
cleanup, modes, Rust validation, and redaction. No recovery flash or retry ran.

Completion review: Completed and promoted only `V12-RUNTIME-HEALTH-205` from
`implemented` to `verified` under transition
`20260804T131416Z-V12-RUNTIME-HEALTH-205`. The result closes only passive
bounded runtime-health truth for one Ultra 205 attempt. Self-test execution,
mining, controls, settings, networking, updates, recovery, other boards,
release readiness, and unbounded runtime health remain separate. The private
hardware roots remain ignored and contain no committed sensitive values.

### task-parity-str012-payout-address-codecs | 2026-08-04 | Implement payout address codecs

- [x] Implement typed Base58Check and SegWit Bech32/Bech32m codecs without a
      new dependency.
- [x] Render and validate P2PKH, P2SH, P2WPKH, P2WSH, and P2TR scripts across
      mainnet, testnet, and regtest.
- [x] Add provenance-bound golden vectors and complete invalid-boundary tests.
- [x] Run focused and mandatory repository gates, then transition only
      `STR-012` when the complete pure contract passes.

Plan:
`docs/parity/work-plans/20260804T131755Z-STR-012/PLAN.md`

Authorization and privacy: this task is pure software work. It authorizes no
hardware, pool, network, credential, settings, owner-address, mining, ASIC,
firmware, OTA, direct-UART, or pin effects. Committed address fixtures must be
public standard vectors, never local owner inputs.

Verification: Complete. Focused strict Clippy, all 258 `bitaxe-stratum` Cargo
tests, the Bazel crate target, mandatory Rust format/strict Clippy/build/tests,
Bright Builds checks with zero findings, all 28 Bazel tests, parity with no
validation errors, progress, redaction, reference cleanliness, and diff checks
passed. Seven provenance-bound public vectors and six behavior tests cover all
five standard scripts, three networks, future witness versions, and the closed
invalid-input boundaries.

Completion review: Completed and promoted only `STR-012` from `not-started` to
`verified` under transition `20260804T132736Z-STR-012`. The implementation adds
no dependency or effectful path. Local owner addresses, configured-address
integration, live payouts, mining, ASIC work, Stratum V2 keys, and hardware
remain separate.

### task-parity-sys005-runtime-orchestration | 2026-08-04 | Close runtime owner scheduling parity

- [x] Centralize the pure periodic-deadline contract used by the safety,
      operator-observation, and production-session owners.
- [x] Bind work creation, bounded result consumption, authoritative readiness,
      and fail-closed power/safety scheduling to explicit single-owner seams.
- [x] Add behavioral and source-ownership regressions, build the real firmware,
      run mandatory gates, and transition only `SYS-005` when its bounded
      software orchestration contract is proven.

Plan: `docs/parity/work-plans/20260804T150000Z-SYS-005/PLAN.md`

Authorization: local software and build work only. No hardware attempt,
credentials, external network request, mining, voltage/fan/power effect, OTA,
recovery, direct UART, or pins.

Verification: Complete. Four checked deadline tests, the strengthened
result-before-submit lifecycle test, startup and bounded-owner source checks,
and the real ESP32-S3 firmware build passed. The ordered Rust gate, Bright
Builds, all 29 Bazel test targets, parity/progress, redaction, reference
cleanliness, and diff checks passed on implementation commit `5c386676`.

Completion review: Completed and promoted only `SYS-005` from `not-started` to
`verified` under transition `20260804T154500Z-SYS-005`. Absolute deadlines now
prevent observation/supervisor drift and production-readiness starvation under
continuous inbox traffic. Live FreeRTOS timing under load, production mining,
pool connectivity, ASIC traffic, accepted/rejected shares, hardware effects,
fault injection, soak behavior, credentials, OTA, recovery, other boards,
direct UART, and pins remain separate.

### task-parity-api010-baseline-epoch-admission | 2026-08-04 | Close production multi-session baseline admission

- [x] Reproduce the production-shaped `baseline_multiple_sessions` failure at
      the real flash-monitor/classifier orchestration seam without retaining or
      exposing the private hardware trace.
- [x] Determine why the initial exact-package flash-monitor transcript contains
      multiple boot epochs and design a closed selection/admission rule that
      cannot accept stale, mixed-device, or ambiguous session evidence.
- [x] Implement the minimal root-cause fix with unit and real-child-process
      coverage, including multiple-session, malformed, stale, and single-ready
      epoch cases.
- [x] Run the complete mandatory verification sequence and review public output
      for secrets before proposing any separately task-gated hardware attempt.

Plan: `docs/parity/work-plans/20260804T192918Z-API-010/PLAN.md`. This follow-up
continues from `docs/parity/work-plans/20260804T185605Z-API-010/PLAN.md` without
modifying that immutable file.

Authorization: read-only inspection of committed code and private local
attempt structure plus synthetic software tests only. Do not read, print,
summarize, commit, or copy the private serial trace. No flash, monitor, HTTP,
theme mutation, restart, hardware retry, credentials, mining, controls, OTA,
recovery, direct UART, pins, or physical electrical action is authorized by
this task.

Verification: Complete. Seventeen focused Phase 33 tests, the real-child
production-shaped two-epoch regression, strict Clippy, all-target/all-feature
Cargo build and tests, Bright Builds with zero findings, all 34 Bazel tests,
parity with no validation errors, progress, semantic redaction, pinned-reference
cleanliness, immutable-plan comparison, sensitive-output review, and diff
checks passed.

Completion review: Completed the host-orchestration root-cause fix without
reading the private hardware trace or touching hardware. Exact-package settings,
theme, and operator-snapshot captures now admit only an independently complete,
safe terminal epoch after a sequential stale prefix; malformed, mixed,
interleaved, nonsequential, stale-only, and incomplete evidence fails closed.
The original whole-trace classifier is unchanged, `API-010` remains
`implemented`, no evidence was promoted, and any live retry requires a new
task-gated hardware contract.

### task-parity-api010-open-plan-lineage | 2026-08-04 | Reconcile immutable API-010 plan lineage

- [x] Preserve both immutable API-010 plans and make the deterministic selector
      resume only the newest plan in an explicitly linked same-row chain.
- [x] Keep unlinked same-row duplicates and every cross-row ambiguity
      fail-closed with focused regressions.
- [x] Run focused and mandatory gates, archive this software-only task, and
      rerun the selector without creating parity evidence or changing status.

Plan: `docs/parity/work-plans/20260804T192918Z-API-010/PLAN.md`. The follow-up
plan directly names
`docs/parity/work-plans/20260804T185605Z-API-010/PLAN.md`, which is the immutable
lineage edge this task makes machine-checkable.

Authorization: local selector, synthetic filesystem tests, tracker, and
worklog changes only. No hardware, credentials, network requests, settings,
theme mutation, restart, mining, controls, OTA, recovery, direct UART, or pins.

Verification: Complete. Ten focused Cargo selector tests, the Bazel parity
target, the mandatory Rust sequence, Bright Builds with zero findings, all 34
Bazel tests, parity with no validation errors, progress, semantic redaction,
pinned-reference cleanliness, immutable-plan comparison, sensitive-output
review, and diff checks passed. The real selector resumes the newer linked
`API-010` plan and emits no candidate list.

Completion review: Completed the selector root-cause fix without changing
either immutable plan. Surviving same-row plans now form a chronological chain
only when every newer document directly references the immediately older
backticked path; unlinked or cross-row ambiguity still fails closed. `API-010`
remains `implemented`, no `RESULT.md` or parity evidence was created, no status
was promoted, and no live retry or hardware interaction was authorized.

### task-parity-api010-flash-monitor-child-diagnostics | 2026-08-04 | Preserve typed initial-child failure evidence

- [x] Enable the existing private `phase36-effect-result-v1` contract on the
      initial exact-package flash-monitor child and strictly parse its closed
      device-effect state.
- [x] Classify only allowlisted dual-evidence terminal markers from child
      stderr, preserve the numeric exit/timeout facts, and expose no raw child
      output or operational identifiers.
- [x] Include `ThemeDurabilityError.publicValue` in the public automation
      failure envelope and prove the behavior through real child processes.
- [x] Run focused tests and the complete repository verification sequence,
      review the diff for sensitive values, and push the implementation
      checkpoint without changing `API-010` or authorizing hardware.

Plan: `docs/parity/work-plans/20260804T200849Z-API-010/PLAN.md` remains the
immutable open plan. This software-only follow-up addresses the distinct
terminal boundary recorded by `attempt-003`; it does not reopen or alter that
hardware ordinal.

Diagnosis: `captureThemeDurability` held the initial flash-monitor child's
stdout and stderr only in memory, collapsed every nonzero exit to the same
`process_failed` message, and did not enable the flash tool's existing
mode-`0600` `phase36-effect-result-v1` artifact. The CLI then excluded
`ThemeDurabilityError.publicValue` from the otherwise shared typed failure
envelope. Consequently the exhausted attempt proves where orchestration
stopped but cannot distinguish no device effect, confirmed partial effect,
completed flash followed by monitor failure, or the flash tool's closed
dual-evidence terminal marker.

Scope and privacy: modify only repository-owned automation/process contracts,
the theme durability shell, and focused tests. The typed effect result stays
under the supervisor-owned mode-`0700` private root as mode `0600`. Public
failure facts may contain only the fixed stage, an allowlisted terminal marker,
the existing closed effect status, a bounded numeric exit code, timeout state,
and safe recovery booleans. They must never contain stdout/stderr text, command
arguments, paths, origins, theme or hostname values, ports, USB/network
identifiers, credentials, tokens, or raw traces.

Hardware boundary: this task is software-only. Do not run detector, flash,
monitor, restart, HTTP mutation, recovery, or any other device command. A new
hardware ordinal requires a later task with its own exact command, evidence,
recovery, retry, and stop contract after this fix is verified and pushed.

Accepted outcome: focused unit and real-child-process tests prove the private
effect artifact, closed marker classification, public theme failure facts,
missing/malformed artifact handling, earliest-failure precedence, and sensitive
value exclusion. `API-010` remains `implemented`; no checklist transition or
progress synchronization is requested.

Verification: Complete. `//tools/automation:automation_test` and
`//tools/flash:tests` passed with unit and real-child-process coverage. The
ordered formatting, strict Clippy, all-target/all-feature build and Cargo tests,
Bright Builds, all 34 Bazel tests, parity validation, progress, semantic
redaction, pinned-reference cleanliness, and diff checks passed on source
commit `8c93b1b73a0e62ba4fecb1ae46604d30ac29916a`.

Completion review: Completed the host diagnostic fix without hardware or
private-value exposure. Initial flash-monitor failures now retain a strict
mode-`0600` exact-package effect result and publish only the fixed stage,
bounded exit/timeout facts, a closed dual-evidence marker, and completed,
confirmed-partial, no-effect, missing, or invalid effect status. Launch and
malformed-artifact failures preserve the primary category. The prior attempt's
underlying child reason remains unrecoverable retroactively, `API-010` remains
`implemented`, and a future hardware ordinal still requires a new complete
task contract after this pushed fix.

### task-parity-api010-live-theme-durability-attempt-007 | 2026-08-04 | Retry after normal-power remediation

- [x] Preserve the pushed 16 KiB observer fix and attempt-006 record, then
      commit and push the linked immutable attempt-007 plan.
- [x] After the user reports one full normal barrel/USB power cycle, build the
      exact pushed package and run one fresh protected detector.
- [x] Run the single bounded capture only if board-info admission objectively
      proves the detector boundary changed; otherwise record the terminal
      outcome and stop.
- [x] Promote only `API-010` on complete typed evidence; otherwise preserve the
      earliest category, withhold evidence, keep `implemented`, and stop.

Plan: `docs/parity/work-plans/20260804T224128Z-API-010/PLAN.md`. This immutable
plan continues the pushed attempt-006 outcome at `486d0718` without changing
the already verified software fix.

Progress basis: attempt-006 stopped as `bootloader_connect_failed` before
flashing. Protected retry-admission and final-cleanup summaries each observed
the same accessible holder-free physical device for three stable samples with
unchanged enumeration, and cleanup completed. Repository policy maps exactly
this boundary to disconnecting normal USB and barrel/DC power for ten seconds,
then reconnecting normal power followed by USB. The occurrence requires a user
report; only a successful fresh detector is objective proof of change.

Manual occurrence checkpoint: disconnect both normal barrel/DC power and USB
for at least ten seconds, then reconnect normal barrel power followed by USB.
Do not infer or automate this occurrence, and do not run the detector until the
user reports it completed. Standing task authorization already covers the
task-gated commands; this is not a repeated permission request.

Authorization, exact commands, private paths, recovery, retry bounds, stop
conditions, and promotion criteria are defined in the linked plan. No hardware
action is allowed before the reported occurrence.

Verification: The reported normal power cycle moved the bootloader boundary,
the one detector passed, and the exact package flash completed. The bounded
capture then closed `evidence_invalid`: terminal baseline classification was
not admissible, with closed offline category `runtime_origin_missing`. Private
trace reduction found 51 distinct panic-reset boot sessions and 52 stack
overflows with no runtime-origin or Wi-Fi-state marker. Startup reaches the
rendered operator display immediately before the overflow. Exact-package ELF
disassembly shows the 8 KiB operator-sensor task enters with a 2 KiB frame and
its startup screen collection reaches a 7,872-byte full API-snapshot frame.
No theme mutation, restart, restoration, or recovery flash occurred; the
public projection and `RESULT.md` remain absent and `API-010` remains
`implemented`.

Completion review: Attempt-007 is truthfully closed without retry, evidence,
or promotion. The earlier observer-stack hypothesis targeted the wrong thread;
the new bounded evidence identifies the screen path's full API-snapshot
dependency as the reproducible stack-budget violation. A new immutable plan
and regression-backed narrow screen projection are required before another
hardware attempt. This task claims no network discovery, mining, ASIC,
hardware-control, display-input, OTA, partition, recovery, other-board, or
release parity beyond the exact admitted transaction.

### task-parity-nonverified-plan-closure | 2026-08-10 | Add truthful terminal parity-plan closure

- [x] Add a validated `CLOSURE.md` lifecycle artifact for terminal parity plans
      that remain below `verified`.
- [x] Teach deterministic plan selection to close only valid non-verified
      dispositions while keeping their rows in the unfinished candidate queue.
- [x] Close the exhausted API-010 plan without changing checklist status,
      progress history, or README parity status.
- [x] Run focused and mandatory verification, record completion, archive this
      maintenance task, and prepare the audited commits for synchronized push.

Plan: Resume
`docs/parity/work-plans/20260805T005320Z-API-010/PLAN.md` solely to repair its
terminal unchanged-status lifecycle. The immutable plan remains unedited.

Authorization: local repository maintenance and tests only. No hardware,
detector, credentials, network discovery, flash, settings mutation, mining,
hardware control, OTA, recovery, direct UART, or pins.

Verification: Focused Cargo and Bazel parity tests pass, including direct
coverage of result-based closure, checklist-status advance, valid unchanged
non-verified closure, candidate retention, and every specified malformed or
ambiguous closure case. The full formatting, strict Clippy, all-target build,
all-feature tests, Bright Builds, `just test`, parity report and progress,
redaction, reference, and diff-check gate passes. The real Bazel-built selector
reports `maybe_open_plan: null`, retains `API-010` in the ordered unfinished
candidate queue, and the checklist, progress ledger, and README hashes remain
byte-for-byte equal to their pre-change baselines.

Completion review: Added a fail-closed, plan-digest-bound `CLOSURE.md` contract
for terminal `blocked`, `cancelled`, or `superseded` plans that remain below
`verified`. Closed the exhausted API-010 plan as `implemented` and `blocked`
without a verification claim, parity transition, progress event, README
rewrite, or hardware access. The blocked API-010 attempt task remains active
and unarchived; physical hardware access and a fresh task-gated plan remain
required before another attempt. Implementation source commit:
`181573862e109dd63cbda3a36886f6d040b62f34`.

### task-parity-cfg005-full-settings-persistence | 2026-08-10 | Complete runtime settings persistence

- [x] Persist every validated upstream REST setting through one serialized,
      commit-confirm-publish transaction.
- [x] Preserve atomic validation, unknown-field compatibility, secret-free
      diagnostics, and all downstream safety and effect gates.
- [x] Add exhaustive reference-derived and adapter-boundary regressions, then
      build the real firmware and run every mandatory gate.
- [x] Promote only `CFG-005` when the complete software contract is proven;
      otherwise record the exact blocker without claiming verification.

Plan: `docs/parity/work-plans/20260810T032554Z-CFG-005/PLAN.md`

Selection: `CFG-001` was skipped because its remaining voltage/frequency
behavior requires hardware evidence. `CFG-005` was the next deterministic
candidate and the first software-actionable row: validation covered the full
upstream schema, while the production route persisted only hostname and the
project-owned boot preference.

Authorization: local source, public pinned reference, fixtures, builds, and
tests only. No hardware, credentials, external network, USB, serial, mining,
ASIC traffic, voltage/frequency/fan/thermal/power effects, OTA, recovery,
direct UART, or pins.

Verification: [CFG-005 result](docs/parity/work-plans/20260810T032554Z-CFG-005/RESULT.md)
binds implementation commit `5faf33c119653b58abe857425e5a46fad06a0a08`
and the pinned reference. Formatting, strict Clippy, all-target build,
all-feature tests, Bright Builds, focused Cargo/Bazel API tests, the real
ESP32-S3 firmware build, redaction, reference, immutable-plan, and diff checks
passed before promotion. The checklist transitioned from `implemented` to
`verified` with `unit,golden,workflow` evidence, and progress synchronized to
40 of 94 active rows (42.6%).

Completion review: Every validated upstream REST field now reaches exact typed
NVS write, single commit, independent private reconciliation, and non-secret
snapshot publication through the production adapter. Invalid known inputs fail
before storage and unknown-only inputs remain inert. The exhaustive fixture
covers 42 REST fields and 44 writes, including both legacy mirrors. No hardware
was accessed. Live NVS media durability, hostname application, credential
consumption, network reconnection, mining, ASIC work, and all hardware-control
effects remain separate non-claims and retain their existing gates.

### task-ultra205-boot-recovery-attempt-011 | 2026-08-11 | Retry boot recovery with the canonical observation CLI

- [x] Commit and push the linked immutable attempt-011 plan.
- [x] Add and pass a focused regression for the exact observation campaign
      flags, including absence of a mining profile and pool credentials.
- [x] Run the complete required software gate, commit and push the regression,
      then build and admit one clean exact package.
- [x] Run one new detector and, only after success, one observation campaign
      using canonical `--flag value` arguments.
- [x] Record detector, flash, safe NVS, runtime, cleanup, and privacy outcomes
      independently; stop without another retry or parity promotion.

Plan: `docs/parity/work-plans/20260811T151310Z-API-010/PLAN.md`. It directly
continues attempt-010 after its pre-process `cli_argument_rejected` outcome.

Authorization and effects: standing task authorization covers only the exact
software regression, package, detector, and observation command in the linked
plan. The campaign retains the attempt-010 factory-package and safe NVS
replacement effects: local Wi-Fi credentials, `mineonboot=false`, and the
observation marker only. Prior hostname, pool, and other settings may be
removed. Mining, pool access, hardware controls, OTA, erase-flash, raw writes,
discovery, direct UART, and electrical manipulation remain prohibited.

Evidence, recovery, retry, and acceptance: use new private mode-0700
`wrapper-011` and `attempt-011` roots with mode-0600 artifacts. Preserve the
earliest typed failure and treat completed flash separately from runtime proof.
Release owned resources on every path. This task authorizes one detector and
one conditional campaign only. Complete only on exact-package stable runtime,
safe observation state, cleanup, and privacy; otherwise withhold evidence and
stop with `API-010` still `implemented`.

Verification: The immutable plan was pushed at `5d647eb7`. Focused Cargo and
Bazel flash tests passed, and the complete Cargo, Bright Builds, Bazel, package,
parity, redaction, reference, selector, and diff gate passed. Parity immediately
after the all-Bazel suite hit macOS `os error 35` in two complete gate passes;
resource checks were healthy, and the same remaining sequence passed in a fresh
Bazel server each time. Regression commit `fc12e24f` was pushed, and its clean
schema-v3 package admitted six
digest-bound artifacts and the pinned reference. The sole detector admitted
one Ultra 205. The canonical campaign completed the factory flash and safe NVS
seed, then accepted 1,049 trusted runtime markers and five fresh observation
checkpoints across 360 seconds. Its result was redacted and sealed; USB cleanup,
holder release, private modes, and public-result privacy checks passed.

Completion review: The current pushed package recovered the prior panic-reset
boot loop through the normal USB flash path without a factory reset. Runtime
identity was trusted, serial classification was clean, safety stayed fresh,
`mineonboot` remained false, no mining profile or pool configuration was used,
and no parity promotion occurred. `API-010` remains `implemented` because theme
mutation and restart durability were not exercised. The result claims no
network longevity, mining, ASIC or hardware-control effect, OTA, erase, raw
write, direct UART, pins, other-board behavior, or release readiness.

### task-parity-api010-live-theme-durability-attempt-012 | 2026-08-11 | Verify theme durability after boot recovery

- [x] Commit and push the linked immutable attempt-012 plan.
- [x] Re-run focused theme, device-session, CLI, and redaction regressions; fix
      only a reproduced current blocker.
- [x] Run the complete software gate and admit one clean exact package.
- [x] Run one new detector and, only after success, one bounded theme capture.
- [x] Promote only `API-010` on complete v1 evidence; otherwise record the
      earliest typed boundary and stop without retry.

Plan: `docs/parity/work-plans/20260811T155722Z-API-010/PLAN.md`. It continues
the closed API-010 lineage after the exact-package attempt-011 result proved
successful flashing and stable trusted runtime for 360 seconds.

Authorization and effects: standing task authorization covers only the plan's
one detector and conditional `verify-theme-durability` transaction. Allowed
effects are one exact-package flash with safe local Wi-Fi seed, one generated
non-secret theme mutation, one normal software restart, exact theme
restoration, and at most the workflow's recovery flash. Mining, pool access,
hardware controls, Wi-Fi/hostname mutation, OTA, erase, raw writes, discovery,
direct UART, and electrical manipulation remain prohibited.

Evidence, privacy, recovery, and retry: use absent ignored mode-0700
`wrapper-012` and `attempt-012` roots with mode-0600 files. Public output may
contain only closed categories, safe booleans/counts, cryptographic identities,
and the redacted projection. Preserve the earliest failure and completed flash
effect through restoration/recovery, release all owned resources, and do not
retry.

Verification: The complete plan gate passed, including ordered Cargo checks,
Bright Builds, all Bazel tests, parity/progress, redaction, reference,
continuation-aware selector, and diff checks. Plan commit `f2520d1e` was pushed.
Focused theme, device-session, CLI, and redaction regressions and the exact-head
full gate passed. The sole detector and conditional capture passed, producing
the redacted v1 projection with exact package identity, one restart, same-device
recovery, ordinal `N+1`, persisted theme equality, confirmed restoration,
disabled mining/hardware control, cleanup, private modes, and redaction.
Evidence commit `d789664c` was preserved as `SOURCE_COMMIT`; transition
`20260811T155722Z-API-010` changed only `API-010` to `verified` with
`unit,golden,api-compare,workflow,hardware-smoke`, and progress synchronized to
41 of 94 active rows (43.6%).

Completion review: The exact pushed firmware proved GET/POST, immediate
readback, one normal software restart, same-device exact-build recovery,
post-restart persistence, and confirmed original-theme restoration. No recovery
flash was required, no mining or hardware control occurred, and all private and
public privacy checks passed. Installed AxeOS browser behavior, repeated or
power-loss durability, mining, networking longevity, updates, other boards, and
release readiness remain explicit non-claims.

### task-parity-selector-historical-closure-forward-transition | 2026-08-11 | Preserve historical closures after promotion

- [x] Reproduce the selector failure after `API-010` advances from the status
      recorded by an older immutable non-verified closure.
- [x] Bind closure status validation to the immutable plan status and validate
      later checklist movement independently as a forward transition.
- [x] Add a focused regression for a historical implemented closure followed
      by checklist verification.
- [x] Run focused parity tests and the complete final repository gate.
- [x] Review, archive this follow-up task, commit, and push the API-010
      finalization and selector fix.

Trigger: The API-010 final selector gate failed after every earlier build,
test, redaction, and reference check passed because a historical blocked
closure still recorded `implemented` while the current checklist correctly
recorded `verified`.

Scope: Update only the parity selector's historical-closure validation and its
tests. Do not edit immutable parity plans or closures, change any additional
checklist row, rerun hardware, or expand API-010's verified claims.

Verification: The focused 295-test Cargo parity suite, Bazel parity target,
and selector regression passed. The complete ordered Cargo, Bright Builds,
Bazel, parity/progress, redaction, and reference gate passed. Selector,
immutable-plan hash, transition, progress, task uniqueness, privacy, and diff
checks passed without another hardware run.

Completion review: Historical non-verified closures now remain valid when a
later continuation legitimately advances the row, while regressions and other
non-forward checklist changes still fail closed through the independent status
advance guard. The fix changes no evidence, plan, closure, or additional parity
claim.

### task-parity-api002-system-info-contract | 2026-08-11 | Complete and verify system-info parity

- [x] Commit and push the immutable API-002 plan and task contract.
- [x] Implement the exhaustive pinned system-info field/type/conditional
      contract with secret-safe confirmed settings and runtime inputs.
- [x] Add a typed aggregate-only capture workflow and regression coverage.
- [x] Run the complete software gate, push the implementation, and admit one
      clean exact package.
- [x] Run one detector and one conditional passive capture.
- [x] Promote `API-002` only on complete accepted evidence.
- [x] Commit and push the immutable stack-fix retry plan.
- [x] Build and admit one clean exact package containing the stack fix.
- [x] Run one detector and one conditional passive `attempt-002` capture.
- [x] Promote only on complete independently validated evidence.

Plans:

- Closed: `docs/parity/work-plans/20260811T164522Z-API-002/PLAN.md`.
- Completed: `docs/parity/work-plans/20260811T174900Z-API-002/PLAN.md`.

Authorization and effects: standing task authorization covers the plan's one
detector and one conditional capture. The capture may perform one exact-package
flash-monitor with the ignored Wi-Fi credential input, passive same-origin
HTTP/WebSocket/retained-log reads, normal USB reset/re-enumeration inherent to
the flash, and at most one exact-package recovery flash after an initial flash
effect. It may not read pool credentials or mutate settings, mine, control
hardware, restart through HTTP, discover the network, update, erase, write raw
data, inject faults, terminate foreign processes, or use direct UART or pins.

Evidence, privacy, recovery, and retry: use absent ignored mode-0700
`wrapper-001` and `attempt-001` roots with mode-0600 files. Raw response,
configuration, hostname, origin, port, USB/network/process, credential, serial,
and trace material stays private. Public evidence is aggregate field/type and
identity data only. Preserve the earliest typed failure, recover only as the
plan permits, release owned resources, and do not retry.

Verification: The complete plan gate passed, including ordered Cargo checks,
Bright Builds, all Bazel tests, parity/progress, redaction, reference,
continuation-aware selector, sensitive-output, task uniqueness, and diff
checks. The immutable plan SHA-256 is
`942264a2dccbf729001c3c40024659424842c125735bb6817d7b6114dbb5cd20`.

Completion review: The exhaustive software contract and capture workflow were
implemented, verified, committed, and pushed. The one authorized exact-package
hardware attempt failed closed as `evidence_invalid` because the ESP-IDF `main`
task repeatedly overflowed while startup readiness constructed the full
operator API snapshot. Recovery and cleanup completed, no public projection
was emitted, and no retry occurred. The root cause is fixed in software with a
platform-only startup path, bounded snapshot footprint, source ownership guard,
and real-firmware stack audit, but that fix has no hardware evidence. The plan
is closed at
`docs/parity/work-plans/20260811T164522Z-API-002/CLOSURE.md`; `API-002` remains
`implemented`, and a fresh immutable plan is required before another bounded
attempt.

Fresh retry authorization and effects: The targeted startup-stack fix at
`84b90c9e677b4def1d0ab7508e2b8e64dd08c617` satisfies the new-information
gate. Standing authorization covers exactly these commands after the active
plan and clean package are pushed and admitted:

1. `test ! -e scratch/api002-system-info/wrapper-002 && (umask 077; mkdir -m 700 -p scratch/api002-system-info/wrapper-002 && just detect-ultra205 > scratch/api002-system-info/wrapper-002/detector.stdout 2>&1)`
2. Only after command 1 succeeds:
   `test ! -e scratch/api002-system-info/attempt-002 && test ! -e docs/parity/evidence/api002-system-info/system-info-projection.json && (umask 077; just capture-system-info-evidence --private-root scratch/api002-system-info/attempt-002 --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --detector-output scratch/api002-system-info/wrapper-002/detector.stdout --projection docs/parity/evidence/api002-system-info/system-info-projection.json --capture-timeout-seconds 360 > scratch/api002-system-info/wrapper-002/capture.stdout 2> scratch/api002-system-info/wrapper-002/capture.stderr)`

The objective is stable exact-build boot and aggregate-only system-info capture.
Raw serial, responses, settings, hostnames, origins, ports, USB/network/process
identities, credentials, and traces remain private under the ignored mode-0700
roots in mode-0600 files. One factory flash, its normal USB reset and
re-enumeration, passive same-origin reads, and at most one exact-package
recovery flash after an initial flash effect are allowed. Settings mutation,
HTTP restart, pool-credential access, mining, hardware control, discovery,
update, erase, raw write, fault injection, foreign-process termination, direct
UART, and pin work are prohibited. Preserve the first typed failure, release
owned resources, publish only after every schema/identity/coherence/safety/
cleanup/redaction gate passes, and do not retry this plan. Accepted terminal
categories are `hardware_blocked`, `evidence_invalid`, `timeout`, and
`process_failed`; any failure withholds promotion and requires a new plan.

Retry plan verification: The complete ordered software gate, parity lifecycle
validation, redaction/reference checks, task uniqueness, sensitive-output, and
diff checks passed. The immutable active plan SHA-256 is
`8c64ece32a6044e7d2b38a1219a92f02d4adfd519c31722e8cbb8965e52e6cb9`.

Final completion review: The single fresh detector and conditional capture
passed against exact package source `524b445e`. The independently validated v1
projection proves stable same-build boot after the startup-stack fix, coherent
same-session HTTP and WebSocket revisions and retained tuples, all 94 required
system-info fields with correct types and conditional absence, confirmed
settings, disabled mining and hardware control, cleanup, private artifact
modes, and redaction. Evidence commit `dada4fba` was preserved as
`SOURCE_COMMIT`; transition `20260811T174900Z-API-002` changed only `API-002`
to `verified`, and progress synchronized to 42 of 94 active rows (44.7%). No
live found-block event, mining, hardware controls, reconnect longevity,
updates, other-board behavior, or release readiness is claimed.

### task-parity-api003-live-multifield-patch | 2026-08-11 | Verify production multi-field settings PATCH

- [x] Commit and push the immutable API-003 plan and task contract.
- [x] Add the typed aggregate-only capture and validator with focused
      regressions and real-child-process coverage.
- [x] Run the complete software gate, commit and push implementation, and admit
      one exact clean package.
- [x] Run exactly one detector and one conditional bounded capture.
- [x] Promote only `API-003` on complete independently validated evidence;
      otherwise record the typed boundary and stop without retry.

Plan: `docs/parity/work-plans/20260811T182057Z-API-003/PLAN.md`.

Plan closure: The immutable plan is closed non-verified at
`docs/parity/work-plans/20260811T182057Z-API-003/CLOSURE.md`. Its combined
hostname-plus-theme system PATCH is impossible because theme belongs to the
separate `/api/theme` route and is not an accepted `/api/system` field. No
hardware attempt was spent and `API-003` remains `implemented`. The next safe
action is a fresh linked plan using two actual benign system-settings fields.

Corrected active plan:
`docs/parity/work-plans/20260811T182900Z-API-003/PLAN.md`. It continues the
closed predecessor with a source-proved hostname-plus-rotation `/api/system`
transaction. Standing authorization covers only its absent corrected wrapper
and attempt roots, exactly one detector, and one conditional capture after the
repo-owned command, complete gate, pushed implementation, and exact package
admission exist. Allowed effects are one exact-package flash, one generated
atomic hostname/rotation PATCH and combined readback, one exact restoration
PATCH and combined readback, normal USB reset/cleanup, and at most one recovery
flash. All prior privacy, prohibited-effect, earliest-failure, no-retry, and
promotion conditions remain in force; `theme` is no longer part of the active
contract.

Authorization and effects: standing task authorization covers only the plan's
one detector and conditional capture after the repo-owned command, complete
software gate, pushed implementation, and exact package admission exist. The
capture may perform one exact-package factory flash with ignored local Wi-Fi
input, one generated atomic hostname-plus-rotation PATCH, immediate same-origin
readback, one exact atomic restoration PATCH, confirmed restoration, normal USB
reset/re-enumeration, cleanup, and at most one exact-package recovery flash
after an initial flash effect. It may not read pool credentials, restart, mine,
control hardware, scan or discover the network, update, erase, write raw flash,
inject faults, terminate foreign processes, use direct UART, or manipulate
pins.

Evidence, privacy, recovery, and retry: use absent ignored mode-0700
`wrapper-001` and `attempt-001` roots with mode-0600 files. Origins, hostnames,
rotations, settings, ports, USB/network/process identities, credentials, HTTP
bodies, serial output, and traces stay private. Public evidence is limited to
closed categories, hashes, counts, and safe booleans. Preserve the earliest
typed failure through restoration/recovery, release owned resources, and do not
retry. A failure withholds public evidence, `RESULT.md`, and promotion.

Acceptance: require exact clean package identity, one admitted Ultra 205,
trusted same-origin HTTP, exactly one combined benign mutation request with
both generated values in one readback, exactly one combined restoration request
with both originals in one readback, disabled mining and hardware control,
cleanup, private modes, independent projection validation, redaction, and the
complete mandatory gate. This does not claim settings durability, sensitive or
safety-control live mutation, networking longevity, mining, other boards, or
release readiness.

Completion review: Exact package source
`3dea210228722634360daeda1327f2676e78db3a` passed the complete software and
package gates. The sole detector and conditional capture passed on one admitted
Ultra 205, and the independently validated aggregate projection proves one
atomic hostname/rotation mutation with immediate combined readback, one atomic
restoration with combined readback, disabled mining and hardware control,
cleanup, private modes, and redaction. A naming-only redaction collision in two
digest keys was corrected to `baseline` with a production-shaped regression;
no raw value was exposed and no hardware retry occurred. Evidence source commit
`a98aa507d13797afe1e183f5d58c909ebb91da7a` is preserved; transition
`20260811T185834Z-API-003` changed only `API-003` to `verified`, and progress
synchronized to 43 of 94 active rows (45.7%). Settings durability, secret or
safety-control mutations, mining, hardware controls, reconnect longevity,
updates, other boards, and release readiness remain non-claims.

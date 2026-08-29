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

### task-parity-log001-live-retained-stream | 2026-08-11 | Verify retained log delivery and raw streaming

- [x] Select `LOG-001` as the first actionable row after recording concrete
      blockers for every earlier selector candidate.
- [x] Add a typed private-first exact-package retained-download and raw
      WebSocket correlation capture with an independently validated redacted
      projection.
- [x] Add focused behavior, failure, privacy, mode, and real-process
      regressions; run every mandatory software gate and push before hardware.
- [x] Run exactly one detector-gated Ultra 205 capture, transition only
      `LOG-001` on complete evidence, and finalize progress and task history.

Plan: `docs/parity/work-plans/20260811T190828Z-LOG-001/PLAN.md`

Authorization and hardware contract: after the plan and implementation are
clean, verified, committed, and pushed, standing task authorization permits
only `just package`, one protected `just detect-ultra205`, and, only after
detector success, one `just capture-log-buffer-evidence` invocation using the
exact paths and arguments in the immutable plan. The capture may perform one
exact-package factory flash with owner-supplied Wi-Fi credentials and
`mineonboot=false`, then bounded receive-only USB plus trusted same-origin
HTTP and `/api/ws` observation. It may not read pool credentials, mutate
settings, request a restart, mine, initialize or submit ASIC work, actuate
voltage/frequency/fan/thermal/power controls, scan or discover networks, use
OTA, erase or write raw partitions, inject faults, terminate foreign
processes, use direct UART, or manipulate pins or other electrical interfaces.

Evidence and privacy: the absent-before-use wrapper and attempt roots are
ignored mode-`0700` directories with mode-`0600` files. Raw downloads, frames,
origins, ports, USB/network identifiers, Wi-Fi values, credentials, and
process/serial traces remain private. The public projection may contain only
closed provenance, cryptographic digests, bounded counts, header/correlation
booleans, safe-state and cleanup facts, and `redaction_status=passed`.

Recovery, retry, and stop: detector failure stops before writes. Preserve the
earliest typed failure and always close the owned WebSocket and release owned
process/USB resources. There is no settings state to restore and no recovery
flash is permitted. Exactly one detector and conditional capture are
authorized. Any detector/readiness failure, child launch/exit/timeout,
malformed or missing evidence, package/session mismatch, header/body/frame
correlation failure, cleanup failure, privacy failure, or safety invariant
violation ends without retry. Accepted terminal categories are `complete`,
`hardware_blocked`, `evidence_invalid`, `timeout`, and `process_failed`.

Acceptance: `complete` requires one admitted board 205, exact package and same-
boot origin identity, passive safe state, exact upstream-compatible headers on
both bounded downloads, one plain-text `/api/ws` marker, the baseline retained
body as an exact prefix, exactly one newly retained matching marker, cleanup,
private modes, independent projection validation, and redaction. Only then may
`LOG-001` become `verified`; otherwise withhold evidence and `RESULT.md`, keep
the row `implemented`, create a truthful closure, and stop.

Verification: The immutable-plan, implementation, and final evidence gates
pass the ordered Rust sequence, Bright Builds, all 36 Bazel tests,
parity/progress, semantic redaction, pinned-reference cleanliness, selector,
task-uniqueness, privacy, modes, immutable-plan, and diff checks. Exact package
source `f1aca309239d38c1764992794cab2aa80832d037` passed the sole detector and
sole capture. The Rust validator and aggregate projection prove exact headers,
baseline-prefix preservation, a text-protocol marker retained exactly once,
disabled mining and hardware control, cleanup, private modes, and redaction.

Completion review: Evidence source commit
`0389eebc51b0a9d77596e10963bd8e386350e098` is preserved. Transition
`20260811T193954Z-LOG-001` changed only `LOG-001` to `verified`, and progress
synchronized to 44 of 94 active rows (46.8%). Reset persistence,
maximum-capacity live wrap, long-duration or multi-client streaming, mining,
hardware controls, updates, other boards, and release readiness remain
explicit non-claims.

### task-parity-rel001-partition-size-normalization-attempt-002 | 2026-08-11 | Retry canonical partition transition evidence

- [x] Commit and push the fresh immutable REL-001 plan before source changes.
- [x] Normalize only accepted ESP-IDF partition-size suffix case and prove the
      actual checked-in table plus negative drift cases through real runfiles.
- [x] Run the complete ordered gate, commit, and push the clean fix before
      hardware use.
- [x] Build the exact package and run exactly one fresh detector plus
      conditional attempt-002 capture.
- [x] Promote only `REL-001` on complete typed evidence; otherwise withhold
      evidence, record a non-verifying closure, and stop without retry.

Plan: `docs/parity/work-plans/20260811T202225Z-REL-001/PLAN.md`.

Progress basis: attempt-001 passed exact package and detector admission but
stopped before any device effect because the capture comparator required `8K`
while the package-hashed checked-in table uses ESP-IDF-equivalent spelling
`8k`. The linked prior closure supplies a closed discriminator and requires a
fresh task, immutable plan, targeted regression, and attempt ordinal.

Hardware contract:

1. `just package`
2. `test ! -e scratch/rel001-ota-slot/wrapper-002 && (umask 077; mkdir -m 700 -p scratch/rel001-ota-slot/wrapper-002 && just detect-ultra205 > scratch/rel001-ota-slot/wrapper-002/detector.stdout 2>&1)`
3. Only after command 2 succeeds:
   `test ! -e scratch/rel001-ota-slot/attempt-002 && test ! -e docs/parity/evidence/rel001-ota-slot/partition-layout-projection.json && (umask 077; just capture-partition-layout-evidence --private-root scratch/rel001-ota-slot/attempt-002 --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --detector-output scratch/rel001-ota-slot/wrapper-002/detector.stdout --projection docs/parity/evidence/rel001-ota-slot/partition-layout-projection.json --capture-timeout-seconds 360 > scratch/rel001-ota-slot/wrapper-002/capture.stdout 2> scratch/rel001-ota-slot/wrapper-002/capture.stderr)`

Objective and preconditions: prove the exact clean Ultra 205 package's
factory-to-`ota_0` transition after correcting only the known partition-size
comparison defect. Source and reference must be clean and pushed, all six
package artifacts and the exact eight-row table must pass digest admission,
the ignored Wi-Fi credential input must exist without inspection or exposure,
private/public targets must be absent, and exactly one board 205 must pass the
detector.

Allowed effects: one exact factory-package flash; replacement NVS containing
only owner-supplied Wi-Fi credentials and `mineonboot=false`; repo-owned USB
reset/re-enumeration; bounded receive-only USB and same-origin HTTP; one upload
of the exact package OTA image; and its single scheduled software restart.

Prohibited effects: mining, pool access, ASIC initialization/work, voltage,
frequency, fan, thermal, or power control; OTAWWW; rollback; erase-flash;
interrupted update; recovery upload; arbitrary raw writes; network discovery;
foreign-process termination; direct UART; and pins, pads, headers, GPIO,
probes, jumpers, soldering, or injected signals.

Evidence and privacy: wrapper/attempt roots are ignored mode-`0700`
directories with mode-`0600` artifacts. Origins, hostnames, ports, USB/network
identities, Wi-Fi values, credentials, OTA bytes, HTTP bodies, commands, and
raw traces stay private. Public evidence is aggregate-only closed schemas,
digests, counts, partition/transition booleans, disabled effect facts, cleanup,
modes, and redaction status.

Recovery, retry, and stop: detector failure stops before writes. Preserve the
earliest typed failure through cleanup. After a completed upload, leave the
same exact package in the valid slot ESP-IDF selected. Do not perform a second
flash, rollback, erase, interruption, or recovery effect. Release all owned
resources. This contract authorizes one detector and conditionally one capture
only; any failure consumes attempt-002 and withholds evidence without retry.

Acceptance: complete typed proof requires exact package/partition provenance,
safe factory baseline, reader admission before one upload, same physical
device, service loss/recovery, exact build, changed session, ordinal `N+1`,
software reset, successful boot validation, `factory` to `ota_0`, disabled
mining/hardware control, cleanup, protected modes, redaction, and independent
validation. Otherwise keep `REL-001` at `implemented` and create no result.

Verification: The actual-table and negative-drift regressions, complete ordered
software/evidence gates, one exact clean package, one detector, one conditional
capture, independent Rust validator, semantic redaction, protected modes,
cleanup, transition receipt, and synchronized progress all pass.

Completion review: Evidence source commit
`7385114678a71f6d3e46f92439d4f2c3b6a7cfeb` is preserved. Transition
`20260811T205203Z-REL-001` changed only `REL-001` to `verified`, and progress
synchronized to 45 of 94 active rows (47.9%). The exact device safely moved
from `factory` to `ota_0` with exact build recovery and boot ordinal `N+1`.
Rollback, erase, interrupted-update, OTAWWW/static-partition, other-board,
mining, hardware-control, and release-readiness parity remain non-claims.

### task-parity-rel001-live-ota-slot-transition | 2026-08-11 | Prove the Ultra 205 factory-to-OTA slot transition

- [x] Commit and push the linked immutable REL-001 plan before source changes.
- [x] Add the bounded typed OTA device-session transaction without changing
      the existing reboot interfaces.
- [x] Add the private-first aggregate partition-layout capture, independent
      validator, and behavior/privacy/process-boundary regressions.
- [x] Run the complete ordered software gate, commit, and push the clean
      implementation before hardware use.
- [x] Build the exact package, run one detector and conditional capture, and
      promote only `REL-001` on complete typed evidence; otherwise withhold
      evidence and stop without retry.

Plan: `docs/parity/work-plans/20260811T195144Z-REL-001/PLAN.md`.

Hardware contract:

1. `just package`
2. `test ! -e scratch/rel001-ota-slot/wrapper-001 && (umask 077; mkdir -m 700 -p scratch/rel001-ota-slot/wrapper-001 && just detect-ultra205 > scratch/rel001-ota-slot/wrapper-001/detector.stdout 2>&1)`
3. Only after command 2 succeeds:
   `test ! -e scratch/rel001-ota-slot/attempt-001 && test ! -e docs/parity/evidence/rel001-ota-slot/partition-layout-projection.json && (umask 077; just capture-partition-layout-evidence --private-root scratch/rel001-ota-slot/attempt-001 --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --detector-output scratch/rel001-ota-slot/wrapper-001/detector.stdout --projection docs/parity/evidence/rel001-ota-slot/partition-layout-projection.json --capture-timeout-seconds 360 > scratch/rel001-ota-slot/wrapper-001/capture.stdout 2> scratch/rel001-ota-slot/wrapper-001/capture.stderr)`

Objective and preconditions: prove the clean exact Ultra 205 package's
installed partition layout by starting from its factory app and completing one
normal update into `ota_0`. The worktree/source commit and pinned reference
must be clean and pushed, the package manifest must bind that exact source and
contain the canonical partition-table plus OTA artifacts, the ignored
`wifi-credentials.json` must exist without exposure, private/public targets
must be absent, and exactly one board 205 must pass detector admission.

Allowed effects: one exact factory-package flash; replacement NVS containing
only owner-supplied Wi-Fi credentials and `mineonboot=false`; repo-owned USB
reset/re-enumeration; bounded receive-only serial and same-origin HTTP
observation; one upload of the package's exact OTA image to
`/api/system/OTA`; and its single scheduled software restart.

Prohibited effects: mining, pool access, ASIC initialization or work, voltage,
frequency, fan, thermal, or power control; OTAWWW; rollback; erase-flash;
interrupted update; recovery upload; arbitrary raw writes; network discovery;
foreign-process termination; direct UART; pins, pads, headers, GPIO, probes,
jumpers, soldering, or injected signals.

Evidence and privacy: wrapper and attempt roots are ignored mode-0700 private
directories with mode-0600 artifacts. OTA bytes, origins, hostnames, ports,
USB and network identities, Wi-Fi values, credentials, HTTP bodies, commands,
and raw serial/process traces stay private. Public evidence is aggregate-only:
closed schemas/categories, provenance and artifact digests, bounded counts,
exact partition-contract and transition booleans, safe-state, cleanup, modes,
and redaction facts.

Recovery, retry, and stop: detector failure stops before any write. Preserve
the earliest typed failure through cleanup. After a completed upload, leave
the same exact package in whichever valid factory/OTA slot ESP-IDF selected;
do not add a second flash, rollback, erase, interruption, or recovery effect.
Release every owned USB, socket, and child-process resource. This contract
authorizes one detector and, conditionally, one capture only. Any failure
withholds public evidence and consumes the attempt without unchanged retry.

Acceptance: `complete` requires exact package/partition provenance, safe
factory baseline, reader admission before one complete OTA upload, same
physical device, service loss/recovery, exact recovered build, boot-session
change, ordinal `N+1`, software reset, successful boot validation,
`factory` to `ota_0`, mining/hardware control disabled, cleanup, protected
modes, redaction, and independent evidence validation. Otherwise record the
closed category, keep `REL-001` at `implemented`, and create no `RESULT.md`.

Verification: Focused Cargo and Bazel OTA/device-session regressions pass. The
complete ordered implementation gate passed: `cargo fmt --all`,
`cargo clippy --all-targets --all-features -- -D warnings`,
`cargo build --all-targets --all-features`, `cargo test --all-features`,
`bun scripts/bright-builds-check.ts all`, `just test`, `just parity`, and
`just parity-progress`. `just verify-redaction`, `just verify-reference`,
`git diff --check`, immutable-plan, branch/upstream, and full-diff reviews also
pass. Exact package `1e4c8a30d27e3f193d0b3f77faa157fb2b737309` passed
provenance, cleanliness, six-artifact, and digest admission. The one detector
passed. The one capture then stopped before any device effect as
`evidence_invalid`: the strict partition comparator expected `8K` while the
package-hashed checked-in table uses equivalent ESP-IDF spelling `8k`. No
public projection exists and no retry is permitted under this plan.
The complete ordered gate, redaction check, reference check, parity report, and
progress report all passed again on the closure state.

Completion review: This attempt is closed without verification at
`docs/parity/work-plans/20260811T195144Z-REL-001/CLOSURE.md`. Its exact
pre-effect comparator discriminator was resolved by successor task
`task-parity-rel001-partition-size-normalization-attempt-002`, whose evidence
source commit `7385114678a71f6d3e46f92439d4f2c3b6a7cfeb` and transition
`20260811T205203Z-REL-001` verify REL-001 without retrying attempt-001. This
predecessor is superseded and resolved; rollback, large erase,
interrupted-update, OTAWWW/static-partition, other-board, mining,
hardware-control, and release-readiness parity remain non-claims.

### task-parity-rel002-force-close-attempt-002 | 2026-08-11 | Prove interrupted OTA abort and native rollback after forced teardown

- [x] Commit and push the immutable `REL-002` attempt-002 plan before
      implementation.
- [ ] Force and observe bounded full socket teardown after the strict OTA
      prefix, with a non-cooperative real-child regression.
- [ ] Run focused and mandatory software gates; commit and push the exact
      implementation before hardware use.
- [ ] Run one protected detector and, only after admission, one protected
      attempt-002 capture on board 205.
- [ ] Validate the closed public projection and promote only `REL-002` when
      every interruption, same-device, rollback, restoration, cleanup, and
      privacy fact passes.

Plan: `docs/parity/work-plans/20260811T214737Z-REL-002/PLAN.md`.

Objective and preconditions: close only the `REL-002` rollback-enabled SDK
behavior on one connected Ultra 205. The worktree and pinned reference must be
clean, source must be pushed, normal and isolated rollback-probe packages must
bind the same source/reference, `wifi-credentials.json` remains an opaque
ignored input, and every new private/output path must be absent before use.

Authorized hardware command: after the linked plan's software gates and clean
implementation push, build its exact normal package and probe, run its one
protected `just detect-ultra205`, then conditionally run its exact protected
`just capture-sdkconfig-rollback-evidence ...` attempt-002 command once.

Allowed effects: repo-owned USB reset/re-enumeration; one exact normal factory
flash; replacement NVS containing only owner-supplied Wi-Fi credentials and
`mineonboot=false`; bounded receive-only USB and same-origin HTTP; one bounded
truncated application OTA write followed by forced host connection teardown;
one complete rollback-probe application OTA; its scheduled software restart;
and one normal probe restart that permits native ESP-IDF bootloader rollback.
If normal restoration cannot be confirmed, one exact normal factory recovery
flash is allowed only for recovery and cannot produce success evidence.

Prohibited effects: OTAWWW or SPIFFS update, erase-flash, arbitrary raw writes,
bootloader or partition-table corruption, power interruption, mining, ASIC
work or initialization, pool access, voltage, frequency, fan, thermal or power
control, network discovery, foreign-process termination, direct UART, pins,
pads, headers, GPIO, probes, jumpers, soldering, or injected signals.

Evidence and privacy: wrapper-002 and attempt-002 are ignored mode-`0700`
directories containing only mode-`0600` files. Credentials, origins,
hostnames, ports, USB/network/process identities, HTTP bodies, firmware bytes,
commands, and raw serial/child traces remain private. Only the closed redacted
v1 projection may be committed, containing provenance/digests, bounded counts,
safe booleans, typed terminal category, cleanup, modes, and redaction.

Recovery, retry, and stop: detector failure stops before writes. Preserve the
earliest typed failure through cleanup and optional exact-package recovery;
recovery remains secondary. Release every owned resource. Attempt-002 is the
only authorized fresh ordinal and is consumed by any conditional capture
start. Stop without retry on success or on `package_invalid`, `process_failed`,
`timeout`, `hardware_blocked`, `evidence_invalid`,
`interruption_not_observed`, `probe_boot_failed`, `rollback_not_observed`, or
`recovery_failed`.

Acceptance: complete only when the exact normal baseline remains unchanged
after one partial upload and the device retains the protocol abort, the same
physical device boots the admitted pending-validation probe in `ota_0` at
`N+1`, one normal restart causes native rollback to the exact normal factory
build at the next ordinal, mining and hardware control remain disabled,
cleanup/modes/redaction pass, and the typed projection validates. Otherwise
withhold evidence, create a truthful closure, and leave `REL-002` at
`implemented`.

Completion review: Superseded before implementation or hardware. Node 24 and
the real half-open child proved that FIN followed by a delayed reset closes the
client locally but leaves the peer writable half live. Experimental edits were
removed, no device or credential input was used, and attempt-002 remains
unconsumed. Continue only through a fresh immutable plan that flushes without
FIN, immediately resets, and awaits local close. See `CLOSURE.md` beside the
linked plan.

### task-parity-rel002-retained-boot-log-attempt-005 | 2026-08-11 | Admit retained probe and rollback boot semantics

- [x] Commit and push the immutable `REL-002` attempt-005 plan before
      implementation.
- [x] Replace late-serial semantic checks with exact retained probe/final boot
      log admission while preserving typed serial-delivery correlation.
- [x] Add success, missing-marker, fetch-failure, recovery, precedence,
      no-public-evidence, and privacy regressions.
- [x] Run focused and mandatory software gates; commit and push the exact
      implementation before hardware use.
- [x] Run one protected detector and, only after admission, one protected
      attempt-005 capture on board 205.
- [x] Validate the closed public projection and promote only `REL-002` when all
      interruption, same-device, rollback, restoration, cleanup, and privacy
      facts pass.

Plan: `docs/parity/work-plans/20260811T231354Z-REL-002/PLAN.md`.

Objective and preconditions: close only `REL-002` on one Ultra 205. Source,
upstream, reference, normal/probe provenance, credential opacity, and fresh
private/public paths must pass before the exact linked commands are eligible.

Authorized effects: the linked plan's one normal factory flash, replacement
NVS with owner Wi-Fi and `mineonboot=false`, bounded receive-only USB and HTTP,
one reset-aborted partial OTA, one complete rollback probe, two planned
software restarts, and conditional exact normal recovery flash. Run one fresh
detector and at most one conditional attempt-005 only after a clean push.

Prohibited effects: OTAWWW/SPIFFS update, erase, raw writes, bootloader/table
corruption, power interruption, mining, ASIC/pool activity, voltage, frequency,
fan, thermal/power control, discovery, foreign-process termination, direct
UART, pins, pads, headers, GPIO, probes, jumpers, soldering, or signals.

Evidence and privacy: wrapper-005/attempt-005 are ignored mode-`0700` roots
with mode-`0600` files. Operational device, network, credential, command,
image, and trace values remain private. Only the closed redacted v1 projection
may be committed.

Recovery, retry, and stop: detector failure stops before writes. Preserve the
earliest typed category through cleanup and optional exact-package recovery.
Any conditional capture start consumes attempt-005; release every resource and
stop without retry on success or any admitted terminal category.

Acceptance: require the exact safe normal baseline, canonical retained partial-
upload abort, same-device pending probe boot in `ota_0` at `N+1`, native
rollback to the exact factory build at the next ordinal, exact retained probe
and final boot semantics, disabled mining/control, cleanup, modes, redaction,
and valid projection. Otherwise withhold evidence, close truthfully, and keep
`REL-002` implemented.

Completion review: Verified by the single detector-gated attempt-005 against
exact pushed source `e6b260da5717bf807eb85b9cfdbb20fe54b7b3a6`.
The closed projection and independent validator prove the canonical retained
partial-upload abort, unchanged normal baseline, same-device exact probe boot
in `ota_0` at `N+1`, exact retained pending/safe semantics, native rollback to
the exact factory build at the next ordinal, disabled mining/control, cleanup,
normal restoration without recovery flash, private modes, and redaction. See
`docs/parity/work-plans/20260811T231354Z-REL-002/RESULT.md`. Residual risks and
non-claims are limited there; broader update, hardware-control, mining, other-
board, and release behavior remain separate rows.

### task-parity-cfg001-ultra205-defaults-attempt-001 | 2026-08-11 | Verify the Ultra 205 configured defaults

- [x] Seed exact typed Ultra 205 defaults with owner Wi-Fi and
      `mineonboot=false` through the ordinary flash workflow.
- [x] Retain a closed firmware attestation of all 27 loaded default fields
      without exposing any raw configured, credential, device, or network
      value.
- [x] Implement one exact-package HTTP/WebSocket observation and validation of
      every API-visible default against the same pinned model.
- [x] Add typed contracts and behavior-focused unit, real-process, failure,
      recovery, no-output, and privacy regressions.
- [x] Run the complete ordered software gate and push before hardware use.
- [x] Spend one fresh detector and at most one conditional attempt-001.
- [x] Promote only `CFG-001` after independent validation and final gates.

Plan: `docs/parity/work-plans/20260811T234907Z-CFG-001/PLAN.md`.

Permitted commands: the exact software and hardware commands in the linked
immutable plan, culminating in `just capture-ultra205-defaults-evidence` with
fresh wrapper-001/attempt-001 paths and the detector-admitted port.

Objective: prove the exact package loads all 27 pinned Ultra 205 configured
defaults while the deliberate `mineonboot=false` override keeps mining and
hardware control disabled.

Authorized effects: one exact normal package flash; replacement NVS containing
typed Ultra 205 defaults, owner Wi-Fi, and `mineonboot=false`; bounded receive-
only USB and same-origin HTTP/WebSocket observation; and at most one exact-
package recovery flash after failure. No recovery effect can create success.

Prohibited effects: mining, ASIC initialization/work or pool connections,
frequency/voltage/fan/thermal/power control, self-test execution, OTA, erase,
arbitrary raw writes, discovery, foreign-process termination, power
interruption, direct UART, pins, pads, headers, GPIO, probes, jumpers,
soldering, or injected signals.

Evidence and privacy: wrapper-001 and attempt-001 are ignored mode-`0700`
roots with mode-`0600` private files. Pool/credential/configured values,
origins, hostnames, ports, USB/network/process identities, commands, HTTP
bodies, and raw traces remain private. Only the closed redacted
`bitaxe-ultra205-defaults-evidence-v1` projection may be committed.

Recovery, retry, and stop: detector failure stops before writes. Preserve the
earliest `process_failed`, `timeout`, `hardware_blocked`, or `evidence_invalid`
category through cleanup and optional exact-package recovery. Any conditional
capture start consumes attempt-001. Release resources and stop without retry
on success or any admitted terminal category.

Acceptance: exact source/reference/package identity; one board 205; all 27
loaded defaults and every API-visible default match; retained attestation
continuity; `mineonboot=false`; disabled mining/control; cleanup; modes;
redaction; independent validation; and complete gates. Otherwise withhold
public evidence, close truthfully, and keep `CFG-001` implemented.

Verification: The complete plan-only software gate passed. A single transient
host resource-spawn failure at `just parity` passed on one boundary retry after
the preceding build quiesced; all tests, Bright Builds, parity, progress,
redaction, and reference checks are green.

Completion review: Exact-package attempt-001 passed on one detector-admitted
Ultra 205. The independently validated closed projection proves all 27 loaded
defaults, all 23 API-visible defaults in HTTP and WebSocket, retained-marker
continuity, `mineonboot=false`, disabled mining/control, cleanup, modes, and
redaction. `CFG-001` transitioned to `verified`; actuation, mining, controls,
self-test, OTA, recovery, and non-205 behavior remain separate.

### task-parity-cfg006-defaults-matrix | 2026-08-04 | Complete board defaults matrix

- [x] Add typed exact defaults for all 20 numbered upstream board seeds and the
      explicit custom seed.
- [x] Bind every discriminator to a provenance-bearing golden fixture and the
      existing board catalog.
- [x] Run focused and mandatory gates, then transition only `CFG-006` to
      `implemented` while withholding every non-205 hardware claim.

Previous plan: `docs/parity/work-plans/20260804T133030Z-CFG-006/PLAN.md`

Active plan: `docs/parity/work-plans/20260812T004157Z-CFG-006/PLAN.md`

Authorization: pure software and public upstream seed data only. No hardware,
credentials, network, settings, mining, controls, OTA, direct UART, or pins.

Verification: Focused strict Clippy, all 51 `bitaxe-config` tests through Cargo
and Bazel, the mandatory Rust sequence, Bright Builds, all 28 Bazel test
targets, parity/progress, redaction, reference cleanliness, and diff checks
passed on implementation commit `1583feb3`.

Implementation review: The bounded pure matrix implementation is complete and
`CFG-006` reached `implemented` with `unit,golden` evidence. The first result
withheld verification because live seeded defaults and runtime behavior for
non-205 profiles were conservatively treated as part of the row. No runtime
selection or hardware behavior changed.

Targeted verification continuation:

- [x] Compare the public Rust defaults matrix directly to the complete pinned
      reference CSV inventory with fail-closed parity-report validation.
- [x] Prove representative inventory, parsing, and value drift is rejected.
- [x] Run focused and mandatory gates, then promote only the declarative
      `CFG-006` row if its direct reference, golden, and catalog evidence passes.

Continuation authorization: pure software and the checked-out public pinned
reference only. No hardware, credentials, network, settings, mining, controls,
OTA, direct UART, or pins.

Continuation verification: Nine focused regressions, the full ordered Rust
sequence, Bright Builds, all 37 Bazel test targets, direct parity comparison,
progress, redaction, reference cleanliness, and diff checks passed on exact
source commit `428041800ae232955a7468c384527cde83263503`.

Completion review: `CFG-006` transitioned to `verified` with
`unit,golden,workflow` evidence. Direct parity-report validation now binds the
public Rust matrix to exactly all 20 numbered pinned CSV seeds plus the custom
seed, while existing golden and catalog checks remain independent. See
`docs/parity/work-plans/20260812T004157Z-CFG-006/RESULT.md`. Live non-205 boot,
NVS seeding, runtime selection, and hardware behavior remain separate
non-claims; no hardware or private input was accessed.

### task-parity-net001-reconnect-lifecycle-attempt-001 | 2026-08-12 | Verify the post-boot Wi-Fi reconnect lifecycle

- [x] Select `NET-001`, inspect the exact reference/implementation gap, and
      persist the linked immutable plan before implementation.
- [x] Add a pure reconnect state machine and thin ESP-IDF event worker matching
      fallback, reason, 5,000-ms retry, AP-client, DHCP-reset, and repeated-
      cycle behavior.
- [x] Add the clear-before-effect one-shot NVS probe, exact flash integration,
      typed host capture, closed public projection, and production-shaped
      regression coverage.
- [x] Run focused and mandatory software gates; commit and push the exact
      implementation before hardware use.
- [x] Run one protected detector and, only after admission, one protected
      attempt-001 capture on board 205.
- [x] Evaluate promotion against the complete acceptance contract; withhold
      public evidence and keep `NET-001` implemented because the host timing
      validator rejected the sequential recovered marker before HTTP quorum.

Plan: `docs/parity/work-plans/20260812T025425Z-NET-001/PLAN.md`.

Objective and preconditions: close only `NET-001` on one Ultra 205. Source,
upstream, reference, package provenance, credential opacity, and fresh
private/public paths must pass before the exact linked commands are eligible.

Authorized effects: one exact normal package flash; replacement NVS with the
exact Ultra 205 defaults, owner Wi-Fi, `mineonboot=false`, and one private
`netreconprobe` marker; repo-owned USB reset/re-enumeration; bounded receive-
only USB and same-origin HTTP; one clear-before-effect station disconnect; the
normal configuration-AP fallback and station reconnect; and at most one
ordinary exact-package recovery flash without the marker. The successful final
state retains the exact package, owner Wi-Fi, exact defaults, and disabled
mining.

Prohibited effects: router or RF changes, discovery, credential mutation after
boot, erase, ad hoc/raw writes, OTA, power interruption, mining, ASIC/pool
activity, voltage, frequency, fan, thermal/power control, self-test, foreign-
process termination, direct UART, pins, pads, headers, GPIO, probes, jumpers,
soldering, or signals.

Evidence and privacy: wrapper-001/attempt-001 are ignored mode-`0700` roots
with mode-`0600` files. Credential, hostname, USB, device, network, origin,
command, process, HTTP-body, and raw serial values remain private. Only the
closed redacted `bitaxe-network-reconnect-evidence-v1` projection may be
committed.

Recovery, retry, and stop: the probe marker must be erased and committed before
the intentional disconnect. Detector failure stops before writes. Preserve the
earliest typed category through cleanup and optional ordinary exact-package
recovery. Any conditional capture start consumes attempt-001; release every
resource and stop without retry on success or any admitted terminal category.

Acceptance: require the exact package and board identity, one post-boot
disconnect, immediate AP fallback, typed reason, a first retry no earlier than
5,000 ms, same-boot DHCP recovery, retry reset, client-only mode, 15,000 ms of
stable service, final HTTP/retained-log quorum, `mineonboot=false`, disabled
mining/control, cleanup, modes, redaction, and valid projection. Otherwise
withhold evidence, close truthfully, and keep `NET-001` implemented.

Verification: Immutable-plan gate passed in order: `cargo fmt --all`, strict
Clippy, the all-target/all-feature Cargo build and tests, Bright Builds, all 37
Bazel test targets, parity, progress, redaction, pinned-reference, selector,
task-uniqueness, reference-cleanliness, and diff checks. The selector now
returns this exact `NET-001` plan as the sole open plan. No hardware, package,
credential, NVS, or network effect occurred.

Completion review: Attempt-001 consumed its sole detector and hardware budget.
The device emitted exactly one disconnect, a first retry after 5,033 ms, DHCP
recovery, client-only return, and 15,026 ms of stability. The host validator
incorrectly required connected and recovered log calls to share one exact
millisecond; their valid 28-ms separation produced `reconnect_timing_invalid`
before final HTTP quorum. Public evidence was withheld, ordinary exact-package
recovery succeeded, private modes passed, and `NET-001` remains implemented.
See `docs/parity/work-plans/20260812T025425Z-NET-001/CLOSURE.md`. A new immutable
plan and ordinal are required for another hardware proof. The linked
attempt-002 task subsequently produced accepted evidence and supersedes this
non-verifying attempt.

### task-parity-net001-reconnect-lifecycle-attempt-002 | 2026-08-12 | Complete the corrected live reconnect proof

- [x] Select `NET-001`, inspect the attempt-001 closure and corrected timing
      regression, and persist the linked immutable attempt-002 plan.
- [x] Run focused and mandatory plan-only software gates; commit and push the
      exact immutable plan before package or hardware use.
- [x] Make the real-child stdout regression launcher-independent, rerun the
      complete gate, and commit and push the focused execution change.
- [x] Build the exact clean execution-commit package, run one protected detector,
      and only after admission run one protected attempt-002 capture.
- [x] Promote only `NET-001` when the live lifecycle reaches final same-origin
      HTTP/retained-log quorum and the closed public projection independently
      validates with safety, cleanup, private-mode, and redaction facts.

Plan: `docs/parity/work-plans/20260812T034058Z-NET-001/PLAN.md`.

Objective and preconditions: close only `NET-001` using the corrected bounded
connected-to-recovered timing predicate. Source, upstream, reference, package
provenance, credential opacity, and fresh wrapper-002/attempt-002/public paths
must pass before the linked effect commands are eligible.

Authorized effects: one exact normal package flash; replacement NVS with exact
Ultra 205 defaults, owner Wi-Fi, `mineonboot=false`, and one private
`netreconprobe` marker; repo-owned USB reset/re-enumeration; bounded receive-
only USB and same-origin HTTP; one clear-before-effect station disconnect; the
normal configuration-AP fallback and station reconnect; and at most one
ordinary exact-package recovery flash without the marker. A successful final
state retains the exact package, owner Wi-Fi, exact defaults, and disabled
mining.

Prohibited effects: router or RF changes, discovery, credential mutation after
boot, erase, ad hoc/raw writes, OTA, power interruption, mining, ASIC/pool
activity, voltage, frequency, fan, thermal/power control, self-test, foreign-
process termination, direct UART, pins, pads, headers, GPIO, probes, jumpers,
soldering, or signals.

Evidence and privacy: wrapper-002/attempt-002 are ignored mode-0700 roots with
mode-0600 files. Credential, hostname, USB, device, network, origin, command,
process, HTTP-body, and raw serial values remain private. Only the closed
redacted `bitaxe-network-reconnect-evidence-v1` projection may be committed.

Recovery, retry, and stop: the probe marker must be erased and committed before
the intentional disconnect. Detector failure stops before writes. Preserve the
earliest typed category through cleanup and optional ordinary exact-package
recovery. Any conditional capture start consumes attempt-002; release every
resource and stop without retry on success or any admitted terminal category.

Acceptance: require exact package and board identity, one post-boot disconnect,
immediate AP fallback, typed reason, a first retry no earlier than 5,000 ms,
same-boot DHCP recovery, retry reset, client-only mode, 15,000 ms of stable
service, final HTTP/retained-log quorum, `mineonboot=false`, disabled mining and
control, cleanup, modes, redaction, and valid projection. Otherwise withhold
evidence, close truthfully, and keep `NET-001` implemented.

Verification: The canonical timing and real-child stdout regressions passed;
an additional direct-Bun run diagnosed the launcher-dependent fixture now
scheduled after the plan commit. The ordered Cargo format, strict Clippy, all-
target build, all-feature test, Bright Builds, 37-target Bazel, parity,
progress, redaction, reference, generated-contract, selector, task-uniqueness,
fresh-path, reference-cleanliness, and diff gates passed. The selector returns
only the linked attempt-002 plan and its immutable SHA-256 is
`42a402866befc801bc635aeb367a381d4473aec72846798bafd8176fb83a95f9`.

Completion review: Attempt-002 used one detector and one conditional capture.
The exact `e56afbe4` package proved one disconnect, immediate fallback, first
retry after 5,022 ms, same-boot DHCP recovery, retry reset, client-only return,
15,000-ms stability, final HTTP/build quorum, disabled effects, cleanup,
private modes, and redaction. The independent Rust validator accepted the
public projection, no recovery flash was needed, and only `NET-001` was
promoted to verified. Repeated reconnects, provisioning-client suppression,
IPv6, router/RF failures, other boards, mining, controls, updates, recovery,
and release readiness remain non-claims.

### task-parity-net002-provisioning-network-attempt-006 | 2026-08-12 | Separate persisted preference from runtime safety

- [x] Select `NET-002` and bind the continuation to attempt-005's closed live
      network quorum.
- [x] Remove only the invalid `startMiningOnBoot === false` postcondition and
      add paired preference/runtime-safety regressions.
- [x] Run every mandatory software gate, commit, and push.
- [x] Run one detector and at most one conditional attempt-006 with fresh
      protected paths.
- [x] Promote only after the complete immutable-plan success quorum passes.

Plan: `docs/parity/work-plans/20260812T073939Z-NET-002/PLAN.md`.

Authorization: the linked plan's bounded USB, exact CoreWLAN, local network,
cleanup, and exact recovery effects only. Capture start consumes the ordinal.

Verification: Plan-only ordered Cargo, Bright Builds, all 37 Bazel tests,
parity/progress, redaction, reference, generated contracts, selector, task,
immutable-plan, fresh-path, reference-cleanliness, and diff gates pass.
Immutable plan SHA-256 is
`b886ba70f6e7e8058e17d7342d104eddbaf63759921ea1c8381e38b4af60afcc`.
Implementation verification passes focused automation, exact package build,
ordered Cargo, Bright Builds, all 37 Bazel tests, parity/progress, redaction,
reference, generated contracts, selector, immutable-plan, fresh-path, and diff
checks.

Completion review: Attempt-006 passed the complete immutable-plan quorum and
published independently validated redacted evidence for exact-device CoreWLAN
association, DHCP, wildcard DNS, captive redirect, system-info, disabled
runtime mining/control, cleanup, and exact-package recovery. The evidence was
committed, `NET-002` transitioned to verified, and progress synchronized to 50
of 94 active rows. Credential submission, station handoff, repeated
provisioning, other boards, mining, controls, updates, and release readiness
remain non-claims.

### task-parity-net003-scan-ipv6 | 2026-08-04 | Implement Wi-Fi scan and IPv6 reporting

- [x] Add the bounded 20-network scan response and explicit numeric auth-mode
      plus link-local/global IPv6 projection contracts.
- [x] Retain one ESP-IDF Wi-Fi owner for exclusive scans, restore AP-only mode,
      register the access-gated endpoint, and publish station-only IPv6 events.
- [x] Add focused/ownership regressions, build the real firmware, run every
      mandatory gate, and transition only `NET-003` to `implemented`.

Plan: `docs/parity/work-plans/20260804T170000Z-NET-003/PLAN.md`

Authorization: local software and build work only. No hardware attempt,
credentials, external network request, mining, ASIC traffic, voltage/fan/power
effect, OTA, recovery, direct UART, or pins.

Verification: Five focused scan/IPv6 tests, 23 API-comparator tests, six Wi-Fi
ownership tests, the synthetic route fixture, and the real ESP32-S3 build pass.
The ordered Rust sequence, Bright Builds, all 30 Bazel targets, API compare,
parity/progress, redaction, reference cleanliness, and diff checks also pass.

Completion review: The bounded scan and station IPv6 implementation is
complete. Exact-package attempt-001 later proved 20 bounded scan records,
same-session connected client-only service, and stable unique-local v6
reporting. `NET-003` transitioned to verified under
`20260812T083128Z-NET-003`; this implementation task is complete and archived.

### task-parity-net003-live-scan-address-attempt-001 | 2026-08-12 | Verify live Wi-Fi scan and station v6 reporting

- [x] Add a closed typed capture and validator that admit one exact-package
      same-session scan plus before/after connected station-address state.
- [x] Prove aggregate scan shape, numeric auth/signal bounds, exact identity,
      connection preservation, stable v6 reporting, private modes, and public
      redaction without exposing observed network or device identifiers.
- [x] Run every software gate, push the exact package, spend one detector and
      at most one attempt-001, then promote only on the complete live quorum.

Plan: `docs/parity/work-plans/20260812T080258Z-NET-003/PLAN.md`.

Hardware contract: the only permitted live command is the exact detector and
conditional `just capture-network-scan-evidence` sequence in the linked plan.
Its objective is one exact-package board-205 boot followed by private system-
info, exactly one same-origin Wi-Fi scan, and private post-scan system-info.
The ignored wrapper and attempt roots are `ProtectedOperational`, mode `0700`
with mode-`0600` files; only closed aggregates and safe provenance may enter
the public projection. The allowed effects are the normal exact-package flash,
repo-owned USB reset/re-enumeration, passive serial receive, same-origin HTTP
reads, and one bounded scan. No credential mutation, host network discovery,
association change, repeated scan, erase, raw write, OTA, restart, mining,
ASIC work, hardware control, direct UART, or pins is authorized.

Recovery and stop rules: preserve the earliest typed failure and perform at
most one ordinary exact-package recovery flash after a started flash effect;
recovery cannot create success. Detector failure, missing input, launch
failure, timeout, non-ready hardware, malformed evidence, scan failure, absent
or invalid station address, lost service, cleanup failure, or any privacy
violation stops the attempt, withholds evidence, consumes ordinal 001, and
forbids an unchanged retry. Accepted terminal categories are `process_failed`,
`timeout`, `hardware_blocked`, `evidence_invalid`, and
`service_recovery_failed`.

Verification: Plan-only and implementation gates passed ordered Cargo, Bright
Builds, all 37 Bazel tests, the real ESP32-S3 image, parity/progress, redaction,
reference, generated contracts, selector, task, immutable-plan, fresh-path,
reference-cleanliness, and diff checks. Immutable plan SHA-256 is
`071a2b0a2d0a6b2ab84fcc854d8cefe765a194c47b5bf588b10014c9810bada2`.
Attempt-001 additionally passed independent Rust validation, private-mode and
no-holder checks, and semantic redaction.

Completion review: Attempt-001 passed with one exact-package boot, 20 bounded
exact-shape scan records, same-session connected client-only service, stable
unique-local v6 reporting, monotonic uptime, disabled mining/control, cleanup,
no recovery, private modes, independent validation, and redaction. The
evidence was committed, `NET-003` transitioned to verified, progress
synchronized to 51 of 94 active rows, and this task is complete and archived.

### task-parity-asic002-sealed-initialization-promotion | 2026-08-12 | Promote sealed BM1366 initialization proof

- [x] Add a typed, closed projector and validator for the sealed accepted-share
      campaign's full BM1366 initialization boundary.
- [x] Prove protected artifact seals/modes, all nine preparation completions,
      retained production UART, trusted package/runtime identity, live work,
      safe stop, cleanup, and current-source compatibility without exposing
      protected values.
- [x] Run every software and privacy gate, publish one redacted projection,
      and promote only `ASIC-002` when the complete quorum passes.

Plan: `docs/parity/work-plans/20260812T083542Z-ASIC-002/PLAN.md`.

Authorization: projection-only read access to the ignored protected
`scratch/ultra205-accepted-pool-share/attempt-007` artifacts. No detector,
flash, reset, USB session, credential read, serial/network request, mining,
fan/voltage/power/ASIC actuation, recovery, direct UART, pins, or other
hardware effect is permitted. Preserve raw artifacts unchanged; only closed
redaction-safe categories, counts, digests, commits, and booleans may be
published.

Verification and stop rule: require valid seals, mode 0700 root and mode 0600
files, exact accepted campaign state, 18 accepted and zero invalid preparation
events ending at completed production-UART retention, byte-identical current
initialization paths, focused regressions, independent Rust validation, every
repository gate, redaction, and reference cleanliness. Any failure leaves
`ASIC-002` implemented and stops this plan without a hardware fallback.

Completion review: The sealed exact-package attempt proved all nine ordered
BM1366 initialization steps, exactly one chip, mining-ready completion,
retained production UART, live accepted work, trusted identity and safety,
safe stop, cleanup, protected modes, source compatibility, independent
validation, and redaction. Evidence commit
`5694c245622ceb15dd7f3924cac7327f5d99bf1c` was pushed, `ASIC-002`
transitioned to verified under `20260812T090906Z-ASIC-002`, progress
synchronized to 52 of 94 active rows, and this task is complete and archived.

### task-parity-asic003-sealed-work-send-promotion | 2026-08-12 | Promote sealed BM1366 work-send proof

- [x] Add a typed, closed projector and validator that derive ASIC work-send
      proof from the committed sealed initialization projection.
- [x] Prove exact work-module compatibility, bounded production dispatch/UART-
      write span compatibility, live qualified accepted work, safety, cleanup,
      and redaction without exposing work or operational values.
- [x] Run every software and privacy gate, publish one redacted projection,
      and promote only `ASIC-003` when the complete quorum passes.

Plan: `docs/parity/work-plans/20260812T091446Z-ASIC-003/PLAN.md`.

Authorization: read-only use of committed public evidence and Git history
only. No protected campaign read, detector, flash, reset, USB session,
credential read, serial/network request, mining, fan/voltage/power/ASIC
actuation, recovery, direct UART, pins, or other hardware effect is permitted.
Only closed redaction-safe categories, counts, digests, commits, and booleans
may be published.

Verification and stop rule: require a valid committed ASIC initialization
projection, exact digest and commit binding, byte-identical BM1366 work/
production/command modules, compatible bounded worker-dispatch and adapter-
write spans, focused regressions, independent Rust validation, every
repository gate, redaction, and reference cleanliness. Any failure leaves
`ASIC-003` implemented and stops this plan without a hardware fallback.

Completion review: The source-bound exact-package proof established fixed
BM1366 work encoding, gated production dispatch and UART write, a qualified
correlated result, accepted submit response, safety, cleanup, source
compatibility, independent validation, and redaction. Evidence commit
`12e6941cc7b61cbb5a0d3571587fa242cadfce57` was pushed, `ASIC-003`
transitioned to verified under `20260812T093510Z-ASIC-003`, progress
synchronized to 53 of 94 active rows, and this task is complete and archived.

### task-parity-asic004-sealed-result-parsing-promotion | 2026-08-12 | Promote sealed BM1366 result-parsing proof

- [x] Add a typed, closed projector and validator that derive ASIC result-
      parsing proof from the committed sealed work-send projection.
- [x] Prove strict live nonce parsing/correlation, unchanged transcript and
      compatible accepted-result spans, typed soft discards, safety, cleanup,
      and redaction without exposing result or operational values.
- [x] Run every software and privacy gate, publish one redacted projection,
      and promote only `ASIC-004` when the complete quorum passes.

Plan: `docs/parity/work-plans/20260812T093928Z-ASIC-004/PLAN.md`.

Authorization: read-only use of committed public evidence and Git history
only. No protected campaign read, detector, flash, reset, USB session,
credential read, serial/network request, mining, fan/voltage/power/ASIC
actuation, recovery, direct UART, pins, or other hardware effect is permitted.
Only closed redaction-safe categories, constants, digests, commits, and
booleans may be published.

Verification and stop rule: require a valid committed ASIC work-send
projection, exact digest and commit binding, unchanged transcript module,
compatible bounded parser/adapter/worker/correlation spans, focused
regressions, independent Rust validation, every repository gate, redaction,
and reference cleanliness. Any failure leaves `ASIC-004` implemented and
stops this plan without a hardware fallback.

Completion review: The source-bound exact-package proof established strict
11-byte BM1366 frame admission, nonce decoding, compatible adapter/worker/
correlation behavior, eight typed soft-discard categories, a live qualified
result followed by an accepted response, safety, cleanup, independent
validation, and redaction. Evidence commit
`06067c6240558811073c6a3a71cea5dad2432250` was pushed, `ASIC-004`
transitioned to verified under `20260812T101604Z-ASIC-004`, progress
synchronized to 54 of 94 active rows, and this task is complete and archived.

### task-parity-asic005-serial-transport-promotion | 2026-08-12 | Verify the accepted BM1366 serial transport

- [x] Select `ASIC-005` from a clean synchronized preflight and bind the
      immutable plan to the accepted hardware lineage.
- [x] Add and verify a closed public contract that joins the validated live
      work-send and result-parsing projections through the unchanged UART
      transport.
- [x] Publish the redacted evidence, promote only `ASIC-005` when the complete
      quorum passes, synchronize progress, and archive this task.

Plan: `docs/parity/work-plans/20260812T102245Z-ASIC-005/PLAN.md`.

Authorization: Software-only evidence derivation. No hardware, USB, network,
credentials, protected artifacts, raw traces, or direct electrical interfaces
are authorized or required by this plan.

Verification: Plan-only gate passed after one bounded retry of a transient
host-resource error: ordered Cargo, Bright Builds, all 37 Bazel tests,
parity/progress, redaction, reference, reference-cleanliness, task-uniqueness,
and diff checks are green. Immutable plan SHA-256 is
`f08426c24227ea69502135a472811d99bbc7ad5f559159a1956f123a8baeb641`.
Focused Rust contract tests and the canonical Bazel automation target pass,
including TypeScript compilation, malformed/incomplete source rejection,
module/span/dirty-path drift rejection, typed launch failure, sensitive-output
guarding, and real-child validator/file behavior. Full implementation gates
passed: ordered Cargo, Bright Builds, all 37 Bazel tests, parity/progress, the
real ESP32-S3 package, generated contracts, redaction, reference, immutable-
plan, reference-cleanliness, and diff checks are green.
The committed projector accepted both exact prerequisite digests, unchanged
UART and adapter modules, compatible bounded production TX/RX spans, current
transport constants and failure semantics, and atomically published the mode-
0644 independently validated projection at SHA-256
`bad828db694ee59c4ef3d77b2e58ef89e0195ef382526b97912d0a71e882ad69`.
Transition `20260812T103956Z-ASIC-005` promoted only `ASIC-005`; deterministic
progress now records 55 of 94 active rows (58.5%). The final ordered Cargo,
Bright Builds, all 37 Bazel tests, parity/progress, redaction, reference,
independent evidence validation, immutable-digest, cleanliness, and diff gates
passed; `just parity` required one bounded retry after the recurring transient
macOS resource error and then reported no validation errors.

Completion review: The source-bound proof joins live accepted production work
TX and qualified result RX through unchanged UART ownership and bounded current
transport semantics, with independent validation, atomic publication, cleanup,
and redaction. No hardware rerun or protected evidence access occurred. The
evidence and transition commits were pushed, the final repository gate passed,
and the row is verified. This task is complete and archived.

### task-parity-asic007-frequency-transition-promotion | 2026-08-12 | Verify the accepted BM1366 frequency ramp

- [x] Select `ASIC-007` from a clean synchronized preflight and bind an
      immutable plan to the accepted conservative Ultra 205 lineage.
- [x] Add and verify a closed public contract that proves the bounded
      50-to-400-MHz ramp completed before accepted live work.
- [x] Publish the redacted projection, promote only `ASIC-007` when the full
      quorum passes, synchronize progress, and archive this task.

Plan: `docs/parity/work-plans/20260812T104903Z-ASIC-007/PLAN.md`.

Authorization: Software-only derivation from committed public evidence and Git
history. No protected campaign access, detector, package build, flash, reset,
USB/network session, credentials, mining, fan/voltage/power/ASIC actuation,
recovery, direct UART, pins, or other hardware effect is permitted.

Verification and stop rule: Require the exact independently validated ASIC-002
projection digest, accepted/current/reference commit binding, unchanged ramp
and actuation modules, compatible unique executor spans, focused regressions,
independent Rust validation, every repository gate, redaction, reference
cleanliness, and atomic publication. Any failure leaves `ASIC-007` implemented
and stops this plan without a hardware fallback.

Verification: Focused contract and automation tests, real-child integration,
ordered Cargo, Bright Builds, all 37 Bazel tests, parity/progress, generated
contracts, independent projection validation, redaction, reference, immutable
digests, source compatibility, task uniqueness, and diff checks pass. Public
projection SHA-256 is
`34ac6bc0df593bd75b6026eedcecda5f4b34e00cde0f3541a156794f2c7512ae`.
Transition `20260812T110614Z-ASIC-007` promoted only `ASIC-007`; progress now
records 56 of 94 active rows (59.6%). The final ordered Cargo, Bright Builds,
all 37 Bazel tests, parity/progress, generated contracts, independent evidence
validation, redaction, reference, immutable-digest, task-uniqueness,
reference-cleanliness, and diff gates pass.

Completion review: The source-bound proof establishes that the accepted
conservative Ultra 205 completed all 56 ramp writes and delays before live
initialized work and an accepted response, followed by confirmed safe stop and
cleanup. No protected evidence or new hardware effect occurred. The evidence
and transition commits were pushed, the final repository gate passed, and the
row is verified. This task is complete and archived.

### task-parity-str001-socket-promotion | 2026-08-12 | Verify the accepted Stratum v1 socket path

- [x] Select `STR-001` from a clean synchronized preflight and bind an
      immutable plan to the accepted conservative Ultra 205 lineage.
- [x] Add and verify a closed public contract that proves the accepted session
      traversed the unchanged bounded production TCP adapter.
- [x] Publish the redacted projection, promote only `STR-001` when the full
      quorum passes, synchronize progress, and archive this task.

Plan: `docs/parity/work-plans/20260812T111705Z-STR-001/PLAN.md`.

Authorization: Software-only derivation from committed public evidence and Git
history. No protected campaign access, detector, package build, flash, reset,
USB/network session, credentials, mining, pool contact, fan/voltage/power/ASIC
actuation, recovery, direct UART, pins, or other hardware effect is permitted.

Verification and stop rule: Require the exact independently validated ASIC-002
projection digest, accepted/current/reference commit binding, unchanged TCP
transport module, compatible unique owner/lifecycle semantics, focused
regressions, independent Rust validation, every repository gate, redaction,
reference cleanliness, and atomic publication. Any failure leaves `STR-001`
implemented and stops this plan without a hardware fallback.

Verification: Focused Rust and TypeScript contracts, real-child integration,
ordered Cargo, Bright Builds, all 37 Bazel tests, parity/progress, generated
contracts, independent projection validation, redaction, reference, immutable
digests, source compatibility, task uniqueness, and diff checks pass. Public
projection SHA-256 is
`dcb3eed396a268114b017d7ef4fbca9c427a390d7acf405fc52fbef6472122b8`.
Transition `20260812T113857Z-STR-001` promoted only `STR-001`; progress now
records 57 of 94 active rows (60.6%). The final ordered Cargo, Bright Builds,
all 37 Bazel tests, parity/progress, generated contracts, independent evidence
validation, redaction, reference, immutable-digest, task-uniqueness,
reference-cleanliness, and diff gates pass. `just parity` required one bounded
retry after the recurring transient macOS resource error and then reported no
validation errors.

Completion review: The source-bound proof establishes that the accepted
conservative Ultra 205 traversed the unchanged production TCP adapter through
an authorized Stratum v1 lifecycle to a real accepted submit response, then
completed safe stop and cleanup. No protected evidence or new hardware effect
occurred. The evidence and transition commits were pushed, the final repository
gate passed, and the row is verified. This task is complete and archived.

### task-parity-str006-protocol-coordinator-promotion | 2026-08-12 | Verify the accepted production protocol coordinator

- [x] Select `STR-006` from a clean synchronized preflight and bind an
      immutable plan to the accepted conservative Ultra 205 lineage.
- [x] Add and verify a closed public contract joining the accepted socket,
      hardware-preparation, work-send, result, and safe-stop lifecycle.
- [x] Replace the false single-occurrence ASIC-worker guard with the exact two
      legitimate dispatch spans and a production-shaped regression.
- [x] Publish the redacted projection, promote only `STR-006` when the full
      quorum passes, synchronize progress, and archive this task.

Active plan: `docs/parity/work-plans/20260812T122256Z-STR-006/PLAN.md`.

Closed predecessor:
`docs/parity/work-plans/20260812T114949Z-STR-006/CLOSURE.md`.

Authorization: Software-only derivation from committed public evidence and Git
history. No protected campaign access, detector, package build, flash, reset,
USB/network session, credentials, mining, pool contact, fan/voltage/power/ASIC
actuation, recovery, direct UART, pins, or other hardware effect is permitted.

Verification and stop rule: Require all four exact independently validated
source projection digests, shared accepted/current/reference lineage,
unchanged coordinator/recovery/owner modules, compatible unique lifecycle
semantics, focused regressions, independent Rust validation, every repository
gate, redaction, reference cleanliness, and atomic publication. Any failure
leaves `STR-006` implemented and stops this plan without a hardware fallback.

Verification: Focused contract and production-shaped projector tests,
real-child integration, ordered Cargo, Bright Builds, all 37 Bazel tests,
parity/progress, generated contracts, independent projection validation,
redaction, reference, immutable digests, source compatibility, task uniqueness,
and diff checks pass. Public projection SHA-256 is
`f008171f26b7a8ae6b08859e3cfef4f0c5bf88937c049dd66b6f868c9bbfd6f7`.
Transition `20260812T123922Z-STR-006` promoted only `STR-006`; progress now
records 58 of 94 active rows verified (61.7%).

Completion review: The source-bound proof establishes that the accepted
conservative Ultra 205 traversed the unchanged single-owner coordinator from
all six readiness gates through hardware preparation, authorized pool
operation, initialized ASIC dispatch, qualified result correlation, a real
accepted submit response, ordered safe stop, watchdog feeding, and cleanup. No
protected evidence or new hardware effect occurred. The evidence and
transition commits were pushed and the row is verified. This task is complete
and archived.

### task-parity-str007-mining-criteria-promotion | 2026-08-12 | Verify bounded mining smoke and soak criteria

- [x] Select `STR-007` from a clean synchronized preflight and separate the
      criteria claim from the terminal attempt-004 continuity task.
- [x] Add and verify a closed public contract joining the committed Phase 21
      smoke/soak proof, verified coordinator, and current fail-closed criteria.
- [x] Publish the redacted projection, promote only `STR-007`, synchronize
      progress, and archive this task when every gate passes.

Active plan: `docs/parity/work-plans/20260812T133713Z-STR-007/PLAN.md`.

Closed predecessors:
`docs/parity/work-plans/20260812T124802Z-STR-007/CLOSURE.md` and
`docs/parity/work-plans/20260812T132655Z-STR-007/CLOSURE.md`.

Authorization: Software-only derivation from committed public evidence and Git
history. No protected campaign access, attempt-005, detector, package build,
flash, reset, USB/network session, credentials, mining, pool contact,
fan/voltage/power/ASIC actuation, recovery, direct UART, pins, or other hardware
effect is permitted.

Verification and stop rule: Require exact admitted public digests and closed
facts, independent validators, current source identity and cleanliness, unique
current criteria spans, focused and real-child regressions, every repository
gate, redaction, reference cleanliness, and atomic publication. Any failure
leaves `STR-007` implemented and stops without hardware fallback or reopening
the terminal default-profile soak task.

Verification: Focused contract, projector, wrapper-shape, absolute validator-
boundary, and real-child tests pass. Ordered Cargo, Bright Builds, all 37 Bazel
targets, parity/progress, independent validation, redaction, reference,
immutable digests, mode, candidate absence, source and task cleanliness, and
diff checks pass. Public projection SHA-256 is
`c1ccb65e6a49d04049aabb2be1295949163526a197e20e3de51fc65d38c2a80f`.
Transition `20260812T135247Z-STR-007` promoted only `STR-007`; progress now
records 59 of 94 active rows verified (62.8%).

Completion review: The sealed proof joins the committed Phase 21 controlled
no-share smoke and approved 300-second bounded soak, verified coordinator
compatibility, and current exact 600-second upstream-default fail-closed
criteria. It does not reopen or satisfy the terminal attempt-004 continuity
task. No protected evidence or new hardware effect occurred. The evidence was
independently validated and the row is verified. This task is complete and
archived.

### task-parity-pwr001-asic-reset-evidence-audit | 2026-08-12 | Audit ASIC reset hardware evidence

Plan: `docs/parity/work-plans/20260812T185214Z-PWR-001/PLAN.md`.

- [x] Add and validate a typed redacted PWR-001 projection derived from the
      sealed accepted-share attempt and validated ASIC-002 projection.
- [x] Fix the selector's terminal-closed-lineage reconciliation order with a
      regression that preserves genuine multi-row and unlinked-plan failures.
- [x] Prove active-low 100 ms/100 ms reset semantics, the completed reset-and-
      detect boundary, exactly one downstream BM1366 response, accepted work,
      fail-closed hold-low, safe stop, cleanup, and unchanged owning paths.
- [x] Run all mandatory gates and promote PWR-001 only on the complete closed
      quorum without another hardware attempt.

Hardware contract: No hardware interaction is permitted or required by this
task. It may read the ignored protected attempt only through the existing
projection boundary and may publish only closed, redacted facts. Detector,
flash, reset, USB, serial, network, credentials, mining, GPIO, direct UART,
pins, voltage, fan, power, and fault injection are prohibited.

Verification: The focused Rust contract, selector, TypeScript projector,
failure-withholding, real-child, and repository redaction regressions pass.
The ordered Cargo checks, Bright Builds, all 41 Bazel tests, parity/progress,
independent source and final validators, redaction across 14 semantic
artifacts, pinned-reference cleanliness, generated contracts, immutable
digests, exact source compatibility, private-candidate absence, public mode,
task binding, and diff checks pass. Projection SHA-256 is
`11bb816e6f6e2393b796b13c49ae7db5d181f719dc94898ca00e17ce384d469b`.
Transition `20260812T193339Z-PWR-001` promoted only `PWR-001`; progress records
60 of 94 active rows verified (63.8%).

Completion review: The source-bound proof establishes that the exact admitted
Ultra 205 executed the unchanged production active-low 100 ms/100 ms reset,
received one BM1366 response, advanced to accepted work, and completed
fail-closed safe stop and cleanup. It does not claim electrical waveform or
scheduler measurement. No protected evidence or new hardware effect occurred.
The evidence checkpoint was pushed and the row is verified. This task is
complete and archived.

### task-parity-pwr002-asic-power-initialization-audit | 2026-08-12 | Verify ASIC power initialization

- [x] Select `PWR-002` as the first actionable parity row after the temporarily
      unavailable physical-observation-gated `API-009` candidate.
- [x] Audit the pinned reference, production preparation and rollback
      transactions, accepted Ultra 205 campaign, ASIC-002 projection, and
      source-compatibility boundary.
- [x] Commit and push the immutable plan at
      `docs/parity/work-plans/20260812T193941Z-PWR-002/PLAN.md` before source
      implementation.
- [x] Add the Rust-owned `bitaxe-asic-power-initialization-evidence-v1`
      contract, independent validator, generated binding, closed projector,
      command surface, and behavior regressions.
- [x] Publish and independently validate one aggregate-only PWR-002 projection
      from the accepted ASIC-002 evidence without another hardware run.
- [x] Run the ordered repository gates, transition only `PWR-002` when the
      complete quorum passes, synchronize progress, record the result, archive
      this task, and push every audited commit.

Evidence and privacy: The only public output may contain schemas, commits,
SHA-256 digests, closed categories, fixed safe constants, bounded counts, and
booleans. It must not expose hostnames, origins, ports, USB identities, network
identifiers, credentials, raw traces, or ignored/private paths. The projector
must validate the existing source projection and its exact digest, source and
reference identities, immutable plan/task lineage, package/runtime trust,
complete nine-step initialization, downstream accepted work, safe stop,
cleanup, no hardware rerun, and source compatibility before atomic publication.

Safety and authorization: This is a software-only audit of an already accepted
hardware campaign. It authorizes no detector, flash, USB/serial session,
network request, credential access, mining rerun, GPIO/pin action, power,
voltage, fan, reset, direct UART, fault injection, or other device effect. It
does not claim analog voltage accuracy, electrical timing or waveforms,
automatic fan behavior, arbitrary profile behavior, or physically injected
fault recovery.

Promotion: Keep `PWR-002` at `implemented` unless the typed projection and all
repository gates prove the exact admitted firmware completed the closed
fresh-safety, fan/RPM, 1100 mV, 500 ms stabilization, active-low ASIC-enable,
reset/detect, mining-ready, retained-UART transaction, successful downstream
work, safe stop, and cleanup. On any incomplete or ambiguous boundary, withhold
the final projection, record the blocker, and stop without hardware.

Verification: The focused Rust contract, TypeScript projector,
failure-withholding, production-shaped ambiguity, real-child, and repository
redaction regressions pass. The ordered Cargo checks, Bright Builds, all 41
Bazel tests, parity/progress, independent source and final validators,
redaction across 15 semantic artifacts, pinned-reference cleanliness,
generated contracts, immutable digests, source compatibility, private-candidate
absence, public mode, task binding, and diff checks pass. Projection SHA-256 is
`0668c274d09b3e39d7d5edfea4b2e66c97248ff77de9192981f3af00e547ddfe`.
Transition `20260812T202359Z-PWR-002` promoted only `PWR-002`; progress records
61 of 94 active rows verified (64.9%).

Completion review: The source-bound proof establishes that the exact admitted
Ultra 205 completed the conservative 400 MHz, 1100 mV, and 100% fan power
transaction, all nine preparation steps, exactly-one-chip initialization,
accepted work, rollback semantics, safe stop, and cleanup. It does not claim
analog accuracy, electrical waveforms, arbitrary profiles, automatic fan
behavior, injected fault recovery, or thermal/soak closure. No protected
evidence or new hardware effect occurred. The evidence checkpoint was pushed,
the row is verified, and this task is complete and archived.

### task-parity-pwr003-core-voltage-control-audit | 2026-08-12 | Audit Ultra 205 core-voltage control

- [x] Select PWR-003 as the first actionable parity row and freeze the
      immutable plan at
      `docs/parity/work-plans/20260812T203223Z-PWR-003/PLAN.md`.
- [x] Add a typed core-voltage-control evidence contract, independent
      validator, projector, generated binding, and command surface.
- [x] Add focused regressions for the accepted PWR-002 source quorum, exact
      DS4432U address/register/code and write route, 500 ms stabilization,
      active-low safe shutdown, source drift, publication withholding,
      candidate cleanup, and sensitive-output absence.
- [ ] Produce and independently validate one public redacted PWR-003
      projection from the sealed accepted PWR-002 evidence without rerunning
      hardware.
- [x] Run all required Cargo, Bright Builds, Bazel, parity, redaction,
      reference, generated-contract, diff, and source-compatibility gates.
- [ ] Promote only PWR-003 if the closed projection passes, record RESULT.md,
      archive this task, and preserve every explicit non-claim.

This is a software-only evidence audit. It permits reading committed evidence,
source, task, plan, and pinned reference state; running repository build, test,
validation, and projection commands; and publishing one redacted typed
projection. It does not permit a detector run, package build, flash, reset, USB
session, serial monitor, network request, credentials, mining rerun, voltage,
fan, power, GPIO, I2C, direct UART, pin, fault-injection, or other hardware
effect. Failed validation must preserve `PWR-003` at `implemented`, withhold
the final projection, clean any candidate, and record the earliest typed
failure. Acceptance requires the exact source projection and digest, trusted
package/runtime identity, issued 1100 mV command, source-bound DS4432U
constants and single-write route, complete stabilization, successful
downstream work, active-low safe stop, cleanup, no hardware rerun, and passed
redaction.

Blocked checkpoint: The one allowed projection attempt from clean pushed
implementation commit `10a72b06` failed closed as `evidence_invalid` before
candidate creation because the semantic fragment `CORE_VOLTAGE_STABILIZATION_MS,`
occurs twice in `mining_actuation_adapter.rs` (import and use). The projector
incorrectly required that substring to be unique. No projection was published,
no hardware command ran, and PWR-003 remains `implemented`. The next safe action
is a new bounded software retry contract that replaces the ambiguous substring
with a source-shaped unique fragment and adds the production-file regression
that this attempt exposed.

Plan closure:
`docs/parity/work-plans/20260812T203223Z-PWR-003/CLOSURE.md` records the
non-verifying terminal outcome. PWR-003 remains `implemented`, no checklist
transition or progress synchronization is warranted, and this task remained
active until a fresh bounded software-only retry plan existed.

Completion review: Superseded by
`task-parity-pwr003-core-voltage-control-evidence-retry` after the required
fresh task and immutable plan were created. The prior plan remains truthfully
closed and immutable; its failed ordinal is not reused. No parity claim,
projection, hardware action, or checklist transition is attributed to this
superseded task.

### task-parity-pwr003-core-voltage-control-evidence-retry | 2026-08-12 | Retry the sealed PWR-003 projection

- [x] Select PWR-003 as the next actionable row after the temporarily
      unavailable two-prompt physical-observation gate for API-009.
- [x] Freeze the fresh immutable software-only retry plan at
      `docs/parity/work-plans/20260812T212218Z-PWR-003/PLAN.md`.
- [x] Replace the ambiguous stabilization substring with a source-shaped
      unique matcher and bind the projector to this plan/task lineage.
- [x] Add a regression over the real production file that proves the complete
      DS4432U address/register/code and write route matcher set, including the
      intended 500 ms stabilization use site.
- [x] Run and push every focused and mandatory software, privacy, reference,
      integrity, source-compatibility, and diff gate before projection.
- [x] Run exactly one fresh sealed projection from the accepted PWR-002 source,
      independently validate it, and promote only on the complete quorum.
- [x] Record RESULT.md, archive this task, preserve all non-claims, and push
      the synchronized verified transition if and only if evidence passes.

This is a software-only evidence audit. It permits the exact source/test edits,
repository verification, and one atomic redacted projection described by the
linked plan. It permits no detector, package, flash, reset, USB/serial,
network, credential, mining, voltage, fan, power, GPIO, I2C, direct UART, pin,
fault-injection, or other hardware effect. A failure must preserve the earliest
typed category, remove any candidate, withhold the final projection, keep
PWR-003 `implemented`, and stop without another projection attempt.

Prior lineage: The superseded task and
`docs/parity/work-plans/20260812T203223Z-PWR-003/CLOSURE.md` establish that the
previous sole attempt failed before candidate creation only because the
configured stabilization substring occurred twice. The accepted PWR-002
projection remains the sole hardware evidence source; no hardware rerun is
authorized or needed.

Evidence checkpoint: Clean pushed implementation commit `a2fefad3` produced
the sole fresh projection successfully. Independent validation, mode `0644`,
candidate cleanup, source/plan digests, source compatibility, and sensitive
value absence pass. Projection SHA-256 is
`11dd1abbf6fda86d203fdcff49b420ab5139e1d29c35f4d17000c61c3112ae68`.
`docs/parity/work-plans/20260812T212218Z-PWR-003/RESULT.md` records the exact
evidence, conclusion, and non-claims.

Verification: Transition `20260812T215256Z-PWR-003` changed only PWR-003 to
`verified`, refreshed its evidence and ownership cells, and bound the immutable
plan and RESULT.md. Immediate progress synchronization from evidence commit
`a94ef6409a0c17c405951170659b3d9d87d08894` records 62 of 94 active rows
verified (66.0%). The complete ordered Cargo, Bright Builds, 41-test Bazel,
parity, progress, redaction, independent-validator, reference, digest, mode,
candidate, task-lineage, and diff gates pass.

Completion review: The closed projection proves the accepted Ultra 205 power
campaign used the unchanged production DS4432U address/register/code and
single-write route, waited 500 ms before active-low ASIC enable, reached
accepted downstream work, and completed safe stop and cleanup without a new
hardware run. Direct analog measurement, setpoint accuracy, rail timing or
waveform, arbitrary targets, INA260 correlation, injected faults, other
profiles, boards, and ASIC families remain non-claims. The row is verified and
this task is complete and archived.

### task-parity-pwr005-ds4432u-evidence-reconciliation | 2026-08-12 | Reconcile DS4432U support evidence

- [x] Select PWR-005 as the first actionable row after API-009's unavailable
      fresh two-prompt operator-readiness gate.
- [x] Freeze the immutable software-only plan at
      `docs/parity/work-plans/20260812T220119Z-PWR-005/PLAN.md`.
- [x] Independently validate the accepted PWR-003 projection, exact digest,
      result lineage, source/reference identities, current DS4432U ownership,
      final mode, and redaction.
- [x] Add a PWR-005-specific RESULT.md using only the overlapping closed facts;
      do not duplicate the evidence contract or rerun hardware.
- [x] Run every focused and mandatory software, privacy, reference, integrity,
      and diff gate before the evidence checkpoint and final transition.
- [x] Promote only PWR-005 on the complete closed quorum, archive this task,
      synchronize progress, and preserve every explicit non-claim.

This is a software-only evidence reconciliation. It permits reading committed
source, reference, task, plan, result, and accepted evidence; running repository
validation and verification commands; writing the row-specific result; and
transitioning only PWR-005 if every gate passes. It permits no detector,
package, flash, reset, USB/serial, network, credential, mining, voltage, fan,
power, GPIO, I2C, direct UART, pin, fault-injection, or other hardware effect.

Acceptance requires the exact existing PWR-003 projection with SHA-256
`11dd1abbf6fda86d203fdcff49b420ab5139e1d29c35f4d17000c61c3112ae68`,
its Rust validator, board 205 and exact source/reference identities, trusted
package/runtime, the source-compatible typed DS4432U address `0x48`, output
register `0xf8`, code `0xe1`, exactly one write, successful initialized work,
an accepted submit, safe stop, cleanup, no hardware rerun, and passed
redaction. A failed boundary leaves PWR-005 `implemented`, changes no checklist
field, and records the earliest blocker without a hardware retry.

Verification: Immutable plan SHA-256
`0c376bb8940a1f445cee0cfe49930f9e6147a9ad9c50814c277717e52ac51bf7`.
Transition `20260812T221105Z-PWR-005` changed only PWR-005 to `verified`,
replaced its stale observe-only ownership cell, and added
`hardware-regression` evidence. Immediate progress synchronization from exact
pushed source commit `cd7e394b553a6514794f4bada904d15d7e01e6dd`
records 63 of 94 active rows verified (67.0%). The ordered Cargo, Bright
Builds, all-40-target Bazel, parity, progress, focused contract, automation,
redaction, reference, task-binding, digest, mode, and diff gates passed. One
guessed nonexistent per-file Bazel target was replaced by the repository-owned
`//tools/automation:automation_test`, which passed.

Completion review: PWR-005 is verified from the immutable PWR-003 projection
and row-specific RESULT.md. The accepted Ultra 205 campaign proves the exact
DS4432U address, output-zero register, conservative code, typed single-write
route, successful downstream work, safe stop, cleanup, no hardware rerun, and
redaction. No duplicate projector or hardware action was used. Analog accuracy,
rail timing or waveform, DS4432U reads/output one, arbitrary targets, fault
injection, INA260 correlation, other profiles, boards, and ASIC families remain
non-claims. The task is complete and archived.

### task-parity-pwr006-ina260-live-projection | 2026-08-12 | Project accepted live INA260 evidence

- [x] Audit the current PWR-006 implementation, pinned INA260 reference, stale
      checklist note, accepted API-002 hardware capture, and source compatibility.
- [x] Add a Rust-owned `bitaxe-ina260-evidence-v1` contract and a narrow
      software-only projector over the existing protected API-002 snapshots.
- [x] Prove fresh complete INA260 current, bus-voltage, and power observations
      agree across the same HTTP and WebSocket acquisition without publishing
      any raw value, stamp, boot session, origin, port, or network identifier.
- [x] Independently validate and publish one redacted public PWR-006 projection,
      run every mandatory gate, and promote only PWR-006 if the complete quorum
      passes.

Dependencies: Completed API-002 live system-info capture at exact source commit
`524b445ee45c986a1366cfe64d2cbcbe41178da8`; immutable PWR-006 plan
`docs/parity/work-plans/20260812T222308Z-PWR-006/PLAN.md`; ignored protected
attempt root `scratch/api002-system-info/attempt-002`; pinned reference commit
`c1915b0a63bfabebdb95a515cedfee05146c1d50`.

Evidence and safety contract: This is a software-only projection. It may read
the existing mode-0700 API-002 attempt root and its mode-0600 JSON artifacts,
validate closed schema, digest, identity, freshness, safe-range, correlation,
cleanup, and redaction predicates, and publish only schemas, commits, digests,
fixed INA260 address/register constants, counts, categories, and booleans. Raw
power, voltage, current, acquisition stamps, boot sessions, hostnames, origins,
ports, USB/network identifiers, credentials, retained logs, and traces remain
private and must never appear in public output. No detector, package build,
flash, reset, USB/serial access, network request, credential read, mining,
voltage, fan, power, I2C/GPIO, direct UART, pin, fault-injection, or recovery
effect is allowed. No hardware retry exists because no hardware effect occurs.

Evidence checkpoint: Clean pushed implementation commit `bff0e547` produced
the sole projection successfully, and pushed result checkpoint
`6642f21ca49182f7b787aa3f7eaea4a2377edc66` binds it to the immutable plan.
Independent validation, mode `0644`, candidate cleanup, source/plan digests,
source compatibility, and sensitive-value absence pass. Projection SHA-256 is
`c9624b3c77e4021137a375de2a70c2bf7425bc947af6ba59c4e42fbceb25634d`.

Verification: Transition `20260812T232805Z-PWR-006` changed only PWR-006 to
`verified`, refreshed its ownership, evidence, and stale note, and bound the
immutable plan and RESULT.md. Immediate progress synchronization from exact
pushed source commit `6642f21ca49182f7b787aa3f7eaea4a2377edc66` records 64
of 94 active rows verified (68.1%). The ordered Cargo, Bright Builds, all-41-
target Bazel, parity, progress, redaction, source/final validator, reference,
generated-contract, digest, mode, candidate, task-binding, compatibility, and
diff gates pass. The macOS policy hold and byte-identical generated-artifact
workaround affected only local ignored test executables and no repository or
device state.

Completion review: PWR-006 is verified from the accepted API-002 Ultra 205
capture and typed row-specific projection. The closed evidence proves the
production read-only INA260 address/register set and complete fresh correlated
HTTP/WebSocket current, bus-voltage, and power sample, exact package and boot
identity, nine compatible current source paths, disabled mining/control,
cleanup, no hardware rerun, and redaction. Calibration beyond the admitted
conversion, long-duration drift, write/control effects, fan, voltage, ASIC,
mining, other boards, and release readiness remain non-claims. The task is
complete and archived.

### task-parity-thr002-fan-evidence-reconciliation | 2026-08-13 | Reconcile accepted fan-response evidence

- [x] Select THR-002 as the first actionable row after API-009's repeated stop
      and THR-001's need for a distinct bounded fault-stimulus contract.
- [x] Freeze the immutable software-only plan at
      `docs/parity/work-plans/20260813T024957Z-THR-002/PLAN.md`.
- [x] Wire the pure fan decisions into a bounded upstream-cadence production
      runtime that owns no raw I2C and uses only the existing typed actuation
      queue, with focused orchestration and ownership regressions.
- [x] Independently validate the accepted PWR-002 projection, exact digest,
      immutable result lineage, source/reference identities, unchanged EMC2101
      ownership, final mode, and redaction.
- [x] Add a THR-002-specific `RESULT.md` using the composed workflow and
      physical fan-response facts; do not duplicate the evidence contract or
      rerun hardware.
- [x] Run every focused and mandatory software, privacy, reference, integrity,
      and diff gate before the evidence checkpoint and final transition.
- [x] Promote only THR-002 on the complete closed quorum, archive this task,
      synchronize progress, and preserve every explicit non-claim.

This is a software-only implementation and evidence reconciliation. It permits
the high-level controller scheduler, startup wiring, focused tests, reading
committed source, reference, task, plan, result, and accepted evidence, running
repository validation and verification commands, writing the row-specific
result, and transitioning only THR-002 if every gate passes. It permits no detector,
package, flash, reset, USB/serial, network, credential, mining, voltage, fan,
power, GPIO, I2C, direct UART, pin, fault-injection, or other hardware effect.

Acceptance requires the exact existing PWR-002 projection with SHA-256
`0668c274d09b3e39d7d5edfea4b2e66c97248ff77de9192981f3af00e547ddfe`,
its Rust validator, board 205 and exact source/reference identities, trusted
package/runtime, fresh safety, the typed production 100% fan command,
fan-before-voltage ordering, a fresh nonzero post-command RPM, successful
initialized work, an accepted submit, safe stop, cleanup, no hardware rerun,
and passed redaction. Passing pure controller, duty-conversion, tachometer, and
fan-fault tests must provide the `unit,workflow` portion. A failed boundary
leaves THR-002 `implemented`, changes no checklist field, and records the
earliest blocker without a hardware retry.

Verification: Transition `20260813T032952Z-THR-002` changed only THR-002 from
`implemented` with `unit,workflow` evidence to `verified` with
`unit,workflow,hardware-regression`, bound immutable plan SHA-256
`3adf53a19701e33ef195898560b7f8f17baef0f36033912bc86de54ace0d178d`,
RESULT.md SHA-256
`3505303892578beb884bce95521a6714c0818866c3244e56ca2e45b3c6f52186`,
source commit `b76eca69be9b3b6a5590aa7678f1b8766dd62b5f`, and reference commit
`c1915b0a63bfabebdb95a515cedfee05146c1d50`. Immediate progress
synchronization records 65 of 94 active rows verified (69.1%). The ordered
Cargo, Bright Builds, all-42-target Bazel, parity, progress, redaction,
reference, evidence-validator, digest, mode, source-compatibility, task-binding,
and diff gates passed.

Completion review: THR-002 is verified from the production high-level
controller workflow and the accepted PWR-002 Ultra 205 fan-response evidence.
The closed quorum proves upstream priority decisions at a bounded 100 ms
cadence, typed actuation ownership, the physical 100% command before voltage, a
fresh nonzero post-command RPM, successful downstream work, safe stop, cleanup,
and redaction without a new hardware run. Automatic live transitions among all
modes, PID tuning quality, long-duration regulation, fan-fault injection,
thermal stress, other boards, and release readiness remain non-claims. The task
is complete and archived.

### task-parity-thr003-pid-controller | 2026-08-13 | Match the pinned fan PID state machine

- [x] Select THR-003 after API-009's repeated stop and THR-001's unavailable
      safe fault-stimulus command.
- [x] Freeze the immutable software-only plan at
      `docs/parity/work-plans/20260813T033800Z-THR-003/PLAN.md`.
- [x] Replace the simplified PID reducer with exact pinned initialization,
      input EMA, reverse P-on-E, 100 ms gain scaling, limits, and anti-windup.
- [x] Expand the provenance-bound sequential golden vectors and focused
      production state-retention/ownership regressions.
- [x] Independently revalidate the accepted PWR-002 fan-actuation projection
      without a new evidence schema or hardware run.
- [x] Run every focused and mandatory gate, write THR-003 RESULT.md, and commit
      and push the complete evidence source before checklist mutation.
- [x] Promote only THR-003 on the closed quorum, synchronize progress, archive
      this task, and preserve every live closed-loop non-claim.

This task is software-only and permits pure Rust PID implementation, fixtures,
tests, firmware builds, evidence/result composition, typed checklist
transition, and repository verification. It permits no detector, package
capture, flash, reset, USB/serial, network, credentials, mining, voltage, fan,
power, I2C/GPIO, fault injection, direct UART, pins, pads, or other hardware
effect.

Acceptance requires exact sequential agreement with pinned PID behavior,
production 100 ms scheduling, unchanged typed actuator ownership and safety
qualification, a valid row-specific result, and the byte-identical accepted
PWR-002 projection at SHA-256
`0668c274d09b3e39d7d5edfea4b2e66c97248ff77de9192981f3af00e547ddfe`
and mode `0644` only as physical actuator-chain evidence. Live automatic
closed-loop response, analog RPM accuracy, settling, tuning quality, arbitrary
duties, fault/overheat behavior, other boards, and release readiness remain
non-claims. A failed gate leaves THR-003 `implemented` and changes no checklist
or progress field.

Verification: Transition `20260813T040000Z-THR-003` changed only THR-003 from
`implemented` with `unit` evidence to `verified` with
`unit,golden,workflow,hardware-regression`, bound immutable plan SHA-256
`3acea362f65f63ccab564b1d4af98a22f4f026dffecf258a5a5d70ca119e0348`,
RESULT.md SHA-256
`6536cae83b6b5397caa3dc5bb96324719fb582adeeb8c8fca5bb303d36584f5d`,
source commit `6f8c043a0d188404f63e73fbf8e3a5427e876f71`, and reference commit
`c1915b0a63bfabebdb95a515cedfee05146c1d50`. Immediate progress
synchronization records 66 of 94 active rows verified (70.2%). The ordered
Cargo, Bright Builds, all-42-target Bazel, focused, normal/rollback firmware,
parity, progress, redaction, reference, evidence-validator, digest, mode,
source-compatibility, task-binding, sensitive-output, and diff gates passed.

Completion review: THR-003 is verified from exact state-by-state pure PID
vectors, the production 100 ms scheduler and retained-state workflow, and the
accepted PWR-002 Ultra 205 actuator-chain regression. The closed quorum proves
input EMA, C-float reverse P-on-error computation, initialization, limits,
clamps, anti-windup, state retention, typed ownership, physical 100% command
and nonzero RPM response, cleanup, and redaction without a new hardware run.
Live automatic response, RPM accuracy, settling, tuning quality, arbitrary
duties, injected faults, overheat behavior, other boards, and release readiness
remain non-claims. The task is complete and archived.

### task-parity-io001-i2c-retry-contract | 2026-08-04 | Match shared I2C transfer policy

- [x] Add an exact host-testable 500 ms, three-attempt, 10 ms-delay transfer
      policy matching the pinned reference.
- [x] Route every display, sensor, and actuation transfer through the single
      retry owner without widening address or effect capabilities.
- [x] Extend bypass regressions, build the real firmware, run all mandatory
      gates, and transition only `IO-001` to `implemented`.

Plan: `docs/parity/work-plans/20260804T135918Z-IO-001/PLAN.md`

Authorization: software-only I2C contract work. No hardware, credentials,
network requests, mining, voltage/fan effects, OTA, direct UART, or pins.

Verification: Four focused retry tests, source-ownership tests, and the real
ESP-IDF firmware build passed. The mandatory ordered Rust sequence, Bright
Builds checks, all 29 Bazel test targets, parity/progress, redaction, reference
cleanliness, and diff checks passed on implementation commit `b15073c9`.

Completion review: The exact bounded transfer policy is implemented and
`IO-001` is `implemented` with `unit,workflow,hardware-smoke` evidence. The task
remains active and unarchived because the hardware breadcrumb predates the
retry change; live transient-fault, timeout, and shared-bus behavior remain
unverified. No hardware, credentials, mining, controls, OTA, UART, or pins ran.

Evidence-reconciliation continuation plan:
`docs/parity/work-plans/20260813T041410Z-IO-001/PLAN.md`.

- [x] Validate the post-retry INA260 and EMC2101 projections, sealed
      post-retry campaign preparation, physical actuation projections, and
      existing SSD1306 smoke boundary as one exact-claim quorum.
- [x] Prove retry/bus/transfer source compatibility and preserve injected
      electrical faults, live exhaustion, waveform timing, probing, arbitrary
      values, other devices, and other boards as explicit non-claims.
- [x] Run focused firmware/evidence checks and every mandatory repository gate;
      promote only IO-001 on a complete valid result.

Continuation authorization: read-only committed public evidence, closed
aggregate fields and digests from the existing protected API-009 attempt-007,
local validators, source comparisons, tests, builds, documentation, and one
typed IO-001 transition only. No detector, package capture, flash, reset,
USB/serial/network session, credential use, mining, voltage/fan/power effect,
HTTP command, OTA, recovery, direct UART, pins, pads, GPIO, raw I2C, fault
injection, or other hardware action is permitted. Protected origins,
hostnames, ports, identities, network values, credentials, workers, addresses,
passwords, tokens, traces, and sensor values must not enter committed output.

Final verification: Transition `20260813T044300Z-IO-001` changed only IO-001
from `implemented` with `unit,workflow,hardware-smoke` evidence to `verified`
with `unit,workflow,hardware-smoke,hardware-regression`, bound immutable plan
SHA-256
`1796d9ccf478a595557762e9197e811afefc68a35c2e7c8a87c2743f626f9c12`,
RESULT.md SHA-256
`bc931141cce6949c19728868e08ca38eed0d90eefc648a1621726d8d4d139630`,
source commit `c59382df76bb65922034f22caddb2ee731a0dd77`, and reference commit
`c1915b0a63bfabebdb95a515cedfee05146c1d50`. Immediate progress
synchronization records 67 of 94 active rows verified (71.3%). The ordered
Cargo, Bright Builds, all-42-target Bazel, focused, normal/rollback firmware,
parity, progress, redaction, reference, contract/evidence-validator, digest,
mode, source-compatibility, task-binding, sensitive-output, and diff gates
passed.

Final completion review: IO-001 is verified from exact retry and ownership
tests, the retained SSD1306 hardware-smoke boundary, post-retry typed INA260
and EMC2101 reads, the independently validated physical fan/DS4432U chain, and
the sealed post-retry completed preparation transaction. Injected electrical
faults, live terminal exhaustion, waveform/timing measurement, probing or
scanning, arbitrary addresses/registers/values, unsupported devices, other
boards, and release readiness remain non-claims. No new device effect occurred
during reconciliation. The task is complete and archived.

### task-api009-sensor-sweep-latency | 2026-08-15 | Diagnose and fix active sensor freshness loss

- [x] Add a deterministic production-shaped feedback loop that reproduces an
      unchanged observation epoch and stale active-safety sample when one
      shared-I2C acquisition consumes the current retry/timeout envelope.
- [x] Add redaction-safe, boot-scoped producer timing facts that identify only
      the delayed acquisition stage, bounded duration bucket, sweep outcome,
      and revision; never publish sensor values or device/network identity.
- [x] Rank and falsify the sensor-read, display-flush, actuation-queue, producer
      scheduling, and consumer-sampling hypotheses against the same timing
      model and the attempt-031 closed boundary.
- [x] Fix the confirmed producer/runtime cause without weakening the 1,000 ms
      active-safety freshness requirement, hiding acquisition failures, or
      accepting a stale epoch; add focused regressions for success, timeout,
      recovery, marker redaction, and failure precedence.
- [x] Run the ordered Cargo, firmware, Bright Builds, Bazel, parity, redaction,
      reference-cleanliness, source-ownership, sensitive-output, and diff gates;
      simplify, review, commit, and push the exact implementation.

Authorization and stop rule: this block permits source, fixtures, deterministic
simulation, local child processes, builds, and repository verification only.
It does not authorize protected attempt data, credentials, detector, package
effects, USB/device/network/HTTP sessions, flash, reset, mining, ASIC traffic,
hardware controls, OTA, recovery, external UART/BAP, pins, or attempt-032. A
future hardware ordinal requires a separate exact clean-package task contract
after this task proves a materially changed boundary. Stop if the deterministic
seam cannot distinguish the delayed producer stage without sensitive values, or
if the verified fix would require weakening the active-safety contract.

Final verification: The pre-change production-shaped test reproduced a sensor
transfer starting at 500 ms remaining blocked until 2,030 ms under the general
three-attempt, 500-ms timeout contract. The bounded implementation passed the
same test in three uncached runs. Focused retry, diagnostic, display, source-
ownership, marker, evidence, and automation tests passed. Ordered Cargo format,
clippy, all-target build, and all-feature tests passed, as did the firmware
build, all 45 Bazel test targets, Bright Builds checks, parity validation and
progress, redaction, reference cleanliness, sensitive-output scan, and diff
checks. Commit `c2fb0c93` was pushed to `origin/main`.

Final completion review: Runtime sensor, display, and actuation transfers now
share the producer's absolute publication deadline with 100 ms headroom while
startup retains the complete upstream retry envelope. Display budget exhaustion
defers work without claiming a render or permanently disabling the panel. A
closed campaign diagnostic identifies only stage, outcome, coarse duration
bucket, and revision publicly; boot session remains private and no sensor,
device, USB, network, or credential value is exposed. A live trigger remains
unclassified until a separately contracted attempt observes the new boundary.
No hardware effect occurred during this task. The task is complete and archived
at 2026-08-15T07:54:56Z.

### task-api009-programmatic-pilot-attempt-032 | 2026-08-15 | Prove command effects with bounded runtime I2C

- [x] Require clean synchronized pushed source descended from fix commit
      `c2fb0c93`, non-empty ignored Wi-Fi input without reading it, and fresh
      detector, attempt, and public-projection paths.
- [x] Run `just package` as the sole package admission/build surface and advance
      only after its zero exit, exact HEAD source identity, reference identity,
      and required manifest artifacts are confirmed.
- [x] Create protected `scratch/api009-command-effects/detector-032`, run
      exactly one `just detect-ultra205`, and advance only after its zero exit,
      mode-`0600` output, and repo-owned one-device admission are confirmed.
- [x] Invoke exactly once:
      `just api-command-effects-campaign --private-root scratch/api009-command-effects/attempt-032 --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --detector-output scratch/api009-command-effects/detector-032/detector.stdout --projection docs/parity/evidence/api009-command-effects/command-effects-projection.json --duration-seconds 600`.
- [x] Validate and redact a sealed projection only on the complete command,
      same-device restart, exact-package, recovery, safe-stop, and cleanup
      quorum; otherwise withhold it and record the earliest typed failure plus
      only the closed sensor stage, outcome, duration bucket, and revision when
      the new diagnostic is available.

Objective and effects: test only the pushed runtime-I2C deadline and diagnostic
boundary that follows consumed attempt-031. The sole run may flash/reset the
exact package, seed private Wi-Fi and the generated local fixture, initialize
and mine the conservative profile for at most 600 active seconds, issue one each
pause/dismiss/IDENTIFY/resume/software-restart request, observe HTTP plus
WebSocket and receive-only native USB, and perform same-device recovery,
terminal safe stop, child termination, and USB cleanup. Active mining retains
the immutable 1,000-ms freshness rule. Automated effects and recovery retain
finite repo-owned timeouts; no human checkpoint or display claim is part of this
attempt.

Evidence/privacy/recovery: detector and attempt directories are ignored
mode-`0700` roots with mode-`0600` files. Only the named aggregate projection
may become public after ready validation and redaction. Origins, ports,
hostnames, addresses, USB/network/process identity, credentials, fixture data,
frame text, sensor values, boot session, and raw traces remain private. Public
failure output may contain only the closed stage/outcome/duration/revision
diagnostic and safe recovery booleans. The earliest primary failure survives
recovery; safe stop, child termination, and holder cleanup run on every exit.
Campaign start consumes attempt-032; no attempt-033 or same-contract retry is
authorized. Any nonzero preflight, missing artifact, detector/identity/build
ambiguity, non-ready category, malformed projection, failed safe stop/recovery/
cleanup, or absent device stops with API-009 `implemented` and final evidence
withheld. OTA, erase, factory reset, power cycle, external UART/BAP, USB duplex,
pins/pads/GPIO, arbitrary settings, external pool, stress, direct controls,
fault injection, non-205 hardware, and human display claims remain prohibited.

Final verification: Clean synchronized source `a92196e4`, exact package and
reference identity, ignored private-input presence, fresh paths, the one
protected detector admission, and the single campaign invocation all passed.
The campaign terminated `hardware_blocked` with active-safety staleness, an
unchanged observation epoch, and all required observation freshness flags
false. Safe stop and process/USB cleanup were confirmed, recovery was not
required, and the public evidence projection was correctly withheld.

Final completion review: The bounded retry change did not eliminate the live
freshness loss. The closed terminal diagnostic contained revision 8 and a
slow-but-successful display event in the `under_250_ms` bucket. That later event
had overwritten the earlier actionable failure. The automation wrapper also
omitted the otherwise valid diagnostic because the private Rust `u64` boot
session exceeded JavaScript's safe-integer range, even though boot identity is
never public. These are software diagnostic defects and materially new
information, not user-caused failures. Attempt-032 is consumed, attempt-033 is
not authorized, API-009 remains `implemented`, and the task is complete and
archived at 2026-08-15T08:06:26Z.

### task-api009-diagnostic-precedence-u64 | 2026-08-15 | Preserve the actionable sensor failure

- [x] Add a red regression proving a later lower-severity I2C pressure event
      cannot replace an earlier budget-exhausted or driver-failed stage in the
      boot-scoped campaign diagnostic.
- [x] Retain and project the highest-severity event with stable deterministic
      tie handling while still emitting each redaction-safe transition marker.
- [x] Accept the Rust-validated private `u64` boot session at the TypeScript
      handoff without requiring JavaScript-safe integer precision, and continue
      omitting that value from every public result.
- [x] Prove primary failure/recovery precedence, malformed evidence rejection,
      diagnostic redaction, schema consistency, and complete simulated campaign
      behavior; run all mandatory gates, simplify, review, commit, and push.

Authorization and stop rule: source, deterministic fixtures, local child
processes, builds, and repository verification only. This task authorizes no
protected-attempt reads beyond the already extracted closed attempt-032 facts,
credentials, detector, package effect, USB/device/network/HTTP session, flash,
reset, mining, ASIC traffic, controls, recovery, OTA, UART/BAP, pins, or
attempt-033. Stop if diagnostic ordering cannot preserve the actionable cause
without exposing private identity or if a fix would weaken active safety.

Final verification: Red production-seam regressions reproduced both defects.
Focused firmware and host integration suites pass with lower-, higher-, and
equal-severity retention, above-safe-integer private boot identity, malformed
optional evidence rejection, recovery precedence, and public redaction checks.
Ordered Cargo format, clippy, all-target build, and all-feature tests passed,
as did Bright Builds, firmware build, all 45 Bazel test targets, parity and
progress, redaction, reference cleanliness, sensitive-output scan, and diff
checks.

Final completion review: The retained diagnostic now preserves the earliest
event at the highest observed severity while every event still advances the
revision and emits its marker. The TypeScript handoff validates the private
Rust `u64` as a positive finite integer without relying on unsafe arithmetic or
publishing it. No active-safety rule was weakened and no hardware effect
occurred. The task is complete and archived at 2026-08-15T08:18:00Z.

### task-api009-programmatic-pilot-attempt-033 | 2026-08-15 | Expose the actionable runtime sensor stage

- [x] Require clean synchronized pushed source containing diagnostic fix
      `bee8c1c9`, non-empty ignored Wi-Fi input without reading it, and fresh
      detector, attempt, and public-projection paths.
- [x] Run `just package` as the sole package admission/build surface and advance
      only after its zero exit, exact HEAD/reference identity, and required
      manifest artifacts are confirmed.
- [x] Create protected `scratch/api009-command-effects/detector-033`, run
      exactly one `just detect-ultra205`, and advance only after its zero exit,
      mode-`0600` output, and repo-owned one-device admission are confirmed.
- [x] Invoke exactly once:
      `just api-command-effects-campaign --private-root scratch/api009-command-effects/attempt-033 --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --detector-output scratch/api009-command-effects/detector-033/detector.stdout --projection docs/parity/evidence/api009-command-effects/command-effects-projection.json --duration-seconds 600`.
- [x] Validate and redact a sealed projection only on the complete command,
      same-device restart, exact-package, recovery, safe-stop, and cleanup
      quorum. Otherwise withhold it and record only the earliest typed category,
      closed highest-severity sensor stage/outcome/duration/revision, and safe
      recovery booleans.

Objective and effects: obtain one live observation of the materially corrected
severity-preserving diagnostic boundary; this is not an unchanged promotion
retry. The sole run may flash/reset the exact package, seed private Wi-Fi and
the generated local fixture, initialize and mine the conservative profile for
at most 600 active seconds, issue one each pause/dismiss/IDENTIFY/resume/software
restart request, observe HTTP plus WebSocket and receive-only native USB, and
perform same-device recovery, terminal safe stop, child termination, and USB
cleanup. Active mining keeps the immutable 1,000-ms freshness rule. Automated
effects and recovery retain finite repo-owned timeouts; no human checkpoint or
display claim is part of this attempt.

Evidence/privacy/recovery: detector and attempt directories are ignored
mode-`0700` roots with mode-`0600` files. Only the named aggregate projection
may become public after ready validation and redaction. Origins, ports,
hostnames, addresses, USB/network/process identity, credentials, fixture data,
frame text, sensor values, boot session, and raw traces remain private. Public
failure output is limited to the closed diagnostic and safe recovery booleans.
The earliest primary failure survives recovery; safe stop, child termination,
and holder cleanup run on every exit. Campaign start consumes attempt-033; no
attempt-034 or same-contract retry is authorized. Any nonzero preflight,
missing artifact, detector/identity/build ambiguity, non-ready category,
malformed projection, failed safe stop/recovery/cleanup, or absent device stops
with API-009 `implemented` and evidence withheld. OTA, erase, factory reset,
power cycle, external UART/BAP, USB duplex, pins/pads/GPIO, arbitrary settings,
external pool, stress, direct controls, fault injection, non-205 hardware, and
human display claims remain prohibited.

Final verification: Clean synchronized source `e22b17fd`, exact package and
reference identity, opaque ignored input, fresh paths, and one successful
protected detector admission passed. The sole campaign invocation returned
`evidence_invalid`; the public projection was withheld. Exact-signature
matching proved the shell had pre-created the private attempt root, and the
root contained no campaign child directory or artifact. No detector holder
remained.

Final completion review: Redirecting wrapper output into the attempt root
created that root before the automation process could perform its freshness
admission. The fail-closed guard rejected it before any campaign child or
hardware effect launched, so false cleanup booleans describe an unentered
lifecycle rather than unsafe residual state. No user action contributed.
Attempt-033 is consumed, attempt-034 is not authorized, API-009 remains
`implemented`, and the task is complete and archived at 2026-08-15T08:30:00Z.

### task-api009-wrapper-root-preflight | 2026-08-15 | Prevent attempt-root capture collisions

- [x] Add a production-seam regression proving a pre-created API-009 private
      attempt root fails before any child process or hardware effect launches.
- [x] Document at the freshness guard that shell stdout/stderr capture must use
      a separate protected sibling root because redirection occurs before the
      command can admit and create its attempt root.
- [x] Verify the focused host suite, all mandatory repository gates, redaction,
      reference cleanliness, and diff review; then archive, commit, and push.

Authorization and stop rule: source, deterministic fixtures, local child
fakes, builds, and repository verification only. No detector, credential,
package effect, USB/device/network/HTTP session, flash, reset, mining, recovery,
OTA, UART/BAP, pins, or attempt-034 is authorized. Stop if preventing this
collision would require admitting non-empty or pre-populated evidence roots.

Final verification: The focused production-seam test simulates shell-created
wrapper output inside the private attempt root and proves typed
`evidence_invalid` with zero child launches. The full host suite includes the
new module. Ordered Cargo format, strict Clippy, all-target build, and
all-feature tests passed; Bright Builds, firmware build, all 45 Bazel tests,
parity validation/progress, redaction, reference cleanliness, and diff checks
also passed. One transient parity-report output failure reran cleanly with
`validation_errors: none`.

Final completion review: The freshness guard remains strict; it now documents
that wrapper capture belongs in a protected sibling directory. The dedicated
test keeps the production and test files within their governed line budgets
and prevents this orchestration collision from being mistaken for a campaign
or hardware failure. No hardware effect occurred. The task is complete and
archived at 2026-08-15T08:45:00Z.

### task-api009-programmatic-pilot-attempt-034 | 2026-08-15 | Observe the corrected sensor diagnostic

- [x] Require clean synchronized pushed source containing `3ae3c85e`, opaque
      non-empty ignored Wi-Fi input, absent attempt/public paths, and a fresh
      protected sibling `wrapper-034` path distinct from `attempt-034`.
- [x] Build and validate the exact HEAD/reference package through `just package`.
- [x] Create protected `detector-034`, run exactly one `just detect-ultra205`,
      and advance only after its zero exit, private modes, and one-device
      admission are confirmed.
- [x] Invoke exactly once the existing `just api-command-effects-campaign`
      command with `attempt-034`, the exact package, opaque credentials,
      detector output, public projection, and 600-second bound. Redirect only
      into the separate mode-`0700` `wrapper-034`; keep `attempt-034` absent for
      command-owned freshness admission.
- [x] Publish only a ready independently validated redacted projection;
      otherwise withhold it and record the earliest typed category, the closed
      highest-severity sensor diagnostic, and safe recovery/cleanup booleans.

Objective/effects: obtain one live reading of the materially changed diagnostic
and wrapper contract. The sole campaign may exact-package flash/reset, seed
private Wi-Fi and a generated local fixture, conservatively initialize/mine for
at most 600 active seconds, issue one each pause/dismiss/IDENTIFY/resume/software
restart request, observe HTTP/WebSocket/receive-only native USB, and perform
same-device recovery, safe stop, child termination, and USB cleanup. The active
1,000-ms freshness rule and all automated/recovery timeouts remain unchanged.
No human checkpoint or physical-display claim is included.

Privacy/recovery/stop: detector, wrapper, and attempt paths are ignored
mode-`0700` roots with mode-`0600` files. Origins, ports, hostnames, addresses,
USB/network/process identity, credentials, fixture/frame/sensor values, boot
session, and raw traces remain private. The earliest primary failure survives
recovery; cleanup runs on every entered lifecycle. Campaign start consumes
attempt-034; no attempt-035 or unchanged retry is authorized. Preflight,
identity/build/detector ambiguity, any non-ready category, malformed evidence,
failed recovery/cleanup, or absent device stops with evidence withheld and
API-009 `implemented`. OTA, erase, factory reset, power cycle, UART/BAP, USB
duplex, pins/pads/GPIO, arbitrary settings, external pool, stress, direct
controls, fault injection, non-205 hardware, and human display claims remain
prohibited.

Final verification: Clean synchronized source `dae58db9`, exact package and
reference identity, opaque input, absent command-owned attempt path, separate
protected wrapper, and one successful detector admission passed. The sole
campaign terminated `hardware_blocked`; safe stop and cleanup were true,
recovery was not required, no holder remained, and all private file modes
passed. The public projection was withheld. The closed diagnostic reported
revision 4, stage `asic_temperature`, outcome `budget_exhausted`, and duration
bucket `under_100_ms`.

Final completion review: The sibling-wrapper contract eliminated the attempt-
root collision and exposed the next live runtime boundary. Source analysis
shows a preceding driver failure can consume the shared deadline and then be
replaced by the downstream ASIC-temperature exhaustion because driver failure
was ranked below budget exhaustion. No user action contributed. Attempt-034 is
consumed, attempt-035 is not authorized, API-009 remains `implemented`, and the
task is complete and archived at 2026-08-15T09:00:00Z.

### task-api009-diagnostic-causal-precedence | 2026-08-15 | Preserve the upstream driver failure

- [x] Add a red regression proving an earlier driver failure is not replaced by
      a downstream budget exhaustion caused by the shared absolute deadline.
- [x] Rank a concrete driver failure above budget exhaustion while keeping both
      above invalid/unavailable/slow-success outcomes and retaining stable ties.
- [x] Run focused and complete software/privacy/reference gates, simplify,
      review, archive, commit, and push before any new hardware ordinal.

Authorization and stop rule: source, deterministic fixtures, builds, and local
tests only. No protected attempt reads beyond the already public closed
attempt-034 diagnostic, detector, credential, package effect, USB/device/
network/HTTP session, flash, reset, mining, recovery, OTA, UART/BAP, pins, or
attempt-035 is authorized. Stop if causal precedence would weaken the runtime
deadline, suppress transition markers, or expose private values.

Final verification: The new production-shaped diagnostic test failed before
the change with the downstream ASIC-temperature budget exhaustion replacing an
earlier power driver failure, then passed after the two severity values were
reordered. Existing lower/higher/equal severity, marker, and redaction tests
also pass. Ordered Cargo format, strict Clippy, all-target build, all-feature
tests, Bright Builds, firmware build, all 45 Bazel targets, parity/progress,
redaction, reference cleanliness, and diff checks pass.

Final completion review: Concrete driver failure is now the highest diagnostic
severity; budget exhaustion remains above invalid, unavailable, recovered, and
slow successful outcomes. Stable ties and every transition revision/marker are
unchanged, and no runtime deadline or safety behavior was weakened. No hardware
effect occurred. The task is complete and archived at 2026-08-15T09:12:00Z.

### task-api009-programmatic-pilot-attempt-035 | 2026-08-15 | Identify the causal runtime driver stage

- [x] Require clean synchronized pushed `86987839`, opaque non-empty ignored
      Wi-Fi input, exact HEAD/reference package, and absent detector, wrapper,
      attempt, and public-projection paths.
- [x] Run `just package`; create private `detector-035`; run exactly one
      `just detect-ultra205`; require exact artifacts, private modes, and one
      admitted board-205 device.
- [x] Create separate private `wrapper-035`, keep `attempt-035` absent, and
      invoke exactly once the existing 600-second `just
      api-command-effects-campaign` with the exact package, opaque input,
      detector output, attempt root, and public projection.
- [x] Publish only a ready independently validated redacted projection;
      otherwise withhold it and record the earliest typed category, causal
      highest-severity stage/outcome/duration/revision, and recovery booleans.

Effects: one exact-package flash/reset, private Wi-Fi/local-fixture seed,
conservative initialization/mining for at most 600 active seconds, one each
pause/dismiss/IDENTIFY/resume/software restart, HTTP/WebSocket/receive-only USB
observation, same-device recovery, safe stop, child termination, and USB
cleanup. Active freshness and automated/recovery bounds remain unchanged. No
human checkpoint or physical-display claim is included.

Privacy/retry/stop: detector, wrapper, and attempt roots are ignored mode-0700
with mode-0600 files. Never publish origins, ports, hostnames, addresses,
device/USB/network/process identity, credentials, fixture/frame/sensor values,
boot session, or raw traces. Preserve the earliest primary failure; cleanup
runs on every entered lifecycle. Campaign start consumes attempt-035; no
attempt-036 or unchanged retry is authorized. Any preflight/identity/detector/
build ambiguity, non-ready result, malformed evidence, failed recovery/cleanup,
or absent device stops with API-009 `implemented` and evidence withheld. OTA,
erase, factory reset, power cycle, UART/BAP, USB duplex, pins/pads/GPIO,
arbitrary settings, external pool, stress, direct controls, fault injection,
non-205 hardware, and human display claims remain prohibited.

Final verification: After a read-only zsh preflight initially stopped because
the reserved lowercase `path` variable replaced executable lookup, the safe
variable-name rerun passed without consuming the ordinal or touching hardware.
Clean pushed `4251d71f`, exact package/reference, one protected detector, the
separate wrapper, and one campaign passed admission. The campaign stopped
`hardware_blocked`; safe stop/cleanup and recovery passed, no secondary failure
or holder remained, and evidence was withheld. The closed diagnostic reported
revision 11, `power / budget_exhausted / under_100_ms`.

Final completion review: Power is the first sweep stage, so this proves the
operator owner began a sweep after its previous-publication deadline. Source
comparison shows Rust pthreads default to priority 5 while upstream creates the
power-management sensor task at priority 10. No user action contributed.
Attempt-035 is consumed, attempt-036 is not authorized, API-009 remains
`implemented`, and the task is complete and archived at 2026-08-15T09:35:00Z.

### task-api009-sensor-owner-priority | 2026-08-15 | Match upstream sensor scheduling priority

- [x] Raise only the operator sensor/display/I2C owner from the ESP pthread
      default priority 5 to upstream power-management priority 10 before its
      first runtime action.
- [x] Add source-ownership proof that the constant and current-task priority
      application remain local to this owner and do not change global pthread,
      mining worker, freshness, retry, or timeout configuration.
- [x] Run focused and complete gates, simplify, review, archive, commit, and
      push before any new hardware ordinal.

Authorization: software/tests/builds only. No detector, credentials, USB,
device/network/HTTP session, flash/reset/mining/recovery, OTA, UART/BAP, pins,
or attempt-036. Stop if the priority cannot be applied locally, exceeds the
upstream value, or changes the active-safety or I2C deadline contract.

Final verification: The operator sensor/display/I2C thread now raises only its
current FreeRTOS task to priority 10 before entering runtime work. The source
ownership regression requires one local current-task call, the exact upstream
priority, and no corresponding mining-worker call. The focused ownership test,
real firmware build, mandatory Cargo sequence, Bright Builds checks, Bazel test
surface, parity report with no validation errors, parity progress, redaction,
reference cleanliness, and diff checks all passed.

Final completion review: The fix changes neither global pthread defaults nor
freshness, retries, deadlines, or mining scheduling. Applying the priority to
the current task is the smallest robust alignment with upstream and avoids a
broader scheduler configuration change. No hardware effect occurred. The task
is complete and archived at 2026-08-15T09:52:00Z; attempt-036 still requires a
separate exact-package hardware contract.

### task-api009-programmatic-pilot-attempt-036 | 2026-08-15 | Verify the prioritized sensor owner

- [x] Require clean synchronized pushed `7917de87`, opaque non-empty ignored
      Wi-Fi input, exact HEAD/reference package identity, and absent detector,
      wrapper, command-owned attempt, and public-projection paths.
- [x] Run `just package`; create protected `detector-036`; run exactly one
      `just detect-ultra205`; require its zero exit, private modes, and one
      admitted board-205 device.
- [x] Create separate protected `wrapper-036`, keep `attempt-036` absent, and
      invoke exactly once the existing 600-second `just
      api-command-effects-campaign` with the exact package, opaque input,
      detector output, attempt root, and public projection.
- [x] Publish only a ready independently validated redacted projection;
      otherwise withhold it and record the earliest typed category, causal
      highest-severity stage/outcome/duration/revision, and recovery booleans.

Effects: one exact-package flash/reset, private Wi-Fi/local-fixture seed,
conservative initialization/mining for at most 600 active seconds, one each
pause/dismiss/IDENTIFY/resume/software restart, HTTP/WebSocket/receive-only USB
observation, same-device recovery, safe stop, child termination, and USB
cleanup. Active freshness and automated/recovery bounds remain unchanged. No
human checkpoint or physical-display claim is included.

Privacy/retry/stop: detector, wrapper, and attempt roots are ignored mode-0700
with mode-0600 files. Never publish origins, ports, hostnames, addresses,
device/USB/network/process identity, credentials, fixture/frame/sensor values,
boot session, or raw traces. Preserve the earliest primary failure; cleanup
runs on every entered lifecycle. Campaign start consumes attempt-036; no
attempt-037 or unchanged retry is authorized. Any preflight/identity/detector/
build ambiguity, non-ready result, malformed evidence, failed safe stop/
recovery/cleanup, or absent device stops with API-009 `implemented` and
evidence withheld. OTA, erase, factory reset, power cycle, UART/BAP, USB duplex,
pins/pads/GPIO, arbitrary settings, external pool, stress, direct controls,
fault injection, non-205 hardware, and human display claims remain prohibited.

Final verification: Clean pushed `cf663df4`, the exact package/reference, one
protected detector, separate wrapper, and one campaign passed admission. The
campaign stopped `hardware_blocked / command_effects`; safe stop, cleanup, and
recovery passed without a secondary failure, all private files retained mode
0600, and the public projection was withheld. The priority fix succeeded: the
closed sensor diagnostic reported revision 1, `display / ready / under_500_ms`.

Final completion review: The command campaign proved one pause request and its
HTTP-generation/native-USB safe-stop quorum, then made one dismiss request and
failed before its confirmation. Exact package identity and safety remained
valid, and the active campaign ended well before the phase deadline. The host
loop can immediately fail on transient WebSocket or HTTP observation loss even
though those witnesses are independent and bounded phase deadlines remain.
No user action contributed. Attempt-036 is consumed, attempt-037 is not
authorized, API-009 remains `implemented`, and the task is complete and
archived at 2026-08-15T10:20:00Z.

### task-api009-transient-witness-continuity | 2026-08-15 | Tolerate bounded witness transport loss

- [x] Make transient WebSocket connect/read/peer-close loss degrade to the
      independent receive-only USB witness instead of immediately failing an
      in-progress command effect.
- [x] Make transient HTTP status reads wait for the existing phase deadline;
      keep malformed successful responses, identity drift, request ambiguity,
      missing required witnesses, and deadline expiry fail closed.
- [x] Add deterministic regressions proving pause/dismiss progress survives
      transient witness loss while stale/duplicate generations and missing
      terminal proof still fail.
- [x] Run focused and complete gates, simplify, review, archive, commit, and
      push before any new hardware ordinal.

Authorization: source, deterministic fixtures, local child processes, builds,
and repository verification only. Protected attempt-036 may be used only to
derive closed failure facts; never publish its identities, origins, ports,
addresses, credentials, boot session, raw traces, or values. No detector,
credentials, package effect, USB/device/network/HTTP session, flash, reset,
mining, recovery, OTA, UART/BAP, pins, or attempt-037 is authorized. Stop if
continuity would retry a command request, weaken identity/safety validation,
accept malformed evidence, or remove the existing automated phase deadlines.

Final verification: Closed/I/O WebSocket loss now clears only the incomplete
frame and reconnects; protocol, capacity, malformed transition, and oversized
evidence still fail closed. Unavailable HTTP reads wait within the existing
phase deadline, while successful malformed bodies remain terminal. Command
POSTs remain request-once. The 48 focused command-effects tests prove transient
HTTP/WebSocket loss, pause-to-dismiss continuity, malformed inputs, duplicate
generations, missing witnesses, recovery precedence, and deadlines. Mandatory
Cargo, Bright Builds, real firmware build, full Bazel tests, parity with no
validation errors, redaction, reference, and diff gates passed.

Final completion review: The independent-witness policy now matches the design
without adding retries, extending deadlines, or weakening identity, safety, or
evidence validation. The continuity logic and tests were split into focused
submodules to satisfy code-shape limits. No hardware effect occurred during
this task. It is complete and archived at 2026-08-15T10:40:00Z; attempt-037
still requires a separately committed exact-package contract.

### task-api009-programmatic-pilot-attempt-037 | 2026-08-15 | Verify independent witness continuity

- [x] Require clean synchronized pushed `223b7990`, opaque non-empty ignored
      Wi-Fi input, exact HEAD/reference package identity, and absent detector,
      wrapper, command-owned attempt, and public-projection paths.
- [x] Run `just package`; create protected `detector-037`; run exactly one
      `just detect-ultra205`; require its zero exit, private modes, and one
      admitted board-205 device.
- [x] Create separate protected `wrapper-037`, keep `attempt-037` absent, and
      invoke exactly once the existing 600-second `just
      api-command-effects-campaign` with the exact package, opaque input,
      detector output, attempt root, and public projection.
- [x] Publish only a ready independently validated redacted projection;
      otherwise withhold it and record the earliest typed category, command
      progress, closed sensor diagnostic, and recovery booleans.

Effects: one exact-package flash/reset, private Wi-Fi/local-fixture seed,
conservative initialization/mining for at most 600 active seconds, one each
pause/dismiss/IDENTIFY/resume/software restart, HTTP/WebSocket/receive-only USB
observation, same-device recovery, safe stop, child termination, and USB
cleanup. Active freshness and automated/recovery bounds remain unchanged. No
human checkpoint or physical-display claim is included.

Privacy/retry/stop: detector, wrapper, and attempt roots are ignored mode-0700
with mode-0600 files. Never publish origins, ports, hostnames, addresses,
device/USB/network/process identity, credentials, fixture/frame/sensor values,
boot session, or raw traces. Preserve the earliest primary failure; cleanup
runs on every entered lifecycle. Campaign start consumes attempt-037; no
attempt-038 or unchanged retry is authorized. Any preflight/identity/detector/
build ambiguity, non-ready result, malformed evidence, failed safe stop/
recovery/cleanup, or absent device stops with API-009 `implemented` and
evidence withheld. OTA, erase, factory reset, power cycle, UART/BAP, USB duplex,
pins/pads/GPIO, arbitrary settings, external pool, stress, direct controls,
fault injection, non-205 hardware, and human display claims remain prohibited.

Final verification: Clean pushed `f2bc0625`, exact package/reference identity,
one detector, separate wrapper, and one campaign passed admission. The local
post-detector check expected obsolete `port=` syntax after the current detector
had already succeeded with `port: ` and `usb_session: ready`; it was corrected
without rerunning detection. The campaign stopped `hardware_blocked /
command_effects`; safe stop, cleanup, and recovery passed, no secondary failure
remained, private modes held, and public evidence was withheld. The sensor
diagnostic remained `display / ready / under_500_ms` at revision 1.

Final completion review: Exact package identity and safety remained valid. The
campaign made one pause request but stopped before pause confirmation after
roughly eight active seconds. The current `network_correlation_failed` label
cannot distinguish serial, deadline, WebSocket, HTTP, status identity, sample
validation, or state-machine causes. No user action contributed. Attempt-037
is consumed, attempt-038 is not authorized, API-009 remains `implemented`, and
the task is complete and archived at 2026-08-15T11:08:00Z.

### task-api009-command-failure-diagnostic | 2026-08-15 | Identify the exact command observer failure

- [x] Add a redaction-safe first-failure diagnostic with closed command phase
      and cause vocabularies to private campaign evidence and public failure
      output.
- [x] Preserve the first diagnostic through recovery and distinguish serial,
      deadline, WebSocket, HTTP parsing, identity/safety, state-machine,
      terminal, and incomplete-quorum failures.
- [x] Add deterministic regressions for every cause, malformed diagnostics,
      public redaction, and primary-failure precedence.
- [x] Run focused and complete gates, simplify, review, archive, commit, and
      push before any new hardware ordinal.

Authorization: source, deterministic fixtures, local child processes, builds,
and repository verification only. Protected attempt-037 may be used only for
the closed facts already recorded; never publish its identities, origins,
ports, addresses, credentials, boot session, raw traces, or values. No
detector, credentials, package effect, USB/device/network/HTTP session, flash,
reset, mining, recovery, OTA, UART/BAP, pins, or attempt-038 is authorized.
Stop if the diagnostic needs raw data or could replace an earlier failure.

Final verification: Campaign evidence now carries optional
`mining-command-failure-diagnostic-v1` with Rust-enforced phase and cause enums.
The observer records only the first cause, recovery cannot replace it, and an
otherwise incomplete command quorum closes as `terminal / quorum_incomplete`.
The host validates the same closed vocabularies and publishes only schema,
phase, and cause. Fifty focused Rust tests and three focused host tests cover
every enum label, transport/malformed boundaries, first-failure precedence,
malformed optional diagnostics, recovery preservation, and absence of private
values. Mandatory Cargo, Bright Builds, real firmware, full Bazel, parity with
no validation errors, redaction, reference, and diff gates passed.

Final completion review: Diagnostic ownership is split into focused failure,
HTTP-read, WebSocket-witness, and model modules. No command behavior, request
count, safety rule, identity check, deadline, or recovery behavior changed.
No hardware effect occurred during this task. It is complete and archived at
2026-08-15T11:30:00Z; attempt-038 still requires a separate exact-package
contract.

### task-api009-programmatic-pilot-attempt-038 | 2026-08-15 | Capture causal command result

- [x] Require clean synchronized pushed `6602383b`, opaque non-empty ignored
      Wi-Fi input, exact HEAD/reference package, and absent detector, wrapper,
      command-owned attempt, and public-projection paths.
- [x] Run `just package`, then exactly one protected `detector-038`; require
      zero exit, one `port: ` record, `usb_session: ready`, private modes, and
      one admitted board-205 device.
- [x] Use separate protected `wrapper-038`, keep `attempt-038` absent, and run
      exactly once the existing 600-second programmatic campaign.
- [x] Publish only a ready independently validated redacted projection;
      otherwise withhold it and record the earliest category, typed command
      phase/cause, sensor diagnostic, and recovery booleans.

Effects: one exact-package flash/reset, private Wi-Fi/local-fixture seed,
conservative initialization/mining for at most 600 active seconds, one each
pause/dismiss/IDENTIFY/resume/software restart, HTTP/WebSocket/receive-only USB
observation, same-device recovery, safe stop, child termination, and USB
cleanup. No human checkpoint or physical-display claim is included.

Privacy/retry/stop: all private roots/files remain ignored mode 0700/0600.
Never publish identities, origins, ports, addresses, hostnames, credentials,
boot session, values, or traces. Preserve the first failure through recovery.
Campaign start consumes attempt-038; no attempt-039 or unchanged retry is
authorized. Any admission ambiguity, non-ready result, malformed evidence, or
failed safe stop/recovery/cleanup stops with evidence withheld and API-009
`implemented`. OTA, erase, factory reset, power cycle, UART/BAP, USB duplex,
pins/GPIO, arbitrary settings, external pool, stress, direct controls, fault
injection, non-205 hardware, and human display claims remain prohibited.

Final verification: Clean synchronized pushed contract `20f25a76`, exact
package/reference identity, one detector, separate wrapper, and one campaign
passed admission. The campaign stopped `hardware_blocked / command_effects`
with first failure `terminal / phase_deadline`. Every command effect completed
exactly once: notification, pause, dismiss, IDENTIFY render and natural clear,
and resume. Same-package and safety validation, serial transition witnesses,
safe-stop recovery, child cleanup, and USB cleanup passed. The closed sensor
diagnostic was `display / ready / under_500_ms`; no secondary recovery failure
remained, private modes held, and public evidence was withheld.

Final completion review: The host entered `Terminal` after active resume, then
applied its generic 15-second automated-phase deadline while firmware still
owned an admitted 600-active-second resumable lease. The failure is therefore
a contradictory host lifecycle deadline, not a command, sensor, device, or
user failure. Attempt-038 is consumed, attempt-039 is not authorized, API-009
remains `implemented`, and the task is complete and archived at
2026-08-15T12:05:00Z.

### task-api009-terminal-active-duration-contract | 2026-08-15 | Let the admitted lease own terminal timing

- [x] Reproduce attempt-038's `terminal / phase_deadline` at the host phase
      boundary without reading or copying private identities, values, or logs.
- [x] Remove the contradictory 15-second terminal phase deadline while keeping
      the firmware's 600-active-second lease, outer process deadline, serial
      terminal validation, and post-consumption HTTP deadline finite.
- [x] Add deterministic tests proving command completion can wait for lease
      consumption and that post-consumption confirmation still times out.
- [x] Run focused and complete gates, simplify, review, archive, commit, and
      push before any new hardware ordinal.

Authorization: source, deterministic fixtures, local child processes, builds,
and repository verification only. Protected attempt-038 may be used only to
derive the closed command-effect and typed failure facts recorded in its
archive entry; never publish identities, origins, ports, addresses,
credentials, boot session, raw traces, or values. No detector, credentials,
package effect, USB/device/network/HTTP session, flash, reset, mining, recovery,
OTA, UART/BAP, pins, or attempt-039 is authorized. Stop if the change weakens
the admitted 600-active-second bound, terminal validation, cleanup, recovery,
or post-consumption deadline.

Final verification: The deterministic reproduction proves `Terminal` no
longer inherits the generic 15-second command-phase deadline, while the exact
15-second post-consumption HTTP confirmation boundary remains fail-closed. The
serial capture now admits the firmware's 600-second activation budget plus 600
active seconds and 180 seconds of terminal grace. The host child uses the
existing complete 3,850-second transaction budget instead of a hard-coded
900-second limit. Focused Rust and host tests, mandatory Cargo, Bright Builds,
real firmware, full Bazel, parity, redaction, reference, and diff gates pass.

Final completion review: The change removes obsolete human-gated campaign
capture without changing the separately replayable unbounded physical-display
UAT. Command/recovery deadlines, request-once behavior, safety validation,
firmware lease consumption, terminal safe stop, evidence sealing, and cleanup
remain finite and fail-closed. No hardware effect occurred during this task.
It is complete and archived at 2026-08-15T12:35:00Z; attempt-039 still requires
a separately committed exact-package contract.

### task-api009-programmatic-pilot-attempt-039 | 2026-08-15 | Verify lease-owned terminal completion

- [x] Require clean synchronized pushed `57fafecf`, opaque non-empty ignored
      Wi-Fi input, exact HEAD/reference package, and absent detector, wrapper,
      command-owned attempt, and public-projection paths.
- [x] Run `just package`, then exactly one protected `detector-039`; require
      zero exit, one `port: ` record, `usb_session: ready`, private modes, and
      one admitted board-205 device.
- [x] Use separate protected `wrapper-039`, keep `attempt-039` absent, and run
      exactly once the existing 600-active-second programmatic campaign under
      the finite activation, capture, child, recovery, and cleanup bounds.
- [x] Publish only a ready independently validated redacted projection;
      otherwise withhold it and record the earliest category, typed command
      phase/cause, sensor diagnostic, and recovery booleans.

Effects: one exact-package flash/reset, private Wi-Fi/local-fixture seed,
conservative initialization/mining for at most 600 active seconds, one each
pause/dismiss/IDENTIFY/resume/software restart, HTTP/WebSocket/receive-only USB
observation, same-device recovery, safe stop, child termination, and USB
cleanup. No human checkpoint or physical-display claim is included.

Privacy/retry/stop: all private roots/files remain ignored mode 0700/0600.
Never publish identities, origins, ports, addresses, hostnames, credentials,
boot session, values, or traces. Preserve the first failure through recovery.
Campaign start consumes attempt-039; no attempt-040 or unchanged retry is
authorized. Any admission ambiguity, non-ready result, malformed evidence, or
failed safe stop/recovery/cleanup stops with evidence withheld and API-009
`implemented`. OTA, erase, factory reset, power cycle, UART/BAP, USB duplex,
pins/GPIO, arbitrary settings, external pool, stress, direct controls, fault
injection, non-205 hardware, and human display claims remain prohibited.

Final verification: Clean synchronized pushed contract `2d992bc1`, exact
package/reference identity, one detector, separate wrapper, and one campaign
passed admission. The campaign stopped `hardware_blocked / command_effects`
with first failure `pause / command_state_machine`. Notification and pause
completed exactly once, including the HTTP generation and serial safe-stop
witness. One dismiss request was issued but not confirmed; no IDENTIFY, resume,
or restart request followed. Same-package and safety validation passed. USB
cleanup and private modes passed, the closed sensor diagnostic was
`display / ready / under_250_ms`, and public evidence was withheld.

Final completion review: The dismiss exchange did not produce an accepted
HTTP result, and the host collapsed that request boundary into a generic state
machine cause instead of preserving complete-write versus pre-delivery facts.
Recovery then issued a redundant pause despite the already-proved paused safe
stop; that request failed and produced the secondary recovery failure. No user
action contributed. Attempt-039 is consumed, attempt-040 is not authorized,
API-009 remains `implemented`, and the task is complete and archived at
2026-08-15T13:05:00Z.

### task-api009-command-delivery-and-safe-recovery | 2026-08-15 | Preserve request ambiguity and proved safe stop

- [x] Reproduce a fully flushed command request whose response is unavailable;
      require the state machine to wait for the authoritative generation and
      postcondition without issuing a second request.
- [x] Distinguish command-request failures from generic state-machine failures
      in the closed public diagnostic without exposing transport details.
- [x] Preserve an already HTTP- and serial-confirmed paused safe stop through a
      later command failure without issuing a redundant recovery pause.
- [x] Add deterministic delivery, explicit rejection, pre-delivery failure,
      recovery, redaction, and primary-precedence tests.
- [x] Run focused and complete gates, simplify, review, archive, commit, and
      push before any new hardware ordinal.

Authorization: source, deterministic local HTTP/child fixtures, builds, and
repository verification only. Protected attempt-039 may be used only for the
closed facts recorded in its archive entry; never publish identities, origins,
ports, addresses, credentials, boot session, raw traces, or values. No
detector, credentials, package effect, USB/device/network session, flash,
reset, mining, recovery, OTA, UART/BAP, pins, or attempt-040 is authorized.
Stop if delivery ambiguity could claim an effect without its generation and
machine postcondition, retry a command, accept an explicit non-200 response,
or weaken recovery from a state not already proved paused and hardware-safe.

Final verification: The real TCP regression proves a fully flushed dismiss
request with no response advances only after the authoritative generation and
postcondition, with one request total. Explicit non-200 and pre-delivery
failures remain rejected. The public diagnostic now distinguishes the closed
`command_request` cause. A later request failure reuses a safe stop only when
pause is already confirmed by HTTP and receive-only USB, no resume was
requested, and the current serial state remains safely stopped. Focused Rust
and Bun tests, mandatory Cargo, Bright Builds, the real ESP32-S3 firmware
build, full Bazel, parity/progress, redaction, reference, and diff gates pass.

Final completion review: The transport observation already contained the
needed complete-write boundary, so the simplest fix preserves it and delegates
effect truth to command status instead of adding retries or another protocol.
The recovery change removes an unsafe redundant request without weakening
unproved or resumed-state recovery. The helper split keeps the orchestrator
within its file-size limit and its Bazel source membership is explicit. No
hardware effect occurred. The task is complete and archived; a new hardware
ordinal requires its own committed exact-package contract.

### task-api009-programmatic-pilot-attempt-040 | 2026-08-15 | Verify status-owned ambiguous delivery

- [x] Require clean synchronized pushed `333674f3`, opaque non-empty ignored
      Wi-Fi input, exact HEAD/reference package, and absent detector, wrapper,
      command-owned attempt, and public-projection paths.
- [x] Run `just package`, then exactly one protected `detector-040`; require
      zero exit, one `port: ` record, `usb_session: ready`, private modes, and
      one admitted board-205 device.
- [x] Use separate protected `wrapper-040`, keep `attempt-040` absent, and run
      exactly once the existing 600-active-second programmatic campaign under
      the finite activation, capture, child, recovery, and cleanup bounds.
- [x] Publish only a ready independently validated redacted projection;
      otherwise withhold it and record the earliest category, typed command
      phase/cause, sensor diagnostic, and recovery booleans.

Effects: one exact-package flash/reset, private Wi-Fi/local-fixture seed,
conservative initialization/mining for at most 600 active seconds, one each
pause/dismiss/IDENTIFY/resume/software restart, HTTP/WebSocket/receive-only USB
observation, same-device recovery, safe stop, child termination, and USB
cleanup. No human checkpoint or physical-display claim is included.

Privacy/retry/stop: all private roots/files remain ignored mode 0700/0600.
Never publish identities, origins, ports, addresses, hostnames, credentials,
boot session, values, or traces. Preserve the first failure through recovery.
Campaign start consumes attempt-040; no attempt-041 or unchanged retry is
authorized. Any admission ambiguity, non-ready result, malformed evidence, or
failed safe stop/recovery/cleanup stops with evidence withheld and API-009
`implemented`. OTA, erase, factory reset, power cycle, UART/BAP, USB duplex,
pins/GPIO, arbitrary settings, external pool, stress, direct controls, fault
injection, non-205 hardware, and human display claims remain prohibited.

Final verification: Clean synchronized pushed contract `1511c434`, exact
package/reference identity, one detector, separate wrapper, and one campaign
passed admission. Every command effect completed once: notification, pause,
dismiss, IDENTIFY render and natural clear, resume, and the 600-active-second
lease. Same-package and safety validation passed. The campaign then stopped
`hardware_blocked / command_effects` with first failure
`terminal / serial_ended`; USB cleanup and private modes passed, no recovery
request or secondary recovery failure occurred, and public evidence was
withheld.

Final completion review: The receive-only owner closed cleanly as soon as the
serial analyzer accepted lease consumption, while the concurrent network
worker had not yet published its post-terminal HTTP confirmation. The worker
therefore saw serial end at the lifecycle boundary and failed before the
terminal join. This is a host ownership race after all command effects, not a
device command failure or user action. Attempt-040 is consumed, attempt-041 is
not authorized, API-009 remains `implemented`, and the task is complete and
archived.

### task-api009-terminal-capture-handoff | 2026-08-15 | Join serial closure to terminal HTTP proof

- [x] Reproduce receive-only capture ending immediately after its authoritative
      serial analyzer accepts the consumed terminal marker while the network
      worker still awaits its post-terminal HTTP sample.
- [x] Hand the analyzer's closed terminal facts to the network coordinator
      before marking serial input finished; reject contradictory terminal
      views and preserve the first failure.
- [x] Allow a worker with authoritative terminal consumption to finish the
      existing 15-second HTTP confirmation after receive-only input closes;
      missing consumption still fails immediately as `serial_ended`.
- [x] Add deterministic ordering, contradiction, bounded-deadline, cleanup,
      redaction, and primary-precedence regressions.
- [x] Run focused and complete gates, simplify, review, archive, commit, and
      push before any new hardware ordinal.

Authorization: source, deterministic fixtures, local child processes, builds,
and repository verification only. Protected attempt-040 may be used only for
the closed facts recorded in its archive entry; never publish identities,
origins, ports, addresses, credentials, boot session, raw traces, or values.
No detector, credentials, package effect, USB/device/network session, flash,
reset, mining, recovery, OTA, UART/BAP, pins, or attempt-041 is authorized.
Stop if the handoff can manufacture terminal consumption, replace a parser
failure, extend the 15-second confirmation bound, or weaken exact-package,
safe-stop, cleanup, or request-once behavior.

Final verification: Deterministic regressions prove the serial analyzer's
accepted terminal fact is installed before input closure, a consumed terminal
keeps the post-terminal HTTP join alive, missing consumption still maps to
`serial_ended`, contradictory pool-persistence views fail closed, an earlier
failure remains primary, and the 15-second deadline is unchanged. Focused and
all-feature Rust tests, mandatory Cargo, Bright Builds, real firmware, all 45
Bazel test targets, parity/progress, redaction, reference, and diff gates pass.

Final completion review: The fix reconciles one closed terminal fact from the
authoritative serial capture into the concurrent worker; it does not replay
logs, extend the USB reader, retry commands, or add another protocol. The
worker may use only its pre-existing bounded HTTP deadline after USB closes.
The change is smaller and safer than retaining the serial reader solely for a
network postcondition. No hardware effect occurred during this task. It is
complete and archived; attempt-041 still requires a separately committed
exact-package contract.

### task-api009-programmatic-pilot-attempt-041 | 2026-08-15 | Verify terminal capture handoff

- [x] Required clean synchronized pushed `60457bf1`, opaque non-empty ignored
      Wi-Fi input, exact HEAD/reference package, and absent detector, wrapper,
      command-owned attempt, and public-projection paths.
- [x] Ran `just package`, then exactly one protected `detector-041`; observed
      zero exit, one admitted board-205 device, and private artifact modes.
- [x] Used separate protected `wrapper-041` and ran exactly once the existing
      600-active-second programmatic campaign under its finite bounds.
- [x] Withheld the public projection after the non-ready result and recorded
      the closed terminal command diagnostic and safe cleanup facts.

Effects: one exact-package flash/reset, private Wi-Fi/local-fixture seed,
conservative initialization/mining for at most 600 active seconds, one each
pause/dismiss/IDENTIFY/resume/software restart, HTTP/WebSocket/receive-only USB
observation, same-device recovery, safe stop, child termination, and USB
cleanup. No human checkpoint or physical-display claim was included.

Privacy/retry/stop: all private roots/files remained ignored mode 0700/0600.
No identities, origins, ports, addresses, hostnames, credentials, boot session,
values, or traces were published. Attempt-041 is consumed; no attempt-042 or
unchanged retry is authorized. API-009 remains `implemented`.

Completion review: The campaign stopped `hardware_blocked` with the closed
command diagnostic `terminal / serial_ended`; cleanup completed, recovery was
not attempted, and evidence was withheld. Privacy-safe counters show the
analyzer accepted the consumed terminal and thousands of valid markers after
transient UTF-8/JSON framing damage, but the handoff discarded that terminal
because the independent serial result was non-ready. This was a host evidence-
orchestration defect, not a user action. Follow-up work is tracked by
`task-api009-serial-resynchronization`.

### task-api009-serial-resynchronization | 2026-08-15 | Preserve recovered marker streams

- [x] Reproduced attempt-041's production-shaped boundary: a trusted serial
      stream has recoverable UTF-8/JSON framing damage, later accepts a valid
      consumed marker, and closes while terminal HTTP confirmation is pending.
- [x] Treated only framing damage followed by a fully valid marker as recovered;
      retained private corruption counters while keeping schema, semantic,
      contract, and unrecovered final corruption failures terminal.
- [x] Handed an accepted consumed marker to the network coordinator independently
      of an unrelated serial failure so `serial_ended` cannot replace the true
      primary failure; preserved contradiction and earliest-failure precedence.
- [x] Ran focused regressions and the complete Cargo, Bright Builds, firmware,
      Bazel, parity, redaction, reference-cleanliness, and diff-review gates.

This task was software-only. It did not access USB, device/network state,
credentials, or protected raw traces. Attempt-041 remains consumed and no
attempt-042 or unchanged hardware retry was authorized by this task. The fix
does not accept malformed schema or semantics, manufacture a terminal marker,
extend a deadline, retry a command, or weaken exact-package, safe-stop, cleanup,
privacy, or request-once behavior.

Completion review: The serial analyzer now keeps UTF-8/JSON framing damage
pending across receive chunks and clears it only after a fully valid marker
proves resynchronization. Unrecovered corruption still closes as
`marker_invalid`, while well-framed schema and semantic failures remain
immediate. An independently accepted consumed terminal always reaches the
network join, so a serial primary failure cannot be mislabeled `serial_ended`.
Production-shaped split-chunk, unrecovered corruption, schema failure, and
terminal handoff regressions pass. Full verification is clean; no hardware
effect occurred.

### task-api009-programmatic-pilot-attempt-042 | 2026-08-15 | Verify recovered serial framing

- [x] Required clean synchronized pushed `2a97230c`, opaque non-empty ignored
      Wi-Fi input, exact HEAD/reference package, and absent detector, wrapper,
      command-owned attempt, and public-projection paths.
- [x] Ran `just package`, then exactly one protected `detector-042`; observed
      zero exit, one admitted board-205 device, and private artifact modes.
- [x] Used separate protected `wrapper-042`, kept `attempt-042` absent, and ran
      exactly once the existing 600-active-second programmatic campaign under
      its finite bounds.
- [x] Withheld the public projection after the non-ready result and recorded
      the typed terminal-state, serial, recovery, and cleanup facts.

Effects: one exact-package flash/reset, private Wi-Fi/local-fixture seed,
conservative initialization/mining for at most 600 active seconds, one each
pause/dismiss/IDENTIFY/resume/software restart, HTTP/WebSocket/receive-only USB
observation, same-device recovery, safe stop, child termination, and USB
cleanup. No human checkpoint or physical-display claim was included.

Privacy/retry/stop: all private roots/files remained ignored mode 0700/0600.
No identities, origins, ports, addresses, hostnames, credentials, boot session,
values, or traces were published. Attempt-042 is consumed; no attempt-043 or
unchanged retry is authorized. API-009 remains `implemented`.

Completion review: Every requested command effect completed exactly once,
including IDENTIFY render/clear receipts and both independent transition
witnesses. The 600-active-second lease reached its terminal reason, serial
capture was clean apart from one harmless trailing fragment, and USB/process
cleanup passed. The public result stopped `hardware_blocked` with the secondary
closed diagnostic `terminal / serial_ended`; recovery was not attempted and
the projection remained absent.

The authoritative terminal marker instead exposed the primary lifecycle
contradiction: `campaign_lease_consumed` was paired with campaign state `armed`,
so the analyzer correctly rejected terminal confirmation as
`terminal_state_unconfirmed`. This is a firmware state-transition defect, not
serial corruption or user action. Follow-up software work is tracked by
`task-api009-consumed-terminal-state`.

### task-api009-consumed-terminal-state | 2026-08-15 | Make lease consumption terminal

- [x] Reproduced terminal lease consumption both while a resumable safe stop
      was pending and after its hardware confirmation had left the lease armed.
- [x] Made every terminal transition cancel resumability and settle an
      already-stopped admitted lease as `consumed` without issuing another
      hardware stop or manufacturing a safe-stop receipt.
- [x] Added state-machine, marker, host-handoff, and primary-failure
      regressions for the attempt-042 signature using deterministic fixtures.
- [x] Ran focused and complete repository gates, simplified and reviewed the
      diff, and prepared the verified correction for commit and push before
      any attempt-043 contract.

Authorization: This task was software-only. It did not access credentials,
USB, device/network state, identities, origins, ports, addresses, boot session,
raw traces, or sensor values. It performed no detector, flash/reset, mining,
recovery, OTA, erase, factory reset, UART/BAP, USB duplex, pins/GPIO, direct
control, fault injection, or attempt-043 effect. API-009 remains `implemented`
and public evidence remains withheld.

Final verification: The original state machine deterministically reproduced
both `campaign_lease_consumed / armed` races. The corrected lifecycle cancels
resumability at the terminal boundary, consumes an already-confirmed stopped
lease directly, and lets an in-flight hardware confirmation finish terminally.
No duplicate `SafeStopHardware` effect is emitted. The host capture handoff now
classifies the old contradictory marker as `terminal_state_unconfirmed` before
serial closure, preserving it as a `serial_witness` diagnostic instead of the
later `serial_ended` symptom.

Completion review: Focused production-session and campaign tests, ordered
Cargo format/strict-lint/build/test, Bright Builds, the real ESP32-S3 firmware
build, all 45 Bazel tests, parity/progress, redaction, reference cleanliness,
sensitive-output, and diff checks pass. The fix changes one terminal boundary
and one diagnostic handoff; it does not retry effects, loosen marker semantics,
extend deadlines, or weaken package, privacy, safety, cleanup, or request-once
contracts. A fresh hardware ordinal requires its own committed contract.

### task-api009-programmatic-pilot-attempt-043 | 2026-08-15 | Verify terminal lease consumption

- [x] Required clean synchronized pushed `cf41ecaf`, opaque non-empty ignored
      Wi-Fi input, exact HEAD/reference package, and absent detector, wrapper,
      command-owned attempt, and public-projection paths.
- [x] Ran `just package`, then exactly one protected `detector-043`; observed
      zero exit, one admitted board-205 device, and private artifact modes.
- [x] Used separate protected `wrapper-043`, kept `attempt-043` absent, and ran
      exactly once the existing 600-active-second programmatic campaign under
      its finite bounds.
- [x] Withheld the public projection after the non-ready result and recorded
      the earliest typed campaign, restart-session, and cleanup facts.

Effects: one exact-package flash/reset, private Wi-Fi/local-fixture seed,
conservative initialization/mining for at most 600 active seconds, one each
pause/dismiss/IDENTIFY/resume/software restart, HTTP/WebSocket/receive-only USB
observation, same-device recovery, safe stop, child termination, and USB
cleanup. No human checkpoint or physical-display claim was included.

Privacy/retry/stop: all private roots/files remained ignored mode 0700/0600.
No identities, origins, ports, addresses, hostnames, credentials, boot session,
values, or traces were published. Attempt-043 is consumed; no attempt-044 or
unchanged retry is authorized. API-009 remains `implemented`.

Completion review: The command-effects campaign itself passed with terminal
lease consumption, clean serial analysis, confirmed safe stop, and ready USB
cleanup. The subsequent shared restart transaction sent exactly one request,
received its successful response, observed service loss and post-restart bytes,
and retained the same stable physical device with cleanup complete. It then
stopped `service_recovery_timeout`: its first recovery GET connected before
service shutdown completed, wrote the request, received no response, and used
the complete remaining 360-second transaction deadline as that one socket read
timeout. No later recovery poll could run. The aggregate wrapper therefore
closed `hardware_blocked` with cleanup uncredited and withheld the projection.
This is a host observation-budget defect, not a device command, safe-stop, or
user-action failure. Follow-up software work is tracked by
`task-device-session-recovery-poll-budget`.

### task-device-session-recovery-poll-budget | 2026-08-15 | Bound each post-restart HTTP poll

- [x] Reproduced the attempt-043 failure at the real HTTP/device-session seam:
      an accepted recovery connection that never responds no longer consumes
      the entire transaction deadline or prevents a later successful poll.
- [x] Gave each recovery observation a finite per-exchange budget capped by
      the overall transaction deadline, without changing request-once restart,
      exact-package, same-device, or terminal-category behavior.
- [x] Added a production-shaped stalled-first-request regression, retained the
      real-child transaction/file integration test, and proved the overall
      timeout remains authoritative.
- [x] Ran focused and complete repository gates, simplified and reviewed the
      diff, and prepared the software-only correction for commit and push
      before any new hardware contract.

Authorization: This task was software-only. It read protected attempt-043
artifacts only to derive bounded categories and booleans and did not publish
private response bodies, identities, origins, ports, addresses, hostnames,
credentials, boot sessions, traces, or sensor values. It performed no detector,
USB, flash/reset, mining, restart, OTA, erase, factory reset, UART/BAP,
pins/GPIO, direct control, fault injection, attempt-044, or public evidence
promotion.

Verification: The new loopback regression accepts and fully reads a first HTTP
request while deliberately withholding its response, proves that exchange
times out independently, then accepts a second request and returns a successful
response before the unchanged overall deadline. A separate boundary test proves
the per-exchange deadline never extends the overall transaction deadline. All
82 device-session unit tests and four CLI integrations pass, including the
real-child transaction and protected-file projection test. Ordered Cargo
format/strict-lint/build/test, Bright Builds, real firmware build, all 45 Bazel
tests, parity/progress, redaction, and pinned-reference checks pass.

Completion review: Post-restart GETs now use the shared HTTP transport's
10-second total-exchange budget, capped by the remaining transaction window.
Only recovery observation changed; baseline confirmation and the single restart
request retain their existing lifecycle. The fix adds no retry of the restart
effect, no origin discovery, no new protocol, and no relaxed evidence quorum.
API-009 remains `implemented`; a fresh hardware ordinal still requires its own
committed contract.

### task-api009-programmatic-pilot-attempt-044 | 2026-08-15 | Verify bounded restart recovery polling

- [x] Required clean synchronized pushed `5ba7c192`, opaque non-empty ignored
      Wi-Fi input, exact current HEAD/reference package, and absent detector,
      wrapper, command-owned attempt, and public-projection paths.
- [x] Ran `just package`, then exactly one protected `detector-044`; observed
      zero exit, one admitted board-205 device, and private artifact modes.
- [x] Used separate protected `wrapper-044`, kept `attempt-044` absent, and ran
      exactly once the existing 600-active-second programmatic campaign under
      its finite bounds.
- [x] Independently validated and retained the ready redacted projection with
      exact command, restart-session, safe-stop, and cleanup facts.

Objective and effects: verified that the complete no-human API-009 command
campaign remained accepted and the shared restart transaction recovered after
service loss using bounded observations. The sole run performed one
exact-package flash/reset, private Wi-Fi/local-fixture seed, bounded
conservative mining, one each pause/dismiss/IDENTIFY/resume/software-restart,
HTTP/WebSocket/receive-only USB observation, same-device recovery, safe stop,
child termination, and USB cleanup. No human checkpoint or physical-display
claim was included.

Privacy/retry/stop: detector, wrapper, and attempt roots remained ignored mode
0700 with mode-0600 files. The aggregate projection passed redaction and no
private identities, origins, ports, addresses, hostnames, credentials, boot
sessions, values, response bodies, or traces were published. Attempt-044 is
consumed; no attempt-045 or unchanged programmatic retry is authorized.

Verification: The sealed v1 projection binds exact source `16f6c8de`, the
pinned reference, exact package/workflow digests, board 205, the deterministic
local fixture, one pause/resume/IDENTIFY/dismiss request apiece, every
claim-specific postcondition, IDENTIFY render/clear receipts, retained and
receive-only serial transition witnesses, same-boot/package safety, terminal
HTTP and pool persistence, mining disabled, hardware control disabled,
confirmed safe stop, and cleanup. The restart projection is `ready` with one
acknowledged request, service loss, correlated pre/post serial, the same stable
physical device, trusted origin, exact recovered build, changed boot session,
ordinal N+1, software reset, matching postcondition, and cleanup.

Completion review: Attempt-044 closes the entire programmatic API-009 quorum
and proves the per-exchange recovery budget fixed attempt-043 without weakening
the overall deadline or request-once effect. No recovery request was needed and
no secondary recovery failure occurred. The independently replayable physical
display UAT remains the sole promotion prerequisite and is tracked by
`task-api009-physical-display-uat-001`.

### task-programmatic-device-verification-platform | 2026-08-15 | Centralize autonomous device transactions and proof

- [x] Add a privacy-safe, access-gated `/api/system/command-status` extension
      with boot-scoped command generations, state revisions, and display-render
      receipts published only after a successful framebuffer flush.
- [x] Deepen `bitaxe-device-session` behind one typed transaction interface
      with production and deterministic adapters, read-only inspection,
      request-once effects, same-device continuity, recovery, cleanup, typed
      failures, and sealed public projections.
- [x] Migrate API command effects, hostname durability, and partition-layout
      OTA/restart orchestration to the common transaction implementation; all
      new live workflows must use the same interface.
- [x] Add pure, simulated, and real-child regressions for every public API-009
      failure signature, transport loss, stale/duplicate generations, display
      failure, restart ambiguity, projection integrity, privacy, cleanup, and
      earliest-failure precedence.
- [x] Pass focused and full repository gates, review the diff for unintended
      effects, commit, and push before authorizing any fresh hardware attempt.
- [x] Plan and run sequential detector-gated exact-package pilot attempts only
      after a complete no-human simulated command-effects campaign passes.
- [x] Preserve programmatic evidence independently from one replayable,
      unbounded-readiness physical display UAT required for API-009 promotion.

Software verification review (2026-08-15): the complete no-human simulated
command-effects campaign and its typed failure regressions pass, as do the
real-child transaction seam, all Cargo format/lint/build/test gates, all 44
Bazel tests, firmware build, Bright Builds, parity/progress, redaction, pinned
reference cleanliness, and diff checks. The shared interface reuses the
authoritative reboot/OTA lifecycle; its compatibility commands contain no
independent device lifecycle. The implementation is ready for commit and push.
No fresh hardware attempt ran under this software task.

Published software checkpoint: commit `c9faaaa0` is pushed to `origin/main`.
The commit-and-push workflow confirmed the remote default branch, fetched the
current remote, verified a fast-forward relationship, and pushed without a
history rewrite.

Software authorization and privacy: source, tests, deterministic loopback and
real-child fixtures, documentation, task records, builds, local protected
mode-`0700` roots, and mode-`0600` private artifacts were authorized. Public
outputs contain only closed categories, booleans, counts, opaque boot sessions,
monotonic revisions, and digests. They contain no origins, hostnames, settings
values, addresses, ports, USB/network/process identities, credentials, frame
text, sensor values, commands, or raw traces.

Hardware gate: no fresh device attempt was authorized by this software task.
Every pilot used its own complete task contract, clean pushed source and exact
package, `just detect-ultra205`, fresh ordinal, finite automated effect/recovery
bounds, evidence withholding, and accepted stop categories. HTTP remains the
only command transport; WebSocket and receive-only native USB are observers.
External UART/BAP, USB request/response, erase, factory reset, power-cycle,
fault injection, arbitrary settings writes, mining stress, direct controls,
pins, pads, GPIO, probes, jumpers, and non-205 devices remain prohibited.

Completion review: The platform implementation and deterministic proof are
complete, and attempt-044 supplied the sealed exact-package hardware projection
for all programmatic API-009 effects. The separate display UAT intentionally
remains active because only a human can confirm illuminated pixels; it can
replay without rerunning or invalidating the completed programmatic campaign.

### task-api009-programmatic-refresh-attempt-045 | 2026-08-15 | Refresh exact-package proof after durable UAT fix

- [x] Require a clean synchronized pushed contract HEAD containing correction
      `3e6d88f6`, opaque non-empty ignored Wi-Fi input, the exact current
      HEAD/reference package, and absent protected `detector-045`, `wrapper-045`,
      command-owned `attempt-045`, and new public projection paths.
- [x] Run `just package`, then exactly one protected `just detect-ultra205`;
      proceed only for zero exit, exactly one admitted board-205 device, and
      mode-0700/mode-0600 private artifacts.
- [x] Run exactly once
      `just api-command-effects-campaign --private-root
      scratch/api009-command-effects/attempt-045 --package-manifest
      bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json
      --wifi-credentials wifi-credentials.json --detector-output
      scratch/api009-command-effects/detector-045/detector.stdout --projection
      docs/parity/evidence/api009-command-effects/command-effects-projection-attempt-045.json
      --duration-seconds 600` from a separate protected `wrapper-045`.
- [x] Independently validate the typed non-ready result, exact package and
      workflow identity, evidence withholding, successful recovery, safe stop,
      process/USB cleanup, private modes, sensitive-output absence, and diff.
- [x] Archive this task after its one typed failure disposition; never retry
      attempt-045 or start attempt-046 without a diagnosed and verified
      contract/software change.

Objective and effects: refresh API-009's no-human programmatic evidence after
the durable delayed-UAT firmware change. The sole run may perform one
exact-package flash/reset, private Wi-Fi/local-fixture seed, bounded
conservative mining, one each pause/dismiss/IDENTIFY/resume/software-restart,
HTTP/WebSocket/receive-only USB observation, same-device recovery, safe stop,
child termination, and USB cleanup. No human checkpoint or physical-display
claim is included.

Privacy/recovery/stop: detector, wrapper, and attempt roots are ignored mode
0700 with mode-0600 files. The public projection may contain only the existing
closed aggregate API-009 schema and must expose no private identities, origins,
ports, addresses, hostnames, credentials, boot sessions, values, response
bodies, or traces. Existing finite effect/recovery/cleanup bounds and earliest-
failure precedence remain authoritative. Any detector failure, non-ready
campaign result, recovery failure, malformed evidence, cleanup failure, or
identity/build mismatch consumes attempt-045, withholds the new projection,
records the typed disposition, and stops.

Prohibited: external UART/BAP, native USB request/response, pins/pads/GPIO,
erase/factory reset, OTA, arbitrary settings writes, fault injection, direct
voltage/frequency/fan/thermal/power control, mining stress, physical power
cycling, non-205 hardware, a second campaign invocation, or a visual claim.

Verification: The clean pushed contract was packaged, one protected detector
admitted exactly one board-205 device, and the campaign was invoked exactly
once. It returned the typed `hardware_blocked` category at the pause join after
one pause request. No restart occurred and the new public projection was
withheld. Recovery later confirmed pause and safe stop; child/process and USB
cleanup succeeded; every protected artifact retained its required private mode.

Completion review: Attempt-045 was consumed and stopped safely. The failure was
not caused by user timing or display observation. Diagnosis found that the
serial safe-stop fact could be replaced before the later HTTP pause generation,
while `PauseJoinState` latched only the HTTP half of the asynchronous proof.
Follow-up task `task-api009-pause-join-asynchronous-witness` owns the software
correction and verification. No attempt-046 is authorized by this archive.

### task-api009-pause-join-asynchronous-witness | 2026-08-15 | Latch independent pause witnesses

- [x] Diagnose attempt-045's typed pause-phase deadline without rerunning the
      consumed hardware campaign or exposing protected evidence.
- [x] Fix the command-effects pause join so boot-scoped HTTP and receive-only
      serial facts may arrive in separate polling cycles.
- [x] Add unit and production-seam regressions for genuinely disjoint witness
      ordering while preserving request-once and fail-closed deadline behavior.
- [x] Pass the ordered Rust and full repository gates, review the diff, commit,
      and push the verified software correction.
- [x] Create, commit, and push a separate attempt-046 contract only after the
      correction is verified; do not run hardware under this software task.

Root cause: the serial observer correctly replaces its current safe-stop value
on every campaign marker, but `PauseJoinState` latched only the later HTTP fact.
Its nominal serial-first test repeated `true` during the HTTP observation, so
it did not model independent asynchronous observations. Attempt-045 therefore
timed out when the one-shot serial proof preceded the authoritative HTTP pause
generation instead of overlapping it.

Safety/privacy: source, deterministic tests, builds, and aggregate task records
only. This task authorizes no flash, device command, USB access, Wi-Fi use,
mining, display claim, or other hardware effect. Protected attempt-045 inputs
remain private and no withheld projection may be reconstructed or published.

Verification: Focused pause-join and programmatic-seam regressions pass. The
ordered Cargo format, clippy, build, and all-feature test gates pass. Bright
Builds, the ESP32-S3 firmware build, all 45 Bazel tests, parity and progress,
redaction, pinned-reference cleanliness, and diff checks pass. One parity run
encountered a transient host resource error after report generation; its
isolated retry passed without any source or contract change.

Completion review: The minimal correction belongs in the boot-scoped join, not
the serial observer: it preserves current marker truth for other consumers
while retaining the two independent facts only for this bounded pause command.
The correction was published as `dcb01c58`; the separate
`task-api009-programmatic-refresh-attempt-046` contract owns all subsequent
hardware effects and disposition.

### task-api009-programmatic-refresh-attempt-046 | 2026-08-15 | Retry after asynchronous pause-join fix

- [x] Require a clean synchronized pushed contract HEAD containing correction
      `dcb01c58`, opaque non-empty ignored Wi-Fi input, the exact current
      HEAD/reference package, and absent protected `detector-046`, `wrapper-046`,
      command-owned `attempt-046`, and new public projection paths.
- [x] Run `just package`, then exactly one protected `just detect-ultra205`;
      proceed only for zero exit, exactly one admitted board-205 device, and
      mode-0700/mode-0600 private artifacts.
- [x] Run exactly once
      `just api-command-effects-campaign --private-root
      scratch/api009-command-effects/attempt-046 --package-manifest
      bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json
      --wifi-credentials wifi-credentials.json --detector-output
      scratch/api009-command-effects/detector-046/detector.stdout --projection
      docs/parity/evidence/api009-command-effects/command-effects-projection-attempt-046.json
      --duration-seconds 600` from a separate protected `wrapper-046`.
- [x] Independently validate a ready redacted projection, exact package and
      workflow identity, every command/restart postcondition, safe stop,
      process/USB cleanup, private modes, sensitive-output absence, and diff.
- [x] Archive this task after one accepted projection; never retry attempt-046
      or start attempt-047 without a diagnosed and verified contract/software
      change.

Objective and effects: refresh API-009's no-human programmatic evidence after
the durable delayed-UAT firmware change and the asynchronous pause-join fix.
The sole run may perform one exact-package flash/reset, private
Wi-Fi/local-fixture seed, bounded conservative mining, one each
pause/dismiss/IDENTIFY/resume/software-restart, HTTP/WebSocket/receive-only USB
observation, same-device recovery, safe stop, child termination, and USB
cleanup. No human checkpoint or physical-display claim is included.

Privacy/recovery/stop: detector, wrapper, and attempt roots are ignored mode
0700 with mode-0600 files. The public projection may contain only the existing
closed aggregate API-009 schema and must expose no private identities, origins,
ports, addresses, hostnames, credentials, boot sessions, values, response
bodies, or traces. Existing finite effect/recovery/cleanup bounds and earliest-
failure precedence remain authoritative. Any detector failure, non-ready
campaign result, recovery failure, malformed evidence, cleanup failure, or
identity/build mismatch consumes attempt-046, withholds the new projection,
records the typed disposition, and stops.

Prohibited: external UART/BAP, native USB request/response, pins/pads/GPIO,
erase/factory reset, OTA, arbitrary settings writes, fault injection, direct
voltage/frequency/fan/thermal/power control, mining stress, physical power
cycling, non-205 hardware, a second campaign invocation, or a visual claim.

Verification: The clean pushed contract was packaged and one protected
detector admitted exactly one board-205 device. The campaign ran exactly once
and emitted a sealed v1 projection binding the exact source, reference,
package, and workflow identities. One pause, dismiss, IDENTIFY, resume, and
software restart each met their claim-specific HTTP, render, retained/serial,
same-device, build, boot-session, N+1 ordinal, service-loss, and postcondition
requirements. Safe stop, disabled mining/control, cleanup, private modes, and
redaction independently validate. Recovery was not required.

Completion review: Attempt-046 closes the refreshed no-human programmatic
quorum after the asynchronous join correction. The detector validator initially
expected an automation JSON envelope while the current CLI emitted colon-
delimited labels; the already successful protected capture was validated in
place and detection was not rerun. The separate physical display UAT remains
the only API-009 promotion prerequisite and is owned by
`task-api009-physical-display-uat-001`.

### task-api009-physical-display-uat-001 | 2026-08-15 | Confirm illuminated IDENTIFY pixels

- [x] Commit and push the independently validated attempt-044 programmatic
      projection and this UAT contract before sending another IDENTIFY request.
- [x] Consume attempt-001 as a pre-effect admission failure: the private root
      remained empty, no IDENTIFY request was sent, no observation was requested,
      and public UAT evidence was withheld.
- [x] Make delayed replay durable by keeping the currently connected origin
      observable through private receive-only USB, admitting exactly one fresh
      same-session runtime-origin observation, and returning a typed terminal
      category before IDENTIFY when admission fails.
- [x] Rebuild and rerun the no-human programmatic campaign against the changed
      exact package before reusing its evidence for a physical UAT.
- [x] Consume attempt-002 as a pre-effect host failure: the fresh private root
      did not exist, the CLI failed before creating an admission receipt, no
      IDENTIFY request was sent, and no visual observation was requested.
- [x] Make `display-uat-live` create its own fresh mode-0700 root atomically and
      add a real-child regression proving an absent root reaches typed
      pre-effect admission with a mode-0600 receipt.
- [x] Consume attempt-003 as a second pre-effect host failure: Bazel launched
      the CLI outside the workspace while relative inputs remained unresolved,
      so no root, receipt, IDENTIFY request, or observation occurred.
- [x] Resolve display-live and finalize paths against Bazel's trusted workspace
      directory and prove the exact relative-path launch shape reaches a typed
      zero-request receipt from a different process working directory.
- [x] Wait without a deadline for the user to say they are watching; then run
      one fresh protected detector, one bounded receive-only origin capture, and
      one bounded `api-command-display-uat` machine pass against the exact
      programmatic evidence binding.
- [x] Ask for one durable response covering both observations: the IDENTIFY
      frame was visibly rendered and later cleared back to a non-IDENTIFY
      frame. A missed observation may replay this UAT without rerunning or
      invalidating the programmatic campaign.
- [x] Write only the two mode-0600 confirmation files from the user's actual
      observations, finalize the redacted UAT projection, run all promotion
      gates, and promote API-009 only if both projections validate together.

Exact command and effects: after fresh `just detect-ultra205` admission, run
one bounded `just monitor --board 205 --port <detector-port>
--capture-timeout-seconds 20` capture into a new mode-0600 private file, then
`just api-command-display-uat --port <detector-port> --private-root
scratch/api009-display-uat/attempt-005 --intent-input
scratch/api009-command-effects/attempt-046/display-uat-intent.private.json
--runtime-observation-input <private-monitor-file> --programmatic-evidence
docs/parity/evidence/api009-command-effects/command-effects-projection-attempt-046.json`.
The machine pass may perform read-only HTTP/USB inspection and exactly one
bounded HTTP IDENTIFY request. It must prove the successful framebuffer receipt
and natural non-IDENTIFY clear receipt from the same boot and package. No flash,
restart, mining, settings mutation, voltage/frequency/fan/thermal/power control,
OTA, erase, factory reset, UART/BAP, USB duplex, pins/GPIO, or other hardware
effect is authorized.

Readiness/replay/stop: human readiness and the later response wait are
unbounded and never expire because of chat latency or absence. Each machine
pass retains finite HTTP/render/clear and cleanup limits. If the user misses
the frame, a fresh UAT ordinal/root may replay IDENTIFY after fresh detector
admission without rerunning the programmatic campaign; malformed machine proof,
identity/build drift, failed cleanup, or contradictory user observation stops
promotion for diagnosis. Do not infer pixels from software receipts.

Evidence/privacy: the detector and UAT roots are ignored mode 0700 with
mode-0600 files. The final public UAT projection may contain only aggregate
booleans, one request count, board category, the programmatic-evidence digest,
and redaction status. Never publish identities, origins, ports, addresses,
hostnames, credentials, boot sessions, generations, values, frame text,
response bodies, or traces.

Verification: The fresh-origin parser, unavailable/malformed command-status
boundaries, v2 origin-free intent, private typed admission receipt, and real
child-process CLI regression pass. Attempt-046 supplies the sealed exact-package
programmatic command-effects proof. Attempt-005 supplies one machine render and
clear receipt, exact-build and USB admission, one request, the user's durable
render-and-clear confirmation, a sealed v1 UAT projection, and passed
redaction. Ordered Cargo gates, the real ESP32-S3 firmware build, Bright Builds,
all Bazel targets, parity/progress, pinned-reference cleanliness, and diff
checks pass.

Completion review: Complete. Attempts 001 through 003 exposed and corrected
the delayed-origin, private-root ownership, and Bazel-relative-path contract
defects before any IDENTIFY effect. Attempt-004 reached a valid machine proof
but the user missed the display; no physical claim was inferred or published.
Attempt-005 reused the still-valid exact-package programmatic campaign after a
fresh detector admission, sent one IDENTIFY request, proved the correlated
machine render and clear receipts, and recorded the user's independent
confirmation that the physical frame rendered and cleared. API-009 is promoted
only from the two jointly validated, redaction-safe projections. Residual
non-claims are recorded in the immutable result.

### task-parity-thr001-emc2101-live-thermal | 2026-08-13 | Correct and prove Ultra 205 thermal readings

- [x] Correct the production Ultra 205 EMC2101 internal-temperature path to
      apply the pinned board-205 `+5 C` offset through a pure validated reducer.
- [x] Add the independent `bitaxe-emc2101-thermal-evidence-v1` contract and
      one repo-owned exact-package capture command with protected artifacts,
      typed failures, recovery, cleanup, and atomic evidence withholding.
- [x] Pass the focused, real-firmware, package, mandatory, privacy, reference,
      generated-contract, plan/task, and diff gates from the immutable plan.
- [x] Run exactly one detector-gated read-only `attempt-001` and promote only
      THR-001 if its complete live thermal quorum passes.
- [x] Replace the stale source-fragment admission with the checked-in
      production semantic boundary and add a regression that reads the actual
      source admitted by the evidence command.
- [x] Advance the closed evidence contract, validator, protected paths, and
      generated binding to attempt ordinal 2 without weakening any runtime,
      privacy, or evidence-withholding invariant.
- [x] Commit and push the fix, admit an exact clean package, and run at most one
      detector-gated read-only `attempt-002`; promote only on the full quorum.
- [x] Add a private Rust validator that parses acquisition-stamp members as
      exact `u64` values and proves equal fresh safe HTTP/WebSocket thermal
      inputs without emitting raw values.
- [x] Integrate the validator into the TypeScript shell, add wide/mismatched/
      malformed and real-child regressions, and advance the closed contract,
      generated binding, and protected paths to attempt ordinal 3.
- [x] Commit and push the fix, admit an exact clean package, and run at most one
      detector-gated read-only `attempt-003`; promote only on the full quorum.
- [x] Correct open-plan selection so valid terminal closures do not require
      immutable continuation links, and prove the real THR-001 selector state.

Plan: `docs/parity/work-plans/20260813T015631Z-THR-001/PLAN.md`.

Prior closed plans:
`docs/parity/work-plans/20260813T011207Z-THR-001/CLOSURE.md` and
`docs/parity/work-plans/20260813T001637Z-THR-001/CLOSURE.md`.

Dependencies and authorization: Standing task authorization covered only the
plan's exact two-command detector/capture sequence after its immutable plan and
complete software implementation were separately committed, pushed, clean,
and package-admitted. Attempt-003 is consumed and must not be retried. No
further hardware effect is authorized by this task without a new immutable
plan defining a bounded hardware-regression stimulus and recovery contract.

Evidence and privacy: `scratch/thr001-emc2101/wrapper-003` and
`scratch/thr001-emc2101/attempt-003` remain ignored protected roots. The only
public artifact is
`docs/parity/evidence/thr001-emc2101-thermal/thermal-projection.json`.
NeverPersistRaw values remain protected. Raw temperatures, acquisition stamps,
boot sessions, settings, hostnames, origins, ports, USB/network identifiers,
HTTP bodies, credentials, logs, commands, PIDs, private paths, and traces never
enter terminal or Git output.

Retry and promotion: Attempt-003 completed successfully as `hardware-smoke`,
but the authoritative final parity validator classifies THR-001 as an active
safety-control row and requires `hardware-regression` for `verified`. The
transition tool also forbids automatic demotion after a verified receipt, so
the uncommitted invalid receipt and derived progress files were discarded and
the authoritative checklist remained `implemented`. A future plan must define
safe, bounded overheat/fault stimulus, expected response, recovery, cleanup,
redaction, and a fresh attempt ordinal before promotion is eligible.

Verification: Immutable-plan, focused, mandatory, privacy, reference,
generated-contract, exact-package, detector, and protected-mode gates passed.
Attempt-003 completed and its public projection passed independent Rust
validation with the exact source/reference/package, live read-only thermal,
boot, safety, cleanup, mode, and redaction quorum. Final `just parity` correctly
rejected `verified` because the evidence cell lacked `hardware-regression`.

Completion review: The lossless host fix and read-only Ultra 205 evidence are
complete and retained, but verification is not claimed. THR-001 remains
`implemented`; this task remains active at a terminal `stop_impossible_contract`
boundary because its immutable plan prohibited the fault stimulus required by
the authoritative verifier. The next safe action is a distinct plan for
bounded thermal hardware-regression evidence, not a retry of attempt-003.

Injected-fault hardware-regression plan:
`docs/parity/work-plans/20260813T073353Z-THR-001/PLAN.md`.

- [x] Confirm the authoritative gap can be exercised without physical heating,
      fan/voltage/power changes, mining, raw I2C, or electrical manipulation.
- [x] Add a strict consume-before-use private intent and one-shot NVS stimulus
      tuple bound to board 205, exact package/plan, and attempt ordinal 4.
- [x] Add the bounded production-owner stimulus state machine and prove real
      EMC2101 reads, exactly five injected invalid outcomes, typed fault
      projection, ordered markers, fresh recovery, aborts, and no replay.
- [x] Add the private-first capture/restoration transaction and independently
      validated `bitaxe-emc2101-thermal-fault-evidence-v1` projection.
- [x] Pass all focused, real-firmware/package, mandatory, privacy, reference,
      generated-contract, process-boundary, task/plan, and diff gates; commit
      and push before effects.
- [x] Run exactly one detector-gated attempt-004; its hardware-regression quorum
      failed, promotion was withheld, and execution stopped without attempt-005.

Attempt-004 authorization: standing task authorization covers the single
fresh hardware regression only after the linked immutable plan and complete
implementation are separately committed, pushed, clean, exact-package-bound,
and detector-admitted. Allowed effects are the exact-package USB flash/reset,
private Wi-Fi plus consume-before-use one-shot NVS stimulus, five one-second
typed invalid-temperature overlays while real EMC2101 reads continue, ordinary
exact-package restoration, read-only same-origin API/WebSocket/log capture, and
cleanup. The linked plan freezes the expected fault, aborts, recovery,
restoration, evidence/privacy policy, terminal categories, and stop outcomes.
No physical heating, fan/voltage/frequency/power change, mining, pool input,
ASIC work, raw I2C/GPIO, public diagnostic setter, erase, OTA, rollback, power
cycle, direct UART, pin/pad/header manipulation, injected electrical signal,
attempt-005, or claim of physical overheat/open/short fault is authorized.

Attempt-004 closure: The detector admitted one Ultra 205 and the exact clean
package from implementation commit `8a1ddbd6`. The protected stimulus boot
reached `fault_observed` and then aborted with the closed reason
`fault_projection_missing`; the complete marker and recovery quorum did not
pass, so no candidate or public thermal-fault projection was published.
Ordinary exact-package restoration independently passed stable boot, fresh
safe HTTP/WebSocket thermal truth, disabled mining and hardware control,
cleanup, private modes, and redaction. The public failure adapter rendered the
internal `evidence_invalid` primary as `process_failed`; the missing typed-error
registration is fixed after the attempt with a regression, without changing
the consumed result.

Completion review: THR-001 remains `implemented`. Attempt-004 is exhausted and
must never be retried; attempt-005 is not authorized by this plan. See
`docs/parity/work-plans/20260813T073353Z-THR-001/CLOSURE.md`. A future
continuation must reproduce the production owner/reducer projection loss in a
software regression before defining any new bounded hardware ordinal.

Software diagnosis continuation:
`docs/parity/work-plans/20260815T181534Z-THR-001/PLAN.md`.

- [x] Reproduce attempt-004's exact `fault_projection_missing` category through
      the real stimulus, reducer, stale-processing, and next-sweep order in one
      fast deterministic software loop.
- [x] Rank and falsify multiple causes, apply the smallest root-cause fix, and
      preserve ordinary non-stimulus fault/fresh/stale semantics.
- [x] Pass focused, firmware, mandatory, privacy, reference, task/plan, and diff
      gates; commit and push with THR-001 still `implemented`.
- [x] Close the software-only plan and create a separate immutable attempt-005
      contract only after the correction is clean and pushed.

Authorization and stop: this continuation is software-only. It authorizes no
detector, package, USB, serial, HTTP, device, NVS, sensor, display, mining,
control, reset, OTA, erase, or attempt-005 effect. Stop if the production-order
loop cannot reproduce the consumed category or a fix would weaken ordinary
safety freshness.

Attempt-005 continuation:
`docs/parity/work-plans/20260815T182438Z-THR-001/PLAN.md`.

- [x] Advance the consumed ordinal, private roots, projection path, immutable
      plan binding, tests, and generated command contract to attempt 5.
- [x] Pass focused and mandatory software gates; commit and push the complete
      binding change before packaging or device admission.
- [x] Build the exact clean package, admit one Ultra 205, and run the bounded
      fault/restoration campaign exactly once.
- [x] Promote only on the complete independently validated hardware-regression
      quorum; otherwise withhold evidence and stop without attempt 6.

Authorization: only the exact attempt-005 command, effects, privacy, recovery,
cleanup, and stop contract in the linked immutable plan applies. No attempt-006
or effect outside that plan is authorized.

Attempt-005 closure: The sole campaign returned `evidence_invalid`. Device-side
logs reached `fault_observed` and `recovered` with no abort, but the early
baseline witness was absent and the host validator rejected canonical ESP-
prefixed production markers because its fixture and parser assumed bare lines.
Ordinary exact-package recovery, child cleanup, USB cleanup, and evidence
withholding passed. THR-001 remains `implemented`; attempt-005 is consumed and
attempt-006 is not authorized. See
`docs/parity/work-plans/20260815T182438Z-THR-001/CLOSURE.md`.

Marker-observation software continuation:
`docs/parity/work-plans/20260815T185700Z-THR-001/PLAN.md`.

- [x] Reproduce canonical ESP-prefix rejection and late baseline attachment in
      deterministic real-child tests at the production evidence seam.
- [x] Implement one strict shared payload parser and a bounded replayable or
      reader-armed ordered marker witness without weakening the quorum.
- [x] Pass focused and complete software gates, commit and push, and close with
      THR-001 still `implemented` and no attempt-006 authority.

Authorization: this continuation is software-only. No package, detector, USB,
serial, HTTP, device, NVS, sensor, control, reset, OTA, erase, or hardware
effect is authorized.

Completion review: The exact replay-origin contract is fixed and pushed at
`9fa31503`; focused and complete software gates pass. THR-001 remains
`implemented`, and hardware verification is still required. See
`docs/parity/work-plans/20260815T195949Z-THR-001/CLOSURE.md`. A distinct
immutable plan is required before attempt-007.

Attempt-007 continuation:
`docs/parity/work-plans/20260815T201754Z-THR-001/PLAN.md`.

- [x] Advance the consumed ordinal, protected roots, projection path,
      immutable plan binding, tests, generated command contract, and runfile to
      attempt 7.
- [x] Pass focused and mandatory software gates; commit and push the complete
      binding change before packaging, detection, or device access.
- [x] Build the exact clean package, admit one Ultra 205, and run the bounded
      fault/restoration campaign exactly once.
- [x] Promote only on the complete independently validated hardware-regression
      quorum; otherwise withhold evidence and stop without attempt 8.

Authorization: only the exact attempt-007 command, effects, privacy, recovery,
cleanup, and stop contract in the linked immutable plan applies after its
separate pushed implementation and software gates. No attempt-008 or effect
outside that plan is authorized.

Closure: The production marker envelope and late-attachment replay defects are
fixed and pushed at `6f637e87`; all required gates passed without hardware.
THR-001 remains `implemented`, attempt-005 remains consumed, and this plan does
not authorize attempt-006. See
`docs/parity/work-plans/20260815T185700Z-THR-001/CLOSURE.md`.

Attempt-006 continuation:
`docs/parity/work-plans/20260815T192115Z-THR-001/PLAN.md`.

- [x] Correct the selector's terminal-closure lineage reset with a production-
      shape regression while preserving strict checks for unclosed plans.
- [x] Advance the consumed ordinal, private roots, projection path, immutable
      plan binding, tests, generated command contract, and runfile to attempt 6.
- [x] Pass focused and mandatory software gates; commit and push the complete
      binding change before packaging, detection, or device access.
- [x] Build the exact clean package, admit one Ultra 205, and run the bounded
      fault/restoration campaign exactly once.
- [x] Promote only on the complete independently validated hardware-regression
      quorum; otherwise withhold evidence and stop without attempt 7.

Authorization: only the exact attempt-006 command, effects, privacy, recovery,
cleanup, and stop contract in the linked immutable plan applies after its
separate pushed implementation and software gates. No attempt-007 or effect
outside that plan is authorized.

Attempt-006 closure: The exact package and one admitted Ultra 205 completed the
sole campaign as `evidence_invalid`. Direct fault/recovery markers and eleven
complete retained replay triplets prove the device state machine succeeded, but
the strict host parser omitted the exact replay-producer tag
`bitaxe_firmware::boot_evidence`. Ordinary recovery, USB/process cleanup,
protected modes, and withholding passed. THR-001 remains `implemented`;
attempt-006 is consumed and attempt-007 is not authorized. See
`docs/parity/work-plans/20260815T192115Z-THR-001/CLOSURE.md`.

Replay-origin software continuation:
`docs/parity/work-plans/20260815T195949Z-THR-001/PLAN.md`.

- [x] Reproduce the exact direct/replay producer-tag mismatch through the real-
      child late-attachment seam.
- [x] Implement and prove a closed canonical replay-origin contract without
      admitting arbitrary tags or weakening marker order.
- [x] Pass focused and complete software gates, commit and push, and close with
      THR-001 still `implemented` and no attempt-007 authority.

Authorization: this continuation is software-only. No package, detector, USB,
serial, HTTP, device, NVS, sensor, control, reset, OTA, erase, or hardware
effect is authorized.

Completion review: Complete. The exact pushed attempt-007 package passed the
bounded typed EMC2101 invalid-sample hardware regression, independent evidence
validation, fresh ordinary restoration, no-replay, safe disabled state,
redaction, and holder cleanup. THR-001 is verified from the sealed projection
and immutable result; attempt-007 is consumed. Residual non-claims include
physical heating, electrical open/short behavior, calibration, loaded thermal
control, mining, other boards, and release readiness.

### task-parity-io002-adc-observation-attempt-004 | 2026-08-15 | Admit unique ADC provenance and verify observation

- [x] Bind `bitaxe-adc-observation-evidence-v1` to immutable plan
      `docs/parity/work-plans/20260815T232350Z-IO-002/PLAN.md`, protected
      `attempt-004` paths, and public schema ordinal 4.
- [x] Register the existing ADC evidence test module in the deployed Bazel test
      entrypoint, reproduce the attempt-003 provenance failure there, and
      replace the ambiguous bit-width token with exact unique initializer
      context plus missing, duplicate, and drift regressions.
- [x] Run the complete focused and mandatory software/firmware/privacy gates,
      commit and push the exact implementation, and rebuild its clean package
      before device access.
- [x] Run only the plan's exact detector command and, after successful one-
      device admission plus local Wi-Fi input availability, its exact one-shot
      `just capture-adc-observation-evidence ... --capture-timeout-seconds 360`
      attempt-004 command.
- [x] Promote only IO-002 on the complete exact-package passive disabled-state
      ADC/API quorum; otherwise withhold the projection, preserve
      `implemented`, record the earliest typed blocker and accepted stop
      outcome, and do not retry.

Plan: `docs/parity/work-plans/20260815T232350Z-IO-002/PLAN.md`

Objective and effects: fix the newly discriminating attempt-003 provenance and
test-registration boundaries, then verify the already accepted millivolt-domain
contract with one passive safe-state Ultra 205 observation. The sole run may
factory-flash/reset the exact clean package, seed private Wi-Fi, derive a same-
origin device only from protected current-session serial evidence, perform
read-only HTTP, WebSocket, and retained-log observations, and use at most one
exact-package recovery flash after a post-flash failure. It must keep mining and
hardware control disabled. Settings/restart requests, pool input, ASIC work,
voltage, frequency, fan or power control, raw ADC/GPIO/I2C, OTA, erase, fault
injection, physical power actions, direct UART, and every pin/pad/header/probe/
jumper/solder/signal manipulation are prohibited.

Evidence/privacy/recovery: wrapper and attempt directories are ignored
mode-`0700` roots with mode-`0600` files. Raw ADC values, stamps, logs, commands,
origins, ports, hostnames, USB/network/process identity, credentials, settings,
and traces remain private. Only the named aggregate projection may become
public after independent validation and redaction. Starting capture consumes
attempt-004; no unchanged retry or attempt-005 is authorized. Every post-flash
failure preserves the earliest category and runs bounded recovery and cleanup.
Detector failure or ambiguity, missing credentials, unsafe state, malformed or
incomplete proof, failed cleanup or recovery, privacy failure, nonzero command,
or recurrence of the corrected provenance signature stops with IO-002
`implemented` and evidence withheld.

Verification: complete. Clean pushed implementation
`166d1e9f3c4065946e6e3bb60398671bcdceab62`, one detector-admitted Ultra 205,
and the independently validated projection at
`docs/parity/evidence/io002-adc/adc-observation-projection.json` prove the
scoped ADC/API quorum. The projection binds attempt 4, exact source, reference,
package, plan and workflow identity, unique current source semantics, ADC unit
1/channel 1/GPIO 2, 12 dB attenuation, default resolution, curve calibration,
500 ms cadence, finite nonnegative integer-millivolt HTTP/WebSocket samples,
fresh and monotonic acquisition state, disabled mining and hardware control,
same boot session, exact correlation, cleanup without recovery, and passed
redaction. See
`docs/parity/work-plans/20260815T232350Z-IO-002/RESULT.md`.

Completion review: Complete. The omitted ADC test module and ambiguous upstream
breadcrumb were reproduced at the deployed boundary, fixed, and guarded by 337
automation tests including missing, duplicate, and drift regressions. Attempt-
004 consumed its sole ordinal and completed without retry. The archived-state
matrix also exposed a lifecycle-coupled test that expected a completed task to
remain active; it now uses an immutable fixture while production capture still
fails closed on inactive tasks. IO-002 is verified with
`unit,workflow,hardware-smoke` evidence. Energized-rail accuracy, external
calibration, induced failure, voltage actuation, load behavior, long-duration
drift, other boards, and release readiness remain explicit non-claims.

### task-parity-ui004-projection-continuation | 2026-08-16 | Publish the preserved UI workflow quorum

- [x] Commit and push immutable plan
      `docs/parity/work-plans/20260816T000806Z-UI-004/PLAN.md` before editing
      implementation source.
- [x] Add distinct attempt/projector source identities, exact unchanged
      UI/static-serving path admission, owner-only redirect regression, and
      prior plan/closure plus protected-artifact bindings.
- [x] Pass focused, real-child, generated-contract, static UI, firmware,
      mandatory, privacy, reference, immutable-plan, task-uniqueness and diff
      gates; commit and push before projection.
- [x] Run exactly one software-only `bitaxe-ui-workflow-evidence-v1`
      projection/validation transaction over the preserved attempt-001, with
      `umask 077` capture files and no detector, hardware or browser rerun.
- [x] Promote only UI-004 on the complete independently validated closed
      quorum; otherwise withhold evidence, preserve `implemented`, record the
      earliest typed failure, and stop without retry.

Plan: `docs/parity/work-plans/20260816T000806Z-UI-004/PLAN.md`

Authorization: repository source, tests, contracts, generated bindings,
documentation, and one protected software-only projector/validator transaction.
The repo-owned projector may read the preserved ignored attempt-001 operator
projection, browser attestation and digest-bound artifacts only to validate and
aggregate their closed facts. Their contents must not be printed, summarized,
copied into Git, or exposed. Captured source
`bf5b74f98cdb117ca5682b0118a61743db85856f` must remain ancestral and the ten
plan-listed UI/static-serving paths must be byte-unchanged and clean; current
projector source is recorded separately.

Exact transaction: only the plan's `umask 077; just
project-ui-workflow-evidence ...` command and one owner-only redirected
`just validate-ui-workflow-evidence ...` invocation after clean synchronized
pushed implementation. Starting the projector consumes the transaction. No
retry, detector, credentials, USB, device/network access, HTTP, browser,
flash/reset/restart, hardware recovery, mining, settings/theme mutation, OTA,
display/input claim, direct UART, pin/pad/header/GPIO interaction, or hardware
control is authorized.

Evidence/privacy/stops: private roots and files stay mode `0700`/`0600`; only
`docs/parity/evidence/ui004-live-workflows/ui-workflow-projection.json` may be
published after independent validation. Origins, addresses, ports, hostnames,
identities, page values, bodies, frames, screenshots, traces, credentials and
private paths must not enter Git or terminal summaries. Any source, digest,
schema, mode, privacy, validator, cleanup or quorum failure withholds evidence,
keeps UI-004 `implemented`, and closes this task without another transaction.

Verification: `cargo fmt --all`, `cargo clippy --all-targets --all-features --
-D warnings`, `cargo build --all-targets --all-features`, `cargo test
--all-features`, the forced uncached focused Bazel projector suite, Bright
Builds checks, `just test`, `just parity`, `just parity-progress`, `just
verify-redaction`, `just verify-reference`, and `just build` pass. The current,
prior-plan, and prior-closure SHA-256 bindings match; the captured commit is
ancestral; and all ten compatibility paths are unchanged and clean.

Completion review: Complete. The sole software-only projector and independent
validator exited zero without a detector, device, network, browser, credential,
or hardware rerun. The public projection binds captured source
`bf5b74f98cdb117ca5682b0118a61743db85856f`, clean synchronized projector source
`19d8f99fd5969c87d9a55b0fefa9558875e9f0fd`, exact package/reference identity,
the prior plan/closure and current plan, unchanged served-UI sources, the closed
desktop/mobile workflow quorum, disabled mining/control, cleanup, protected
modes, independent validation, and passed redaction. UI-004 is verified with
`unit,workflow,static-route,hardware-smoke` evidence. Physical panel/input,
mutation during this continuation, upload, OTAWWW, mining, other-board and
release claims remain excluded. See
`docs/parity/work-plans/20260816T000806Z-UI-004/RESULT.md`.

### task-parity-ui001-display-behavior | 2026-08-04 | Complete Ultra 205 display driver behavior

- [x] Add the pure exact-panel, rotation, inversion, timeout, wake/priority,
      and edge-triggered power contract with boundary regressions.
- [x] Load confirmed display settings with upstream defaults and fail closed on
      malformed or unsupported stored values.
- [x] Retain one configured firmware display owner across runtime frames and
      prove configuration/render/power ordering plus sensor-failure isolation.

Plan: `docs/parity/work-plans/20260804T230000Z-UI-001/PLAN.md`

Authorization: local software, synthetic display/settings fixtures, and build
work only. No hardware attempt, credentials, external service, mining, pool
connection, frequency/voltage/fan/power effect, OTA, recovery, direct UART,
pins, or physical button interaction.

Verification: Focused core/config tests, display adapter and ownership tests,
the canonical firmware build, the mandatory Rust sequence, Bright Builds,
`just test`, parity/progress, redaction, reference cleanliness, immutable-plan,
and diff checks pass on the implementation tree.

Completion review: The exact panel settings and runtime power behavior are
software-implemented with closed configuration handling and one logical owner.
UI-002 carousel content, UI-003 physical input, live panel
orientation/inversion/timeout, and operator-visible behavior remain below
verified, so this implemented task remains active rather than archived.

Source-bound verification plan:
`docs/parity/work-plans/20260816T064239Z-UI-001/PLAN.md`.

- [x] Validate the exact committed API-009 display-UAT and programmatic
      projections without reading protected attempts or rerunning hardware.
- [x] Add a closed UI-001 evidence contract, independent validator, projector,
      exhaustive boundary regressions, and complete Bazel/runfiles ownership.
- [x] Bind captured source `522d5abd`, current pushed display semantics, pinned
      reference behavior, this active task, and the immutable plan into one
      aggregate-only public projection.
- [x] Pass every focused, mandatory, privacy, package, reference, parity,
      progress, selector, digest, permission, sensitive-value, and diff gate;
      promote and archive only on the complete accepted quorum.

Authorization: committed public evidence, repository source/Git history,
deterministic tests, documentation, checklist tooling, and local builds only.
Do not read credentials or protected attempt artifacts and do not access the
detector, USB/serial, device/network/HTTP, physical display, operator
checkpoint, mining, settings mutation, restart, OTA, recovery, hardware
control, external UART/BAP, or any pin/pad/header/GPIO/probe/jumper/solder/
signal interface. No new hardware attempt is authorized or required.

Promotion requires the exact sealed board-205 UAT to prove one machine and
operator-confirmed IDENTIFY render and natural clear, its exact programmatic
projection to prove package/reference identity, safe stop, cleanup, and
redaction, and current source checks to prove the display-owned paths remain
compatible with that captured package. The resulting UI-001 projection must be
independently validated, aggregate-only, and state `hardware_rerun_used: false`.
Physical geometry, brightness, every rotation/inversion setting on hardware,
timeout duration/current draw, physical input, UI-002 content, other boards,
mining, soak, update/recovery, and release readiness remain non-claims.

Completion review: Complete. The sealed projection joins the exact committed
API-009 physical display UAT to the unchanged current display implementation
and pinned reference semantics. Independent validation, redaction, source and
task binding, the mandatory Rust sequence, Bright Builds, all 45 Bazel tests,
parity/progress, packaging, reference cleanliness, and diff checks passed.
UI-001 is verified with `unit,workflow,hardware-smoke` evidence and no new
hardware attempt. Physical geometry, brightness, every setting on hardware,
input, UI-002 content, other boards, mining, update/recovery, and release
readiness remain explicit non-claims. See
`docs/parity/work-plans/20260816T064239Z-UI-001/RESULT.md`.

### task-parity-ui002-screen-flow | 2026-08-04 | Implement bounded Ultra 205 screen flow

- [x] Add the pure priority, overlay, intro, carousel, notification, and
      four-line frame contract with exact timing and privacy regressions.
- [x] Project existing firmware runtime facts without operator-publication,
      statistics-drain, retained-log, mining-state, or credential side effects.
- [x] Retain one screen owner beside the display owner, use the absolute 500 ms
      cadence, redraw only changed frames, and preserve sensor isolation.

Plan: `docs/parity/work-plans/20260805T001000Z-UI-002/PLAN.md`

Authorization: local software, synthetic screen/runtime fixtures, and build
work only. No hardware attempt, credentials, external service, mining, pool
connection, frequency/voltage/fan/power effect, OTA, recovery, direct UART,
pins, or physical button interaction.

Verification: Twelve focused pure screen-flow tests, both firmware display
adapter/source-ownership targets, and the real ESP-IDF firmware Bazel target
pass. The ordered full Rust sequence, Bright Builds checks, all 34 Bazel test
targets, parity validation/progress, redaction, reference cleanliness,
immutable-plan, and diff checks also pass.

Completion review: Implementation commit
`9b2f37945b34a0e9fece56c8aa90703afda3ac63` and the commit-bound `RESULT.md`
support the typed `implemented` transition with `unit,workflow` evidence. The
task remains active because UI-002 is below `verified`; UI-003 physical input,
live screen content, animation/bitmap parity, mining, and hardware-control
behavior remain separate evidence gaps.

Source-bound verification plan:
`docs/parity/work-plans/20260816T073911Z-UI-002/PLAN.md`.

- [x] Validate the exact committed API-009 display-UAT and command-effects
      projections without reading protected attempts or rerunning hardware.
- [x] Add a closed UI-002 evidence contract, independent validator, projector,
      exhaustive boundary regressions, and complete Bazel/runfiles ownership.
- [x] Bind captured source `522d5abd`, current pushed screen-flow semantics,
      pinned reference behavior, this active task, and the immutable plan into
      one aggregate-only public projection.
- [x] Pass every focused, mandatory, privacy, package, reference, parity,
      progress, selector, digest, permission, sensitive-value, and diff gate;
      promote and archive only on the complete accepted quorum.

Authorization: committed public evidence, repository source/Git history,
deterministic tests, documentation, checklist tooling, and local builds only.
Do not read credentials or protected attempt artifacts and do not access the
detector, USB/serial, device/network/HTTP, physical display, browser, operator
checkpoint, mining, settings mutation, restart, OTA, recovery, hardware
control, external UART/BAP, or any pin/pad/header/GPIO/probe/jumper/solder/
signal interface. No new hardware attempt or human checkpoint is authorized or
required.

Promotion requires the exact sealed board-205 UAT to prove one machine- and
operator-confirmed IDENTIFY render and natural clear, its exact programmatic
projection to prove package/reference identity, safe stop, cleanup, and
redaction, and current source checks to prove the full screen-flow paths remain
compatible with that captured package and pinned reference. The resulting
UI-002 projection must be independently validated, aggregate-only, and state
`hardware_rerun_used: false`. Physical proof of every page, dwell, notification,
new-block state, input path, animation/bitmap/QR detail, pixel geometry,
brightness, other boards, mining, soak, update/recovery, and release readiness
remain non-claims.

Completion review: Complete. The sealed projection joins the exact committed
API-009 physical IDENTIFY UAT to clean current screen-flow source and pinned
reference semantics. Independent validation, redaction, source and task
binding, the mandatory Rust sequence, Bright Builds, all 45 Bazel tests,
parity/progress, packaging, reference cleanliness, and diff checks passed.
UI-002 is verified with `unit,workflow,hardware-smoke` evidence and no new
hardware attempt. Physical observation of every page, dwell, notification,
input path, graphical detail, other boards, mining, update/recovery, and
release readiness remain explicit non-claims. See
`docs/parity/work-plans/20260816T073911Z-UI-002/RESULT.md`.

### task-parity-pwr006-legacy-wire-units | 2026-08-16 | Correct legacy INA260 API units

- [x] Convert SI-typed input voltage and current to upstream-compatible
      millivolts and milliamps only at the legacy API and statistics boundaries.
- [x] Rename ambiguous Rust wire members and preserve campaign safety checks in
      their explicit physical domains.
- [x] Bind the correction to the pinned INA260, system-info, statistics, and
      AxeOS conversion paths with behavior-focused regressions.
- [x] Re-evaluate the sealed PWR-006 evidence without inventing new hardware
      observations, run all required gates, and restore only the evidence level
      actually supported.

Plan: `docs/parity/work-plans/20260816T082924Z-PWR-006/PLAN.md`

Authorization: repository edits, tests, and read-only reuse of committed
evidence only. No flash, USB/serial/network access, credentials, mining,
voltage/frequency/fan/power actuation, OTA, erase, fault injection, direct UART,
pins, or physical manipulation. A fresh hardware run requires a separate exact
task contract.

Verification: focused API, campaign-safety, PWR-006 contract, INA260 automation,
reference, redaction, immutable-plan, generated-contract, and diff checks; the
ordered Rust sequence; Bright Builds; `just test`; `just parity`; and
`just parity-progress`.

Completion review: Complete. The legacy API and statistics boundaries now
serialize input voltage/current in upstream-compatible millivolts/milliamps
while internal safety remains in volts/amps and power remains in watts. The v2
projection binds the accepted read-only Ultra 205 capture to its historical
source semantics, current corrected source, both immutable plans, and pinned
reference/UI conversions without a device rerun or raw-value publication. The
independent validator, redaction, ordered Rust sequence, Bright Builds, all 45
Bazel tests, packaging, parity/progress, reference cleanliness, and diff checks
passed. PWR-006 remains verified; external-meter accuracy, calibration, load
behavior, hardware control, other boards, mining, and release readiness remain
non-claims. See
`docs/parity/work-plans/20260816T082924Z-PWR-006/RESULT.md`.

### task-parity-ui003-boot-button | 2026-08-04 | Implement bounded Ultra 205 boot-button input

- [x] Add a pure active-low debounce and exact 2,000 ms short/long press
      classifier with bounce, regression, and one-shot long-press coverage.
- [x] Retain one GPIO0 input owner and route normal short clicks to identify
      cancellation or screen advance, normal long presses to configuration-AP
      toggle, and self-test long presses to an explicit unavailable boundary.
- [x] Wake the display on admitted short input, preserve display/sensor/Wi-Fi
      failure isolation, and expose only redaction-safe input status categories.

Plan: `docs/parity/work-plans/20260805T020000Z-UI-003/PLAN.md`

Authorization: local software, synthetic input/runtime fixtures, and build
work only. No hardware attempt, physical button press, credentials, external
service, mining, pool connection, frequency/voltage/fan/power effect, OTA,
recovery, direct UART, pins, or physical electrical manipulation.

Verification: Six focused pure input tests, fourteen screen-flow tests, both
firmware display/Wi-Fi source-ownership targets, and the real ESP32-S3 firmware
target pass. The mandatory ordered Rust sequence, Bright Builds checks, all 34
Bazel test targets, parity validation/progress, redaction, reference
cleanliness, immutable-plan, sensitive-log, and diff checks also pass.

Completion review: The bounded active-low classifier, retained GPIO0 pull-up
owner, atomic identify cancellation, manual screen advance/display wake, and
typed configuration-AP toggle are software-implemented with closed failure
categories. Physical button observation, exact LVGL event timing, self-test
cancellation, live configuration-AP toggling, and all hardware behavior remain
below verified, so this implemented task remains active rather than archived.

Verification continuation (2026-08-16):

- [x] Add a typed integrated exact-package input UAT with a fresh protected
      private root, transcript-free serial reducer, durable live checkpoint,
      aggregate public projection, independent validator, and focused tests.
- [ ] Commit and push the implementation, run the sole authorized
      `attempt-001`, and admit exactly one post-checkpoint physical short click
      routed to screen advance.
- [ ] Run every mandatory gate, promote only UI-003 if the complete quorum
      validates, then record the result and archive this task atomically.

Continuation plan:
`docs/parity/work-plans/20260816T093555Z-UI-003/PLAN.md`

Continuation authorization: the original software-only authorization remains
historical for the implementation phase. After the continuation implementation
is clean, verified, committed, and pushed, standing authorization permits the
three exact commands in the continuation plan: package, detector, and one
integrated exact-package `input-uat` attempt at
`scratch/ui003-input/attempt-001`. The sole human effect is one brief press and
release of the provided BOOT button after a live ready checkpoint. Long press,
configuration-AP toggle, self-test, credentials, network access, mining,
voltage/frequency/fan/thermal/power/ASIC control, OTA, recovery, direct UART,
pins, and physical electrical manipulation remain prohibited.

Continuation evidence, recovery, retry, and stop: the public projection is
aggregate-only and must exclude serial text, port/USB/network/process identity,
credentials, private paths, and device-private values. `Ctrl-C` or refusal
releases USB ownership and withholds positive evidence. `attempt-001` is the
only authorized effectful run; any retry requires verified new information and
a new immutable plan. Stop on detector, identity, package, flash, startup,
input-owner, checkpoint, marker, cleanup, validator, redaction, or projection
failure, or on successful verified projection.

Attempt-001 review (2026-08-16): The clean exact package was built and one
detector-admitted Ultra 205 was flashed, but the live workflow stopped before
the checkpoint with `runtime_attestation_invalid`; no BOOT press occurred and
no public projection was written. The root cause was an unframed arbitrary USB
chunk boundary splitting a runtime-attestation marker. The reducer now retains
a bounded partial line and focused Cargo/Bazel tests prove split-marker
recovery. The immutable continuation authorized only `attempt-001`, so UI-003
remains implemented and this task remains active pending a fresh plan and
attempt ordinal. See the continuation `CLOSURE.md`.

Attempt-002 verification plan:
`docs/parity/work-plans/20260816T102741Z-UI-003/PLAN.md`

- [x] Rebind the typed input UAT and protected root to fresh attempt-002 while
      preserving the public projection and fixed GPIO/timing semantics.
- [x] Preserve bounded incremental serial framing and add a closed,
      redaction-safe runtime-attestation failure discriminator with focused
      pure and integrated regressions.
- [x] Pass every focused and mandatory gate, commit and push the exact source,
      build its package, run only attempt-002, and promote UI-003 only if one
      post-checkpoint physical short click validates completely.

Attempt-002 authorization: pushed commit `f713c086` is verified new information
that fixes attempt-001's exact split-line boundary. After this immutable plan
and its rebound implementation are clean, fully gated, committed, and pushed,
the plan authorizes one package build, one detector, one exact-package factory
flash and receive-only observation, and one human press-and-release of the
provided BOOT button lasting less than two seconds after the live ready
checkpoint. No human-response deadline applies.

Attempt-002 evidence, recovery, retry, and non-scope: use only fresh ignored
mode-0700 `scratch/ui003-input/attempt-002` with mode-0600 private files and the
existing aggregate-only public projection path. Record exact source/reference,
package, detector command and one-device board-info success, UAT command,
cleanup, and closed outcome without committing port/USB/process/network/private
identity or serial text. Interruption/refusal releases USB ownership and
withholds evidence. Starting the UAT consumes attempt-002; no unchanged retry
or later ordinal is authorized. Long press, configuration AP, self-test,
credentials, network access, mining, controls, updates, recovery writes,
external UART, physical power action, and electrical pin/pad/header/probe work
remain prohibited. Any incomplete boundary leaves UI-003 `implemented`.

Attempt-002 review (2026-08-16): The exact clean pushed package and sole
detector/UAT attempt passed. After trusted repeated runtime attestation and the
durable live checkpoint, one brief BOOT press produced exactly one production
short-click screen advance with no long press. Cleanup completed and the
independent validator accepted the aggregate-only redacted projection. The
hardware quorum is complete; only the audited evidence commit, UI-003
transition, progress synchronization, task archive, and final gates remain.

Metadata correction review: The complete physical evidence and pushed
evidence commit remain valid, but the first uncommitted transition draft used
only the plan's `hardware-smoke` label. The final parity validator correctly
requires `hardware-regression` for active safety-control rows, including
UI-003. All generated transition, progress, README, and archive state was
reverted before commit. The bounded production input UAT qualifies as a
runtime-display-input hardware regression; replay with both labels, then
archive only after every validator passes. No hardware retry is required.

Completion review: Complete. The corrected transition retains the immutable
plan's `hardware-smoke` label and adds the safety policy's required
`hardware-regression` label for the same bounded exact-package input exercise.
`just parity` now reports no validation errors, progress is synchronized at 74
of 94 active rows verified (78.7%), and UI-003 alone is `verified`. The
validator-accepted projection proves one post-checkpoint physical short click
through the production GPIO0 owner to screen advance, with prohibited
duplicate/unexpected/long-press outcomes, cleanup, disabled mining/control,
no transcript, and redaction. Physical long press, configuration AP,
self-test, other boards, mining, controls, OTA, recovery, UART/BAP, and
electrical work remain non-claims. See attempt-002 `RESULT.md`.

### task-parity-stat002-statistics-history | 2026-08-04 | Implement production statistics history

- [x] Add the exact bounded 720-sample history, timestamp admission, configured
      retention decision, zero-frequency clearing, and focused regressions.
- [x] Start one absolute-cadence firmware producer that records confirmed
      runtime snapshots independently of HTTP request timing.
- [x] Return the complete owned history through the existing API projection,
      prove sole ownership and request-time immutability, and run every gate.

Plan: `docs/parity/work-plans/20260804T211000Z-STAT-002/PLAN.md`

Verification continuation:
`docs/parity/work-plans/20260816T204646Z-STAT-002/PLAN.md`

Authorization: local software, synthetic snapshots, and build work only. No
hardware attempt, credentials, external service, mining campaign, pool
connection, frequency/voltage/fan/power effect, OTA, recovery, direct UART, or
pins.

Verification: The focused Cargo and Bazel tests, real firmware build, ordered
Rust format/Clippy/build/test sequence, Bright Builds checks, all 32 Bazel test
targets, parity validation/progress, redaction, reference cleanliness, and diff
checks passed on the implementation tree.

Completion review: Implementation commit
`35f8bb676b91bdb702dd9026cb0379f5b12e45e6` and typed transition
`20260804T215500Z-STAT-002` establish `implemented` with
`unit,workflow,api-compare` evidence. The task remains active because live
cadence, telemetry accuracy, long-duration retention, device API, and browser
behavior remain below verified.

- [x] Add and verify a typed private-first statistics-history hardware evidence
      workflow with exact-package identity and protected publication.
- [ ] Execute the single detector-gated mining-disabled attempt-001, restore the
      exact original `statsFrequency`, and independently validate its projection.
- [ ] Promote only on the complete live cadence/API quorum; otherwise record the
      earliest blocker and next safe action without a second attempt.

Continuation authorization: one exact-package board-205 factory flash, normal
USB reset/re-enumeration, ignored Wi-Fi credential seeding, one current-session
same-origin API transaction, and a temporary one-field `statsFrequency`
mutation are authorized only under the continuation plan. Exact restoration is
mandatory, with one bounded same-package recovery flash followed by restoration
PATCH/readback if the original origin is lost. No mining, pool, ASIC work,
frequency/voltage/fan/thermal/power control, OTA, erase, fault injection,
browser, direct UART, pins, physical electrical action, retry or attempt-002 is
authorized.

Attempt-001 closure: exact clean source
`01e48e12f7b063f923fdfc589c129448cb559064` passed every software/package gate
and the detector admitted exactly one Ultra 205. The capture failed closed with
typed `timeout` at `initial_flash_monitor`: the exact flash effect completed,
but the supervisor and child capture shared the same 360-second boundary. No
origin/API transaction or `statsFrequency` mutation occurred, so restoration
and recovery were not required. The projection/candidate remain absent,
protected modes and cleanup pass, and no checklist transition or progress sync
is authorized. Closure:
`docs/parity/work-plans/20260816T204646Z-STAT-002/CLOSURE.md`.

Next safe action: create a fresh immutable attempt-002 plan only after a
verified source correction gives the child flash-monitor sole timeout ownership
or a strictly larger bounded supervisor cleanup grace, with a real-child
boundary regression. Attempt-001 must not be retried.

Attempt-002 correction and verification plan:
`docs/parity/work-plans/20260816T213710Z-STAT-002/PLAN.md`

- [x] Give the 360-second flash-monitor child a strictly later bounded
      supervisor deadline with an explicit 60-second cleanup/result grace,
      covering both initial and recovery children.
- [x] Prove the corrected boundary with exact arithmetic, failure-preservation,
      and a real spawned child that completes its own timeout cleanup before
      the supervisor can terminate it.
- [x] Rebind every closed task/plan/path/invocation source to fresh attempt-002,
      pass all software/package/privacy gates, and push the exact source before
      device access.
- [x] Run only the plan's detector and single conditional attempt-002; promote
      only on the independently validated cadence/API/restoration quorum.

Attempt-002 authorization: one exact clean pushed board-205 factory package,
normal USB reset/re-enumeration, ignored Wi-Fi seeding, one current-session
same-origin API transaction, and one temporary `statsFrequency`-only mutation.
The child capture remains 360 seconds and its supervisor has exactly 60 seconds
of additional bounded cleanup/result grace. Exact original-setting restoration
is mandatory; after mutation only, one same-package recovery flash plus
restoration PATCH/readback is allowed if the admitted origin is lost.

Attempt-002 privacy and safety: use only the fresh mode-0700 wrapper
`scratch/stat002-statistics-history/wrapper-002`, distinct mode-0600 redirects,
the previously absent supervisor-owned mode-0700 child
`scratch/stat002-statistics-history/attempt-002`, and the closed aggregate-only
projection. Credentials, settings values, raw statistics, origins, endpoints,
hostnames, ports, USB/network/process identity, bodies, logs, commands, PIDs,
and traces remain private. Statistics voltage/current remain millivolt/
milliamp legacy wire fields and are not accuracy evidence.

Attempt-002 prohibitions and stops: no pool, mining, ASIC work, arbitrary
frequency/voltage/fan/thermal/power control, OTA, erase, fault injection,
physical power action, browser, direct UART, or electrical pin/pad/header/GPIO/
probe/jumper/solder/signal work. Starting capture consumes attempt-002; never
reuse attempt-001, retry attempt-002, or start attempt-003. Stop on detector or
identity failure, missing input, drift, pre-effect failure, incomplete quorum,
failed restoration/recovery/cleanup/mode/privacy, nonzero command exit, or
successful projection.

Attempt-002 closure: exact pushed source
`265be8c99881be035cc54801d6aab5f4d936065d`, pinned reference, package,
software/privacy gates, and one-device detector passed. The corrected
420-second supervisor outlived the 360-second monitor boundary. The protected
child record proves completed exact-package flash and trusted full-duration
runtime attestation, but factory flash/NVS/USB setup plus monitor capture used
the whole supervisor lifetime before effect-result delivery. The public result
failed closed as `timeout` at `initial_flash_monitor`; no origin/API request or
`statsFrequency` mutation occurred, so restoration/recovery were unnecessary.
Projection/candidate absence, owner-only modes, Git synchronization, and USB/
tool cleanup pass. No checklist transition or progress sync is authorized.
Closure: `docs/parity/work-plans/20260816T213710Z-STAT-002/CLOSURE.md`.

Next safe action: a fresh immutable attempt-003 plan requires a verified whole-
operation timeout correction. Prefer the existing bounded 900-second process-
adapter lifetime as owner while the child retains its 360-second monitor
timeout, or derive a complete bound that separately includes pre-monitor flash/
setup, monitor, post-monitor evidence/effect delivery, and cleanup. Add a scaled
real-child regression whose pre-monitor plus capture duration exceeds the old
420-second policy. Never reuse attempts 001 or 002.

Attempt-003 correction and verification plan:
`docs/parity/work-plans/20260816T221106Z-STAT-002/PLAN.md`

- [x] Remove the partial 420-second override so the existing bounded 900-second
      process adapter owns the entire flash/NVS/gates/monitor/result lifecycle.
- [x] Prove source ownership and a scaled real child with pre-monitor, capture,
      and post-monitor phases that exceeds the old boundary and completes under
      the whole-operation owner; preserve typed timeout failure behavior.
- [x] Rebind the closed task/plan/path/invocation surface to fresh attempt-003,
      pass every software/package/privacy gate, and push the exact source.
- [x] Run only the plan's detector and conditional attempt-003; promote only on
      the independently validated cadence/API/restoration quorum.

Attempt-003 authorization: one exact clean pushed board-205 factory package,
normal USB reset/re-enumeration, ignored Wi-Fi seed, one current-session same-
origin API transaction, and a temporary `statsFrequency`-only mutation. The
child owns its 360-second monitor timeout; the existing 900-second adapter bound
owns the whole child lifecycle. Exact restoration is mandatory. Only after a
mutation, one same-package recovery flash plus restoration PATCH/readback is
allowed if the admitted origin is lost.

Attempt-003 privacy/safety: use only fresh mode-0700 wrapper
`scratch/stat002-statistics-history/wrapper-003`, distinct mode-0600 redirects,
the absent supervisor-owned mode-0700 child
`scratch/stat002-statistics-history/attempt-003`, and the closed aggregate-only
projection. Credentials, settings values, raw statistics, origins, endpoints,
hostnames, ports, USB/network/process identity, bodies, logs, commands, PIDs,
and traces remain private. Legacy statistics voltage/current are millivolts/
milliamps and are not accuracy evidence.

Attempt-003 prohibitions/stops: no pool, mining, ASIC work, arbitrary hardware
control, OTA, erase, fault injection, physical power action, browser, direct
UART, or electrical pin/pad/header/GPIO/probe/jumper/solder/signal work. Starting
capture consumes attempt-003; never reuse attempts 001/002, retry attempt-003,
or start attempt-004. Stop on detector/identity failure, missing input, drift,
pre-effect failure, incomplete quorum, failed restoration/recovery/cleanup/
mode/privacy, nonzero exit, or successful projection.

Completion review (2026-08-16): Complete. Exact pushed implementation commit
`0fe0c9aa81e3b604b6262c22f74a5e657b28596b` removed the partial 420-second
override and the single attempt-003 completed beyond that former boundary under
the bounded 900-second whole-operation owner. The independently validated,
aggregate-only projection proves one detector-admitted Ultra 205, passive safe
state, one `statsFrequency`-only mutation, confirmed readback, four finite
exact-width samples with three exact 1,000-millisecond intervals, immediate
repeat stability, later producer growth, exact restoration, cleanup, owner-only
modes, and redaction. Typed transition `20260816T221106Z-STAT-002` promotes only
`STAT-002` to `verified`; synchronized progress is 75 of 94 active rows (79.8%).
See attempt-003 `RESULT.md`.

Residual risks: Physical telemetry accuracy, browser charts, live full-horizon
retention, mining, ASIC work, hardware controls, updates, recovery behavior,
other boards, and release readiness remain non-claims. Legacy statistics
`voltage` and `current` wire fields remain millivolts and milliamps.

### task-parity-rel003-large-erase-recovery | 2026-08-17 | Verify release-image large-erase recovery

- [x] Add a plan-bound repo-owned large-erase command and typed private-first
      release-recovery evidence workflow with exact package admission,
      supervised USB ownership, safe restore, recovery precedence, independent
      validation, and real-process regressions.
- [x] Run every focused and mandatory software, firmware, package, release-
      gate, privacy, reference, immutable-plan, and exact-source gate; commit
      and push before device access.
- [x] Run only the frozen detector and sole conditional attempt-001 capture;
      jointly admit prior accepted release/rollback evidence and promote
      REL-003 only if complete large-erase restoration proof passes.

Plan: `docs/parity/work-plans/20260817T005227Z-REL-003/PLAN.md`.

Authorization and exact effects: after the plan and implementation are clean,
fully gated, committed, pushed, and repackaged from exact source, run only the
plan's one protected detector and one conditional
`just capture-release-recovery-evidence` attempt-001 command. The capture may
erase the complete flash of the sole detector-admitted Ultra 205 exactly once,
then restore the exact factory package plus an opaque owner Wi-Fi NVS seed with
`mineonboot=false` and monitor the restored passive safe state. Large erase
intentionally removes all onboard NVS, OTA state, applications, static assets,
coredump data, pool settings, hostname, theme, and operator tuning. Factory
content and Wi-Fi connectivity are restored; all other settings return to
package defaults. Ignored local Wi-Fi/pool files remain untouched and
recoverable, but pool values are not reseeded and mining stays disabled.

Recovery, safety, retry, and prohibited effects: if the primary restore fails
before flash transfer completes, one recovery-only exact factory flash with the
same Wi-Fi seed is allowed. If transfer completes but runtime proof is absent,
do not reflash unchanged. Preserve the earliest failure through cleanup and
release every owned USB/process resource. Attempt-001 is consumed when capture
starts; never reuse it or erase again under this task. No OTAWWW, interrupted
power, eFuse action, arbitrary raw write, mining, pool connection, voltage/
frequency/fan/thermal/power control, physical power action, direct UART, or
pin/pad/header/GPIO/probe/jumper/solder/signal manipulation is authorized.

Evidence and promotion: use only fresh ignored mode-`0700`
`scratch/rel003-large-erase/wrapper-001` and `attempt-001` roots with mode-
`0600` descendants. Wi-Fi contents, device/USB/network/process identities,
origins, hostnames, settings, commands, logs, PIDs, traces, and raw firmware
bytes remain private. The public projection may contain only public
provenance, cryptographic identities, closed categories, bounded counts, and
safe booleans. REL-003 promotion additionally requires the already accepted
Phase 18/19 release evidence and verified REL-002 interrupted-update/rollback
result. Any incomplete fact withholds promotion and leaves REL-003
`implemented`.

Completion review: implementation commit
`70493a51249df2f82eb5b046be7dc95b137c7e97` fixed canonical package artifact
paths and added the task-bound typed recovery workflow. The exact clean package
passed the release gate. The sole detector and attempt-001 then completed one
full erase, exact factory and owner-Wi-Fi/default-NVS restore, trusted exact-
package runtime proof, SPIFFS readiness, `mineonboot=false`, passive safe
state, cleanup, protected modes, independent validation, and redaction without
a recovery reflash. The accepted Phase 18/19 and verified REL-002 artifacts
supply the release-gate, provenance, package workflow, failed/interrupted
update, rollback, and recovery terms; the new projection supplies the former
large-erase gap. See
`docs/parity/work-plans/20260817T005227Z-REL-003/RESULT.md`.

Residual risks: onboard settings were intentionally reset to owner Wi-Fi plus
package defaults; pool values remain local and were not reseeded. OTAWWW,
power-loss interruption, eFuse anti-rollback, repeated erase, release signing,
factory provisioning scale, other boards, mining, controls, direct UART, and
electrical work remain separate non-claims.

### task-parity-stat001-hashrate-monitor | 2026-08-04 | Implement the hashrate monitor

- [x] Add exact bounded counter conversion, reset behavior, error percentage,
      and hierarchical 1-minute, 10-minute, and 1-hour averages.
- [x] Carry parsed register values through the sole production owner and admit
      passive reads only while its ASIC session is already active.
- [x] Publish all hashrate windows through the existing runtime/API projection,
      add focused ownership regressions, and run every mandatory gate.

Plan: `docs/parity/work-plans/20260804T200000Z-STAT-001/PLAN.md`

Authorization: local software, synthetic register observations, and build work
only. No hardware attempt, credentials, external service, mining campaign,
pool connection, frequency/voltage/fan/power effect, OTA, recovery, direct UART,
or pins.

Verification: The full Rust sequence, Bright Builds checks, all 31 Bazel tests,
the real firmware build, parity validation/progress, redaction, reference
cleanliness, and diff checks passed on the implementation tree.

Completion review: Implementation commit
`e0c3b1e9043e033b24135b31a1293bf22afe8759` and typed transition
`20260804T205500Z-STAT-001` establish `implemented` with `unit,workflow`
evidence. The task remains active because live BM1366 counter accuracy and
hardware/API/UI behavior remain below verified.

Verification-promotion plan:
`docs/parity/work-plans/20260816T005443Z-STAT-001/PLAN.md`

- [x] Add a private-first closed hashrate quorum to the existing conservative
      600-second campaign, with independent validation and no raw-value
      publication.
- [x] Run the plan's full pre-hardware suite, commit and push the implementation,
      then execute only detector command 1 and the sole attempt-001 command 2.
- [ ] Promote only STAT-001 on the complete exact-package HTTP/WebSocket,
      BM1366 topology, rolling-window, terminal-zero, safe-stop, cleanup, mode,
      seal, and redaction quorum; otherwise preserve `implemented`, withhold the
      projection, record the earliest typed blocker, and do not retry.

Attempt-001 authorization: one exact clean board-205 package may be factory-
flashed and normally reset/re-enumerated; ignored local Wi-Fi and pool inputs
may be seeded privately; and the repo-owned conservative 400 MHz / 1100 mV /
100% fan profile may mine for exactly 600 active seconds while the current
session's serial-derived origin is observed through HTTP and WebSocket. The
campaign must pause, safe-stop, clean up USB ownership, and may use at most one
exact-package recovery flash after a post-flash failure. No upstream-default or
overclock profile, arbitrary control target, unbounded mining, OTA, erase,
fault injection, physical power action, direct UART, or electrical pin/pad/
header/probe/jumper/solder/signal manipulation is authorized.

Evidence/privacy/retry: `scratch/stat001-hashrate-monitor/wrapper-001` and
`attempt-001` are ignored mode-`0700` roots with mode-`0600` files. Credentials,
pool/owner/worker fields, origins, ports, USB/network identity, exact hashrates,
sensors, HTTP/WebSocket bodies, serial, commands, PIDs, and traces stay private.
Only the plan-named closed projection may become public after independent
validation. Starting command 2 consumes attempt-001. Every post-flash failure
preserves the earliest typed category and performs bounded safe stop, recovery,
and cleanup. Detector ambiguity/failure, missing inputs, unsafe state,
malformed/incomplete proof, cleanup/recovery/privacy failure, or nonzero command
stops without retry and leaves STAT-001 `implemented`.

Attempt-001 outcome: detector command 1 admitted exactly one Ultra 205. Capture
command 2 then failed its immutable pre-effect source/reference admission with
typed category `evidence_invalid`; the broad `update_hash_counter` fragment had
eight legitimate upstream occurrences while the wrapper required one. No
attempt root, flash, mining, or public projection was created. Attempt-001 is
consumed, no retry was run, and STAT-001 remains `implemented`. The faulty
fragment and a second ambiguous source fragment were narrowed and a current-
repository admission regression was added after the failure. See
`docs/parity/work-plans/20260816T005443Z-STAT-001/CLOSURE.md`.

Next safe action: a future immutable STAT-001 plan may authorize fresh
attempt-002 after the admission correction is committed, pushed, fully gated,
and bound to a newly built exact package. It must repeat detector admission and
may not reuse attempt-001 or infer hardware evidence from this pre-effect
failure.

Attempt-002 verification-promotion plan:
`docs/parity/work-plans/20260816T020135Z-STAT-001/PLAN.md`

- [x] Rebind the closed `bitaxe-hashrate-monitor-evidence-v1` workflow,
      independent validator, generated contract, current task/plan admission,
      and protected paths to fresh attempt-002 without changing production
      hashrate behavior or the evidence quorum.
- [x] Run every focused and mandatory software, firmware, privacy, reference,
      package, and exact-source gate; commit and push before device access.
- [x] Run only the exact plan detector and one conditional attempt-002 command,
      then promote STAT-001 only if the complete independently validated
      exact-package hashrate quorum passes.

Attempt-002 authorization: after the immutable plan/task checkpoint and exact
implementation are clean, fully gated, committed, and pushed, one exact
board-205 package may be factory-flashed/reset; ignored local Wi-Fi and pool
credentials may be seeded privately; and the repo-owned conservative 400 MHz /
1100 mV / 100% fan profile may mine for exactly 600 accumulated active seconds
while protected current-session HTTP, WebSocket, and serial observations are
joined. The campaign must pause, safe-stop, clean up USB/process ownership, and
may use at most one exact-package recovery flash after a post-flash failure.

Attempt-002 evidence, privacy, recovery, and retry: only fresh ignored mode-
`0700` `scratch/stat001-hashrate-monitor/wrapper-002` and `attempt-002` roots
with mode-`0600` files are permitted. Credentials, pool/owner/worker fields,
origins, ports, USB/network/process identities, exact hashrates, sensors,
HTTP/WebSocket bodies, serial, commands, PIDs, and traces remain private. The
sole public aggregate projection is written only after independent validation.
Starting the capture consumes attempt-002; preserve the earliest typed failure,
run bounded safe stop/recovery/cleanup after post-flash failures, and do not run
an unchanged retry or attempt-003. Detector ambiguity/failure, missing inputs,
unsafe state, malformed/incomplete proof, cleanup/recovery/privacy failure, or
nonzero command stops with STAT-001 `implemented` and evidence withheld.

Attempt-002 prohibited effects and acceptance: no upstream-default/overclock
profile, arbitrary control target, unbounded mining, OTA, erase, fault
injection, physical power action, direct UART, or pin/pad/header/probe/jumper/
solder/signal manipulation is permitted. Promotion requires the immutable
plan's exact clean source/reference/package and detector identity; one ASIC,
four domains, one-second cadence and pinned register semantics; twenty active
windows with work renewal; changing coherent positive HTTP/WebSocket current
hashrate and positive rolling windows after warmup; bounded error; terminal
zero current rate; safe stop, cleanup, protected modes, seal, independent
validation, and redaction. Any missing fact withholds promotion.

Attempt-002 completion review: the fresh detector admitted exactly one Ultra
205, but the sole capture stopped with closed category `hardware_blocked` and
a sealed `admission_failed` result before package admission or campaign USB
execution. Source inspection found that the wrapper paired `soak` with
`conservative`, although campaign admission permits the conservative
600-second profile only for `live-share`. The wrapper and acceptance boundary
now use `live-share`, and the real-child regression rejects the former invalid
stage/profile pair. No public projection exists, attempt-002 was not retried,
and STAT-001 remains `implemented`. A future immutable plan may authorize
fresh attempt-003 only from the committed, pushed, fully gated correction and
a newly built exact package. See
`docs/parity/work-plans/20260816T020135Z-STAT-001/CLOSURE.md`.

Attempt-003 verification-promotion plan:
`docs/parity/work-plans/20260816T022946Z-STAT-001/PLAN.md`

- [x] Rebind `bitaxe-hashrate-monitor-evidence-v1`, the independent validator,
      generated contract, task/plan admission, Bazel runfiles, and protected
      paths to fresh attempt-003.
- [x] Prove the corrected `live-share` plus `conservative` child command and
      reject the former `soak` plus `conservative` pair at the real boundary.
- [x] Pass every focused and mandatory software, firmware, privacy, reference,
      package, and exact-source gate; commit and push before device access.
- [x] Run only the exact plan detector and one conditional attempt-003 command,
      then promote only on the complete independently validated quorum.

Attempt-003 authorization: pushed commit `0d058a66` materially fixes the exact
attempt-002 pre-package campaign-admission boundary and regression-guards the
real child command. After this immutable task/plan checkpoint and the rebound
implementation are clean, fully gated, committed, and pushed, one exact board-
205 package may be factory-flashed/reset; ignored local Wi-Fi and pool inputs
may be seeded privately; and the repo-owned `live-share` conservative 400 MHz /
1100 mV / 100% fan profile may mine for exactly 600 accumulated active seconds
while protected current-session HTTP, WebSocket, and serial observations are
joined. The campaign must pause, safe-stop, clean up USB/process ownership, and
may use at most one exact-package recovery flash after a post-flash failure.

Attempt-003 evidence, privacy, recovery, and retry: use only fresh ignored
mode-`0700` `scratch/stat001-hashrate-monitor/wrapper-003` and `attempt-003`
roots with mode-`0600` files. Credentials, pool/owner/worker fields, origins,
ports, USB/network/process identities, exact hashrates, sensors, HTTP/WebSocket
bodies, serial, commands, PIDs, and traces remain private. The sole public
aggregate projection is written only after independent validation. Starting
the capture consumes attempt-003; preserve the earliest typed failure, run
bounded safe stop/recovery/cleanup after post-flash failures, and do not run an
unchanged retry or attempt-004. Detector ambiguity/failure, missing inputs,
unsafe state, malformed/incomplete proof, cleanup/recovery/privacy failure, or
nonzero command stops with STAT-001 `implemented` and evidence withheld.

Attempt-003 prohibited effects and acceptance: no upstream-default/overclock
profile, arbitrary control target, unbounded mining, OTA, erase, fault
injection, physical power action, direct UART, or pin/pad/header/probe/jumper/
solder/signal manipulation is permitted. Promotion requires the linked plan's
exact clean source/reference/package and detector identity; one ASIC, four
domains, one-second cadence and pinned register semantics; twenty active
windows with work renewal; changing coherent positive HTTP/WebSocket current
hashrate and positive rolling windows after warmup; bounded error; terminal
zero current rate; safe stop, cleanup, protected modes, seal, independent
validation, and redaction. Any missing fact withholds promotion.

Attempt-003 completion review: exact clean pushed source/package `3b03502e`,
all software/privacy/reference gates, and one detector passed. The sole
campaign crossed attempt-002's corrected boundary: package admission,
`live-share` plus `conservative`, protocol readiness, observation start, 1,361
accepted markers, and 366,166 active milliseconds were sealed. It then stopped
as `hardware_blocked` / `runtime_identity_untrusted`: all 41 UTF-8 runtime-
attestation candidates reduced to `malformed`. Safe stop and USB cleanup pass,
protected modes and the result seal pass, and the public projection is absent.
STAT-001 remains `implemented`; attempt-003 is consumed and attempt-004 is not
authorized. The linked `CLOSURE.md` requires a software-only closed parse-
discriminator diagnosis and targeted regression-backed fix before any future
hardware ordinal.

Runtime-attestation diagnosis plan:
`docs/parity/work-plans/20260816T030231Z-STAT-001/PLAN.md`

- [x] Add bounded, closed parse-failure counts to the runtime-attestation
      accumulator, production serial diagnostics, and sealed campaign result.
- [x] Reproduce the source-owned producer/parser mismatch at the real serial
      boundary and apply the minimum regression-backed token-boundary fix.
- [x] Run every focused and mandatory gate, record a non-promotion closure, and
      leave attempt-004 unauthorized until a separate immutable hardware plan.

Software-only authorization: local code, fixtures, tests, builds,
documentation, and ordinary git operations only. Do not access protected
attempt-003 artifacts, credentials, detector/device/hardware, network runtime,
or any electrical interface. Do not flash, mine, actuate, erase, update, inject
faults, create a public projection, or start attempt-004.

Runtime-attestation diagnosis completion review: pushed implementation commit
`f26fff55c1513f342946f16999d8564cc761ba01` proves the source-owned
`runtime_boot_attestation=unavailable` diagnostic was falsely admitted by a raw
substring matcher. The shared parser and production serial analyzer now require
the complete whitespace-delimited marker token, distinguish lookalikes, and
emit only closed value-free parse-failure categories with saturating counts.
Focused regressions, the complete ordered repository gates, redaction,
reference verification, and the canonical firmware build pass. No protected
attempt evidence or hardware was accessed, no checklist field changed, and
STAT-001 remains `implemented`. See
`docs/parity/work-plans/20260816T030231Z-STAT-001/CLOSURE.md`.

Next safe action: a separately committed immutable STAT-001 plan may authorize
fresh attempt-004 only after binding this pushed correction to a newly built
exact package and restating the complete detector, evidence/privacy, recovery,
retry, and acceptance contract. This plan does not authorize attempt-004.

Attempt-004 verification-promotion plan:
`docs/parity/work-plans/20260816T033934Z-STAT-001/PLAN.md`

- [x] Rebind `bitaxe-hashrate-monitor-evidence-v1`, the independent validator,
      generated contract, task/plan admission, Bazel runfiles, and protected
      paths to fresh attempt-004 and sealed campaign-result v10.
- [x] Preserve the new closed runtime-attestation parse discriminator for a
      non-ready campaign without exposing raw values, source text, or protected
      identifiers.
- [x] Pass every focused and mandatory software, firmware, privacy, reference,
      package, and exact-source gate; commit and push before device access.
- [x] Run only the exact detector and conditional attempt-004 commands in the
      linked plan, then promote only on the complete independently validated
      hashrate quorum.

Attempt-004 authorization: pushed commit `f26fff55` fixes attempt-003's exact
false-marker boundary with a shared whitespace-delimited marker matcher,
closed parser diagnostics, production-shaped tests, and campaign-result v10.
After this immutable task/plan checkpoint and its rebound implementation are
clean, fully gated, committed, and pushed, one exact board-205 package may be
factory-flashed/reset; ignored local Wi-Fi and pool inputs may be seeded
privately; and the repo-owned conservative 400 MHz / 1100 mV / 100% fan profile
may mine for exactly 600 accumulated active seconds while protected current-
session HTTP, reconstructed WebSocket, and serial observations are joined.

Attempt-004 exact commands: after the clean pushed implementation and package
gates, create absent mode-0700 `scratch/stat001-hashrate-monitor/wrapper-004`
with private mode-0600 detector streams and run exactly one
`just detect-ultra205`. Only after its zero exit, one-device board-205
admission, cleanup/holder proof, nonempty opaque credential inputs, and absent
attempt/projection paths, run exactly once:
`just capture-hashrate-monitor-evidence --private-root scratch/stat001-hashrate-monitor/attempt-004 --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --pool-credentials pool-credentials.json --detector-output scratch/stat001-hashrate-monitor/wrapper-004/detector.stdout --projection docs/parity/evidence/stat001-hashrate-monitor/hashrate-monitor-projection.json --duration-seconds 600 --capture-timeout-seconds 1500`, with distinct private capture stdout/stderr under the wrapper root. The full
shell guards and redirections are frozen in the linked plan.

Attempt-004 evidence/privacy/recovery: the wrapper and supervisor-owned attempt
roots are ignored mode-0700 directories with mode-0600 files. Credentials,
pool/owner/worker fields, origins, ports, hostnames, USB/network/process
identity, raw hashrates and sensors, bodies, logs, commands, PIDs, and traces
remain private; only the plan-named closed projection may become public after
independent validation. Starting capture consumes attempt-004. Preserve the
earliest typed failure, perform the base campaign's bounded safe stop,
recovery, seal, and cleanup, and never retry unchanged or start attempt-005.

Attempt-004 safety, prohibited effects, and acceptance: require fresh bounded
safety truth, 4.5-5.5 V input, at most 15 W, ASIC temperature below 75 C, and
fresh nonzero fan RPM after the 100% command. No upstream-default/overclock or
arbitrary control profile, automatic fan, unbounded mining, OTA, erase, raw
write, fault injection, physical power action, external UART, or pin/pad/
header/GPIO/probe/jumper/solder/signal work is permitted. Promotion requires
the linked plan's exact package and detector identity, one ASIC/four domains,
one-second cadence, twenty complete windows with work renewal, changing
coherent positive HTTP/WebSocket hashrates and warm rolling windows, bounded
error, terminal zero, safe stop, cleanup, private modes, seals, independent
validation, and redaction. Any missing fact preserves `implemented`, withholds
the projection, and selects one closed terminal outcome.

Attempt-004 closure:
`docs/parity/work-plans/20260816T033934Z-STAT-001/CLOSURE.md`

Attempt-004 completion review: pushed source `1368e573` passed every software,
privacy, package, exact-source, detector, and credential/path admission gate.
The sealed campaign itself was accepted with trusted runtime identity, zero
parse failures, confirmed safe stop, and ready USB cleanup. Promotion was
correctly withheld because live-share produced network status `not_required`,
zero covered windows, and no transport quorum. The root cause is production
source: `CampaignNetworkCoordinator` runs `observe_network` only for `Soak`
and `CommandEffects`, while this safety-bounded STAT-001 workflow must use
conservative `LiveShare`. Attempt-004 is consumed; no attempt-005 is authorized
under this plan. The next safe action is a fresh software plan that admits
`LiveShare` to the existing network observer with regression coverage, then a
separately planned fresh hardware ordinal after that fix is pushed.

LiveShare network-observer software plan:
`docs/parity/work-plans/20260816T044527Z-STAT-001/PLAN.md`

- [x] Centralize campaign-stage network observation in one closed production
      policy so serial admission, worker selection, timeouts, and finish
      semantics cannot drift.
- [x] Route conservative `LiveShare` and `Soak` through the existing continuity
      observer while preserving `CommandEffects` and both `not_required`
      stages exactly.
- [x] Add focused stage-mapping and campaign/network regressions, then pass all
      mandatory software, package, privacy, reference, and parity gates.
- [x] Commit and push the correction, close the software-only plan without a
      checklist transition, and leave any fresh hardware ordinal to a separate
      immutable plan.

Software-only authorization: local source, fixtures, tests, builds,
documentation, and ordinary git operations only. Do not read attempt-004
protected artifacts, credentials, detector output, USB/device/network runtime,
or private endpoints. Do not detect, flash, monitor, mine, actuate, update,
erase, inject faults, manipulate physical power, use external UART, touch pins,
pads, headers, GPIO, probes, jumpers, solder, or signals, create a projection,
or start attempt-005. Completion proves only the corrected host observation
policy; STAT-001 remains `implemented` until separate hardware evidence passes.

Completion review: Source commit
`89e8c34c794e6cfca499e4f392699be39e20e7dd` centralizes the exact five-stage
network-observation policy and routes conservative `LiveShare` through the
continuity observer. Focused network tests passed 28/28, the broader campaign
slice passed 27/27, the canonical package built, privacy/reference checks
passed, and the full ordered repository gate sequence passed. No checklist cell
changed. Residual risk is hardware-only: a separately authorized fresh plan
must prove the corrected observer against one newly built exact package and a
new detector-gated Ultra 205 campaign before STAT-001 can move beyond
`implemented`.

Attempt-005 verification-promotion plan:
`docs/parity/work-plans/20260816T050533Z-STAT-001/PLAN.md`

- [x] Rebind the private-first hashrate workflow, independent validator,
      generated contract, task/plan admission, Bazel inputs, and tests from
      consumed attempt-004 to fresh attempt-005.
- [x] Prove the exact five-stage network-observer correction, conservative
      400 MHz / 1,100 mV / 100% profile, volt-typed input safety, and every
      source, package, privacy, reference, and mandatory repository gate.
- [x] Commit and push the exact implementation and rebuild its package before
      accessing the detector, credential presence, or hardware.
- [x] Run only the plan's exact detector and conditional attempt-005 capture,
      then promote STAT-001 only on the complete independently validated
      network/hashrate/safe-stop/cleanup quorum.

Attempt-005 authorization: pushed source
`89e8c34c794e6cfca499e4f392699be39e20e7dd` materially corrects attempt-004's
exact `LiveShare` network-observer boundary. After this immutable plan and its
attempt rebind are fully gated, committed, pushed, and packaged, exactly one
detector and at most one conditional capture may factory-flash/reset one Ultra
205, seed ignored opaque credentials, and run the repo-owned conservative
profile for 600 accumulated active seconds. The ASIC command is 400 MHz,
1,100 millivolts core voltage, and 100% fan; independent input safety remains
4.5-5.5 volts, at most 15 W, below 75 C, with fresh nonzero fan RPM.

Evidence/privacy/recovery/retry: use only fresh ignored mode-`0700`
`scratch/stat001-hashrate-monitor/wrapper-005` and `attempt-005` roots with
mode-`0600` files. Credentials, owner/worker fields, endpoints, origins, ports,
USB/network/process identity, raw hashrates and sensors, bodies, logs, commands,
PIDs, and traces remain private. Only the closed projection may be published
after independent validation. Starting capture consumes attempt-005; preserve
the earliest failure, safe-stop/recover/seal/clean up, and never retry unchanged
or start attempt-006. No other profile, arbitrary control, unbounded mining,
OTA, erase, raw write, fault injection, physical power action, external UART,
or electrical pin/pad/header/GPIO/probe/jumper/solder/signal work is authorized.

Attempt-005 closure:
`docs/parity/work-plans/20260816T050533Z-STAT-001/CLOSURE.md`

Attempt-005 completion review: exact pushed source `1090cf6e`, pinned reference,
clean package identity, every software/privacy gate, and the sole detector
passed. The one capture crossed attempt-004's corrected LiveShare observer
boundary and produced 11 complete network windows over 310,615 active
milliseconds with trusted identity, fresh safety, 61 HTTP successes, 298
WebSocket frames without transport failures, changing coherent positive
hashrates in both transports, terminal zero, confirmed safe stop, persistence,
USB cleanup, seals, modes, and redaction. It then failed closed as
`watchdog_unresponsive`; the projection was withheld and STAT-001 remains
`implemented`. Current closed evidence cannot distinguish the watchdog sample
predicate from HTTP/WebSocket per-window sequence advancement. Attempt-005 is
consumed, no retry or attempt-006 is authorized, and the next safe action is a
software-only discriminator plan before any further hardware ordinal.

Watchdog diagnostic-completeness plan:
`docs/parity/work-plans/20260816T060214Z-STAT-001/PLAN.md`

- [x] Replace the lossy watchdog boolean diagnosis with one closed,
      value-free earliest-failure discriminator covering every sample
      predicate and HTTP/WebSocket checkpoint/feed advancement boundary.
- [x] Carry the discriminator through sealed network evidence, campaign-result
      v11, and the hashrate wrapper's seal- and category-gated failure envelope.
- [x] Add exhaustive focused regressions, run every mandatory gate, commit and
      push, then close without a checklist transition or hardware access.

Software-only authorization: local source, fixtures, tests, builds,
documentation, and ordinary git operations only. Do not read attempt-005
protected artifacts, credentials, detector output, USB/device/network runtime,
or private endpoints. Do not detect, flash, monitor, mine, actuate, update,
erase, inject faults, manipulate physical power, use external UART, touch any
electrical pin/pad/header/GPIO/probe/jumper/solder/signal interface, create a
projection, or start attempt-006. Completion preserves STAT-001 as
`implemented`; a separate future immutable plan is required for hardware.

Diagnostic completion review: pushed source `f9232963` replaces the ambiguous
watchdog boolean with twelve closed value-free earliest-failure labels across
sample predicates and HTTP/WebSocket checkpoint/feed advancement. Network v5,
campaign-result v11, and the seal/category-gated wrapper carry the diagnostic.
All focused and mandatory software, package, privacy, reference, parity, and
diff gates pass. No hardware or protected input was accessed, no checklist
field changed, and STAT-001 remains `implemented`. See
`docs/parity/work-plans/20260816T060214Z-STAT-001/CLOSURE.md`.

Next safe action: a separate immutable plan may bind pushed `f9232963` to a
new exact package and authorize one detector-gated attempt-006. Any repeated
watchdog failure must surface one sealed discriminator before another source
change or retry; this plan does not authorize attempt-006.

Attempt-006 verification-promotion plan:
`docs/parity/work-plans/20260816T173058Z-STAT-001/PLAN.md`

- [x] Rebind the private-first hashrate workflow, independent validator,
      generated contract, immutable task/plan admission, Bazel inputs, and
      protected paths from consumed attempt-005 to fresh attempt-006.
- [x] Preserve campaign-result v11 and prove every closed watchdog
      discriminator, success `none`, earliest-failure precedence, real-child
      envelope, profile-unit, volt-typed safety, seal, mode, and privacy gate.
- [x] Pass every focused and mandatory gate, commit and push the exact source,
      rebuild its package, then run only the linked detector and conditional
      attempt-006 capture.
- [ ] Promote only STAT-001 on the complete independently validated twenty-
      window network/hashrate/watchdog/terminal-zero/safe-stop/cleanup quorum;
      otherwise withhold evidence, record the earliest closed blocker, and stop.

Attempt-006 authorization: pushed commit `f9232963` materially fixes the exact
attempt-005 ambiguity by carrying twelve closed watchdog failure labels through
campaign-result v11 and the sealed wrapper envelope. After this immutable plan
and its rebound implementation are clean, fully gated, committed, pushed, and
packaged, exactly one detector and at most one conditional capture may
factory-flash/reset one Ultra 205, seed ignored opaque credentials, and run the
repo-owned conservative 400 MHz / 1,100 mV / 100% profile for 600 accumulated
active seconds. Input safety remains independently 4.5-5.5 volts, at most 15 W,
below 75 C, with fresh nonzero fan RPM.

Attempt-006 evidence, recovery, retry, and non-scope: use only fresh ignored
mode-0700 `scratch/stat001-hashrate-monitor/wrapper-006` and `attempt-006`
roots with mode-0600 files. Credentials, workers, endpoints, origins, ports,
USB/network/process identity, raw hashrates/sensors, bodies, logs, commands,
PIDs, and traces remain private. Only the closed projection may publish after
independent validation. Starting capture consumes attempt-006; preserve the
earliest category/discriminator, safe-stop/recover/seal/clean up, and never
retry unchanged or start attempt-007. No other profile, arbitrary control,
unbounded mining, OTA, erase, raw write, fault injection, physical power
action, external UART, or electrical pin/pad/header/GPIO/probe/jumper/solder/
signal work is authorized. Any incomplete boundary leaves STAT-001
`implemented`.

Attempt-006 completion review: exact pushed source `5a1c6960`, pinned
reference, clean board-205 package, software gates, privacy gates, detector,
and protected-path admission passed. The sole capture failed closed as
`watchdog_unresponsive` with the new sealed value-free discriminator
`watchdog_not_participating`. Package/runtime identity were trusted, runtime
attestation parsing and production serial were clean, terminal HTTP/WebSocket/
pool state was valid, safe stop and USB cleanup passed, the result seal and
protected modes passed, and no public projection was written. STAT-001 remains
`implemented`; the checklist and progress history are unchanged. See
`docs/parity/work-plans/20260816T173058Z-STAT-001/CLOSURE.md`.

Next safe action: a separate immutable software-only plan must trace the live
watchdog participant registry, campaign predicate, checkpoint/feed ownership,
and task lifecycle, reproduce the proved `watchdog_not_participating` boundary,
and make a targeted source fix. Attempt-006 is consumed; no unchanged retry or
attempt-007 is authorized.

Watchdog-classifier diagnosis plan:
`docs/parity/work-plans/20260816T180839Z-STAT-001/PLAN.md`

- [x] Map every closed runtime-health watchdog reason before the generic
      participation-consistency guard, preserving earliest-failure precedence
      and value-free public labels.
- [x] Rotate the sealed network and campaign-result schemas and update every
      producer, consumer, wrapper gate, fixture, and production-shaped
      regression for the complete vocabulary.
- [x] Pass every focused and mandatory software, firmware, package, privacy,
      provenance, parity, and diff gate; record a non-promotion closure and
      leave STAT-001 and progress history unchanged.

Software-only authorization: local source, fixtures, tests, builds,
documentation, and ordinary git operations only. Do not access attempt-006,
ignored credentials, detector/device/network runtime, or private values. Do
not flash, reset, mine, actuate, update, erase, inject faults, use direct UART
or electrical interfaces, create a public projection, or start attempt-007.

Classifier-correction completion review: pushed implementation `91ab642b`
preserves every evaluator-owned watchdog reason before the generic
participation guard, rotates campaign-result/network-continuity to v12/v6, and
passes focused, full repository, firmware-package, privacy, provenance,
parity-invariance, immutable-plan, and diff gates. No hardware or protected
attempt input was accessed. STAT-001 remains `implemented`; the checklist and
progress history are unchanged. See
`docs/parity/work-plans/20260816T180839Z-STAT-001/CLOSURE.md`.

Next safe action: run the clean synchronized selector in a new invocation. A
fresh immutable plan may authorize one attempt-007 only if it admits pushed
v12/v6 source and defines the complete hardware, privacy, recovery, cleanup,
retry, stop, and promotion contract. Never reuse attempt-006.

Attempt-007 verification-promotion plan:
`docs/parity/work-plans/20260816T183130Z-STAT-001/PLAN.md`

- [x] Rebind the private-first workflow, independent validator, generated
      contract, immutable plan/task admission, Bazel input, fixtures, and
      protected paths from consumed attempt-006 to fresh attempt-007.
- [x] Preserve campaign-result v12, network-continuity v6, every closed
      reason-specific watchdog value, success `none`, earliest-failure, seal,
      mode, privacy, profile-unit, and volt-typed input-safety boundaries.
- [x] Pass every focused and mandatory gate, commit and push the exact source,
      rebuild its package, then run only the linked detector and at most one
      conditional attempt-007 capture.
- [ ] Promote only STAT-001 on the complete independently validated twenty-
      window network/hashrate/watchdog/terminal-zero/safe-stop/cleanup quorum;
      otherwise withhold projection, record the earliest closed blocker, and
      stop without retry.

Attempt-007 objective and commands: pushed commit `91ab642b` materially fixes
attempt-006's exact collapsed watchdog discriminator at the production-shaped
and real-child boundaries. After this immutable plan and attempt rebind are
fully gated, committed, pushed, and packaged, run exactly the linked plan's
one `just detect-ultra205` command and, only on clean one-device board-205
admission plus absent fresh paths and nonempty ignored credentials, its one
`just capture-hashrate-monitor-evidence` command for
`scratch/stat001-hashrate-monitor/attempt-007`, conservative `live-share`, 600
active seconds, and a 1,500-second capture timeout. The wrapper root is
`scratch/stat001-hashrate-monitor/wrapper-007`; the public candidate remains
`docs/parity/evidence/stat001-hashrate-monitor/hashrate-monitor-projection.json`.

Attempt-007 allowed effects and safety: one exact board-205 factory flash and
normal USB reset/re-enumeration; private ignored Wi-Fi/pool seeding; one
bounded 400 MHz, 1,100 millivolt core-voltage, 100-percent-fan conservative
campaign; protected current-session serial/HTTP/WebSocket observation; pause,
safe stop, cleanup, sealing; and at most one supervisor-owned exact-package
recovery flash after a post-flash failure. Require fresh 4.5-5.5 V input, at
most 15 W, ASIC temperature below 75 C, and fresh nonzero fan RPM. Input
voltage is volts and remains distinct from the millivolt core setpoint.

Attempt-007 evidence/privacy: use only a fresh ignored mode-0700 wrapper parent
with distinct mode-0600 detector/capture siblings and a previously absent
supervisor-owned mode-0700 attempt child containing only mode-0600 artifacts.
Credentials, workers, endpoints, origins, ports, hostnames, USB/network/process
identity, raw hashrates/sensors, bodies, logs, commands, PIDs, and traces remain
private. Only the closed projection may publish after independent validation.
Never read, print, summarize, commit, or expose credential contents or private
operational values.

Attempt-007 prohibited effects, recovery, retry, and stops: no other profile,
arbitrary control, automatic fan, unbounded mining, OTA, erase, raw write,
fault injection, physical power action, direct UART, or electrical
pin/pad/header/GPIO/probe/jumper/solder/signal work. Preserve the earliest
failure and watchdog reason through bounded safe stop, recovery, seal, and
cleanup. Starting capture consumes attempt-007; never reuse attempt-006, retry
attempt-007, or start attempt-008. Stop on detector ambiguity/failure, non-205
identity, missing inputs, drift, unsafe state, incomplete quorum, failed
recovery/cleanup/seal/mode/privacy, nonzero command exit, or successful
projection. Accepted terminal outcomes are `complete`,
`stop_repeated_boundary`, `stop_hardware_blocker`,
`stop_authority_boundary`, and `stop_impossible_contract`.

Attempt-007 closure:
`docs/parity/work-plans/20260816T183130Z-STAT-001/CLOSURE.md`

Attempt-007 completion review: exact pushed source `ec9bedd3`, pinned
reference, clean board-205 package, every software/privacy gate, and the sole
detector passed. The one capture admitted the package and trusted runtime,
covered 15 of 20 required network windows, and retained valid terminal HTTP,
WebSocket, and persisted-pool state. It then failed closed as
`watchdog_unresponsive` with the reason-specific sealed discriminator
`watchdog_feed_stale`. Fresh safety, confirmed safe stop, ready USB cleanup,
the result/network seals, protected modes, and redaction passed. The public
projection is absent, parity promotion is false, and STAT-001 remains
`implemented`; attempt-007 is consumed and no retry or attempt-008 is
authorized. The next safe action is a software-only diagnosis of watchdog
checkpoint/feed scheduling and task lifecycle before any further hardware
ordinal.

Watchdog-timeout correction plan:
`docs/parity/work-plans/20260816T192025Z-STAT-001/PLAN.md`

- [x] Replace the unrelated 2,000-ms projected feed-freshness threshold with
      the exact compiled ESP-IDF task-watchdog timeout supplied by the firmware
      boundary to the pure evaluator.
- [x] Prove 2,001 ms, the exact configured boundary, and the first stale
      millisecond with focused pure and firmware-ownership regressions while
      preserving every existing closed watchdog failure.
- [x] Run the complete software, firmware, package, privacy, provenance,
      parity-invariance, immutable-plan, and diff gates; commit and push a
      non-promotion closure without accessing hardware or attempt-007.

Software-only authorization: local source, fixtures, tests, builds,
documentation, and ordinary git operations only. Do not access protected
attempts, ignored credentials, detector/device/network runtime, private values,
or a public projection. Do not flash, reset, monitor, mine, actuate, update,
erase, inject faults, manipulate power, use direct UART, or touch electrical
interfaces. This plan does not authorize attempt-008 and leaves STAT-001
`implemented` with unchanged checklist and progress history.

Watchdog-timeout correction completion review: pushed source `145eff42`
replaces the false 2,000-ms task-watchdog freshness boundary with the compiled
ESP-IDF five-second timeout supplied by the firmware adapter. Focused tests
prove 2,001 ms and 5,000 ms fresh, 5,001 ms stale, and every existing closed
failure unchanged. Exact firmware/package and all full software, privacy,
reference, parity-invariance, immutable-plan, and diff gates passed. No
protected attempt, credential, detector, device, network runtime, or hardware
effect was accessed. STAT-001 remains `implemented`; the checklist and
progress history are unchanged. See
`docs/parity/work-plans/20260816T192025Z-STAT-001/CLOSURE.md`.

Next safe action: a fresh immutable plan may consider one detector-gated
attempt-008 only after binding exact pushed source `145eff42` and defining the
complete hardware, privacy, recovery, cleanup, retry, stop, and promotion
contract. Attempt-007 is consumed and must not be reused.

Attempt-008 plan:
`docs/parity/work-plans/20260816T200554Z-STAT-001/PLAN.md`

Attempt-008 objective and commands: pushed correction `145eff42` fixes the
exact attempt-007 `watchdog_feed_stale` boundary, while current pushed source
`d6f4c6ab` adds only its truthful closure. After the immutable plan and narrow
attempt rebind are fully gated, committed, pushed, and packaged, run exactly
the linked plan's one `just detect-ultra205` command and, only on clean
one-device board-205 admission plus absent fresh paths and nonempty ignored
credentials, its one `just capture-hashrate-monitor-evidence` command for
`scratch/stat001-hashrate-monitor/attempt-008`, conservative `live-share`, 600
active seconds, and a 1,500-second capture timeout. The wrapper root is
`scratch/stat001-hashrate-monitor/wrapper-008`; the public candidate remains
`docs/parity/evidence/stat001-hashrate-monitor/hashrate-monitor-projection.json`.

Attempt-008 allowed effects and safety: one exact board-205 factory flash and
normal USB reset/re-enumeration; private ignored Wi-Fi/pool seeding; one
bounded 400 MHz, 1,100 millivolt core-voltage, 100-percent-fan conservative
campaign; protected current-session serial/HTTP/WebSocket observation; pause,
safe stop, cleanup, sealing; and at most one supervisor-owned exact-package
recovery flash after a post-flash failure. Require fresh 4.5-5.5 V input, at
most 15 W, ASIC temperature below 75 C, and fresh nonzero fan RPM. Input
voltage is volts and remains distinct from the millivolt core setpoint.

Attempt-008 evidence/privacy: use only a fresh ignored mode-0700 wrapper parent
with distinct mode-0600 detector/capture siblings and a previously absent
supervisor-owned mode-0700 attempt child containing only mode-0600 artifacts.
Credentials, workers, endpoints, origins, ports, hostnames, USB/network/process
identity, raw hashrates/sensors, bodies, logs, commands, PIDs, and traces remain
private. Only the closed projection may publish after independent validation.
Never read, print, summarize, commit, or expose credential contents or private
operational values.

Attempt-008 prohibited effects, recovery, retry, and stops: no other profile,
arbitrary control, automatic fan, unbounded mining, OTA, erase, raw write,
fault injection, physical power action, direct UART, or electrical
pin/pad/header/GPIO/probe/jumper/solder/signal work. Preserve the earliest
failure and watchdog reason through bounded safe stop, recovery, seal, and
cleanup. Starting capture consumes attempt-008; never reuse attempt-007, retry
attempt-008, or start attempt-009. Stop on detector ambiguity/failure, non-205
identity, missing inputs, drift, unsafe state, incomplete quorum, failed
recovery/cleanup/seal/mode/privacy, nonzero command exit, or successful
projection. Accepted terminal outcomes are `complete`,
`stop_repeated_boundary`, `stop_hardware_blocker`, `stop_authority_boundary`,
and `stop_impossible_contract`.

Attempt-008 closure:
`docs/parity/work-plans/20260816T200554Z-STAT-001/CLOSURE.md`

Attempt-008 completion review: exact pushed source `72c75876`, the pinned
reference, clean board-205 package, every software/privacy gate, and the sole
detector passed. The one capture admitted the exact package and completed 14
of 20 required windows before failing closed as `watchdog_unresponsive` with
the same sealed discriminator `watchdog_feed_stale` seen in attempt-007, now
with the compiled five-second timeout correction present. Runtime attestation
had no parse failure. Confirmed safe stop, ready USB cleanup, both seals,
protected modes, and projection withholding passed. The public projection is
absent, parity promotion is false, and STAT-001 remains `implemented` with
unchanged checklist and progress history. Attempt-008 is consumed and no retry
or attempt-009 is authorized.

Next safe action: a fresh immutable software-only STAT-001 diagnosis must
trace the real watchdog feed owner, checkpoint cadence, task blocking, and
scheduler lifecycle, reproduce the post-window feed-staleness transition, and
apply a targeted source correction before any new hardware ordinal can be
considered.

Cooperative watchdog-progress correction plan:
`docs/parity/work-plans/20260816T225404Z-STAT-001/PLAN.md`

- [x] Make recurring ESP task-watchdog feeds reflect completed cooperative
      progress inside the production-owner dispatch/effect cascade while
      preserving the owner task as the sole subscription and feed authority.
- [x] Prove a progressing multi-event cascade remains feedable and a single
      unfinished effect is not masked, with production-shaped ownership and
      configured-timeout regressions.
- [x] Run all focused, firmware-package, privacy, reference, parity-invariance,
      immutable-plan, and mandatory repository gates; commit and push a
      software-only closure without a checklist transition or hardware access.

Software-only authorization: local source, fixtures, tests, builds,
documentation, and ordinary git operations only. Do not access protected
attempts, ignored credentials, detector/device/network runtime, private
values, or a public projection. Do not detect, flash, reset, monitor, mine,
actuate, update, erase, inject faults, manipulate power, use direct UART, or
touch electrical interfaces. Attempt-008 remains consumed; this plan does not
authorize attempt-009. STAT-001 remains `implemented` until a separately
authorized exact-package hardware campaign proves the full quorum.

Completion review: pushed source
`f5f1be9b4614c155df96aaa78a2271c60065f84f` makes recurring task-watchdog
feeds track completed production-owner event/effect progress without weakening
the compiled timeout or masking an unfinished effect. Focused regressions, the
exact firmware package, all mandatory repository gates, privacy, reference,
parity invariance, plan immutability, and diff review pass. No hardware,
detector, credentials, protected attempt evidence, or public projection was
accessed; STAT-001 remains `implemented` with unchanged checklist and progress
history. See
`docs/parity/work-plans/20260816T225404Z-STAT-001/CLOSURE.md`.

Residual risk and next safe action: software cannot prove live watchdog
freshness or the twenty-window hashrate quorum. A fresh immutable plan must
bind the pushed correction to an exact board-205 package and authorize a new
attempt-009 contract before hardware use; attempts 007 and 008 remain
consumed.

Attempt-009 verification-promotion plan:
`docs/parity/work-plans/20260816T231527Z-STAT-001/PLAN.md`

- [x] Rebind the private-first workflow, Rust validator, generated contract,
      immutable task/plan admission, Bazel runfiles, protected roots, and
      real-child fixtures from consumed attempt-008 to fresh attempt-009.
- [x] Run every focused and mandatory software, firmware, privacy, reference,
      package, and exact-source gate; commit and push before device access.
- [x] Run only the frozen detector and sole conditional attempt-009 command,
      then promote STAT-001 only if the complete independent quorum passes.

Attempt-009 progress basis and authorization: pushed correction
`f5f1be9b4614c155df96aaa78a2271c60065f84f` makes production-owner watchdog
feeds follow completed event/effect progress, the bounded inbox wait, and
campaign publication without weakening the five-second timeout or feeding
through an unfinished effect. After this plan/task checkpoint and rebound
implementation are clean, fully gated, committed, and pushed, one exact board-
205 package may be factory-flashed/reset; ignored local Wi-Fi and pool inputs
may be seeded privately; and the repo-owned `live-share` conservative 400 MHz /
1100 mV / 100% fan profile may mine for exactly 600 accumulated active seconds
while protected current-session HTTP, WebSocket, and serial observations are
joined. The campaign must pause, safe-stop, release USB/process ownership, and
may use at most one supervisor-owned exact-package recovery flash after a
post-flash failure.

Attempt-009 units and safety: 1100 mV is the ASIC core setpoint, while INA260
input bus voltage is independently measured in volts. Admission requires fresh
4.5-5.5 V input, at most 15 W, ASIC temperature below 75 C, and fresh nonzero
fan RPM after the 100% fan command. The two voltage domains must not be
compared. Stop on stale/unsafe state, ambiguous/non-205 detector identity,
source/reference/package drift, missing credentials, or cleanup/recovery
failure.

Attempt-009 evidence, privacy, recovery, and retry: only fresh ignored mode-
`0700` `scratch/stat001-hashrate-monitor/wrapper-009` and `attempt-009` roots
with mode-`0600` files are permitted. Credentials, pool/owner/worker fields,
origins, ports, USB/network/process identities, exact hashrates, sensors,
HTTP/WebSocket bodies, serial, commands, PIDs, and traces remain private. The
sole public aggregate projection is written only after independent validation.
Starting capture consumes attempt-009; preserve the earliest typed failure,
run bounded safe stop/recovery/cleanup after post-flash failures, and do not
retry unchanged or start attempt-010.

Attempt-009 prohibited effects and acceptance: no upstream-default/overclock
profile, arbitrary control target, automatic fan mode, unbounded mining, OTA,
erase, raw writes, fault injection, physical power action, direct UART, or pin/
pad/header/GPIO/probe/jumper/solder/signal manipulation is permitted. Promotion
requires exact clean source/reference/package/plan and detector identity; one
ASIC/four domains and one-second cadence; all twenty active windows with work
renewal; changing coherent positive HTTP/WebSocket current and rolling rates;
bounded error; watchdog failure `none`; terminal zero current rate; safe stop,
cleanup, protected modes, seals, independent validation, and redaction. Any
missing fact withholds promotion and leaves STAT-001 `implemented`.

Attempt-009 completion review: exact clean pushed source/package
`6fd586dab96a3eca15b7dd68d92de60c275bc5de`, every software/privacy/reference
gate, and one fresh detector passed. The sole capture then repeated the sealed
attempt-008 boundary: `hardware_blocked` / `watchdog_unresponsive` /
`watchdog_feed_stale` after 14 of 20 required windows. Runtime identity was
trusted, runtime-attestation parsing passed, safe stop was confirmed, USB
cleanup was ready, protected modes and result/network seals passed, and no
public projection exists. Attempt-009 is consumed, no retry ran, STAT-001
remains `implemented`, and the checklist/progress history are unchanged. See
`docs/parity/work-plans/20260816T231527Z-STAT-001/CLOSURE.md`.

Residual risk and next safe action: the repeated boundary disproves the
sufficiency of the completed-event/effect feed correction but does not isolate
the remaining owner phase, subscription/reporting, clock-age, or scheduler
cause. A new immutable software-only plan must add closed diagnostics and a
production-shaped regression before any targeted correction. Attempt-010 is
not authorized without new verified progress and a new complete hardware
contract.

Campaign watchdog-policy correction plan:
`docs/parity/work-plans/20260816T235908Z-STAT-001/PLAN.md`

- [x] Reproduce the false 2,000-ms campaign rejection against the firmware's
      configured-timeout `feed_fresh` verdict.
- [x] Remove only the duplicate numeric consumer policy while preserving typed
      reason, participation, presence, per-window advancement, precedence, and
      value-free evidence checks.
- [x] Run focused and mandatory software, firmware-package, privacy,
      reference, parity-invariance, immutable-plan, and diff gates; commit and
      push a software-only closure without checklist transition or hardware.

Software-only authorization: local source, fixtures, tests, builds,
documentation, and ordinary git operations only. Do not access protected
attempts, credentials, detector/device/network runtime, private values, or a
public projection. Do not detect, flash, reset, monitor, mine, actuate, update,
erase, inject faults, manipulate power, use direct UART, or touch electrical
interfaces. Attempt-009 remains consumed; this plan does not authorize
attempt-010. STAT-001 remains `implemented` and the checklist/progress history
remain unchanged.

Campaign watchdog-policy correction completion review: source commit
`9e9d6545dbe4881f1cb81ca61da2c152dd791c9b` removes the duplicate host
2,000-ms freshness threshold. The real campaign regression failed before the
fix and passes afterward; producer `feed_fresh` remains accepted through the
compiled 5,000-ms timeout, producer `feed_stale` remains rejected after it,
and every focused and mandatory software gate passes. No firmware schema,
hardware behavior, checklist field, or progress-history entry changed. See
`docs/parity/work-plans/20260816T235908Z-STAT-001/CLOSURE.md`.

Residual risk and next safe action: the correction explains attempts 008 and
009 but does not itself prove the live twenty-window quorum. Attempt-010
requires a separate immutable hardware plan using this exact pushed source,
with the complete detector, evidence, privacy, recovery, retry, cleanup, and
stop contract. No hardware retry is authorized by this completed software
plan, so STAT-001 remains `implemented`.

Attempt-010 verification-promotion plan:
`docs/parity/work-plans/20260817T001716Z-STAT-001/PLAN.md`

- [x] Rebind the private-first workflow, Rust validator, generated contract,
      immutable task/plan admission, Bazel inputs, protected roots, and
      real-child fixtures from consumed attempt-009 to fresh attempt-010.
- [x] Run every focused and mandatory software, firmware, privacy, reference,
      package, and exact-source gate; commit and push before device access.
- [x] Run only the frozen detector and sole conditional attempt-010 command,
      then promote STAT-001 only if the complete independent quorum passes.

Attempt-010 progress basis and authorization: pushed correction
`9e9d6545dbe4881f1cb81ca61da2c152dd791c9b` removes the host campaign's
contradictory 2,000-ms feed-age policy while retaining the exact-package
producer's compiled 5,000-ms verdict and every structural watchdog check.
After this plan/task checkpoint and attempt rebind are clean, fully gated,
committed, pushed, and packaged, one exact board-205 package may be factory-
flashed/reset; ignored local Wi-Fi and pool inputs may be seeded privately;
and the repo-owned `live-share` conservative 400 MHz / 1,100 mV / 100% fan
profile may mine for exactly 600 accumulated active seconds while protected
current-session HTTP, WebSocket, and serial observations are joined. The
campaign must pause, safe-stop, release USB/process ownership, and may use at
most one supervisor-owned exact-package recovery flash after a post-flash
failure.

Attempt-010 units and safety: 1,100 mV is the ASIC core setpoint, while INA260
input bus voltage is independently measured in volts. Admission requires fresh
4.5-5.5 V input, at most 15 W, ASIC temperature below 75 C, and fresh nonzero
fan RPM after the 100% fan command. The two voltage domains must not be
compared. Stop on stale/unsafe state, ambiguous/non-205 detector identity,
source/reference/package drift, missing credentials, or cleanup/recovery
failure.

Attempt-010 evidence, privacy, recovery, and retry: only fresh ignored mode-
`0700` `scratch/stat001-hashrate-monitor/wrapper-010` and `attempt-010` roots
with mode-`0600` files are permitted. Credentials, pool/owner/worker fields,
origins, ports, USB/network/process identities, exact hashrates, sensors,
HTTP/WebSocket bodies, serial, commands, PIDs, and traces remain private. The
sole public aggregate projection is written only after independent validation.
Starting capture consumes attempt-010; preserve the earliest typed failure,
run bounded safe stop/recovery/cleanup after post-flash failures, and do not
retry unchanged or start attempt-011.

Attempt-010 prohibited effects and acceptance: no upstream-default/overclock
profile, arbitrary control target, automatic fan mode, unbounded mining, OTA,
erase, raw writes, fault injection, physical power action, direct UART, or
pin/pad/header/GPIO/probe/jumper/solder/signal manipulation is permitted.
Promotion requires exact clean source/reference/package/plan and detector
identity; one ASIC/four domains and one-second cadence; all twenty active
windows with work renewal; changing coherent positive HTTP/WebSocket current
and rolling rates; bounded error; watchdog failure `none`; terminal zero
current rate; safe stop, cleanup, protected modes, seals, independent
validation, and redaction. Any missing fact withholds promotion and leaves
STAT-001 `implemented`.

Attempt-010 completion review: exact pushed source/package
`495ad95d512546ed6c24d528204f779e88e3fdb2`, the pinned reference, every
focused and mandatory software/privacy gate, and one fresh detector passed.
The sole capture then repeated the sealed attempts-008/009 boundary after 14
of 20 required windows: `hardware_blocked` / `watchdog_unresponsive` /
producer-owned `watchdog_feed_stale`. Runtime identity was trusted,
runtime-attestation parsing was clean, safety remained valid, terminal HTTP
and WebSocket observations completed, safe stop was confirmed, USB cleanup
was ready, protected modes and all seals passed, and no public projection was
written. Attempt-010 is consumed, no retry ran, and progress policy selects
`stop_repeated_boundary`. STAT-001 remains `implemented`; checklist and
progress history are unchanged. See
`docs/parity/work-plans/20260817T001716Z-STAT-001/CLOSURE.md`.

Residual risk and next safe action: removing the duplicate host threshold
proves this recurrence is the device producer's stale verdict, but the closed
evidence does not distinguish producer timing, scheduler delay, checkpoint
publication, or subscription state. Do not start attempt-011. A future
software-only immutable plan may add new closed discriminators and a
production-shaped regression; hardware requires an objectively changed
authoritative boundary plus a new complete contract.

Owner-phase diagnostic and publication-cadence correction plan:
`docs/parity/work-plans/20260817T020911Z-STAT-001/PLAN.md`

- [x] Add a closed task-watchdog owner-phase discriminator from the firmware
      owner through runtime health, HTTP/WebSocket, sealed campaign evidence,
      and value-free private-first failure reporting.
- [x] Bound synchronous campaign-status retained/serial publication to one
      second while preserving per-event state tracking, first/terminal markers,
      safety, feedback, hashrate service, and maximum-gap requirements.
- [x] Prove 600-second high-event-rate cadence bounds, prompt terminal output,
      v13/v7 schema/seal behavior, source/evaluator binding, redaction, and all
      mandatory gates; push a software-only closure without transition.

Software-only authorization: local source, fixtures, tests, builds,
documentation, and ordinary git operations only. Do not access protected
attempts, credentials, detector/device/network runtime, private values, or
public projection candidates. Do not detect, flash, reset, monitor, mine,
actuate, update, erase, inject faults, manipulate power, use direct UART, or
touch electrical interfaces. Attempt-010 remains consumed and attempt-011 is
not authorized. STAT-001 remains `implemented`; checklist/progress are
unchanged.

Completion review: pushed implementation `edef059b` adds the phase
discriminator, cadence correction, v13/v7 sealed evidence, and all required
regressions without hardware access or a parity transition. Focused and full
mandatory gates pass. Residual risk is live-only: this correction has not yet
completed a detector-gated twenty-window campaign. A future attempt-011 needs
its own exact-source immutable contract; this closure does not authorize it.

Attempt-011 verification-promotion plan:
`docs/parity/work-plans/20260817T030355Z-STAT-001/PLAN.md`

- [x] Rebind the private-first workflow, Rust validator, generated contract,
      immutable task/plan admission, Bazel inputs, protected roots, v13/v7
      schemas, owner phase, and real-child fixtures from consumed attempt-010
      to fresh attempt-011.
- [x] Run every focused and mandatory software, firmware, privacy, reference,
      package, and exact-source gate; commit and push before device access.
- [x] Run only the frozen detector and sole conditional attempt-011 command,
      then promote STAT-001 only if the complete independent quorum passes.

Attempt-011 progress basis and authorization: exact pushed correction
`edef059bfc1d5dcc79f997c46fa022d8e1bd8ffc` bounds status publication and
adds a closed owner-phase discriminator after attempts 008-010 repeated the
same producer-stale boundary. After this plan/task checkpoint and the
attempt-011 rebind are clean, fully gated, committed, pushed, and packaged,
one exact board-205 package may be factory-flashed/reset; ignored local Wi-Fi
and pool inputs may be seeded privately; and the repo-owned `live-share`
conservative 400 MHz / 1,100 mV / 100% fan profile may mine for exactly 600
accumulated active seconds while protected current-session HTTP, WebSocket,
and serial observations are joined. The campaign must pause, safe-stop,
release USB/process ownership, and may use at most one supervisor-owned exact-
package recovery flash after a post-flash failure. The exact commands, privacy
layout, evidence joins, recovery behavior, retry bound, and stop outcomes are
frozen in the immutable plan above.

Attempt-011 units and safety: 1,100 mV is the ASIC core setpoint, while INA260
input bus voltage is independently measured in volts. Admission requires
fresh 4.5-5.5 V input, at most 15 W, ASIC temperature below 75 C, and fresh
nonzero fan RPM after the 100% fan command. Never compare the two voltage
domains. Stop on stale/unsafe state, ambiguous/non-205 detector identity,
source/reference/package drift, missing credentials, or cleanup/recovery
failure.

Attempt-011 evidence, privacy, recovery, and retry: only fresh ignored mode-
`0700` `scratch/stat001-hashrate-monitor/wrapper-011` and `attempt-011` roots
with mode-`0600` files are permitted. Credentials, pool/owner/worker fields,
origins, ports, USB/network/process identities, exact hashrates, sensors,
HTTP/WebSocket bodies, serial, commands, PIDs, and traces remain private. The
sole public aggregate projection is written only after independent validation.
Starting capture consumes attempt-011; preserve the earliest typed failure
and closed owner phase, run bounded safe stop/recovery/cleanup after post-
flash failures, and do not retry unchanged or start attempt-012.

Attempt-011 prohibited effects and acceptance: no upstream-default/overclock
profile, arbitrary control target, automatic fan mode, unbounded mining, OTA,
erase, raw writes, fault injection, physical power action, direct UART, or
pin/pad/header/GPIO/probe/jumper/solder/signal manipulation is permitted.
Promotion requires exact clean source/reference/package/plan and detector
identity; one ASIC/four domains and one-second cadence; all twenty active
windows with work renewal; changing coherent positive HTTP/WebSocket current
and rolling rates; bounded error; watchdog failure `none`; terminal zero
current rate; safe stop, cleanup, protected modes, seals, independent
validation, and redaction. Any missing fact withholds promotion and leaves
STAT-001 `implemented`.

Attempt-011 completion review: exact pushed source/package `43acffd3`, pinned
reference, focused/full gates, and one detector passed. The sole capture
failed closed after 5/20 windows at the new sealed signature
`watchdog_invalid_observation` / owner phase `waiting_inbox`; runtime identity,
attestation parsing, safety, terminal transports, pool persistence, safe stop,
cleanup, modes, seals, and redaction passed, and no public projection was
written. Attempt-011 is consumed, no retry ran, and terminal outcome is
`stop_hardware_blocker`. STAT-001 remains `implemented`; checklist/progress
are unchanged. See
`docs/parity/work-plans/20260817T030355Z-STAT-001/CLOSURE.md`.

Residual risk and next safe action: source tracing shows runtime health reads
evaluation time before copying concurrent watchdog observations, allowing a
new feed timestamp to overtake that earlier time and fail checked age
subtraction. Do not start attempt-012. A fresh software-only immutable plan
must add a controlled interleaving regression and make evaluation time no
earlier than the copied observations while preserving every existing guard.

Runtime-health snapshot-ordering correction plan:
`docs/parity/work-plans/20260817T035514Z-STAT-001/PLAN.md`

- [x] Move runtime-health evaluation-time sampling after copied supervisor,
      watchdog, and owner-phase observations; remove stale caller time input.
- [x] Prove the exact concurrent-feed interleaving, future-feed rejection,
      post-copy zero-age freshness, production source ordering, and caller
      ownership with behavior and source regressions.
- [x] Bind runtime-health core/adapter sources into the hashrate evaluator
      inventory and run every focused/mandatory gate; push a software-only
      closure without transition or hardware.

Software-only authorization: local source, fixtures, tests, builds,
documentation, and ordinary git operations only. Do not access protected
attempts, credentials, detector/device/network runtime, private values, or
public projection candidates. Do not detect, flash, reset, monitor, mine,
actuate, update, erase, inject faults, manipulate power, use direct UART, or
touch electrical interfaces. Attempt-011 remains consumed and attempt-012 is
not authorized. STAT-001 remains `implemented`; checklist/progress are
unchanged.

Snapshot-ordering completion review: exact pushed implementation `0b5338f6`
copies checkpoint/watchdog/phase observations before sampling evaluation time,
removes caller-supplied stale time, proves the old invalid and corrected fresh
interleavings, and expands the independently checked evaluator inventory to 15
sources. Focused and full mandatory gates pass. No hardware, credentials,
protected attempt, checklist, progress, or README change occurred. Residual
risk is live-only: the fix has not completed a detector-gated twenty-window
campaign. A future attempt-012 needs its own exact-source immutable contract;
this closure does not authorize it. See
`docs/parity/work-plans/20260817T035514Z-STAT-001/CLOSURE.md`.

Attempt-012 verification-promotion plan:
`docs/parity/work-plans/20260817T042626Z-STAT-001/PLAN.md`

- [x] Rebind the private-first workflow, validator, generated contract, task/
      plan admission, Bazel inputs, protected roots, v13/v7 schemas, owner
      phase, 15-source identity, and fixtures from attempt-011 to attempt-012.
- [x] Run every focused and mandatory software, firmware, privacy, reference,
      package, and exact-source gate; commit and push before device access.
- [x] Run only the frozen detector and sole conditional attempt-012 command,
      then promote STAT-001 only if the complete independent quorum passes.

Attempt-012 progress basis and authorization: exact pushed correction
`0b5338f6c1224dbdae6e664cd286e114ad611c6c` makes evaluation time postdate
copied watchdog facts and proves the attempt-011 invalid-observation race.
After this plan/task checkpoint and attempt rebind are clean, fully gated,
committed, pushed, and packaged, one exact board-205 package may be factory-
flashed/reset; ignored local Wi-Fi and pool inputs may be seeded privately;
and the repo-owned conservative 400 MHz / 1,100 mV / 100% fan profile may mine
for exactly 600 accumulated active seconds while protected current-session
HTTP, WebSocket, and serial observations are joined. The campaign must pause,
safe-stop, release ownership, and may use at most one supervisor-owned exact-
package recovery flash after a post-flash failure. Exact commands, privacy,
evidence, recovery, retry, and stop rules are frozen in the plan above.

Attempt-012 safety/privacy: 1,100 mV is the ASIC core setpoint; INA260 bus
input is independently measured in volts and must remain fresh 4.5-5.5 V with
at most 15 W, ASIC temperature below 75 C, and fresh nonzero fan RPM. Only
fresh ignored mode-`0700` wrapper/attempt-012 roots and mode-`0600` files are
permitted. Credentials, owner/worker fields, endpoints, identities, exact
hashrates/sensors, bodies, logs, commands, PIDs, and traces remain private.
The public projection is written only after independent validation.

Attempt-012 effects/retry/acceptance: starting capture consumes the ordinal;
preserve earliest failure and owner phase, run bounded safe stop/recovery/
cleanup, and do not retry unchanged or start attempt-013. No overclock,
arbitrary controls, unbounded mining, OTA, erase, raw write, fault injection,
physical power action, direct UART, or electrical manipulation is permitted.
Promotion requires exact source/reference/package/plan and detector identity,
one ASIC/four domains, one-second cadence, all twenty windows with work
renewal, changing coherent positive HTTP/WebSocket rates and warm rolling
windows, bounded error, watchdog `none`, terminal zero, safe stop, cleanup,
modes, seals, validation, and redaction. Missing facts withhold promotion.

Attempt-012 completion review: exact pushed source/package `ae094a1d`, pinned
reference, focused/full gates, and one detector passed. The sole capture
failed closed after 13/20 windows at the new sealed signature
`watchdog_feed_stale` / owner phase `waiting_inbox`; attempt-011's invalid-
observation race did not recur. Runtime identity, attestation, safety, terminal
transports, pool persistence, safe stop, cleanup, modes, seals, and redaction
passed, and no public projection was written. Attempt-012 is consumed, no
retry ran, and terminal outcome is `stop_hardware_blocker`. STAT-001 remains
`implemented`; checklist/progress are unchanged. See
`docs/parity/work-plans/20260817T042626Z-STAT-001/CLOSURE.md`.

Residual risk and next safe action: the owner feed became stale while phase
was `waiting_inbox`, whose requested timeout should be bounded by the
one-second readiness deadline. Current evidence cannot distinguish scheduler
starvation, timed-wait overrun, or task-priority/runtime behavior. Do not start
attempt-013. A fresh software-only plan must add coherent closed wait-entry,
deadline, and overrun diagnostics plus production-shaped scheduling tests
before any targeted fix or new hardware contract.

Waiting-inbox deadline diagnostic plan:
`docs/parity/work-plans/20260817T045834Z-STAT-001/PLAN.md`

- [x] Add coherent atomic receive-deadline ownership and closed runtime-health
      wait states for not-waiting, within-deadline, overrun, and invalid facts.
- [x] Carry the closed state through v14/v8 sealed campaign diagnostics with
      exact boundary, precedence, missing/unknown, redaction, and real-child
      regressions.
- [x] Pin and prove the ESP-IDF pthread priority-5 contract against upstream,
      bind new evaluator sources, and run every focused/mandatory gate; push a
      software-only closure without transition or hardware.

Software-only authorization: local source, fixtures, tests, builds,
documentation, and ordinary git operations only. Do not access protected
attempts, credentials, detector/device/network runtime, private values, or
public projection candidates. Do not detect, flash, reset, monitor, mine,
actuate, update, erase, inject faults, manipulate power, use direct UART, or
touch electrical interfaces. Attempt-012 remains consumed and attempt-013 is
not authorized. STAT-001 remains `implemented`; checklist/progress are
unchanged.

Wait-deadline diagnostic completion review: exact pushed implementation
`9604d145` atomically publishes a wrap-aware deadline before `waiting_inbox`,
derives four closed wait states after copying observations, carries them
through v14/v8 sealed evidence, pins upstream-aligned pthread priority 5, and
expands evaluator identity to 18 sources. Focused, real Xtensa, and all
mandatory gates pass. No hardware, credentials, protected attempt, checklist,
progress, or README change occurred. Residual risk is live-only: the state has
not yet classified the waiting-inbox stale-feed boundary on device. A future
attempt-013 requires its own exact-source immutable contract; this closure does
not authorize it. See
`docs/parity/work-plans/20260817T045834Z-STAT-001/CLOSURE.md`.

Attempt-013 verification-promotion plan:
`docs/parity/work-plans/20260817T054416Z-STAT-001/PLAN.md`

- [x] Rebind workflow/validator/generated contract/task-plan/Bazel/protected
      roots/fixtures from attempt-012 to attempt-013, preserving v14/v8,
      owner phase, wait state, priority 5, and 18-source identity.
- [x] Run focused/full software, firmware, privacy, reference, package, and
      exact-source gates; commit/push before device access.
- [x] Run only the frozen detector and sole conditional attempt-013 command;
      promote only if the complete independent quorum passes.

Attempt-013 authorization: exact pushed diagnostic `9604d145` classifies
waiting-inbox deadline state. After plan/rebind are clean, gated, committed,
pushed, and packaged, one exact board-205 package may be factory-flashed/reset,
privately seeded from ignored credentials, and run the conservative 400 MHz /
1,100 mV / 100% fan profile for exactly 600 active seconds with protected
serial/HTTP/WebSocket joins, pause, safe stop, cleanup, sealing, and at most one
supervisor recovery flash. Exact commands and stop rules are frozen in PLAN.

Safety/privacy/retry: input bus truth is fresh 4.5-5.5 V and <=15 W, ASIC temp
<75 C, fan RPM fresh/nonzero; core mV and bus volts remain distinct. Only fresh
ignored wrapper/attempt-013 0700 roots and 0600 files are allowed. Credentials,
endpoints, identities, exact values, bodies, logs, commands, PIDs, traces stay
private. Starting capture consumes ordinal; no unchanged retry or attempt-014.
No overclock, arbitrary controls, unbounded mining, OTA, erase, raw writes,
fault injection, power action, direct UART, or electrical manipulation.
Promotion requires exact identity, 20 windows/work renewal, coherent changing
rates, warm windows, bounded error, watchdog none, terminal zero, safe stop,
cleanup, modes, seals, validation, and redaction; missing facts withhold it.

Attempt-013 completion review: exact pushed source/package `43cc4178`, pinned
reference, all gates, and detector passed. Sole capture stopped after 12/20
windows with sealed `watchdog_feed_stale`, phase `waiting_inbox`, wait state
`within_deadline`; identity, safety, terminal state, cleanup, modes, seals, and
redaction passed, projection absent. Attempt consumed, no retry, outcome
`stop_hardware_blocker`; STAT-001/checklist/progress unchanged. See
`docs/parity/work-plans/20260817T054416Z-STAT-001/CLOSURE.md`.

Residual risk/next action: runtime health copies mutex feed history before
separately atomic phase/deadline, allowing old-feed/new-wait mixed snapshots.
Do not start attempt-014. A fresh software plan must add a bounded coherent
single-writer snapshot/seqlock and exact interleaving regression before any new
hardware contract.

Coherent watchdog-snapshot correction plan:
`docs/parity/work-plans/20260817T062043Z-STAT-001/PLAN.md`

- [x] Replace the separate feed-history and owner phase/deadline reads with one
      bounded coherent single-writer observation snapshot.
- [x] Regression-test the exact old-feed/new-wait interleaving, stable reads,
      retry exhaustion, and production runtime-health ownership.
- [x] Run focused firmware/package/privacy/reference checks and every mandatory
      repository gate; push a non-promotion closure with STAT-001 unchanged.

Software-only authorization: local source, tests, deterministic fixtures,
firmware/package builds, documentation, and Git operations only. Do not access
protected attempts, credentials, detector/device/USB/network runtime, private
values, or a public projection. Do not flash, reset, monitor, mine, actuate,
update, erase, inject faults, manipulate power, use direct UART or electrical
interfaces, retry attempt-013, or create/run attempt-014. This plan cannot
change STAT-001, checklist, progress, or README fields.

Coherent-snapshot completion review: pushed source `f5a8fd14` places feed
history, owner phase, and wait deadline behind one sequence-bracketed firmware
snapshot with eight bounded fail-closed retries. The exact old-feed/new-wait
interleaving is rejected then retried to the new coherent instant; stable,
retry-exhaustion, poison, ownership, evaluator, firmware/package, privacy,
reference, and every mandatory gate pass. No hardware or public evidence ran,
and STAT-001/checklist/progress remain unchanged. See the linked `CLOSURE.md`.

Residual risk/next action: live completion remains hardware-only. A separate
immutable plan may bind pushed `f5a8fd14` to a new exact package and authorize
one detector-gated attempt-014 with the full existing conservative-profile,
unit, safety, evidence, privacy, recovery, cleanup, retry, stop, and promotion
contract. Attempt-013 remains consumed and must not be retried.

Attempt-014 verification-promotion plan:
`docs/parity/work-plans/20260817T065250Z-STAT-001/PLAN.md`

- [x] Rebind the private-first workflow, validator, generated contract,
      immutable task/plan admission, Bazel inputs, fresh protected roots, and
      fixtures from attempt-013 to attempt-014 while preserving v14/v8,
      coherent watchdog snapshot, priority 5, and 18-source identity.
- [x] Run every focused and mandatory software, firmware, privacy, reference,
      package, and exact-source gate; commit and push before device access.
- [x] Run only the frozen detector and sole conditional attempt-014 command;
      promote only if the complete independent quorum passes.

Attempt-014 progress basis: pushed `f5a8fd14` materially fixes attempt-013's
exact `watchdog_feed_stale` / `waiting_inbox` / `within_deadline` mixed-
snapshot signature with a sequence-bracketed coherent firmware snapshot and
exact interleaving regression. This is a verified-fix continuation, not an
unchanged retry.

Attempt-014 authorization: after plan/rebind are clean, fully gated, committed,
pushed, and packaged, one exact board-205 package may be factory-flashed/reset,
privately seeded from ignored Wi-Fi/pool inputs, and run only at conservative
400 MHz / 1,100 mV ASIC core / 100% fan for exactly 600 active seconds with
protected serial/HTTP/WebSocket joins, pause, safe stop, cleanup, sealing, and
at most one supervisor-owned recovery flash. Exact commands are frozen in the
linked plan. Input-bus safety is independently fresh 4.5-5.5 V, <=15 W, ASIC
temperature <75 C, and fan RPM fresh/nonzero; core millivolts and bus volts
remain distinct.

Evidence/privacy/retry: use only fresh ignored wrapper-014 and attempt-014
mode-0700 roots with mode-0600 files. Credentials, endpoints, identities,
exact values, bodies, logs, commands, PIDs, and traces stay private; only the
independently valid aggregate projection may publish. Starting capture consumes
the ordinal; preserve earliest failure/phase/wait through safe stop, recovery,
sealing, and cleanup. Never retry attempt-014, reuse attempt-013, or start
attempt-015. No overclock, arbitrary controls, unbounded mining, OTA, erase,
raw writes, fault injection, power action, direct UART, BAP, or electrical
manipulation. Promotion requires exact identity, 20 windows/work renewal,
coherent changing positive rates, warm windows, bounded error, watchdog none,
terminal zero, safe stop, cleanup, modes, seals, validation, and redaction;
missing facts withhold it. Recurrence of attempt-013's exact signature selects
`stop_repeated_boundary`.

Attempt-014 completion review: exact clean pushed source/package `579f8315`,
the pinned reference, focused/full gates, and the sole protected detector
passed. The one capture did not repeat attempt-013's `watchdog_feed_stale`
signature. It failed closed after 302,436 active ms and 3/20 credited windows
as `hardware_blocked` / `watchdog_unresponsive` / `watchdog_unproved`, with
owner phase `waiting_inbox` and wait state `within_deadline`. Runtime identity
and attestation were trusted, safety and same-package state were valid, terminal
HTTP/WebSocket/pool state passed, safe stop was confirmed, USB cleanup was
ready, private modes and result/network digests passed, and no public projection
was written. Attempt-014 is consumed, no retry ran, terminal outcome is
`stop_hardware_blocker`, and STAT-001/checklist/progress remain unchanged. See
`docs/parity/work-plans/20260817T065250Z-STAT-001/CLOSURE.md`.

Residual risk/next action: `watchdog_unproved` means the projected coherent
snapshot had no admitted latest watchdog observation, but current closed
evidence cannot distinguish genuine pre-subscription state, eight-retry
exhaustion, poisoned history, publication-lifecycle loss, or transport
reconstruction. Do not start attempt-015. A new software-only immutable plan
must add a closed value-free coherent-read outcome across firmware/runtime
health and sealed evidence, reproduce the live-shaped transition, and apply a
targeted regression-backed fix before any further hardware ordinal.

Coherent-read diagnostic and earliest-tuple correction plan:
`docs/parity/work-plans/20260817T073552Z-STAT-001/PLAN.md`

- [x] Add stable/uninitialized/retry-exhausted/history-poisoned read outcomes
      from the firmware coherent store through runtime health and wire views.
- [x] Map read failures to exact fail-closed reasons and latch outcome, owner
      phase, and wait state with the earliest watchdog failure so terminal
      samples cannot overwrite the diagnostic tuple.
- [x] Rotate private campaign result/network evidence to v15/v9, add the
      attempt-014-shaped regression, and pass every focused/mandatory gate.

Software-only authorization: local source, tests, deterministic fixtures,
firmware/package builds, documentation, and Git operations only. Do not access
protected attempts, credentials, detector/device/USB/network runtime, private
values, or public projection candidates. Do not flash, reset, monitor, mine,
actuate, update, erase, inject faults, manipulate power, use external UART/BAP,
touch electrical interfaces, retry attempt-014, or create/run attempt-015.
STAT-001/checklist/progress/README remain unchanged.

Coherent-read diagnostic completion review: pushed source `c3b0dcb9` adds the
closed `stable`, `uninitialized`, `retry_exhausted`, and `history_poisoned`
outcomes from the firmware store through HTTP/WebSocket/retained health and
private campaign result v15/network v9. Retry exhaustion and poison now map to
distinct fail-closed reasons instead of generic `unproved`. The campaign latches
read outcome, owner phase, and wait state with the earliest watchdog failure;
the attempt-014-shaped regression proves later terminal samples cannot overwrite
that tuple. Generated contracts, 18-source identity, real firmware/package,
privacy/reference, file-length, and every mandatory gate pass. No hardware or
public projection ran, and STAT-001/checklist/progress remain unchanged. See
the linked `CLOSURE.md`.

Residual risk/next action: attempt-014's actual coherent-read outcome remains
unknowable because its v14/v8 evidence predates the discriminator and carried a
mixed diagnostic tuple. A separate immutable hardware plan may consider one
fresh attempt-015 only as a progress-backed diagnostic of the corrected v15/v9
boundary, with the complete existing detector, unit, safety, privacy, recovery,
cleanup, retry, stop, and promotion contract. Never reuse attempt-014 or infer
its hidden cause.

Attempt-015 verification-promotion plan:
`docs/parity/work-plans/20260817T082220Z-STAT-001/PLAN.md`

- [x] Rebind the private-first workflow, validator, generated contract,
      immutable task/plan admission, Bazel inputs, fresh protected roots, and
      fixtures from attempt-014 to attempt-015 while preserving result v15,
      network v9, coherent read outcome, earliest tuple, and 18-source identity.
- [x] Run every focused and mandatory software, firmware, privacy, reference,
      package, and exact-source gate; commit and push before device access.
- [x] Run only the frozen detector and sole conditional attempt-015 command;
      promote only if the complete independent quorum passes.

Attempt-015 progress basis: pushed `c3b0dcb9` fixes attempt-014's invalid mixed
diagnostic tuple and adds stable/uninitialized/retry-exhausted/history-poisoned
outcomes with exact failure labels through v15/v9 sealed evidence. This is a
verified-fix continuation at the real evidence boundary, not an unchanged
retry; attempt-014's older tuple cannot establish recurrence.

Attempt-015 authorization: after plan/rebind are clean, fully gated, committed,
pushed, and packaged, one exact board-205 package may be factory-flashed/reset,
privately seeded from ignored Wi-Fi/pool inputs, and run only at conservative
400 MHz / 1,100 mV ASIC core / 100% fan for exactly 600 active seconds with
protected serial/HTTP/WebSocket joins, pause, safe stop, cleanup, sealing, and
at most one supervisor-owned recovery flash. Exact commands are frozen in the
linked plan. Input-bus safety remains independently fresh 4.5-5.5 V, <=15 W,
ASIC temperature <75 C, and fan RPM fresh/nonzero; core mV and bus V remain
distinct.

Evidence/privacy/retry: only fresh ignored wrapper-015 and attempt-015 mode-
0700 roots with mode-0600 files are allowed. Credentials, endpoints,
identities, exact values, bodies, logs, commands, PIDs, and traces stay private;
only the independently valid projection may publish. Starting capture consumes
the ordinal; preserve earliest failure/read-outcome/phase/wait through safe
stop, recovery, sealing, and cleanup. Never retry attempt-015, reuse attempt-
014, or start attempt-016. No overclock, arbitrary controls, unbounded mining,
OTA, erase, raw write, fault injection, power action, direct UART/BAP, or
electrical manipulation. Promotion requires exact identity, stable read
outcome, 20 windows/work renewal, coherent changing positive rates, warm
windows, bounded error, watchdog none, terminal zero, safe stop, cleanup,
modes, seals, validation, and redaction; missing facts withhold it.

Attempt-015 completion review: exact clean pushed source/package `1892800b`,
the pinned reference, focused/full gates, and the sole protected detector
passed. The single capture produced the first trustworthy v15/v9 tuple after
364,110 active ms and 12/20 credited windows: `hardware_blocked` /
`watchdog_unresponsive` / `watchdog_feed_stale` / read outcome `stable` /
owner phase `handling_inbox` / wait state `not_waiting`. Runtime identity and
attestation were trusted; safety, same-package state, terminal HTTP/WebSocket/
pool state, safe stop, USB cleanup, private modes, seals, and redaction passed;
the public projection is absent. Attempt-015 is consumed, no retry ran,
terminal outcome is `stop_hardware_blocker`, and STAT-001/checklist/progress
remain unchanged. See
`docs/parity/work-plans/20260817T082220Z-STAT-001/CLOSURE.md`.

Residual risk/next action: the stable read rules out store retry exhaustion,
poison, and uninitialized state, while `handling_inbox/not_waiting` rules out the
receive wait. Current evidence cannot distinguish inbox-to-event mapping,
session evaluation, or one blocking effect before its completion feed. Do not
start attempt-016. A new software-only immutable plan must add a closed value-
free inbox/feedback/effect subphase, reproduce the post-window stale-feed
transition, and apply a targeted regression-backed correction before hardware.

Owner-work subphase and entry-feed correction plan:
`docs/parity/work-plans/20260817T090156Z-STAT-001/PLAN.md`

- [x] Add one closed, value-free owner subphase for inbox mapping, session
      evaluation, and every production effect category, stored coherently with
      phase/wait and projected through runtime-health surfaces.
- [x] Feed at session-evaluation and effect-execution entry while retaining
      completion feeds, and regress the attempt-015-shaped inherited stale-age
      boundary without masking a genuinely blocking operation.
- [x] Latch subphase with the earliest private campaign watchdog tuple, rotate
      result/network evidence to v16/v10, and pass every focused/mandatory gate.

Software-only authorization: local source, tests, deterministic fixtures,
firmware/package builds, documentation, and Git operations only. Do not access
protected attempts, credentials, detector/device/USB/network runtime, private
values, or public projection candidates. Do not flash, reset, monitor, mine,
actuate, update, erase, inject faults, manipulate power, use external UART/BAP,
touch electrical interfaces, retry attempt-015, or create/run attempt-016.
STAT-001/checklist/progress/README remain unchanged.

Owner-work correction completion review: pushed source `177fffe9` adds the
closed value-free inbox/evaluation/effect subphase, copies it in the coherent
watchdog snapshot, projects it through runtime-health and private evidence, and
latches it with the earliest watchdog tuple. Entry feeds now reset inherited
age before session evaluation and effect execution while the regression proves
a genuinely blocking effect still becomes stale. Private campaign schemas are
v16/v10; generated contracts, 18-source identity, real firmware/package,
redaction/reference, file-length, and every mandatory gate pass. No hardware or
public projection ran, and STAT-001/checklist/progress remain unchanged. See
the linked `CLOSURE.md`.

Residual risk/next action: attempt-015 predates the subphase and its exact
blocking boundary remains unknowable. Do not retry it. A separate immutable
hardware plan may consider one fresh attempt-016 only from the clean pushed
correction, with the complete detector, conservative profile, safety, privacy,
recovery, cleanup, retry, stop, and promotion contract. Any precise blocking
subphase must receive a targeted diagnosis and regression-backed correction
before another continuation.

Attempt-016 verification-promotion plan:
`docs/parity/work-plans/20260817T095432Z-STAT-001/PLAN.md`

- [x] Rebind the private-first workflow, validator, generated contract,
      immutable task/plan admission, Bazel inputs, fresh protected roots, and
      fixtures from attempt-015 to attempt-016 while preserving result v16,
      network v10, coherent owner subphase, earliest tuple, and 18-source
      identity.
- [x] Run every focused and mandatory software, firmware, privacy, reference,
      package, and exact-source gate; commit and push before device access.
- [x] Run only the frozen detector and sole conditional attempt-016 command;
      promote only if the complete independent quorum passes.

Attempt-016 progress basis: pushed `177fffe9` corrects attempt-015's ambiguous
`watchdog_feed_stale/stable/handling_inbox/not_waiting` boundary by adding one
closed inbox/evaluation/effect subphase through coherent v16/v10 evidence and
feeding at handler/effect entry without masking genuinely blocking work. This
is a verified-fix continuation at a materially more discriminating boundary,
not an unchanged retry.

Attempt-016 authorization: after plan/rebind are clean, fully gated, committed,
pushed, and packaged, one exact board-205 package may be factory-flashed/reset,
privately seeded from ignored Wi-Fi/pool inputs, and run only at conservative
400 MHz / 1,100 mV ASIC core / 100% fan for exactly 600 active seconds with
protected serial/HTTP/WebSocket joins, pause, safe stop, cleanup, sealing, and
at most one supervisor-owned recovery flash. Exact commands are frozen in the
linked plan. Input-bus safety remains independently fresh 4.5-5.5 V, <=15 W,
ASIC temperature <75 C, and fan RPM fresh/nonzero; core mV and bus V remain
distinct.

Evidence/privacy/retry: only fresh ignored wrapper-016 and attempt-016 mode-
0700 roots with mode-0600 files are allowed. Credentials, endpoints,
identities, exact values, bodies, logs, commands, PIDs, and traces stay private;
only the independently valid projection may publish. Starting capture consumes
the ordinal; preserve earliest failure/read-outcome/phase/subphase/wait through
safe stop, recovery, sealing, and cleanup. Never retry attempt-016, reuse
attempt-015, or start attempt-017. No overclock, arbitrary controls, unbounded
mining, OTA, erase, raw write, fault injection, power action, direct UART/BAP,
or electrical manipulation. Promotion requires exact identity, stable read
outcome, 20 windows/work renewal, coherent changing positive rates, warm
windows, bounded error, watchdog none, terminal zero, safe stop, cleanup,
modes, seals, validation, and redaction; missing facts withhold it.

Attempt-016 completion review: exact clean pushed source/package `223d10bc`,
the pinned reference, focused/full gates, and the sole protected detector
passed. The one capture failed closed after 364,314 active ms and 4/20 windows
as `hardware_blocked` / `watchdog_unresponsive` /
`watchdog_snapshot_retry_exhausted`, with coherent read outcome
`retry_exhausted`, owner phase/subphase `unavailable/unavailable`, and wait
state `not_waiting`. Runtime identity, attestation, safety, terminal HTTP/
WebSocket/pool state, safe stop, USB cleanup, protected modes, seals, and
redaction passed; the public projection is absent. Attempt-016 is consumed, no
retry ran, terminal outcome is `stop_hardware_blocker`, and STAT-001/checklist/
progress remain unchanged. See the linked `CLOSURE.md`.

Residual risk/next action: active owner work now makes adjacent subphase and
feed publications, while the coherent reader has only eight immediate spin
retries. The precise outcome is consistent with continuous publication
contention and must be reproduced in software without inferring a private
effect. Do not start attempt-017. A fresh software-only plan must add the exact
contention regression and a bounded targeted writer/reader correction that
still fails closed for a genuinely stuck publication before any new hardware
contract.

Coherent-publication contention correction plan:
`docs/parity/work-plans/20260817T104623Z-STAT-001/PLAN.md`

- [x] Reproduce finite odd-sequence writer preemption and continuous sequence
      contention against the exact eight-attempt coherent reader.
- [x] Fuse owner-entry subphase plus watchdog observation into one publication
      and yield between bounded retries without weakening stuck-writer failure.
- [x] Preserve every v16/v10 diagnostic, 18-source identity, firmware/package,
      privacy/reference, and mandatory gate; close without hardware/promotion.

Software-only authorization: local source, deterministic tests, builds,
documentation, and Git operations only. Do not access protected attempts,
credentials, detector/device/USB/network runtime, private values, or public
projection candidates. Do not flash, reset, monitor, mine, actuate, update,
erase, inject faults, manipulate power, use external UART/BAP, touch electrical
interfaces, retry attempt-016, or create/run attempt-017. STAT-001/checklist/
progress/README remain unchanged.

Coherent-publication correction completion review: pushed source `c274be94`
fuses owner-entry subphase plus optional watchdog observation into one seqlock
publication and yields between the unchanged eight coherent-read attempts.
Finite odd-writer preemption now recovers after one scheduler handoff; a
permanently odd writer and continuous sequence changes still return exact
`retry_exhausted`; poison, stable, uninitialized, phase/wait, unavailable-feed,
v16/v10, earliest-tuple, and value-free behavior remain intact. Focused tests,
18-source identity, generated contracts, real firmware/package, redaction/
reference, file-length, and every mandatory gate pass. No hardware or public
projection ran, and STAT-001/checklist/progress remain unchanged. See the
linked `CLOSURE.md`.

Residual risk/next action: the correction has not yet been observed under live
mining load. A separate immutable hardware plan may consider one fresh
attempt-017 only from clean pushed `c274be94`, with the complete detector,
conservative profile, unit, safety, privacy, recovery, cleanup, retry, stop,
and promotion contract. Never reuse attempt-016; any new precise boundary must
receive its own regression-backed correction before another continuation.

Attempt-017 verification-promotion plan:
`docs/parity/work-plans/20260817T114224Z-STAT-001/PLAN.md`

- [x] Rebind workflow, validator, contract, task/plan, Bazel, roots, and
      fixtures from attempt-016 to 017, preserving v16/v10, fused writer,
      bounded reader, earliest tuple, and 18-source identity.
- [x] Pass focused/full gates, commit/push, and rebuild exact package before
      device access.
- [x] Run only the frozen detector and sole capture; promote only on the full
      independently validated quorum.

Attempt-017 progress basis: pushed `c274be94` fixes attempt-016's exact retry-
exhausted tuple with fused entry publication and scheduler-aware bounded reads;
finite contention recovers and stuck contention remains fail-closed.

Authorization/privacy/retry: after clean pushed gates, one exact board-205
package may run 400 MHz / 1,100 mV core / 100% fan for 600 active seconds with
protected joins, safe stop, cleanup, sealing, and one bounded recovery flash.
Fresh bus safety is 4.5-5.5 V, <=15 W, ASIC <75 C, fan RPM nonzero; core mV and
bus V remain distinct. Only fresh ignored wrapper/attempt-017 0700 roots and
0600 files; no private publication. No retry/018, overclock, arbitrary control,
OTA, erase, fault injection, power action, direct UART/BAP, or electrical work.
Exact attempt-016 tuple recurrence selects `stop_repeated_boundary`.

Attempt-017 completion review: exact clean pushed source/package `b6d560b6`,
pinned reference, focused/full gates, and sole detector passed. The capture
failed closed after 314,248 active ms and 0/20 windows with exact repeated tuple
`watchdog_snapshot_retry_exhausted/retry_exhausted/unavailable/unavailable/
not_waiting`. Identity, attestation, safety, terminal HTTP/WebSocket/pool,
safe stop, cleanup, modes, seals, and redaction passed; projection is absent.
Attempt-017 is consumed, no retry ran, and the terminal outcome is
`stop_repeated_boundary`. STAT-001/checklist/progress remain unchanged. See the
linked `CLOSURE.md`.

Terminal blocker: do not create attempt-018 or continue this hardware-attempt
lineage. Pushed `c274be94` targeted the exact attempt-016 boundary and attempt-
017 repeated it unchanged. Any future STAT-001 work must establish a materially
different source-level diagnosis and authority contract; direct UART, pins,
ad hoc electrical work, and ordinal-only retries remain prohibited.

Temporary user-authorized watchdog diagnostic loop | 2026-08-17:

- Formal parity plan files and attempt records are temporarily waived for this
  STAT-001 debugging loop. Each hardware run still requires a distinct clean
  pushed code fix, exact rebuilt package, fresh detector, absent private child,
  and repo-owned command; no unchanged reruns are allowed.
- Private roots use `scratch/stat001-watchdog-debug/run-N` with mode 0700 and
  mode-0600 files. Credentials, admitted port, endpoints, identities, sensor/
  rate values, logs, bodies, commands, PIDs, and traces never print or commit.
  No public projection is produced.
- Each run uses only board 205, `live-share`, conservative 400 MHz / 1,100 mV
  core / 100% fan, 600 active seconds, fresh bus 4.5-5.5 V, <=15 W, ASIC <75 C,
  and nonzero fan RPM. The campaign owns flash, seed, observation, safe stop,
  cleanup, sealing, and at most one exact-package recovery flash.
- Allowed effects remain factory flash/reset, ignored Wi-Fi/pool seed, bounded
  conservative mining, safe stop, cleanup, and recovery. No overclock,
  arbitrary control, OTA, erase, fault injection, physical power action,
  external UART/BAP, pins, pads, probes, jumpers, solder, or electrical work.
- Stop immediately on identity/safety/cleanup/recovery/privacy failure. After a
  failed run, inspect only sealed allowlisted categories and apply a materially
  targeted source fix before another run. End the exception once the watchdog
  boundary is cleared or a non-software/human blocker is proven.
- Exception ended after exact-package run-010 on pushed source `e70cefa7`
  completed 600,216 active milliseconds with all 20 continuity windows,
  accepted submit evidence, stable/valid watchdog state, trusted identity,
  fresh safety, confirmed safe stop, terminal HTTP/WebSocket/pool joins, ready
  USB cleanup, and redaction. The private diagnostic campaign remains ignored
  local evidence and was not promoted into the public parity checklist.

Attempt-018 audited verification-promotion plan:
`docs/parity/work-plans/20260818T022212Z-STAT-001/PLAN.md`

- [x] Rebind the immutable hashrate evidence workflow, plan digest, protected
      roots, ordinal, fixtures, source identity, result v16, and network v11 to
      fresh attempt-018 without changing production behavior.
- [x] Pass every focused and mandatory software/firmware gate, commit and push
      the rebind, and build the exact package before hardware access.
- [x] Run only the plan's frozen detector and sole conditional capture; promote
      `STAT-001` only if the complete independent quorum passes.

Attempt-018 authority: the materially different source path is established by
the serialized watchdog snapshot, per-action shutdown feeds, full-duration
live-share lease, closed correlation diagnostics, and exact terminal-horizon
handling. After clean pushed gates, one exact board-205 package may run at the
conservative 400 MHz / 1,100 mV ASIC core / 100% fan profile for exactly 600
active seconds with ignored local Wi-Fi/pool inputs, protected joins, safe stop,
cleanup, sealing, and at most one supervisor-owned exact-package recovery
flash. Fresh input bus safety remains 4.5-5.5 V, <=15 W, ASIC <75 C, and fan
RPM nonzero. Core millivolts and bus volts remain distinct.

Attempt-018 privacy/retry/stop: only fresh ignored wrapper-018 and attempt-018
mode-0700 roots with mode-0600 files are allowed. Credentials, endpoints,
identities, exact values, bodies, serial, commands, PIDs, and traces stay
private; only an independently valid projection may publish. Starting capture
consumes the ordinal. Do not retry it or start attempt-019. Stop on any nonzero
command, detector/identity/safety/privacy/cleanup/recovery failure, or missing
quorum fact. No overclock, arbitrary controls, OTA, erase, fault injection,
physical power action, external UART/BAP, or electrical manipulation is
authorized.

Attempt-018 completion review: exact clean pushed source/package `e14b98d5`,
the pinned reference, immutable plan, focused/full gates, and sole detector
passed. The one capture failed closed after 273,286 active ms and 9/20 windows
as `hardware_blocked` / `network_correlation_failed`, with stable/valid
watchdog, fresh safety, ready USB cleanup, mixed session/ordinal identity, and
the first mixed reset category `panic`. Work renewal, terminal joins, and safe
stop did not establish the complete same-session horizon. The independently
validated projection is absent, attempt-018 is consumed, and no retry ran.
`STAT-001`, checklist, progress history, and README remain unchanged. See the
linked `WORKLOG.md` and `CLOSURE.md`.

Next safe action: do not start attempt-019. A fresh software-only `STAT-001`
plan must add a redaction-safe panic discriminator, reproduce or isolate the
source-level cause, and apply a targeted regression-backed fix before any new
hardware authority contract.

Panic-diagnostic software plan:
`docs/parity/work-plans/20260818T040753Z-STAT-001/PLAN.md`

- [x] Add a complete value-free ESP-IDF/Rust panic-signature and closed task-
      family classifier at the existing complete-line serial analyzer boundary.
- [x] Rotate private diagnostics and bind every reachable reducer/vocabulary
      source into the hashrate evaluator identity and Bazel runfiles.
- [x] Add behavior/privacy/schema/drift/real-process regressions, run every
      mandatory gate, and close without hardware or parity promotion.

Authorization is software-only: source, fixtures, tests, builds, documentation,
and Git. Do not inspect raw/private attempt data, access credentials or device
runtime, flash, reset, monitor, mine, actuate, update, erase, inject faults,
manipulate power, use external UART/BAP, touch electrical interfaces, reuse
attempt-018, or create/run attempt-019. `STAT-001`, checklist, progress history,
and README remain unchanged under this plan.

Panic-diagnostic completion review: pushed source `0abd10ad` classifies seven
closed panic signatures and twelve task families from complete serial lines,
retains only first closed labels plus a saturating recognized-line count, and
maps a panic reboot without an observed signature to `unknown`. Private
diagnostics rotate to v4 and retain no raw line, task, address, backtrace, or
payload. The hashrate evaluator identity now binds all 21 reachable sources via
package-local runfiles and real-process fixtures. Focused tests, ordered Rust
checks, 382 flash tests, firmware/package, Bright Builds, redaction/reference,
full Bazel, parity, progress, and diff gates pass. No hardware, credentials,
protected attempt data, or projection ran; `STAT-001`, checklist, progress, and
README remain unchanged. See the linked `WORKLOG.md` and `CLOSURE.md`.

Next safe action: a separate immutable hardware plan may consider one fresh
attempt-019 only from clean pushed `0abd10ad`, with diagnostic v4 and the
21-source identity fully rebound plus the complete detector, conservative
profile, safety, privacy, recovery, cleanup, retry, stop, and promotion
contract. Never retry by ordinal alone; any recurrence must select diagnosis
from the new closed panic signature/task/count tuple.

Attempt-019 verification-promotion plan:
`docs/parity/work-plans/20260818T050654Z-STAT-001/PLAN.md`

- [x] Rebind plan/task/root/ordinal/fixture surfaces to fresh attempt-019 while
      preserving result v16, network v11, diagnostic v4, and 21-source identity.
- [x] Pass focused/full gates, commit and push the exact source/package, then
      run only the frozen detector and sole conditional capture.
- [x] Promote only on the complete independent quorum; otherwise preserve
      `implemented`, the earliest closed panic/watchdog/correlation tuple,
      evidence withholding, safe stop, cleanup, recovery, and no attempt-020.

Attempt-019 progress basis: pushed `0abd10ad` adds the previously absent
value-free discriminator for attempt-018's mixed-session `panic` boundary.
Seven panic signatures, twelve task families, a recognized-line count, and an
unknown fallback now flow through private diagnostic v4; all 21 reachable
sources are identity-bound. This materially changes the observable failure
boundary and is not an ordinal-only retry.

Authorization/privacy/retry: after clean pushed gates, one exact board-205
package may run 400 MHz / 1,100 mV ASIC core / 100% fan for 600 active seconds
with protected joins, safe stop, cleanup, sealing, and one bounded recovery
flash. Fresh input-bus safety remains 4.5-5.5 V, <=15 W, ASIC <75 C, and fan
RPM nonzero; core mV and bus V are distinct. Only fresh ignored wrapper/attempt-
019 mode-0700 roots and mode-0600 files may hold private values. No retry/020,
overclock, arbitrary control, OTA, erase, fault injection, physical power
action, direct UART/BAP, or electrical work is authorized. Exact commands and
promotion/stop conditions are frozen in the linked plan.

Attempt-019 completion review: exact pushed implementation `7d78889a`, pinned
reference, immutable plan, focused/full gates, and one detector passed. The
sole capture completed 600 active seconds with 20/20 renewed windows, one
BM1366/four-domain 1,000 ms sampling, coherent changing positive HTTP and
WebSocket rates, warm windows, terminal zeros, submit evidence, stable
watchdog, no panic or mixed session, trusted identity, fresh safety, terminal
joins, confirmed safe stop, complete cleanup, protected modes, no rerun, and
passed redaction. Independent projection validation passed; committed evidence
`d58f0ade` and transition `20260818T060031Z-STAT-001` move only `STAT-001` to
`verified`. Progress is 77/94 active rows, 81.9%. See the linked `RESULT.md`
and `docs/parity/evidence/stat001-hashrate-monitor/hashrate-monitor-projection.json`.

Residual risks/non-claims: profitability, absolute laboratory-calibrated
accuracy, arbitrary profiles or pools, other ASICs/boards, unbounded mining,
overclocking, fault injection, OTA, release readiness, external UART/BAP, and
electrical-interface behavior remain separate.

### task-parity-safe10-prerequisite-readiness | 2026-08-18 | Verify live production prerequisite readiness

- [x] Add a typed private-first SAFE-10 projector and independent validator
      over preserved accepted attempt-003 evidence.
- [x] Bind exact detector/seal/digests, live required/fresh observations,
      readiness transition, accepted work, 20/20 continuity, safety/watchdog,
      safe stop/cleanup, and attempt-to-current source compatibility.
- [x] Pass all gates, run the sole software projection command, and promote only
      on complete independently validated closed evidence.

Plan: `docs/parity/work-plans/20260818T122819Z-SAFE-10/PLAN.md`

Authorization: read-only classification of protected attempt-003, repository
source/tests/builds, one closed public projection, docs, Git commit, and push.
No detector execution, credentials, device/USB/network runtime, flash, monitor,
mining, restart, recovery, new hardware attempt, external UART/BAP, pins, or
electrical work. Protected artifacts remain immutable and raw/private values
must never enter the projection, Git, logs, or summaries.

Closure review: implementation `cf772601` added the typed contract/projector and
`1c0ad96d` added the missing validator runfiles dependency. All focused/full
gates pass. The sole projection command failed before candidate creation as
`process_failed` because the validator was initially absent from specialized
binary runfiles; no projection was emitted and protected evidence was unchanged.
The runfile is now verified executable, but this plan prohibited retry, so
`SAFE-10` remains `implemented`. See `WORKLOG.md` and `CLOSURE.md`.

Next safe action: a fresh software-only plan may run the corrected exact
projection command once, independently validate it, and promote only on the
complete closed quorum. No hardware or external state is required.

Projection retry plan:
`docs/parity/work-plans/20260818T132739Z-SAFE-10/PLAN.md`

- [x] Rotate only the immutable plan binding; preserve schema, sources,
      protected inputs, prerequisite semantics, privacy, and promotion quorum.
- [x] Pass focused/full gates, commit/push, and run the sole corrected software
      projection command with absent candidate/projection admission.
- [x] Independently validate, commit evidence/RESULT, promote only SAFE-10,
      sync progress, archive this task, final-gate, and push.

Authorization: repository source/tests/builds, read-only protected attempt-003
classification, one public closed projection, docs, Git commit, and push only.
No detector execution, credentials, device/USB/network runtime, flash, monitor,
mining, restart, recovery, new attempt, external UART/BAP, pins, or electrical
work.

Evidence review: exact clean pushed source `01d5cdcb`, the preserved sealed
attempt-003 bundle, 19-path current inventory, nine byte-identical production
paths, two reference paths, independent Rust validation, mode checks, and direct
redaction review all passed. The closed projection proves all five required/
fresh prerequisite classes, ready unblocked production transition, 600,746
active ms, accepted work, 20/20 windows, terminal settlement, safe stop, and
cleanup. See this plan's `RESULT.md` and `WORKLOG.md`.

Completion review: evidence commit `2394f995`, transition
`20260818T132739Z-SAFE-10`, and synchronized progress promote only `SAFE-10` to
`verified` with `unit,workflow,hardware-smoke,hardware-regression`. The accepted
projection is
`docs/parity/evidence/safe10-prerequisite-readiness/safe10-projection.json`.
Residual risks remain limited to SAFE-11 blocker labels, fault injection,
individual active-control policies, self-test, arbitrary telemetry, other
ASICs/boards, arbitrary profiles/pools, unbounded mining, OTA, recovery, and
release readiness.

### task-parity-safe11-fail-closed-reasons | 2026-08-18 | Verify production fail-closed reason propagation

- [x] Add an exhaustive current production blocker-to-runtime-to-API regression
      covering exact labels, fail-closed state, uniqueness, and redaction.
- [x] Reconcile the stale Phase 22 ledger with the real production readiness,
      runtime-state, API, and pinned-reference boundaries.
- [x] Bind accepted SAFE-10 live prerequisite proof to current source evidence,
      run every required gate, and promote only `SAFE-11` on the complete quorum.

Plan: `docs/parity/work-plans/20260818T135714Z-SAFE-11/PLAN.md`

Plan closure review: pre-implementation source review found that the frozen
promotion rule incorrectly required `OperatorPaused` to project
`safe_blocked`. Production intentionally disables work submission while
projecting that one operator-controlled state as `paused`; failure reasons use
`safe_blocked`. No implementation or parity transition ran under this plan.
See the linked `CLOSURE.md`.

Next safe action: create a fresh immutable SAFE-11 plan with two exact classes:
operator pause remains work-blocked/paused with no API failure reason, while
every failure blocker remains work-blocked/safe-blocked with its exact
redaction-safe API reason.

Corrected verification plan:
`docs/parity/work-plans/20260818T140738Z-SAFE-11/PLAN.md`

- [x] Preserve operator pause as work-blocked/paused without an API failure
      reason and prove all sixteen failure variants work-blocked/safe-blocked
      with their exact API reason.
- [x] Bind the corrected current-source evidence to accepted SAFE-10 live
      prerequisite proof, then promote only on the complete privacy-safe quorum.

Authorization: local source, tests, documentation, builds, and Git only. No
protected input, detector, credentials, device/USB/network runtime, flash,
monitor, mining, restart, recovery, hardware attempt, fault injection, external
UART/BAP, pins, or electrical work.

Promotion requires the exact closed production reason vocabulary to remain
work-blocked, safe-blocked, API-visible, unique, and redaction-safe; the accepted
SAFE-10 detector-gated board-205 projection to validate independently; current
source/reference and privacy evidence to pass; and all mandatory gates to pass.
Fault injection, individual active controls, self-test, other boards/ASICs,
unbounded mining, OTA/recovery, and release readiness remain non-claims.

Verification review: implementation `0fee4942` adds four focused API regressions
covering all 17 typed labels, one operator-paused state, sixteen fail-closed
failure states, exact API reasons, ready-state clearing, uniqueness, and
redaction-safe spelling. Focused Cargo/Bazel tests, the ordered Cargo gates,
Bright Builds, all 47 Bazel targets, independent SAFE-10 validation, reference
cleanliness, source-inventory preservation, redaction review, file-size, parity,
progress, and diff checks passed. Evidence commit `f0e4ea45` binds the accepted
source/reference and non-claims in the corrected plan's `RESULT.md` and
`docs/parity/evidence/safe11-production-blocker-reasons/summary.md`.

Completion review: transition `20260818T144926Z-SAFE-11` promotes only
`SAFE-11` to `verified` with
`unit,workflow,hardware-smoke,hardware-regression`; synchronized progress is
79/94 active rows (84.0%). Residual risks remain live fault injection,
individual active-control effects, self-test, BAP/UART, other boards/ASICs,
arbitrary profiles/pools, unbounded mining, OTA/recovery, and release readiness.

### task-parity-cfg07-runtime-credentials | 2026-08-18 | Verify runtime-only credential handling

- [x] Add a typed public-only CFG-07 projector and independent validator over
      committed accepted same-chain mining evidence.
- [x] Bind required/forwarded local Wi-Fi and pool inputs, accepted live mining,
      safe stop, current source compatibility, and zero committed values without
      opening credential files or protected attempt artifacts.
- [x] Update the Phase 30 artifact for CFG-07 only, run every gate, and promote
      only on complete independently validated redacted evidence.

Plan: `docs/parity/work-plans/20260818T150603Z-CFG-07/PLAN.md`

Authorization: local source/tests/docs/builds, committed-public evidence reads,
one public projection, and Git only. No credential-file or protected-attempt
access, detector, device/USB/network runtime, flash, monitor, mining, restart,
recovery, hardware attempt, fault injection, external UART/BAP, pins, or
electrical work.

Promotion requires exact public evidence for local-owner-supplied runtime
inputs, accepted same-chain mining, safe stop, attempt/current source
compatibility, no committed credential values or raw artifacts, Phase 30 CFG-07
structured proof, independent validation, redaction, and all mandatory gates.
STR-09, ASIC-11, credential contents, rotation/persistence, arbitrary profiles/
pools, other boards/ASICs, unbounded mining, OTA/recovery, and release readiness
remain non-claims.

Verification review: implementation `04ecfab5` adds the typed Rust contract,
independent validator, public-only projector, 17-path evaluator/source identity,
seven-path attempt/current semantic compatibility, CLI/Just/Bazel wiring, and
real-validator regressions. The sole projection independently validates at mode
0644 and proves required/forwarded local inputs, accepted same-chain mining,
accepted submit, safe stop, cleanup, no committed credential values, no raw
artifacts, and passed redaction. The canonical Phase 30 artifact now admits
CFG-07 only; focused and mandatory software/firmware/package/privacy/reference
gates all pass. See this plan's `RESULT.md`, `WORKLOG.md`, and
`docs/parity/evidence/cfg07-runtime-credentials/summary.md`.

Completion review: evidence commit `a5118afb` and transition
`20260818T160014Z-CFG-07` promote only `CFG-07` to `verified` with
`unit,workflow,hardware-smoke,hardware-regression`; synchronized progress is
80/94 active rows (85.1%). Residual risks remain credential contents and
rotation/persistence, STR-09, ASIC-11, arbitrary profiles/pools, active-control
effects, self-test, BAP/UART, other boards/ASICs, unbounded mining,
OTA/recovery, and release readiness.

### task-parity-asic09-mode-separation | 2026-08-18 | Verify BM1366 diagnostic/production separation

- [x] Independently validate the accepted initialization, work-send,
      result-parsing, and serial-transport hardware projections.
- [x] Bind current diagnostic admission and production executor separation
      tests to the accepted live production chain.
- [x] Produce exact source-bound evidence, run every gate, and promote only
      `ASIC-09` on the complete redacted quorum.

Plan: `docs/parity/work-plans/20260818T160811Z-ASIC-09/PLAN.md`

Authorization: local tests/docs/builds, committed-public evidence reads, and
Git only. No credential or protected-attempt access, detector,
device/USB/network runtime, flash, monitor, mining, restart, recovery, hardware
attempt, fault injection, external UART/BAP, pins, or electrical work.

Promotion requires independently validated exact-package live initialization,
production-ready-gated work, qualified result, accepted response, retained
production UART, safe stop, cleanup, current mode-separation source/tests, and
redaction. Adjacent ASIC/Stratum rows, arbitrary diagnostics, active controls,
other boards/ASICs, arbitrary pools/profiles, unbounded mining, OTA/recovery,
and release readiness remain non-claims.

Verification review: plan commit `7f8ca3bb` and evidence commit `db698524`
join independently validated ASIC-002, ASIC-003, ASIC-004, and ASIC-005
projections into one same-attempt live production chain. Current adapter-gate
tests prove fail-closed diagnostic admission, production-command tests prove
production-only work/result variants, and the production executor source
contains no diagnostic-work command. Reference verification and `just package`
passed. See this plan's `RESULT.md`, `WORKLOG.md`, and
`docs/parity/evidence/asic09-mode-separation/summary.md`.

Completion review: evidence commit `db698524` and transition
`20260819T145312Z-ASIC-09` promote only `ASIC-09` to `verified` with
`unit,golden,workflow,hardware-smoke,hardware-regression`; synchronized
progress is 81/94 active rows (86.2%). Residual risks remain arbitrary
diagnostic builds, frequency transitions, voltage/fan/thermal behavior,
nonzero version-mask or multi-midstate breadth, arbitrary-load serial
behavior, other ASICs/boards, arbitrary pools/profiles, unbounded mining,
OTA/recovery, ASIC-10, ASIC-11, ASIC-12, STR-08, STR-09, and release
readiness.

### task-parity-asic10-work-registry | 2026-08-19 | Verify pool-derived BM1366 work registry

- [x] Independently validate the accepted initialization and work-send
      hardware projections.
- [x] Bind current production-work registry and production-session tests to
      the accepted live production chain.
- [x] Produce exact source-bound evidence, run every gate, and promote only
      `ASIC-10` on the complete redacted quorum.

Plan: `docs/parity/work-plans/20260819T150619Z-ASIC-10/PLAN.md`

Authorization: local tests/docs/builds, committed-public evidence reads, and
Git only. No credential or protected-attempt access, detector,
device/USB/network runtime, flash, monitor, mining, restart, recovery, hardware
attempt, fault injection, external UART/BAP, pins, or electrical work.

Promotion requires independently validated exact-package live pool-derived
work registration, production dispatch, qualified result, accepted response,
safe stop, cleanup, current registry source/tests, and redaction. Adjacent
ASIC/Stratum/safety rows, live clean-jobs or reconnect, active controls, other
boards/ASICs, arbitrary pools/profiles, unbounded mining, OTA/recovery, and
release readiness remain non-claims.

Verification review: plan commit `9a57318a` and evidence commit `934cf3dc`
join independently validated ASIC-002 and ASIC-003 projections into one
same-attempt live production chain. Current production-work tests prove
enqueue, dispatch context, generation advance, clean-jobs, reconnect, and
redaction. Production-session tests bind ASIC effects to generation and
valid-job context. Reference verification and `just package` passed. See this
plan's `RESULT.md`, `WORKLOG.md`, and
`docs/parity/evidence/asic10-work-registry/summary.md`.

Completion review: evidence commit `934cf3dc` and transition
`20260819T150848Z-ASIC-10` promote only `ASIC-10` to `verified` with
`unit,golden,workflow,hardware-smoke,hardware-regression`; synchronized
progress is 82/94 active rows (87.2%). Residual risks remain
result-correlation ownership, submit classification, frequency transitions,
voltage/fan/thermal behavior, nonzero version-mask or multi-midstate breadth,
live clean-jobs or reconnect, other ASICs/boards, arbitrary pools/profiles,
unbounded mining, OTA/recovery, ASIC-11, ASIC-12, STR-08, STR-09, SAFE-12,
SAFE-13, and release readiness.

### task-parity-asic11-result-correlation | 2026-08-19 | Verify BM1366 result correlation before submit

- [x] Independently validate the accepted initialization, work-send, and
      result-parsing hardware projections.
- [x] Bind current production-work correlation and production-session tests to
      the accepted live production chain.
- [x] Produce exact source-bound evidence, run every gate, and promote only
      `ASIC-11` on the complete redacted quorum.

Plan: `docs/parity/work-plans/20260819T151339Z-ASIC-11/PLAN.md`

Authorization: local tests/docs/builds, committed-public evidence reads, and
Git only. No credential or protected-attempt access, detector,
device/USB/network runtime, flash, monitor, mining, restart, recovery, hardware
attempt, fault injection, external UART/BAP, pins, or electrical work.

Promotion requires independently validated exact-package live qualified result
correlation before submit intent, accepted response, safe stop, cleanup,
current correlation source/tests, and redaction. Adjacent ASIC/Stratum/safety
rows, submit classification ownership, rejected-share hardware, share-hash or
network-target policy beyond the accepted qualified result, live clean-jobs or
reconnect, active controls, other boards/ASICs, arbitrary pools/profiles,
unbounded mining, OTA/recovery, and release readiness remain non-claims.

Verification review: plan commit `bbbf390d` and evidence commits `69b6f4eb`
and `9cf0ca65` join independently validated ASIC-002 through ASIC-004
projections into one same-attempt live production chain and record the exact
Phase 30 ASIC-11 structured proof. Current production-work tests prove submit
intent only for the current generation and active job, and fail-close
uncorrelated, stale, duplicate, generation-mismatched, and drifted-target
results. Production-session tests bind ASIC effects to generation and
valid-job context. Reference verification and `just package` passed. See this
plan's `RESULT.md`, `WORKLOG.md`, and
`docs/parity/evidence/asic11-result-correlation/summary.md`.

Completion review: evidence commit `9cf0ca65` and transition
`20260819T151924Z-ASIC-11` promote only `ASIC-11` to `verified` with
`unit,golden,workflow,hardware-smoke,hardware-regression`; synchronized
progress is 83/94 active rows (88.3%). Residual risks remain submit-response
classification ownership, share-reject hardware, frequency transitions,
voltage/fan/thermal behavior, nonzero version-mask or multi-midstate breadth,
share-hash or network-target policy beyond the accepted qualified result, live
clean-jobs or reconnect, other ASICs/boards, arbitrary pools/profiles,
unbounded mining, OTA/recovery, ASIC-12, STR-08, STR-09, SAFE-12, SAFE-13, and
release readiness.

### task-parity-asic12-fail-closed-redaction | 2026-08-19 | Verify BM1366 production blocker redaction

- [x] Move exact production status rendering into the host-testable ASIC core
      and cover every public state and fail-closed blocker reason.
- [x] Preserve the firmware logging surface while proving closed labels, safe
      disabled-state fields, and redacted work/result/target/submit context.
- [x] Validate the accepted ASIC-002 through ASIC-005 projection chain and
      promote only `ASIC-12` when its full current-source and hardware-backed
      evidence quorum passes.

Plan: `docs/parity/work-plans/20260820T041751Z-ASIC-12/PLAN.md`

Authorization: local source edits, tests, committed-public evidence reads,
documentation, build/package, Git commit, and push only. No credentials,
protected attempt roots, detector, USB/device/network runtime, flash, monitor,
mining, restart, recovery, hardware attempt, fault injection, external UART,
BAP accessory, pins, or electrical work.

Evidence and promotion contract: bind the accepted ASIC-002 initialization,
ASIC-003 work-send, ASIC-004 result-parsing, and ASIC-005 serial-transport
projections to current exact fail-closed rendering and redaction tests. Public
evidence may contain only paths, commits, digests, closed labels, booleans,
counts, and command outcomes. Promote only on independent projection validation,
current source/tests, accepted-share hardware bridge proof, safe stop, cleanup,
redaction, and all mandatory gates. Hardware fault injection for every blocker,
other ASICs/boards, arbitrary loads/pools/profiles, active safety controls,
OTA/recovery, unbounded mining, and release readiness remain non-claims.

Verification: implementation source `30e0340695e1f307dfcdc7aa6949da07beb616f5`
moves exact public production status rendering into the pure ASIC core while
the firmware shell preserves its info/warning levels. Eleven production ASIC,
21 production-work, and 70 production-session tests pass. All four accepted
projection validators pass with matching digests and mode `0644`; the ordered
Rust gates, managed checks, reference verification, and current Ultra 205
package build pass. The source-bound summary is
`docs/parity/evidence/asic12-fail-closed-redaction/summary.md`.

Completion review: implementation commit `30e03406`, evidence commit
`d0b5df4b`, and transition `20260820T044331Z-ASIC-12` promote only `ASIC-12`
to `verified` with
`unit,golden,workflow,hardware-smoke,hardware-regression`; synchronized
progress is 84/94 active rows (89.4%). Residual risks remain hardware fault
injection for every blocker, arbitrary diagnostic builds, nonzero version-mask
or multi-midstate breadth, arbitrary-load serial behavior, rejected-share
hardware, frequency transitions, voltage/fan/thermal behavior, other
ASICs/boards, arbitrary pools/profiles, unbounded mining, OTA/recovery,
STR-08, STR-09, SAFE-12, SAFE-13, and release readiness.

### task-parity-str08-live-socket-lifecycle | 2026-08-19 | Verify the live Stratum socket lifecycle

- [x] Independently validate the accepted STR-001 socket and STR-006
      coordinator projections.
- [x] Bind current live-runtime, production-session, and transport-loopback
      tests to the accepted socket lifecycle.
- [x] Produce exact source-bound evidence, run every gate, and promote only
      `STR-08` on the complete redacted quorum.

Plan: `docs/parity/work-plans/20260820T045045Z-STR-08/PLAN.md`

Authorization: local tests, committed-public evidence reads, documentation,
build/package, Git commit, and push only. No credentials, protected attempt
roots, detector, USB/device/network runtime, flash, monitor, mining, restart,
recovery, hardware attempt, fault injection, external UART/BAP, pins, or
electrical work.

Promotion requires independently validated exact-package live socket success,
authorization before ASIC dispatch, accepted-share hardware, ordered safe stop,
cleanup, current lifecycle/transport tests, and redaction. Hardware fallback or
reconnect, exact upstream timeout/keepalive equivalence, DNS/IP-family parity,
arbitrary pools, TLS, Stratum v2, rejected-share hardware, unbounded stability,
other boards, updates/recovery, and release readiness remain non-claims.

Verification: source `8f86924a34e3988da15b0bc6b274ecd1c3806c21`
joins independently validated STR-001 and STR-006 projections with matching
digests and mode `0644`. Forty-six live-runtime tests, 70 production-session
tests, and the firmware production-transport loopback target pass. The ordered
Rust gates, managed checks, reference verification, and current Ultra 205
package build pass. The source-bound summary is
`docs/parity/evidence/str08-live-socket-lifecycle/summary.md`.

Completion review: plan commit `8f86924a`, evidence commit `d4f8c4de`, and
transition `20260820T045751Z-STR-08` promote only `STR-08` to `verified` with
`unit,workflow,hardware-smoke,hardware-regression`; synchronized progress is
85/94 active rows (90.4%). Residual risks remain fallback or reconnect on
hardware, exact upstream timeout or keepalive equivalence, DNS/IP-family parity,
arbitrary pools, TLS, Stratum v2, rejected-share hardware, unbounded stability,
other boards, updates/recovery, profitability, STR-09, SAFE-12, SAFE-13, and
release readiness.

### task-parity-str09-submit-response-classification | 2026-08-20 | Verify live submit-response classification

- [x] Independently validate the accepted STR-001 socket, STR-006 coordinator,
      and ASIC-004 result-correlation projections.
- [x] Bind current submit-response, live-runtime, and production-session tests
      to the accepted hardware response.
- [x] Add exact STR-09 proof to the canonical Phase 30 artifact and its
      current-artifact regression, then promote only `STR-09` on the complete
      redacted quorum.

Plan: `docs/parity/work-plans/20260820T050854Z-STR-09/PLAN.md`

Authorization: local source/tests, committed-public evidence reads,
documentation, build/package, Git commit, and push only. No credentials,
protected attempt roots, detector, USB/device/network runtime, flash, monitor,
mining, restart, recovery, hardware attempt, fault injection, external
UART/BAP, pins, or electrical work.

Promotion requires an accepted hardware share classified only from matching
current-generation ASIC-derived submit intent, plus exact-package identity,
safe stop, cleanup, current tests, independent validation, Phase 30 structured
proof, and redaction. Rejected-share hardware, stale/mismatched response paths
on hardware, fallback/reconnect hardware, arbitrary pools, TLS, Stratum v2,
unbounded mining, other boards/ASICs, updates/recovery, and release readiness
remain non-claims.

Verification: implementation source `532ab568228312157b3164820d9ad9f9ae221dbf`
adds the three exact STR-09 Phase 30 fields and requires all current promotions
in the checked-in artifact regressions. Six submit-response, 46 live-runtime,
and 70 production-session tests pass, as does the parity target. All three
accepted projection validators pass with matching digests and mode `0644`; the
ordered Rust gates, managed checks, reference verification, and current Ultra
205 package build pass. The source-bound summary is
`docs/parity/evidence/str09-submit-response-classification/summary.md`.

Completion review: implementation commit `532ab568`, evidence commit
`0e9e1abf`, and transition `20260820T052508Z-STR-09` promote only `STR-09` to
`verified` with `unit,workflow,hardware-smoke,hardware-regression`;
synchronized progress is 86/94 active rows (91.5%). Residual risks remain
rejected-share hardware, stale/mismatched response paths on hardware,
fallback/reconnect hardware, arbitrary pools, TLS, Stratum v2, unbounded
mining, other boards/ASICs, updates/recovery, profitability, SAFE-12, SAFE-13,
and release readiness.

### task-parity-safe12-production-safe-stop | 2026-08-20 | Verify production mining safe stop

- [x] Independently validate the accepted SAFE-10, STR-006, PWR-002, and
      PWR-003 hardware projections.
- [x] Bind current production-session safe-stop and firmware actuation/status/
      owner-progress tests to the accepted live stop.
- [x] Produce exact source-bound evidence, run every gate, and promote only
      `SAFE-12` on the complete redacted quorum.

Plan: `docs/parity/work-plans/20260820T052841Z-SAFE-12/PLAN.md`

Authorization: local tests, committed-public evidence reads, documentation,
build/package, Git commit, and push only. No credentials, protected attempt
roots, detector, USB/device/network runtime, flash, monitor, mining, restart,
recovery, hardware attempt, fault injection, external UART/BAP, pins, or
electrical work.

Promotion requires detector-gated live safety hardware proof for the complete
ordered production stop, disabled mining/control/submission state, consumed
lease, terminal confirmation, cleanup, current tests, independent validation,
and redaction. Fault-injected hardware stop, electrical timing/waveforms,
power-loss interruption, automatic thermal/fan fault recovery, arbitrary
profiles/pools, other boards/ASICs, unbounded mining, OTA/recovery, and release
readiness remain non-claims.

Verification: source `308f312f63951daceb2e49ead2a515e979e91453`
joins independently validated SAFE-10, STR-006, PWR-002, and PWR-003
projections with matching digests and mode `0644`. Eight safe-stop tests, 70
production-session tests, and three focused firmware targets pass. The ordered
Rust gates, managed checks, reference verification, and current Ultra 205
package build pass. The source-bound summary is
`docs/parity/evidence/safe12-production-safe-stop/summary.md`.

Completion review: plan commit `308f312f`, evidence commit `5224ef05`, and
transition `20260820T054247Z-SAFE-12` promote only `SAFE-12` to `verified` with
`unit,workflow,hardware-smoke,hardware-regression`; synchronized progress is
87/94 active rows (92.6%). Residual risks remain fault-injected hardware stop,
electrical timing/waveforms, power-loss interruption, automatic thermal/fan
fault recovery, arbitrary profiles/pools, other boards/ASICs, unbounded mining,
OTA/recovery, SAFE-13, and release readiness.

### task-parity-safe13-live-watchdog-responsiveness | 2026-08-20 | Verify watchdog responsiveness under live load

- [x] Independently validate the accepted SAFE-10, STR-006, and runtime-health
      projections.
- [x] Bind current watchdog/runtime-health/session tests and firmware owner-
      progress/checkpoint observation targets to the accepted live campaign.
- [x] Produce exact source-bound evidence, run every gate, and promote only
      `SAFE-13` on the complete redacted quorum.

Plan: `docs/parity/work-plans/20260820T054935Z-SAFE-13/PLAN.md`

Authorization: local tests, committed-public evidence reads, documentation,
build/package, Git commit, and push only. No credentials, protected attempt
roots, detector, USB/device/network runtime, flash, monitor, mining, restart,
recovery, hardware attempt, fault injection, external UART/BAP, pins, or
electrical work.

Promotion requires detector-gated live safety hardware proof for watchdog
validity through all bounded campaign windows, current subscription/feed/
progress semantics, fresh participating observation, non-regressing sequences,
healthy supervisor checkpoints, safe stop, cleanup, independent validation,
and redaction. Deliberate starvation/stalls on hardware, actual watchdog reset/
recovery, unbounded load, every firmware task, other boards/ASICs, fault
injection, OTA/recovery, and release readiness remain non-claims.

Verification: source `57dba7b6673e5a25e28c5b1b4db83662d91735f3`
joins independently validated SAFE-10, STR-006, and runtime-health projections
with matching digests and mode `0644`. Six watchdog, 27 runtime-health, and 70
production-session tests pass, as do the focused firmware progress/checkpoint/
observation targets. The ordered Rust gates, managed checks, reference
verification, and current Ultra 205 package build pass. The source-bound
summary is
`docs/parity/evidence/safe13-live-watchdog-responsiveness/summary.md`.

Completion review: plan commit `57dba7b6`, evidence commit `bb15b6d8`, and
transition `20260820T055801Z-SAFE-13` promote only `SAFE-13` to `verified` with
`unit,workflow,hardware-smoke,hardware-regression`; synchronized progress is
88/94 active rows (93.6%). Residual risks remain deliberate starvation/stalls
on hardware, actual watchdog reset/recovery, unbounded load, every firmware
task, other boards/ASICs, fault injection, OTA/recovery, and release readiness.

### task-parity-stat003-scoreboard | 2026-08-04 | Implement production scoreboard

- [x] Add the exact stable top-20 valid-nonce scoreboard and bounded indexed
      persistence codec with focused regression coverage.
- [x] Carry one redacted candidate from current-generation nonce correlation
      through a typed production-session effect without changing submit policy.
- [x] Add transactional indexed-NVS ownership, boot load, read-only API
      projection, production ownership tests, and every mandatory gate.

Plan: `docs/parity/work-plans/20260804T220000Z-STAT-003/PLAN.md`

Authorization: local software, synthetic nonce/work fixtures, and build work
only. No hardware attempt, credentials, external service, mining campaign, pool
connection, frequency/voltage/fan/power effect, OTA, recovery, direct UART, or
pins.

Verification: Ten focused API tests, three production-session tests, five
firmware ownership tests, the complete Cargo suite, real firmware build, Bright
Builds checks, all 33 Bazel tests, parity validation/progress, redaction,
reference cleanliness, and diff checks passed on the implementation tree.

Completion review: Software implementation is complete at
`0f3d46a77f5b2492880921cf524bc052d2283bc4` and the typed transition
`20260804T225500Z-STAT-003` records `implemented` with
`unit,workflow,api-compare`. Live nonce difficulty, device persistence, API and
browser behavior, mining, and accepted/rejected share outcomes remain below
verified, so this task remains active rather than archived.

Verification-promotion plan:
`docs/parity/work-plans/20260818T064430Z-STAT-003/PLAN.md`

- [x] Add the missing `/scoreboard` operator route and complete private-first
      Rust/TypeScript evidence contract with source-identity and real-process
      regressions.
- [x] Pass focused/full gates, commit and push the exact source/package, then
      run only the frozen detector and sole conditional attempt-001 capture.
- [ ] Promote only on the complete causal mining/API/UI/restart durability
      quorum; otherwise preserve `implemented`, evidence withholding, safe
      stop, cleanup, recovery policy, and no attempt-002.

Attempt-001 authorization: after clean pushed gates, one exact board-205
package may run the conservative 400 MHz / 1,100 mV core / 100% fan campaign
for 600 active seconds. Its normal owner Wi-Fi/pool NVS seed intentionally
replaces prior settings and scoreboard records before mining; the final state
keeps the new scoreboard, owner inputs, package defaults, and safe stop. One
passive receive-only capture and one normal HTTP restart may prove live API/UI
and boot durability. Private roots, recovery, retry, stop, promotion, privacy,
and prohibited effects are frozen in the linked plan.

Attempt-001 closure review: exact pushed source/package `a337babc`, pinned
reference, focused/full gates, one detector, and the sole capture passed
admission. The campaign completed 600,320 active ms, 20/20 renewed windows,
204 scoreboard candidates, accepted submit, trusted identity, fresh safety,
stable watchdog, no panic/mixed reset, terminal HTTP/WebSocket/pool joins,
confirmed safe stop, ready cleanup, modes, seals, and redaction. It failed
closed as `terminal_state_unconfirmed` because the latest marker reported
`campaign_lease_consumed` without authoritative campaign state `consumed`.
No scoreboard API/SPA/restart step or public projection ran; no retry is
authorized, and `STAT-003` remains `implemented`. See the linked `CLOSURE.md`.

Next safe action: a fresh software-only plan must reproduce and correct the
lease-reason/non-consumed-state terminal publication/handoff boundary without
accepting reason-only terminal state or weakening safe stop. The wrapper's
missing typed `hardware_blocked` classification is corrected under this plan,
but a new hardware ordinal requires a separately gated post-fix plan.

Terminal-settlement correction plan:
`docs/parity/work-plans/20260818T082357Z-STAT-003/PLAN.md`

- [x] Add a closed terminal-settlement reducer and reproduce complete terminal
      transports racing the coordinator's final analyzer handoff.
- [x] Request serial closure before finalization, accept/fail only after the
      final handoff, and preserve every earlier failure and safety gate.
- [x] Rotate network evidence/consumers to v12, pass all gates, commit/push,
      and close without hardware, attempt-002, or parity promotion.

Refined diagnosis: the sealed attempt-001 observation contains final campaign
state `consumed`; the earlier closure's reason-without-state statement applied
to the concurrent network worker snapshot, not the analyzer's final marker.
The worker returned failed evidence before coordinator final handoff could
settle that snapshot. This plan is software-only and does not access protected
values or authorize another device effect.

Terminal-settlement correction review: source
`ca42d7de79ee250161904f1ae14f1bc2ff833324` adds one pure reducer, makes
terminal transport quorum/deadline request capture closure, waits for the
coordinator's final analyzer handoff before acceptance or failure, preserves
earlier failures, and rotates network evidence plus hashrate/scoreboard
consumers to v12 closed diagnostics. Focused tests, the ordered Cargo gates,
Bright Builds, all 47 Bazel test targets, firmware build/package, redaction,
reference, parity, and progress checks passed. No hardware or protected input
was used; `STAT-003` remains `implemented`. See this plan's `WORKLOG.md` and
`CLOSURE.md`.

Next safe action: create a fresh exact-source/package hardware plan before any
attempt-002. Preserve the detector, privacy, recovery, retry, stop, and full
scoreboard API/SPA/restart promotion contract; do not treat this software
correction as live parity evidence.

Attempt-002 verification plan:
`docs/parity/work-plans/20260818T090846Z-STAT-003/PLAN.md`

- [x] Rotate the private-first scoreboard evidence workflow from consumed
      attempt-001 to fresh attempt-002 and bind the new immutable plan, task,
      ordinal, paths, runfiles, contracts, fixtures, and validators.
- [x] Pass focused/full gates, commit and push the exact implementation, build
      the bound package, and run only wrapper-002 detector admission.
- [x] Run at most one conditional attempt-002 and promote only on the complete
      accepted mining/API/SPA/restart persistence quorum; otherwise preserve
      the earliest closed failure and stop without attempt-003.

Attempt-002 authorization: after clean pushed gates, one exact board-205
package may repeat the conservative 400 MHz / 1,100 mV core / 100% fan campaign
for 600 accumulated active seconds because the prior terminal race now has a
targeted verified correction. Its NVS seed intentionally clears earlier
scoreboard records and retains the owner-supplied Wi-Fi/pool inputs and the new
safe-stopped scoreboard. One passive same-origin capture and one normal HTTP
restart may prove UI/API and boot durability. The linked plan freezes private
roots, exact commands, recovery, retry, stop, promotion, privacy, and prohibited
effects; starting the capture consumes attempt-002 and no attempt-003 is
authorized.

Attempt-002 closure review: exact pushed/package-bound source `e9034ea1`, the
pinned reference, all focused/full gates, 31-path evaluator identity, one
detector, and the sole capture passed admission. The sealed campaign completed
600,148 active ms, 20/20 renewed windows, 202 scoreboard candidates, accepted
submit, trusted identity, fresh safety, stable watchdog, no panic/mixed reset,
terminal HTTP/WebSocket/pool joins, final consumed serial state, confirmed safe
stop, and ready cleanup. It failed public `evidence_invalid` because network v12
reached `accepted_after_serial_close` after natural analyzer closure while the
model incorrectly required `terminal_close_requested=true`. API/SPA/restart and
public projection remained withheld; no retry ran and `STAT-003` remains
`implemented`. See this plan's `WORKLOG.md` and `CLOSURE.md`.

Next safe action: a software-only plan must make closure initiator diagnostic,
not acceptance truth; false remains a valid closed boolean when final consumed,
serial-finished, accepted settlement and every transport/safety gate pass. Only
a later immutable exact-package plan may authorize attempt-003 after that
targeted correction is fully verified and pushed.

Natural-closure correction plan:
`docs/parity/work-plans/20260818T095707Z-STAT-003/PLAN.md`

- [x] Prove natural analyzer closure with final consumed state and complete
      terminal quorum is valid when `terminal_close_requested=false`.
- [x] Remove only the closure-initiator truth requirement, retain the field as
      mandatory closed diagnostics, and update both evidence consumers.
- [x] Pass focused/full gates, commit/push, and close without detector,
      credentials, device access, attempt-003, or parity promotion.

Authorization: software, deterministic child processes, firmware/package
builds, docs, and Git only. Attempt-002 remains immutable failed evidence. No
detector, credentials, protected attempt roots, USB/device/network runtime,
flash, monitor, mining, restart, public projection, attempt-003, recovery,
external UART/BAP, pins, or electrical work is authorized by this plan.

Natural-closure correction review: source `9da1d2c3` removes only the worker-
request initiator from network acceptance truth while retaining its v12 boolean
diagnostic. Rust and real-child regressions prove worker-requested true and
analyzer-natural false acceptance, missing/non-boolean rejection, final-state
requirements, and both hashrate/scoreboard consumers. Focused and mandatory
gates, all 47 Bazel targets, firmware/package, redaction, reference, parity, and
progress checks passed. No hardware or protected input was used; `STAT-003`
remains `implemented`. See this plan's `WORKLOG.md` and `CLOSURE.md`.

Next safe action: a fresh immutable hardware plan may rotate to attempt-003
only after exact package/source binding and must reuse the complete detector,
safety, privacy, recovery, cleanup, API/SPA/restart, and promotion contract. A
recurrence of the same closed boundary after this fix stops further retries.

Attempt-003 verification plan:
`docs/parity/work-plans/20260818T102038Z-STAT-003/PLAN.md`

- [x] Rotate the private-first scoreboard evidence workflow from consumed
      attempt-002 to fresh attempt-003 and bind the immutable plan, task,
      ordinal, paths, runfiles, contracts, fixtures, and validators.
- [x] Pass focused/full gates, commit/push the exact implementation, build the
      bound package, and run only wrapper-003 detector admission.
- [x] Run at most one conditional attempt-003 and promote only on the complete
      accepted mining/API/SPA/restart persistence quorum; otherwise preserve
      the earliest closed failure and stop without attempt-004.

Attempt-003 authorization: the exact attempt-002 failure now has a targeted,
production-shaped, fully gated and pushed correction. After clean pushed gates,
one exact board-205 package may repeat the conservative 400 MHz / 1,100 mV core /
100% fan campaign for 600 accumulated active seconds. Its NVS seed intentionally
clears earlier scoreboard records and retains owner-supplied Wi-Fi/pool inputs
plus the new safe-stopped scoreboard. One passive same-origin capture and one
normal HTTP restart may prove UI/API and boot durability. The linked plan
freezes private roots, commands, recovery, retry, stop, promotion, privacy, and
prohibited effects; starting capture consumes attempt-003 and no attempt-004 is
authorized.

Attempt-003 closure review: exact source/package `60a56d49`, pinned reference,
focused/full gates, one detector, and the sole capture passed admission. The
sealed campaign completed 600,746 active ms, accepted network v12, 20/20
renewed windows, 175 candidates, accepted submit, trusted identity, fresh
safety, stable watchdog, natural final closure, terminal transport/pool joins,
safe stop, and cleanup. The private 20-entry scoreboard repeated identically
and the live SPA route passed. The exact restart changed session, incremented
ordinal once, reported `software_cpu`, and kept boot mining disabled, but the
verifier rejected closed non-active `paused` because it hardcodes only
`safe_blocked`. Later scoreboard reads and projection remained withheld; no
retry ran and `STAT-003` remains `implemented`. See this plan's `WORKLOG.md`
and `CLOSURE.md`.

Next safe action: a software-only plan must centralize disabled boot mining as
false boot intent plus either closed non-active state (`paused` or
`safe_blocked`), reject active/unknown/enabled shapes, and retain every exact
restart, identity, persistence, safety, privacy, and source gate. Only a later
immutable plan may authorize attempt-004 after that fix is fully verified and
pushed; recurrence of the same signature stops further retries.

Stopped-state verifier correction plan:
`docs/parity/work-plans/20260818T112336Z-STAT-003/PLAN.md`

- [x] Define disabled boot mining once as false boot intent plus closed
      non-active `paused` or `safe_blocked`, rejecting active/unknown/enabled.
- [x] Use the predicate in restart admission and final evidence, with pure and
      full real-child paused-restart regression coverage.
- [x] Pass focused/full gates, commit/push, and close without detector,
      credentials, device access, attempt-004, or parity promotion.

Authorization: software, deterministic child processes, firmware/package
builds, docs, and Git only. Attempt-003 remains immutable failed evidence. No
detector, credentials, protected attempt roots, USB/device/network runtime,
flash, monitor, mining, restart, public projection, attempt-004, recovery,
external UART/BAP, pins, or electrical work is authorized by this plan.

Stopped-state correction review: source `251205a5` centralizes disabled boot
mining as false boot intent plus `paused` or `safe_blocked`, rejects active/
unknown/enabled shapes, and uses the predicate for both restart admission and
evidence. Pure and full real-child paused-restart tests pass alongside existing
safe-blocked success and restart-drift withholding. Focused/full gates, all 47
Bazel targets, firmware/package, redaction, reference, parity, and progress
checks passed. No hardware or protected input was used; `STAT-003` remains
`implemented`. See this plan's `WORKLOG.md` and `CLOSURE.md`.

Next safe action: a fresh immutable hardware plan may rotate to attempt-004 only
after exact package/source binding and must retain every detector, safety,
privacy, recovery, persistence, cleanup, and promotion gate. Recurrence of the
same stopped-state boundary after this fix stops further retries.

Attempt-004 verification plan:
`docs/parity/work-plans/20260818T114249Z-STAT-003/PLAN.md`

- [x] Rotate the scoreboard workflow from consumed attempt-003 to fresh
      attempt-004 and bind the immutable plan, task, ordinal, paths, runfiles,
      contracts, fixtures, and validators.
- [x] Pass focused/full gates, commit/push, build the bound package, and run
      only wrapper-004 detector admission.
- [x] Run at most one conditional attempt-004 and promote only on the complete
      accepted campaign/API/SPA/restart persistence quorum; otherwise preserve
      the first closed failure and stop without attempt-005.

Attempt-004 authorization: attempt-003's exact stopped-state failure now has a
targeted, exhaustive, real-child, fully gated and pushed correction. After
clean pushed gates, one exact board-205 package may repeat the conservative
400 MHz / 1,100 mV core / 100% fan campaign for 600 accumulated active seconds.
Its NVS seed intentionally clears prior scoreboard records and retains owner
Wi-Fi/pool inputs plus the safe-stopped scoreboard. One passive same-origin
capture and one normal HTTP restart may prove UI/API and boot durability. The
linked plan freezes roots, commands, recovery, retry, stop, promotion, privacy,
and prohibited effects; starting capture consumes attempt-004 and no
attempt-005 is authorized.

Attempt-004 closure review: exact source/package `ca972836`, pinned reference,
focused/full gates, one detector, and the sole capture passed admission. The
sealed campaign stopped at 229,579 active ms with distinct closed
`network_unavailable`, 8/20 windows, no qualified candidate, and no submit
response. Identity, safety, watchdog, panic/mixed-reset/correlation diagnostics,
final terminal settlement, terminal transport/pool, safe stop, cleanup, modes,
and sealing remained valid. API/SPA/restart and projection were withheld; no
retry ran and `STAT-003` remains `implemented`. See this plan's `WORKLOG.md` and
`CLOSURE.md`.

Next safe action: do not retry from unchanged external state. A future
scoreboard plan requires an objective repo-owned signal that the protected
owner pool/network path is available again, without exposing endpoint or
credentials. The selector may skip `STAT-003` as environment-blocked and work
the next actionable parity row meanwhile.

Readiness-gated attempt-005 plan:
`docs/parity/work-plans/20260820T150151Z-STAT-003/PLAN.md`

- [x] Add a repo-owned, private-root-exclusive readiness tool that requires
      three consecutive bounded Stratum V1 configure/subscribe/authorize
      sessions without submitting shares or exposing protected pool values.
- [x] Rotate only the scoreboard workflow's plan/task/path/ordinal contracts
      from consumed attempt 4 to fresh attempt 5 and pass every focused/full
      software, privacy, reference, firmware, package, and source gate.
- [x] Run the readiness command exactly once. Only after its exact clean-source
      private result is `ready`, run one detector and the sole conditional
      attempt-005 conservative mining/share scoreboard capture.
- [ ] Promote only on the complete campaign/API/SPA/restart persistence quorum;
      otherwise preserve `implemented`, earliest failure, safe stop, cleanup,
      public withholding, and no attempt-006.

Authorization: On 2026-08-20 the user explicitly authorized use of the existing
ignored pool credentials, mining, and share submission to simplify and complete
STAT-003. The linked immutable plan limits the preflight to three closed
Stratum V1 handshake sessions and the conditional hardware path to one existing
400 MHz / 1,100 mV / 100% fan / 600-active-second attempt. It defines exact
commands, protected roots, privacy, effects, recovery, retry, promotion, and
stop conditions. No other endpoint, pool, profile, hardware, destructive,
fault-injection, UART, pin, or electrical effect is authorized.

Verification: Implementation commit
`2eb620c530f612f7097e1b53d35c1e18b39ced07`; seven focused readiness tests,
one real CLI/subprocess test, scoreboard Rust/TypeScript and generated-contract
tests, ordered workspace Rust gates, Bright Builds, all 48 Bazel tests,
redaction, reference, parity/progress, and package passed. No protected input or
external/device effect was used during implementation verification.

Completion review: The sole readiness command passed 3/3 bounded Stratum V1
sessions and objectively changed the prior network boundary. Attempt-005 then
completed 600,306 active ms, 20/20 windows, 19 accepted shares, fresh safety,
watchdog, safe stop, cleanup, live SPA, and one valid restart. Promotion was
withheld because the verifier required full pre/post scoreboard equality even
though pinned upstream persists difficulty with `%.1f` and reloads it with
`%lf`; every other field and both immediate repeats were exact. Attempt-005 is
consumed, no attempt-006 is authorized, and `STAT-003` remains implemented.
See the linked `WORKLOG.md` and `CLOSURE.md`.

Durable-projection verifier correction plan:
`docs/parity/work-plans/20260820T171138Z-STAT-003/PLAN.md`

- [x] Add a source-bound one-decimal durable difficulty projection and retain
      exact raw digests for same-boot repeat checks.
- [x] Require pre-restart durable projection to equal the raw post-restart
      scoreboard while preserving exact count, order, and non-difficulty data.
- [x] Add positive and negative regressions, pass every software gate, and
      close without protected attempt access, hardware, attempt-006, or parity
      promotion.

Authorization: software, deterministic local child processes, firmware/package
builds, docs, and Git only. No credentials, protected attempt-005 artifacts,
detector, USB/device or external network runtime, flash, monitor, mining, share
submission, device restart, evidence projection/promotion, attempt-006,
recovery, external UART/BAP, pins, or electrical work is authorized.

Verification: Source `4594760b08e606959d952a1fc7803095967e5bf2`
adds exact ties-to-even one-decimal durable projection, raw/durable digests,
full-precision restart success, wrong-difficulty/non-difficulty/order/repeat
failures, and source binding to both persistence codecs. Ordered Cargo gates,
Bright Builds, focused automation, all 48 Bazel tests, firmware build/package,
redaction, reference, parity/progress, selector, and diff checks passed.

Completion review: The verifier now admits precisely the upstream-compatible
restart transformation and no other field drift. `.bazelignore` also prevents
full Bazel discovery from traversing protected/generated local trees, restoring
`just test` to normal completion. No protected evidence or hardware effect was
used; `STAT-003` remains `implemented`. See this plan's `WORKLOG.md` and
`CLOSURE.md`. A future evidence re-evaluation or attempt requires its own
immutable authorization and exact pushed-source binding.

Protected attempt-005 re-evaluation plan:
`docs/parity/work-plans/20260820T220854Z-STAT-003/PLAN.md`

- [x] Add a repo-owned read-only protected recheck that binds the old capture
      and corrected clean pushed evaluator without exposing private values.
- [ ] Recompute every campaign/API/SPA/restart/durable-persistence fact from
      allowlisted sealed attempt-005 artifacts and publish only after
      independent validation/redaction.
- [ ] Promote only on the complete quorum; otherwise preserve `implemented`,
      immutable protected evidence, projection withholding, and no attempt-006.

Authorization: the user explicitly authorizes this evidence plan and the work
necessary to test and verify `STAT-003`. The linked plan permits one read-only
protected attempt-005 re-evaluation and conditional redacted promotion. It
forbids credentials, external network, detector/device/USB access, flash,
monitor, mining, shares, restart, recovery, attempt-006, UART/BAP, pins, and
electrical work.

Verification: Evaluator `d7ecc5066babe15a37d181bd4b799c235985f8fa`
passed focused success/failure, mode, symlink, privacy, source, validator, full
Cargo/Bazel, firmware/package, Bright Builds, redaction, reference, parity, and
selector gates before clean push.

Completion review: The sole protected recheck stopped closed with no candidate
or projection. The retained original package manifest is absent, and an exact-
source rebuild with the retained original build timestamp still changes the
path-sensitive app ELF hash, so its byte digest cannot be truthfully
reconstructed. `STAT-003` remains `implemented`; protected evidence stayed
immutable and no hardware effect occurred. See this plan's `WORKLOG.md` and
`CLOSURE.md`. A fresh v2 software contract may bind the retained exact package
identity and old manifest-admission boundary without fabricating the missing
manifest digest.

Truthful v2 retained-identity evaluation plan:
`docs/parity/work-plans/20260820T224453Z-STAT-003/PLAN.md`

- [x] Add strict v1/v2 evidence validation and a retained capture-package
      identity commitment without an unavailable manifest-byte claim.
- [x] Run one diagnostic-complete v2 protected evaluation and independently
      validate/redact the projection.
- [x] Promote only on the complete v2 quorum; otherwise preserve
      `implemented`, immutable evidence, and no attempt-006.

Authorization: the user explicitly authorizes the evidence plan and all work
necessary to test and verify this row. The linked plan permits one read-only v2
protected evaluation and conditional promotion, with no hardware, device,
network, credential, mining, share, restart, recovery, UART/pin, or electrical
effect.

Verification: V2 evaluator `cbc5fa7f` passed strict v1/v2 contracts, protected
success/failure/privacy fixtures, ordered Cargo gates, Bright Builds, all 48
Bazel tests, firmware build/package, redaction, reference, parity/progress, and
selector gates. The sole v2 protected command exited zero; independent Rust
validation and semantic redaction (`checked=1`) passed for projection
`e8054e9176154f154a82b4c9f5301f9d87f64ca558e2ad117be7c37fc4efe920`.

Completion review: The complete accepted campaign, live SPA, exact restart,
stable repeats, exact non-difficulty fields/order/count, and one-decimal durable
difficulty projection now have truthful source-bound public evidence. No
hardware rerun occurred. `STAT-003` is eligible for its planned isolated
promotion and task archival.

Promotion review: Transition `20260820T230613Z-STAT-003` changed only this row
to `verified` with `unit,workflow,api-compare,static-route,hardware-smoke,
hardware-regression`. The transition binds the v2 plan/result, independently
validated projection, passed redaction, exact capture/evaluator sources, safe
stop, cleanup, and explicit non-claims. Progress synchronized to 89/94 verified
(94.7%). The task is complete and ready for immediate archive.

### task-parity-self001-full-lifecycle | 2026-08-21 | Verify the complete Ultra 205 self-test lifecycle

- [x] Implement a consume-before-use boot-time self-test owner with no public
      mutation route and no ordinary-boot behavior change.
- [x] Prove controlled failure, complete safe-stop, physical BOOT-button
      cancellation/restart, full upstream-compatible pass, and auto-restart.
- [x] Preserve and exactly restore settings, protected evidence, process/USB
      cleanup, and redaction; promote only on the complete hardware regression.

Plan: `docs/parity/work-plans/20260821T180800Z-SELF-001/PLAN.md`

Authorization: repository implementation, tests, build/package, Git commit and
push, one exact detector, one exact-package two-phase board-205 campaign,
private NVS marker/restoration writes, bounded fan/voltage/ASIC diagnostic
effects, the controlled post-load evaluation failure, safe-stop, the built-in
BOOT-button hold selected by the user, software restarts, passive USB/network
observation, and bounded recovery defined by the plan. No external UART, pins,
pads, probes, jumpers, accessories, pool traffic, share submission, sensor or
electrical fault injection, OTA, erase-flash, arbitrary raw writes, other
boards, or retry is authorized.

Evidence/privacy: the supervisor exclusively creates the absent mode-0700
attempt root and mode-0600 artifacts. Credential and settings values remain
`NeverPersistRaw` or protected inputs and never reach terminal or committed
evidence. Only the closed aggregate projection may be promoted after
independent validation and redaction.

Recovery/stop: preserve the earliest failure; attempt every independent
safe-stop step; hold reset low; disable core voltage and ASIC; cool to at most
45 C; settle fan at 30%; restore the exact package/settings with
`mineonboot=false`; release owned processes/USB; withhold `RESULT.md`; and stop
without unchanged retry. Human readiness has no deadline and begins only after
the device is confirmed safe.

Software verification: implementation commit
`e95259ec0d5bfe100ba4d6b096179075476595f7`; focused pure, firmware,
flash-tool, automation, restoration, and evidence-contract tests pass with the
canonical ESP32-S3 package, ordered Cargo gates, Bright Builds, all 51 Bazel
tests, parity/progress, redaction, reference cleanliness, file-size,
sensitive-value, and diff checks. Software evidence:
`docs/parity/evidence/self001-full-lifecycle/summary.md`.

Attempt-001 hardware preflight on 2026-08-21 stopped before backup, mutation,
flash, or self-test effects because the supervisor used the nonexistent
`/api/system/theme` route; the CLI also collapsed its typed category. The
protected failure is retained and no projection was published. Fresh
attempt-002 is authorized only by
`docs/parity/work-plans/20260821T192123Z-SELF-001-RETRY/PLAN.md` after the route
and typed-failure repairs pass all gates and are committed, pushed, and
package-bound. It is a changed attempt after verified progress; no unchanged
retry is allowed.

Attempt-002 on 2026-08-21 proved corrected settings/theme backup, then stopped
before USB because the self-test intent owner applied a 64-character digest
validator to valid 40-character commits. An exact dry-run reproduced
`invalid_source_commit`; receive-only serial proved ordinary advancing runtime
with no self-test markers. No projection or device effect occurred.
Attempt-003 is authorized only by
`docs/parity/work-plans/20260821T200723Z-SELF-001-RETRY-2/PLAN.md` after exact
commit-length tests, an automatic real-command dry-run gate, durable
pre-effect campaign state, full verification, commit/push, and package binding.
Attempts 001 and 002 remain terminal and preserved.

Attempt-003 installed the exact package, completed controlled load and
safe-stop, published `cancel_ready`, and accepted the physical BOOT hold; later
serial proved ordinary runtime. Resume missed the lease-bound cancellation
receipt because it was logged only during early boot before monitor attachment.
Phase B did not start, no projection was published, and exact settings/theme
were restored with `mineonboot=false`. Attempt-004 is authorized only by
`docs/parity/work-plans/20260821T211712Z-SELF-001-RETRY-3/PLAN.md` after a
serial-only 10-second persisted-receipt replay, missing-receipt recovery, full
verification, commit/push, and package binding.

Attempt-004 proved the complete failure/cancel lifecycle and advanced the pass
run through warm-up, measurement, evaluation, and safe-stop, then failed closed
as `domain_failed`. Automatic recovery restored the exact package/settings and
withheld projection. The defect is synthetic Rust domain attribution from
`small_core_id % 4`; upstream uses BM1366 counter registers `0x88–0x8B`.
Attempt-005 is authorized only by
`docs/parity/work-plans/20260822T024037Z-SELF-001-RETRY-4/PLAN.md` after the
existing typed register-read and pure hashrate-monitor path replaces that
mapping, all gates pass, and a new exact package is bound.

Completion review: Exact clean source `a11b579b` attempt-005 completed the
controlled failure, safe-stop, physical cancellation/restart, passing BM1366
counter-domain self-test, pass receipt/restart, exact settings/theme
restoration with `mineonboot=false`, cleanup, independent validation, and
redaction. Public projection:
`docs/parity/evidence/self001-full-lifecycle/self-test-projection.json`.
Result: `docs/parity/work-plans/20260822T024037Z-SELF-001-RETRY-4/RESULT.md`.
Residual risks are limited to the explicit non-claims in that result.

Promotion review: Transition `20260822T033110Z-SELF-001` changed only this row
to `verified` with `unit,workflow,hardware-regression`, bound to final plan,
result, projection, exact source/reference/package identity, both hardware
phases, safe-stop, both receipts/restarts, restoration, cleanup, independent
validation, and redaction. Progress synchronized to 90/94 verified (95.7%).
The task is complete and ready for immediate archive.

### task-parity-str005-stratum-v2 | 2026-08-22 | Implement Stratum V2 protocol and firmware ownership

- [x] Implement bounded Stratum V2 framing, Noise transport, SetupConnection,
      standard and extended channel, job, target, and share-message behavior
      against the pinned reference implementation.
- [x] Add a single firmware Stratum V2 task owner with typed configuration,
      protocol-coordinator primary/fallback selection, bounded reconnect and
      timeout behavior, explicit memory limits, and secret-free diagnostics.
- [x] Add provenance-bound golden fixtures and focused pure, transport,
      lifecycle, fallback, malformed-input, and redaction tests; build/package
      the ESP32-S3 firmware and pass every mandatory repository gate.
- [x] Add the deterministic host-owned SV2 Noise pool fixture, private
      detector-gated campaign, safe-stop/restoration path, closed evidence
      projection, and independent validator.
- [x] Transition only `STR-005` to the strongest evidence-supported status:
      `implemented` with `unit,golden,workflow`, or `verified` only after the
      exact accepted Ultra 205 campaign adds `hardware-regression`.
- [x] Run the exact attempt-002 campaign once; preserve `implemented` after its
      pre-effect `evidence_invalid` closure and consume the ordinal.
- [x] Reproduce and distinguish the collapsed pre-effect `evidence_invalid`
      boundary through the real Bazel launcher without consuming a hardware
      attempt.
- [x] Fix the diagnosed campaign boundary, add regression coverage at the real
      process/runfiles seam, and pass the complete software verification gates.
- [x] Authorize a fresh hardware ordinal only through a new immutable
      continuation plan whose checkpoint discriminator proves the changed
      pre-effect boundary before any effect.
- [x] Run the exact attempt-003 campaign once; preserve `implemented` after its
      pre-root `hardware_blocked` / `unclassified` closure and consume the
      ordinal.
- [x] Add typed runtime monitor, origin, settings, restoration-input, and
      restore-package admission plus a read-only real-device command.
- [x] Run the read-only admission under the new immutable plan; withhold
      attempt-004 after the closed `restore_package` blocker.

Plan: `docs/parity/work-plans/20260822T040442Z-STR-005/PLAN.md`

Consumed hardware continuation plan:
`docs/parity/work-plans/20260822T063702Z-STR-005-RETRY/PLAN.md`

Consumed hardware continuation plan:
`docs/parity/work-plans/20260822T165408Z-STR-005-RETRY2/PLAN.md`

Closed runtime-admission plan:
`docs/parity/work-plans/20260822T171824Z-STR-005-RUNTIME-ADMISSION/PLAN.md`

Reference scope:

- `reference/esp-miner/components/stratum_v2/sv2_protocol.c`
- `reference/esp-miner/components/stratum_v2/sv2_noise.c`
- `reference/esp-miner/main/tasks/stratum_v2_task.c`
- `reference/esp-miner/main/tasks/protocol_coordinator.c`
- `reference/esp-miner/main/nvs_config.c`

Authorization: repository source, fixture, test, documentation, build/package,
Git commit, push, network, pool, and Ultra 205 hardware work under the linked
immutable plan. Attempts 001, 002, and 003 are consumed. Attempt-004 remains
unused and is not effect-eligible because the closed runtime-admission plan did
not reach `runtime_admission_ready`. No campaign command is currently
authorized. Effects remain
ineligible until the repo-owned command, private
schemas, validator, recovery, tests, full gates, clean exact package, and pushed
implementation commit exist. The campaign may use one host-owned local SV2
Noise pool fixture, exact package flash, temporary private Wi-Fi/SV2 NVS,
conservative 400 MHz/1100 mV/100% fan mining, one accepted share, safe stop,
exact setting/package restoration, and cleanup. Third-party pools, external
UART, pins/pads/headers/GPIO, probes, jumpers, electrical work, fault injection,
unbounded mining, arbitrary profiles, OTA, erase, and raw secret output remain
prohibited. The full objective, privacy/evidence policy, preconditions,
effects, limits, recovery, retry bound, and stop conditions are authoritative
in the linked plan.

Evidence/status boundary: promotion into `Active` and this explicit deferred
activation do not themselves change checklist status. `implemented` requires
the complete bounded software owner, current pinned-reference provenance,
deterministic tests, canonical firmware package, redaction review, and all
mandatory gates. `verified` additionally requires the linked accepted local
SV2 Noise pool/Ultra 205 hardware regression with safe-stop, exact restoration,
cleanup, independent validation, and redaction. External production-pool
interoperability remains a non-claim.

Progress: commit `4718a9e5` repaired legacy verified-plan selection and added
the only audited deferred activation path, then transitioned `STR-005` to
`in-progress`. The current pure slice adds official SRI Noise NX, bounded
six-byte framing, pinned standard/extended messages, channel/job/target/share
state, BM1366 work conversion, provenance-bound golden vectors, fail-closed
malformed/tamper/nonce boundaries, and redaction-safe diagnostics. Focused Cargo
tests and canonical Bazel `//crates/bitaxe-stratum:tests` pass. Firmware owner,
campaign, package, full gates, and hardware evidence remain pending.

Software completion review: the firmware now selects exactly one V1/V2 owner,
admits V2 effects only through the consumed 180-second conservative campaign,
requires PSRAM plus fresh Ultra 205 safety before preparation, uses the shared
ASIC/safety/watchdog owners, bounds pre-work retry to the configured V2 pool
pair, poisons failed Noise sessions, and always attempts terminal safe stop.
The host fixture completes a real TCP Noise handshake and validates one
target-qualified standard share before success. The private flash campaign
stage admits only standard-channel canonical Base58Check authority credentials
and requires ordered V2 runtime plus safe-stop markers. Canonical firmware
build/package, all 52 Bazel tests, 23 pure V2 tests, the real fixture test, 393
flash tests, Bright Builds, license generation, and reference cleanliness pass.
No hardware or external-network effect occurred in the software checkpoint.
The outer command now fails before mutation unless current settings are exactly
reconstructible from protected local inputs and an exact prior package is
available; it supervises the owned fixture/campaign, restores package and
settings on success or failure, and publishes only after an independent
validator passes. The authorized ordinals later closed before any effect, so
`verified` is not claimed.

Status transition: source/evidence commit
`abf6c1bdfaf3f929f2fea30ec630635262221755` is bound by transition
`20260822T061900Z-STR-005` and code-span metadata correction
`20260822T062500Z-STR-005-TARGETS`; `STR-005` is `implemented` with
`unit,golden,workflow`. The task remains active because the exact-restoration
outer hardware campaign and `hardware-regression` evidence are incomplete.

Hardware attempt closure: attempt-001 and the regression-backed attempt-002
both stopped as `evidence_invalid` before passive monitoring, private-root
creation, fixture start, NVS construction, USB campaign ownership, flash,
network, mining, or hardware control. Attempt-002 ran only after every gate
passed on clean pushed source `c8de00ca`; the exact package, credential modes,
ignored absent paths, and synchronized Git state were re-confirmed afterward.
No third attempt is authorized. The row remains `implemented`, the hardware
checkbox remains open, and a fresh audited continuation must first expose a
closed pre-effect checkpoint discriminator; see
`docs/parity/work-plans/20260822T063702Z-STR-005-RETRY/CLOSURE.md`.

Bug-fix continuation: investigation started after explicit user authorization
on 2026-08-22. This phase is software-only: it may add value-free closed
pre-effect diagnostics and real-launch regression coverage, but it must not run
the consumed campaign or touch hardware. A new effect contract is required
after the root cause and fix are independently proven.

Root-cause finding: the attempt-002 workspace patch resolved source file paths
through `BUILD_WORKSPACE_DIRECTORY`, but the campaign's Git, flash, validator,
route, and fixture children still inherited Bazel execroot as `cwd`. The first
actual predicate, `git check-ignore -q` for the private attempt path, therefore
returned nonzero and collapsed to `evidence_invalid`. The real-launch regression
reproduced that exact exit and passes after every campaign child receives the
resolved workspace explicitly. A separate read-only preflight now exposes only
closed checkpoint names and proves `effect_started=false` plus
`private_root_created=false`.

Software fix verification: the exact real-launch loop first failed with child
Git top-level at Bazel execroot and ignored-path exit 1, then passed twice after
workspace binding. The hermetic campaign/preflight tests, aggregate automation
suite, ordered Cargo format/clippy/build/test gates, Bright Builds, all 53 Bazel
tests, canonical firmware build/package, parity/progress, redaction, reference
cleanliness, open-plan selection, sensitive-value review, and diff checks pass.
The shared source-workspace resolver now also rejects nested Bazel-output
`MODULE.bazel` copies without Git identity. All temporary debug probes were
removed. No detector, USB, device, fixture, network, pool, mining, or hardware
effect ran during diagnosis and software verification.

Attempt-003 outcome: clean pushed source and package `39aefd23` passed the
no-effect `pre_effect_ready` checkpoint and admitted exactly one Ultra 205. The
single campaign stopped after about 20 seconds as `hardware_blocked` with the
checkpoint still `unclassified`. The attempt root and public projection remain
absent, proving fixture start, NVS construction, flash, pool traffic, mining,
share submission, and hardware control did not begin. No owned process remains;
a post-attempt detector again confirmed the same USB session ready. The timing
and execution order place the remaining gap in passive runtime monitor/origin
or settings/restoration admission, but the consumed evidence cannot distinguish
those sub-boundaries. See the linked attempt-003 closure. No retry is allowed.

Runtime-admission software verification: the effectful campaign and read-only
diagnostic now share one monitor/origin/settings/restoration/package admission
path with closed failure checkpoints and `runtime_admission_ready`. Pure origin
cardinality, protected preflight, failure redaction, exact attempt-004 parsing,
and real-launch workspace/ignored-path tests pass. Ordered Cargo gates, all 53
Bazel tests, canonical build/package, Bright Builds, parity/progress, redaction,
reference cleanliness, selector lineage, sensitive-value review, and diff checks
pass; focused tests also pass after the final error-classification cleanup. No
device access occurred during this implementation/gate cycle.

Runtime-admission outcome: clean pushed source and package `24180a94` passed
`pre_effect_ready`, then a fresh detector admitted exactly one Ultra 205. The
read-only command passed passive monitor completion, unique runtime-origin,
same-origin system/theme reads, and exact restoration-input reconstruction, but
stopped as `hardware_blocked` / `restore_package`. Attempt-004 was not consumed;
its root and the public projection remain absent, and no fixture, flash, pool,
mining, share, or hardware-control effect ran. The bounded local inventory has
71 retained package manifests but cannot construct one exact restorable package
for the firmware currently running. STR-005 remains `implemented`. Do not flash
a new baseline or weaken exact rollback; see the linked closure.

Recovery continuation:
`docs/parity/work-plans/20260824T000902Z-STR-005-RESTORE-RECOVERY/PLAN.md`

Recovery outcome: clean pushed source and exact current package `5a0a3010`
passed all gates, and a fresh detector admitted one Ultra 205. Bounded search
and the single timestamp-pinned rebuild did not yield an exact installed
package. Firmware-only fallback completed three allowed ranges, skipped NVS and
coredump storage, then stopped as `hardware_blocked` / `snapshot_capture` when
the 4 MiB factory read reached the fixed 300-second child limit. No bundle or
public projection was published; attempt-004 remains unused and absent, no
write or campaign effect began, private modes were repaired and verified, and
post-run detection passed. `STR-005` remains `implemented`; this plan is closed
and authorizes no retry. See the linked closure.

Supersession review: Superseded by the decision-complete decomposition plan `docs/parity/work-plans/20260828T175218Z-STR-005-DECOMPOSITION/PLAN.md` and successor tasks task-str005-tcp-payload-205, task-str005-noise-auth-205, task-str005-v2-channel-job-205, task-str005-bm1366-share-205, task-str005-evidence-promotion. Existing plans, closures, evidence, attempt ordinals, and terminal decisions remain immutable; this archival does not change STR-005 from `implemented | unit,golden,workflow`.

### task-parity-str005-installed-package-recovery | 2026-08-23 | Recover installed firmware and verify STR-005

- [x] Implement one protected restore-recovery owner with exact runtime identity,
      bounded artifact search, one timestamp-pinned rebuild, and firmware-only
      flash snapshot fallback that never reads NVS or coredump storage.
- [x] Add a typed `package_v3` / `flash_snapshot_v1` restore bundle, independent
      validator, historical-package admission, and exact snapshot restore adapter.
- [x] Require the explicit validated restore bundle in the attempt-004 campaign
      and prove original package/runtime/settings restoration on every terminal path.
- [x] Pass all pure, real-process, flash-rendering, restoration, redaction,
      Cargo, Bright Builds, Bazel, firmware, package, parity, and reference gates.
- [ ] Commit and push the exact implementation before device access; recover the
      installed bundle without mutation and run attempt-004 only after readiness.
- [x] Promote only `STR-005` on one independently accepted attempt-004 projection;
      otherwise close truthfully, preserve `implemented`, and authorize no retry.

Plan: `docs/parity/work-plans/20260824T000902Z-STR-005-RESTORE-RECOVERY/PLAN.md`

Authorization: repository source/test/docs/build/package, Git commit/push,
read-only runtime HTTP/USB observation, bounded Git/submodule/toolchain network,
one protected installed-firmware recovery, and the single attempt-004 campaign
under the immutable plan. Recovery may search only repository/Bazel package
locations, create one owned detached worktree, perform one exact historical
rebuild, and, only if no exact package is recovered, read the eight plan-listed
firmware ranges. It must never read raw NVS or coredump storage. Attempt-004 may
flash the exact current package, use the local SV2 fixture and conservative
400 MHz/1100 mV/100% fan profile, then restore the admitted original bundle and
settings. Historical/raw restoration writes are allowed only after campaign
effects and only through the plan-bound recovery adapter. A new baseline,
external pool, direct UART/pins, fault injection, OTA, erase, arbitrary writes,
unbounded mining, and attempt-005 remain prohibited.

Software checkpoint: the recovery owner parses the installed source/reference,
ELF digest, timestamp, build identity, IDF version, and running partition; scans
bounded repository/Bazel package locations; performs at most one clean detached-
worktree rebuild with canonical stable/volatile provenance; and captures only
the eight admitted firmware ranges when rebuild recovery is unavailable. The
independent validator enforces protected modes, containment, schema-v3 artifact
digests or exact snapshot ranges, plan/source binding, and a closed redacted
projection. Attempt-004 preflight/runtime admission require the fixed bundle,
and the Rust restore adapter admits historical packages separately from normal
current-workspace flashing or renders one managed eight-range write followed by
Wi-Fi seed and exact runtime/settings verification. Ordered Cargo gates, 395
flash tests, recovery/campaign tests, Bright Builds, all 54 Bazel tests,
canonical firmware/package, parity/progress, redaction, reference cleanliness,
selector lineage, and diff checks pass. No device effect ran during software
implementation.

Hardware closure: the recovery fallback stopped before readiness at the bounded
factory-read timeout. Attempt-004 was withheld and remains unused; no device
write, fixture, pool, mining, ASIC-control, or settings effect began. Cleanup,
protected modes, absent campaign outputs, and a fresh one-board detector passed.
The diagnosed readback timeout and interrupted-file mode defect is corrected in
software but was not rerun. Closure:
`docs/parity/work-plans/20260824T000902Z-STR-005-RESTORE-RECOVERY/CLOSURE.md`.

Supersession review: Superseded by the decision-complete decomposition plan `docs/parity/work-plans/20260828T175218Z-STR-005-DECOMPOSITION/PLAN.md` and successor tasks task-str005-tcp-payload-205, task-str005-noise-auth-205, task-str005-v2-channel-job-205, task-str005-bm1366-share-205, task-str005-evidence-promotion. Existing plans, closures, evidence, attempt ordinals, and terminal decisions remain immutable; this archival does not change STR-005 from `implemented | unit,golden,workflow`.

### task-parity-str005-installed-package-recovery-002 | 2026-08-23 | Retry installed firmware recovery at corrected readback bounds

- [x] Bind recovery-002 to the corrected explicit-baud/protected-target command,
      a fresh private root, fresh public projection, and immutable continuation.
- [x] Pass every software, privacy, package, reference, selector, and diff gate;
      commit/push before device access and build the exact clean package.
- [x] Run recovery-002 once, independently validate its exact restore bundle,
      and withhold the campaign unless all no-effect readiness gates pass.
- [ ] Run still-unused attempt-004 once only after admission, then safe-stop,
      restore the original bundle/settings, prove cleanup and exact identity,
      and independently validate the closed campaign projection.
- [x] Promote only `STR-005` on complete accepted hardware evidence; otherwise
      close truthfully at `implemented`, withhold `RESULT.md`, and do not retry.

Plan: `docs/parity/work-plans/20260824T012436Z-STR-005-RESTORE-RECOVERY2/PLAN.md`

Authorization: repository source/test/docs/build/package, Git commit/push,
read-only runtime HTTP/USB observation, bounded Git/submodule/toolchain network,
one fresh protected recovery-002, and the still-unused attempt-004 campaign
under the immutable plan. Recovery may search only repository/Bazel package
locations, create one owned detached worktree, perform one timestamp-pinned
historical rebuild, and then read only the eight allowlisted firmware ranges at
explicit 460800 baud with 600 seconds per range. It never reads NVS or coredump
storage. Attempt-004 may flash the exact current package, use the local SV2
fixture and conservative 400 MHz/1100 mV/100% fan profile, then restore the
admitted original bundle and settings. A new baseline, external pool, direct
UART/pins, fault injection, OTA, erase, arbitrary writes, unbounded mining,
attempt-005, and unchanged recovery retry remain prohibited.

Evidence/status boundary: recovery readiness requires one independently
validated exact package or eight-range snapshot bundle with protected modes,
containment, digests, installed identity, source/plan binding, runtime
continuity, cleanup, and a closed redacted projection. `STR-005` stays
`implemented` unless attempt-004 additionally proves the local Noise handshake,
channel/job/work/share lifecycle, accepted response, complete safe stop, exact
original runtime/settings restoration, cleanup, and independent redaction-safe
validation.

Progress: authorized after recovery-001 closed on the changed 4 MiB readback
boundary. The recovery, campaign, historical restore, runfile, and focused test
bindings now target only the new task/plan/root/projection, and focused recovery,
campaign, launcher, and all 395 flash tests pass. No recovery-002 or attempt-004
effect has begun.

Software verification: ordered Cargo format/clippy/build/test, Bright Builds,
all 54 Bazel tests, canonical firmware build/package, parity/progress,
redaction, pinned-reference cleanliness, open-plan selection, sensitive-value
review, and final diff checks pass. The selector resumes only `STR-005` through
the new immutable plan. Commit/push and an exact clean package rebuild remain
the final pre-device gates.

Hardware closure: clean pushed source/package `de081a94` and a fresh detector
passed. Recovery-002 completed all eight explicit-baud firmware-only reads,
excluded NVS/coredump, created a protected snapshot bundle, and proved runtime
continuity, but the owner stopped `evidence_invalid` at
`independent_validation` and published no final projection. The same retained
bundle/candidate passes a bounded post-run invocation of the validator with the
original source/plan bindings, but the failed owner retained no diagnostic that
can safely classify the child-only discrepancy. The post-run result does not
override the terminal command. Attempt-004 remains unused and absent; no write,
fixture, pool, mining, ASIC-control, or settings effect began. Private modes,
process cleanup, unpublished-candidate containment, and post-run one-board
detection pass. `STR-005` remains `implemented`; no retry is authorized.
Closure:
`docs/parity/work-plans/20260824T012436Z-STR-005-RESTORE-RECOVERY2/CLOSURE.md`.

Supersession review: Superseded by the decision-complete decomposition plan `docs/parity/work-plans/20260828T175218Z-STR-005-DECOMPOSITION/PLAN.md` and successor tasks task-str005-tcp-payload-205, task-str005-noise-auth-205, task-str005-v2-channel-job-205, task-str005-bm1366-share-205, task-str005-evidence-promotion. Existing plans, closures, evidence, attempt ordinals, and terminal decisions remain immutable; this archival does not change STR-005 from `implemented | unit,golden,workflow`.

### task-parity-str005-installed-package-recovery-003 | 2026-08-24 | Diagnose validator child and continue STR-005 recovery

- [x] Add a protected closed validator-child receipt and real Bazel launcher
      regression for acceptance, rejection, launch failure, timeout, output
      limit, working-directory binding, modes, and redaction.
- [x] Bind the fresh recovery-003 root/projection/bundle across recovery,
      admission, campaign, restore, runfiles, task, and tests.
- [x] Pass every software/privacy/package/reference/selector/diff gate,
      commit/push, and build the exact clean package before device access.
- [x] Run recovery-003 once and withhold attempt-004 unless the owner publishes
      an independently accepted exact restore-readiness projection.
- [ ] If admitted, run attempt-004 once, safe-stop, restore the original
      bundle/settings, prove exact runtime/cleanup, and validate evidence.
- [x] Promote only `STR-005` on complete accepted hardware evidence; otherwise
      close at `implemented`, withhold `RESULT.md`, and do not retry.

Plan: `docs/parity/work-plans/20260824T214920Z-STR-005-RESTORE-RECOVERY3/PLAN.md`

Authorization: repository source/test/docs/build/package, Git commit/push,
bounded real-process validation, read-only runtime HTTP/USB observation, one
fresh protected recovery-003, and the still-unused attempt-004 campaign under
the immutable plan. Recovery retains a private closed validator-child receipt,
searches only repository/Bazel package locations, performs at most one
timestamp-pinned historical rebuild, then reads only the eight allowlisted
firmware ranges at 460800 baud with 600 seconds per range. It never reads NVS or
coredump. Attempt-004 may flash the exact current package, use the local SV2
fixture and 400 MHz/1100 mV/100% fan profile, then restore the admitted original
bundle/settings. A new baseline, external pool, direct UART/pins, fault
injection, OTA, erase, arbitrary writes, unbounded mining, attempt-005, and
unchanged retry remain prohibited.

Evidence/status boundary: recovery readiness requires an accepted owner result,
protected validator receipt, exact package or snapshot bundle, runtime
continuity, independent validation, cleanup, and a closed redacted projection.
The retained recovery-002 bundle is diagnostic evidence only and is not an
admitted campaign restore source. `STR-005` remains `implemented` until the
complete attempt-004 local Noise/share/safe-stop/exact-restoration evidence is
independently accepted.

Progress: authorized from the recovery-002 child-only validator discrepancy.
The dedicated child runner strips inherited nested `JS_BINARY__*` state, bounds
and hashes output, retains one closed protected receipt for every outcome, and
requires an accepted receipt during later campaign admission. Focused recovery,
campaign, real-launch, and all 395 flash tests pass across acceptance,
rejection, launch failure, timeout, output limit, workspace binding, modes, and
secret-canary exclusion. No recovery-003 or attempt-004 effect has begun.

Software verification: ordered Cargo format/clippy/build/test, Bright Builds,
all 55 Bazel tests including the real nested-launch regression, canonical
firmware build/package, parity/progress, redaction, reference cleanliness,
selector lineage, sensitive-value review, file-length checks, and final diff
review pass. Commit/push and the exact clean package rebuild are the remaining
atomic pre-device steps.

Hardware closure: exact pushed source/package `b33c89d1` and a fresh detector
passed, but recovery-003 stopped `hardware_blocked` at
`runtime_monitor_process` before installed identity, package search/rebuild,
flash readback, bundle, validator, or receipt creation. The empty private root
is `0700`; no public projection/candidate or campaign root exists. Attempt-004
remains unused, no write/readback/fixture/pool/mining/ASIC/settings effect began,
all children exited, and post-run one-board detection passed. The validator-
child software boundary remains regression-proved but was not reached on the
device path. `STR-005` remains `implemented`; no retry is authorized. Closure:
`docs/parity/work-plans/20260824T214920Z-STR-005-RESTORE-RECOVERY3/CLOSURE.md`.

Supersession review: Superseded by the decision-complete decomposition plan `docs/parity/work-plans/20260828T175218Z-STR-005-DECOMPOSITION/PLAN.md` and successor tasks task-str005-tcp-payload-205, task-str005-noise-auth-205, task-str005-v2-channel-job-205, task-str005-bm1366-share-205, task-str005-evidence-promotion. Existing plans, closures, evidence, attempt ordinals, and terminal decisions remain immutable; this archival does not change STR-005 from `implemented | unit,golden,workflow`.

### task-parity-str005-autonomous-continuation | 2026-08-25 | Autonomously finish STR-005 with progress-gated attempts

- [x] Add a tight red-capable runtime-monitor feedback loop and protected closed
      monitor-child receipt at the real process/USB boundary.
- [x] Diagnose the recovery-003 boundary, add the regression before the fix,
      prove red/green, and pass every software/privacy/package/reference gate.
- [x] After push/package/detector, prove the real fixed boundary once with:
      `just stratum-v2-runtime-monitor-diagnostic --board 205 --port <detector-port> --private-root scratch/str005-runtime-monitor-diagnostic/diagnostic-001 --redact-evidence`.
- [x] Commit/push, build the exact package, and run recovery-004 once with:
      `just stratum-v2-restore-recovery --board 205 --port <detector-port> --private-root scratch/str005-installed-package-recovery/recovery-004 --projection docs/parity/evidence/str005-installed-package-recovery/restore-readiness-projection-004.json --redact-evidence`.
- [x] For each later progress-backed recovery, append the fresh ordinal, exact
      expanded command, boundary signature, verified fix/regression, terminal
      outcome, and continuation decision to this block before effects.
- [x] After accepted recovery readiness, bind its exact bundle and run the
      existing preflight, runtime admission, and single attempt-004 campaign.
- [x] Promote only `STR-005` after accepted hardware plus exact restoration;
      otherwise stop only at a policy terminal and close truthfully.

Plan: `docs/parity/work-plans/20260825T123346Z-STR-005-AUTONOMOUS-CONTINUATION/PLAN.md`

Authorization: autonomous repository diagnosis/fixes/tests/docs/build/package,
Git commit/push, network/toolchain use, read-only runtime HTTP/USB observation,
fresh progress-backed recovery ordinals, and the still-unused single
attempt-004 campaign under the rolling plan. No repeated human confirmation or
new plan is required. Every fresh ordinal still requires a fully expanded exact
command recorded here, a changed regression-backed boundary, all gates, exact
clean pushed package, detector admission, fresh roots, one execution, cleanup,
and one closed policy outcome. Unchanged retries and reused roots are forbidden.

Initial authoritative boundary: recovery-003 ended
`hardware_blocked/runtime_monitor_process` before identity capture. Recovery-004
is ineligible until a protected monitor receipt, red/green real-process
reproduction, targeted fix, accepted read-only real-USB diagnostic, complete
gates, push, and exact package exist. Attempt-004 remains unused.

Standing non-claims: raw NVS/coredump capture, new baseline, external pool,
direct UART/pins, fault injection, OTA, erase, arbitrary writes, unbounded
mining, repeated attempt-004, attempt-005 campaign, and release readiness.

Progress ledger: the exact `bazel test
//tools/automation:stratum_v2_restore_unit_test` loop failed red at
`hardware_blocked/runtime_monitor_process` in 0.2 seconds when the production
caller rendered its former 15-second capture bound. Changing only capture to
the contract-required 60 seconds and child lifetime to 75 seconds made the same
real-process loop green, confirming the highest-ranked timeout hypothesis. The
recovery path now retains initial/final protected monitor receipts with closed
exit, timeout, output-limit, origin-count, USB-cleanup, launcher, cwd, and digest
facts. A dedicated read-only real-USB diagnostic is bound as the required
real-boundary regression before recovery-004. Focused recovery, campaign,
launcher, and all 395 flash tests pass. No autonomous-continuation hardware
effect has begun.

Software verification: ordered Cargo format/clippy/build/test, Bright Builds,
all 55 Bazel tests, focused red/green and closed-receipt tests, canonical
firmware build/package, parity/progress, redaction, reference cleanliness,
selector lineage, sensitive-value review, file-length checks, and diff review
pass. One full Bazel run transiently reported both firmware Cargo actions
failed; the narrowed concurrent build, a forced fresh concurrent rebuild, and
an unchanged full rerun all passed, so no deterministic source or shared-target
failure reproduced. No workaround or suppressed gate was added.

Recovery-004 outcome: pushed source/package `ed69cc24`, fresh detector, and the
read-only `diagnostic-001` real USB boundary passed `runtime_monitor_ready` with
a protected accepted receipt. Recovery-004 then passed both qualified runtime
monitors, identity/runtime continuity, all eight firmware-only reads, protected
modes, and bundle construction, but stopped `evidence_invalid` at
`independent_validation`. Its new receipt signature is child exit 1, no timeout,
output limit, or spawn failure, zero stdout bytes, bounded stderr digest/count,
and `validation_accepted=false`. No final projection or campaign root was
published; attempt-004 remains unused. The unpublished candidate was preserved
outside the sealed root, all children exited, and post-run one-board detection
passed. No device write, NVS access, fixture, pool, mining, ASIC, or settings
effect began.

Continuation decision: `continue_after_verified_fix`. The exact workspace
feedback command `bazel run
//tools/automation:stratum_v2_restore_workspace_launcher_test` reproduced the
child-only failure in milliseconds. It showed the nested Bazel launcher failed
before validator stdout; direct CLI launch then exposed a Node runtime boundary.
The Bazel wrapper's resolved `JS_BINARY__NODE_BINARY`, rather than
`process.execPath`, is the correct independent child executable. The same exact
workspace command now passes accepted and rejected fixtures. Recovery-005 is
the next fresh ordinal and is ineligible until focused/full gates, push, exact
package, and detector pass. Exact command:
`just stratum-v2-restore-recovery --board 205 --port <detector-port> --private-root scratch/str005-installed-package-recovery/recovery-005 --projection docs/parity/evidence/str005-installed-package-recovery/restore-readiness-projection-005.json --redact-evidence`.

Recovery-005 outcome: exact pushed source/package `caf24be8` and fresh detector
passed, but the initial monitor stopped `hardware_blocked/runtime_monitor_process`.
Its protected signature is `timeout` with one unique origin and substantial
bounded runtime output already observed, no stderr, launch failure, or output
limit, and cleanup not yet reached when the outer 75-second supervisor expired.
The root contains only the protected initial-monitor receipt; no identity,
search/rebuild, readback, bundle, projection, validator, fixture, campaign, or
device effect began. A post-run detector proved cleanup and one ready board.

Continuation decision: `continue_after_verified_fix`. Device-session source
proves the monitor command may independently spend 10 seconds probing, 60
seconds admitting/reacquiring, 60 seconds capturing, and 60 seconds final-
cleaning. The new timing regression failed red at 75 seconds and passed green
after changing only the outer ceiling to 210 seconds; the internal 60-second
capture and every device-session phase bound remain unchanged. Before
recovery-006, the exact real-USB regression is:
`just stratum-v2-runtime-monitor-diagnostic --board 205 --port <detector-port> --private-root scratch/str005-runtime-monitor-diagnostic/diagnostic-002 --redact-evidence`.
Recovery-006 exact command:
`just stratum-v2-restore-recovery --board 205 --port <detector-port> --private-root scratch/str005-installed-package-recovery/recovery-006 --projection docs/parity/evidence/str005-installed-package-recovery/restore-readiness-projection-006.json --redact-evidence`.

Recovery-006 and campaign outcome: `diagnostic-002`, focused/full gates, pushed
source/package `7d5d9504`, and fresh detector passed. Recovery-006 independently
accepted its eight-range snapshot bundle and published the closed redacted
readiness projection. A real-Git regression then fixed collapsed untracked-
directory preflight status, and a four-file host-only descendant allowlist
admitted only the reviewed preflight/lineage changes. Exact package `78784a4a`,
`pre_effect_ready`, fresh detector, and `runtime_admission_ready` passed.

The single attempt-004 campaign was consumed. Current factory and temporary NVS
writes completed/verified, but active mining never began (`active_ms=0`), no
protocol/share transition was observed, and the owner ended
`timeout/unclassified`. USB cleanup passed. Original restoration was attempted
once and failed (`restored=false`), so the public campaign projection and
`RESULT.md` are withheld. Post-campaign detector plus protected same-session
monitor/API audit prove the current package is running, original package is not,
`mineonboot=false`, mining is `safe_blocked`, hashrate is zero, and no shares
were accepted/rejected.

Terminal outcome: `stop_authority_boundary`. Attempt-004 and its once-only
restoration are consumed; no second write is authorized. `STR-005` remains
`implemented`. Closure:
`docs/parity/work-plans/20260825T123346Z-STR-005-AUTONOMOUS-CONTINUATION/CLOSURE.md`.

Supersession review: Superseded by the decision-complete decomposition plan `docs/parity/work-plans/20260828T175218Z-STR-005-DECOMPOSITION/PLAN.md` and successor tasks task-str005-tcp-payload-205, task-str005-noise-auth-205, task-str005-v2-channel-job-205, task-str005-bm1366-share-205, task-str005-evidence-promotion. Existing plans, closures, evidence, attempt ordinals, and terminal decisions remain immutable; this archival does not change STR-005 from `implemented | unit,golden,workflow`.

### task-str005-exact-restoration-remediation | 2026-08-25 | Restore the pre-campaign Ultra 205 state

- [x] Add a red/green real-adapter regression proving the historical recovery
      bundle fails only on current-host source equality.
- [x] Add exact current-source authorization, remediation-plan binding,
      admission-only restore validation, and protected diagnostics.
- [x] Add the resumable host owner for snapshot/Wi-Fi/settings/theme restoration
      and exact original-runtime verification.
- [x] Pass all software, privacy, package, reference, selector, and diff gates;
      commit/push and build the exact host package before device access.
- [x] Run exact no-effect preflight, fresh detector, and remediation-001 once;
      use settings-only resume only from proved `firmware_restored` state.
- [ ] Publish remediation evidence and archive only this task on exact success;
      never retry STR-005, promote STR-005, or weaken its campaign closure.

Plan: `docs/parity/work-plans/20260825T150417Z-STR-005-EXACT-RESTORATION/PLAN.md`

Authorization: source/test/docs/build/package, Git commit/push, read-only current
runtime HTTP/USB observation, exact recovery-006 snapshot write, Wi-Fi NVS seed,
settings/theme restoration, fresh detector/reacquisition, protected evidence,
and one remediation ordinal under the linked plan. The canonical host package
is a provenance gate and must not be flashed. The task permits no campaign,
external pool, raw NVS/coredump read, direct UART/pins, fault injection, OTA,
erase, arbitrary writes, or STR-005 promotion.

Precondition: the board remains reachable on campaign package `78784a4a`, with
`mineonboot=false`, mining `safe_blocked`, zero hashrate, and zero shares. The
recovery-006 bundle/projection/receipts, attempt-004 backup, and ignored local
credential files must remain protected and unchanged. Any mismatch stops before
effects.

Progress: the exact Rust feedback loop first failed red at
`restore_installed=blocked reason=identity_contract` with a valid historical
snapshot and clean current host. It now passes green only with the exact
current-source-bound remediation authorization and rejects tampering before
USB. Admission-only validates all eight protected ranges without snapshots or
commands. The host owner implements fixed preflight/start/resume paths,
protected authorization/state/child receipts, current-safe admission, one
snapshot plus Wi-Fi seed, original-runtime proof, settings/theme restoration,
settings-only resume, independent projection validation, and closed evidence.
Focused Rust/TypeScript restoration and all flash tests pass. The later exact
hardware outcome is recorded below.

Remediation outcome: pushed source/package `276bb178` passed the complete
software gate set, exact no-effect admission, and fresh one-board detection.
The single remediation-001 invocation then stopped at `snapshot_restore` before
launching a write child. Its protected stderr digest exactly matches the local
`executor_program_mismatch`: the snapshot adapter rendered the required
managed `esptool.py` transaction, while the shared execution environment
accepts only `espflash`. The state remains `flash_started`, so settings-only
resume is ineligible and no second host invocation is authorized.

A fresh detector and same-session read-only runtime audit prove the unchanged
campaign package is running with the pinned reference, `mineonboot=false`,
mining `safe_blocked`, zero hashrate, and zero accepted/rejected shares. USB
cleanup passed. No restoration projection or `RESULT.md` exists, `STR-005`
remains `implemented`, and this task remains active but blocked at
`stop_authority_boundary`. Closure:
`docs/parity/work-plans/20260825T150417Z-STR-005-EXACT-RESTORATION/CLOSURE.md`.

Supersession review: Superseded by the decision-complete decomposition plan `docs/parity/work-plans/20260828T175218Z-STR-005-DECOMPOSITION/PLAN.md` and successor tasks task-str005-tcp-payload-205, task-str005-noise-auth-205, task-str005-v2-channel-job-205, task-str005-bm1366-share-205, task-str005-evidence-promotion. Existing plans, closures, evidence, attempt ordinals, and terminal decisions remain immutable; this archival does not change STR-005 from `implemented | unit,golden,workflow`.

### task-str005-restore-and-verify-continuation | 2026-08-25 | Restore Ultra 205 and verify STR-005

- [x] Add the exact managed-esptool USB executor with closed transfer
      diagnostics and real-child regression coverage.
- [ ] Restore recovery-006 plus exact settings/theme through fresh remediation
      ordinal 2 and independently publish the restoration projection.
- [x] Replace default-route fixture addressing with same-subnet admission and
      add closed Stratum V2 transport/fixture terminal diagnostics.
- [ ] Run the local authenticated fixture through fresh attempt-005 only after
      exact original-state admission; safe-stop and restore on every outcome.
- [ ] Continue only after a new regression-proved boundary; never retry an
      unchanged signature or reuse a sealed root.
- [ ] Promote only STR-005 after accepted Noise/channel/job/BM1366/share,
      safe-stop, cleanup, and exact restoration evidence.

Plan: `docs/parity/work-plans/20260825T215446Z-STR-005-RESTORE-AND-VERIFY/PLAN.md`

Authorization: repository diagnosis/fixes/tests/docs/build/package, Git
commit/push, task-gated USB and local-network fixture use, fresh progress-backed
restoration/campaign ordinals, exact recovery-006 writes, Wi-Fi NVS seed,
settings/theme restoration, conservative Ultra 205 mining, protected evidence,
and STR-005-only promotion under the linked immutable plan. External pools, raw
NVS/coredump reads, a new baseline, direct UART/pins, fault injection, OTA,
erase, arbitrary writes, other boards, and unbounded mining remain prohibited.

Initial boundary: remediation-001 stopped before launching a write child because
the snapshot adapter rendered managed `esptool.py` while the shared executor
accepted only `espflash`. Current device evidence remains campaign source
`78784a4a`, pinned reference, `mineonboot=false`, mining `safe_blocked`, and zero
hashrate/shares. The first eligible hardware action is remediation ordinal 2
after the exact executor boundary passes red/green, all gates, push, package,
and no-effect admission.

Verification: Pending.

Completion review: Pending. `STR-005` remains `implemented` until the full
hardware chain and exact final restoration are independently accepted.

Progress: the generic executor still rejects non-espflash programs; a private
managed-esptool type now admits only the exact snapshot transaction and runs it
through the existing USB lease, owned-child supervision, effect classifier,
reacquisition, retry, and cleanup. Separate protected snapshot/Wi-Fi receipts
retain pre-transfer, partial, complete, termination, count, and digest facts.
The real-child partial-transfer regression and standalone/campaign restore
authorizations pass. Attempt-005 now selects one non-tunnel host address on the
fresh device subnet, binds the fixture there, publishes seven closed firmware
transport details, retains eight fixture progress facts, and maps child,
fixture, and restoration failures to typed checkpoints. Focused tests, ordered
Cargo gates, Bright Builds, and all 55 Bazel tests pass. One combined automation
run exposed unrelated timing-sensitive child tests; their unchanged focused
reruns and the unchanged full Bazel rerun passed.

Restoration-002 outcome: exact pushed source/package `e3bd08bb`, admission-only
preflight, and both fresh detectors passed. The managed eight-range snapshot
write and separate Wi-Fi seed each completed and verified once with closed
`ready/completed/exited_success` diagnostics, one attempt, transfer started and
completed, no raw output, and clean USB reacquisition. The board now runs exact
original source `a11b579b`, app digest `32e2de54`, factory partition, and pinned
reference; exact settings/theme restoration completed with `mineonboot=false`,
zero hashrate, and zero shares.

The final projection was truthfully withheld at `final_runtime` because the
restored historical source reports `miningActivity=paused`, not `safe_blocked`.
Source inspection proves this is its intentional operator-paused projection
when `mineonboot=false`; forcing `safe_blocked` would require changing exact
settings or manufacturing a different blocker. The immutable plan requires both
facts, so the terminal result is `stop_impossible_contract`. Attempt-005 was not
created or run. This task remains active but blocked; STR-005 remains
`implemented`. Closure:
`docs/parity/work-plans/20260825T215446Z-STR-005-RESTORE-AND-VERIFY/CLOSURE.md`.

Supersession review: Superseded by the decision-complete decomposition plan `docs/parity/work-plans/20260828T175218Z-STR-005-DECOMPOSITION/PLAN.md` and successor tasks task-str005-tcp-payload-205, task-str005-noise-auth-205, task-str005-v2-channel-job-205, task-str005-bm1366-share-205, task-str005-evidence-promotion. Existing plans, closures, evidence, attempt ordinals, and terminal decisions remain immutable; this archival does not change STR-005 from `implemented | unit,golden,workflow`.

### task-str005-inactive-restoration-and-campaign-continuation | 2026-08-26 | Finalize restoration and verify STR-005

- [x] Finalize remediation-002 read-only with exact original identity,
      settings/theme, `mineonboot=false`, zero work, and closed inactive state.
- [x] Publish the independently validated restoration projection without any
      flash, NVS, settings, theme, or restart mutation.
- [x] Rebind the untouched attempt-005 local-fixture campaign and its exact
      restoration authorization to the new immutable plan.
- [x] Run attempt-005 once after full gates, clean push/package, fresh detector,
      no-effect preflight, and runtime admission.
- [x] Continue only after a new regression-proved boundary and exact original
      restoration; stop repeated, hardware, authority, or impossible outcomes.
- [ ] Promote only STR-005 after the complete accepted hardware chain and exact
      final inactive restoration.

Plan: `docs/parity/work-plans/20260826T135721Z-STR-005-INACTIVE-RESTORATION/PLAN.md`

Authorization: source/test/docs/build/package, Git commit/push, read-only
remediation-002 validation, local authenticated fixture/network use, one fresh
attempt-005, conservative Ultra 205 hardware work, safe-stop, exact recovery-006
restoration, protected evidence, and STR-005-only promotion. The plan permits no
restoration reflash before campaign, external pool, raw NVS/coredump read, new
baseline, direct UART/pins, fault injection, OTA, erase, arbitrary write, other
board, or unbounded mining.

Initial truth: remediation-002 completed exact original firmware and settings,
with `mineonboot=false`, zero hashrate/shares, and `miningActivity=paused` as the
historical source's intentional operator-paused state. The old literal
`safe_blocked` contract is terminal and cannot be reinterpreted. This task uses
the closed `mining_inactive` union (`paused` or `safe_blocked`) without weakening
identity, configuration, work, restoration, cleanup, or redaction evidence.

Verification: Pending.

Completion review: Pending. `STR-005` remains `implemented` until accepted
attempt-005 hardware evidence and exact final restoration are independently
validated.

Progress: a dedicated read-only finalizer validates the sealed remediation-002
state, authorization, successful snapshot/Wi-Fi diagnostics, exact bundle and
backup, current original runtime, every restorable setting, exact theme,
`mineonboot=false`, zero work/shares, and the closed expected category `paused`.
It exposes no mutation command and independently validates the v2 projection
with `mining_inactive=true`. Attempt-005 and its campaign restoration receipt
now bind this plan; final restoration accepts only `paused` or `safe_blocked`
with zero work. Focused tests, ordered Cargo gates, Bright Builds, and all 55
Bazel tests pass. One combined run hit the unchanged command-effects timing
test; its focused real-child rerun, isolated aggregate rerun, and unchanged full
suite all passed.

Read-only finalization outcome: exact pushed source/package `30376b18`, sealed
remediation-002 receipts, fresh one-board detection, qualified monitor, and
same-origin settings/theme reads passed. The independently validated v2
projection records exact original identity/restoration, `mineonboot=false`,
`mining_inactive=true`, category `paused`, zero hashrate/shares, cleanup, and
redaction. No flash, NVS, settings, theme, restart, fixture, campaign, ASIC, or
mining mutation occurred. Attempt-005 remains fresh and ineligible until this
projection is committed/pushed and the new exact package is rebuilt.

Attempt-005 preflight boundary: exact package `11a06443` and fresh detection
passed, but no-effect restore admission stopped before root, fixture, or device
effects because a stale four-file post-recovery allowlist rejected the later
authorized descendant source. Exact bundle, readiness projection, validator
receipt, recovery plan, current task/plan, clean package, and ancestry already
form independent trust gates; changed-file count is not a restoration invariant.
The focused regression now accepts a true descendant and rejects a non-ancestor.
Attempt-005 remains absent and requires the full gate/push/package cycle before
another preflight.

Remediation-003 outcome: exact pushed source/package `ea9f2622`, fresh
admission/detection, managed snapshot write, Wi-Fi seed, original identity,
settings/theme, inactive `paused` state, zero work/shares, cleanup, and
independent v2 projection validation all passed. Both write diagnostics are
closed, completed, first-attempt, and raw-output-free. The board is again on the
exact original firmware. Attempt-006 remains absent and ineligible until this
recovery projection is committed/pushed and its new exact campaign package is
built.

Attempt-006 outcome: exact package `45adb606`, both no-effect gates, and fresh
detection passed. The single campaign connected TCP—proving the listener-window
fix—but stopped at closed `transport/handshake`; the fixture reached
`connection_accepted` and stopped `noise`, with no channel/work/share. Safe-stop
and USB cleanup passed. Both rollback writes completed and the original firmware
is running, but nine named non-secret settings fields remain mismatched, so
restoration is not accepted and attempt-007 is ineligible.

Pinned upstream verifies the responder Schnorr signature without certificate
date checks. The official Rust Noise crate additionally checks the signed dates
against the ESP wall clock, while this firmware has no SNTP owner. The local
fixture's former host-now/300-second certificate can therefore fail on-device
despite correct authentication. The targeted parity fix issues a signed fixture
certificate from zero through `u32::MAX`; authentication remains mandatory and
the host regression verifies the extreme validity boundary. External pools and
production clock policy remain non-claims. Fresh remediation-004 must first
restore exact settings through roots `preflight-004`/`remediation-004` and its
own projection. Only then may fresh attempt-007 use root
`scratch/str005-stratum-v2/attempt-007`, after the full gate/push/package cycle.

Remediation-004 outcome: exact pushed source/package `305872e4`, both fresh
detectors, admission, first-attempt snapshot/Wi-Fi writes, original identity,
all settings/theme, inactive `paused` state, zero work/shares, cleanup, and
independent projection validation passed. The board is again exactly restored.
Attempt-007 remains absent and requires this projection to be committed/pushed
and a new exact package built before its no-effect gates.

Attempt-007 outcome: exact package `ec75e680`, fresh detector, preflight, and
runtime admission passed. The signed full-domain certificate still produced the
same authoritative `transport/handshake` plus fixture `noise` signature after
TCP acceptance; no channel/work/share occurred. Safe-stop and USB cleanup
passed. Because the exact post-fix signature repeated, the terminal campaign
decision is `stop_repeated_boundary`; attempt-008 and STR-005 promotion are
prohibited under this plan.

Rollback snapshot and Wi-Fi writes both completed, but final restoration again
reported false. No campaign retry is allowed. Fresh remediation-005 is the sole
remaining safety action: reuse the independently proven standalone restoration
owner with roots `preflight-005`/`remediation-005` and projection
`restoration-projection-remediation-005.json`, then close without campaign or
promotion. It requires the complete gate/push/package/detector cycle and may run
once only.

Attempt-005 outcome: changed preflight and runtime admission passed on exact
package `d54b7947`; the single campaign was consumed. Current package and NVS
writes completed, hardware preparation completed, and terminal safe-stop/USB
cleanup passed, but the device stopped `transport/connect` before channel/work.
The fixture terminal stayed at `listener_ready/accept`, proving no connection.
The selected host exactly matched `en0`, while the macOS firewall, block-all,
and stealth modes were disabled. The fixture's 120-second accept deadline began
before two package writes, reboot/reacquisition, and up to 60 seconds of firmware
preflight; it could close before the first device connection attempt. The new
regression binds the accept window to 300 seconds, still within the fixture's
existing maximum and campaign outer bound.

Attempt-005 restoration was attempted once but stopped pre-transfer because the
host passed the admitted absolute bundle path to the Rust adapter's exact
relative-path contract. No snapshot diagnostic was created. Fresh detector and
monitor prove the safe current package `d54b7947` with mining/control disabled.
The targeted fix passes `args.restoreBundle` to the child while retaining the
absolute path only for host-side reads. Before any campaign continuation,
remediation-003 must restore the original through fresh preflight/effect roots
and projection. After exact restoration, attempt-006 is the next eligible fresh
campaign ordinal with root `scratch/str005-stratum-v2/attempt-006`; both require
the complete gate/push/package/detector cycle and may each run once only.

Final restoration: remediation-005 on exact pushed source/package `28f9f1c2`
passed fresh admission/detection, snapshot and Wi-Fi writes, exact original
identity/settings/theme, inactive `paused`, zero work/shares, cleanup, and
independent projection validation. The board is safely restored. Campaign
attempts 005, 006, and 007 are consumed; attempt-007 repeated the post-fix
`transport/handshake` plus fixture `noise` signature, so the final outcome is
`stop_repeated_boundary`. No attempt-008, campaign projection, `RESULT.md`,
hardware-regression evidence, or promotion is authorized. STR-005 remains
`implemented`; this task remains active but blocked. Closure:
`docs/parity/work-plans/20260826T135721Z-STR-005-INACTIVE-RESTORATION/CLOSURE.md`.

Supersession review: Superseded by the decision-complete decomposition plan `docs/parity/work-plans/20260828T175218Z-STR-005-DECOMPOSITION/PLAN.md` and successor tasks task-str005-tcp-payload-205, task-str005-noise-auth-205, task-str005-v2-channel-job-205, task-str005-bm1366-share-205, task-str005-evidence-promotion. Existing plans, closures, evidence, attempt ordinals, and terminal decisions remain immutable; this archival does not change STR-005 from `implemented | unit,golden,workflow`.

### task-str005-noise-handshake-diagnostic | 2026-08-26 | Diagnose the STR-005 Noise handshake without mining

- [x] Add a typed Noise completion failure model and deterministic red/green
      coverage for message length, decrypt, public key, certificate time,
      certificate signature, state, and other failures.
- [x] Add a consume-before-use boot marker and sole no-mining firmware owner
      that exercises only Wi-Fi, TCP, Noise, and one encrypted client proof.
- [x] Add a handshake-only local fixture with closed progress and terminal
      receipts that distinguish both sides of the Noise exchange.
- [x] Add `just stratum-v2-noise-diagnostic preflight|start` with protected
      roots, public closed projection, exact recovery-006 restoration, and no
      campaign, ASIC, voltage, fan, or mining effects.
- [x] Run one fresh diagnostic ordinal only after all gates, clean push/package,
      no-effect preflight, and fresh one-board detection; continue only after a
      new authoritative signature receives a regression-proved fix.
- [ ] Close and archive only this diagnostic task after authenticated Noise and
      exact original restoration; keep STR-005 `implemented` and require a
      separate future campaign plan for verification.

Plan: `docs/parity/work-plans/20260826T210025Z-STR-005-NOISE-DIAGNOSTIC/PLAN.md`

Authorization: source/test/docs/build/package, Git commit/push, task-gated USB
and same-subnet local fixture use, fresh progress-backed diagnostic ordinals,
boot-time one-shot NVS marker writes, Wi-Fi/TCP/Noise-only device execution,
protected evidence, and exact recovery-006 firmware/settings restoration under
the linked immutable plan. Campaign attempts, ASIC work, voltage/fan mutation,
mining, external pools, raw NVS/coredump reads, new baselines, direct UART/pins,
fault injection, OTA, erase, arbitrary writes, other boards, and STR-005
promotion are prohibited.

Initial boundary: attempt-007 repeated the authoritative firmware
`transport/handshake` plus fixture `noise` signature after TCP acceptance even
with a signed full-domain certificate. The existing production adapters erase
the exact Noise completion error and responder substage, so another mining
campaign cannot add information. The Ultra 205 is exactly restored to source
`a11b579b`, app digest `32e2de54`, factory partition, pinned reference, exact
settings/theme, `mineonboot=false`, inactive `paused`, and zero work/shares.

Verification: Focused red/green Noise classification, real TCP fixture, exact
peer, partial/timeout act-one, process-group, validator launcher, NVS tuple,
historical restore-authority, and firmware source-ownership regressions pass.
Ordered Cargo gates, Bright Builds, all Bazel tests, canonical ESP32-S3 build
and packages, parity/progress, reference cleanliness, redaction, three fresh
detectors/preflights, and all three exact restorations pass.

Completion review: Terminal at `stop_repeated_boundary`. The diagnostic owner
proved a pre-Noise local transport blocker: the exact device peer completed TCP,
firmware reported a successful 64-byte act-one write, and the fixture received
zero bytes before timeout. No unchanged or further ordinal is eligible under
the plan. The task remains active but blocked; STR-005 remains `implemented`,
and no `RESULT.md`, hardware-regression evidence, archive transition, campaign,
or promotion exists. Closure:
`docs/parity/work-plans/20260826T210025Z-STR-005-NOISE-DIAGNOSTIC/CLOSURE.md`.

Progress: the exact Noise completion boundary now preserves seven closed
failure categories while production retains its stable public error surface.
The focused regression first failed by conflating an expired valid signature
with `certificate_signature`, then passed for certificate time/signature,
decrypt, length, and state. A consume-before-use `sv2diag*` tuple selects one
Wi-Fi/TCP/Noise-only owner ahead of self-test, production protocol, and the
normal fan; source-ownership tests prohibit ASIC, actuation, fan, voltage,
session, and mining reachability. The handshake-only fixture independently
records both acts and decrypts one encrypted client proof. The private flash
intent/NVS seed contains no campaign tuple, and the host owner binds exact clean
package/plan/recovery inputs, same-subnet fixture selection, managed child
process groups, closed projection validation, and recovery-006 restoration in
the post-effect path. Ordered Cargo gates, the real ESP32-S3 build, Bright
Builds, all Bazel tests, parity/progress, reference cleanliness, redaction, and
focused real-child regressions pass before the clean implementation commit.

Diagnostic-001 outcome: exact package `35cf3865`, fresh detection, no-effect
preflight, factory/NVS writes, sole diagnostic owner, exact recovery-006 writes,
settings/theme restoration, inactive original runtime, zero work, and cleanup
completed. The new closed signature is firmware `act_one_sent/act_two_read`
paired with fixture `connection_accepted/act_one_read`; neither side reached
authentication and no hardware/mining owner ran. The public projection was
withheld because the independent validator child path named the Bazel target
instead of its generated wrapper. A real-launch regression now resolves the
validator from runfiles or the exact Bazel wrapper. The fixture now rejects
non-device peers before consuming its sole session and records a bounded
unexpected-peer count. These targeted changes rebind fresh diagnostic ordinal 2
to root `scratch/str005-noise-diagnostic/diagnostic-002` and projection
`docs/parity/evidence/str005-noise-diagnostic/noise-diagnostic-projection-002.json`;
diagnostic-001 is sealed and will not be retried or reused.

Diagnostic-002 outcome: exact package `8470275c`, fresh detection/no-effect
preflight, diagnostic and recovery writes, exact original identity/settings,
inactive zero-work state, and cleanup passed. Exact peer admission observed the
device connection with zero unexpected peers, but the fixture again stopped at
`act_one_read`; the late-attached serial monitor retained no diagnostic stage or
terminal. The closed peer discriminator makes this a new signature rather than
an unchanged diagnostic-001 retry. The candidate failed only through the nested
validator launcher environment; the same independent validator accepted its
absolute protected candidate, and the redacted failed projection is published
as `noise-diagnostic-projection-002.json`. The validator now runs through its
repo-owned Bazel target. A ten-second post-Wi-Fi monitor-arm delay and a bounded
fixture act-one byte count with `complete/eof/timeout/io` classification bind
fresh diagnostic ordinal 3 to root `diagnostic-003` and projection
`noise-diagnostic-projection-003.json`. Diagnostic-002 is sealed.

Diagnostic-003 outcome: exact package `dc6e5f0c`, fresh detection/no-effect
preflight, the ten-second monitor arm, diagnostic and recovery writes, exact
original identity/settings, inactive zero-work state, cleanup, and independent
projection publication passed. Firmware retained `tcp_connected`,
`act_one_created`, and `act_one_sent` before `act_two_read`; the exact device
peer had zero unexpected peers and received exactly zero act-one bytes before a
typed `timeout`. This repeats the post-instrumentation transport boundary and
stops further ordinals. The accepted failed projection is
`docs/parity/evidence/str005-noise-diagnostic/noise-diagnostic-projection-003.json`.

Supersession review: Superseded by the decision-complete decomposition plan `docs/parity/work-plans/20260828T175218Z-STR-005-DECOMPOSITION/PLAN.md` and successor tasks task-str005-tcp-payload-205, task-str005-noise-auth-205, task-str005-v2-channel-job-205, task-str005-bm1366-share-205, task-str005-evidence-promotion. Existing plans, closures, evidence, attempt ordinals, and terminal decisions remain immutable; this archival does not change STR-005 from `implemented | unit,golden,workflow`.

### task-str005-preconnect-noise-and-verification | 2026-08-27 | Precompute Noise act one and verify STR-005

- [x] Add the fast delayed-preparation TCP regression and observe it fail before
      changing transport order.
- [x] Add `PreparedNoiseInitiator` and require resolution, Noise/act-one
      preparation, then TCP connection in diagnostic and production V2 paths.
- [x] Add bounded preparation/connect/write/read timings and rebind the
      no-mining workflow to diagnostic ordinal 4 with exact restoration.
- [ ] Prove authenticated local Noise and encrypted client proof on Ultra 205;
      continue only after new regression-proved signatures.
- [ ] Rebind and run local-fixture campaign attempt 008 only after diagnostic
      success is committed, pushed, and packaged.
- [ ] Promote only STR-005 after Noise/channel/job/BM1366/share, safe-stop,
      cleanup, exact restoration, independent validation, and redaction.

Plan: `docs/parity/work-plans/20260828T030951Z-STR-005-PRECONNECT-NOISE-VERIFY/PLAN.md`

Authorization: repository source/test/docs/build/package, Git commit/push,
task-gated USB and same-subnet authenticated fixture use, diagnostic ordinal 4,
campaign attempt 008, progress-backed fresh ordinals after verified fixes,
conservative Ultra 205 mining, protected evidence, and exact recovery-006
firmware/settings restoration. External pools, other boards, direct UART/pins,
raw NVS/coredump access, new baselines, fault injection, OTA, erase, arbitrary
writes, and unbounded mining remain prohibited.

Initial boundary: diagnostic 003 proved the exact device peer completed TCP,
firmware reached `act_one_sent`, and the fixture received zero act-one bytes
before its ten-second timeout. The Rust path currently creates the secp256k1
context/keypair and ElligatorSwift act one only after connecting. The first
eligible change is a red/green pre-connect-order regression; no hardware ordinal
is eligible until that fix passes every gate and is committed/pushed/packaged.

Verification: Required red/green delayed-preparation TCP loop, prepared-Noise
observer, fixture peer/byte/category/timing, diagnostic marker/timing parser,
projection validator, restore authorization, firmware order ownership, real
child, ordered Cargo, Bright Builds, all Bazel, canonical ESP32-S3 build/package,
parity/progress, reference, redaction, fresh detector/preflight, diagnostic 4,
exact restoration, and post-run detection all passed at their claimed scopes.

Completion review: Terminal at `stop_repeated_boundary`. Precomputing Noise
before connect made the exact fast regression pass but did not change the Ultra
205 boundary: the exact device peer connected and the fixture received zero
bytes before timeout. The plan explicitly makes that post-fix repetition
terminal. Diagnostic ordinal 5 and campaign attempt 008 were not created or
run. STR-005 remains `implemented`; no `RESULT.md`, hardware-regression evidence,
archive transition, or promotion exists. Closure:
`docs/parity/work-plans/20260828T030951Z-STR-005-PRECONNECT-NOISE-VERIFY/CLOSURE.md`.

Progress: the required real-TCP loop first reproduced the authoritative split
with a 25 ms responder deadline and 75 ms post-connect preparation: zero of 64
act-one bytes arrived. Reversing only the generic effect seam to prepare before
connect made the same command pass. A non-debuggable `PreparedNoiseInitiator`
now owns the initialized state and exact act one; both production and diagnostic
firmware consume the shared order seam before any connector call. Diagnostic
markers retain bounded keypair, act-one, connect, write, and read durations;
the fixture retains exact peer, byte count, read category, first-byte, and total
read timing. Ordinal 4, its protected restore authority, the new immutable plan,
and independent projection contract are rebound. Focused protocol, fixture,
firmware ownership/build, flash, validator, and real-child tests pass.

Diagnostic-004 outcome: exact package `a13d91b8`, fresh detection/no-effect
preflight, diagnostic and recovery writes, exact original identity/settings,
inactive zero-work state, cleanup, independent failed projection, and post-run
detection passed. The exact peer connected with zero unexpected peers, but the
fixture again received zero act-one bytes and timed out; no firmware diagnostic
stage or bounded timing marker was retained. This is the plan's explicit
repeated zero-byte-after-precomputation stop condition. The accepted failed
projection is `docs/parity/evidence/str005-noise-diagnostic/noise-diagnostic-projection-004.json`.

Supersession review: Superseded by the decision-complete decomposition plan `docs/parity/work-plans/20260828T175218Z-STR-005-DECOMPOSITION/PLAN.md` and successor tasks task-str005-tcp-payload-205, task-str005-noise-auth-205, task-str005-v2-channel-job-205, task-str005-bm1366-share-205, task-str005-evidence-promotion. Existing plans, closures, evidence, attempt ordinals, and terminal decisions remain immutable; this archival does not change STR-005 from `implemented | unit,golden,workflow`.

### task-str005-verification-decomposition | 2026-08-28 | Formally decompose STR-005 verification

- [x] Create immutable decomposition plan
      `docs/parity/work-plans/20260828T175218Z-STR-005-DECOMPOSITION/PLAN.md`.
- [x] Add a supersession review to and archive the ten accumulated STR-005
      implementation, recovery, remediation, diagnostic, and campaign records
      without changing their historical plans, evidence, ordinals, or terminal
      decisions.
- [x] Add five dependency-ordered successor tasks with only the TCP payload
      boundary active and every later task dependency-blocked under `Future`.
- [x] Preserve the STR-005 checklist status and progress totals, leave old
      effect commands ineligible, and introduce no firmware or hardware effect.

Authorization: task tracker, immutable plan, task archive, verification,
commit, and push only. Firmware changes, new command authority, hardware,
network, package, flash, NVS, mining, evidence creation, `RESULT.md`, and parity
promotion were prohibited.

Verification: The successor and archived task IDs are unique across the active
and archived trackers; exact-record comparison proves all ten native records
changed only by their appended supersession review. Only
`task-str005-tcp-payload-205` is active, and every later child records its exact
dependency and effect boundary. Ordered Cargo format, Clippy, build, and test;
Bright Builds; all 56 Bazel tests; parity; progress; pinned-reference;
redaction; closure-digest; and diff checks passed. The STR-005 checklist row
remains `implemented | unit,golden,workflow`, existing effect commands remain
bound to archived authorities, and no hardware command was added or run.

Completion review: Complete. STR-005 verification is now split into TCP
payload, Noise authentication, channel/job, BM1366 accepted-share, and
evidence-promotion tasks. Recovery-006 and the exact-restoration machinery
remain shared prerequisites. Diagnostic projections cannot substitute for the
final same-run cumulative campaign, and campaign ordinal `attempt-008` remains
unconsumed. STR-005 remains `implemented` with `unit,golden,workflow`. Residual
risk: the additional `parity next-item` probe still rejects two pre-existing
immutable STR-005 closures whose historical metadata uses `Final parity status`
instead of the parser's `Final status`; this plan preserves those closures
unchanged and does not rely on that selector for the new task graph.

### task-parity-legacy-closure-admission | 2026-08-28 | Unblock parity next-item on immutable STR-005 closures

- [x] Reproduce the exact `next-item --format json` failure on the clean synced
      repository and retain it as the end-to-end red/green boundary.
- [x] Add regression coverage for exact admission of the two historical
      STR-005 closures and rejection of path, plan, closure, metadata, and
      canonical-schema downgrade drift.
- [x] Add a narrow digest-pinned compatibility path without changing either
      historical closure or weakening canonical closure validation.
- [x] Pass focused and full repository gates, prove `next-item` returns valid
      JSON, archive this task, commit, and push.

Authorization: repository task, Rust parser/tests, build graph when required,
verification, task archive, commit, and push only. Historical plans, closures,
evidence, checklist fields, progress history, firmware, network, USB, hardware,
credentials, mining, and parity promotion remained unchanged.

Verification: The exact parser regression first failed with `parity plan
closure requires exactly one concrete final status`, then passed for both real
historical closure/plan pairs. Focused tests accept the complete legacy plus
canonical decomposition lineage and reject directory, plan-byte,
closure-byte, task, row, status, terminal-decision, mixed-schema, and caller
identity drift. Ordered Cargo format, Clippy, build, and all-feature tests;
Bright Builds; all 56 Bazel tests; parity; progress; `next-item --format json`;
pinned-reference; redaction; and diff checks passed. Cargo verification used an
isolated task target after the pre-existing default macOS cache stalled in
uninterruptible I/O. The original command now returns valid JSON with no open
plan and five candidates, including STR-005.

Completion review: Complete. Canonical closure validation is unchanged. A
private compatibility module admits only the two exact STR-005 historical
records by repository-relative plan directory, plan digest, closure digest,
row, status, task, and terminal decision; any copied or changed legacy record
fails closed. The historical artifacts and parity checklist remain unchanged,
and no parity row, firmware, network, or hardware work occurred.

### task-str005-tcp-payload-205 | 2026-08-28 | Prove Ultra 205 TCP payload delivery

- [x] Create a separate immutable execution plan before implementation,
      network use, package installation, or hardware effects.
- [x] Prove the exact Ultra 205 peer connects to the admitted same-subnet
      fixture and delivers one fixed 64-byte non-secret canary.
- [x] Retain typed connect, write, peer, byte-count, and fixture-receipt
      boundaries in a closed independently validated projection.
- [x] Restore the exact recovery-006 firmware/settings state and prove inactive
      zero-work runtime plus USB/process cleanup.

Depends on: the archived STR-005 lineage and decomposition plan
`docs/parity/work-plans/20260828T175218Z-STR-005-DECOMPOSITION/PLAN.md`.

Plan: `docs/parity/work-plans/20260828T185251Z-STR-005/PLAN.md`

Connection-identity continuation plan:
`docs/parity/work-plans/20260829T032813Z-STR-005-CONNECTION-IDENTITY/PLAN.md`

Authorization: planning, repository implementation, tests, build/package,
commit, and push after a child plan exists. No hardware or network effect is
eligible until that immutable plan defines its exact command, evidence,
privacy, recovery, retry, and stop contract. The task-local hardware namespace
begins at `diagnostic-001`. Noise, Stratum V2 protocol messages, ASIC, fan,
voltage, mining, external pools, other boards, direct UART/pins, raw
NVS/coredump access, fault injection, OTA, erase, and arbitrary writes remain
excluded.

Diagnostic-001 command contract: after the immutable plan and a separate
implementation/evidence-contract commit are clean, fully verified, committed,
and pushed, run `just package`, then `just detect-ultra205`, then exactly one
repo-owned command:

`just stratum-v2-tcp-payload start --board 205 --port <detector-port> --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --restore-bundle scratch/str005-installed-package-recovery/recovery-006/restore-bundle.private.json --private-parent scratch/str005-tcp-payload/diagnostic-001 --projection docs/parity/evidence/str005-tcp-payload/tcp-payload-projection-001.json --plan docs/parity/work-plans/20260828T185251Z-STR-005/PLAN.md --diagnostic-ordinal 1 --capture-timeout-seconds 360 --redact-evidence`

The command may flash the exact package, seed only private Wi-Fi plus a
generated same-subnet fixture endpoint and bounded diagnostic lease, accept one
exact-peer TCP connection, send the fixed public byte sequence `0x00..0x3f`,
receive-only monitor the result, and execute exact recovery-006
firmware/settings restoration. Mining, Noise, V2 messages, ASIC work, fan,
voltage, thermal/power control, external pools, discovery, other boards,
direct UART/pins, raw NVS/coredump access, fault injection, OTA, erase, and
arbitrary writes remain prohibited.

Evidence/privacy: the absent private parent is mode `0700`; supervisor-owned
children and distinct stdout/stderr siblings are mode `0600`, secret-sanitized,
ignored, sealed, and never promoted. Credential values never reach disk or
terminal. The public projection contains only closed categories, booleans,
bounded counts/timings, safe digests/provenance, restoration/cleanup truth, and
`redaction_status: passed`, and must pass an identity-bound independent
validator.

Recovery/retry/stop: preserve the earliest typed failure, always restore the
exact recovery-006 package/settings/appearance plus `mineonboot=false`, prove
inactive zero-work runtime, fresh board admission, USB/holder cleanup, and zero
owned processes. Diagnostic-001 runs once. No unchanged retry is allowed; a
later ordinal requires a distinct closed signature and regression-backed fix or
objective authorized remediation. Accepted evidence completes only this child;
STR-005 remains `implemented | unit,golden,workflow` until the final cumulative
share campaign and promotion task.

Diagnostic-001 outcome: `continue_after_verified_fix`. The protected workflow
stopped at `timeout:fixture_ready` before listener readiness, credential access,
flash, NVS seed, device/network effect, or projection publication. Its private
stderr confirms the fixture rejected the accidental 360-second session timeout
against its 300-second maximum. The root is consumed and must not be reused.

Diagnostic-002 continuation: the targeted fix routes the production fixture
launcher through a tested argument constructor with a 120-second session
timeout while retaining the separate 360-second monitor capture. After the fix
is fully verified, committed, pushed, and repackaged from clean exact HEAD, run
fresh detector admission and exactly one command:

`just stratum-v2-tcp-payload start --board 205 --port <detector-port> --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --restore-bundle scratch/str005-installed-package-recovery/recovery-006/restore-bundle.private.json --private-parent scratch/str005-tcp-payload/diagnostic-002 --projection docs/parity/evidence/str005-tcp-payload/tcp-payload-projection-002.json --plan docs/parity/work-plans/20260828T185251Z-STR-005/PLAN.md --diagnostic-ordinal 2 --capture-timeout-seconds 360 --redact-evidence`

All allowed/prohibited effects, privacy, exact restoration, retry, stop, and
non-promotion boundaries remain unchanged. Recurrence of the same fixture-bound
signature selects `stop_repeated_boundary`; no diagnostic-003 is authorized.

Recovery-001 iterative continuation: the user explicitly resumed this task
under the repository's iterative hardware-fix authorization. After the exact
current recovery admission and recovery-only supervisor pass every required
gate, are committed and pushed, and `just package` binds the clean source, run
fresh detector admission and exactly one command:

`just stratum-v2-tcp-payload recover --board 205 --port <detector-port> --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --restore-bundle scratch/str005-installed-package-recovery/recovery-006/restore-bundle.private.json --private-parent scratch/str005-tcp-payload/recovery-001 --projection docs/parity/evidence/str005-tcp-payload/tcp-payload-projection-002.json --plan docs/parity/work-plans/20260828T185251Z-STR-005/PLAN.md --diagnostic-ordinal 2 --capture-timeout-seconds 360 --redact-evidence`

Recovery-001 may only admit the exact Ultra 205, restore the recovery-006
package ranges, restore the protected settings backup using the ignored local
Wi-Fi/pool inputs, confirm exact runtime identity, `mineonboot=false`, inactive
zero-work/share state, and USB/process cleanup. It must not start a fixture,
send TCP bytes, enter Noise or V2 protocol, touch ASIC/fan/voltage controls,
mine, discover targets, mutate unrelated settings, or publish parity evidence.
The absent private root is mode `0700` with mode-`0600` secret-sanitized files.
No unchanged recovery retry is allowed; a distinct regression-backed fix is
required after any typed failure.

Recovery-001 outcome: `continue_after_verified_fix`. Fresh detector admission
passed, but the restore child stopped before writes at
`restore_installed=blocked reason=authorization_action`. The current plan/root
mapping was accepted; a second inline action/root allowlist omitted
`tcp_payload_recovery`. Recovery-001 is consumed and produced no restore result.

Recovery-002 continuation: a focused production-seam regression now proves the
action is admitted only with the current recovery root. After the fix is fully
verified, committed, pushed, and packaged from clean exact HEAD, run fresh
detector admission and exactly one command:

`just stratum-v2-tcp-payload recover --board 205 --port <detector-port> --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --restore-bundle scratch/str005-installed-package-recovery/recovery-006/restore-bundle.private.json --private-parent scratch/str005-tcp-payload/recovery-002 --projection docs/parity/evidence/str005-tcp-payload/tcp-payload-projection-002.json --plan docs/parity/work-plans/20260828T185251Z-STR-005/PLAN.md --diagnostic-ordinal 2 --capture-timeout-seconds 360 --redact-evidence`

The recovery-only effects, privacy, restoration acceptance, prohibitions, and
no-unchanged-retry boundaries remain exactly those declared for recovery-001.

Recovery-002 outcome: `complete`. Exact recovery-006 package identity and
protected settings were restored and confirmed with `mineonboot=false`,
inactive mining, zero work/share state, fresh final detector admission, and
complete cleanup. The task may resume TCP diagnosis from this proved baseline.

Diagnostic-003 continuation: the diagnostic child now retains bounded partial
stdout/stderr and parses closed firmware stages, timings, and terminal markers
even when the managed monitor reaches its timeout. Current-task diagnostic
restoration is admitted before any new effect. After all gates pass, the source
is committed/pushed, and an exact clean package is built, run fresh detector
admission and exactly one command:

`just stratum-v2-tcp-payload start --board 205 --port <detector-port> --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --restore-bundle scratch/str005-installed-package-recovery/recovery-006/restore-bundle.private.json --private-parent scratch/str005-tcp-payload/diagnostic-003 --projection docs/parity/evidence/str005-tcp-payload/tcp-payload-projection-003.json --plan docs/parity/work-plans/20260828T185251Z-STR-005/PLAN.md --diagnostic-ordinal 3 --capture-timeout-seconds 360 --redact-evidence`

Diagnostic-003 reuses the original fixed-payload effect and all safety/privacy
limits. Its purpose is to distinguish monitor-armed, resolve, connect,
configure/write, terminal, fixture receipt, and restoration boundaries; it does
not authorize Noise, protocol, ASIC, mining, promotion, or unchanged retries.

Diagnostic-003 outcome: `continue_after_verified_fix`. The supervised factory
flash stopped before transfer, so the firmware owner and TCP fixture were never
reached. Exact recovery-006 restoration still completed. Partial child output
proved that the flash/NVS commands were rendered and USB ownership opened, but
no monitor command ran; the sanitized failure was
`flash_failed_before_transfer`.

Diagnostic-004 continuation: the inner command now emits the closed USB command
diagnostic on flash failure, and pre-monitor child failure terminates the
fixture immediately instead of waiting its full accept deadline. After all
gates pass and the exact source is committed/pushed/packaged, run fresh detector
admission and exactly one command:

`just stratum-v2-tcp-payload start --board 205 --port <detector-port> --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --restore-bundle scratch/str005-installed-package-recovery/recovery-006/restore-bundle.private.json --private-parent scratch/str005-tcp-payload/diagnostic-004 --projection docs/parity/evidence/str005-tcp-payload/tcp-payload-projection-004.json --plan docs/parity/work-plans/20260828T185251Z-STR-005/PLAN.md --diagnostic-ordinal 4 --capture-timeout-seconds 360 --redact-evidence`

All original payload, privacy, safety, restoration, and non-promotion limits
remain. Recurrence of the same closed flash-before-transfer signature stops as
a repeated hardware/host boundary; it is not TCP evidence and does not permit
diagnostic-005 without a targeted boundary fix.

Diagnostic-004 outcome: `continue_after_verified_fix`. The exact-peer fixture
accepted the device connection, while the firmware emitted `monitor_armed`,
`resolved`, `tcp_connected`, and `payload_sent` with a 158 ms connect and 0 ms
buffer write. The fixture then timed out with zero of 64 bytes and no extras.
The diagnostic child retained an accepted firmware terminal but remained open
for the bounded monitor capture, so the private candidate was conservatively
rejected. Exact recovery-006 identity/settings, inactive zero-work state, and
cleanup all passed. This is a distinct post-write TCP boundary, not a repeat of
the diagnostic-003 flash failure.

Diagnostic-005 continuation: the targeted transport fix keeps the firmware
socket alive until the fixture has validated exactly `0x00..0x3f`, observed no
extra byte, and returned one fixed non-secret `0xa5` receipt on the same exact
peer connection. The firmware reads only that one receipt byte and emits a
closed `receipt_acknowledged` stage; the fixture records only the boolean
`receipt_ack_sent`. A focused real-loopback regression proves the receipt
round-trip, and a supervisor regression proves that complete firmware and
fixture evidence remains eligible when only the intentionally bounded monitor
process times out. After every gate passes and the exact source is
committed/pushed/packaged, run fresh detector admission and exactly one command:

`just stratum-v2-tcp-payload start --board 205 --port <detector-port> --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --restore-bundle scratch/str005-installed-package-recovery/recovery-006/restore-bundle.private.json --private-parent scratch/str005-tcp-payload/diagnostic-005 --projection docs/parity/evidence/str005-tcp-payload/tcp-payload-projection-005.json --plan docs/parity/work-plans/20260828T185251Z-STR-005/PLAN.md --diagnostic-ordinal 5 --capture-timeout-seconds 360 --redact-evidence`

The single fixed receipt is only a delivery acknowledgment, not Noise or a
Stratum V2 message. Every original privacy, exact restoration, safety,
prohibition, cleanup, non-promotion, and no-unchanged-retry boundary remains.
Failure before receipt, recurrence of zero received bytes, or incomplete
restoration selects a typed stop/next-fix outcome and cannot produce accepted
evidence.

Diagnostic-005 outcome: `stop_repeated_boundary` for the receipt-only
hypothesis. From clean pushed source and fresh one-board admission, firmware
again reached monitor armed, resolve, exact-peer connect, and payload-buffer
write, then failed at the new `receipt` boundary. The fixture again accepted
the exact peer but timed out with zero of 64 bytes, no extras, and no receipt.
Exact recovery-006 identity/settings, inactive zero-work state, fresh USB
admission, and owned-process cleanup all passed. This disproves immediate
socket destruction as the sole cause; the receipt-only approach must not be
retried unchanged.

Diagnostic-006 continuation: under the user's standing iterative hardware-fix
authorization, the next targeted discriminator gives the fixed one-way payload
an explicit TCP completion boundary. The firmware mirrors the proven
production transport's `TCP_NODELAY` configuration, writes only `0x00..0x3f`,
then half-closes only the socket write direction before waiting for the fixed
`0xa5` receipt. The fixture requires exactly 64 matching bytes followed by EOF
and returns the receipt on the still-open read direction. A red production-seam
regression failed before this sequence existed, then passed with exact
write-before-half-close-before-receipt ordering; real loopback also proves the
half-close/receipt round trip. After every gate passes and the exact source is
committed/pushed/packaged, run fresh detector admission and exactly one command:

`just stratum-v2-tcp-payload start --board 205 --port <detector-port> --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --restore-bundle scratch/str005-installed-package-recovery/recovery-006/restore-bundle.private.json --private-parent scratch/str005-tcp-payload/diagnostic-006 --projection docs/parity/evidence/str005-tcp-payload/tcp-payload-projection-006.json --plan docs/parity/work-plans/20260828T185251Z-STR-005/PLAN.md --diagnostic-ordinal 6 --capture-timeout-seconds 360 --redact-evidence`

The half-close affects only this one admitted local fixture socket and cannot
touch storage or hardware controls. Every prior privacy, restoration, safety,
prohibition, cleanup, non-promotion, and no-unchanged-retry boundary remains.
Diagnostic-006 stops at its typed result; it does not authorize a seventh
hardware attempt without another distinct evidence-backed change.

Diagnostic-006 outcome: `continue_after_verified_discriminator`. Firmware
reached exact-peer connect and payload-buffer write, then failed immediately at
the new `shutdown` boundary before `write_half_closed`; connect took 179 ms and
the buffer write reported 0 ms. The exact-peer fixture again received zero
bytes. Exact recovery-006 identity/settings, inactive zero-work state, USB
cleanup, and zero owned processes all passed. The failure arrived about 30 ms
after the write marker, excluding the configured ten-second send timeout and
isolating the next question to the shutdown error class.

Diagnostic-007 continuation: without changing any device, network, payload,
socket, restoration, or cleanup effect, map the write-half-close failure into
one of six closed value-free categories: `shutdown_would_block`,
`shutdown_not_connected`, `shutdown_out_of_memory`, `shutdown_invalid_input`,
`shutdown_unsupported`, or `shutdown_other`. A red source regression requires
the complete closed vocabulary and passes only after the production owner maps
`std::io::ErrorKind` without logging raw errno or socket values. After every
gate passes and exact source is committed/pushed/packaged, run fresh detector
admission and exactly one command:

`just stratum-v2-tcp-payload start --board 205 --port <detector-port> --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --restore-bundle scratch/str005-installed-package-recovery/recovery-006/restore-bundle.private.json --private-parent scratch/str005-tcp-payload/diagnostic-007 --projection docs/parity/evidence/str005-tcp-payload/tcp-payload-projection-007.json --plan docs/parity/work-plans/20260828T185251Z-STR-005/PLAN.md --diagnostic-ordinal 7 --capture-timeout-seconds 360 --redact-evidence`

Diagnostic-007 is classification-only and admits no broader effect. Every prior
privacy, exact restoration, safety, prohibition, cleanup, non-promotion, and
no-unchanged-retry boundary remains. Its result must select a software fix or a
typed stop; no raw OS error, endpoint, credential, or device identity may be
promoted.

Diagnostic-007 outcome: `continue_after_verified_capture_fix`. Fresh one-board
admission and the exact package passed, and the exact-peer fixture accepted one
device connection, but the bounded serial child retained no TCP diagnostic
stage, timing, or terminal marker. The closed result therefore remained
`timeout`; it neither classified nor contradicted diagnostic-006's shutdown
failure. The fixture received zero bytes, no public projection was published,
and exact recovery-006 identity/settings, inactive zero-work runtime, USB
cleanup, and zero owned processes all passed. The next attempt must first make
post-flash monitor attachment deterministic enough to retain the classifier;
an unchanged diagnostic-007 retry is prohibited.

Diagnostic-008 continuation: diagnostic-006 captured its first firmware line
at 2,136 ms uptime, while diagnostic-007 attached only at 42,186 ms—well after
the approximately 17.5-second shutdown terminal. Replace reliance on the
arbitrary ten-second countdown with boot-lifetime replay of the complete closed
diagnostic transcript. After the initial terminal, the sole diagnostic owner
replays every completed stage, bounded timing, and terminal marker every five
seconds through 120 seconds; it performs no additional socket, fixture,
credential, storage, hardware, or control effect. A virtual-time regression
starts red with no replay, then proves the measured 42,186 ms late attachment
observes a complete replay at 45,000 ms and that exactly 24 replay slots end at
120,000 ms. After every gate passes and exact source is
committed/pushed/packaged, run fresh detector admission and exactly one command:

`just stratum-v2-tcp-payload start --board 205 --port <detector-port> --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --restore-bundle scratch/str005-installed-package-recovery/recovery-006/restore-bundle.private.json --private-parent scratch/str005-tcp-payload/diagnostic-008 --projection docs/parity/evidence/str005-tcp-payload/tcp-payload-projection-008.json --plan docs/parity/work-plans/20260828T185251Z-STR-005/PLAN.md --diagnostic-ordinal 8 --capture-timeout-seconds 360 --redact-evidence`

Diagnostic-008 changes only observation durability. The payload/socket effect,
closed shutdown classifier, exact restoration, privacy, safety, prohibitions,
cleanup, non-promotion, and no-unchanged-retry boundaries remain unchanged.
Replay contains only the existing value-free public marker schema and stops
before the bounded monitor horizon.

Diagnostic-008 outcome: `complete_capture_fix_and_stop_transport_boundary`.
The first isolated invocation stopped before effects because a committed
recovery projection checked out at mode `0644`; applying its required local
`0600` mode made the full read-only preflight pass. The admitted hardware run
then retained one original plus all 24 replayed copies of every completed
marker: monitor armed, resolve, exact-peer connect, zero-millisecond local
buffer write, payload sent, write-half-close, and terminal `receipt`. Connect
took 257 ms. This proves late monitor attachment is fixed and that
`shutdown(Write)` succeeded in this run. The exact-peer fixture nevertheless
timed out with zero of 64 bytes, no extras, and no receipt, so payload delivery
remains unproved and no public projection was published.

The diagnostic's inline restore stopped before writes because the isolated
worktree's generated ESP-IDF tools canonicalized outside that workspace. Two
recovery-only continuations also stopped before writes while successively
identifying the local esptool-containment and NVS-Python dependencies. After
installing real ignored local copies of those exact managed tools, the
recovery-only supervisor completed exact recovery-006 identity/settings,
`mineonboot=false`, inactive zero-work runtime, fresh device admission, and
cleanup. No unchanged hardware diagnostic retry is authorized. The next work
must explain how the ESP-IDF socket can report a complete write and successful
write half-close while the admitted fixture receives zero bytes.

Diagnostic-009 continuation: the immutable connection-identity plan hardens
all recovery tooling before effects, privately joins the firmware local TCP
port to a bounded three-candidate exact-peer fixture inventory, publishes only
closed tuple/count/correlated-receipt fields in a v2 projection, and records
locally reported bytes independently of terminal acceptance. After a distinct
implementation commit is fully verified, pushed, and packaged, run its exact
ordinal-9 command from the continuation plan once. Diagnostic-010 is prohibited
unless diagnostic-009 proves exactly one tuple-matched connection with zero
correlated bytes and complete restoration.

Diagnostic-009 outcome: `complete`. Exact clean source `e0398abb` produced one
and only one exact-peer connection; the replayed firmware local port matched
the fixture remote port, all pre-send/post-send/post-shutdown socket error
families were `none`, the standard adapter reported all 64 bytes, and the
correlated fixture candidate received exact `0x00..0x3f`, observed no extras,
and returned the fixed receipt. Inline restore completed both write boundaries
but its final USB sampler reported `identity_drift`; fresh recovery-003 then
proved exact recovery-006 identity/settings, `mineonboot=false`, inactive
zero-work/share state, fresh device admission, and cleanup. A recovery-aware
offline finalizer independently validated and published the mode-`0644` v2
projection without another hardware effect. Diagnostic-010 is not eligible and
must not run.

Verification: diagnostic-001 stopped before effects at
`timeout:fixture_ready`; its regression-backed fixture-timeout fix passed every
gate and was pushed as `35ae9cb33458ad6c76f6eedef5d0538720d80367`.
Diagnostic-002 used that exact package and fresh one-board admission, then
sealed `payload_read` with zero of 64 bytes before the mandatory restore path
stopped at `hardware_blocked:restoration`. No public projection was published.

Completion review: complete at diagnostic-009. The accepted projection joins
exact clean package identity, one correlated TCP connection, exact fixed
payload and receipt, bounded replay, socket state, recovery-003, cleanup,
independent validation, and redaction. Residual non-claims are Noise and V2
messages, authentication, channel/job/share behavior, mining, ASIC/fan/voltage
effects, soak, other boards, and parity promotion. STR-005 remains
`implemented` with `unit,golden,workflow`; continue only through the separate
Noise-authentication child.

# Parity work plan

- Run ID: `20260802T214207Z-STAT-004`
- Parity row: `STAT-004`
- Initial status: `implemented`
- Source commit: `90d7f7b5c1d6ccb5d30caecd1ca4ca41d0b7d2fc`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-stat-004-work-queue-verification`

## Selection

`bazel run //tools/parity:report -- next-item --format json` reported no open
plan. It ranked the following candidates before `STAT-004`; each is ineligible
for this invocation:

- `CFG-001`: the remaining frequency/voltage-default claim is safety-critical.
  Existing mining-soak authorizations are consumed and explicitly prohibit
  parity promotion; no row-specific promotable hardware contract exists.
- `CFG-005`: the production PATCH route deliberately excludes broader
  voltage, frequency, fan, and credential writes, so the pure update model
  does not close the effectful runtime-settings row.
- `NET-001`: retry, fallback, IPv6, and long-running reconnect behavior retains
  explicit network and hardware evidence gaps without a row-specific contract.
- `ASIC-002`, `ASIC-003`, `ASIC-004`, `ASIC-005`, and `ASIC-007`: full
  initialization, production dispatch, live result parsing, loaded serial
  transport, and frequency transitions retain explicit hardware-regression
  gaps.
- `STR-001`, `STR-006`, and `STR-007`: live socket, coordinator, watchdog, and
  soak behavior retain detector-gated runtime evidence gaps; the last soak
  authorization is consumed and closed at a repeated boundary.
- `API-002`, `API-003`, and `API-009`: full live response fields, broad PATCH
  effects, and device commands retain firmware-effect or hardware evidence
  gaps.
- `PWR-001`, `PWR-002`, `PWR-003`, `PWR-005`, and `PWR-006`: reset, power,
  voltage, DS4432U, and INA260 behavior are safety-critical and lack a current
  row-specific hardware-regression contract.
- `THR-001`, `THR-002`, and `THR-003`: thermal, fan, and PID rows are covered
  by the checklist's fan/thermal hardware-evidence gate; no promotable fan
  actuation contract exists, and the PID audit also found unclosed reference
  math differences.
- `SELF-001`: self-test hardware modes remain unsupported and require their own
  bounded hardware-regression contract.
- `LOG-001`: no-init header validation and soft-reboot retention are not
  implemented by the firmware buffer, and live firmware logging lifecycle
  evidence remains absent.

`STAT-004` is the first bounded software-verifiable candidate. The pinned
queue surface has deterministic storage semantics: capacity twelve, FIFO
ordering including ring wrap-around, clear/drop behavior, and empty/full
boundaries. Blocking, timed waiting, task wakeups, and scheduler behavior are
owned by unverified `SYS-005`, while clean-jobs mining-generation behavior is
owned by `STR-003`, `STR-006`, and the production-runtime rows.

## Scope and non-scope

Make the reference-derived work-queue contract executable for the Rust queue:
initial empty state, exact capacity, FIFO ordering across reuse, lossless
full-boundary rejection for shell-owned backpressure, empty-boundary signaling,
and complete clear/drop semantics. Preserve the existing mining queue's
clean-jobs and valid-job invalidation coverage as supporting integration proof.

Do not modify the pinned reference tree, copy GPL source expression, add
threads or timing behavior to the pure queue, open network connections, access
credentials, touch hardware, dispatch ASIC work, classify shares, or claim
condition-variable timing, FreeRTOS scheduling, live mining, or hardware
parity. The plan follows repository task/archive and evidence-integrity
guidance plus the loaded Bright Builds architecture, code-shape, verification,
testing, and Rust standards.

## Implementation

- [ ] Add a pinned, provenance-bearing `STAT-004` golden fixture for the exact
      queue data-structure boundary and its explicit scheduler non-claims.
- [ ] Add behavior-focused fixture tests for capacity, FIFO wrap-around,
      full/empty boundary preservation, and clear/drop ownership.
- [ ] Strengthen mining-queue invalidation coverage only where it supports the
      selected row without widening into live session behavior.
- [ ] Record exact commands, evidence, non-claims, and residual risks in
      `WORKLOG.md` and a terminal `RESULT.md` if every promotion criterion
      passes.

## Verification and promotion

Focused acceptance commands:

- `cargo test -p bitaxe-stratum work_queue --all-features`
- `bazel test //crates/bitaxe-stratum:tests`
- `bazel run //scripts:verify_reference_clean`
- `jq empty crates/bitaxe-stratum/fixtures/v1/work-queue-cases.json`

The mandatory Rust checks, managed Bright Builds checks, full `just test`,
redaction verification, parity validation, and parity-progress validation must
also pass. Promotion to `verified` requires executable `unit,golden` evidence
for every in-scope queue state transition, exact capacity and FIFO wrap-around,
unchanged state on full/empty boundary errors, deterministic drop-on-clear,
and explicit preservation of `SYS-005` plus every live/effectful non-claim.
No hardware, detector, credential, recovery, retry, destructive, or evidence-
privacy gate is entered because this plan is software-only and effect-free.

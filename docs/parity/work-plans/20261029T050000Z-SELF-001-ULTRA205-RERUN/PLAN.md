# SELF-001 on the Ultra 205 at the final Gamma 601 commit

## Identity

- Parity row: `SELF-001`
- Active task: `task-gamma-601-bring-up` (F4.6; owner-approved plan
  `delightful-foraging-quill`, owner decisions 2026-10-26)
- Board: Ultra 205 (BM1366, DS4432U, INA260), registered as board `205`
- The verified attempt's plan:
  `docs/parity/work-plans/20260822T024037Z-SELF-001-RETRY-4/PLAN.md`
- The 601's plan: `docs/parity/work-plans/20261026T000000Z-SELF-001-GAMMA601/PLAN.md`
- Attempts: `scratch/self001-full-lifecycle/attempt-NNN/`, ordinals from
  `006`

## Objective

Rerun the self-test's full lifecycle on the 205, as attempt 005 ran it,
at the final F4 commit. Since attempt 005 (`a11b579b`), the Gamma 601
bring-up changed the shared self-test code: the board-aware contract, the
runtime and its domain measurement, the forwarded USB records, and the
campaign. The 205's verified numbers no longer describe the current code.

The lifecycle is unchanged: a controlled failure, its safe stop, the owner's
physical BOOT cancel, the cancelled receipt and restart, then a passing run,
its receipt and automatic restart, then the settings restored.

## The 205's contract

Unchanged from attempt 005. The 205's contract derives from the same
constants, and a unit test checks the derived values against them
(`the_205_contract_keeps_its_verified_constants`):
- the kind `ultra205_full_v1`;
- 485 MHz / 1200 mV, the 205's catalog default;
- the expected hashrate from 485 MHz and the BM1366's small cores, with four
  domains each within ±33 % of a quarter of it;
- the core voltage window, the power ceiling, the warm-up, measurement,
  temperature, input-voltage and fan limits of attempt 005.

## Authority and fail-closed rules

- Only the existing lease tuple starts the self-test (`selftestkind`
  `ultra205_full_v1`, `selftestlease`, `selftestcase`, `selftest=1`,
  `mineonboot=0`), written by the flash tool's NVS seed for this attempt's
  root.
- Every exit safe-stops and restarts or waits at `cancel_ready`.
- Upstream's bare `selftest=1` (the factory seed) and BOOT held at power-on
  remain non-claims on both boards.

## Lifecycle and effects

1. Preflight: a clean pushed production package at the exact HEAD, the
   reference commit, a fresh detector, the private root and the projection
   absent.
2. Settings backup over the current session's origin; the pool and Wi-Fi
   inputs must restore it exactly.
3. The planned-failure phase: a factory install of the package with the
   owner's Wi-Fi credentials and the tuple, a 5 s load, the planned
   evaluation failure, the safe stop, `cancel_ready` with `safe_state:true`.
4. The owner holds BOOT for two seconds; the cancelled receipt and restart.
5. The pass phase: a factory install with the pass tuple, warm-up,
   measurement, evaluation, the safe stop, the passed receipt and the
   automatic restart.
6. The settings restored over HTTP; the private projection validated at
   `docs/parity/evidence/self001-full-lifecycle/attempt-NNN/self-test-projection.json`.
7. Production reinstalled after every attempt, before any analysis.

Allowed effects are the installs and their resets, the Wi-Fi credentials
applied at the owner's request, 485 MHz / 1200 mV, the fan 10 to 100 %,
diagnostic work only (no pool), and the settings restoration. Prohibited are
any other profile, pool work, a Worker Start or lease, a test build, direct
UART or pins, writing a 601 package, and any retry without a recorded,
verified change.

## Stops

Any unplanned failure; a limit of attempt 005's contract crossed; a reset or
panic in a phase window; a missing `cancel_ready` or receipt. Any stop
pauses the device, reinstalls production, and only then analyses.

## Non-claims

No claim for upstream's factory-seed or BOOT-held starts, the display
messages, actual injected faults, or other boards. Attempt 005's committed
projection and summary stay as they are. Parity changes only through a
reviewed verified-revision authority.

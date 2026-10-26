# SELF-001 on the Gamma 601

## Identity

- Parity row: `SELF-001`
- Active task: `task-gamma-601-bring-up`
- Board: Gamma 601 (BM1370, TPS546), registered as board `601`
- The 205's plan: `docs/parity/work-plans/20260822T024037Z-SELF-001-RETRY-4/PLAN.md`
- Decision: ADR-0041 item 14; owner decision 2026-10-26 (port the self-test
  before promotion)
- Attempts: `scratch/self001-gamma601/attempt-NNN/`, ordinals from `001`

## Objective

Run the hardware self-test's full lifecycle on the 601, as attempt-005 ran it
on the 205: a controlled failure, its safe stop, the owner's physical BOOT
cancel, the cancelled receipt and restart, then a passing run, its receipt
and automatic restart, then the settings restored. The first attempts run the
test-only `gamma601-mining` build (the only 601 build that may power the
BM1370 before promotion). The promoted production build repeats it before
the 601 claims SELF-001.

## The 601's contract

The pure contract derives every board-specific value from the 601's catalog
entry, as upstream's `self_test.c` derives it from `DEVICE_CONFIG`:

- the profile: the catalog default, 525 MHz / 1150 mV;
- the expected hashrate: 525 MHz × 2040 small cores × 0.85 ≈ 910 GH/s, with
  four domains each within ±33 % of a quarter of it;
- the core voltage window: 1150 mV ± 10 %, 1035 to 1265 mV;
- the power ceiling: the 19 W target + 3 W = 22 W;
- the regulator: a TPS546 power fault fails the run (`regulator_fault`, as
  upstream's `test_vreg_faults`), and so does a regulator temperature above
  the TPS546 throttle, 105 C, or none read (`vr_temperature_exceeded`);
- the shared stages and limits: PSRAM, the safety preflight, warm-up to 55 C
  within 180 s at the 10 % minimum fan, 30 s measured with the fan's PID at
  65 C, a 70 C ceiling, the input voltage 4.5 to 5.5 V and the fan above
  1000 rpm.

The input voltage check is a deliberate deviation: upstream skips it without
an INA260. The 601 reads its input from the TPS546 VIN, and the check stays.

## Authority and fail-closed rules

- The TPS546 turns on only under a generation's permit (ADR-0041 item 10).
  The self-test runs as a local generation with a 240 s budget armed at its
  start. The gate revokes it at the bound or on any unsafe observation, and
  the regulator's owner then turns the output off within 100 ms, whatever
  the self-test thread is doing.
- Every exit safe-stops (the reset held, the regulator commanded off and
  confirmed), retires the generation, and restarts or waits at
  `cancel_ready`. The next boot's safe boot commands the output off first.
- The fan is commanded as the local generation's controller; the shared
  idle-only path refuses below 100 % while a generation holds the gate.
- A build that cannot power the BM1370 (a production 601 before promotion)
  refuses the tuple before any effect and clears its flag with a cancelled
  receipt.
- Only the existing lease tuple starts the self-test (`selftestkind`
  `gamma601_full_v1`, `selftestlease`, `selftestcase`, `selftest=1`,
  `mineonboot=0`), written by the flash tool's NVS seed. Upstream's bare
  `selftest=1` (the factory seed) and BOOT held at power-on remain
  non-claims on both boards.

## Lifecycle and effects

1. Preflight: a clean pushed package at the exact HEAD, the reference
   commit, a fresh detector, the private root absent.
2. Settings backup over the current session's origin; the pool and Wi-Fi
   inputs must restore it exactly.
3. The planned-failure phase: a factory install of the package with the
   owner's Wi-Fi credentials and the tuple, a 5 s load, the planned
   evaluation failure, the safe stop, `cancel_ready` with `safe_state:true`.
4. The owner holds BOOT for two seconds; the cancelled receipt and restart.
5. The pass phase: a factory install with the pass tuple, warm-up,
   measurement, evaluation, the safe stop, the passed receipt and the
   automatic restart.
6. The settings restored over HTTP; the private projection validated.
7. Production reinstalled after every attempt, before any analysis.

Allowed effects are the installs and their resets, the Wi-Fi credentials
applied by the owner's request, the regulator on at 1150 mV for at most
240 s per phase, 525 MHz, fan 10 to 100 %, diagnostic work only (no pool),
and the settings restoration. Prohibited are any other profile, pool work,
a Worker Start or lease, direct UART or pins, and any retry without a
recorded, verified change.

## Stops

Any unplanned failure; the output not confirmed off within 4 s; the chip
above 70 C, the regulator above 105 C or the power above 22 W; the output on
past its 240 s bound; a reset or panic in a phase window; a missing
`cancel_ready` or receipt. Any stop pauses the device, reinstalls
production, and only then analyses.

## Non-claims

No claim for other boards, upstream's factory-seed or BOOT-held starts, the
display messages, or a production 601 until its own repeat passes. Parity is
promoted only through a reviewed verified-revision authority.

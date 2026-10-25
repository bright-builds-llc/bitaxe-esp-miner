# Gamma 601: the first local mining run

Phase F3.3 of the Gamma 601 bring-up (`task-gamma-601-bring-up`, ADR-0041
items 10 and 11) ran the test-only `gamma601-mining` firmware through the
local-mining soak `gamma601-first-001`: 10 minutes at 400 MHz / 1150 mV with
the owner's pool, then a paused, stopped device, then production
reinstalled. Attempt 002 passed.

- **Commands** (the contract in `TASKS.md`):
  1. `just detect-ultra205`;
  2. `just package-gamma601-mining`;
  3. `just flash-monitor --board 601 --port <port> --manifest <the 601
     package manifest> --allow-test-firmware gamma601-mining
     --capture-timeout-seconds 360`;
  4. `just otawww-endpoint preflight|serve|finish`, with the owner's
     Connect in a new dedicated Gate tab (the owner picked the 601 in
     Chrome's port chooser);
  5. `just local-mining-soak preflight`, `pause` and `run --soak
     gamma601-first-001`, the pool credentials passed by path only;
  6. `just package --board 601`, `just flash --board 601 --port <port>`
     and `just monitor --board 601 --port <port>
     --capture-timeout-seconds 360`.
- **Sources:** the rule at `6be7d715`, the boot rule at `667f1160`, the
  fix at `bc9c1902`; attempt 001 at `771f24e7`, attempt 002 at `cb4b8956`;
  the Gate at `b775145`; the reference at `c1915b0`.
- **Manifests,** retained privately with the captures: each install's
  `firmware_variant=gamma601-mining`, board 601, at its attempt's source;
  step 6's `production`.
- **Pool:** `pool_config: local-owner-supplied`. No pool value is recorded
  here.

## Attempt 001: stopped on the power fact

- **Before the run:** the boot was paused with the Worker started, the
  reset held and the output confirmed off. The handoff (one failed Connect,
  then the owner's) passed, and so did the soak's preflight and pause
  (`mine_on_boot_off`, `paused`, `asic_stopped`) and its pool and 400 MHz
  profile.
- **The stop:** about 1 s after the resume the run's generation was revoked
  on the power fact (`state=fault`, about 5.0 W, the regulator's offset
  alone). The run ended `Failed` about 62 s later, after the stop's cooling
  proof. After `local_restart_backoff` the device tried once more, from
  about 94 to 156 s after the resume, and stopped the same way (the core
  rail off, the cooling proof, the chip off). The soak never judged the
  failure itself: it was interrupted (`soak_interrupted`) at about 185 s,
  and the soak's `pause` then confirmed `mine_on_boot_off`, `paused` and
  `asic_stopped`.
- **The cause:** with the output no longer confirmed off, any negative IOUT
  was an invalid power sample, and this 601's regulator offset (about
  -0.2 A) reads negative in the first sweeps after the enable, before the
  BM1370 draws a load. The fix (`bc9c1902`) applies the -0.5 A offset floor
  whenever the chip is unloaded (not yet initialized, or the regulator
  commanded off); an initialized chip's negative reading with the regulator
  on stays invalid. It also logs each change of the power fact's fault
  cause.
- **Outcome:** `continue_after_verified_fix`.
- **A deviation from the contract:** its stop rule says to reinstall
  production before analysing. The device was paused with the chip off,
  but stayed on the `gamma601-mining` build through the analysis until
  attempt 002's install; production was reinstalled only after attempt
  002.

## Attempt 002: passed

The soak's verdict, `result=passed`, no failures:

| Metric | Value | Limit |
| --- | --- | --- |
| Mining | 601 s continuous (289 active samples), no outage | 600 s |
| Shares accepted / rejected | 32 / 0 | at least one accepted |
| Hashrate | up to 1077 GH/s; the middle 90 % of active samples 699 to 919 GH/s, mean 813 (400 MHz × 2040 small cores ≈ 816 GH/s) | nonzero |
| Chip temperature peak | 62.6 C | under 75 C |
| Regulator temperature peak | 61.8 C | under 90 C |
| Power peak | 16.4 W | under 22 W |
| Input voltage | 5195 to 5250 mV | 4500 to 5500 mV |
| Fan | at least 3481 rpm | nonzero |
| Largest sample gap | 2.6 s | |

- **The preparation:** from the resume, the power rose from 5.7 W through
  the regulator enable, chip detection, the BM1370 init and the ramp to
  400 MHz. The chip temperature first read at 45.1 C once the init
  completed, about 10.4 s after the resume, and mining was active at
  about 12.4 s.
- **The chip-temperature rule on hardware** (ADR-0041 item 11): before the
  init the chip temperature was unavailable and did not count; after it
  the diode read 45 C rising to 62 C, and the stop rules used it.
- **The stop:** the soak paused the device; the run ended `Requested`, not
  `Failed`, about 31.6 s later (the 601's 30 s cooling proof with the
  regulator cool and the chip unpowered), and the device read paused at
  5.0 W with start-on-boot off.
- **The power-fault diagnostic:** the device's retained log, read after the
  run, holds no `power_fault` line, but its ring had rolled past the enable
  and the preparation, and no serial capture covered the run. So it does
  not show the diagnostic stayed silent; the soak's samples show no power
  fault.

## Production afterwards

The receive-only capture after step 6 holds 56 safe-boot records:
`built=601 stored=601 identity=match`, `reset_held=true`, `off=confirmed`,
20 mV, fan 30 %, one boot. The USB profile reads `worker_started` with a
confirmed baseline. Its 57 `usb_tx_failure stage=flush_timeout` records are
the reader gap between `just flash` and `just monitor`, as in F3.1 and
F3.2.

## Effects

- **Attempt 001:** one mining install, the handoff, start-on-boot saved
  off, the pool and the 400 MHz / 1150 mV profile saved, two revoked run
  starts (each with the output enabled briefly, then off through the
  cooling proof), the soak interrupted, two pauses.
- **Attempt 002:** one mining install, the handoff, the pause, 10 minutes
  of local mining with the owner's pool, the soak's pause and stop, then
  one production install and its receive-only capture.
- The saved pool, profile and start-on-boot off stay in NVS; production
  refuses mining regardless.

There was no Worker Start or lease, factory reset, other frequency or
voltage, network discovery beyond the handoff, direct UART or pin access.

## Not shown here

- **Upstream's 525 MHz default,** and sustained mining beyond 10 minutes.
- **An overheat, a fault or a lease stop on the 601,** and overheat
  recovery's cooling sample: only a requested stop ran.
- **The 205 at these commits:** the regression owed since `2bf484b8` now
  also covers the chip-temperature rule and this fix.
- **Parity:** nothing is promoted; production 601 dispatch stays refused
  until phase F4.

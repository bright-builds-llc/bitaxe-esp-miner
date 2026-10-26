# Gamma 601: the default profile check

Phase F4.1 of the Gamma 601 bring-up (`task-gamma-601-bring-up`, ADR-0041)
ran the test-only `gamma601-mining` firmware through the local-mining soak
`gamma601-default-001`: 10 minutes at upstream's default 525 MHz / 1150 mV
with the owner's pool, then a paused, stopped device, then production
reinstalled. Attempt 001 passed. It is the first run at the default; F3 ran
at 400 MHz, and briefly at 500 MHz for the overheat check.

- **Commands** (the contract in `TASKS.md`):
  1. `just detect-ultra205`; `just package-gamma601-mining`; `just
     flash-monitor --board 601 --port <port> --manifest <the 601 package
     manifest> --allow-test-firmware gamma601-mining
     --capture-timeout-seconds 360`;
  2. `just otawww-endpoint preflight|serve|finish`, Connect in the
     dedicated Gate tab;
  3. `just local-mining-soak preflight --soak gamma601-default-001`,
     `pause` and `run`, the pool credentials passed by path only;
  4. `just package --board 601`, `just flash --board 601 --port <port>`
     and `just monitor --board 601 --port <port>
     --capture-timeout-seconds 360`.
- **Sources:** the soak at `68c8bf8f`, which is also the attempt's
  commit; the reference at `c1915b0`.
- **Manifests,** retained privately with the captures: step 1's
  `firmware_variant=gamma601-mining`, board 601, source `68c8bf8f`; step
  4's `firmware_variant=production`.
- **Pool:** `pool_config: local-owner-supplied`.

## The soak's verdict

`result=passed`, no failures:

| Metric | Value | Limit |
| --- | --- | --- |
| Mining | 601 s continuous (288 active samples), no outage | 600 s |
| Shares accepted / rejected | 29 / 0 | at least one accepted |
| Hashrate | mean 1054 GH/s; 5th to 95th percentile 926 to 1180; peak 1225 (525 MHz × 2040 small cores ≈ 1071 GH/s) | nonzero |
| Power peak | 20.4 W | under 22 W |
| Chip temperature peak | 69.5 C | under 75 C |
| Regulator temperature peak | 76.0 C | under 90 C |
| Input voltage | 5109 to 5195 mV | 4500 to 5500 mV |
| Fan | at least 3446 rpm | nonzero |
| Wi-Fi | at least -50 dBm, never disconnected | |
| Largest sample gap while mining | 2.2 s | |

- **The preparation:** the chip temperature first read 47.6 C about 12.5 s
  after the resume, and mining was active at about 14.6 s.
- **Steady state:** over the last 5 minutes the power read 20.2 to 20.4 W,
  the chip 68.6 to 69.5 C and the regulator about 74 to 76 C. The automatic
  fan reached its top speed, about 8,000 rpm, 46 s after the resume and
  stayed there.
- **Margins:** 1.6 W under the 601's 22 W stop and 5.5 C under the 75 C
  chip trip, with the fan already at its top. A warmer room could reach the
  chip trip in a long run; the soak would then stop and pause, and the
  firmware's own 75 C trip acts at the same temperature.
- **The stop:** the soak paused the device, and it read paused and stopped
  31 s later at 5.0 W, with start-on-boot off.

## Production afterwards

The receive-only capture after step 4 holds 56 safe-boot records:
`built=601 stored=601 identity=match`, `reset_held=true`, `off=confirmed`,
one boot. The USB profile reads `worker_started` with a confirmed
baseline. Its 113 `flush_timeout` records are replays of one early-boot
flush failure, most likely from the reader gap between `just flash` and
`just monitor`. The boot read `boot_ordinal=1`, as the production boots of the
lease check did; recorded, not investigated.

## Effects

One mining install, the handoff, start-on-boot saved off, the pool and the
525/1150 profile saved, 10 minutes of local mining, the soak's pause and
stop, then one production install and its receive-only capture. There was
no Worker Start or lease, factory reset, other frequency or voltage,
network discovery beyond the handoff, direct UART or pin access.

## Not shown here

- **Hours at the default** (phase F4.2's 4 h soak), and how the chip
  temperature behaves with the room warmer.
- **The production build mining:** production still refuses; promotion is
  phase F4.4.
- **Parity:** nothing is promoted.

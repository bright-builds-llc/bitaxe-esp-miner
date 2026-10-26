# Gamma 601: the overheat stop and recovery

Phase F3.4 of the Gamma 601 bring-up (`task-gamma-601-bring-up`, ADR-0041
items 11 and 12, ADR-0040) ran the test-only `gamma601-overheat-trip-test`
firmware (`gamma601-mining` with the 55 C local overheat trip) through the
local-mining soak `gamma601-overheat-001`: mining at 500 MHz / 1150 mV
until the trip, the 601's cooling rule, a restart at the reduced 400 MHz /
1050 mV, and the settings restored. Attempt 001 passed with outcome
`restarted`.

- **Commands** (the contract in `TASKS.md`):
  1. `just detect-ultra205`;
  2. `just package-gamma601-overheat-trip-test`;
  3. `just flash-monitor --board 601 --port <port> --manifest <the 601
     package manifest> --allow-test-firmware gamma601-overheat-trip-test
     --capture-timeout-seconds 360`;
  4. `just otawww-endpoint preflight|serve|finish`, with the owner's
     Connect;
  5. `just local-mining-soak preflight --soak gamma601-overheat-001`,
     `pause` and `run`, the pool credentials passed by path only;
  6. `just package --board 601`, `just flash --board 601 --port <port>`
     and `just monitor --board 601 --port <port>
     --capture-timeout-seconds 360`.
- **Sources:** the variant at `4418b902`, the harness at `e35b2584`, the
  attempt at `4993587d` (the contract); the Gate at `b775145`; the
  reference at `c1915b0`.
- **Manifests,** retained privately with the captures: step 3's
  `firmware_variant=gamma601-overheat-trip-test`, board 601, source
  `4993587d`; step 6's `production`.
- **A deviation:** the contract's step 3 says to confirm the boot marker.
  Its line is retained-only, so the passive capture does not hold it, and it
  had rolled out of the device's log ring by the time the log was read. The
  variant rests on the package and flash logs: the flash tool admitted
  step 3's image only against its manifest's variant marker.
- **Pool:** `pool_config: local-owner-supplied`.

## The soak's verdict

`result=passed`, no failures, outcome `restarted`:

| Metric | Value |
| --- | --- |
| Trip cause | `lowered_trip` (the 55 C trip; the last hashing reading 54.1 C) |
| Overheat mode set to restart | 41.1 s |
| Cooling measured before the restart | 32.7 s (the 601's rule: the chip unpowered, the regulator fresh at or below 95 C; at least 30 s) |
| Restart pair | 400 MHz / 1050 mV (the firmware's reduced profile from 500/1150) |
| After the restart | 302 s hashing, no second trip, the chip at most 52.4 C |
| Shares accepted | 23 |
| Chip / regulator / power peaks | 54.1 C / 55.3 C / 16.9 W |
| Input voltage | 5188 to 5289 mV |
| Fan | at least 3583 rpm |
| Hashrate | up to 1294 GH/s |
| Recovery | `recovery_settled`, `settings_restored` |

The journal's timeline, in seconds after the resume:
- **0.5 to 15:** the preparation; the power rose to 16.2 W and the chip
  temperature first read 49.3 C at 13.1 s, once the BM1370's init
  completed; mining active at 15.2 s at 500 MHz, hashing 994 to 1231 GH/s
  at 16.6 to 16.9 W.
- **About 23:** the chip passed 55 C (54.1 C in the last hashing sample)
  and the run stopped at the trip: power 5.0 W, the chip unpowered, the fan
  to full; the regulator cooled from 47 C.
- **54.8:** overheat mode set, after the stop's 30 s cooling proof (the
  soak's `overheat_tripped` mark, which its trip timing metric counts
  from).
- **95.9:** restarted at 400/1050; it hashed for 302 s at a mean of
  812 GH/s (the middle 90 % of samples 709 to 901 GH/s), at most 13.5 W.
- **398 to 582:** the soak paused the device, waited for the recovery to
  settle, restored the journaled baseline and watched it hold.

**Afterwards:** the soak restored the journaled baseline and required it to
hold for 60 s with the device stopped (`settings_restored`,
`recovery_settled`). A read-only read over HTTP, before the production
install and again after it (kept privately with the captures), shows
400 MHz / 1150 mV, automatic fan (the baseline's manual 100 % value
unchanged), `overheat_mode 0`, paused, start-on-boot off, 5.0 W.

**Production afterwards:** the receive-only capture after step 6 holds 56
safe-boot records: `built=601 stored=601 identity=match`,
`reset_held=true`, `off=confirmed`, 20 mV, fan 30 %, one boot; the USB
profile reads `worker_started` with a confirmed baseline; its 57
`usb_tx_failure stage=flush_timeout` records are the reader gap between
`just flash` and `just monitor`.

## Effects

One trip-test install, the handoff, start-on-boot saved off, the pool and
the 500/1150 profile saved, mining until the trip, the device's own
overheat writes (overheat mode, the reduced pair, the fan at 100 %), the
restart at 400/1050 for 5 minutes, the soak's pause and restore, then one
production install and its receive-only capture. There was no Worker Start
or lease, factory reset, other pair, manual resume, network discovery
beyond the handoff, direct UART or pin access.

## Not shown here

- **A second trip** at 400/1050 and the stays-stopped path on hardware: the
  restart ran at most 52.4 C.
- **The production 75 C trip** on the 601: this check used the lowered
  trip.
- **Upstream's 525 MHz default,** and an overheat under the owner's normal
  fan settings: the fan ran under the firmware's control at the check's
  settings.
- **Parity:** nothing is promoted; production 601 dispatch stays refused
  until phase F4.

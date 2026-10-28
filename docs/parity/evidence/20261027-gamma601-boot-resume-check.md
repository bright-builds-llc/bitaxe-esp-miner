# Gamma 601: the boot-resume check on production

Phase F4.5 of the Gamma 601 bring-up (`task-gamma-601-bring-up`, ADR-0041)
ran the promoted production 601 build through the boot-resume check
`gamma601-boot-resume-001`.
- With mining on boot saved on, a software restart must bring local mining
  back with no resume command.
- With it saved off, a restart must leave the device paused.

Attempt 002 passed. It is the first evidence of the production 601 mining,
and of it honoring its saved start-on-boot preference (ADR-0041 item 15).

- **Commands** (the F4.5 contract in `TASKS.md`):
  1. `just detect-ultra205`; `just package --board 601`; `just
     flash-monitor --board 601 --port <port> --capture-timeout-seconds
     360`, state-preserving;
  2. `just otawww-endpoint preflight|serve|finish`, Connect in the
     dedicated Gate tab;
  3. `just local-mining-soak preflight --soak gamma601-boot-resume-001`,
     `pause` and `run`, with the pool credentials passed by path only;
  4. `just monitor --board 601 --port <port> --capture-timeout-seconds
     360` to confirm production.
- **Sources:** attempt 002 at `33fea4d9`, which is also the installed
  firmware's commit; the reference at `c1915b0`.
- **Manifest,** retained privately with the captures:
  `firmware_variant=production`, board 601, source `33fea4d9`, a clean
  tree.
- **Pool:** `pool_config: local-owner-supplied`.

## Attempt 001: the handoff refused at the Gate's baseline

Attempt 001 ran at `c3ec7ad4`.
- **Install:** production was installed and confirmed (56 safe-boot
  records, `identity=match`, the Worker baseline confirmed, no panic).
- **Handoff:** Connect failed twice at the Gate's `baseline` step with
  `command_rejected`. Hello, discovery and possession had passed, and the
  tab stayed visible and focused.
- **Release:** a `finish` run during the attempts refused with
  `endpoint_owner_live`, because the server was still serving. The page was
  then sent to `about:blank` in the session, and the server was stopped; its
  log reads `server_released`. Three of its five attempts went unused, and
  the endpoint root holds no result.

The cause was found in the code path, since the Gate records no rejection
message at `baseline`:
- F4.3's settings restore saved `display`, which the schema stores as text.
- The Worker's public-settings fingerprint accepted text only for
  `asicfrequency_f` and the two theme keys.
- So every command that carries status evidence failed as
  `session_failed`. By the code, this holds on both boards once `display`
  is saved, which upstream's web UI also does. It was observed on the 601
  only.

The fix, `33fea4d9`, adds `display` to the fingerprint's text keys, with a
regression that ties those keys to the schema's text types. A device
without `display` stored keeps its digest. Outcome:
`continue_after_verified_fix`. Attempt 002's handoff was admitted on its
first Connect.

## The check's verdict (attempt 002)

`result=passed`, no failures.

| Step | Observed | Bound |
| --- | --- | --- |
| Baseline | paused, start-on-boot off, 5.0 W, 525 MHz / 1150 mV | the paused, stopped baseline |
| Restart with start-on-boot on | new boot after 12.8 s, reset `software_cpu`, next boot ordinal | within 180 s |
| Mining on that boot | first mining sample 10.4 s after the boot's first sample (uptime 20 s), with no resume command | within 5 minutes |
| Mining | 302 s accumulated (144 active samples), no outage | 300 s within 15 minutes |
| Shares accepted / rejected | 28 / 0 | at least one accepted |
| Stop | paused; nothing read stopped for about 30 s, so the pause was re-sent at 31.6 s; first stopped sample at 31.7 s, confirmed at 33.8 s, at 5.0 W with start-on-boot off | within 180 s |
| Restart with start-on-boot off | new boot after 10.5 s, reset `software_cpu`, next boot ordinal | within 180 s |
| Hold | paused, not mining, start-on-boot off, 4.995 to 4.996 W, for 122 s (58 samples) | 120 s |

While mining:

| Metric | Value | Limit |
| --- | --- | --- |
| Hashrate | mean 1060 GH/s over the 144 active samples (the first read 0 GH/s); 5th to 95th percentile 917 to 1187; peak 1359 | nonzero in some sample |
| Power peak | 20.5 W | under 22 W |
| Chip temperature peak | 70.6 C | under 75 C |
| Regulator temperature peak | 74.8 C | under 90 C |
| Input voltage | 5063 to 5141 mV | 4500 to 5500 mV |
| Fan | at least 3583 rpm | nonzero |
| Wi-Fi | at least -50 dBm, never disconnected | |

- **Steady state:** over the last 75 mining samples (about 155 s) the
  power read 20.1 to 20.5 W, the chip 69.8 to 70.6 C and the regulator
  68.8 to 74.8 C. That is close to F4.1's last 5 minutes on the retired
  test build: 20.2 to 20.4 W and 68.6 to 69.5 C.
- **The profile:** every sample read 525 MHz / 1150 mV.

## Production afterwards

The device ended paused, with start-on-boot off, on the second restart's
boot of the installed production package. A receive-only `just monitor`
capture afterwards confirms it:
- 56 safe-boot records with `built=601 stored=601 identity=match`;
- the USB profile `worker_started` with a confirmed baseline at
  `33fea4d9`, and no panic;
- `boot_ordinal=8` with reset `software_cpu`, the held restart's boot. The
  install's boot read 6, the mining boot 7. The RTC ordinal resets to 1 only
  on a cold start or an invalid record, so the `boot_ordinal=1` that
  attempt 001's install and earlier production captures read is still
  unexplained.
- The core-dump store reads `ready` for this boot and the one before it,
  with no dump.
- Its 57 `usb_tx_failure` records at stage `flush_timeout` (113 lines
  naming it, as earlier docs counted) replay one early-boot flush failure,
  as in earlier captures.

## Effects

Each attempt:
- one production install and its capture;
- the handoff.

Attempt 002 also:
- saved the pool and 525/1150, after the run's `pause` had confirmed
  start-on-boot off;
- saved start-on-boot on;
- made one restart, then 5 minutes of local mining;
- saved start-on-boot off, paused, and made one restart and a 2-minute
  hold.

There was no Worker Start or lease, factory reset, test build, other
frequency or voltage, network discovery beyond the handoffs, direct UART or
pin access. Between the attempts the device could not be paused over HTTP,
because the endpoint was blocked by the bug. It stayed on production under
the firmware's own protections until attempt 002's install restarted it;
attempt 002's `pause` then confirmed start-on-boot off and the device
paused and stopped.

## Not shown here

- **A power-on boot:** both restarts are software restarts.
- **Hours on production:** F4.2's 4 h soak ran the retired test build,
  whose mining path is the one production now compiles.
- **Parity:** nothing is promoted.

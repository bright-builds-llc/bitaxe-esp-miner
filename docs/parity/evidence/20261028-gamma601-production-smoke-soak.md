# Gamma 601: the production smoke soak

Phase F4.5 of the Gamma 601 bring-up (`task-gamma-601-bring-up`, ADR-0041
item 15) ran the promoted production 601 build through the local-mining soak
`gamma601-production-001`: 10 continuous minutes at upstream's default
525 MHz / 1150 mV with the owner's pool, then a paused, stopped device.
Attempt 005 passed. It is the first 10-minute run on the production build;
F4.1 ran this profile on the retired test build.

- **Commands** (the F4.5 contract in `TASKS.md`):
  1. `just detect-ultra205`; `just package --board 601`; `just
     flash-monitor --board 601 --port <port> --capture-timeout-seconds
     360`, state-preserving;
  2. `just otawww-endpoint preflight|serve|finish`, Connect in the
     dedicated Gate tab;
  3. `just local-mining-soak preflight --soak gamma601-production-001`;
     in attempt 005, `pause --restore-journal <attempt 003's soak root>`;
     then `run`, with the pool credentials passed by path only;
  4. `just monitor --board 601 --port <port> --capture-timeout-seconds
     360` to confirm production.
- **Sources:** attempt 005 at `69dafb3d`, which is also the installed
  firmware's commit; the reference at `c1915b0`.
- **Manifest,** retained privately with the captures:
  `firmware_variant=production`, board 601, source `69dafb3d`.
- **Pool:** `pool_config: local-owner-supplied`.

## Attempt 003: the firmware's 75 C trip

Attempt 003 ran at `cfac7234`.
- **The run:** it mined 171 s (82 active samples, 23 shares accepted,
  none rejected). The chip climbed to 74.875 C and the regulator to
  76.25 C, at 21.2 W, with the fan near its top from about 20 s of
  mining on (8000 rpm at 21 s).
- **The trip:** the firmware ended the run (`ending=Some(Overheat)`, hold
  `local_overheat`). Power read 5.0 W at the next 2 s sample. The soak
  then paused the device (`soak_overheat`, `soak_active_time_short`).
- **Cause:** the setup ran hotter than in the earlier passing runs at the
  same profile. The idle regulator read 33.8 C, against 31.8 C in F4.1
  and 31.3 C in the boot-resume check. Under load the chip ran 4.4 to
  7.3 C hotter at the same times of mining: 6.5 C at both 60 s and 170 s,
  against F4.1.
- **Response:** the firmware's 75 C trip is upstream's fixed
  `THROTTLE_TEMP`, so the threshold stayed. The owner cooled the room
  before the retry.

Outcome `stop_hardware_blocker`, then `continue_after_manual_remediation`.
A first `run` invocation in this attempt refused with `soak_option` before
any device access, because the operator passed preflight's options.

## Attempt 004: the latched overheat mode

Attempt 004 ran at `2b8769a7`.
- **The latch:** after a trip the firmware saves `overheat_mode`, a
  lowered pair (425/1050) and the fan manual at 100 %, as upstream does.
  It clears the latch only once its next run has prepared.
- **The stop:** the soak admitted that latched baseline, saved 525/1150
  over the recovery pair, and resumed. Its live stop then read the latch
  as a trip on the first sample, before any hashing. The latch cleared
  as the preparation finished; the fan stayed manual.
- **The fix,** `69dafb3d`, with regression tests:
  - every run's baseline refuses a latched `overheat_mode` before any
    write (`soak_baseline_overheat`);
  - the recovery `pause` restores a tripped run's latch-clear baseline
    once the firmware's recovery settles: the pair, the fan, and the latch
    cleared. It restores only on the soak's own board and never resumes an
    ordinary soak's device pause;
  - `--restore-journal` reads a sealed earlier attempt's journal without
    writing to it.

Outcome `continue_after_verified_fix`.

## The restore before attempt 005

`pause --restore-journal` on attempt 003's soak root ran on attempt 005's
own root, after its handoff and preflight. Its outcome reads
`recovery_settled`, `settings_restored` and `stop_cleared`: 525/1150, the
fan automatic, the latch clear. Attempt 003's root was not written.

## The soak's verdict (attempt 005)

`result=passed`, no failures:

| Metric | Value | Limit |
| --- | --- | --- |
| Baseline | paused, start-on-boot off, 5.0 W, 525/1150, fan automatic, `overheat_mode` clear, idle regulator 30.75 C | the paused, stopped baseline |
| Mining | 600.5 s continuous (285 active samples), no outage | 600 s |
| Shares accepted / rejected | 39 / 0 | at least one accepted |
| Hashrate | mean 1063 GH/s over the 285 active samples (the first read 0 GH/s); 5th to 95th percentile 948 to 1185 (nearest rank); peak 1459 (525 MHz × 2040 small cores ≈ 1071 GH/s) | nonzero |
| Power peak | 20.2 W | under 22 W |
| Chip temperature peak | 68.1 C | under 75 C |
| Regulator temperature peak | 74.5 C | under 90 C |
| Input voltage | 5086 to 5156 mV | 4500 to 5500 mV |
| Fan | at least 3481 rpm | nonzero |
| Wi-Fi | at least -41 dBm, never disconnected | |
| Largest sample gap while mining | 2.2 s | |

- **Preparation:** the chip first read 45.4 C about 12.6 s after the
  resume, and mining was active at about 14.7 s.
- **Steady state:** over the last 5 minutes the power read 19.9 to
  20.2 W, the chip 67.0 to 68.1 C and the regulator 72.3 to 74.5 C. The
  automatic fan ran high during the preparation, fell to 3481 rpm as
  mining started, passed 7,500 rpm 49 s after the resume, and stayed
  between 7637 and 8047 rpm.
- **Against the other runs at 525/1150:**

  | Run | Idle regulator | Chip at 60 s | Chip at 170 s |
  | --- | --- | --- | --- |
  | F4.1 (test build) | 31.8 C | 64.6 C | 68.4 C |
  | Attempt 003 | 33.8 C | 71.1 C | 74.9 C (trip) |
  | Attempt 005 | 30.8 C | 62.6 C | 66.4 C |

- **Margin:** 6.9 C under the 75 C trip, in a cooled room. Attempt 003
  shows the margin is gone a few degrees warmer.
- **The stop:** the soak paused the device at 615.2 s. Nothing read
  stopped within 30 s, so it sent the pause again at 31.7 s. The stop
  confirmed at 31.9 s, at 5.0 W with start-on-boot off. The boot-resume
  check's timing was within 0.2 s of this.
- **The profile:** every sample read 525 MHz / 1150 mV.

## Production afterwards

The soak's final sample read the device paused, with start-on-boot off and
the fan automatic. A receive-only `just monitor` capture afterwards
confirms production on the install's boot:
- 57 safe-boot records with `built=601 stored=601 identity=match`;
- the USB profile `worker_started` with a confirmed baseline at
  `69dafb3d`, and no panic;
- `boot_ordinal=20` with reset `other`, the install's boot, so no reset
  happened during the soak;
- the core-dump store valid for this boot. The previous boot reads
  `wrong_firmware`, from the previous install.

## Effects

- **Attempt 003:** one production install, the handoff, the pool and
  525/1150 saved, 171 s of local mining ended by the firmware's trip,
  and the soak's pause.
- **Attempt 004:** one production install, the handoff, the pool and
  525/1150 saved, a resume that prepared without hashing, and the pause.
- **Attempt 005:** one production install, the handoff, and the restore of
  attempt 003's baseline settings with the latch cleared. Then the pool
  and 525/1150 saved, 10 minutes of local mining, and the soak's pause and
  stop.

Attempts 003 and 005 ended with a receive-only capture; attempt 004's
stop did not, and attempt 005's install and capture followed it. There
was no Worker Start
or lease, factory reset, test build, other frequency or voltage, network
discovery beyond the handoffs, direct UART or pin access.

## Not shown here

- **Hours on production:** F4.2's 4 h soak ran the retired test build,
  whose mining path production now compiles.
- **A warm room:** at 525/1150 the margin to the 75 C trip is a few
  degrees. Attempt 003 shows the trip working on production; the overheat
  recovery's restart at the lowered pair was not observed, because the
  soak paused first.
- **Parity:** nothing is promoted here; F4.6 proposes it.

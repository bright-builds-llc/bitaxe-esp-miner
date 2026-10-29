# Ultra 205: the F4.6 regression check

Phases F3 and F4 of the Gamma 601 bring-up (`task-gamma-601-bring-up`,
ADR-0041) changed shared code the Ultra 205 runs:
- the promotion's build gates for the 601's dispatch and power path, which
  the 205 build leaves off;
- mining on boot following the saved preference;
- the board-aware self-test contract, compiled with the 205's constants;
- the overheat cooling and recovery core, the shared fan authority and the
  board-aware safety limits;
- the soak harness's refusal of a latched overheat baseline and its
  recovery restore.

`--soak regression-001` ran `watchdog-001`'s checks on production firmware
at the final F4 shared code. Before any write it required the firmware's
board report. Attempt 001 passed.

- **Commands** (the F4.6 205 regression contract in `TASKS.md`):
  1. `just detect-ultra205`;
  2. `just flash-monitor --board 205 --port <port> --capture-timeout-seconds
     360`, state-preserving;
  3. the endpoint handoff (`just otawww-endpoint preflight|serve|finish`)
     with one Connect in the dedicated Gate tab;
  4. `just local-mining-soak preflight --soak regression-001 ...`;
  5. `just local-mining-soak pause ...`;
  6. `just local-mining-soak run ...`, detached.
- **Sources:** production at `c217f6b2`, the package manifest's commit and
  `firmware_variant=production`; the reference at `c1915b0`.
- **Pool:** `pool_config: local-owner-supplied`.

## Attempt 001: passed

The install's USB records read `built=205 stored=205 identity=match` in 56
records, with no panic. The endpoint handoff recorded the endpoint on its
first Connect, with no failed attempts, and released its page and server.
The soak's `pause` found the device paused, with mining on boot off and the
ASIC stopped.

| Observation | Attempt 001 |
| --- | --- |
| Baseline over HTTP | paused, mining on boot off, no pause reason, `overheat_mode` clear, 485 MHz / 1200 mV, the fan automatic, 0.9 W; the board check passed (see below) |
| Preparation | 4 samples `safe_blocked`, then `active` 10.9 s after the resume |
| Hashing | 180,799 ms in one span, no outages |
| Shares | 14 accepted, 0 rejected |
| Peak hashrate / power / chip temperature | 527 GH/s / 13.31 W / 64 C |
| Input voltage while mining | 5.26 to 5.31 V |
| Samples | 89, at most 2,344 ms apart |
| Task watchdog | fresh in every sample |
| Wi-Fi | connected in every sample, at least -45 dBm |
| The stop | 22 samples: `stopping` in 20 (17 of them at idle power), then `paused` in 2, from 43.6 s; never `active` |
| Hashrate while stopping | zeroed (no frozen samples) |
| Final sample | same boot, paused at 0.88 W, mining on boot off, no pause reason, `overheat_mode` clear |
| Verdict | `passed`, no failures |

Notes on these numbers:
- **The board check:** the soak refuses the baseline unless `storedBoard`
  reads `205` and `boardIdentity` `match`, and since `69dafb3d` unless
  `overheat_mode` is clear. The journal records that the baseline was
  admitted, not the board values.
- **The pause:** the stop had not confirmed after 30 s, so the pause was
  sent once more, at 30.4 s, as in the phase C and F2 runs.
- **The stop's length:** a terminal stop waits for its cooling proof, a
  fresh chip reading at or below 45 C (`COOLING_PROOF_TIMEOUT_MS`, 120 s).
  Power was at idle (5 W or less) from 6.8 s and below 1.5 W from 8.9 s,
  and the stopped sample read 45 C at 43.6 s. Phase F2's stop confirmed at 31.2 s.
- **VR temperature:** the 205 does not report one, so its maximum of 0
  means not reported.

## Effects

- one state-preserving production install;
- one endpoint handoff with one Gate Connect;
- the soak's own effects: the recovery-style `pause`, saving the owner's
  pool, 3 minutes of local mining at the default pair, and one pause, sent
  twice.

There was no factory install and no NVS write beyond the pool settings.
The 205 stays on production `c217f6b2`, paused, with mining on boot off.

## What this shows

On the Ultra 205, the shared-code changes of phases F3 and F4 leave
everyday behaviour intact. The install admits the 205, the firmware reports
its board as matching, and mining starts, hashes and stops in order with
the watchdog fresh throughout, against the same checks phases C and F2
passed.

## Not shown here

- **The 205's self-test** at the final commit: its own F4.6 rerun.
- **Stops other than a pause** (`task-stop-paths-hardware-evidence`).
- **The 205's overheat paths** at this commit: the 205's overheat checks
  ran in October on earlier code.

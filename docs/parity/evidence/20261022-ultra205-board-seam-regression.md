# Ultra 205: the board seam regression check

Phase C of the Gamma 601 bring-up (`task-gamma-601-bring-up`, ADR-0041)
changed paths the Ultra 205 runs every day:
- the flash tool's board-aware admission and ROM gate;
- the USB replay cycle;
- the runtime attestation's board;
- the board-identity guard on mining and the self-test;
- the HTTP OTA board check.

`--soak regression-001` ran the strongest existing check on production
firmware, `watchdog-001`'s, which includes `stopping-002`'s. Before any
write, it also required the firmware's board report. Attempt 002 passed.

- **Commands:**
  1. `just detect-ultra205`;
  2. `just flash-monitor --board 205 --port <port> --capture-timeout-seconds
     360`, state-preserving;
  3. the endpoint handoff (`just otawww-endpoint preflight|serve|finish`)
     with one Connect in the dedicated Gate tab;
  4. `just local-mining-soak preflight --soak regression-001 ...`;
  5. `just local-mining-soak run ...`, detached.
- **Sources:** production at `ff6b11ef`; the reference at `c1915b0`.

## Attempt 001: stopped before any write

- **Before the run:** production `335e9f6c`, installed state-preserving,
  reported `built=205 stored=205 identity=match`. A Chrome crash cut a
  first endpoint handoff short, and its server was released. A second
  handoff and the soak preflight passed.
- **The failure:** the run's first system-info read threw before the
  baseline was journaled. A failure to reach the device carried no closed
  code, so the soak reported only `soak_failed`. The device stayed paused.
- **The fix (`ff6b11ef`):** the soak's device client now names such a
  failure: `soak_device_timeout` or `soak_device_unreachable`.

## Attempt 002: passed

The fix moved HEAD, so production was rebuilt at `ff6b11ef` and
reinstalled state-preserving. Then came a fresh handoff, with no failed
attempts.

| Observation | Attempt 002 |
| --- | --- |
| USB board record after the install | `built=205 stored=205 identity=match` |
| Baseline over HTTP | paused, mining on boot off, no pause reason, stopped at 0.93 W; the board check passed (see below) |
| Hashing | 181,443 ms in one span, no outages |
| Shares | 11 accepted, 0 rejected |
| Peak hashrate / power / chip temperature | 555 GH/s / 13.17 W / 62 C |
| Samples | 94, at most 2,213 ms apart |
| Task watchdog | `feed_fresh` in every sample |
| The stop | 18 samples: `stopping` in 16 (12 of them at idle power), then `paused` in 2, from 33.6 s |
| `active` at idle power while stopping | none |
| Hashrate while stopping | zeroed (no frozen samples) |
| Final sample | same boot, paused at 0.78 W, mining on boot off, no pause reason |
| Verdict | `passed`, no failures; an independent recomputation from the private journal agrees |

Notes on these numbers:
- **The board check:** the soak refuses the baseline unless `storedBoard`
  reads `205` and `boardIdentity` `match`. The journal records that the
  baseline was admitted, not those values.
- **The pause:** the stop had not confirmed after 30 s, so the pause was
  sent once more, at 31.6 s, and the stop confirmed about 2 s later.
- **Preparation:** the first 7 samples (about 15 s) read `safe_blocked`.
  The first hashing sample came 16.7 s after the resume.
- **VR temperature:** the 205 does not report one, so its maximum of 0
  means not reported.

## Effects

- two state-preserving production installs;
- two complete endpoint handoffs, each with one Gate Connect, plus one
  cut short by a Chrome crash;
- the soak's own effects: saving the owner's pool, 3 minutes of local
  mining at the default pair, and one pause.

There was no factory install and no NVS write beyond the pool settings.
The pool configuration is local and owner-supplied.

## What this shows

On the Ultra 205, phase C's changes leave everyday behaviour intact:
- the board-aware install admits the 205;
- the firmware reports its board as matching;
- mining starts, hashes and stops in order, with the watchdog fresh
  throughout.

## Not shown here

- **The HTTP OTA board check:** no OTA was attempted.
- **The cause of attempt 001's unreachable read:** the instrumentation
  will name the category if it recurs.

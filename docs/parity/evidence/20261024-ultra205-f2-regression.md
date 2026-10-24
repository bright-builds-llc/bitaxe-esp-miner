# Ultra 205: the phase F2 regression check

Phase F2 of the Gamma 601 bring-up (`task-gamma-601-bring-up`, ADR-0041)
changed shared code the Ultra 205 runs every day:
- the chip family, chip count and platform identity come from board
  identity;
- production results are classified through `AsicFamily`;
- the mining-safety checks and the cooling core take the board's limits;
- fan commands go through an authority shared with the Gamma 601;
- the preparation's reply wait is shared, and its admission and the saved
  profile are board-aware;
- mining dispatch is admitted per board.

`--soak regression-001` ran `watchdog-001`'s checks on production
firmware, and before any write it required the firmware's board report.
Attempt 001 passed.

- **Commands:**
  1. `just detect-ultra205`;
  2. `just flash-monitor --board 205 --port <port> --capture-timeout-seconds
     360`, state-preserving;
  3. the endpoint handoff (`just otawww-endpoint preflight|serve|finish`)
     with one Connect in the dedicated Gate tab;
  4. `just local-mining-soak preflight --soak regression-001 ...`;
  5. `just local-mining-soak run ...`, detached.
- **Sources:** production at `09490a0f`; the reference at `c1915b0`.

## Attempt 001: passed

The install reported the running firmware commit `09490a0f`, the same as
the package manifest. The endpoint handoff recorded the endpoint on its
first Connect, with no failed attempts, and released its page and server.

| Observation | Attempt 001 |
| --- | --- |
| USB board record after the install | `built=205 stored=205 identity=match` |
| Baseline over HTTP | paused, mining on boot off, no pause reason, stopped at 0.95 W; the board check passed (see below) |
| Hashing | 181,406 ms in one span, no outages |
| Shares | 13 accepted, 0 rejected |
| Peak hashrate / power / chip temperature | 524 GH/s / 13.41 W / 63 C |
| Input voltage | 5.23 to 5.28 V |
| Samples | 93, at most 2,220 ms apart |
| Task watchdog | `feed_fresh` in every sample |
| The stop | 17 samples: `active` in 1 (at 13.09 W), then `stopping` in 14 (11 of them at idle power), then `paused` in 2, from 31.2 s |
| `active` at idle power while stopping | none |
| Hashrate while stopping | zeroed (no frozen samples) |
| Final sample | same boot, paused at 0.87 W, mining on boot off, no pause reason |
| Verdict | `passed`, no failures; an independent recomputation from the private journal agrees on every metric |

Notes on these numbers:
- **The board check:** the soak refuses the baseline unless `storedBoard`
  reads `205` and `boardIdentity` `match`. The journal records that the
  baseline was admitted, not those values.
- **The pause:** the stop had not confirmed after 30 s, so the pause was
  sent once more, at 31.2 s, as in the phase C run; the stop confirmed at
  once.
- **Preparation:** the first 6 samples read `safe_blocked` or `active`
  before hashing. The first hashing sample came 14.7 s after the resume.
- **VR temperature:** the 205 does not report one, so its maximum of 0
  means not reported.

## Effects

- one state-preserving production install;
- one endpoint handoff with one Gate Connect;
- the soak's own effects: saving the owner's pool, 3 minutes of local
  mining at the default pair, and one pause, sent twice.

There was no factory install and no NVS write beyond the pool settings.
The pool configuration is local and owner-supplied.

## What this shows

On the Ultra 205, phase F2's shared-code changes leave everyday behaviour
intact: the board-aware install admits the 205, the firmware reports its
board as matching, and mining starts, hashes and stops in order with the
watchdog fresh throughout, against the same checks phase C passed.

## Not shown here

- The Gamma 601's own paths: they need their own hardware evidence
  (phase F3).
- Stops other than a pause (`task-stop-paths-hardware-evidence`).

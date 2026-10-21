# Ultra 205: the status reads `stopping` while a stop runs

The stopping check (`task-mining-activity-stopping`) shows on hardware
that `miningActivity` no longer reads `active` while the ASIC idles
through an ordered safe-stop. Before `d95d7002`, a terminal safe-stop
held its publication until the cooling proof confirmed the stop, so the
API kept the last `active` snapshot. Soak 005 recorded 13-18 s of that
after a Wi-Fi drop, and every lease interrupt showed it too. The
firmware now publishes `stopping` before the stop begins.

Repo-owned commands drove the check: `just package`, `just flash-monitor`
(state-preserving), `just otawww-endpoint` and `just local-mining-soak
--soak stopping-001`. Attempt 001 passed. An independent review
recomputed the verdict from the private journal.

Parity is not promoted.

| Observation       | Attempt 001 (production `91a4e40c`)                                        |
| ----------------- | -------------------------------------------------------------------------- |
| Firmware          | Clean production package, installed state-preserving (`observed_exact_package`); the handoff succeeded on its first attempt |
| Mining            | 180,745 ms of hashing in one span, no outages, Wi-Fi connected throughout, 16 accepted shares and 0 rejected |
| Limits            | 5,168.75–5,222.5 mV, at most 12.33 W, at most 60 C chip, at least 2,420 RPM |
| Pause sampling    | 14 samples from the pause to the confirmed stop, at most 2,087 ms apart    |
| First sample      | 75 ms after the pause: `active`, not yet paused, 12.22 W (still hashing)   |
| `stopping`        | From 2,146 ms to 22,875 ms: 11 samples, paused, power falling from 10.09 W to 1.25–1.48 W (8 of them at 5 W or less), chip cooling from 60 C to 46 C |
| `paused`          | From 24,950 ms, at 0.79 W and 45 C: the cooling proof's threshold          |
| `active` at idle  | 0 samples                                                                  |
| Afterwards        | Paused, powered down, mining on boot off, no pause reason; the same physical device (final detector). The serial port had no holder when the handoff finished, before the run |

Times count from the journaled `paused` event. `result.json` counts from
the first stop sample, 75 ms later (`stop_first_stopping_ms` 2,071,
`stop_first_paused_ms` 24,875).

A first install attempt failed on the host before any device write: the
copied manifest's relative ELF path did not resolve. Its endpoint server
was stopped before any page attempt.

## What this shows

- During a pause's ordered safe-stop, the status read `stopping` by the
  second sample, 2.1 s after the pause. It stayed `stopping` while the
  ASIC idled and the chip cooled, then read `paused` once the stop
  confirmed.
- No sample read `active` while fresh input power showed the ASIC idle.
  So in every sample of the stop, `miningActivity` matched the hardware,
  where it used to report active mining for the whole cooling proof.
- The status moved only forward: `active`, then `stopping`, then
  `paused`.

## Not shown here

- **Stops other than a pause:** a lease interrupt, a Wi-Fi drop or pool
  outage, an overheat trip and a fault use the same publication path.
  Only the pause was exercised on hardware.
- **A failed safe-stop,** which publishes `safe_blocked` with reason
  `hardware_safe_stop_failed`: fake-device coverage only.
- **The hashrate monitor,** which still freezes during the cooling proof:
  `hashRate` read 429 GH/s through all 11 `stopping` samples.
  Making the cooling wait non-blocking is a separate change.

# Ultra 205: a stop no longer freezes the owner while the board cools

The polled cooling stopping check (`task-safe-stop-polled-cooling`) shows
on hardware that a terminal safe-stop no longer holds the production owner
thread through its cooling proof. Before `ba22b7fb`, the owner blocked for
up to 120 s while the chip cooled to 45 C, and the API kept the last
hashrate. The stopping check (`20261021-ultra205-mining-activity-stopping-check.md`)
recorded 429 GH/s through all 11 `stopping` samples.

The owner now runs the stop's power-down in its effect arm and checks the
cooling proof once per loop tick (ADR-0016 amendment).
- Attempt 001 showed that the cooling no longer froze the hashrate. The
  power-down's frequency ramp still did, until between 6.3 s and 8.4 s
  after the pause.
- `38b9e932` zeroes the hashrate with the publication that ends hashing.
  Attempt 002 then passed.

Repo-owned commands drove both attempts: `just package`, `just
flash-monitor` (state-preserving, from the built manifest), `just
otawww-endpoint` and `just local-mining-soak --soak stopping-002`. An
independent review recomputed both verdicts from the private journals.

Parity is not promoted.

| Observation         | Attempt 001 (production `ba22b7fb`) | Attempt 002 (production `38b9e932`) |
| ------------------- | ----------------------------------- | ----------------------------------- |
| Result              | `failed` (`stop_hashrate_frozen`)   | `passed`                            |
| Firmware            | Clean package, installed state-preserving (`observed_exact_package`); handoff on its first attempt | Same |
| Mining              | 180,957 ms of hashing in one span, 11 accepted shares, 0 rejected | 180,705 ms of hashing in one span, 16 accepted shares, 0 rejected |
| Limits              | 5,130–5,223.75 mV, at most 12.22 W, at most 60 C chip, at least 2,411 RPM | 5,151.25–5,211.25 mV, at most 12.35 W, at most 59 C chip, at least 2,407 RPM |
| Pause sampling      | 13 samples, at most 2,106 ms apart  | 13 samples, at most 2,091 ms apart  |
| First sample        | 69 ms: `active`, not yet paused, 12.15 W (still hashing) | 71 ms: `active`, not yet paused, 12.26 W (still hashing) |
| `stopping`          | 2.1 s to 20.8 s, 10 samples         | 2.1 s to 20.8 s, 10 samples         |
| Hashrate while `stopping` | 470.9 GH/s to 6.3 s, then 0   | 0 in every sample                   |
| Power while `stopping` | 10.27 W, falling to 1.2–1.73 W   | 10.36 W, falling to 1.22–1.64 W     |
| `paused`            | From 22.9 s, at 45 C                | From 22.9 s, at 45 C                |
| `active` at idle    | 0 samples                           | 0 samples                           |
| Afterwards          | Paused, powered down, mining on boot off, no pause reason; same physical device; no serial holder after the run | Same |

Times count from the journaled `paused` event. `result.json` counts from
the first stop sample, 69 and 71 ms later (`stop_first_stopping_ms` 2,067
and 2,070; `stop_first_paused_ms` 22,820 and 22,816).

## What this shows

- **The cooling proof no longer freezes the API.** In both attempts the
  hashrate read 0 through the cooling from 56 C to 46 C, where the
  blocking wait kept the last rate.
- **Every `stopping` sample read 0 in attempt 002, from the first at
  2.1 s:** the zero is published with `stopping`. In attempt 001 the
  power-down's frequency ramp, 485 to 50 MHz in 100 ms steps (about 7 s),
  still held the rate at its last value for the first three `stopping`
  samples. With `38b9e932` every `stopping` sample read 0.
- **The rest of the stop matched the stopping check:**
  - the status moved only forward, from `active` to `stopping` to `paused`;
  - power fell to about 1.2–1.7 W within 8.4 s;
  - the stop confirmed as the chip reached 45 C, as with the blocking proof
    (24.9 s in the stopping check, 22.9 s here).

## Not shown here

- **Other owner work during the cooling.** The rest of the owner loop now
  runs while the board cools: inbox commands, readiness, campaign status,
  Gate rejections. Hardware shows only the hashrate, through the API.
- **The power-down itself still blocks the owner,** for about 7 s
  (the frequency ramp).
- **Stops other than a pause:** lease interrupts, network outages,
  overheat trips and faults have unit and session coverage only.
- **A cooling proof that times out,** and a stop at an owner exit: unit
  coverage only.

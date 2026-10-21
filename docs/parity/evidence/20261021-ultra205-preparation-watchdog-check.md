# Ultra 205: hardware preparation feeds the task watchdog

The preparation watchdog check (`task-preparation-rollback-cooling-watchdog`)
shows on hardware that the production owner's task watchdog stays fed
while the firmware prepares the hardware for mining.

**Before `bf98d091`:**
- Preparation took no progress callback. The frequency ramp-up alone runs
  69 delay actions of 100 ms each.
- So the owner went unfed for 9.3-10.1 s in each of the six preparations
  the retained logs captured, past the watchdog's 5 s timeout. The watchdog only warns
  (`CONFIG_ESP_TASK_WDT_PANIC` is unset), but runtime health reported the
  owner stale.
- The retained `runtime_health` lines from six earlier runs on
  19-20 October hold 66 distinct `task_watchdog_reason=feed_stale`
  readings (89 counting the overlapping break logs). There is one episode
  per preparation, every reading in subphase `effect_prepare_hardware`,
  among 9,200 distinct `feed_fresh` readings (12,729 raw).

**After `bf98d091`:** preparation reports progress from the fan proof, the
voltage request and every ASIC action of chip detection and mining-ready
initialization. The owner, the Stratum v2 session and self-test feed their
watchdogs.

Repo-owned commands drove the check: `just package`, `just flash-monitor`
(state-preserving, from the built manifest), `just otawww-endpoint` and
`just local-mining-soak --soak watchdog-001`. Attempt 001 passed. An
independent review recomputed the verdict from the private journal.

Parity is not promoted.

| Observation         | Attempt 001 (production `5166eeb5`)                                       |
| ------------------- | ------------------------------------------------------------------------- |
| Firmware            | Clean production package, installed state-preserving (`observed_exact_package`); the handoff succeeded on its first attempt |
| Preparation         | Samples at 2.1, 4.2, 6.2 and 8.3 s after the resume read subphase `effect_prepare_hardware` with `feed_fresh`; the first `active` sample came at 10.4 s (first nonzero hashrate at 14.6 s) |
| Mining              | 180,187 ms of hashing in one span, no outages, 12 accepted shares and 0 rejected |
| Limits while mining | 5,142.5–5,203.75 mV, at most 12.52 W, at most 60 C chip, at least 2,411 RPM |
| The stop            | 14 samples: 12 `stopping`, then `paused` from 25.1 s; the hashrate read 0 in every `stopping` sample; no `active` at idle power |
| Task watchdog       | `feed_fresh` in all 105 samples from the resume to the final one, 4 of them in the stop's frequency ramp-down (`safe_stop_reduce_frequency_and_nonce_state`) |
| Afterwards          | Paused, powered down, mining on boot off, no pause reason; the same physical device; no serial holder |

## What this shows

- **Preparation no longer leaves the owner's watchdog unfed.** Every sample
  taken during it read `feed_fresh`, where the logs from earlier runs on
  19-20 October show `feed_stale` in each of their six preparations.
- **The stop also stays fed,** through its frequency ramp-down and the
  polled cooling proof (`ba22b7fb`).

## Not shown here

- **The before figure comes from retained logs, not from this soak's
  sampling.** Earlier soaks did not record runtime health.
- **A failed preparation's rollback,** which now reports each safe-shutdown
  step and its cooling proof's polls: unit coverage only. It cannot be
  triggered safely on hardware.
- **The Stratum v2 session and self-test preparations:** not run here.
- **Chip detection's drain** is one action of up to 5 s (about 1.2 s
  normally). Progress before it bounds the gap to that action.

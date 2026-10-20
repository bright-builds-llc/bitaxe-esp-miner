# Ultra 205: local mining soak 005 (overnight)

Local mining soak 005 ran the device on its own overnight, on the Wi-Fi
power-save fix (`44309653`), with no Gate lease (ADR-0038 task 6). It
accumulated 8 hours of mining by status, 7 h 59 min 47 s of hashing (see
the rejudge note below). One Wi-Fi `beacon_timeout` disconnect caused
two short breaks, and local mining recovered from both without a reset.
The run then paused, and the ASIC powered down. Repo-owned commands drove
it: `just otawww-endpoint` and `just local-mining-soak --soak 005`.

Attempt 001 passed. An independent review recomputed the verdict from the
private journal and agrees with `complete`. One caveat applies: the judge
counts mining by status, and status overstated hashing for 13 s (see the
outages). Parity is not promoted.

| Observation      | Result                                                                      |
| ---------------- | --------------------------------------------------------------------------- |
| Firmware commit  | `be2ad02a` (clean package), installed state-preserving                      |
| Power save       | `wifi_power_save=none` in the retained log after boot                       |
| Handoff          | First attempt (`failed_attempts: 0`)                                        |
| Start            | First mining sample 10 s after the resume                                   |
| Mining time      | 28,800,268 ms accumulated in 481 minutes (limit 540)                        |
| Longest span     | 25,575,853 ms (7.1 hours) of unbroken mining                                |
| Outages          | Two, at 426.4 and 427.0 minutes: 21,278 ms and 12,457 ms (bound 60 s)       |
| Shares           | 1,042 accepted, 0 rejected, summed across the pool reconnect                |
| Hashrate         | 366–497 GH/s (5th–95th percentile), median 432                              |
| Mining samples   | 5,186–5,460 mV, at most 12.52 W, at most 61 C chip, at least 2,442 RPM      |
| Readings         | Power, input, chip temperature and fan fresh on every mining sample         |
| Wi-Fi            | Connected on every sample; RSSI −49 to −31 dBm                              |
| Live stops       | None: no reset, device pause, fault or overheat; one boot session           |
| Final stop       | Paused; powered down 25 s and 27 s after the pause (0.77 W at the end)      |
| Device           | Same physical device before and after; `lsof` found no serial holder        |

## The outages

The break logs, read as each break ended, show one Wi-Fi disconnect (uptime
in seconds):
1. `wifi_reconnect=disconnected reason=beacon_timeout` at 26,005.0;
2. reconnected at 26,014.6;
3. `local_mining=run_ended ending=Some(Requested)` at 26,029.2, and
   `run_began` at once. This appears in the second log only.

The first break (21 s) was the device unreachable over HTTP while Wi-Fi
reconnected. For the next 13 s, the status still read `active` on the old
pool session, but the ASIC was not hashing: it drew 1.3–1.6 W, and the
hashrate stayed frozen. The second break (12 s, `safe_blocked`) came when
that session ended and the new run took over. From the last good sample to
mining again took 47 s.

The judge counts mining by status, so it counted those 13 s. The run
cleared 8 hours by only 268 ms, so counted by actual hashing it was about
13 s short. The verdict holds under the contract's definition. A judge
that also checks power would close this gap; so would a status that
stopped reading `active` once the session stalls.

**Rejudged (2026-10-20, `task-local-mining-hashing-judge`).** The judge now
counts mining only while the ASIC hashes: the status reads `active` and no
fresh input power reading is at or below 5 W. Rejudged that way, soak 005's
recorded samples accumulate 28,787,052 ms, 12,948 ms short of 8 hours
(`soak_active_time_short`). A live run under the new judge would have
sampled a little longer, within its 540-minute deadline. Its two breaks
become one 47 s outage, within the 60 s bound. The recorded verdict stands
under the contract it ran under. Mining for 8 hours is not claimed from
this soak; 7 h 59 min 47 s is.

## What this says about the power-save fix

Before the fix, local mining met six outages in about 9.1 hours, about one
every 1.5 hours. Only one of them was confirmed as a `beacon_timeout`; the
others share its shape. Since the fix, there has been one Wi-Fi disconnect
in about 10 hours of mining: soak 004 attempt 002 (2 hours, none) and this
soak (8 hours, one).
- **Expected at the old rate:** about 6.6 outages.
- **Chance of one or fewer with no improvement:** about 1% if the old rate
  were exact. Allowing for the old rate resting on only six events, it is
  nearer 5%. Overnight radio conditions may also differ.

So turning power save off is followed by markedly fewer outages, but not
none. Why the one remaining disconnect happened is not known. Local mining
recovered from it.

## Not shown here

- Mining past 8 hours, or across a longer outage than 21 s.
- Why the access point's beacons were still missed once.
- The cause of the lease handover panic; this soak made no Gate
  connection.
- Overheat recovery, the restart guard, the crash-loop guard or the
  hardware fault latch on hardware.

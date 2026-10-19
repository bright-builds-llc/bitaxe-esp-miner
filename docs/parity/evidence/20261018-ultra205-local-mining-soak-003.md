# Ultra 205: local mining soak 003

Local mining soak 003 ran the device on its own for several hours, with no
Gate lease (ADR-0038 task 6). After an endpoint handoff, it wrote the
owner's saved pool again unchanged and resumed. It accumulated 4 hours of
mining, with one outage that recovered on its own, then paused, and the ASIC
powered down. Repo-owned commands drove it: `just otawww-endpoint` and
`just local-mining-soak --soak 003`.

Attempt 001 passed. An independent review recomputed the verdict from the
private journal and agrees with `complete`. Parity is not promoted.
"Unbounded mining" stays a non-claim: 4 hours of evidence does not show
behavior over days.

| Observation      | Result                                                                      |
| ---------------- | --------------------------------------------------------------------------- |
| Firmware commit  | `1be9a63556313c05f562188d80e87cc12a50afc8` (clean package)                  |
| App ELF SHA-256  | `a07b83353e0fb74e9bd87a0daa300c3a49bc3abc648456c7597c3386df50f825`          |
| Package manifest | `413154cb0ae1a4402be0ced2ff5963c1f86eb07c699ad376f7659866cc7d0a21`          |
| Reference        | `c1915b0a63bfabebdb95a515cedfee05146c1d50`                                  |
| Gate             | `eacd8147ead746711be6a03341a32e3cfed41a3e` (endpoint handoff only, no lease) |
| Install          | State-preserving `flash-monitor`: `observed_exact_package`                  |
| Pool             | The owner's saved pool, written again unchanged (`local-owner-supplied`)    |
| Profile          | 485 MHz and 1200 mV on every mining sample (saved settings, unchanged)      |
| Host             | `caffeinate -is` on AC power; the run in its own session; monotonic clock   |
| Start            | First mining sample 11 s after the resume                                   |
| Mining time      | 14,401,713 ms accumulated in 241 minutes (limit 300 minutes)                |
| Longest span     | 10,397,989 ms of unbroken mining, before the outage                         |
| Outages          | One, 46,175 ms, at 173.5 minutes (bound 60 s); recovered on its own         |
| Samples          | 6,432, of which 6,416 mining; maximum gap 19,212 ms (in the outage)         |
| Shares           | 477 accepted, 1 rejected, summed across the pool reconnect                  |
| Hashrate         | 366–496 GH/s (5th–95th percentile), median 432, maximum 623                 |
| Mining samples   | 5,240–5,298 mV, at most 12.62 W, at most 61 C chip, at least 2,455 RPM      |
| Readings         | Power, input, chip temperature and fan fresh on every mining sample         |
| Wi-Fi            | Connected on every sample; RSSI −48 to −30 dBm                              |
| Stops            | None: no reset, device pause, fault or overheat; one boot session           |
| Stop             | Paused once; powered down 25 s and 27 s after it (0.47 W at the end)        |
| Persistence      | Mining on boot still off; the pool saved before the resume, never rewritten |
| Device           | Same physical device before and after; `lsof` found no serial holder after  |

## The outage

At 173.5 minutes the run lost contact with the device for 19 s. The next
samples showed `safe_blocked` with Wi-Fi connected and the ASIC powered
down (0.9–1.3 W), with the chip cooling from 60 to 44 C. The share
counter then restarted, which marks a new pool session. Power rose as the
ASIC came back up, and mining resumed 46 s after the last mining sample.
Uptime and the boot session continued, so there was no reset.

It matches every earlier outage: soak 002 (attempts 001 and 002) and the
lease check's after phase. Each time, the device was unreachable for about
20 s, then showed a new pool session. Wi-Fi read connected in each case
that recorded it (soak 002 attempt 001 recorded no Wi-Fi state). That points to a
recurring network or pool interruption, which local mining recovers from on
its own. The cause is not established.

## Observations, not pass criteria

- **Hashrate:** two mining samples read 0, right after each mining start
  (0.19 and 174.26 minutes), before the averaging window filled.
- **Chip temperature:** it settled at 59–61 C under the automatic fan,
  below the 75 C limit; the 47 C minimum after the first 30 minutes came
  from the outage's cool-down.
- **VR temperature:** `vrTemp` read 0 on every sample. The Ultra 205
  reports no VR temperature, and the judge applies no VR limit.
- **Handoff:** the first endpoint handoff after the install failed to open
  the port (`operation_failed`), as in lease check attempt 004. A fresh
  handoff succeeded. This looks like the browser's serial port going stale
  after the install re-enumerates USB; it is not a device fault.

## Not shown here

The soak does not show:
- mining past 4 hours, or across an outage longer than 46 s;
- the cause of the recurring outages;
- the cause of lease check attempt 002's panic, which did not recur here;
- overheat recovery, the restart guard, the crash-loop guard or the
  hardware fault latch on hardware.

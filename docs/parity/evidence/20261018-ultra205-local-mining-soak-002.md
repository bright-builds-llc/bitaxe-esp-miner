# Ultra 205: local mining soak 002

Local mining soak 002 ran the device on its own for 2 hours, with no Gate
lease (ADR-0038 task 6). After an endpoint handoff, it saved the owner's pool
again and resumed. It then accumulated 2 hours of mining, with one outage
that recovered on its own. Finally it paused, and the ASIC powered down.
Repo-owned commands drove it: `just otawww-endpoint` and
`just local-mining-soak`.

Attempt 001 failed: two network outages broke the continuous span, which
the original rule required (`TASKS.md`). The owner then amended the rule.
Mining must accumulate within a deadline, and each outage must recover
within 60 s. Attempt 002 ran under the amended rule and passed. An
independent review agrees with `complete` after recomputing the verdict from
the journal. Parity is not promoted, and "unbounded mining" stays a
non-claim.

| Observation      | Result                                                                      |
| ---------------- | --------------------------------------------------------------------------- |
| Firmware commit  | `0beeea07bcd38daf630152052ecc8f29cdf7adb1` (clean package)                  |
| App ELF SHA-256  | `575c698cd7a1a5130826c819dc0d24d1a8d175e9e46ea80793d4442ca0be625f`          |
| Package manifest | `6d6cb30eb7d8090023e29e7d3ab94446e8b26ce675579e729901dbbac9a491e1`          |
| Reference        | `c1915b0a63bfabebdb95a515cedfee05146c1d50`                                  |
| Gate             | `eacd8147ead746711be6a03341a32e3cfed41a3e` (endpoint handoff only, no lease) |
| Install          | State-preserving `flash-monitor`: `observed_exact_package`                  |
| Pool             | The owner's saved pool, written again unchanged (`local-owner-supplied`)    |
| Profile          | 485 MHz and 1200 mV on every mining sample (saved settings, unchanged)      |
| Host             | `caffeinate -is` on AC power (run command); monotonic clock                 |
| Start            | First mining sample 15 s after the resume                                   |
| Mining time      | 7,201,440 ms accumulated in 121 minutes (limit 150 minutes)                 |
| Longest span     | 6,938,673 ms of unbroken mining, after the outage                           |
| Outages          | One, 41,267 ms, at 4.6 minutes (bound 60 s); recovered on its own           |
| Samples          | 3,224, of which 3,208 mining; maximum gap 19,207 ms (in the outage)         |
| Shares           | 336 accepted, 1 rejected, summed across the pool reconnect                  |
| Hashrate         | 363–500 GH/s (5th–95th percentile), median 433, maximum 604                 |
| Mining samples   | 5,239–5,295 mV, at most 12.45 W, at most 61 C chip, at least 2,411 RPM         |
| Readings         | Power, input, chip temperature and fan fresh on every mining sample         |
| Wi-Fi            | Connected on every sample; RSSI −39 to −32 dBm                              |
| Stops            | None: no reset, device pause, fault or overheat; one boot session           |
| Stop             | Paused once; powered down 22 s and 25 s after (0.37 W, 0.55 W), no reason   |
| Persistence      | Mining on boot still off; the owner's pool stays saved                      |
| Device           | Same physical device before and after; no serial holder afterwards          |

## The outage

At 4.6 minutes, the run lost contact with the device for 19 s. The next
sample showed `safe_blocked` with Wi-Fi connected, the ASIC powered down
(1.57 W, then about 0.9 W), and the share counter unchanged. About 10 s later the
counter restarted, which marks a new pool session. Power then rose as
the ASIC came back up, and mining resumed 41 s after the last mining sample.
Uptime kept rising and the boot session never changed, so there was no reset.

It matches the two outages of attempt 001: about 19 s unreachable, then
`safe_blocked`, then a new pool session. Attempt 001's two outages came 72
minutes apart. This run had no other outage in the 116 minutes after this
one. That is an observation, not a finding about any period.

The device reported Wi-Fi connected at every sample, before and after. No
sample was taken while the device was unreachable, so the journal does not
show whether Wi-Fi dropped during those 19 s. The cause stays unconfirmed.
The pattern is consistent with a brief network or pool interruption,
followed by the firmware's documented recovery: the local run ends, and a
new one starts.

## Observations, not pass criteria

- **Hashrate:** it read nonzero on all but two mining samples, so the
  owning-generation fix (`071b5af3`) holds under local mining. The two zero
  readings came right after each mining start, at 0.26 and 5.33 minutes.
- **VR temperature:** `vrTemp` read 0. The Ultra 205 reports no VR
  temperature, and the judge applies no VR limit.
- **Chip temperature:** after 30 minutes it stayed at 59–61 C under the
  automatic fan, well below the 75 C limit.
- **Stopped proof:** both powered-down samples are journaled
  (`stopped_sample`, then `final`), as soak 001's review asked.

## Not shown here

The soak does not show:
- mining for more than 2 hours, or across a longer outage than 41 s;
- the cause of the outages;
- a Gate lease interrupting local mining and restoring it;
- overheat recovery, the restart guard, the crash-loop guard or the hardware
  fault latch on hardware.

Those are later soaks under their own contracts.

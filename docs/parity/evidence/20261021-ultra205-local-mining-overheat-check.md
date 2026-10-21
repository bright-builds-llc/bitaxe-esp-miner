# Ultra 205: local mining overheat recovery

The overheat check (`task-local-mining-overheat-check`) exercised ADR-0038
decision 2 on hardware for the first time. After an overheat stop, local
mining restarts 100 MHz and 100 mV lower once the chip has read 45 C or
less for 30 s.

The Ultra 205 mines at about 60 C and never reaches the 75 C trip, so the
check ran on the test-only variant `overheat-trip-test` (ADR-0040), whose
local trip is 55 C. The recovery, from the overheat label on, runs the
production code. Repo-owned commands drove it:
`just package-overheat-trip-test`, `just flash-monitor
--allow-test-firmware overheat-trip-test`, `just otawww-endpoint` and
`just local-mining-soak --soak overheat-001`.

Attempt 001 passed. Production was reinstalled and confirmed afterwards.
An independent review recomputed the verdict from the private journal.
Parity is not promoted.

| Observation         | Result                                                                    |
| ------------------- | ------------------------------------------------------------------------- |
| Test firmware       | `a2db74c6`, `overheat-trip-test`, clean package, state-preserving install  |
| Variant on device   | Retained boot line `firmware_variant=overheat-trip-test local_overheat_trip_c=55` |
| Starting pair       | 500 MHz / 1200 mV saved and read back (baseline 485 MHz / 1200 mV)        |
| Trip                | `overheat_mode` showed 89.8 s after the first hashing sample; input power began falling about 15 s earlier. The sampled chip peaked at 54 C: the 55 C reading fell between 2 s samples |
| Cause               | The retained revocation detail named no failing fact: the lowered trip    |
| Fan                 | Manual 100% from the trip, as upstream leaves it                          |
| Cooling             | 29.3 s of sampled readings at 45 C or less, down to 36 C                  |
| Restart             | 38.1 s after the trip showed, at 400 MHz / 1100 mV                        |
| Lowered pair        | 301.9 s of hashing, at most 45 C, no second trip                          |
| `overheat_mode`     | Set when the trip's safe-stop ended the run; still set after 400/1100 was saved; clear on the first sample of the restarted run |
| Limits              | 5,212.5–5,461.25 mV, at most 12.19 W, at most 54 C chip, at least 2,437 RPM |
| Cleanup             | Paused, then recovery settled after 161.8 s, then 485/1200, auto fan, manual 100% and `overheat_mode` 0 restored and held 60 s (`settings_restored`; the journaled `final` sample predates the restore) |
| Production          | `a2db74c6` `production` reinstalled; its boot log has no variant line; stopped proof passed; the recovery `pause` wrote the baseline settings back with `overheat_mode` 0 and confirmed they held 60 s |
| Device              | Same physical device throughout; no serial holder afterwards              |

## What this shows

- **The overheat recovery works end to end on hardware, from the
  test-lowered trip:**
  - the trip ends the run as an overheat;
  - the fan goes to manual 100%;
  - the device restarts only after at least 29.3 s of sampled readings at
    45 C or less (the firmware's 30 s window itself is internal);
  - it restarts at the lowered pair inside the catalog, and `overheat_mode`
    reads clear once the restarted run is up.
- At 400 MHz with the fan at 100%, the chip stayed at 45 C or less, and
  the lowered pair did not trip again in the 5 minutes observed.

## Not shown here

- **The production 75 C trip:** it was not reached, and the test variant's
  trip is its own branch.
- **The 55 C trip reading itself:** it fell between samples. The trip is
  inferred from the revocation detail naming no failing fact.
- **The stay-stopped path:** a second trip, or a starting pair whose
  lowered step leaves the catalog, pauses the device itself. That path
  has unit and fake-device coverage only.
- **Overheat recovery across a reboot.**
- **The VR temperature trip.**

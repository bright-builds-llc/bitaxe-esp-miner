# Ultra 205: local mining overheat stop that stays stopped

The stays-stopped check (`task-local-mining-overheat-stays-stopped`)
exercised the second branch of ADR-0038 decision 2 on hardware. When the
pair 100 MHz and 100 mV lower would leave the catalog, an overheat stop
stays stopped: after its cooling the device pauses itself
(`pauseReason` `overheat`) until the operator resumes. The check started
at the default 485 MHz / 1200 mV; the lowered 385 MHz is below the
catalog's 400 MHz floor.

Like the overheat check, it ran on the test-only variant `overheat-trip-test`
(ADR-0040), whose local trip is 55 C. Repo-owned commands drove it:
`just package-overheat-trip-test`, `just flash-monitor
--allow-test-firmware overheat-trip-test`, `just otawww-endpoint` and
`just local-mining-soak --soak overheat-002`.

- **Attempt 001** matched the firmware's path in full, but failed on the
  judge's cooling measure (a judge bug, fixed in `16d3141c`).
- **Attempt 002 passed.** Production was reinstalled and confirmed
  afterwards. An independent review recomputed the verdict from the
  private journal.

Parity is not promoted.

| Observation         | Attempt 002 (test variant `16d3141c`)                                     |
| ------------------- | ------------------------------------------------------------------------- |
| Test firmware       | Clean packages, installed state-preserving (`observed_exact_package`) at each attempt's own commit; production was reinstalled after the last attempt, as the contract's amendment allows |
| Variant on device   | Retained boot line `firmware_variant=overheat-trip-test local_overheat_trip_c=55` |
| Starting pair       | 485 MHz / 1200 mV, saved again and read back                              |
| Trip                | `overheat_mode` showed 108.1 s after the first hashing sample; input power began falling about 15 s earlier. A sample read 55 C as power was already falling |
| Cause               | The retained revocation detail named no failing fact: the lowered trip    |
| Fan                 | Manual 100% from the trip                                                 |
| Restart             | None                                                                      |
| Device pause        | `pauseReason` `overheat`, 30.4 s after the trip showed. The cooling bound consistent with the samples was 32.4 s, against the firmware's 30 s. The retained log shows `stays_stopped=true` then `overheat_stopped paused_by_device=true` |
| Cleanup             | Paused; recovery settled after 154.2 s; then one resume cleared the device's pause and its stop marker, with no failed save, and a second pause stopped the short run. 485/1200, auto fan and `overheat_mode` 0 were restored and held 60 s |
| Limits              | 5,217.5–5,460 mV, at most 11.83 W, at most 55 C chip, at least 2,420 RPM  |
| Production          | `16d3141c` `production` reinstalled; its boot log has no variant line and no `overheat_resumed_after_boot`, so the stop marker was clear; stopped proof passed; the recovery `pause` confirmed the settings on production |
| Device              | Same physical device throughout; no serial holder afterwards              |

## Attempt 001 and the judge fix

Attempt 001 (`a2ef1fd2`) showed the same device behaviour: the lowered
trip, no restart, and the device's own pause 29.0 s after the trip showed,
with every reading from the trip to the pause at 45 C or less. Its
cleanup was the first hardware run of the clearing resume, and it worked.

The judge measured only 23.0 s of cooling. The check's own log read after
the trip left a 6 s sampling gap, which broke its continuity rule, and the
pause sample was left out.

The judge now takes the longest cooling consistent with the samples: from
the last pre-trip or warm reading, after which the firmware's window must
have begun, to the sample that showed the action. It requires the full
30 s, which a correct device meets whenever such a reading bounds the
start, as in all three runs. A gap of more than 5 s at either
end makes the result inconclusive rather than overstated. Measured that
way:

| Run                          | Cooling bound |
| ---------------------------- | ------------- |
| Attempt 001                  | 31.1 s        |
| Attempt 002                  | 32.4 s        |
| Overheat check 001 (restart) | 40.2 s        |

## What this shows

- **The overheat stop that stays stopped works on hardware, from the
  test-lowered trip:**
  - from a pair whose lowered step leaves the catalog, the trip did not
    restart mining;
  - after the cooling the firmware requires, the device paused itself
    with `pauseReason` `overheat`;
  - one operator resume cleared that pause and its stored stop marker,
    so the next boot does not pause itself.
- Together with the overheat check, both branches of ADR-0038 decision 2
  have hardware evidence: restart at the lowered pair, and stay stopped.

## Not shown here

- **The production 75 C trip,** and the firmware's own trip reading: a
  sample read 55 C as input power was already falling, so whether it was
  the trip reading is not shown.
- **A second trip after a restart:** the restart then stays stopped; that
  path has fake-device coverage only.
- **Overheat recovery across a reboot,** and the VR temperature trip.

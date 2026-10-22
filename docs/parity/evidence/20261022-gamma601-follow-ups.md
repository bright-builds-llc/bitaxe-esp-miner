# Gamma 601: Wi-Fi status and the TPS546 CML bit

The C3 follow-ups (`task-gamma-601-bring-up`) installed the schema v2
safe-boot record (`3dbd5ab4`) and observed it twice: after the install,
then after the owner power-cycled the board. Attempt 001.

- **Commands:**
  1. `just detect-ultra205`;
  2. `just flash-monitor --board 601 --port <port>
     --capture-timeout-seconds 360`: one state-preserving install;
  3. the owner removed barrel power and USB, waited about 10 s, then
     restored barrel power and then USB;
  4. `just detect-ultra205`, then `just monitor --board 601 --port <port>
     --capture-timeout-seconds 360`: receive-only, with no reset and no
     write.
- **Sources:** the source at `3dbd5ab4`; the reference at `c1915b0`.

| Record field | After the install (56 records) | After the power cycle (55 records) |
| --- | --- | --- |
| `reset_held` | `true` | `true` |
| `tps546` | `d24s` | `d24s` |
| `off` | `confirmed` | `confirmed` |
| VOUT | 10 mV | 10 mV |
| `start_status` (before the off writes) | `0x0842`: OFF, POWER_GOOD# and CML | `0x0002`: CML only |
| `start_cml` | `0x82` | `0x82` |
| `cml` (each sweep) | `0x82` | `0x82` |
| `faults` | `0x0002` | `0x0002` |
| Fan | 30 %, 3527 to 3564 rpm | 30 %, 3554 to 3583 rpm |
| VR / board temperature | 32 to 33 C / 25 to 27 C | 34 C / 27 to 28 C |
| `wifi` | `disconnected`, `disconnected_other`, then `disconnected_access_point_unavailable` (52) | `disconnected_access_point_unavailable` (55) |
| Board identity | `built=601 stored=601 identity=match` | same |

## What this shows

- **Wi-Fi.** The stored credentials are present and valid; the status
  never reads `credentials_missing` or `credentials_invalid`. The station
  cannot find the stored network (`access_point_unavailable`). The owner
  reports that the 601 holds placeholder network values, so this is the
  expected result. Our firmware reads the stock settings correctly.
- **The output comes up by itself at power-on.** After the power cycle,
  `STATUS_WORD` at the first read had neither OFF nor POWER_GOOD# set. So
  the TPS546's stored defaults turned the output on, in regulation, before
  the safe boot's off writes. The safe boot then confirmed it off, as in
  every record. Upstream boots the same way, since it also writes OFF
  during init.
  - **Residual risk:** at each power-on the core voltage is up until the
    first off write.
  - **What limits it:** the ASIC reset is driven low from the firmware's
    first instruction. The window before that instruction is not shown.
- **The CML bit.** `STATUS_CML = 0x82` (invalid or unsupported command,
  and another communication fault) is already set at the first status
  read after a cold power-on. Only the safe boot's identity reads
  (`IC_DEVICE_ID`) had touched the controller by then. So either those
  reads set it, or something did before the first transaction, such as bus
  activity while the ESP32 boots. The sweeps' reads add no further bits.

## Not shown here

- Which of the two sets the CML bit. A read of `STATUS_CML` before the
  identity read would separate them.
- How long the output stays on after power-on before the off writes.
- Wi-Fi with valid credentials.

## Effects

- the detector checks;
- one state-preserving install with its resets;
- the owner's power cycle;
- one receive-only capture;
- the installed safe boot's own effects.

There were no TPS546 writes beyond the off writes, no NVS writes and no
network requests.

# Gamma 601: the safe boot on hardware

Phase C3 of the Gamma 601 bring-up (`task-gamma-601-bring-up`, ADR-0041
item 5) installed the 601 safe-boot firmware twice, state-preserving, and
observed it from USB. Attempt 001 passed.

- **Commands:**
  1. `just detect-ultra205`, which also checks that exactly one device is
     attached;
  2. `just identify-board --board 601 --port <port>`, which writes nothing;
  3. `just flash-monitor --board 601 --port <port>
     --capture-timeout-seconds 360`;
  4. the detector, then step 3 again (the update cycle).
- **Sources:**
  - the first install from `360431c8`, the second from `8fce6458`. They
    differ only in task text; the firmware code is `8a58c262`'s;
  - the reference at `c1915b0`.
- **Connection:** only the 601 attached, barrel power on, USB through a
  USB-C hub.

## Before the installs

- **Detector:** one Espressif Serial/JTAG device in its runtime
  (`serial_jtag_runtime`), not admitted to ROM.
- **Identification:** the board's own records still named `601`, `BM1370`
  and `gamma`, with 16MB flash. The registry was not changed.

## The installs

Each install:
- built the 601 package at HEAD;
- was admitted by the flash tool, which checks the manifest's board, the
  application's `firmware_board=601` marker and the registered board;
- wrote the five update segments state-preserving, then exited to the
  application over Serial/JTAG.

The receive-only console carries the device's replay and diagnostic
records and the early ESP-IDF boot lines (to about 1.4 s uptime), not the
application's own log. A safe-boot record arrived about every 6.3 s
(6.0 to 6.6 s).

The contract's pass criteria were amended between the two installs
(`8fce6458`): the application's log lines it named are not on this
channel. Both installs are judged under the amended criteria.

| Observation | Install 1 | Install 2 (update) |
| --- | --- | --- |
| Board identity record | `built=601 stored=601 identity=match` | same |
| Safe-boot records in 360 s | 56 | 56 |
| First record's uptime (from the nearest startup record) | about 6.2 s | about 6.2 s |
| ASIC reset held | `true` in every record | same |
| TPS546 | `d24s` | `d24s` |
| Output off | `confirmed` in every record | same |
| VOUT | 10 mV in every record | same |
| VIN | 5.453 to 5.469 V | 5.453 to 5.461 V |
| IOUT | -227 to -194 mA, with the output off | same |
| VR temperature | 32 to 36 C | 32 C |
| Board temperature | 24 to 27 C | 26 to 27 C |
| Fan | 30 %, 3527 to 3545 rpm | 30 %, 3527 to 3554 rpm |
| TPS546 status faults | `0x0002` (CML), latched | same |
| Wi-Fi joined | no, in 360 s | no |
| Startup | complete (`runtime_ready`), no Wi-Fi startup failure | same |

After the second install, the detector found one device with the runtime's
USB profile, not admitted to ROM. It does not check which process holds the
port.

## Effects

- the detector checks;
- one ROM entry, one NVS read and one hard reset for the identification;
- two state-preserving installs, each with its resets;
- the installed firmware's own effects:
  - the ASIC reset held low;
  - the TPS546 off writes;
  - the EMC2101 diode configuration and fan duty;
  - a Wi-Fi attempt with the stored settings.

There was no factory install, no NVS write from the host, no VOUT enable,
no ASIC release and no network request.

## What this shows

- The safe boot holds the 601's ASIC in reset and its core-voltage output
  off. The first record seen, at about 6.2 s of uptime, already confirmed
  the output off at 10 mV, and every later record did too. The output may
  have been off earlier; the records do not show when.
- The fan and the temperature readings work: the fan at 30 %, the board
  near room temperature.
- A second install over USB, the update cycle, behaves the same, and the
  board's stored records still name the 601.

## Not shown here

- **Wi-Fi.** The station did not join in either six-minute capture, and
  no startup failure was reported. The cause is not known: the stored
  network may be unreachable, or a stock-AxeOS setting may be read
  differently. A follow-up is recorded.
- **The `CML` status bit.** It is latched, since the safe boot never sends
  `CLEAR_FAULTS`, so it may predate these boots or come from one of the
  safe boot's own reads. A follow-up is recorded.
- **Not observable on this channel:** the application's boot line and its
  `firmware_board`/`mining_owner` log lines (see the amended contract).
- **The negative current reading.** IOUT reads about -0.2 A with the
  output off. Whether that is an offset of the sensor is not shown.
- **Not covered:** the TPS546 output on (phase D), the BM1370 and mining
  (phases E and F), and HTTP.

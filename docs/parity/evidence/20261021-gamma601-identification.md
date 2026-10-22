# Gamma 601: identified and registered from its own records

The Gamma 601 bring-up's phase B (`task-gamma-601-bring-up`, ADR-0041)
identified the attached board from its own records, read-only, and
registered its base-MAC digest as board 601.

- **Commands:** `just detect-ultra205 --port <port>`, then `just
  identify-board --board 601 --port <port> --register`, from source
  `9453a1a2` against reference `c1915b0`.
- **Connection:** the board enumerated through a USB-C hub, not on the
  direct USB-C cable used for the Ultra 205. Only the 601 was attached.

| Observation            | Attempt 001                                                       |
| ---------------------- | ----------------------------------------------------------------- |
| Detector (before)      | One Espressif Serial/JTAG device in its runtime (`serial_jtag_runtime`), no ROM admission |
| Chip and flash         | ESP32-S3 admitted in the ROM downloader; flash size 16MB          |
| Stored `boardversion`  | `601`                                                             |
| Stored `asicmodel`     | `BM1370`                                                          |
| Stored `devicemodel`   | `gamma`                                                           |
| Match                  | All three name the Gamma 601                                      |
| Registration           | Added; the registry now holds boards 205 and 601                  |
| Detector (after)       | Back in its runtime, no ROM admission; the same physical port identity as before |
| Cleanup                | No serial holder; no leftover pipe directory or tool process      |

## Effects

There were three device effects, and no writes:
1. one reset into the ROM downloader (`board-info`);
2. one read of the NVS partition (0x9000, 0x6000);
3. one hard reset back to the installed stock AxeOS.

The partition, which holds Wi-Fi and pool secrets, passed from esptool to
`nvs_tool` only through private named pipes and memory. Only the three
board fields were kept. No MAC, digest, SSID or other stored value is in
the terminal output or this record.

## What this shows

- **The 601's identity:** the board's own stored records name it a Gamma
  601 with a BM1370, matching the pinned reference
  (`reference/esp-miner/config-601.cvs`). Its ESP32-S3 has 16 MB of flash.
- **The guard:** from now on, the flash tool's ROM gate refuses to write
  to it as an Ultra 205.

## Not shown here

- **The ASIC itself.** Its chip id 0x1370 is read only by mining firmware,
  which comes in phase F.
- **PSRAM, and the power controller and sensors.** These were not
  observed.

# Gamma 601: the factory install joins Wi-Fi

`task-gamma-601-wifi-provisioning`, attempt 001. A factory install of the
601 safe-boot package seeded the 601's upstream defaults
(`reference/esp-miner/config-601.cvs`) with the owner's Wi-Fi credentials,
and the station joined the owner's network.

- **Commands:**
  1. `just detect-ultra205`;
  2. `just flash-monitor --board 601 --factory-reset --wifi-credentials
     wifi-credentials.json --port <port> --capture-timeout-seconds 360`.
     The credentials file was passed by path only; its contents were not
     read, printed or copied.
- **Sources:** the source at `fc4b572f`; the reference at `c1915b0`.
- **Connection:** only the 601 attached, barrel power on, USB through a
  USB-C hub.

| Observation | Attempt 001 |
| --- | --- |
| Detector | one device, runtime USB profile, not admitted to ROM |
| Install | factory image written, NVS seed written (`nvs_seed_status: provided`), exit to the application |
| Safe-boot records in 360 s | 55 |
| `wifi` | `disconnected` once, then `connected` in the other 54 records |
| Board identity | `built=601 stored=601 identity=match` (from the seeded settings) |
| ASIC reset held | `true` in every record |
| TPS546 | `d24s`, `off=confirmed` in every record |
| VOUT / VIN | 10 mV / 5.453 to 5.461 V |
| VR / board temperature | 36 C / 29 to 31 C |
| Fan | 30 %, 3583 to 3612 rpm |
| `start_status` / `start_cml` / `cml` | `0x0842` / `0x82` / `0x82` |

## Effects

- the detector check;
- one factory install. It erased NVS, which held the stock placeholder
  settings, and seeded the 601's upstream defaults and the credentials;
- the safe boot's own effects, including joining the owner's network.

There were no TPS546 writes beyond the off writes, no VOUT enable, no ASIC
release, no mining and no HTTP requests.

## What this shows

- The 601 joins Wi-Fi with real credentials. The earlier
  `access_point_unavailable` was the stock placeholder network.
- The 601 factory install seeds the board's own records: the stored board
  reads `601`, matching the build.
- The safe boot's readings are unchanged by the new settings.

## Not shown here

- **HTTP:** the device's API was not exercised.
- **The `CML` bit:** still `0x82`, latched since boot. Its origin is the
  open bring-up follow-up.
- The board ran a little warmer (29 to 31 C) than in the earlier captures,
  within the contract's bounds.

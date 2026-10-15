# Ultra 205: BAP UART live probe, identity-bound

Read-only BAP probe ordinal 006 under `task-parity-bap-live-accessory-verification`
completed, and the run was bound to one boot of the exact installed package. An
independent review agrees.

BAP-002 is promoted to `verified`. The owner decided on 2026-10-14 to relax
the parity validator's deferred-scope guard: a `BAP-*` row may cite Ultra 205
evidence, but only this identity-bound probe. BAP-001 stays `implemented`,
because the live `deviceModel` answer differs from the reference.

| Item            | Value                                                                                                                      |
| --------------- | -------------------------------------------------------------------------------------------------------------------------- |
| Board / ASIC    | 205 / BM1366                                                                                                               |
| Firmware commit | `ecc7f122201a7e2df6df31cda9e85741ef8508db` (clean)                                                                         |
| App ELF SHA-256 | `b6330fb10f0924c81e8c6b240ac78ff08cb0d85dcd186bd4c3310c9fe46fe4e8`                                                         |
| Package digest  | not retained (the manifest was rebuilt for a later package; the device-reported commit and ELF digest are bound)          |
| Reference       | `c1915b0a63bfabebdb95a515cedfee05146c1d50`                                                                                 |
| Host link       | CP2102-type 3.3 V USB-to-TTL adapter on J6 pins 2/3/4 (GND, RX, TX only; owner-attested; see `docs/hardware/bap-uart-connection.md`), 115200 8-N-1 |
| Pool            | `pool_config: local-owner-supplied`                                                                                        |

## Commands (categories)

1. `just package` at the clean, pushed contract commit.
2. Detector naming the board's port (`serial_jtag_runtime`).
3. State-preserving `just flash` (no factory reset, no credentials), then a
   120 s settle.
4. Detector, then the receive-only USB observer (`just diagnose-usb-reboot-loop`,
   30 s).
5. `just bap probe`: `REQ systemInfo`, `REQ shares`, `SUB hashrate` at
   1,000 ms for 10 s, then `UNSUB hashrate`.
6. Detector, then the observer again.
7. `just bap bind`.

## Observed behaviour

- `REQ systemInfo` answered `deviceModel` `205` and `asicModel` `BM1366`, then
  `pool`, `poolPort` and `poolUser` in reference order. Their values are
  redacted; only their presence is recorded.
- `REQ shares` answered `0/0`.
- `SUB hashrate` returned `ACK subscribed`, then one `RES hashrate 0.00` in
  10 s, then `ACK unsubscribed`. Updates are change-only, like the reference's
  `BAP_send_if_changed`.
- There were zero malformed lines. The byte counts equal the canonical encoding
  of the frames received, and no startup banner or other traffic arrived.

## Identity binding (`bap-identity-binding-v1`: `bound`)

| Check                 | Before        | After         |
| --------------------- | ------------- | ------------- |
| Observer category     | `stable_boot` | `stable_boot` |
| Runtime identity      | exact match   | exact match   |
| Boot ordinal          | 4             | 4             |
| Reset reason category | `other`       | `other`       |
| Uptime                | 151,999 ms    | 196,014 ms    |

- The uptime advanced 44,015 ms while the host measured 44,082 ms elapsed: a
  67 ms difference against a 15,000 ms tolerance.
- The same physical device was seen before and after; its identity is not
  recorded here.
- The boot began about 196 s before the after-read, at the end of the install.
  The probe record was written between the two observer captures.
- Review hardened the judge after this run. It now also requires the boot to
  predate the first read, and the probe record to fall between the two reads.
  The sealed evidence was re-judged under those rules and is still `bound`.

## Ordinal 007: observation while mining (not a promotion input)

Ordinal 007 ran during soak 002 attempt 002, on that attempt's package
`a46f95684bb940c0773c9fe2168a72f573d77f2d`. Its only change from `ecc7f122` is
the runtime revocation clock fix; the BAP code is identical. It is not
identity-bound, because the Gate held the USB port.

- `systemInfo` answered with the same shape, and `shares` answered `4/0`.
- A 30 s `hashrate` subscription was acknowledged, delivered 17 change-only
  updates between 385.60 and 550.64 GH/s, and was unsubscribed.
- There were zero malformed lines.
- The soak's largest HTTP gap (4.6 s) and WebSocket gap (3.6 s) fell during this
  probe. Both are within the soak limits.

## Live round trip, not independent interoperability

The host client reuses `bitaxe_core::bap`, so the live run is a round trip
through the firmware's UART, not interoperability with an independent accessory
codec. The independent leg is the golden fixtures, whose checksums the review
recomputed from the reference `BAP_calculate_checksum`.

## Divergences found by review

- `deviceModel` is the board version (`205`). The reference sends the family
  name (`DEVICE_CONFIG.family.name`, `Ultra`). This blocks BAP-001.
- Subscription values differ from the reference code in source, unit or format;
  these were not exercised live:
  - `fan_speed` is a percentage, not RPM;
  - `best_difficulty` is a plain number, not a suffixed string;
  - `voltage` and `current` are in V and A, not mV and mA;
  - `chipTemp`, `vrTemp` and `power` use a different number format.

## Non-claims

- Settings and their effects.
- The Wi-Fi password subscription.
- The 300-second subscription timeout.
- AP-mode announcements on bound firmware.
- Live error and duplicate-suppression paths.
- Interoperability with an independent accessory codec.
- Accessory electrical behaviour.
- Other boards.

## Privacy

The `pool`, `poolPort` and `poolUser` values were redacted at capture. Neither
the values nor their lengths are recorded. This summary holds no device paths,
physical identity digests, IP addresses, Wi-Fi values or process IDs. The sealed
private records stay uncommitted under ignored mode-0700 roots.

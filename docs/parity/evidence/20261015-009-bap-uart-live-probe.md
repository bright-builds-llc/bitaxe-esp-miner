# Ultra 205: BAP UART live probe 009, identity-bound

Read-only BAP probe ordinal 009 completed, and the run was bound to one boot of
the exact installed package `47c0fa1f`, which carries the fixes from the
ordinal 008 review. It ran under `task-parity-bap-value-sources`, with its
contract in `task-parity-bap-live-accessory-verification`, after the owner's
go-ahead on 2026-10-15. An independent review re-derived the numbers below and
supports promoting BAP-001.

| Item             | Value                                                                                       |
| ---------------- | ------------------------------------------------------------------------------------------- |
| Board / ASIC     | 205 / BM1366                                                                                |
| Firmware commit  | `47c0fa1fdebec626e1026d19f5c3384d75ccd0ba` (clean, pushed)                                  |
| App ELF SHA-256  | `bdd876992632791044dabe61e59ca85c88427ab9ed07baf027a10ab44f33b7ae`                          |
| Manifest SHA-256 | `0bfa70f730bbb7b3ab51bcd51f744797ed4a3d7abf95631cd21bad7a43c6584a`                          |
| Reference        | `c1915b0a63bfabebdb95a515cedfee05146c1d50`                                                  |
| Host link        | CP2102-type 3.3 V adapter on J6 pins 2/3/4 (GND, RX, TX only; owner-attested), 115200 8-N-1 |
| Pool             | `pool_config: local-owner-supplied`                                                         |

## Commands (categories)

These are ordinal 008's seven steps, unchanged except
`--ordinal 9 --listen-seconds 5` and the probe's three added requests:

1. Build the package at the clean, pushed contract commit.
2. Run the detector, naming the board's port.
3. Install with a state-preserving `just flash`: no factory reset, no
   credentials.
4. Wait 120 s, then run the detector and the receive-only USB observer.
5. Run `just bap probe` (14 steps).
6. Run the detector and the observer again.
7. Run `just bap bind`.

## Observed behaviour (record `bap-readonly-probe-v2`: `complete`)

- `REQ systemInfo` answered `deviceModel` `Ultra` and `asicModel` `BM1366`,
  then `pool`, `poolPort` and `poolUser` in reference order. The pool values
  were redacted at capture.
- `REQ shares` answered `0/0`, and `REQ block_height` answered `0`.
- `REQ found_block` answered with the reference token `block_found` and `0`.
- `REQ show_new_block` answered `0`.
- Each subscription used a 1,000 ms interval and listened for 5 s. Each was
  acknowledged `subscribed` and `unsubscribed`:

  | Subscription      | Updates received                                          |
  | ----------------- | --------------------------------------------------------- |
  | `hashrate`        | `0.00`                                                    |
  | `temperature`     | `chipTemp` `29.000000` and `vrTemp` `0.000000`, once each |
  | `power` (W)       | 0.45, 0.37, 0.44                                          |
  | `voltage` (mV)    | 5468.75, 5480.00, 5478.75, 5481.25, 5478.75               |
  | `current` (mA)    | 68.75, 57.50, 66.25, 80.00, 63.75                         |
  | `fan_speed` (RPM) | 3126, 3119, 3126                                          |
  | `best_difficulty` | `0`                                                       |
  | `block_height`    | `0`                                                       |
  | `found_block`     | `0`                                                       |

- At most five updates arrived per 5 s window, and each was a change.
- There were zero malformed lines. Every step's byte count equals the canonical
  encoding of its frames, and no banner, AP announcement or status frame
  arrived.
- The board was not mining, which is lease-gated by design. Hashrate, shares,
  block and best-difficulty values are idle values.

## Identity binding (`bap-identity-binding-v1`: `bound`)

| Check                 | Before        | After         |
| --------------------- | ------------- | ------------- |
| Observer category     | `stable_boot` | `stable_boot` |
| Runtime identity      | exact match   | exact match   |
| Boot ordinal          | 1             | 1             |
| Reset reason category | `other`       | `other`       |
| Uptime                | 152,374 ms    | 232,954 ms    |

- The uptime advanced 80,580 ms while the host measured 80,773 ms: a 193 ms
  difference against a 15,000 ms tolerance. Device uptime restarts at every
  boot, so no reset happened between the reads.
- The boot predates the first read by about 152 s. It falls about 0.5 s
  before the install command finished, which is the install's own post-flash
  reset. The probe record was written between the two observer captures.
- All detector runs saw the same physical device as ordinal 008. Its identity
  is not recorded here.

### Why the boot ordinal is 1

The ordinal lives in reset-retained RTC memory. It restarts at 1 after a
power-on or brownout, or when the retained record is zero or fails its
integrity checks.

- This was not a cold start: the reset reason is `other`.
- A power cycle before the install would have given 2 after the install's
  warm reset.
- The record's address in `.rtc_noinit` depends on each build's link layout,
  and that address has been seen to move between builds. The new image most
  likely read an invalid record and started counting again.

The binding does not compare the ordinal across packages. Within this one
image the address is fixed, and the uptime continuity independently rules out
any reset between the reads.

## Fixes from the 008 review, now on the bound package

| Fix                                                             | Evidence                                                           |
| --------------------------------------------------------------- | ------------------------------------------------------------------ |
| Per-token change detection, as in `BAP_send_if_changed`        | Unit-tested; `chipTemp` did not change in this run, so not live   |
| `REQ found_block` answers `block_found`                         | Observed live                                                      |
| Subscriptions keep publishing and timing out in AP mode         | Unit-tested                                                        |
| Empty values are sent without a value field                     | Unit-tested                                                        |
| Intervals parse like C `atoi`                                   | Unit-tested                                                        |

## Non-claims

- Settings: the firmware answers accepted setting intents with `set_failed`
  (`task-parity-bap-settings-effects`).
- The `wifi` and `shares` subscriptions on hardware. The `wifi_password`
  omission is an owner-accepted divergence.
- Values containing `,` or `*` are refused, as recorded.
- On hardware: the 300-second timeout, AP-mode announcements and errors,
  default-interval and re-subscription timing, per-token resend, and duplicate
  suppression.
- The publish order when several subscriptions are due on one tick.
- Values while mining.
- The all-time best difficulty restored from NVS
  (`task-parity-best-difficulty-persistence`).
- Interoperability with an independent accessory codec: the host reuses
  `bitaxe_core::bap`.
- Accessory electrical behaviour.
- Other boards.

## Privacy

The pool values were redacted at capture. Neither the values nor their lengths
are recorded. This summary holds no device paths, physical identity digests, IP
addresses, Wi-Fi values or process IDs. The sealed records stay uncommitted
under ignored mode-0700 roots.

- Probe record SHA-256: `abdbee03…ee6fd2cb`.
- Binding record SHA-256: `1b7d4b1c…fdf52526`.

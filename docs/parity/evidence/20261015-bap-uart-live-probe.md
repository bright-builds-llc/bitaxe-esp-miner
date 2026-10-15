# Ultra 205: BAP UART live probe 008, identity-bound

Read-only BAP probe ordinal 008 completed, and the run was bound to one boot of
the exact installed package. It ran under `task-parity-bap-value-sources`, with
its contract in `task-parity-bap-live-accessory-verification`. An independent
review re-derived the numbers below.

BAP-001 stays `implemented` after this run. The review found per-parameter
instead of per-token change detection, a `REQ found_block` token divergence,
and three smaller differences. All are fixed after this run (see
[Divergences](#divergences-found-by-review-fixed-after-this-run)). A fresh bound
ordinal on the fixed package is needed before promotion.

| Item             | Value                                                                                       |
| ---------------- | ------------------------------------------------------------------------------------------- |
| Board / ASIC     | 205 / BM1366                                                                                |
| Firmware commit  | `d2941b5ee7e3376e22f11af5c25ff11dbd8b0d31` (clean, pushed)                                  |
| App ELF SHA-256  | `a036cf7d1f4c3f4015975c3ff010f9ebce68f3a1263c60b5c74536f808f9197c`                          |
| Manifest SHA-256 | `edef6805c0cd0c8b185c084e19952d18075c075fee313cd03be529071d4cfc36`                          |
| Reference        | `c1915b0a63bfabebdb95a515cedfee05146c1d50`                                                  |
| Host link        | CP2102-type 3.3 V adapter on J6 pins 2/3/4 (GND, RX, TX only; owner-attested), 115200 8-N-1 |
| Pool             | `pool_config: local-owner-supplied`                                                         |

## Commands (categories)

These are ordinal 006's seven steps, unchanged except
`--ordinal 8 --listen-seconds 5`:

1. Build the package at the clean contract commit.
2. Run the detector, naming the board's port.
3. Install with a state-preserving `just flash`: no factory reset, no
   credentials.
4. Wait 120 s, then run the detector and the receive-only USB observer.
5. Run `just bap probe`.
6. Run the detector and the observer again.
7. Run `just bap bind`.

## Observed behaviour (record `bap-readonly-probe-v2`: `complete`)

- `REQ systemInfo` answered `deviceModel` `Ultra` and `asicModel` `BM1366`,
  then `pool`, `poolPort` and `poolUser` in reference order. The pool values
  were redacted at capture.
- `REQ shares` answered `0/0`.
- Each subscription used a 1,000 ms interval and listened for 5 s. Each was
  acknowledged `subscribed` and `unsubscribed`:

  | Subscription      | Updates received                                                                             |
  | ----------------- | -------------------------------------------------------------------------------------------- |
  | `hashrate`        | one: `0.00`                                                                                  |
  | `temperature`     | `chipTemp` `30.000000` and `vrTemp` `0.000000`, once each (no TPS546, so the reference also reports `vrTemp` 0) |
  | `power` (W)       | 0.37, 0.42, 0.41, 0.31, 0.51                                                                 |
  | `voltage` (mV)    | 5481.25, 5480.00, 5478.75, 5481.25, 5478.75                                                  |
  | `current` (mA)    | 61.25, 60.00, 63.75, 82.50, 57.50                                                            |
  | `fan_speed` (RPM) | 3126, 3119, 3126                                                                             |
  | `best_difficulty` | `0`                                                                                          |
  | `block_height`    | `0`                                                                                          |
  | `found_block`     | `0`                                                                                          |

- At most five updates arrived per 5 s window, and each was a change. Constant
  values were sent once.
- There were zero malformed lines. Every step's byte count equals the canonical
  encoding of its frames, and no banner, AP announcement or status frame
  arrived.
- The board was not mining, which is lease-gated by design. Hashrate, shares
  and block values are idle values.

## Identity binding (`bap-identity-binding-v1`: `bound`)

| Check                 | Before        | After         |
| --------------------- | ------------- | ------------- |
| Observer category     | `stable_boot` | `stable_boot` |
| Runtime identity      | exact match   | exact match   |
| Boot ordinal          | 7             | 7             |
| Reset reason category | `other`       | `other`       |
| Uptime                | 152,874 ms    | 232,004 ms    |

- The uptime advanced 79,130 ms while the host measured 79,179 ms: a 49 ms
  difference against a 15,000 ms tolerance.
- The boot predates the first read and matches the end of the install. The
  probe record was written between the two observer captures.
- All three detector runs saw the same physical device. Its identity is not
  recorded here.

## Divergences found by review (fixed after this run)

None of these was exercised by this run.

- Multi-token subscriptions (`temperature`, `wifi`) resent every token when any
  one changed. The reference's `BAP_send_if_changed` resends only the changed
  token.
- `REQ found_block` answered with the token `found_block`. The reference
  answers `block_found`; only its subscription update uses `found_block`.
- In AP mode after a Wi-Fi loss, subscriptions paused. The reference keeps
  publishing them and timing them out.
- An empty value suppressed the whole frame. The reference sends the frame
  without a value field.
- Intervals were parsed strictly. The reference uses C `atoi`.

Not BAP: `best_difficulty` comes from the session's best only, while the
reference restores the all-time best from NVS at boot. This is a system value
source, tracked separately.

## Non-claims

- Settings and their effects.
- The Wi-Fi subscription. Its `wifi_password` omission is an owner-accepted
  divergence.
- The 300-second timeout.
- First-update timing at the default interval, and the reset on
  re-subscription: this run used 1,000 ms and one `SUB` each.
- AP-mode announcements and errors on bound firmware.
- Live duplicate suppression.
- Values while mining.
- Interoperability with an independent accessory codec: the host reuses
  `bitaxe_core::bap`.
- Accessory electrical behaviour.
- Other boards.

## Privacy

The pool values were redacted at capture. Neither the values nor their lengths
are recorded. This summary holds no device paths, physical identity digests, IP
addresses, Wi-Fi values or process IDs. The sealed records stay uncommitted
under ignored mode-0700 roots. Probe record SHA-256: `bc6f0979…0c3405`; binding
record SHA-256: `1a100099…2ca2dc0`.

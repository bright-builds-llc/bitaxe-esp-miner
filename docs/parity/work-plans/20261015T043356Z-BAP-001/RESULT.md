# Parity work result

- Parity row: `BAP-001`
- Final status: `verified`
- Implementation commit: `47c0fa1fdebec626e1026d19f5c3384d75ccd0ba`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Hardware ordinal: `009`, with the owner's go-ahead on 2026-10-15

## Evidence and verification

The read-only probe ordinal 009 completed on one Ultra 205 through the J6
accessory port. It was bound to one boot of exact package `47c0fa1f`: the
same physical device and exact build before and after, an unchanged boot
ordinal and reset category, and an uptime advance within 193 ms of host time.

Observed live, with every value in the reference's format and unit:

- All five answered requests, including `REQ found_block` answered as
  `block_found`.
- Nine subscriptions, each acknowledged, updated change-only at most once per
  second, and unsubscribed.
- Zero malformed lines.

The unit tests cover the one-second tick, the first update on the next tick,
per-token change detection, the reset on every `SUB`, the strict 300-second
timeout, subscriptions continuing in AP mode, empty values sent without a
value field, and `atoi` intervals.

An independent review re-derived the binding and values from the private
records and checked each fix against the pinned reference. It supports
`verified`, with the non-claims below.

Summary: [`docs/parity/evidence/20261015-009-bap-uart-live-probe.md`](../../evidence/20261015-009-bap-uart-live-probe.md).

## Accepted divergences and non-claims

Accepted divergences:
- The `wifi` subscription never sends the stored Wi-Fi password (owner
  decision 2026-10-14).
- Values containing `,` or `*` are refused rather than sent as broken frames.

Non-claims:
- Settings, which the firmware answers with `set_failed`
  (`task-parity-bap-settings-effects`).
- The publish order of several subscriptions due on one tick.
- On hardware: the `wifi` and `shares` subscriptions, the 300-second timeout,
  AP-mode announcements and errors, timing at the default interval and on
  re-subscription, per-token resend, and duplicate suppression.
- Values while mining.
- The all-time best difficulty
  (`task-parity-best-difficulty-persistence`).
- Interoperability with an independent accessory codec.
- Accessory electrical behaviour.
- Other boards.

# Parity work plan

- Run ID: `20261015T023347Z-BAP-002`
- Parity row: `BAP-002`
- Initial status: `implemented`
- Source commit: `ecc7f122201a7e2df6df31cda9e85741ef8508db`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-bap-live-accessory-verification`
- Continues plan: `docs/parity/work-plans/20260804T180000Z-BAP-002/RESULT.md`
- Hardware ordinal: `006`

## Selection

`BAP-002` was `implemented` with `unit,golden` evidence. Its one stated gap
was firmware UART and live accessory interoperability. The owner authorized
the live BAP probes and the identity-bound promotion run on 2026-10-14.

## Scope and non-scope

One identity-bound, read-only probe on the Ultra 205 through a CP2102 adapter
on J6 (GND, RX and TX only):

- `REQ systemInfo`, `REQ shares`, a 10-second `SUB hashrate` and `UNSUB
  hashrate`;
- bound by `just bap bind` to one boot of the exact package, with
  receive-only USB observer reads before and after.

Out of scope: any `SET` or `CMD` frame, settings and their effects, Wi-Fi
values, AP mode, the subscription timeout, and other boards.

## Verification

- A sealed probe record (`complete`) and a sealed binding record (`bound`).
- An independent review of the private records against the pinned reference.
- A redacted summary in
  `docs/parity/evidence/20261014-bap-uart-live-probe.md`, then
  `just verify-redaction`.

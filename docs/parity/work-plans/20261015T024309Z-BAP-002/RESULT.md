# Parity work result

- Parity row: `BAP-002`
- Final status: `verified`
- Implementation commit: `ecc7f122201a7e2df6df31cda9e85741ef8508db`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Hardware ordinal: `006` of `task-parity-bap-live-accessory-verification`

## Evidence and verification

The evidence and its independent review are recorded in
[`20261015T023347Z-BAP-002/RESULT.md`](../20261015T023347Z-BAP-002/RESULT.md)
and summarized in
[`docs/parity/evidence/20261014-bap-uart-live-probe.md`](../../evidence/20261014-bap-uart-live-probe.md).

- Unit and golden tests match the pinned reference. The review recomputed the
  golden checksums from `BAP_calculate_checksum`.
- A live, read-only probe on the Ultra 205's J6 accessory port completed. It
  was bound to one boot of exact package `ecc7f122`: the same physical device,
  build and boot ordinal before and after, and an uptime advance within 67 ms
  of host time.

The owner decided on 2026-10-14 to relax the parity validator's deferred-scope
guard for BAP. A `BAP-*` row may now cite Ultra 205 evidence, but only the
identity-bound live probe, so reused general Ultra 205 evidence is still
rejected. Regressions cover both cases.

## Limits

The host reuses `bitaxe_core::bap`, so the live run is a round trip through the
firmware UART, not interoperability with an independent accessory codec.

Non-claims:
- an independent accessory implementation;
- live duplicate suppression;
- live error and AP-mode paths;
- setting validation on hardware, and setting effects;
- the Wi-Fi password subscription;
- the 300-second timeout;
- other boards.

The value sources, including `deviceModel`, belong to BAP-001.

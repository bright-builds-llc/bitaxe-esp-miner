# Parity work result

- Parity row: `BAP-002`
- Final status: `implemented` (promotion held; see below)
- Implementation commit: `ecc7f122201a7e2df6df31cda9e85741ef8508db`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Hardware ordinal: `006` of `task-parity-bap-live-accessory-verification`

## Evidence and verification

On one detector-admitted Ultra 205 (BM1366), the read-only probe completed:

- `REQ systemInfo` answered its five tokens in reference order.
- `REQ shares` answered in the accepted/rejected format.
- `SUB hashrate` was acknowledged as `subscribed`, delivered change-only
  updates, and was acknowledged `unsubscribed` after `UNSUB`.
- Every `RES` and `ACK` frame verified its checksum, with zero malformed lines.

The run was bound to one boot of the exact package:

- Before and after the probe, the receive-only USB observer reported
  `stable_boot`, the package's commit and ELF digest, boot ordinal 4 and reset
  category `other`, on the same physical device.
- The uptime advanced 44,015 ms while the host measured 44,082 ms.
- The binding judge was hardened after review: the boot must predate the first
  read, and the probe must fall between the reads. The sealed evidence was
  re-judged under those rules and is still `bound`.

An independent review re-derived the numbers from the private records. It
recomputed golden checksums from the reference `BAP_calculate_checksum`, and
compared the observed tokens with `bap_handlers.c` and `bap_subscription.c`.

Summary: [`docs/parity/evidence/20261014-bap-uart-live-probe.md`](../../evidence/20261014-bap-uart-live-probe.md).

## Limits

The host reuses `bitaxe_core::bap`, so the live run is a round trip through the
firmware UART, not interoperability with an independent accessory codec. The
independent leg is the golden fixtures.

The value sources, including `deviceModel`, belong to BAP-001, which stays
`implemented`.

Non-claims:
- an independent accessory implementation;
- live duplicate suppression;
- live error and AP-mode paths;
- setting validation on hardware, and setting effects;
- the Wi-Fi password subscription;
- the 300-second timeout;
- other boards.

## Promotion held

The independent review supports `verified` for this row, but the parity
validator's 2026-06-28 guard treats every `BAP-*` row as deferred or non-205
scope that may not be verified with Ultra 205 evidence. That guard predates the
in-scope J6 accessory port and this BAP-specific, identity-bound evidence.
Relaxing it is an owner scope decision, so the row records this evidence and
stays `implemented` until that decision is made.

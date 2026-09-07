# ADR-0023: Bound native Serial/JTAG receive progress

Accepted 2026-09-05 during the owner-authorized iterative qualification. This
supersedes only ADR-0021's serial 0.1 wire contract. The fixed USB controller,
direct Web Serial, Controller 0.4, possession 0.2 and ADR-0022's four-cycle
sample remain.

## Evidence and decision

Attempt-009 observed an actual received request padding length of 63,584 bytes
for 65,376 expected bytes, while the response padding remained complete.
The count check rejected it. The exact ESP-IDF 5.5.4 driver has a silent drop
path when its 4,096-byte receive ring fills; the missing 1,792 bytes equal
twenty-eight 64-byte packets. No hardware overflow counter was measured, so
the report distinguishes observed input loss from that supported mechanism.

Use Gate's successor serial 0.2 contract: session-bound cumulative receive
acknowledgements, a 2,048-byte host outstanding window, native chunks no larger
than 1,024 bytes, and exact lexical payload byte-length/SHA-256 verification
before dispatch. Keep 64 KiB control payloads and 65 KiB wire records. The signed
manifest binds these semantics. Credits grant no work or liveness authority.

The bounded uncredited Hello is excluded from counters. Start both counters
after its delimiter/acknowledgement, before possession. Count subsequent raw
bytes actually drained, including incomplete records. Coalesce epoch-bound
credit in the sole writer independently of Worker commands and NVS; preserve
heartbeat priority. Reject stale, excessive or overflowing acknowledgements.

One existing two-second record-write bound includes all credit waits and final
consumption. Refresh an admitted heartbeat before a potentially long record.
An interrupted record must be aborted, never completed or spliced with Restore
or Close. Independent 2.8-second authority expiry and generation revocation
remain, with fresh-session recovery after cancellation.

A complete Close may receive one final consumption credit from its bounded
closing transport epoch, after Work authority is revoked and only while no
newer Hello exists. This does not permit ordinary retired-epoch output. The
canonical contract bounds stale-credit and partial-output bootstrap recovery.

Reuse receive storage and wipe only initialized bytes before reuse. Do not
reallocate secret-bearing vectors or create large stack temporaries. The driver
buffers, 98,304-byte internal allocation reserve, safety thresholds and ordered
hardware shutdown are unchanged.

## Verification and rollout

The actual channel must reproduce input loss with a bounded stalled receiver,
then demonstrate the outstanding-byte invariant, exact delivery or closed
failure, integrity rejection before dispatch and no writes after revocation.
Cross-language lexical/hash vectors, partial records, final short chunks,
counter boundaries, lost credits, blocked commands and cleanup are required.

Gate's canonical specification is `docs/protocol/bwg-worker-serial-0.2.md` in
the pinned Gate source. Publish coordinated commits/archive and a clean native
package before another hardware effect. Requalify four cycles of that exact
pair. Preserve the original campaign, failed attempts, consumed deliveries and
all older evidence without relabeling. No reservation is refunded, no new
campaign is minted, and the 240-second ceiling remains. Mining and unrelated
parity criteria remain unverified until their required evidence exists.

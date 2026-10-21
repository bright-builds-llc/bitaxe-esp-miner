# Ultra 205: local mining soak 005, rejudged by hashing

`docs/parity/evidence/20261020-ultra205-local-mining-soak-005.md` records
soak 005 as it ran. That file is pinned by a verified-revision authority
(NET-001), so this note sits beside it instead of inside it.

On 2026-10-20 (`task-local-mining-hashing-judge`) the soak judge began
counting mining only while the ASIC hashes: the status reads `active` and
no fresh input power reading is at or below 5 W. Before, it counted the
status alone, which read `active` for 13 s while the ASIC idled during
soak 005's one Wi-Fi disconnect.

Rejudged that way, soak 005's recorded samples:
- accumulate 28,787,052 ms, 7 h 59 min 47 s of hashing, 12,948 ms short of
  8 hours (`soak_active_time_short`);
- make its two breaks one 47 s outage, within the 60 s bound.

A live run under the new judge would have sampled a little longer, within
its 540-minute deadline. The recorded verdict stands under the contract it
ran under: 8 hours of mining by status. Mining for 8 hours is not claimed
from this soak; 7 h 59 min 47 s of hashing is.

NET-001 cites soak 005 only for its Wi-Fi disconnect count, which the
rejudge does not change.

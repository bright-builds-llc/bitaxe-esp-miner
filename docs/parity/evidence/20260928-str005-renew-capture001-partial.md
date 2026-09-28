# Capture001 stopped before self-test

Capture001 collected a complete healthy baseline on the qualified f000872f image,
then stopped at capture admission. The sealed outcome is `complete=false` with
`capture_review_missing`. There is no self-test claim, panic, new dump or clear.
Restoration and browser/server/serial release passed.

The existing full core region was independently read and measured as974,848 bytes
of0xFF. Its successful read/return/cleanup producer is sealed at
`scratch/str005-renew/current-recovery002/attempt`, inventory SHA-256
`abc027bca50ae320ce22ba5f691317b5784d6a903dac85e10d56b8c6abf52117`.
Capture001 freshly observed the expected subsequent boot12.

The host admission helper called `baselineConclusion(parts)` without the
successor context. The new collector's valid `first_failure:null` metadata then
failed the legacy closed shape with `v2_object_shape`. Read-only experiments on
the actual saved parts reproduced that rejection; passing the context succeeded,
as did a separate legacy-shape control. The correction passes the context rather
than removing failure metadata or relaxing the schema. The regression went red
on the old call and green after correction, and still rejects reported failures
and successor metadata presented under a legacy owner.

Private failed root: `scratch/str005-renew/capture001/attempt`.
Sealed inventory SHA-256:
`014a1580a5612a112d3ac81d1fd18ef47b3123cf70a081649ac1c7fbbbffac99`.
Offline re-evaluation verifies the software correction only; it does not promote
this historical attempt. A fresh capture must repeat healthy observations and
prove the same post-read boot before its one allowed self-test. The empty-region
proof may be reused only with that unchanged boot/image lineage. No mining,
authorization issuance, reservation, reset or parity promotion occurred here.

# Capture002 stopped before fault issuance

Capture002 passed fresh baseline and capture review on f000872f/a3e25741,
Gate 9643e876, boot 12. The self-test was not claimed or invoked. The sealed result
is incomplete with `self_test_result_missing`.

The saved candidate diagnostic exports contained the six bootstrap categories,
without a current-boot core-store receipt. Applying the published readiness guard
to both actual exports reproduced `panic_store_receipt_missing_or_ambiguous`.
The firmware replays the additional diagnostic slots asynchronously; ordinary
connection readiness alone does not prove core-store readiness. The guard correctly
prevented fault issuance. The original exception was not persisted by this attempt;
the saved inputs and read-only guard reproduction localize the admission failure.

Independent candidate recovery subsequently passed on the same boot 12, with
next20 / last19 / 1,920,000 ms / pending=false and the original campaign unchanged.
Restoration and complete browser/server/serial release passed. No grant, mining,
reservation, reset, new dump or clear occurred.

Private root: `scratch/str005-renew/capture002/attempt`.
Sealed inventory SHA-256:
`633e3b0d094ed5a29d39e49791d376d321d2cc1213b6c62d3e58b13ed2af25b0`.
The unchanged empty-region producer remains sealed with SHA-256
`abc027bca50ae320ce22ba5f691317b5784d6a903dac85e10d56b8c6abf52117`.

The correction polls actual fresh exports for at most 30 seconds before any claim.
Only an absent current-boot receipt is pending; wrong or conflicting source, boot,
capacity, stage or metadata rejects. The final claim repeats the strict checks.
Timeout/rejection now preserves a typed first cause and independently attempts
Stop/Close. A returned fault without observer evidence stays incomplete. Tests
cover delayed arrival, deadline boundaries, drift, missing evidence and cleanup.
This correction does not promote capture002. A new admitted attempt must repeat
fresh same-boot observations. Parity remains 90/95.

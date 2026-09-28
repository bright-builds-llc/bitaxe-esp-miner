# Focused accepted-share execution contract

Hardware requires the exact published task-owned admission manifest and explicit
source/task activation before any device access. This command cannot
install, restart, clear core storage, replay prior grants, or suppress heartbeats.

The command surface is `just str005-share-probe preflight --private-root ROOT --admission MANIFEST --gate-root GATE --fixture-binary FIXTURE`, followed by
`serve --private-root ROOT --authority-directory AUTHORITY` and, after all owners
exit, `finish --private-root ROOT`. Paths are absolute. Evidence directories are
0700, files 0600, stdout and stderr separate private regular files. The root is
new, ignored, exclusive and sealed once. No credentials or authority contents
are copied into evidence. Runtime socket tuples stay in RAM; only independently
compared safe facts are persisted.

Preflight verifies the task-pinned manifest hash, startup003 seal, successful
Renew-successor installation, exact installed source/full ELF, self-test core
checksum/full ELF/captured cutoff, archive-bound official clear and erased
readback, and post-clear authenticated idle restoration/accounting. The Gate and
host source are clean and pushed. Source identities remain distinct from the
installed firmware. Fixture provenance is canonical and source-bound. The old
361 image is ineligible: only a verified `renewSuccessor` installation is admitted.

Fresh detection is required in sibling `detector.stdout.log` for preflight and
`startup-detector.stdout.log` for effect admission (60 seconds, same physical
identity, no prior serial holder). Same-page baseline proof expires after 120
seconds. Cooling and budget review, prewarmed signer and a fresh local fixture
precede issuance; fixture readiness expires after 10 seconds. One fresh ledger
ordinal and nonce reserve 180000 ms; the initial authority is 60000 ms with
20000 ms renewal cadence. Exactly two renewal authorizations are prepared, and
zero to two may actually execute. No renewal creates a new allowance.

The browser invokes Start once, bounds its reply to 30 seconds, then polls exact
known-attempt status. The host validates the real finite-target header, nonce,
submission and native ACK against atomic independent fixture evidence. It never
uses a fixture ACK write alone as device receipt. Normal Stop is requested as
soon as proof is verified or any failure occurs, and at most 45 seconds after
Start reply, through an independent timer even if an observation blocks. These
are host request bounds, not physical ASIC-off claims. A late Start reply gets
another Stop and Close; it never triggers another Start. Gate automatic renewal
uses the completed Start reply as its clock origin, verified from production
functions with mocked controller/parser/checkpoint dependencies.

Failure-independent state, ledger, original budget, retained status and diagnostic
reads each have a 30-second bound. Stop/Close/recovery each have a 150-second bound.
The same page records a fresh authorization recovery checkpoint, then a new native
logical session collects mandatory fresh recovery. Normal Stop resources and
checkpoint, one exact charged completion, unchanged exhausted original budget,
restoration and preservation must pass independently. A post-Stop authority
cancellation is permitted only by the strict existing native normal-stop ordering
validator; it cannot erase or fabricate the earlier accepted-share proof.

Fixture natural completion has a 5-second bound; cleanup independently attempts
termination/reap and listener absence even on proof failure. Final detection and
holder checks cover both original and current nodes. The final reader recomputes
PoW/ACK facts and source history from retained files, verifies fixture completion,
and seals partial failures without claiming qualification. A precise missing
proof remains a blocker. There is no automatic retry, parity promotion, or task
archival from this command.

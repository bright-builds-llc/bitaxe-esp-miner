# Parity work closure

- Parity row: `STR-005`
- Final status: `implemented`
- Outcome: `blocked`
- Verification claimed: `no`
- Plan SHA-256: `669c9d2d097796533b5fea72bb4f868579d1876fb3fdd92589da057d8931ab9c`
- Active task: `task-parity-str005-installed-package-recovery-002`
- Attempt-004 consumed: `no`

## Closure reason

Clean pushed source `de081a9411b323ede1334b4c87740df8839095e1`
and its exact canonical package passed every mandatory gate. A fresh detector
admitted exactly one Ultra 205 before recovery-002.

The corrected explicit 460800-baud fallback completed all eight approved
firmware ranges within their 600-second per-range limits. It retained every
binary at mode `0600`, excluded NVS and coredump storage, validated the
partition table and installed running-image identity, created the protected
snapshot restore bundle, and confirmed the installed runtime remained
unchanged. This proves the recovery-001 readback timeout is resolved.

The owner then stopped at earliest category `evidence_invalid`, checkpoint
`independent_validation`, because its independent validator child returned
nonzero. It did not publish the readiness projection. A bounded post-run
diagnostic found that the exact retained bundle and candidate projection pass
the same validator with the original source and immutable-plan bindings. The
owner retained no private child diagnostic capable of distinguishing a
transient launcher/environment failure from another child-only predicate, so
the discrepancy is not safely attributable. Post-run acceptance does not
override the authoritative failed recovery command or publish its candidate.

Attempt-004 was withheld and remains unused. No flash write, NVS write,
new-baseline adoption, fixture start, pool connection, mining, ASIC control,
settings change, or campaign effect occurred. The unpublished candidate was
moved into the protected private root; every private file/directory passed the
required mode check. No owned process remains, and a post-run detector again
reported exactly one ready Ultra 205.

## Next safe action

Keep `STR-005` at `implemented`. Do not retry recovery-002 or run attempt-004
under this closed plan. A future continuation must first add a protected closed
receipt for the independent-validator child invocation, including bounded
launcher, exit, timeout, working-directory, and output-digest facts, plus a
real-launch regression that reproduces or disproves the child-only discrepancy.
Only a new active task and immutable plan with that changed diagnostic boundary
may authorize a fresh recovery ordinal. Attempt-004 remains available only
because no campaign effect began; attempt-005 remains prohibited.

## Non-claims

This closure does not verify an Ultra 205 Noise handshake, V2 channel, ASIC
work, target-qualified nonce, encrypted share, accepted response, terminal safe
stop, package/settings restoration, external-pool interoperability, mixed-
protocol fallback, other boards, unbounded mining, OTA, or release readiness.
It does not create `RESULT.md`, hardware-regression evidence, or `verified`
status.

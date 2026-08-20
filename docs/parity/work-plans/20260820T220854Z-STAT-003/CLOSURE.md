# Parity work closure

- Parity row: `STAT-003`
- Final status: `implemented`
- Outcome: `blocked`
- Verification claimed: `no`
- Plan SHA-256: `ec0df4b780dd6c9dd1fc6453dfd318feb16d917badd9c0e177db199e9fe1b8ee`
- Active task: `task-parity-stat003-scoreboard`

## Closure reason

The source-bound recheck implementation completed and was pushed at
`d7ecc5066babe15a37d181bd4b799c235985f8fa`. Its positive and fail-closed
regressions plus every mandatory software, firmware, package, privacy,
reference, and parity gate passed.

The sole authorized protected recheck then stopped nonzero and published
nothing. Its candidate was removed, wrapper outputs remained private, and the
attempt root was not modified. Metadata and redaction-safe boolean diagnosis
confirmed the expected capture failure, complete private inventory/modes,
capture source/reference, exact before/after source/reference, changed session,
ordinal +1, software-CPU restart, and disabled boot mining.

The first mismatch was the reconstructed app/package anchor. Attempt-005 did
not retain the original package manifest after the old workflow had admitted
it. A detached exact-source rebuild generated a different build timestamp; a
second build using the retained original timestamp still generated a different
app ELF hash because workspace/build paths affect that hash. Therefore the
original package-manifest byte digest cannot be reconstructed truthfully from
the retained files.

The old capture terminal boundary is still strong evidence: the original
workflow checked manifest source/reference/app identity against both pre- and
post-restart system snapshots before it reached the final restart-persistence
comparison. But the v1 public schema requires a package-manifest SHA-256 and
has no field for a retained capture-package identity digest. Filling the v1
field with a synthetic or relabeled digest would be false evidence, so the
recheck correctly remained closed.

## Next safe action

Create a fresh software-only plan and v2 scoreboard evidence contract that
preserves v1 capture validation while representing re-evaluation honestly:
bind a digest of the retained exact source/reference/app identity, the old
capture plan/closure and expected terminal boundary, the corrected evaluator
plan/source, protected input digest, and every campaign/restart/scoreboard
quorum. It must not claim the unavailable original manifest-byte digest.

Only that new contract may authorize another read-only protected evaluation.
No hardware rerun or attempt-006 is warranted.

## Non-claims

This closure does not verify or promote `STAT-003`, recover or claim the missing
manifest-byte digest, expose protected identity values, or authorize hardware,
network, mining, shares, restart, recovery, attempt-006, other boards/ASICs,
unbounded use, or release readiness.

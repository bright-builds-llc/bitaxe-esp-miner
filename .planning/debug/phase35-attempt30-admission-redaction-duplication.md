---
status: resolved
trigger: phase35-attempt30-admission-redaction-duplication
created: 2026-07-23T03:49:09Z
updated: 2026-07-23T04:02:55Z
---

# Phase 35 Attempt 30 Admission Redaction Duplication

## Failure signal

Attempt 30 completed the authoritative hardware and typed-root validation chain,
but `just verify-redaction` rejected the candidate generation before commit.
The new evidence directory contained protected-operational matches inherited
from the complete historical parity checklist.

## Root cause

The atomic Phase 35 publisher both updated the canonical checklist and copied
the entire projected checklist into the admitted generation. That duplicate was
unnecessary: the decision matrix already binds the before/after checklist
fingerprints, while the canonical checklist is the sole repository truth.

The staged scanner also scanned every line in a modified shareable file, so
unchanged legacy values in the canonical checklist and roadmap obscured the new
generation defect. Full-file scanning remains correct for renamed files and the
complete admitted-evidence tree, but ordinary local and CI diffs must inspect
added content.

## Repair and proof

Commit `d5224161` removes the generated checklist copy, preserves the manifest's
checklist digest and atomic canonical replacement, and scans added lines for
ordinary local and CI diffs. Regressions prove added sensitive values still
fail, unchanged legacy lines do not block safe edits, renames are rescanned in
full, and the admitted tree remains fully scanned.

Attempt 30 and its uncommitted candidate promotion remain non-promotable because
the exact source predates this repair. Fresh Attempt 31 requires the complete
software gate and exact-head preflight.

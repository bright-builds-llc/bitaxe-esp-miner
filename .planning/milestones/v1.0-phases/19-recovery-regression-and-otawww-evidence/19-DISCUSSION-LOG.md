# Phase 19: Recovery Regression And OTAWWW Evidence - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or
> execution agents. Decisions are captured in `19-CONTEXT.md`; this log
> preserves the alternatives considered.

**Date:** 2026-07-03T17:35:50Z
**Phase:** 19-recovery-regression-and-otawww-evidence
**Mode:** Yolo
**Areas discussed:** Recovery and fault-injection gates, recovery regression
evidence shape, OTAWWW/static update claim boundary, release docs/checklist and
redaction

## Recovery And Fault-Injection Gates

| Option | Description | Selected |
| --- | --- | --- |
| Phase-owned allow flags | Require explicit allow flags plus detector, board-info, package identity, restore path, safe-state markers, and redaction before failed-update, large-erase, or interrupted-update actions. | yes |
| Reuse prior evidence only | Cite Phase 16 and Phase 18 non-claims without adding a Phase 19 gate or artifact. | no |
| Raw manual commands | Run direct erase, rollback, or interrupted upload commands from the terminal. | no |

**Chosen recommendation:** Phase-owned allow flags.

**Rationale:** This matches `AGENTS.md` destructive/fault-injection limits and
the Phase 13, 16, and 18 evidence pattern. It lets the phase record useful
pending/blocker evidence when prerequisites are missing without running unsafe
actions.

## Recovery Regression Evidence Shape

| Option | Description | Selected |
| --- | --- | --- |
| Phase 19 wrapper around existing helper pattern | Reuse the Phase 16 recovery helper shape but write Phase 19 artifacts, wording, tests, and docs. | yes |
| New raw recovery script | Implement a separate command flow for erase, failed update, interrupted upload, and restore. | no |
| Documentation-only closure | Update docs and checklist without a current Phase 19 helper or evidence artifact. | no |

**Chosen recommendation:** Phase 19 wrapper around existing helper pattern.

**Rationale:** `scripts/phase16-recovery-regression.sh` already contains the
right safety mechanics: allow flags, detector reruns, board-info, current
manifest checks, factory restore, post-action markers, HTTP/static smoke, and
redaction. Phase 19 should specialize paths and conclusions rather than
duplicating risky low-level command orchestration.

## OTAWWW And Static Update Claim Boundary

| Option | Description | Selected |
| --- | --- | --- |
| Explicit REL-03 gap unless proven | Keep OTAWWW as a V1 parity gap unless whole-`www` update plus interrupted-update recovery evidence is captured. | yes |
| Promote from package/static serving | Treat `www.bin`, live static serving, or route presence as sufficient OTAWWW proof. | no |
| Implement full OTAWWW by default | Attempt whole-`www` partition update parity immediately. | no |

**Chosen recommendation:** Explicit REL-03 gap unless proven.

**Rationale:** Prior phases repeatedly decided that package generation,
`Wrong API input`, and live static serving do not prove whole-`www` update
parity. The safer Phase 19 default is to document owner, blocker, operator
impact, and follow-up unless the plan proves a bounded implementation and
recovery path.

## Release Docs, Checklist, And Redaction

| Option | Description | Selected |
| --- | --- | --- |
| Evidence-first docs/checklist updates | Update release docs, checklist, requirements traceability, and redaction review only after Phase 19 artifacts exist. | yes |
| Goal-first docs updates | Mark the roadmap gap closed from plan intent before evidence artifacts exist. | no |
| Skip redaction for blocked artifacts | Treat pending/blocker logs as safe without redaction review. | no |

**Chosen recommendation:** Evidence-first docs/checklist updates.

**Rationale:** The repository's release policy distinguishes implementation,
route presence, package artifacts, live evidence, blocked evidence, and
verified parity. Redaction review still matters for blocked logs because helper
output can include target, Wi-Fi, USB, NVS, or vendor log details.

## the agent's Discretion

- Exact helper names and evidence JSON fields.
- Exact plan split and whether to wrap Phase 16 helper or create a
  phase-specific script.
- Timeout values and fake-command fixtures for shell tests.
- Final checklist wording as long as unsupported claims are not promoted.

## Deferred Ideas

- Full OTAWWW whole-`www` update parity if Phase 19 cannot safely prove
  whole-partition write and interrupted-update recovery.
- Active safety telemetry belongs to Phase 20.
- Live mining and soak evidence belongs to Phase 21.

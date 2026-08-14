# Parity work closure

- Parity row: `API-009`
- Final status: `implemented`
- Outcome: `blocked`
- Verification claimed: `no`
- Plan SHA-256: `858b4d0626dcccd9a5f691b52ecac843b025f34876acc9f6d9f072db74bd5ffa`
- Active task: `task-parity-api009-command-effect-evidence-audit`

## Closure reason

The attempt-013 pre-active expiry is reproduced and fixed in software. The
previous resumable wall clock began at hardware preparation and could consume
the campaign before its first active state. The Production Mining Session now
owns a bounded activation clock followed by one resumable active epoch that
starts exactly once at first active, remains continuous across pause/resume,
and cannot be reset or replayed. Activation timeout has a distinct typed
blocker, marker, and host terminal category.

Rust capture now reserves activation, the 600-second epoch, and terminal
grace. The checked TypeScript transaction budget includes the same two phases
alongside every USB, retry, recovery, cleanup, and process-termination bound;
the fixture remains larger than its parent. The exact ignored harness changed
from deterministic RED to GREEN and was removed. Focused exact-boundary,
pause/resume, firmware mapping, host marker, cross-language, and real-process
tests pass, followed by every mandatory software/privacy/reference gate, the
real ESP firmware build, selector, task/plan integrity, sensitive-output, and
diff checks.

No hardware-capable command ran under this plan. API-009 remains
`implemented` because the complete five-command device-user quorum and safe
terminal evidence are still absent.

## Next safe action

Commit and push this software fix, then use a fresh clean synchronized
selector. A future attempt-014 is eligible for consideration only under its own
immutable bounded contract because this regression-backed fix objectively
changes the attempt-013 failure boundary.

## Non-claims

This closure does not verify or promote API-009, claim successful live command
effects or a confirmed live safe stop, authorize an unplanned device action,
infer physical IDENTIFY state, access protected attempt data, or expose device,
USB, network, credential, process, path, sensor, or raw-trace material.

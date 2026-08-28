# Formally decompose STR-005 verification

- Run ID: `20260828T175218Z-STR-005-DECOMPOSITION`
- Coordination task: `task-str005-verification-decomposition`
- Parity row: `STR-005`
- Initial status: `implemented`
- Baseline status: `implemented | unit,golden,workflow`
- Baseline source: `b8bbfe05339c2ad3315911a455f773816bb76951`
- Latest terminal evidence: `docs/parity/evidence/str005-noise-diagnostic/noise-diagnostic-projection-004.json`

## Objective

Replace the accumulated STR-005 continuation lineage with five smaller,
dependency-ordered verification tasks. This is a tracker and documentation
change only. It does not authorize firmware changes, hardware or network
effects, new evidence claims, or a parity transition.

## Superseded lineage

Add a supersession review to, then archive, these complete native records:

- `task-parity-str005-stratum-v2`
- `task-parity-str005-installed-package-recovery`
- `task-parity-str005-installed-package-recovery-002`
- `task-parity-str005-installed-package-recovery-003`
- `task-parity-str005-autonomous-continuation`
- `task-str005-exact-restoration-remediation`
- `task-str005-restore-and-verify-continuation`
- `task-str005-inactive-restoration-and-campaign-continuation`
- `task-str005-noise-handshake-diagnostic`
- `task-str005-preconnect-noise-and-verification`

Their plans, closures, projections, ordinals, and terminal decisions remain
immutable. Existing effect commands bound to those archived task IDs become
ineligible and are not rebound by this plan.

## Successor graph

1. `task-str005-tcp-payload-205` proves a fixed 64-byte non-secret payload from
   the exact Ultra 205 peer to the same-subnet fixture. It excludes Noise,
   protocol messages, ASIC, fan, voltage, and mining effects and starts a new
   task-local `diagnostic-001` namespace.
2. `task-str005-noise-auth-205` depends on accepted TCP payload evidence and
   re-proves TCP before act one, act two, authenticated Noise, and an encrypted
   diagnostic proof. It excludes channel/job handling and mining and starts
   its own `diagnostic-001` namespace.
3. `task-str005-v2-channel-job-205` depends on accepted Noise authentication
   and re-proves TCP and Noise before setup, channel, target, and job receipt.
   It excludes ASIC work and share submission and starts `session-001`.
4. `task-str005-bm1366-share-205` depends on accepted channel/job evidence and
   proves the complete cumulative chain on one exact package. It retains the
   global unconsumed campaign ordinal `attempt-008` and the existing 400 MHz,
   1100 mV, 100% fan, 180-second local-fixture safety contract.
5. `task-str005-evidence-promotion` depends on the accepted full-share campaign
   and performs evidence composition and STR-005-only promotion without
   hardware effects.

Only the TCP payload task is active after decomposition. The other four tasks
remain under `Future` with explicit dependencies. Every child requires its own
immutable execution plan before implementation or effects.

## Shared prerequisites

Recovery-006, exact settings restoration, detector admission, protected
evidence handling, redaction, USB cleanup, and earliest-failure preservation
remain reusable prerequisites. Diagnostic projections are prerequisite
evidence only; the final share campaign must re-prove the entire protocol and
hardware chain in one run on one exact package.

## Verification and completion

- Prove every superseded ID exists exactly once in `TASKS.archive.md` and no
  longer exists in `TASKS.md`.
- Prove every successor ID is unique across both task files, with only the TCP
  task active and the remaining dependency order exact.
- Prove the STR-005 checklist row and parity-progress totals are unchanged.
- Confirm no existing effect command is admitted by an archived authority and
  no new effect command was introduced.
- Run ordered Rust, Bright Builds, Bazel, parity, progress, reference,
  redaction, whitespace, and final-diff verification.
- Archive this coordination task, commit, and push the decomposition.

No hardware run, firmware change, projection, `RESULT.md`, checklist
transition, or hardware-regression claim is permitted by this plan.

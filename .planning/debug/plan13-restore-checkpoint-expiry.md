---
status: resolved
trigger: "Plan 13's restore checkpoint expired during an ordinary human response and left a lifecycle frame receiver orphaned."
created: 2026-07-12T02:00:00Z
updated: 2026-07-12T02:23:07Z
---

## Current Focus

hypothesis: Confirmed. The restore checkpoint conflated the human-action response budget with the electronic USB reappearance budget, and terminal cleanup skipped a still-live process group after discarding its persisted group-leader PID.
test: Deterministic state boundaries plus a real process group whose leader exits while a lifecycle frame receiver remains alive.
expecting: Satisfied. The restore checkpoint now exposes a full 300000 ms manual-action window, reappearance retains its independent 60000 ms observation bound, and terminal cleanup proves the complete process group dead.
next_action: Commit the software repair on a clean verified HEAD, then start an entirely fresh Plan 13 attempt; do not resume the tombstoned attempt or reuse its evidence.

## Symptoms

expected: Human power-action checkpoints have a bounded but practical response window; a valid response advances the lifecycle; expiry reaps the owner and every checkpoint receiver descendant before tombstoning.
actual: The restore checkpoint was created with only about 54 seconds remaining, delivery returned `resume_handle_stale`, and a `phase28.1.1-lifecycle-frame.pl receive` process survived with PPID 1.
errors: `phase28_attempt_error=resume_handle_stale`; terminal category `blocked_safe_evidence_invalid`; cleanup time category `blocked`.
reproduction: Run a fresh exact-HEAD attempt through `lifecycle_start`, consume removal, wait for the restore checkpoint, then deliver after its deadline; inspect the process group after tombstoning.
started: Observed on 2026-07-11 after the serial-session reuse repair; similar checkpoint-expiry friction occurred in earlier Plan 13 attempts.

## Ranked Hypotheses

1. **Conflated deadlines (high):** `plan13-lifecycle-restore` derives its 60000 ms deadline from removal attestation even though at least 5000 ms of mandatory USB absence occurs before the restore prompt.
2. **Leader-only liveness check (high):** cleanup returns early when the persisted owner PID is dead without checking whether its process group still contains the frame receiver.
3. **Receiver lacks independent cleanup ownership (medium):** the adapter waits for the socket child on the normal path, but its exit trap does not explicitly own that child if the surrounding owner exits during a deadline race.

## Evidence

- timestamp: 2026-07-11T00:00:00Z
  checked: `scripts/phase28.1.1-hardware-attempt-state.mjs`
  found: Restore checkpoint installation uses `attestation_accepted_ms + 60000`, while the restore checkpoint is installed only after a mandatory USB absence observation of at least 5000 ms.
  implication: The public restore prompt can never offer its nominal full timeout and starts with at most about 55 seconds remaining.
- timestamp: 2026-07-11T00:00:00Z
  checked: `scripts/phase28.1.1-exact-head-hardware-attempt.sh`
  found: Both `terminate_effect_process` and `cleanup_attempt_process` treat a dead group-leader PID as proof that cleanup is complete, without checking `kill -0 -- -PGID`.
  implication: A background receiver that remains in the owner's process group can be orphaned and then hidden when the attempt directory is replaced by a tombstone.

## Working Diagnosis

root_cause: The timing state modeled two independent concerns as one deadline: the restore response deadline was anchored to removal attestation, so the mandatory USB-absence observation consumed part of the operator's 60000 ms budget. Separately, cleanup checked only the group leader, and the runner deleted `child.pid` before terminal handling, so it lost the PGID needed to reap a receiver that outlived the leader.
confidence: high
smallest_correct_seam: Give restore attestation a full manual-action deadline from checkpoint creation, start the closed 60000 ms electronic reappearance observation after restore acceptance, retain the child PGID through terminal handling, and terminate the group even when its leader has exited.

## Fix and Verification

- timestamp: 2026-07-12T02:06:00Z
  checked: red state regression
  found: The restore checkpoint created at 6001 ms had deadline 61001 ms instead of the required full manual-action deadline 306001 ms.
  implication: The failure is deterministic and rooted in state authority rather than user timing or hardware behavior.
- timestamp: 2026-07-12T02:16:00Z
  checked: first real-process cleanup regression
  found: Group-aware liveness checks alone were insufficient because `run-validated-effect` removed `child.pid` before applying the terminal result; the receiver remained live after tombstoning.
  implication: Complete cleanup requires retaining the process-group locator until terminal handling finishes.
- timestamp: 2026-07-12T02:23:07Z
  checked: focused verification
  found: The hardware-attempt state test, accepted-state diagnostic test, and exact-head attempt suite pass; the exact-head suite includes all 84 invalid cases and a real orphan-receiver process boundary. Bash syntax, `shfmt -d`, warning-level `shellcheck`, and `git diff --check` also pass.
  implication: Both root causes are regression-guarded without touching the connected device.

## Resolution

root_cause: Restore timing combined operator-response and electronic-reappearance budgets, while terminal cleanup combined leader death with group death and discarded the saved PGID too early.
confidence: high
fix: Restore checkpoints now receive 300000 ms from checkpoint creation. Restore acceptance independently starts the 60000 ms USB reappearance observation. The lifecycle receiver uses an exact `exec`-owned PID, the runner retains `child.pid` through terminal handling, and cleanup checks and terminates the whole process group even if its leader is already dead.
hardware_status: not exercised; the previous attempt remains tombstoned and its package/evidence must not be reused. A new Plan 13 run requires a clean exact HEAD containing this repair and the normal detector gate.

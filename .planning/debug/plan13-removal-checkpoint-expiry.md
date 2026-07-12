---
status: resolved
trigger: "Plan 13's removal checkpoint expired after five minutes while the user completed the requested both-power disconnection and replied through the app."
created: 2026-07-12T02:56:49Z
updated: 2026-07-12T02:56:49Z
---

## Current Focus

hypothesis: Confirmed. The broker's five-minute physical-action policy is shorter than the observed asynchronous human/app round trip, and its 20-minute lifecycle lease cannot enclose two practical manual waits plus USB observation, a 360-second capture, and cleanup.
test: Deterministic state-machine and real broker/process-boundary fixtures at one millisecond before each deadline, exactly at each deadline, and at the derived lifecycle lease boundary.
expecting: Satisfied. Every physical-action checkpoint now has a bounded 1800000 ms window, electronic USB reappearance remains independently bounded to 60000 ms, capture remains at least 360000 ms, and the enclosing lease has explicit cleanup headroom.
next_action: Commit the repair on a clean exact HEAD, then begin an entirely fresh detector-gated Plan 13 attempt. Do not resume or reuse the expired attempt, package, or evidence.

## Symptoms

expected: Both physical-action checkpoints remain bounded while allowing a realistic asynchronous user response; the electronic USB reappearance observation remains separately bounded to 60 seconds; the lifecycle lease encloses every legitimate wait, capture, and cleanup step.
actual: The removal checkpoint expired after 300000 ms, delivery returned a stale resume handle, and the already-completed exact-head reflash/reinit attempt was terminalized as invalid evidence.
errors: `phase28_attempt_error=resume_handle_stale`; adapter `accepted_state_diagnostic_error: lifecycle checkpoint expired`; terminal `blocked_safe_evidence_invalid`.
reproduction: Start a fresh exact-head attempt through `lifecycle_start`, wait more than 300000 ms after the removal checkpoint is created, then deliver `plan13-both-power-paths-removed`.
started: Observed immediately after the restore-deadline anchoring and process-group cleanup repair on exact HEAD `75328485e8cfaeabd28a47008011d70745bf77d7`.

## Ranked Hypotheses

1. **Manual-action budget remains too short (high):** the lifecycle removal and restore definitions still use 300000 ms, which is shorter than the demonstrated Codex app plus physical-action round trip.
2. **Lifecycle lease is inconsistent with subordinate bounds (high):** the broker hard-codes 1200000 ms although two five-minute manual waits plus a 360-second capture already consume 960000 ms before USB observation and cleanup.
3. **Electronic USB reappearance is the limiting step (low):** the 60000 ms observation begins only after restore acceptance and did not cause this failure.
4. **Hardware or cleanup state caused the stale handle (eliminated):** terminal cleanup left zero active slots, attempt directories, locks, and surviving processes; both device power paths were physically removed as requested.

## Evidence

- timestamp: 2026-07-12T02:44:00Z
  checked: `scripts/phase28.1.1-hardware-attempt-state.mjs`
  found: Every recovery/manual lifecycle checkpoint except connected entry used `timeoutMs: 300_000`; lifecycle removal and restore therefore expired five minutes after their own creation.
  implication: The observed expiry is the intended old policy, not a clock-anchor regression.
- timestamp: 2026-07-12T02:44:00Z
  checked: `scripts/phase28.1.1-exact-head-hardware-attempt.sh`
  found: Owner attachment independently hard-coded `attach_now + 1200000` for the enclosing lifecycle lease.
  implication: Increasing subordinate manual deadlines without deriving the enclosing lease would create a second, earlier expiry boundary.
- timestamp: 2026-07-12T02:47:00Z
  checked: red state regression
  found: The removal checkpoint created at 1000 ms still ended at 301000 ms instead of 1801000 ms.
  implication: A deterministic non-hardware regression reproduces the exact policy defect.

## Working Diagnosis

root_cause: Plan 13 encoded an optimistic five-minute human/app response assumption in every physical checkpoint and encoded an unrelated 20-minute lifecycle lease literal in the broker. Real asynchronous delivery exceeded the manual checkpoint bound, and the enclosing lease was not derived from subordinate timing guarantees.
confidence: high
smallest_correct_seam: Centralize the Plan 13 physical, USB, capture, and headroom constants in the state authority; have checkpoint definitions and broker owner attachment consume that policy; reject inconsistent lease attachment.

## Empirical Override

The protected Plan 13 artifacts are not rewritten. This debug repair records the empirical operational override:

- each physical-action checkpoint: 1800000 ms
- continuous USB absence minimum: 5000 ms
- electronic USB reappearance maximum: 60000 ms
- retained cold-start capture minimum: 360000 ms
- lifecycle cleanup/orchestration headroom: 120000 ms
- derived enclosing lifecycle lease: `2 * 1800000 + 5000 + 60000 + 360000 + 120000 = 4145000 ms`

## Fix and Verification

- timestamp: 2026-07-12T02:56:49Z
  checked: state authority and broker implementation
  found: Physical-action checkpoints consume one 1800000 ms constant; the broker reads the derived 4145000 ms lease from the same authority; owner attachment rejects shorter or longer leases. The accepted-state adapter reads the same USB absence, USB reappearance, and capture constants.
  implication: Manual and electronic waits remain separate, while one authoritative budget encloses their worst legitimate sequence with two minutes of explicit headroom.
- timestamp: 2026-07-12T02:56:49Z
  checked: focused state regression
  found: Removal and restore accept at deadline minus one millisecond, reject exactly at the deadline, keep restore inside the lease, and reject a mismatched lease.
  implication: Exact inclusive/exclusive deadline semantics and lease consistency are regression guarded without hardware access.
- timestamp: 2026-07-12T03:00:00Z
  checked: focused broker and adapter verification
  found: The exact-head broker suite passed all 84 invalid cases, including emitted 1800000 ms manual checkpoint bounds and the 4145000 ms derived lease. The accepted-state adapter suite passed with the shared 5000 ms USB absence, 60000 ms electronic reappearance, and 360000 ms capture floor. State tests, Bash syntax, `shfmt -d`, and `git diff --check` also passed.
  implication: State authority, broker ownership, and the hardware adapter agree on one bounded lifecycle policy; no hardware command was invoked.

## Resolution

root_cause: The remaining Plan 13 policy allowed only five minutes for a physical-action response and used a lifecycle lease literal that was not derived from its subordinate waits.
confidence: high
fix: Use bounded 30-minute windows for physical checkpoints and a 4145000 ms derived lifecycle lease covering both manual windows, USB absence, USB reappearance, the 360-second capture, and 120 seconds of cleanup/orchestration headroom.
hardware_status: not exercised; both Ultra 205 power paths remained removed throughout this software-only investigation.

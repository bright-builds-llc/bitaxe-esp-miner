# Parity work log

## 2026-08-03T23:50:00Z | hardware attempt 001 stopped

- Source commit: `c64e1ab3`
- Actions: built the exact package, passed private detector admission, flashed
  the Ultra 205, captured a safe initial boot, changed the hostname to a
  non-secret test value, confirmed immediate readback, and issued one normal
  restart.
- Verification: the post-restart monitor was launched after USB
  re-enumeration and produced no artifact. The public projection was withheld.
  Recovery restored the original hostname and private readback confirmed it;
  no recovery flash ran.
- Evidence: private mode-0700 artifacts under
  `scratch/v12-hostname-typed/attempt-001`; no private value or raw trace is
  promoted.
- Outcome: `stop_retryable_observation_boundary`.
- Blocker or next safe action: implement and verify a pre-acquired passive
  monitor, push it cleanly, then use the one final retry recorded by the task.

## 2026-08-04T00:09:01Z | hardware attempt 002 terminal blocker

- Source commit: `ca3eeb1c8af67ffe2dc5d3c698a62dc2825ad3ac`
- Actions: rebuilt the exact clean package, passed fresh private detector
  admission, flashed the Ultra 205, captured the safe initial state, changed
  and immediately read back the test hostname, pre-acquired the passive
  monitor, and issued one normal restart.
- Verification: the typed command exhausted the bounded post-restart capture
  and returned `process_failed` without a post-restart artifact. The public
  projection remained absent. Recovery artifacts are present, a value-free
  private comparison proves the restored hostname equals the original, and no
  recovery flash ran.
- Evidence: private mode-0700 artifacts under
  `scratch/v12-hostname-typed/attempt-002`; no private hostname, origin,
  network identifier, USB path, credential, or raw trace is promoted.
- Outcome: `stop_hardware_blocker`; `V12-HOSTNAME-205` remains `implemented`
  with no checklist transition or progress synchronization.
- Blocker or next safe action: none within this task contract. The final retry
  is consumed; a future attempt would require a new targeted diagnosis, a
  distinct regression-backed correction, and a new bounded task contract.

## 2026-08-04T00:30:00Z | missing-artifact root cause reproduced

- Source commit: `7aeffbca`
- Actions: compared the settings workflow with the production monitor command,
  reconstructed both private attempt timelines without exposing protected
  values, and ran a fast synthetic call-site reproduction whose monitor returns
  serial stdout without inventing an evidence file.
- Verification: the reproduction deterministically reported
  `exact_missing_artifact_failure=true`,
  `production_shaped_monitor_writes_no_artifact=true`, and
  `recovery_restored_original=true`. Source inspection confirms routine monitor
  ignores `--evidence-dir`, while both hardware attempts reached recovery after
  one complete capture window with empty post-restart directories.
- Evidence: source contracts, protected artifact presence/timing metadata, and
  the red-capable synthetic reproduction; no private content was printed or
  promoted.
- Outcome: host orchestration defect confirmed. The hardware attempts do not
  prove a firmware hostname-persistence failure.
- Blocker or next safe action: replace the duplicated monitor/restart sequence
  with the repository's typed device-session reboot transaction, prove the real
  process/file boundary, then use the one new-information attempt recorded in
  `task-parity-v12-hostname-device-session-retry`.

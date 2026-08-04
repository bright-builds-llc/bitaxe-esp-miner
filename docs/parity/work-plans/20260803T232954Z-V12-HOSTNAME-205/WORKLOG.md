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

## 2026-08-04T04:28:09Z | typed reboot correction passes software gates

- Source commit: `647f9e26f35a` plus the pending implementation diff.
- Actions: added the private `esp-device-session-reboot-intent-v1` contract and
  `device-session reboot-live`; moved USB identity derivation inside that live
  process; replaced the settings workflow's standalone monitor, fixed delay,
  duplicate restart request, and invented log dependency with the closed typed
  reboot projection; removed the unused long-running process API; and enrolled
  v2 settings evidence in semantic redaction checks.
- Verification: focused Rust and TypeScript tests pass, including a real child
  process that emits production-shaped monitor behavior without creating a
  monitor artifact. The complete mandatory Rust sequence, Bright Builds check,
  `just test`, `just parity`, `just parity-progress`, `just verify-redaction`,
  `just verify-reference`, and diff check pass. Projection admission now
  requires the exact public field set and rejects unexpected identifying fields.
- Evidence: software tests and source contracts only; no public hostname,
  origin, USB identity, network identifier, port, credential, or raw trace was
  produced.
- Outcome: software correction ready to commit and push. `V12-HOSTNAME-205`
  remains `implemented` and no final evidence exists yet.
- Blocker or next safe action: commit and push the clean implementation, then
  run exactly one detector-gated `attempt-003` under the active task contract.

## 2026-08-04T04:37:00Z | hardware attempt 003 passed

- Source and package commit: `cb0fe1f78ad8dd82ec815069739572053fa54c22`.
- Actions: built the exact pushed package, passed fresh private detector
  admission for one Ultra 205, flashed and observed the safe boot, changed and
  immediately read back the test hostname, ran one typed device-session normal
  restart, and restored and confirmed the original private hostname.
- Verification: the closed session reported `ready`, same physical device,
  reader armed with pre-restart delivery, exactly one complete restart request,
  exact build recovery, changed boot session, ordinal `N+1`, software reset,
  expected hostname digest, correlated serial delivery, and complete cleanup.
  The public v2 projection passed exact-field admission and redaction.
- Evidence: committed projection
  `docs/parity/evidence/v12-hostname-205/durability-projection.json` with SHA-256
  `9325d9f02102e8d0fd4f8b0cb887fde4af924ae417e19bae5e0ac6c9bd3c29c5`;
  protected operational evidence remains under
  `scratch/v12-hostname-typed/attempt-003` and its detector sibling.
- Outcome: `passed`; no recovery flash ran and the single authorized hardware
  attempt is consumed.
- Blocker or next safe action: bind `RESULT.md`, transition only
  `V12-HOSTNAME-205` to `verified`, synchronize progress, and archive the
  completed active task.

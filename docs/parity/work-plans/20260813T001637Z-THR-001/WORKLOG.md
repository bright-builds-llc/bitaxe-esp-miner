# Parity work log

## 2026-08-13T00:16:37Z | Selection and root-cause checkpoint

- Source commit: `6bd4cacc85ea0d68da3b435732aa0bcc4da87c0a`
- Actions: Re-ran the clean synchronized selector; skipped API-009 at its
  sealed repeated-boundary stop; selected THR-001; audited its checklist and
  historical evidence, the pinned board-205 thermal path, the current pure and
  firmware thermal owners, and the accepted API-002 private snapshots.
- Verification: The protected API-002 capture remains mode `0700` with six
  mode-`0600` files and contains coherent fresh safe HTTP/WebSocket chip-
  temperature observations. All twelve current production thermal acquisition
  and API paths inspected are byte-compatible with that capture source except
  unrelated later changes in the aggregate runtime-snapshot owner. Source
  comparison then identified the actual semantic gap: upstream board 205 adds
  its configured `+5 C` EMC2101 internal-temperature offset, while the Rust
  Ultra 205 adapter does not.
- Evidence: Planning and protected-input classification only. No raw
  temperature, acquisition stamp, boot session, origin, hostname, port,
  network or USB identity, credential, log, or trace was copied into a public
  artifact or terminal output.
- Outcome: THR-001 is actionable through a production offset fix followed by
  one fresh exact-package read-only hardware capture. Reusing the pre-fix
  temperature as final evidence would be invalid.
- Blocker or next safe action: Freeze, verify, commit, and push the immutable
  plan/task checkpoint before editing implementation files.

## 2026-08-13T00:24:00Z | Immutable-plan verification

- Source commit: `6bd4cacc85ea0d68da3b435732aa0bcc4da87c0a`
- Actions: Froze the THR-001 plan at SHA-256
  `1bfc569edbfe9e307128fad2c0eb2a54c8d8925f6b412e30ed201ca6052b2860`
  and retained exactly one matching active task. No implementation, protected
  attempt, package, evidence projection, checklist field, or progress file
  changed.
- Verification: The ordered Cargo format, strict Clippy, all-target build, and
  all-feature test gates passed; Bright Builds reported zero findings; all 40
  Bazel test targets passed; parity reported no validation errors; progress
  remained 64 of 94 active rows verified (68.1%); repository redaction checked
  17 public artifacts; pinned-reference cleanliness, task uniqueness, plan
  digest, and diff checks passed. One orphaned parity-report client was found
  waiting on a cleared Bazel server command after the test run; terminating
  only that owned client and restarting the Bazel server restored the exact
  report boundary, which then passed without repository changes.
- Evidence: Immutable plan, active task, and this worklog only. No private
  source value, device, credential, network, serial, or hardware input was
  touched by verification.
- Outcome: The plan/task checkpoint is ready for its required commit and push.
- Blocker or next safe action: Commit and push this checkpoint without amend,
  then implement the typed thermal fix and evidence path.

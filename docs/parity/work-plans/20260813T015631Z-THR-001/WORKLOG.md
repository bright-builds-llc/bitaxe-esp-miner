# Parity work log

## 2026-08-13T01:56:31Z | Selection and lossless-validation checkpoint

- Source commit: `f5190e234d954356c4fd3b310a85600840128d31`
- Actions: Re-ran clean synchronized selection, skipped API-009 at its sealed
  repeated boundary, and selected THR-001. Bound this plan to the distinct
  attempt-002 wider-integer diagnosis and chose a Rust private-input validator
  so acquisition stamps remain exact `u64` values.
- Verification: Attempt-002's protected aggregate diagnosis proves every
  preceding source/device/safety/privacy member passed and its safe terminal
  summary uniquely identifies JavaScript safe-integer rejection. Rust
  `serde_json` already supports exact nonnegative `u64` deserialization without
  a new dependency.
- Evidence: Planning and software diagnosis only. Attempts 001/002 remain
  sealed; no private value, device effect, credential, origin, network/USB
  identity, temperature, stamp, boot session, log, command, PID, or trace was
  published.
- Outcome: THR-001 is actionable through one narrow Rust validation boundary,
  host integration, and at most one fresh attempt-003.
- Blocker or next safe action: Freeze, verify, commit, and push this immutable
  plan/task checkpoint before implementation edits.

## 2026-08-13T02:10:00Z | Lossless private-input boundary implemented

- Source commit: `8e04f82d8893efbf5c3d37af7b0c30a843b44e70`
- Actions: Added a private Rust thermal-input validator that deserializes every
  acquisition-stamp member as an exact `u64`, validates fresh equal safe HTTP
  and WebSocket observations without emitting values, and made the TypeScript
  orchestration shell invoke it before constructing closed public evidence.
  Advanced the evidence ordinal and protected attempt paths to attempt-003.
- Verification: Focused Rust tests pass for `u64::MAX`, wider-than-JavaScript-
  safe values, unequal stamps, malformed integer encodings, stale state,
  unsafe temperature, and invalid WebSocket envelopes. The production-shaped
  host suite passes all 295 tests, including validator rejection, timeout,
  launch failure, and a real child-process boundary. The relevant Bazel
  contract and validator targets build successfully.
- Evidence: Software and protected-boundary tests only. No device effect or
  hardware attempt occurred, and no private value, credential, origin,
  network/USB identity, temperature, stamp, boot session, log, command, PID,
  private path, or trace was published.
- Outcome: The attempt-002 host blocker is removed at its exact numeric
  boundary; the full mandatory software gate remains before commit/package.
- Blocker or next safe action: Run the immutable plan's mandatory sequence,
  review the complete diff, then commit and push the software checkpoint.

## 2026-08-13T02:14:00Z | Mandatory software gates passed

- Source commit: `8e04f82d8893efbf5c3d37af7b0c30a843b44e70`
- Actions: Completed the required formatting, lint, build, unit/integration,
  Bright Builds, Bazel, parity, progress, redaction, reference-cleanliness,
  immutable-plan, unique-task, generated-contract, candidate-absence, and diff
  checks. Reviewed the implementation for a simpler boundary; the narrow Rust
  validator removes the duplicated lossy TypeScript numeric/stamp logic.
- Verification: Cargo format, clippy with warnings denied, all-target build,
  and all-feature tests pass. Bright Builds reports zero findings. `just test`
  passes all 41 test targets, both parity commands report no validation errors,
  redaction checks 17 surfaces, the reference is clean, the immutable plan hash
  and unique active task are exact, generated contracts agree, and every
  attempt-003 output path remains absent.
- Evidence: Software verification only. No detector or capture command ran and
  no device effect, private input, raw observation, credential, identity,
  endpoint, path, log, command, PID, or trace was published.
- Outcome: The complete software checkpoint is ready for its separate commit
  and push; hardware remains ineligible until that synchronization completes.
- Blocker or next safe action: Re-run the required pre-commit Rust sequence,
  commit and push the implementation, then build and admit the exact package.

## 2026-08-13T02:29:00Z | Exact-package attempt-003 passed

- Source commit: `021c061b26494a665e35b1e3068ec5b6a2775261`
- Actions: Built and independently admitted the exact clean board-205 package,
  ran the sole detector command, and then consumed the one authorized
  attempt-003 through the repo-owned thermal evidence transaction.
- Verification: The detector admitted exactly one Ultra 205. The capture
  completed with category `complete`; the independent Rust final-evidence
  validator accepted the closed projection. It binds attempt ordinal 3, the
  exact source/reference/package, the current production read-only EMC2101
  source at address `0x4c` register `0x00` with the board `+5 C` offset, one
  finite plausible fresh below-throttle sample, exact lossless HTTP/WebSocket
  value/state/stamp/boot/package correlation, stable boot, disabled mining and
  hardware control, complete cleanup, private modes, and passed redaction.
  Recovery was not used; all protected files are mode `0600` below mode `0700`
  roots and no process holds the attempt root.
- Evidence: The only public artifact is
  `docs/parity/evidence/thr001-emc2101-thermal/thermal-projection.json`.
  Raw temperature, acquisition stamps, boot session, origins, ports, USB and
  network identities, credentials, HTTP bodies, logs, commands, PIDs, paths,
  and traces remain only in ignored protected roots.
- Outcome: The immutable plan's read-only hardware-smoke quorum passed without
  a recovery flash; authoritative checklist promotion remained subject to the
  final parity validator.
- Blocker or next safe action: Commit the evidence without changing the
  checklist, then request the row transition and run the mandatory final
  verifier before treating promotion as accepted.

## 2026-08-13T02:36:00Z | Promotion rejected by authoritative policy

- Source commit: `5c5688fae761cbda27a1c73dd9cbd059b4b18480`
- Actions: Requested a THR-001 `verified` transition using the plan-authorized
  `hardware-smoke` evidence, then ran the mandatory final parity verifier. It
  rejected the promoted row because active safety-control verification requires
  `hardware-regression`; it also identified missing code-span formatting in the
  requested target cell. The transition receipt and all derived progress/task
  archival changes were still uncommitted, so they were removed and the
  authoritative implemented state was preserved.
- Verification: `just parity` produced the exact policy failure before any
  finalization commit or push. A corrective automatic demotion was attempted
  and safely refused by the monotonic transition policy, confirming that the
  invalid uncommitted receipt had to be discarded rather than extended.
- Evidence: The valid read-only projection remains committed. No raw or
  protected device material was promoted, and no additional hardware effect
  occurred after attempt-003.
- Outcome: `stop_impossible_contract`. THR-001 remains `implemented`; this
  immutable plan prohibited the fault stimulus required for hardware-regression
  and therefore cannot satisfy the authoritative verification policy.
- Blocker or next safe action: A fresh plan must define bounded safe thermal
  fault/overheat stimulus, expected response, restoration, retry limits,
  privacy, and hardware-regression evidence. Attempt-003 must not be retried.

## 2026-08-13T02:48:00Z | Terminal-closure selection repaired

- Source commit: `087235d56acc0156b879044629a9e8d93f011601`
- Actions: Corrected parity selection to retire lineages only when every plan
  in that lineage has a valid terminal closure. Added a focused regression
  with two valid same-row closures that intentionally have no immutable plan
  links, while preserving the existing rejection for two unclosed unlinked
  plans.
- Verification: The focused Rust regression passes. The real
  `next-item --format json` command now reports `maybe_open_plan: null` and a
  normal candidate list containing THR-001 at `implemented`, proving all three
  THR-001 closures are consumable without editing any immutable `PLAN.md`.
- Evidence: Host selector behavior only. No device access, hardware effect,
  protected artifact, credential, or private observation was involved.
- Outcome: The plan closure is now recognized deterministically; the separate
  hardware-regression blocker remains unchanged.
- Blocker or next safe action: Run the mandatory sequence, commit and push the
  selector repair, then stop this invocation at the recorded terminal blocker.

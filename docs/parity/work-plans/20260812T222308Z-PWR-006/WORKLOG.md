# Parity work log

## 2026-08-12T22:23:08Z | Selection and immutable-plan checkpoint

- Source commit: `e3f96fadaad2a4865bb019a25ae617fe930ab869`
- Actions: Re-ran the clean selector; retained API-009 at its fresh operator-
  readiness gate; selected PWR-006; and audited the stale checklist note,
  pinned INA260 reference, current sensor/API ownership, the accepted API-002
  private snapshots and public projection, and source compatibility.
- Verification: Clean synchronized `main`; no open plan; reference commit
  `c1915b0a63bfabebdb95a515cedfee05146c1d50`; protected attempt root mode 0700
  and files mode 0600; public source projection SHA-256
  `6ec58fdaeb7cbad3cf103832cd3e59fe470fcb05f6f6a4d41e218ffd6378991a`;
  identical fresh HTTP/WebSocket INA260 values, states, and acquisition stamps;
  safe-range predicates passed; relevant current production paths are byte-
  identical to API-002 source commit `524b445ee45c986a1366cfe64d2cbcbe41178da8`.
- Evidence: Planning and private-input audit only. No raw value, acquisition
  stamp, boot session, origin, port, network identity, credential, retained
  log, or trace is copied into public artifacts.
- Outcome: PWR-006 is actionable through a software-only typed projection of
  already accepted live evidence. A new hardware attempt would add risk without
  strengthening the closed claim.
- Blocker or next safe action: Run every immutable-plan gate, commit and push
  the plan/task checkpoint, then implement the typed projector without device
  interaction.

## 2026-08-12T22:27:00Z | Immutable-plan verification

- Source commit: `e3f96fadaad2a4865bb019a25ae617fe930ab869`
- Actions: Froze the PWR-006 plan at SHA-256
  `e58742236746a59fb68afd92a5fe92b181a71e967e43d323789b9f22a58db818`
  and retained exactly one matching active task. No implementation, source
  projection, private attempt artifact, or checklist field changed.
- Verification: The ordered Cargo format, strict Clippy, all-target build, and
  all-feature test gates passed; Bright Builds reported zero findings; all 40
  Bazel tests passed; parity reported no validation errors; progress remained
  63 of 94 active rows verified (67.0%); repository redaction across 16 public
  artifacts, pinned-reference cleanliness, task uniqueness, plan digest, and
  diff checks passed.
- Evidence: Immutable plan, active task, and this worklog only. No hardware or
  private source artifact was touched by the verification commands.
- Outcome: The plan/task checkpoint is ready to commit and push before any
  implementation begins.
- Blocker or next safe action: Commit and push this checkpoint, then add the
  typed INA260 evidence contract and projector.

## 2026-08-12T23:01:00Z | Typed projection implementation

- Source commit: `d0bf24b1245606e01a35ee6810d5806d612b11f1`
- Actions: Added the Rust-owned `bitaxe-ina260-evidence-v1` contract,
  independent validator, generated TypeScript surface, closed CLI invocation,
  redaction admission, and a software-only projector over the protected
  API-002 attempt. The projector binds exact source, reference, plan, task,
  path, mode, and digest facts; independently validates the admitted source;
  checks current production-source compatibility; and publishes only after
  typed HTTP/WebSocket INA260 correlation and candidate validation.
- Verification: Contract tests passed. Eight focused projector tests passed,
  including stale and uncorrelated samples, source and dirty-path drift,
  validator rejection, launch failure, atomic withholding, sensitive-output
  exclusion, large numeric boot-session stamps, order-independent typed stamp
  correlation, and a real `/bin/sh` child-process/file boundary. The source and
  final validators accepted their artifacts. The final mode-0644 projection
  SHA-256 is
  `c9624b3c77e4021137a375de2a70c2bf7425bc947af6ba59c4e42fbceb25634d`;
  no candidate remains.
- Evidence: `docs/parity/evidence/pwr006-ina260/ina260-projection.json` records
  only closed schema, board, commit, digest, fixed register/address, count,
  state, safety, cleanup, no-rerun, and redaction facts. Raw telemetry,
  acquisition stamps, boot sessions, origins, ports, network identifiers,
  credentials, retained logs, and traces remain private.
- Outcome: The first sealed run safely withheld publication after exposing an
  order-sensitive JSON-object comparison. Component-wise typed correlation
  fixed the root cause; a second withheld run showed that the firmware's u64
  boot-session counter exceeds JavaScript's safe-integer range. The final
  implementation validates nonnegative integral stamp components while the
  exact boot-session hash, sequence, timestamp, values, states, package, and
  revision bindings close correlation. The production projection then passed.
- Host note: A fresh macOS policy scan held newly written temporary executable
  scripts and Bazel binaries in the dynamic loader, causing a diagnostic full
  automation run to time out across many unrelated real-child tests. Direct
  `/bin/sh` execution proved the scripts themselves; the hold later cleared
  for repository binaries. No device interaction occurred.
- Blocker or next safe action: Run every mandatory implementation gate and the
  final privacy/reference/diff checks, then commit and push this evidence
  implementation before transitioning only PWR-006.

## 2026-08-12T23:19:52Z | Evidence implementation verification

- Source commit: `d0bf24b1245606e01a35ee6810d5806d612b11f1`
- Actions: Completed the explicit simplification and full-diff review. Kept one
  narrow projector and reused the existing process, workspace, workflow,
  source-evidence, redaction, and typed-result boundaries rather than adding a
  second capture or hardware path. Compacted the identical generated contract
  files by three physical lines after the first Bright Builds pass identified
  the file-length budget; no contract shape changed.
- Verification: The final ordered sequence passed: `cargo fmt --all`; strict
  all-target/all-feature Clippy; all-target/all-feature build; all-feature
  tests; Bright Builds with zero findings; `just test` with all 41 Bazel test
  targets passing; `just parity` with no validation errors; and
  `just parity-progress` at 63 of 94 active rows verified before transition.
  Repository redaction accepted all 17 public artifact roots; reference
  cleanliness, generated-contract identity, both independent evidence
  validators, immutable plan digest, unique task binding, candidate absence,
  and `git diff --check` passed.
- Evidence: No private value was copied into a tracked file or command output.
  The typed projection remains mode 0644 at SHA-256
  `c9624b3c77e4021137a375de2a70c2bf7425bc947af6ba59c4e42fbceb25634d`.
- Outcome: The evidence implementation is clean and ready to commit and push.
- Blocker or next safe action: Commit and push this checkpoint, save its exact
  source commit, then create the typed transition for PWR-006 only.

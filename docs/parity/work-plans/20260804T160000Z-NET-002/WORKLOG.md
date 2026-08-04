# Parity work log

## 2026-08-04T16:00:00Z | selection and plan

- Source commit: `bb007f5f82c86437974616dea5701bfd2973a095`.
- Actions: Continued deterministic selection, retained the audited boundaries
  on earlier implemented and in-progress candidates, and traced the pinned
  APSTA, SoftAP, DNS, and station-success behavior into the current station-only
  ESP-IDF adapter and HTTP access gate.
- Verification: Confirmed a clean synchronized branch, no open parity plan,
  native ESP-IDF mixed/AP configuration and AP-netif support, HTTP startup after
  Wi-Fi admission, and no existing captive DNS owner.
- Evidence: Pinned reference breadcrumbs, current Wi-Fi/startup/access sources,
  immutable plan, and active task record.
- Outcome: `NET-002` is bounded to software provisioning behavior and will
  remain below verified without a real provisioning client session.
- Blocker or next safe action: Commit the plan and task after mandatory gates,
  then implement the pure DNS core and thin firmware owner.

## 2026-08-04T16:18:00Z | implementation and focused verification

- Source commit: implementation based on planning commit `16d45da8`.
- Actions: Added strict bounded DNS response planning, upstream-shaped AP SSID
  derivation, AP-only and mixed AP+STA startup, failed-station provisioning
  retention, successful-station AP shutdown, and a single category-only UDP/53
  owner. Split the UDP shell from the Wi-Fi state machine during the explicit
  simplification pass.
- Verification: Eight focused DNS/SSID tests, four source-ownership tests, the
  `bitaxe-api` Bazel suite, and the real ESP32-S3 firmware build pass. The first
  cross-compile exposed a target-only owned-string conversion; converting the
  derived SSID through its bounded string slice fixed the root type mismatch,
  and the rebuilt firmware passed.
- Evidence: Pure malformed, non-A, multi-question, additional-record, opcode,
  label, question-count, and response-size boundaries; source checks for exact
  AP shape, one DNS owner, state ordering, and identifier-free DNS logs; real
  firmware ELF.
- Outcome: The bounded implementation is ready for the mandatory repository
  gate. Live provisioning-client behavior remains deliberately unclaimed.
- Blocker or next safe action: Run the ordered Rust and repository-wide gates,
  review the final diff, commit the implementation, and bind a truthful result.

## 2026-08-04T16:31:00Z | mandatory implementation gate

- Source commit: implementation based on planning commit `16d45da8`.
- Actions: Completed the edge and simplification review, rejected reserved or
  overlong DNS label encodings, kept the response core allocation-bounded, and
  reviewed the AP/mixed/connected state ordering plus public-output surfaces.
- Verification: The ordered format, strict Clippy, all-target build, and
  all-feature test sequence passed; Bright Builds reported zero findings; all
  30 Bazel targets, parity/progress, redaction, reference cleanliness, focused
  identifier scans, and diff checks passed. The final real firmware build also
  passed after the module split.
- Evidence: Eight pure DNS/SSID tests, four source-owner tests, the ESP32-S3
  firmware ELF, category-only DNS diagnostics, and an unchanged clean reference.
- Outcome: The implementation tree satisfies every software acceptance
  criterion for `implemented`, while supplying no live-client evidence.
- Blocker or next safe action: Rerun the pre-commit sequence after this durable
  checkpoint, commit the implementation, create `RESULT.md`, and transition
  only `NET-002` if the committed source remains identical to the passed tree.

## 2026-08-04T16:35:00Z | implementation committed

- Source commit: `126cf24b7205f3bdac0a0a70fc022a422973c392`.
- Actions: Committed the pure DNS contract, ESP-IDF configuration-network
  state machine, UDP owner, tests, build wiring, and durable task checkpoints
  without changing the parity checklist.
- Verification: The committed source is the same implementation tree that
  passed every focused, cross-compile, mandatory, parity, privacy, reference,
  and diff gate recorded above.
- Evidence: `RESULT.md` binds the exact implementation commit, reference,
  acceptance tests, conclusion, and live-client non-claims.
- Outcome: `NET-002` is eligible only for `implemented` with `unit,workflow`
  evidence; no hardware or live network evidence exists.
- Blocker or next safe action: Apply the typed transition, synchronize progress,
  finalize the active task record, run final metadata gates, commit, and push.

## 2026-08-04T16:35:00Z | implemented transition

- Source commit: `126cf24b7205f3bdac0a0a70fc022a422973c392`.
- Actions: Applied typed transition `20260804T163500Z-NET-002`, synchronized the
  hash-chained progress record, and finalized the active task's bounded result.
  The first uncommitted projection omitted Markdown code spans around the new
  Rust-owned targets; `just parity` rejected it, so the generated receipt,
  checklist projection, and progress append were transactionally rolled back
  and republished through the same typed command with valid code spans.
- Verification: The final transition receipt and full checklist validator
  accepted the exact change with no validation errors; progress remains 39
  verified of 94 active rows, 41.5 percent complete.
- Evidence: Transition receipt, `RESULT.md`, exact implementation commit,
  focused packet/ownership tests, and the real ESP32-S3 firmware build.
- Outcome: `NET-002` is `implemented` with `unit,workflow`; no live provisioning
  client, reconnect, credential, or hardware evidence was inferred.
- Blocker or next safe action: Run final metadata gates, commit, push, then
  resume deterministic parity selection.

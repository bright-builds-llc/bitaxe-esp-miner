---
status: investigating
trigger: "Plan 13 passed physical lifecycle, USB ownership, passive capture, and cleanup, but the retained cold-start log contained no boot or listener markers."
created: 2026-07-12T04:00:00Z
updated: 2026-07-12T21:17:00Z
---

## Current Focus

hypothesis: An independently powered receive-only UART observer can preserve original UART0 boot bytes across a true both-power board cold start because its adapter node and reader remain present while barrel power and native USB are removed.
test: Verify and push the schema-v3 software authority without hardware, then pause for an RX-to-TP18 and GND-to-TP12 fixture installed only while both board power paths remain removed.
expecting: The continuous UART reader records a quiet pre-boot boundary and one distinct post-boundary boot session with original boot/listener markers, both evidence states, monotonic heartbeats, and all five accepted-state stages.
next_action: Commit and push the verified schema-v3 software authority, prove clean 0/0 upstream sync, then request only `uart-receive-only-fixture-installed` before any adapter-node observation.

## Symptoms

expected: Once both power paths are removed, the lifecycle is already observing the selected node before the user restores barrel power and USB; capture proves that one boot reached listener readiness even when native USB missed original bytes.
actual: Exact HEAD `e622253d2fc4aea4589e0dcf5524081b6b054aaf` passed strict reflash/reinit heartbeat validation, proving the always-on observer and both cadence phases. The retained lifecycle then armed before restoration, observed USB automatically, acquired passive ownership, completed its bounded capture, and cleaned up with zero holders, yet the raw application payload was exactly empty.
errors: Cold monitor bytes `0`; heartbeat markers `0`; boot-evidence markers `0`; accepted-state snapshots `0`; terminal `blocked_safe_evidence_invalid`. The session trace reports pre/post readiness, expected active ownership, stable identity, timeout-after-capture, and complete cleanup.
reproduction: On exact HEAD `e622253d2fc4aea4589e0dcf5524081b6b054aaf`, complete strict reflash/reinit, remove both power paths, wait for `plan13-restore-watcher-armed-v1`, then restore barrel and USB without a response. Attachment succeeds but no application byte arrives despite the boot-lifetime heartbeat.

## Feedback Loop

command: `node scripts/phase28.1.1-hardware-attempt-state-test.mjs && node scripts/phase28.1.1-accepted-state-lifecycle-compare-test.mjs`
red_output: The state authority exposed no response-free public restore action, and the lifecycle validator exposed no session-tagged boot-evidence parser.
properties: Deterministic, hardware-free, and located at the state/evidence contract that failed in the real attempt.

## Working Diagnosis

root_cause: Confirmed up to the transport boundary. The old human acknowledgment race and Stratum-coupled replay were real defects and are repaired, but neither explains the current empty stream. A passive native-USB session can have a present stable node and correct owner while carrying no application bytes after barrel-first boot and later USB attachment.
confidence: high that firmware heartbeat scheduling is not the blocker because reinit validates it; medium on whether the remaining boundary is ESP32-S3 USB Serial/JTAG late-attach behavior or `espflash` passive-reader behavior.
smallest_correct_seam: Add a no-reset transport A/B with positive byte-delivery observation below the Plan 13 evidence parser. Do not change evidence semantics or add another firmware replay mechanism until that boundary is classified.

## Fix in Progress

- The private attempt state uses a response-free `restore_watcher_armed` action, a 30-minute exact-node appearance window, and a 60-second passive ownership-attachment bound while retaining the 4,145,000 ms lease.
- Firmware generates one 128-bit hardware-RNG boot nonce and retains redacted `booted` and `listener_armed` proof in Plan 13 evidence mode. The follow-up correction moves the allowlisted replay task out of the Stratum adapter and schedules it from boot for 10-second ticks strictly before 1,880,000 ms.
- Validation requires original boot/listener lines for reinit, but permits replay-only cold-start proof; equivalent duplicates pass while malformed, missing, conflicting, or multiple-session proof fails.
- Raw monitor, wrapper, and session traces are escrowed under a mode-0700 ignored root as mode-0600 files before validation and again before tombstoning.
- The new late-attach diagnostic returns an opaque handle before fallible preflight, runs the mandatory detector exactly once, proves both connected readers observe one heartbeat session, and then uses a real mode-0600 Unix-socket capability plus owner PID fingerprint to bind the removal token to one isolated lifecycle process.
- After five seconds of exact-node absence it emits a response-free restore action, requires the same physical USB identity and a new enumeration epoch, and captures `espflash` / OS-native / `espflash` without flash, reset, serial writes, scans, credentials, network discovery, or a post-run detector.
- The OS-native reader is a standalone Perl process restricted to read-only, no-controlling-terminal, nonblocking open plus `select` and `sysread`. Raw reader stdout is separated from wrapper/tool stderr, while the default `espflash` monitor interface remains compatible.
- Plan 14 adds a standard-library `uart-native` reader that opens the independently powered adapter read-only, configures only 115200 8N1/local/no-flow-control, and never writes or manipulates modem/reset lines.
- Stable physical USB identity now excludes tty paths, inode data, registry entry IDs, and dynamic instances; native restoration requires the same physical identity with a new enumeration identity, while the continuously connected adapter requires both identities unchanged.
- Schema v3 starts one continuous UART owner before publishing removal, records a newline-aligned quiet byte boundary after verified board-power absence, and accepts only one distinct post-boundary cold session with original and replay evidence plus all five accepted-state stages.

## Remaining Verification

- Treat the pushed A-B-A software authority and its one-shot failed preflight as closed inputs; do not reuse the stale handle or retry the attempt.
- Plan whether OS-native should become the formal passive cold reader, with a gate that does not require a reader already proven silent, while retaining exact-node ownership and no-write guarantees.
- Decide whether native USB can satisfy formal cold-start evidence at all; otherwise define the external UART or alternate-channel boundary explicitly.
- Do not run another Plan 13 hardware chain until the transport classification selects the correct reader or proves an alternate evidence channel is required.
- Treat schema-v2 tool HEAD `7cab0c63b9887e3670b9db20e0eaec50dc4fbf0f` and its failed qualification as closed inputs; do not reuse or retry the handle.
- Plan an external UART or independent data-only capture path that remains connected across both-power cold start.
- If native USB remains a secondary signal, split stable physical identity from enumeration identity before any further hardware authority.
- Finish and push the complete Plan 14 software authority before requesting fixture installation; no detector, serial open, reset, flash, credential, network, or board command is permitted before that checkpoint.
- After the fixture checkpoint, perform at most one schema-v3 qualification and only conditionally one fresh formal Plan 13 chain; retain and stop on any failed qualification.

## Software Verification

- timestamp: 2026-07-12T04:25:00Z
  checked: state authority, lifecycle comparator, strict classifier, exact-head broker/adapter, diagnostic wrapper, and passive monitor suites
  found: The state/lifecycle/classifier Node tests pass; the exact-head suite passes all 84 invalid cases; the diagnostic wrapper passes its response-free lifecycle fixtures; and the phase13 monitor suite proves a mode-0600 active-owner readiness signal plus passive cleanup.
  implication: The software contract is internally consistent and the next hardware attempt can be created only after the coordinating agent completes canonical formatting, Bazel/reference, Rust, commit, and push gates.
- timestamp: 2026-07-12T04:25:00Z
  checked: shell and source hygiene
  found: Bash/Node syntax checks, `shfmt -d`, warning-level `shellcheck`, `git diff --check`, and the reference-tree diff pass. The direct `bitaxe-api` replay tests pass. A direct host-target `bitaxe-firmware` test is not supported because `esp-idf-sys` rejects `aarch64-apple-darwin`; the canonical ESP target build remains required.
  implication: No known software-only blocker remains, but firmware compilation and real native-USB timing still require the repo-owned target and hardware gates.
- timestamp: 2026-07-12T05:00:00Z
  checked: boot-lifetime replay correction
  found: The ownership shell regression, lifecycle/state/classifier/exact-head/monitor suites, seven host cadence/allowlist tests, four affected Bazel targets, canonical `//firmware/bitaxe:firmware` ESP32-S3 build, reference guard, and mandatory Rust format/Clippy/build/test sequence all pass. The exhaustive broker suite retains all 84 invalid cases.
  implication: The follow-up software seam is clean and may be committed as a fresh exact HEAD. Hardware delivery remains deliberately unclaimed.
- timestamp: 2026-07-12T05:03:00Z
  checked: git finalization
  found: Durable findings are committed at `2b504d5`; the verified boot-lifetime replay repair is committed and pushed at `447f735c4df4363d84ea7b1354e32d57e28a68a5`.
  implication: Any subsequent hardware confirmation must begin a new exact-head attempt and build a new package; the failed package and resume handle remain unusable.
- timestamp: 2026-07-12T15:00:00Z
  checked: native-USB late-attach A-B-A software authority
  found: The backward-compatible monitor wrapper, standalone OS-native reader, pure seven-category classifier, and resumable begin/deliver broker pass direct tests and forced-uncached Bazel coverage for stream separation, real Unix-socket framing, owner fingerprints, permissions, token/lease/stale handling, response-free watcher arming, all classification patterns, identity/epoch/holder/probe/node failures, worker cleanup, and forbidden-operation guards. Adjacent serial-session and accepted-state suites also pass. No detector, board-info, monitor, credential, network, flash, reset, or hardware command was used during software verification.
  implication: The diagnostic is ready for root-owned final verification, commit, and push. No hardware classification is claimed yet.
- timestamp: 2026-07-12T16:16:25Z
  checked: schema-v2 OS-native diagnostic implementation and first focused tests
  found: The public adapter is thin; broker/state and lifecycle ownership are split. Pure classifier tests and the real-process diagnostic regression pass for an empty observational `espflash` control, mandatory OS-native preflight, action-before-removal ordering, early-token rejection, non-advancing status, response-free restoration, OS-native-only cold qualification, private permissions, v1 tombstone compatibility, and terminal cleanup.
  implication: The repaired transport authority is software-only and still needs the adjacent formal Plan 13 integration tests, forced-uncached Bazel gates, canonical verification, clean commit, and push before hardware use.
- timestamp: 2026-07-12T16:56:36Z
  checked: complete OS-native qualification and formal Plan 13 software authority
  found: Ten consecutive real-process qualification runs pass after closing receiver-publication and sourced-scope cleanup races. Direct monitor, trace, classifier, accepted-state, and exhaustive 84-case exact-head suites pass. Forced-uncached affected Bazel targets, shell/Perl/Node syntax, `shfmt`, warning-level `shellcheck`, reference/protected-artifact gates, and the mandatory Rust format/Clippy/build/test sequence pass. The private qualification is symlink-rejecting, owner/mode controlled, exact firmware/tool-head bound, contract-digested, and revalidated before formal cold serial capture.
  implication: Software implementation is complete and ready for root-owned review, commit, and push. Hardware qualification, Plan 13 closure, and any parity promotion remain deliberately unclaimed.
- timestamp: 2026-07-12T17:10:00Z
  checked: root review, software finalization, and git authority
  found: Independent review corrected mixed-log heartbeat qualification, explicit Bash capture-status propagation, and cleanup finalization so a qualification remains `cleanup_complete=false` until the broker proves owner/process-group exit. Repeated direct tests, all five forced-uncached affected Bazel targets, adjacent protected/reference checks, full changed-path shell checks, and the mandatory Rust sequence pass. The complete authority is committed and pushed at `7cab0c63b9887e3670b9db20e0eaec50dc4fbf0f`.
  implication: Exactly one hardware qualification was authorized on that clean HEAD; formal Plan 13 remained conditional on its passing private qualification.
- timestamp: 2026-07-12T21:17:00Z
  checked: Plan 14 receive-only UART foundation, schema-v3 authority, and formal integration through Task 2
  found: Direct UART PTY, monitor compatibility, Darwin/Linux identity, detector exact-node, pure classifier, real-process qualification, accepted-state, and exhaustive 84-case exact-head tests pass. All eight forced-uncached affected Bazel targets pass after declaring their complete runfiles, as do the canonical ESP32-S3 build, reference and protected-artifact checks, shell/Perl/Python/Node hygiene, and the mandatory Rust format, Clippy, all-target build, and all-feature test sequence. The lifecycle owns UART before removal action publication, rejects early tokens, preserves the adapter identity and owner across native-node absence, validates only post-boundary bytes, binds formal use to the exact adapter and contract digest, and cleans process groups and holders. No hardware command ran.
  implication: The implementation is software-complete but not yet hardware-authorized. Commit, push, and clean 0/0 sync must still pass before the fixture action is requested.

## Hardware Verification

- timestamp: 2026-07-12T04:54:00Z
  checked: fresh exact-head Plan 13 chain at `4891ce06bb51f872fd41c0baa2412cd660c877eb`
  found: Detector, credential binding, reference guard, package, and reflash/reinit completed. After both power paths were removed, the owner armed the restore watcher before action publication, observed USB restoration without a response token, acquired the stable holder-free node, verified the expected active monitor owner, captured for 360 seconds, and returned to zero processes and zero holders. The cold serial payload contained zero bytes and therefore zero boot evidence, listener evidence, or accepted-state snapshots.
  implication: The native-USB race and cleanup paths are repaired. The remaining blocker is firmware replay availability, not USB appearance, monitor ownership, capture duration, or dangling handles.
- timestamp: 2026-07-12T04:54:00Z
  checked: private trace retention and secrecy
  found: The mode-0700 escrow contains only mode-0600 files, including duplicated pre-validation and tombstone copies with digests. No active attempt directory, lifecycle process, espflash monitor process, or serial holder remains.
  implication: The failed run is diagnosable without resuming or repeating it, and raw local identities remain outside committed evidence.
- timestamp: 2026-07-12T14:03:00Z
  checked: heartbeat-enabled exact-head Plan 13 chain at `e622253d2fc4aea4589e0dcf5524081b6b054aaf`
  found: Strict reflash/reinit passed heartbeat, original-marker, and dedicated-evidence validation. The retained cold-start member passed watcher arming, automatic USB appearance, stable passive ownership, bounded capture, and cleanup, but its application payload remained exactly zero bytes with no heartbeat or evidence marker.
  implication: The boot-lifetime heartbeat is implemented and works when monitoring spans boot, but it cannot close a silent late-attached native-USB transport. No retry is permitted from this result.
- timestamp: 2026-07-12T15:01:04Z
  checked: one-shot late-attach diagnostic at pushed tool HEAD `a6623c8cebe54b85e4cb9e14bdcd83cd1d31b141`
  found: The mandatory baseline detector completed. The connected passive `espflash` preflight then produced zero application bytes and zero heartbeats, while the following read-only OS-native preflight produced 16 well-formed heartbeats from one session. The two-reader gate failed before the removal checkpoint, so no cold A-B-A category exists. The tombstone and private trace were preserved. Terminal cleanup found zero diagnostic processes, lifecycle sockets, and serial holders, but the exact node was absent, leaving accessibility and USB identity unavailable without a prohibited reset or recovery action.
  implication: `espflash` reader silence is reproduced even before late attachment, whereas the OS-native reader proves the firmware and USB transport can deliver bytes in that connected state. This narrows the next seam toward replacing or repairing the passive reader, but the cold behavior and node disappearance require a new plan rather than an ad hoc retry.
- timestamp: 2026-07-12T17:27:24Z
  checked: one authorized schema-v2 qualification at tool HEAD `7cab0c63b9887e3670b9db20e0eaec50dc4fbf0f`
  found: The planned full-power recovery, baseline detector, connected controls, owner-before-action removal observation, token delivery, and five-second absence completed. On response-free restoration, the exact node appeared but the USB identity digest differed, producing `appearance_identity_changed` before OS-native cold capture. Source inspection shows the digest includes enumeration-variant macOS tty and IORegistry entry fields. No qualification file or cold-reader log was created. Terminal cleanup found the exact node present and accessible with zero holders, diagnostic processes, and sockets; no post-run detector ran.
  implication: The gate correctly stopped, but the category cannot distinguish physical-device replacement from an expected enumeration-field change. The no-retry branch is active: do not start Plan 13; plan external UART/data-only evidence next.

## Resolution State

root_cause: The authoritative native-USB reader disappears with board power and re-enumerates only after boot has begun, so it cannot reliably preserve original cold-start bytes. The earlier response race, service-coupled replay, silent `espflash` reader, absence-only removal proof, and physical/enumeration identity conflation were additional independent defects.
fix: Use an independently powered receive-only external UART observer that remains enumerated and continuously owned across board power removal. Preserve native USB only for explicit detection, flashing, and secondary identity evidence. Bind formal cold capture to one private passing schema-v3 qualification and validate only the new post-boundary boot session.
hardware_status: No Plan 14 hardware action has occurred. The last hardware result remains the closed schema-v2 failure at tool HEAD `7cab0c63b9887e3670b9db20e0eaec50dc4fbf0f`; cleanup was complete and no retry occurred. Fixture installation and one-shot schema-v3 qualification remain gated on a newly verified, pushed, clean 0/0 tool HEAD.

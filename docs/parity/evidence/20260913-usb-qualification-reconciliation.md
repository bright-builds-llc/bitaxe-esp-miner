# USB qualification reconciliation — 2026-09-13

The accepted evidence satisfies fixed-USB continuity, bounded live acceptance,
and fresh Hello recovery. One explicit qualification obligation remains:
measured CPU0 main-telemetry cadence under hardware load. Consequently,
`task-fixed-usb-serial-qualification` and its parent migration remain active.
The original live-acceptance task is superseded, not retrospectively passed.

This reconciliation follows the task/archive and evidence rules in `AGENTS.md`,
the Bright Builds sidecar, local overrides and verification standards. It is
read-only with respect to hardware and existing evidence. It neither creates
effect authority nor changes any parity row.

## Evidence and exact execution scope

| Evidence                                                                 | Firmware / Gate                                                                         | Accepted scope                                                                                                                                                                                                            |
| ------------------------------------------------------------------------ | --------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| [September 8 acceptance](20260908-worker-preparation-live-acceptance.md) | `f3bbfd6a05abcffa3735a736c87ff0c250ab8e2a` / `2106f1c1587025d0570e058647a29492159e5d20` | Four continuity cycles; normal work/share/renewal; real foreground and heartbeat-loss windows; shutdown, cooling and cleanup. Host continuation source is separately bound to `88ddb8a507477faa9220588d6ecf1f6de27fd754`. |
| [September 11 recovery](20260911-hello-recovery-live.md)                 | `6b6aa29e2d8ce4c788d0d6b3a41a86fc06f77757` / `632db8110bc74d4a7e3dac60fd915a5900b81f6a` | Four fresh exact-pair cycles, maximum exchanges, live loss, first-attempt authenticated recovery without drain/reset/reflash, separately authorized mining resumption and cleanup.                                        |

These are distinct executions. The later image did not repeat the earlier
normal accepted-share or intentional heartbeat-suppression campaign. The short
recovery runs returned three ASIC nonce results that did not meet the pool share
threshold; they claim no accepted share. The original missing-reply attempt 003
remains unchanged and its physical cause is unassigned.

## Obligation matrix

Every remaining checkbox in the qualification block is covered below. Earlier
progress statements describe their original point in time; this review does not
rewrite them.

| Obligation                                                                                                             | Classification                                   | Evidence and verification boundary                                                                                                                                                                                                                                                                                |
| ---------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------ | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Fixed Serial/JTAG ownership, Controller 0.4 / serial 0.2 / possession 0.2, exact published source and disjoint updates | satisfied                                        | Existing implementation and ownership checks; both exact-pair reports; September 11 initial installation plus four ordinary updates exclude NVS seeding and preserve identity/settings.                                                                                                                           |
| Detector/process cleanup, healthy startup before browser use, stable exact runtime and safe final baseline             | satisfied                                        | Revalidated September 11 operator evidence checks detector provenance, five trusted flash assessments, no startup failure, fresh browser admission, inactive lease, restoration and observed host cleanup.                                                                                                        |
| Four current-profile no-mining cycles and 65536-byte requests/responses                                                | satisfied                                        | Four September 11 cycle receipts with 65376 bytes of request padding; original preservation baseline and artifact identities remain bound. Four is the accepted ADR-0022 sample; old 20-cycle statements are historical.                                                                                          |
| Fragmentation/coalescing and integrity                                                                                 | satisfied, bounded scope                         | Deterministic production-channel/decoder regressions cover fragmented and coalesced records, credits and exact integrity; real maximum-size exchanges verify the qualified hardware transfer. This does not claim an exhaustive hardware chunk-boundary matrix or complete stale-control-reply hardware coverage. |
| Session replacement, foreground closure, heartbeat expiry and serial ownership                                         | satisfied                                        | September 8 separately observed native visibility loss and heartbeat-only suppression; device-local gate/shutdown offsets are 2807/2821 ms and 2805/2826 ms. September 11 closes fresh admission and ownership-release reliability without a drain.                                                               |
| Publish the coordinated diagnostic/protocol fixes before subsequent attempts                                           | satisfied                                        | Published exact Gate/firmware pairs and their required software checks precede the accepted executions; preserved earlier failures are not relabeled.                                                                                                                                                             |
| Wi-Fi static RX/TX 6/6, receive window 12, dynamic RX 32, retained reserve and later owners                            | satisfied, bounded scope                         | Resolved build guard and native builds enforce the profile; later healthy startup and bounded USB/mining executions qualify the observed operating sample. No arbitrary-load memory guarantee is claimed.                                                                                                         |
| Reject missing, duplicate or stale resolved configuration; restore build-validator coverage                            | satisfied                                        | `requireResolvedUsbMemoryContract` in `tools/automation/src/build.ts` enforces exact single assignments, including reserve, stack, affinity, priority and buffer settings; canonical build-validation coverage and native package gates passed.                                                                   |
| Main telemetry handoff, server lifetime, priority, CPU0 affinity and readiness ordering                                | satisfied in software                            | `firmware/bitaxe/src/http_api/cadence_owner.rs`, build guards and the five main-telemetry host cases establish ownership, activation and cleanup; subsequent hardware startup establishes composed service readiness.                                                                                             |
| CPU0 telemetry cadence under qualification load                                                                        | unresolved                                       | Host handoff tests intentionally stop the loop and do not measure periodic execution. The accepted hardware samples lack main-loop interval/missed-publication evidence; healthy startup, Worker samples and a configured 500-ms sleep cannot establish cadence.                                                  |
| Normal mining with renewals and correlated accepted shares                                                             | satisfied by successor; original task superseded | September 8 normal window: 59069 active ms, 26 work items, five accepted shares and three renewals. The failed original normal window remains failed.                                                                                                                                                             |
| Bounded fault windows, safe stop, cooling, volatile lease cleanup and mine-on-boot false                               | satisfied by successor; original task superseded | September 8 foreground/heartbeat windows: 10510/14020 active ms, both within their 30000-ms reservations; qualified shutdown/cooling and resource release. September 11 separately confirms the current recovery baseline.                                                                                        |
| Original campaign completion/retry obligations and 240000-ms ceiling                                                   | superseded                                       | Original ledger remains exhausted at 240000 ms, masks 7, no pending reservation. No original window is retried or refunded. The independently authorized successor charged all three final reservations and separately retained its earlier charges; no claim that its total ledger was limited to 240000 ms.     |
| Fresh Hello recovery and subsequent mining                                                                             | satisfied                                        | September 11 loss gate/shutdown 2651/2676 ms, first reconnect without drain/reset/reflash, post-work authorization checkpoint preserved, distinct resumed generation. Read-only verification checks both sealed outcomes and accounting continuity.                                                               |
| Parent migration completion                                                                                            | unresolved                                       | Implementation/publication and successor live acceptance are supported. Closure still depends on the outstanding telemetry-cadence qualification item.                                                                                                                                                            |

The original live task's remaining preparation/diagnostic, publication,
attempt-011/012, final-window and cleanup checkboxes belong to its exhausted
campaign. Their native record is archived unchanged except for this successor
closure review; they are not marked as successful executions.

## Artifact integrity and source continuity

The September 8 final acceptance index matches its published SHA-256
`dd3396c013312ddf944eefd2013986fc9efd042c75b27511138310e6c817ad69`.
Read-only `readPrevious`, `verifyArtifactSnapshot` and `validateCycle` calls
revalidate all three passed result receipts, their sealed samples, 39 retained
artifact files and 12 cycle-receipt copies. No `judge`, preflight, server or
sealing command is used to rewrite evidence or open a device session.

The retained Hello auditor runs in `verify` mode only. Its source SHA-256 is
`643a4a4f65802cc8514bd5965e64ef0171fef8996f6a0305e8225ce560be7ec4`.
It verifies both sealed phases, 26 retained artifact files, four preserved
cycles, final journals, authorization/accounting continuity, and observed host
cleanup. The loss and resume inventories contain 209 and 72 files respectively
and still match the published report. Hardware stale-reply coverage remains
false. Verification inspects protected operational evidence without exporting
credentials, device identifiers, pool details or private authorization values.

Firmware `f3bbfd6a..6b6aa29e` leaves SDK defaults, main/startup, HTTP telemetry,
production mining, shutdown and safety implementations unchanged. Runtime
changes add authenticated trace review and correlated receive/dispatch/write
observations. The write loop retains 512-byte chunks and its 2000-ms shared
write/drain bound; final cancellation gains a typed abandonment observation.
Bounded trace storage still adds overhead, so earlier measurements cannot be
reassigned to the newer binary.

Gate `2106f1c..632db811` changes actual session behavior: bounded stale-record
bootstrap, synchronous delimiter admission, browser traces, interruption
semantics, post-work authorization checkpoints, fresh possession and cleanup
ordering. This is not a documentation-only change. The two exact-pair campaigns
support their separate milestone obligations, not interchangeable hardware
results.

Retained browser traces explicitly show fragmented assembly: one 317-byte frame
spans five native reads and one 2694-byte frame spans 35. None of the five
retained browser trace windows provides an explicit assembled-frame event with
queued trailing bytes, so hardware read-batch coalescence is not claimed.
Deterministic coverage resides in Gate's `worker-serial.test.ts`,
`worker-serial-bootstrap.test.ts`, `worker-read-interruption-coalesced.test.ts`
and the Rust `bitaxe-worker-control` framing/conformance tests.

## Remaining work and task disposition

Keep migration and qualification active. Before a future hardware effect,
extend the qualification contract with bounded, nonsecret main-telemetry
iteration/publication timing observations and explicit acceptance criteria
derived from the existing cadence behavior. Verify the instrumentation and
publish an exact package, then collect cadence-under-load evidence through a
fresh qualified session. Do not invent a hard 500-ms deadline from a loop that
sleeps 500 ms plus execution time, reuse exhausted mining allowances, or widen
safety deadlines. No such instrumentation or hardware action occurs here.

Archive the original live-acceptance record as superseded by the accepted
successor, retaining its failed/exhausted result. BWG restoration and STR-005
continue to depend on the still-open qualification and require their own
current serial successor contracts. Historical recovery-006 instructions do
not become executable through this reconciliation.

Existing preflight admission still requires active task IDs. Archiving the
original live task intentionally prevents its normal preflight from admitting
another campaign; archived IDs must not be accepted as authority.

## Verification

Current verification passed:

- Ordered `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, and `cargo test --all-features`: 2170 passed, one preexisting ignored. Host debug information was disabled for dev/test profiles; behavior and deadlines were unchanged.
- `bun scripts/bright-builds-check.ts all`: zero findings. The existing oversized active lesson set uses bounded loading; the August 30 audit baseline has only four later lessons, so no new maintenance trigger applies.
- `just test`: all 103 targets passed. The first run passed 102 targets but two simulated process cases inside the automation suite exceeded their existing 10-second bounds while Cargo was also running. After Cargo completed, the unchanged automation suite passed in 75.9 seconds; those cases took about 0.4 seconds each. Original failure logs are retained; no fixture, assertion or timeout was changed, and no machine-wide cause is inferred.
- `just verify-native-usb-ownership`, `just verify-reference`, `just verify-redaction`, `just parity`, and `just parity-progress`: passed. Semantic redaction checked 22 typed evidence documents; this report separately underwent manual metadata/privacy review.
- New report formatting uses the available GFM/frontmatter-aware `mdformat` in check mode after a scoped write. Historical task/archive formatting is preserved. Diff review and an independent source/evidence review found no blocking issue.
- Native-record checks verify the entire preexisting archive prefix, exact appended records, preserved original live-task checkboxes, unique task IDs and resolvable report links. A direct preflight boundary check rejects the missing original live task as `active_task_missing` before package/authority access; only the git-cleanliness dependency is stubbed for this software check.

Parity remains **90/95 active rows verified, 99 total, four deferred (94.7%)**.
The checklist, README and progress history are unchanged. Only this report and
the relevant task/archive records are published; no hardware effect occurred.

---
generated_by: gsd-discuss-phase
lifecycle_mode: yolo
phase_lifecycle_id: 35-2026-07-17T17-00-37
generated_at: 2026-07-17T20:07:50.170Z
---

# Phase 35: Detector-Gated Correlated Evidence and Exact Parity Promotion - Context

**Gathered:** 2026-07-17
**Status:** Ready for planning
**Mode:** Yolo

<domain>
## Phase Boundary

Deliver one bounded, detector-gated Ultra 205 evidence chain for the exact current firmware package. The chain must correlate read-only telemetry, confirmed hostname persistence across one approved normal reboot, truthful runtime/package identity, passive health, lifecycle cleanup, and redaction before admitting only evidence-supported v1.2 operator-runtime parity rows. Active control, self-test effects, watchdog intervention, mining and the archived Phase 28.1.1 lineage, credentials, direct UART or pins, OTA/recovery, other boards, and every broader claim remain deterministic non-promotions.

</domain>

<decisions>
## Implementation Decisions

### Detector admission and evidence-root structure

- **D-01:** Use three ordered, typed gates. Gate 1 performs only pure current-HEAD, reference-cleanliness, manifest-v3, executable-image, and package/runtime-identity admission and freezes the exact factory bytes. Gate 2 runs the sole reset-capable `just detect-ultra205` preflight and constructs a board-205 run capability bound to the stable physical-identity digest. Gate 3 alone may resolve the target, flash, monitor, PATCH, reboot, capture, restore, or stage promotion.
- **D-02:** Create one mode-0700, content-addressed staging root per attempt. Keep raw commands, device paths, origins, physical/enumeration identities, trace records, and other local identifiers only in mode-0600 protected files. Bind the root to one run identifier and root-contract digest.
- **D-03:** The evidence root must bind the full source and reference commits, manifest and package digests, frozen factory-image digest, board category, detector and target-lock digests, boot epochs, operator-snapshot revisions, monotonic event sequence, and predecessor event digest. Human labels and mutable paths never authenticate the chain.
- **D-04:** A failed or interrupted root is sealed as a deterministic non-promotion and cannot be resumed, retried in place, or spliced with another root. Any later attempt requires a fresh root and an explicitly valid Phase 35 lifecycle; no failed attempt may contribute positive proof.

### Correlated capture, reboot, cleanup, and redaction

- **D-05:** Reuse the proven Phase 33 detector, admitted-package flash, passive-monitor, HTTP PATCH/readback, restart-after-response, physical-identity, restoration, and process-cleanup mechanics through a thin Phase 35 shell supervisor. Keep chronology validation, epoch correlation, inventory, redaction admission, and final evidence decisions in typed Rust.
- **D-06:** Model the approved reboot as two internally coherent epochs, not one mixed-session snapshot. Boot A owns pre-PATCH and storage-confirmed immediate system-info, WebSocket, and retained-log revisions. Boot B owns the same-board post-reboot projections. Join them only through the protected run identifier, exact package identity, unchanged stable physical identity, boot ordinal `N → N+1`, `software_cpu` reset reason, response-before-effect proof, and matching non-secret hostname digest.
- **D-07:** Within each boot epoch, correlate read-only sensor acquisition truth, system-info, live WebSocket, retained marker, and runtime-health record through the same boot session and monotonic operator-snapshot revision. Do not weaken the Phase 34 same-session validator to accept cross-reboot mixtures.
- **D-08:** The sole approved reset in the proof interval is the existing access-gated application restart after its response completes. Detector reset and package flash belong to setup chronology outside the proof interval. Power cycling, raw reset, OTA, panic/watchdog/fault reset, extra reset, and archived diagnostics invalidate the attempt.
- **D-09:** Use the complete passive ESP32-S3 monitor contract, at least 360 seconds of capture, at least 420 seconds of shell wall-clock budget, bounded ownership/readiness gates, process-tree reap, zero unexpected serial holders, and verified restoration of the original hostname. Cleanup or restoration failure prevents admission even when earlier observations passed.
- **D-10:** Derive the shareable projection only after protected-root inventory, chronology, current-head, reference-cleanliness, exact-package/runtime identity, no-actuation, cleanup, restoration, and redaction gates all pass. Shareable evidence may contain only approved categories, counts, durations, booleans, and digests—never credentials, settings values, raw targets, origins, network identities, device paths, USB identities, PIDs, MACs, SSIDs, or secrets.

### Exact allowlist promotion and deterministic non-promotion

- **D-11:** Extend the closed Phase 31 admission pattern into a Phase 35 typed decision matrix. Parse eligible evidence into domain facts and require one explicit `Promote(exact_row, evidence_digest)` or `DoNotPromote(typed_reason)` result for every Phase 35-owned outcome.
- **D-12:** Prefer dedicated, purpose-built v1.2 parity row identifiers when an existing checklist row is broader than the evidence. Never mark a broad settings, safety, watchdog, self-test, mining, or production-ready row verified from a narrower passive observation or hostname proof.
- **D-13:** The matrix must be exhaustive over the eligible operator-runtime allowlist and every exclusion already modeled by the v1.2 contract. A compile-time or test-time completeness guard must fail when a new row or exclusion lacks an explicit decision.
- **D-14:** Generate the final admission artifact and checklist projection in a validated staging generation, prove every non-allowlisted row remains byte-identical, then admit the generation atomically. The admitted verdict is created last; lifecycle completion, plan completion, green tests, or an unadmitted evidence directory never count as parity proof.
- **D-15:** Preserve `STR-09`, `ASIC-11`, and `CFG-07` plus every Phase 30/archived-lineage non-claim exactly. Active control, self-test effects, watchdog intervention, mining/share behavior, credentials, direct UART/pins, OTA/recovery, non-205 boards, and broad production/verified claims always receive deterministic non-promotion in the Phase 35 artifact.

### Attempt lineage and private classifier boundary

- **D-16:** Attempts 1 through 12 are sealed, immutable historical evidence. The private-first and HTTP-timing software corrections do not rewrite, retry, splice, promote, or change the conclusion of any prior attempt.
- **D-17:** Boot A classification and private target derivation consume only the immutable, secret-sanitized `flash-monitor.classifier-input.log` below the protected root. Dual `flash-monitor` closes and hashes that private log and its private record without creating `flash-monitor.log`. Only after the classifier passes does the supervisor invoke the already-built software-only finalizer with the verified digest; that finalizer creates the distinct commit-redacted compatibility and admission projection and proves the private digest is unchanged.
- **D-18:** Attempt 12 is sealed non-promotion history after `http_diagnostic_invalid` before PATCH or mutation. Its root remains non-reusable, and the verified HTTP-timing repair does not retroactively change that conclusion.
- **D-19:** Attempt 13 is the first authorized fresh continuation. It requires a passing exact-current-HEAD preflight and every fresh-attempt invariant in `docs/hardware/hardware-attempt-policy.md`. Any later fresh ordinal requires `continue_after_verified_fix` or `continue_after_manual_remediation`; elapsed time, a renamed category, or an unchanged retry is never progress. Each fresh root invokes the full Phase 35 hardware command exactly once, and only `complete` may unlock admission audit or `35-04-SUMMARY.md`.
- **D-20:** Attempt 13 executed once at exact source `02f128db56b332e50e11f57935f29e22e3830f66` and repeated `http_diagnostic_invalid` after the targeted timing fix. Its root is sealed non-promotable and non-reusable. The selected decision is `stop_repeated_boundary`; no later ordinal, Task 3, evidence admission, checklist promotion, or `35-04-SUMMARY.md` is authorized by the current workflow.
- **D-21:** A sealed-input replay proved that attempt 13's generic invalid fallback had two additional deterministic causes: curl scheme case was not canonicalized, and the configured 10-second request deadline was incorrectly reused as an exact observed-duration ceiling. Commit `53d8bcee` normalizes scheme case and admits only a separately bounded 11-second observation envelope; the unchanged sealed shape now classifies as `request_transmission_incomplete`. The user's 2026-07-21 authority permits attempt 14 and later fresh ordinals only after a distinct regression-backed fix or confirmed non-invasive remediation. There is no fixed ordinal cap, but unchanged retries remain prohibited and every fresh attempt still requires the complete exact-current-HEAD gate.
- **D-22:** Attempt 14 executed once at exact source `8afbed3248fb00e02d1a09f726b48ec241b552da` and stopped before mutation with the new typed category `request_transmission_incomplete`. Its projection proves TCP connection but zero request bytes, no status, headers, or body, and complete cleanup without a secondary failure. The root is sealed non-promotable and non-reusable. A fresh attempt 15 is not authorized until this exact boundary is deterministically reproduced and fixed in software with regression coverage and the complete exact-current-HEAD gate.
- **D-23:** A deterministic fake-curl regression and isolated loopback observation proved curl exit 56 is a receive-side failure that can follow a complete bodyless GET even when `%{size_request}` remains zero. Commit `0dd2134e` preserves that raw counter but derives request completion from positive request bytes or the closed receive-error category; exit 55 remains the send-failure boundary. Exact sealed-input replay now advances attempt 14 to `response_status_missing` without altering its immutable seal or input digests. This distinct regression-backed fix selects `continue_after_verified_fix` and authorizes fresh attempt 15 after the complete exact-current-HEAD gate. Later ordinals remain uncapped only when each follows another qualifying fix or confirmed non-invasive remediation; blind retries remain prohibited.
- **D-24:** Attempt 15 executed once at exact source `1c4979f67c0b12daee356ae5df1c1c5468ba1013` and repeated the primary category `request_transmission_incomplete`. Unlike attempt 14's receive error, its redacted projection records curl timeout category 28 after TCP connection at the configured deadline, with a zero raw request-byte counter and no response facts. The root is sealed non-promotable and non-reusable with no secondary restoration or cleanup failure. The current progress contract selects `stop_repeated_boundary`: software diagnosis remains authorized, but no attempt 16 is authorized unless a later explicit policy decision validly reopens the loop after a distinct verified fix.
- **D-25:** Attempt 15 exposed a different authoritative boundary signature despite repeating the coarse terminal category: response timeout after TCP connection, rather than attempt 14's receive error. Local peers proved the host curl request-size counter stayed zero after complete and successful bodyless GETs, so commit `d097bbbf` replaces curl with a schema-v2 Rust probe that records send completion only after the full write and transport flush succeed. Real adapter/runfiles regressions cover valid response, silent response after complete send, short-write failure, and TLS failure without persisting raw requests. The repository policy now stops recurrence of the same redacted authoritative signature, not a coarse category that conceals a newly discriminating subtype. The user's latest 2026-07-21 post-fix authority and this verified distinct fix select `continue_after_verified_fix` for fresh attempt 16 after the complete exact-current-HEAD gate; unchanged retries and every stronger boundary remain prohibited.
- **D-26:** Attempt 16 executed once at exact source `823309599209cde451435c85bb882fe8a456f80d` after the complete software gate and exact-head preflight passed. The schema-v2 probe proved TCP connection and complete request write plus transport flush, then timed out without any response status, headers, or body. Its authoritative signature is `response_status_missing` plus `response_timeout`; the root is sealed non-promotable and non-reusable and mutation never started. This is a newly discriminating response-side boundary, so software diagnosis is authorized, but attempt 17 requires a deterministic reproduction and verified fix or a confirmed permitted non-invasive remediation. An unchanged retry is prohibited.
- **D-27:** Exact release-ELF analysis proves the Phase 34 ordered system-info publisher reserves 6,080 bytes and candidate collection reserves 1,456 bytes before HTTP framework, retained-record, and JSON-writer overhead on the unchanged 8 KiB ESP-IDF HTTP server task. The narrow repair raises only that task to an explicit 16 KiB and adds a source guard; Phase 34 completion-ordered retention and issuance remain intact. The user's standing authority and this distinct regression-backed fix select `continue_after_verified_fix` for fresh attempt 17 after the complete clean exact-current-HEAD gate and preflight. Hardware remains the required runtime proof of task-stack sufficiency.
- **D-28:** Attempt 17 executed once at exact source `98463e8a735233b4e283b6535d3c9f375a984523` after both gates passed. It validates the 16 KiB HTTP task repair: the original read reached schema-v2 `ready` with complete status, headers, and body in 396 milliseconds. The supervisor then stopped before mutation with `pre_patch_mismatch`, no restoration or cleanup secondary, and a sealed non-reusable root. This newly discriminating capture-coherence boundary authorizes software diagnosis; attempt 18 requires a deterministic regression-backed fix or confirmed permitted remediation and another exact-head gate.
- **D-29:** Attempt 17 exposed a fixture/production contract split. Production omitted the setting digest, sourced boot ordinal from an absent API field, discarded the WebSocket envelope boundary, fabricated retained evidence and duration, and required equal API/WebSocket revisions despite completion-ordered publication. The repair uses serial-classified boot identity, protected live API/WebSocket-data/retained-log artifacts, a hash of the validated private hostname, real monotonic bounds, and a strictly later same-session WebSocket revision. Boot B classification now slices the passive trace only after HTTP service loss and forwards the baseline identity. Hermetic regressions cover the complete production adapter and fail-closed boundaries. This distinct verified fix selects `continue_after_verified_fix` and authorizes fresh attempt 18 only after the complete clean exact-current-HEAD gate and preflight.
- **D-30:** Attempt 18 executed once at exact source `065240279c4657945ffce70d2baa501b4da7ceae`. Boot A pre-capture and PATCH succeeded; the coherent post-PATCH API and WebSocket artifacts were followed by malformed chunk framing on the retained-log response, so the supervisor preserved `boot_a_capture_failed`, restored the original setting, cleaned up, and sealed the root. The WebSocket helper initiated close but returned before its close event, leaving no ordering proof before the next HTTP request. A delayed-close loopback regression now proves the helper waits for peer closure, and Phase 35 requires an exact close marker before retained-log capture. This distinct verified fix selects `continue_after_verified_fix` and authorizes fresh attempt 19 only after the complete clean exact-current-HEAD gate and preflight.
- **D-31:** Attempt 19 executed once at exact source `6a88300f84d0db1907455974372fe0468f4957e3` after the complete gate and exact-head preflight. The internal detector admitted one board-205 target, then the flash process failed to connect before Boot A capture or mutation. Cleanup passed and the root is sealed non-promotable and non-reusable. The redacted authoritative signature is `flash_or_boot_a_failed` plus `target_connection_failed`. Policy selects `continue_after_manual_remediation`: one USB and barrel-power reset requires user confirmation, after which fresh attempt 20 requires another exact-head preflight. Recurrence of the same signature after remediation selects `stop_hardware_blocker`; an unchanged retry is prohibited.
- **D-32:** After the user confirmed the exact USB and barrel-power remediation, attempt 20 executed once at exact source `b06bf416cf65283c53aa0f69c15ed216a9858eaa` after fresh exact-head preflight. Detector admission again passed, then the same `flash_or_boot_a_failed` plus `target_connection_failed` signature recurred before Boot A capture or mutation. Cleanup passed and the root is sealed non-promotable and non-reusable. Policy therefore selects `stop_hardware_blocker`: attempt 21 is prohibited, Phase 35 remains incomplete, and no stronger electrical interface or unchanged retry is authorized.
- **D-33:** Attempts 19 and 20 remain immutable, but their existing safe counters support an offline `phase35-flash-boundary-v1` classification of `stage=factory` and `terminal_boundary=post_info_pre_transfer_failed`: both children completed device information and failed before transfer progress. The prior hardware-blocker stop applied to an unchanged, coarsely observed retry. The separately authorized espflash 4.5.0/reset/typed-boundary repair is a materially different verified software path. Attempt 21 may run only after the full clean software gate and exact-current-HEAD preflight; its in-invocation 4 KiB checksum probe must classify `ready` before credential access or writes. Recurrence of `flash_or_boot_a_failed/factory/post_info_pre_transfer_failed` selects `stop_repeated_boundary`.
- **D-34:** Attempt 21 executed once at exact source `e007c06a5350b197a7f2a1af1bb6a41472be651d` after the full software gate, espflash 4.5.0 doctor check, and exact-head preflight. The sole detector invocation selected one candidate but its reset-capable board-info connection failed, so the supervisor stopped before the typed checksum probe, credential access, or writes and sealed the fresh root as `connection_failure` with no flash stage. This is distinct from Attempts 19–20. Official espflash history shows that 4.5.0 includes a Windows-motivated reset-order change whose review explicitly lacked USB-JTAG-Serial validation; that is a strong software-compatibility hypothesis, not yet hardware proof. Policy selects one exact non-invasive USB/barrel-power remediation and a fresh Attempt 22 only after user confirmation and another exact-head preflight. Recurrence selects `stop_hardware_blocker`; no unchanged retry is allowed.
- **D-35:** After the user-confirmed remediation, Attempt 22 executed once at exact source `55a8f31ac9be6a2c056cd04f8cc226b923782b22`. Detector, checksum probe, factory, NVS, and monitor all classified `ready`; the supervisor then stopped before Boot A classification or mutation because dual capture reused the legacy one-shot marker gate and returned nonzero after a timeout. Offline classification of the immutable private input passed with `category=none`, proving the authoritative evidence was present and the rejection occurred too early. The repair permits only a dual-mode timeout to enter an explicit pending-private-classification state; ordinary captures and child failures remain fail-closed, and admitted derivation remains impossible until digest-bound finalization after classification. Attempt 22 stays sealed non-promotable. This new regression-backed software boundary selects `continue_after_verified_fix` for fresh Attempt 23 after the complete clean gate, atomic commits, and exact-current-HEAD preflight.
- **D-36:** Attempt 23 executed once at exact source `ead2347d32ed0dbb8be43c74a3fb3a85a32734a1` after the complete gate and exact-head preflight. It validated the Attempt 22 repair by passing private Phase 33 baseline classification and dual finalization, then stopped before mutation with `boot_a_pre_capture_failed` when the retained-log GET observed invalid chunk framing after a closed WebSocket exchange. Firmware source showed cadence tasks calling `httpd_ws_send_frame_async` outside HTTPD work-queue ownership; a stale queued send could target a descriptor reused by ordinary HTTP. The repair gives each registration a generation lease with disconnect cleanup, copies frame data and the lease into `httpd_queue_work`, and revalidates the exact current lease plus WebSocket protocol state inside HTTPD context before the sole direct send. Attempt 23 remains sealed non-promotable. This distinct regression-backed fix selects `continue_after_verified_fix` for fresh Attempt 24 after the complete clean gate, atomic commits, and exact-current-HEAD preflight. Recurrence of the same retained-chunk signature after this targeted fix selects `stop_repeated_boundary`.
- **D-37:** Attempt 24 executed once at exact source `dec8b8a6bef8f504ec83a7eebe03b69a08be5064` after doctor and exact-head preflight passed. It stopped at the read-only checksum probe before credential access or writes with `flash_boundary_invalid`. Protected shape inspection and installed espflash 4.5.0 source prove the child emitted a valid leading-zero-elided checksum because espflash prints the MD5 `u128` with unpadded lowercase hexadecimal formatting. The adapter incorrectly required exactly 32 digits, and the resulting classifier projection serialized `post_info_pre_transfer_failure` while the shell's canonical category is `post_info_pre_transfer_failed`. The repair accepts exactly one official 1-through-32-digit checksum shape and explicitly serializes the canonical boundary; real-process and immutable-input regressions pass. Attempt 24 remains sealed non-promotable. This distinct verified fix selects `continue_after_verified_fix` for fresh Attempt 25 after the complete clean gate, atomic checkpoint commit, and exact-current-HEAD preflight.
- **D-38:** Attempt 25 executed once at exact source `f3a4d350492f5cc1073c0f62bd1a20f8af4355e2` after doctor and exact-head preflight passed. Probe, factory, NVS, monitor, private Boot A classification, dual finalization, original/immediate HTTP reads, PATCH, and storage-confirmed readback passed. The approved-reboot helper then failed before issuing its POST because the built supervisor distributed `phase13-monitor-capture.sh` without its required adjacent `process-group.sh`; restoration still classified `ready`, and cleanup recorded only the exited passive-monitor child as a secondary outcome. The repair adds the missing production runfile and a built-target closure regression that loads the helper from Bazel runfiles. Attempt 25 remains sealed non-promotable. This distinct verified fix selects `continue_after_verified_fix` for fresh Attempt 26 after the complete clean gate, atomic checkpoint commit, and exact-current-HEAD preflight.
- **D-39:** Attempt 26 executed once at exact source `a4de3c3a480bb29075c1c17df5c7cb8fe9d69f7c` after doctor and exact-head preflight passed. It proved the Attempt 25 runfiles repair: the passive monitor reached pre-attach and active-owner readiness, the reboot POST completed, service loss was observed, and post-cleanup readiness, restoration, and cleanup passed. The bounded passive capture contained zero serial bytes, so the private Boot B classifier recorded `post_restart_identity_missing`. The public primary category again remained `approved_reboot_failed` after its targeted fix. At that checkpoint policy selected `stop_repeated_boundary`, and Phase 35 remained incomplete without eligible evidence.
- **D-40:** The separately authorized Attempt 26 diagnosis preserves its immutable seal while identifying a newly discriminating backend mismatch: Phase 35 hard-coded a fixed-path espflash runtime observer even though repository hardware evidence had already shown that observer could own the USB node and still deliver zero application bytes. The canonical replacement separates espflash bootloader operations, OS-native receive-only runtime observation, HTTP application control, physical identity, enumeration identity, and proof. Attempt 27 is authorized only after the new device-session tool proves pre-reboot application-byte delivery, sends exactly one restart POST, reacquires at most one matching physical device, and validates the same trusted HTTP origin against a hybrid quorum of changed boot session, exact `N → N+1` RTC ordinal, `software_cpu`, exact build identity, and the persisted hostname digest. USB re-enumeration, sampled HTTP loss, and post-reboot serial bytes are corroborating observations rather than mandatory facts. This is a regression-backed contract and backend change, not a renamed category or unchanged retry; Attempts 1 through 26 remain sealed and non-promotable.
- **D-41:** Attempt 27 preserves D-40 and sealed before restart with `observer_unqualified`: the macOS adapter produced no candidate from 33 initial samples even though a protected read-only comparison found one exact callout and the canonical shell parser reproduced the detector-bound physical identity. The deterministic defect is the Rust parser's requirement that nested ioreg property lines begin with a quote after whitespace trimming; real tree output places branch characters before the quoted key. Attempt 28 may run only after a sanitized nested-tree regression, the narrow parser repair, a clean commit, the complete software gate, and exact-head preflight. Attempt 27 remains sealed, non-promotable, non-reusable, and ineligible for splicing.
- **D-42:** Attempt 28 proves D-41's repair: initial same-device qualification, receive-only pre/post bytes, and one fully transmitted restart request all passed. Recovery then emitted `usb_identity_drift` because the adapter constructed separate candidates for one macOS serial service's callout and dial-in aliases. Attempt 29 may run only after a paired-alias regression, canonical callout candidate selection, a clean commit, the complete software gate, and exact-head preflight. A fresh protected GET confirms the original setting is restored; Attempt 28 remains sealed and non-promotable.
- **D-43:** Attempt 29 proves the canonical-callout repair and the complete device-session contract: same-device qualification, receive-only correlation, exactly one restart request, service recovery, exact build identity, changed boot session, `N → N+1` ordinal, `software_cpu`, persisted hostname, restoration, and cleanup all passed. Admission then sealed `validator_rejected`; protected offline replay narrowed the safe category to `inventory_mismatch`, and typed diagnosis found the producer-added newline on `boot_a_api` while the validator hashes exact embedded document bytes. Attempt 30 may run only after a regression covers all six epoch documents, the byte-exact producer repair is committed, and the complete software gate and exact-head preflight pass. Attempt 29 remains sealed, non-promotable, non-reusable, and ineligible for splicing.

### the agent's Discretion

- Exact Rust module, type, command, file, and private helper names, provided the pure admission core and thin effectful shell remain structurally separate.
- Exact content-addressing and event-chain representation, provided SHA-256-grade digests, ordered chronology, immutable admitted inputs, and fail-closed parsing are preserved.
- Exact dedicated v1.2 checklist row identifiers and note wording, provided each row is no broader than its evidence and all unchanged rows are mechanically guarded.
- How to factor reusable Phase 33 shell helpers, provided no behavior regression, no second detector invocation, and no Phase 33 evidence claim is retroactively widened.

</decisions>

<canonical-refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase and project contract

- `.planning/PROJECT.md` — v1.2 operator-ready boundary, accepted debt, prohibitions, and exact-evidence principles.
- `.planning/ROADMAP.md` — Phase 35 goal, dependencies, success criteria, and sole final-admission ownership.
- `.planning/REQUIREMENTS.md` — CFG-12 and EVD-10 through EVD-15 plus the complete v1.2 out-of-scope list.
- `AGENTS.md` — detector gate, hardware-first evidence policy, timeouts, serial ownership, credential/redaction rules, direct-UART/pin prohibition, and archived-lineage closure.

### Upstream phase contracts

- `.planning/phases/31-operator-claim-and-telemetry-contract/31-CONTEXT.md` — closed typed claim admission and exhaustive excluded categories.
- `.planning/phases/33-confirmed-settings-durability/33-CONTEXT.md` — approved normal reboot, same-board identity, passive capture, restoration, and CFG-12 Phase 35 ownership.
- `.planning/phases/33-confirmed-settings-durability/33-VERIFICATION.md` — software completion and exact-current-package hardware proof still pending.
- `.planning/phases/34-provenance-runtime-health-and-coherent-operator-snapshot/34-CONTEXT.md` — canonical identity, manifest-v3 admission, coherent snapshot, and passive-only boundary.
- `.planning/phases/34-provenance-runtime-health-and-coherent-operator-snapshot/34-VERIFICATION.md` — fresh 10/10 completion and Phase 35 unblocking evidence.
- `.planning/phases/34-provenance-runtime-health-and-coherent-operator-snapshot/34-SECURITY.md` — verified passive-health threat mitigations required before Phase 35.

### Existing implementation and evidence machinery

- `scripts/detect-ultra205.sh` — exact one-board detector and reset policy.
- `scripts/phase33-confirmed-settings-durability.sh` — existing admitted-package, PATCH, passive-restart, same-board, cleanup, restoration, and redacted-summary mechanics.
- `tools/flash/src/package_admission.rs` — current manifest-v3 and immutable executable-image admission.
- `tools/parity/src/operator_snapshot_evidence.rs` — coherent same-session operator-snapshot correlation and retained/public projection validation.
- `tools/parity/src/v12_admission.rs` — closed typed eligible/ineligible claim pattern and exclusion vocabulary.
- `tools/parity/src/operator_evidence/generation.rs` — staging validation, ownership, durability, recovery, and atomic evidence exchange patterns.
- `scripts/phase30-no-promotion-contract-test.sh` — authoritative conservative mining/credential/archived-lineage non-promotion contract.
- `docs/parity/checklist.md` — current parity rows and evidence notes that Phase 35 may update only through exact typed admission.

### Engineering standards

- `standards/core/architecture.md` — functional-core and imperative-shell boundary.
- `standards/core/code-shape.md` — control flow, optional naming, rerunnable scripts, and module-size guidance.
- `standards/core/testing.md` — behavior-oriented unit-test and Arrange/Act/Assert requirements.
- `standards/core/verification.md` — sync-first and repository-native verification contract.
- `standards/languages/rust.md` — Rust domain typing, module shape, adapter, naming, and test rules.

</canonical-refs>

<code-context>
## Existing Code Insights

### Reusable Assets

- `scripts/detect-ultra205.sh`: already fails closed on zero/multiple candidates and board-info failure while emitting protected session traces.
- `scripts/phase33-confirmed-settings-durability.sh`: already implements the exact detector-once, package-currentness, passive monitor, restart-after-response, same-identity, cleanup, restoration, and redacted-output sequence Phase 35 must extend.
- `tools/flash/src/package_admission.rs`: already owns canonical manifest-v3, ELF/application, immutable snapshot, and pre-effect hardware admission.
- `tools/parity/src/operator_snapshot_evidence.rs`: already validates same-boot-session/revision coherence across public and retained projections.
- `tools/parity/src/operator_evidence/generation.rs`: already supplies validated staging, symlink/path rejection, directory syncing, atomic exchange, rollback, and durability-failure categories.
- `tools/parity/src/v12_admission.rs`: already demonstrates a closed typed claim/exclusion matrix and tests that untyped strings cannot become eligible.

### Established Patterns

- Domain decisions live in pure Rust types and reducers; Bash and firmware adapters remain thin imperative shells.
- Hardware effects require detector-derived authority and exact admitted package identity before target resolution or credential access.
- Raw local evidence stays mode-0600 under mode-0700 gitignored roots; committed evidence is a separately derived redacted projection.
- Evidence promotion is staged, validated completely, synced, and atomically exchanged; partial or mixed roots never advance claims.
- Compatibility surfaces, lifecycle completion, and green tests do not authenticate parity without exact eligible evidence.

### Integration Points

- Add a Phase 35 `just`/Bazel evidence entrypoint that composes the detector, package admission, the Phase 33-compatible hardware shell, and the parity CLI.
- Extend the parity tool with Phase 35 run-contract parsing, two-epoch correlation, inventory/redaction/no-actuation validation, and the exhaustive promotion matrix.
- Extend the checked-in parity checklist only through the admitted Phase 35 generation and exact row allowlist.
- Keep simulation and failure-injection fixtures green before each fresh real-hardware qualification attempt.

</code-context>

<specifics>
## Specific Ideas

- Treat detector/package setup and the application-restart proof interval as separate chronology segments inside one evidence root.
- Preserve Phase 34’s strict same-session validator twice—once for boot A and once for boot B—then join the epoch bundles through typed continuity facts.
- Create the final admitted verdict last, after cleanup and restoration, so no positive artifact exists while effect cleanup is still uncertain.
- Use digests for the generated non-secret hostname and protected identities; never promote their raw values.

</specifics>

<deferred>
## Deferred Ideas

- General-purpose signed attestations or cross-organization evidence exchange belong in a future supply-chain milestone if evidence must cross trust boundaries.
- A reusable all-Rust HIL runner belongs in a later tooling phase after Phase 35 proves the current shell mechanics and typed admission boundary.
- A canonical registry that regenerates the entire historical checklist is a separate architecture migration, not part of Phase 35.

</deferred>

*Phase: 35-detector-gated-correlated-evidence-and-exact-parity-promotion*
*Context gathered: 2026-07-17*

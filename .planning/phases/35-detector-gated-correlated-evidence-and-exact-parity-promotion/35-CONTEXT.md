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
- Add simulation/failure-injection fixtures for every gate before the sole real hardware qualification is attempted.

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

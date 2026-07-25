---
generated_by: gsd-discuss-phase
lifecycle_mode: yolo
phase_lifecycle_id: 36-2026-07-23T15-20-53
generated_at: 2026-07-23T15:26:57.039Z
---

# Phase 36: Substantive Evidence Admission and Exact Re-Promotion - Context

**Gathered:** 2026-07-23
**Status:** Ready for planning
**Mode:** Yolo

<domain>
## Phase Boundary

Phase 36 repairs the v1.2 evidence contract so each affected promotion is supported by typed sensor, runtime-health, independently observed runtime-identity, and independently evidenced no-actuation facts. It then re-evaluates immutable Attempt 31 evidence under the repaired contract and atomically corrects the exact affected rows.

This is a software-first admission and re-evaluation phase. It does not rewrite Attempt 31 private bytes, widen the four Phase 35 claims, authorize hardware automatically, revisit the archived Phase 28.1.1 lineage, or add active control, self-test effects, watchdog intervention, mining, credentials, OTA/recovery, other boards, direct UART, or pin work.

</domain>

<decisions>
## Implementation Decisions

### Successor evidence contract

- **D-01:** Introduce a versioned Phase 36 successor admission envelope that references and fingerprints the immutable Phase 35 root. Do not retrofit new roles into `phase35-evidence-v1`, change Attempt 31's root digest, rewrite protected artifacts, or splice evidence.
- **D-02:** The successor envelope must distinguish the evidence-source commit from the current evaluator commit and identify which prior generation it supersedes. Exactly one generation resolver determines the authoritative current decision.
- **D-03:** Offline insufficiency is a typed result, not an invitation to infer missing facts or rerun hardware.

### Substantive sensor and runtime-health admission

- **D-04:** Preserve the existing operator-snapshot identity/revision validator and compose a stricter typed parser over the same API, WebSocket, and retained documents.
- **D-05:** Sensor admission validates actual truth state (`fresh`, `stale`, `unavailable`, or `fault`), values only when legal for that state, producer sequence and monotonic acquisition stamp, failure reason, and independent power, temperature, and tachometer semantics.
- **D-06:** Runtime-health admission validates lifecycle/health state, supervisor availability, checkpoint category, checkpoint sequence, checkpoint age, health/staleness category, and the existing separation between checkpoint visibility and unproved task-watchdog participation.
- **D-07:** Construct an explicit typed join from each admitted sensor and runtime-health fact to its operator-snapshot boot session and revision. Mixed or absent provenance is unrepresentable after parsing.
- **D-08:** Snapshot substance and runtime-health evidence remain distinct claim types. Neither can be manufactured from generic root eligibility or reused to authenticate the other.
- **D-09:** Prefer a full typed substantive fact bundle that an independent reviewer can reconstruct. Use a versioned canonical digest only as a bounded representation detail when retained capacity requires it, with exhaustive field-mutation tests proving completeness.

### Independently observed runtime identity

- **D-10:** First attempt read-only replay of Attempt 31's immutable protected device-session request, event ledger, private result, and redacted projection. Admission, not the supervisor, reconstructs the same-device Boot B join.
- **D-11:** The reconstructed fact must independently correlate observed source commit, pinned reference commit, application ELF digest, boot/session identity, and same physical device with the exact admitted manifest, executable, factory image, and package.
- **D-12:** If the complete ledger is unavailable, a typed private terminal-result plus public-projection fallback may authenticate only the independently observed fields whose provenance remains complete. Derived booleans alone do not gain broader authority.
- **D-13:** If neither immutable path closes the join, return `immutable_artifacts_insufficient`; do not rebuild runtime identity from manifest fields or claim observation from package identity.

### Independent no-actuation proof

- **D-14:** Run a privacy-preserving retrospective completeness classifier over immutable Attempt 31 artifacts before designing a new hardware attempt. Re-admit no-actuation only when every effect-capable path and the complete bounded interval are independently and immutably observable.
- **D-15:** The existing supervisor-authored Boolean, its digest, generic chronology, source guards, and cleanup success cannot independently authenticate no actuation.
- **D-16:** If Attempt 31 is insufficient, a future authorized run must use an exclusive capability-enforced effect broker that owns an append-only typed ledger. The ledger records each permitted setup flash/probe, passive observation, hostname read/PATCH/restoration, exactly-once approved reboot, and cleanup transition before execution.
- **D-17:** Closed effect types must make prohibited active-control, self-test, watchdog-intervention, mining, credential, OTA/recovery, and other out-of-scope actions unrepresentable. Structural source, runfiles, and real-process guards must prove the supervisor cannot bypass the broker through direct command or transport paths.
- **D-18:** Unknown, missing, duplicate, out-of-order, unclosed, or bypassed operations fail the no-actuation claim closed.

### Claim-specific correction and re-promotion

- **D-19:** Replace the Phase 35 "eligible root promotes all four rows" rule with common root prerequisites plus explicit claim-specific prerequisites.
- **D-20:** The hostname row requires the confirmed storage, reload, reboot, restoration, and cleanup facts already established by Attempt 31.
- **D-21:** The package-identity row requires the independently observed runtime-identity and exact-package join.
- **D-22:** The coherent-snapshot row requires admitted sensor substance plus the typed boot/revision provenance join.
- **D-23:** The runtime-health row requires admitted health and checkpoint substance.
- **D-24:** Every row that claims passive operation without actuation also requires independently admitted no-actuation proof.
- **D-25:** Each decision records the digest of its exact admitted claim facts. The matrix supports typed `promote`, `demote`, `preserve`, and `do_not_promote` outcomes; it must not require four positive promotions.
- **D-26:** Reuse the existing staged validation, rollback, durability, and atomic exchange mechanics so affected corrections and promotions publish together while every unrelated checklist row and all exact non-claims remain byte-identical.
- **D-27:** A new exact-current-source hardware run is eligible only after the hermetic successor validator emits the closed `immutable_artifacts_insufficient` result and a later Phase 36 plan explicitly documents authority, recovery, exact effects, evidence, and the progress-gated attempt contract.

### Reconciliation timing

- **D-28:** Reconcile Phase 34 Nyquist rows, `.planning/PROJECT.md`, `.planning/STATE.md`, requirement traceability, and the milestone audit only after substantive admission, claim-specific decisions, redaction, and fresh independent verification pass.
- **D-29:** The milestone audit remains `gaps_found` and SYS-02, EVD-11, EVD-12, EVD-14, and EVD-15 remain unclosed until that verification.

### the agent's Discretion

- Exact Phase 36 Rust module, type, schema, artifact-role, and CLI names.
- Exact canonical encoding for substantive fact bundles, provided every required field is typed, reviewable, versioned, and exhaustively mutation-tested.
- Exact supersession metadata and atomic-generation directory layout.
- Exact implementation language of the future effect broker, provided it is repository-owned, exclusive, independently ledgered, privacy-preserving, and structurally non-bypassable.

</decisions>

<canonical-refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase and audit contract

- `.planning/PROJECT.md` — v1.2 operator-ready boundary, safety limits, and evidence principles.
- `.planning/ROADMAP.md` — Phase 36 goal, success criteria, and hardware-rerun gate.
- `.planning/REQUIREMENTS.md` — SYS-02, EVD-11, EVD-12, EVD-14, EVD-15, and exact out-of-scope claims.
- `.planning/v1.2-MILESTONE-AUDIT.md` — authoritative admission gaps, non-claims, Nyquist debt, and re-evaluation recommendation.
- `AGENTS.md` — evidence privacy, detector, hardware attempt, direct-UART/pin, timeout, and archived-lineage rules.

### Upstream decisions and immutable evidence history

- `.planning/phases/31-operator-claim-and-telemetry-contract/31-CONTEXT.md` — closed claim and observation semantics.
- `.planning/phases/34-provenance-runtime-health-and-coherent-operator-snapshot/34-CONTEXT.md` — canonical runtime identity, coherent snapshot, and passive health contracts.
- `.planning/phases/34-provenance-runtime-health-and-coherent-operator-snapshot/34-VERIFICATION.md` — final Phase 34 software verification and exact remaining scope.
- `.planning/phases/34-provenance-runtime-health-and-coherent-operator-snapshot/34-VALIDATION.md` — Nyquist rows that require later reconciliation.
- `.planning/phases/35-detector-gated-correlated-evidence-and-exact-parity-promotion/35-CONTEXT.md` — Attempt 31 immutability, exact four-row scope, and non-claims.
- `.planning/phases/35-detector-gated-correlated-evidence-and-exact-parity-promotion/35-04-SUMMARY.md` — final Phase 35 admission and publication mechanics.
- `.planning/phases/35-detector-gated-correlated-evidence-and-exact-parity-promotion/35-VERIFICATION.md` — historical Phase 35 phase-level result that the milestone audit supersedes for integration truth.

### Evidence, session, and retry policy

- `docs/parity/evidence-policy.md` — `NeverPersistRaw`, protected operational artifacts, immutable classifier inputs, and public projections.
- `docs/hardware/esp-device-session.md` — qualified transport responsibilities and same-device application identity quorum.
- `docs/hardware/hardware-attempt-policy.md` — progress-gated continuation and exact-current-HEAD attempt invariants.

### Existing implementation

- `tools/parity/src/operator_snapshot_evidence.rs` — existing identity/revision/retained-membership validator to preserve and compose.
- `tools/parity/src/phase35_evidence.rs` — current closed 14-role evidence admission.
- `tools/parity/src/phase35_evidence/contract.rs` — generic admission facts that need a typed successor.
- `tools/parity/src/phase35_evidence/inventory.rs` — current role/digest contract.
- `tools/parity/src/phase35_evidence/projection.rs` — current public projection boundary.
- `tools/parity/src/phase35_promotion/types.rs` — closed claim scopes and decisions.
- `tools/parity/src/phase35_promotion/evaluator.rs` — current generic-root four-row promotion rule.
- `tools/parity/src/operator_evidence/generation/phase35.rs` — staged validation, rollback, durability, and atomic exchange.
- `tools/device-session/src/model.rs` — independently observed Boot B and same-device result model.
- `scripts/phase35-correlated-evidence-root.sh` — package-derived runtime identity construction.
- `scripts/phase35-correlated-evidence-document.sh` — current document and no-actuation artifact construction.
- `scripts/phase35-correlated-evidence-effects.sh` — bounded Phase 35 effect sequence and capture boundary.
- `docs/parity/checklist.md` — exact affected rows and all unrelated rows that must remain stable.

### Engineering standards

- `standards/core/architecture.md` — functional core, typed boundaries, and illegal-state modeling.
- `standards/core/code-shape.md` — shallow control flow, script diagnostics, rerun safety, and module size.
- `standards/core/testing.md` — one-concern behavior tests with Arrange/Act/Assert.
- `standards/core/verification.md` — repository-native verification and clean commit gate.
- `standards/languages/rust.md` — Rust domain types, adapters, optional naming, and module layout.

</canonical-refs>

<code-context>
## Existing Code Insights

### Reusable Assets

- `operator_snapshot_evidence`: reuse the strict boot-session, revision chronology, and retained-membership validator unchanged, then compose substantive parsers around it.
- `phase35_evidence`: reuse digest, ordered inventory, regular-file, ownership, symlink, chronology, and redaction guards as lower-level primitives.
- Phase 31/32 observation types: reuse the existing truth-state, stamp, sequence, and independent sensor failure semantics instead of re-encoding primitives.
- Phase 34 runtime-health types and serializers: reuse the closed lifecycle/checkpoint/watchdog vocabulary.
- `tools/device-session`: replay its immutable observed-event/result model for runtime identity rather than trusting package-derived JSON.
- `phase35_promotion`: reuse closed scope enums, checklist preservation, and deterministic non-promotion vocabulary while replacing generic eligibility.
- `operator_evidence/generation/phase35.rs`: reuse the rollback-capable staged publisher and atomic checklist exchange.
- Existing Phase 35 invalid fixtures and real-process tests: extend with one-field mutation, missing-role, supersession, insufficiency, bypass, and partial-publication cases.

### Established Patterns

- Pure Rust types and reducers own admission and promotion; Bash and device adapters remain thin imperative shells.
- Protected artifacts remain mode `0600` below mode `0700` ignored roots and are never copied into committed generations.
- Private classifiers consume immutable secret-sanitized inputs before a distinct commit-redacted projection is produced.
- Green tests, lifecycle completion, source guards, and supervisor declarations do not authenticate parity facts without eligible evidence.
- Hardware continuation requires newly verified information and never follows automatically from validator failure.

### Integration Points

- Add a Phase 36 successor parser and root-supersession resolver beside the Phase 35 modules rather than silently changing v1 semantics.
- Add substantive sensor/health domain parsers around the existing operator-snapshot validator.
- Add read-only device-session replay and legacy effect-completeness classifiers for Attempt 31 sufficiency.
- Replace the four-row completeness guard with a claim-prerequisite matrix and atomic correction/re-promotion publisher.
- Add a future broker-backed hardware path only if the offline result and later plan explicitly authorize it.
- Run `just verify-redaction` before any evidence-related commit and use fresh GSD verification as the completion authority.

</code-context>

<specifics>
## Specific Ideas

- Treat Attempt 31 as immutable historical input to a new decision, never as a schema that Phase 36 is free to rewrite.
- Make every promoted row cite a digest of its own admitted facts so an independent reviewer can reconstruct why that row changed.
- Name the exact insufficiency boundary explicitly; "validator rejected" is too coarse to authorize hardware.
- Produce the corrected decision matrix and checklist update in one atomic generation, with rollback proving that no transient partial correction can become current.

</specifics>

<deferred>
## Deferred Ideas

- On-device running-partition hashing is an escalation option for a future authorized evidence run only if immutable Attempt 31 runtime-identity artifacts are insufficient.
- Firmware-owned actuator counters are optional defense in depth for a future evidence architecture, not a substitute for the Phase 36 retrospective classifier or exclusive host effect ledger.
- A reusable cross-platform host observer and a general all-Rust HIL runner remain later tooling work.

</deferred>

***

*Phase: 36-substantive-evidence-admission-and-exact-re-promotion*
*Context gathered: 2026-07-23*

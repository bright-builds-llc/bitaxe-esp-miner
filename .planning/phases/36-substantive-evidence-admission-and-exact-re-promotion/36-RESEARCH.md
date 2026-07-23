---
phase: 36
slug: substantive-evidence-admission-and-exact-re-promotion
status: complete
researched: 2026-07-23
phase_lifecycle_id: 36-2026-07-23T15-20-53
lifecycle_mode: yolo
---

# Phase 36 Research: Substantive Evidence Admission and Exact Re-Promotion

## Research Question

How can Phase 36 close SYS-02, EVD-11, EVD-12, EVD-14, and EVD-15 without rewriting Attempt 31, inferring missing facts, exposing protected evidence, or automatically consuming another hardware attempt?

## Recommended Architecture

Build a versioned `phase36-evidence-v1` successor around four pure typed decisions and one thin protected-input shell:

1. A successor envelope fingerprints the immutable Phase 35 root and records separate evidence-source and evaluator commits.
2. A substantive snapshot parser composes the unchanged identity/revision validator with strict sensor and runtime-health domain parsers.
3. A device-session replay reducer independently reconstructs the same-device Boot B package join from immutable observations.
4. A legacy effect-completeness reducer classifies whether Attempt 31 independently covers the complete bounded no-actuation interval.
5. A claim-specific evaluator emits `promote`, `demote`, `preserve`, or `do_not_promote` per row and feeds the existing rollback-capable atomic publisher.

The shell may open only explicitly supplied protected files, snapshot them immutably, and pass bytes into the pure reducers. It must not discover targets, read credentials, invoke hardware, or render protected values.

## Attempt 31 Sufficiency Boundaries

Committed shareable Phase 35 artifacts are sufficient to identify the historical root and its four decisions, but not to authenticate the three reopened substantive boundaries:

- The public snapshot projection contains session/revision/count facts rather than sensor values, truth states, stamps, failure reasons, and full health/checkpoint facts.
- `runtime-identity.json` is package-derived; the independently observed device-session result was not an inventory role.
- `no_actuation_verified=true` was authored by the supervisor and is not an independent ledger.

Phase 36 must therefore implement safe offline classifiers before deciding eligibility:

| Boundary | Eligible immutable input | Closed insufficiency result |
| --- | --- | --- |
| Snapshot/health | Exact API, WebSocket, and retained documents with complete typed fields plus a provable sensor-to-snapshot boot/revision join | `snapshot_substance_insufficient` or `runtime_health_insufficient` |
| Runtime identity | Protected device-session request/event ledger/result plus public projection, or a narrower paired result/projection that still proves the exact observed fields | `runtime_identity_observation_insufficient` |
| No actuation | Independently owned artifact set that exhaustively covers every effect-capable path and the full bounded interval | `independent_effect_observation_insufficient` |

The classifier may inspect only paths explicitly provided beneath an already protected root and may output only closed categories, counts, booleans, and digests. If protected artifacts are absent or cannot be safely admitted, the result is insufficiency—not a request to search other local locations.

## Substantive Fact Model

For each captured projection, parse:

- boot session and operator snapshot revision;
- power current, bus voltage, and wattage from one atomic INA260 observation;
- temperature and tachometer as independent observations;
- state: `fresh`, `stale`, `unavailable`, or `fault`;
- legal numeric value presence for the state;
- producer sequence, monotonic acquisition stamp, and typed reason;
- runtime-health lifecycle state, supervisor availability, checkpoint category, sequence, age, and health/staleness category;
- task-watchdog participation as independently available or `unproved`.

Construct the sensor/runtime-to-operator provenance join as a domain type. Raw unrelated session identifiers, missing revisions, mixed revisions, reused stamps, illegal numeric values, and contradictory states must fail during parsing.

## Runtime Identity Replay

Reuse `tools/device-session/src/model.rs` observation and result vocabulary. Replay must prove:

- the same admitted physical device before and after reboot;
- Boot B source commit equals the expected evidence-source commit;
- Boot B reference commit equals the pinned clean reference;
- Boot B `app_elf_sha256` equals the admitted firmware ELF/package relation;
- the admitted manifest, executable, factory image, and package digests remain exact;
- the redacted projection agrees with the private replay result without gaining additional authority.

Do not treat package equality as runtime observation or a terminal Boolean as proof of its observation chain.

## Independent Effect Proof

Retrospective admission is permitted only when immutable legacy artifacts prove complete independent coverage. The current supervisor Boolean and chronology do not qualify.

If the retrospective reducer returns `independent_effect_observation_insufficient`, a future hardware plan may introduce an exclusive effect broker. Its closed effect vocabulary should include only package probe/flash, passive observation, hostname read, hostname PATCH, hostname restoration, exactly-once approved reboot, and cleanup. The broker owns an append-only ledger and records authorization, invocation, result, and closure for each sequence. Structural and real-process tests must reject direct `curl`, flash, serial, device-session, or other bypass paths.

## Claim-Specific Promotion

Common prerequisites cover root integrity, reference cleanliness, current evaluator identity, redaction, restoration, cleanup, and atomic publication. Claim prerequisites remain separate:

| Claim | Required admitted facts |
| --- | --- |
| Hostname durability | Existing confirmed storage, reboot, restoration, and cleanup chain |
| Package/runtime identity | Independently replayed runtime identity plus exact package join |
| Coherent snapshot | Typed sensor substance plus exact boot/revision provenance join |
| Passive runtime health | Typed health/checkpoint substance |

Every passive-without-actuation claim also requires independent effect proof. Each decision carries its claim-fact digest. Unsupported rows demote atomically; the evaluator must not require four promotions.

## Security and Privacy Threats

- **T36-01 protected-value disclosure (high):** raw paths, device identities, endpoints, document bodies, and secret-bearing values escape through CLI output, fixtures, errors, logs, or public generations.
- **T36-02 evidence rewriting/splicing (high):** the successor mutates Attempt 31 or combines unrelated evidence.
- **T36-03 supervisor self-attestation (high):** package fields or supervisor booleans are accepted as independent runtime/no-actuation observations.
- **T36-04 broker bypass (high):** a future supervisor reaches an effect-capable command outside the exclusive ledger.
- **T36-05 partial checklist correction (high):** matrix and checklist diverge during a failed publication.
- **T36-06 unauthorized hardware fallback (high):** classifier insufficiency directly triggers device work.

Mitigate these through immutable snapshots, closed role enums, typed insufficiency, protected-input/public-output separation, exhaustive mutation fixtures, structural bypass guards, rollback tests, and a plan-level manual authorization boundary for any future attempt.

## Integration Points

- `tools/parity/src/operator_snapshot_evidence.rs`: preserve identity validation; add composition points for substantive parsing.
- `tools/parity/src/phase35_evidence*`: reuse digests, path safety, inventory, and projections without changing v1 semantics.
- `tools/parity/src/phase35_promotion*`: reuse closed scopes and checklist preservation; replace four-positive completeness in a Phase 36 successor.
- `tools/device-session/src/model.rs`: reuse observed runtime identity vocabulary and reducer rules.
- `tools/parity/src/operator_evidence/generation/phase35.rs`: reuse staged exchange, rollback, and durability behavior.
- `crates/bitaxe-safety/src/observation.rs`, `crates/bitaxe-safety/src/sensor_acquisition.rs`, and Phase 34 health models: source typed truth semantics.
- `docs/parity/checklist.md`, `.planning/PROJECT.md`, `.planning/REQUIREMENTS.md`, `.planning/STATE.md`, `.planning/v1.2-MILESTONE-AUDIT.md`, and Phase 34 validation metadata: reconcile only after fresh verification.

## Implementation Order

1. Wave 0: build fixture factories, mutation matrices, privacy tests, and validator command surfaces.
2. Implement the successor envelope, substantive facts, runtime replay, and legacy effect sufficiency reducers.
3. Implement claim-specific correction and atomic successor publication.
4. Run safe offline Attempt 31 classification using explicit protected inputs only when locally available; otherwise record the exact insufficiency categories.
5. If and only if the result is insufficient, stop Phase 36 hardware work. A separate later plan would be required to authorize and implement the broker-backed run.
6. Reconcile planning/docs only after verification passes.

## Validation Architecture

### Test Layers

| Layer | Purpose | Command |
| --- | --- | --- |
| Pure unit | Parse sensor/health facts, joins, identity replay, effect completeness, and claim prerequisites | `cargo test -p bitaxe-parity phase36` |
| Mutation fixtures | Reject every missing, extra, contradictory, mixed-session, self-attested, bypass, and partial-publication case | `bazel test //tools/parity:phase36_evidence_tests` |
| Real-process privacy | Prove protected inputs never reach terminal or shareable files and explicit-path admission fails closed | `bazel test //scripts:phase36_evidence_test` |
| Redaction | Scan staged/committed evidence surfaces | `just verify-redaction` |
| Repository regression | Preserve parity/reference and existing phase behavior | `just parity && just verify-reference && just test` |
| Rust pre-commit | Required repository sequence | `cargo fmt --all && cargo clippy --all-targets --all-features -- -D warnings && cargo build --all-targets --all-features && cargo test --all-features` |

### Requirement Coverage

| Requirement | Automated proof |
| --- | --- |
| SYS-02 | Device-session replay mutation matrix and exact package join |
| EVD-11 | Successor inventory, immutable supersession, same-device runtime observation |
| EVD-12 | Sensor/health substantive field and provenance-join mutation matrices |
| EVD-14 | Independent effect completeness, privacy, cleanup, reference, and atomicity gates |
| EVD-15 | Per-claim prerequisite matrix, deterministic demotion, and exact non-claim preservation |

### Wave 0 Gaps

- Add a Phase 36 fixture builder that can vary one field or role at a time without real hardware or protected values.
- Add a fake protected-root process harness proving explicit-path, permission, immutable-snapshot, and output-redaction behavior.
- Add atomic publisher failure injection for successor-generation/checklist rollback.
- Add exact requirement-to-test rows to `36-VALIDATION.md` before implementation.

### Manual-Only Boundary

No hardware verification is authorized by the base Phase 36 plans. If immutable evidence is insufficient, the automated result must stop at a typed category. Any later real-hardware plan requires explicit authorization and the full detector, recovery, evidence, timeout, privacy, and progress-gated attempt contract.

## Planning Recommendation

Use four plans: Wave 0 validation scaffolding; successor substantive/runtime/effect admission; claim-specific atomic correction and safe Attempt 31 classification; final verification and documentation reconciliation. Keep any broker-backed hardware attempt out of these plans unless an earlier plan produces the exact typed insufficiency result and a later explicitly authorized Phase 36 plan is added.

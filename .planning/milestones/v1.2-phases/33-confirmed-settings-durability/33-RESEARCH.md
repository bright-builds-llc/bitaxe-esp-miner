---
generated_by: gsd-phase-researcher
lifecycle_mode: yolo
phase_lifecycle_id: 33-2026-07-14T01-50-49
generated_at: 2026-07-14T02:09:48.349Z
phase: 33
requirements:
  - CFG-09
  - CFG-10
  - CFG-11
  - CFG-12
  - CFG-13
---

# Phase 33: Confirmed Settings Durability - Research

## Summary

Phase 33 should deepen the existing settings path rather than replace it. The pure API crate already has a closed hostname-only v1.2 authority decision and an ordered persistence executor; the firmware adapter already writes, commits, reloads, and stores an NVS snapshot. The missing guarantees sit at their boundary:

1. `handle_settings_patch` currently uses the broad compatibility plan directly, so any valid known setting can still reach NVS even though Phase 31 permits only exact hostname authority.
2. `SettingsPersistenceAdapter::reload` returns only `Result<()>`; the executor cannot inspect the actual reloaded hostname or prove it matches the typed request.
3. `FirmwareSettingsAdapter::reload` refreshes the shared snapshot as a side effect and silently skips individual read failures. It may therefore publish an incomplete snapshot before reconciliation.
4. After reload, the route calls `apply_persisted_settings_writes(plan.writes())`, overlaying requested values and making API visibility optimistic rather than storage-confirmed.
5. There is no transaction-wide writer serialization. Separate HTTP requests can interleave adapter operations and publication.
6. Immediate `/api/system/info` is already wired to `current_settings_snapshot()` through `runtime_snapshot::apply_settings_snapshot`, so fixing the authoritative store is sufficient for Phase 33 immediate readback without introducing Phase 34's global revision model.

The smallest robust architecture is one pure compatibility-and-authority decision feeding one serialized firmware transaction. The transaction writes the typed hostname, commits, independently reopens and reads NVS, builds a complete candidate snapshot, reconciles the typed hostname exactly, and publishes that candidate atomically immediately before success. The requested-write overlay should be removed from this path.

No new third-party dependency is needed.

## Recommended Architecture

### 1. One pure request decision

Keep the broad parser because it defines stable AxeOS response behavior, but separate validation from effect authority:

```text
raw body
  -> parse JSON
  -> validate every known field with existing compatibility rules
  -> classify exact v1.2 field set
       exact valid hostname -> Authorized(Hostname)
       valid unknown/unsupported/empty/mixed -> CompatibilityOnly(category)
       malformed/non-object/invalid known -> generic public error
```

This ordering matters. `decide_v12_settings_value` currently classifies a multi-field object before validating its known values. Phase 33 needs a combined pure decision or explicit broad-validation-first composition so an invalid known value inside a mixed request still returns `Wrong API input`, while a fully valid mixed request remains an empty-success no-op.

The authorized branch should carry the existing validated `bitaxe_api::v12_settings::Hostname` newtype. Do not unwrap it back into an unchecked `String` until the adapter boundary.

### 2. One serialized confirmation transaction

Add a process-lifetime transaction lock owned by the settings adapter or a narrow settings service. The lock must cover storage mutation through confirmed publication:

```text
acquire transaction lock
  -> open read/write NVS namespace
  -> write hostname (including same-value requests)
  -> commit
  -> independently reopen read-only namespace
  -> read a complete candidate NVS snapshot with fallible errors
  -> load and parse candidate hostname into the typed domain value
  -> exact-match requested and reloaded hostname
  -> atomically replace the current confirmed snapshot
release lock
  -> schedule empty success response
  -> schedule best-effort live netif hostname effect
```

Readers do not need the transaction lock if the store replacement itself stays atomic under the existing snapshot mutex. They continue seeing the last confirmed snapshot until the candidate is published.

The current `EspDefaultNvsPartition::take()` lifecycle and namespace handle opening belong in this imperative shell. The pure crate should model ordering and outcomes, not ESP-IDF handles or mutexes.

### 3. Reload must return evidence, not mutate first

Change the persistence adapter contract so reload produces a candidate snapshot or a typed confirmation input. The executor must be able to verify:

- a fresh read-only namespace was opened after commit;
- the hostname key read succeeded with the expected storage type;
- schema loading produced a valid `Hostname`;
- the reloaded typed value exactly equals the request;
- publication completed before `PublicSuccess` is appended.

Do not reuse `refresh_current_settings_snapshot` as-is: it publishes during reload and converts per-key read failures into skipped values. Introduce a fallible candidate builder, then publish only after reconciliation. Startup loading may keep best-effort behavior if clearly separate from the authoritative PATCH transaction, but successful PATCH confirmation may not.

### 4. Post-commit uncertainty is explicit

Invalid input is rejected before effects, satisfying CFG-09. For a valid request, failures before commit can abandon the uncommitted handle. A commit can succeed even if later reload, reconciliation, or publication fails. Phase 33 must not promise compensating rollback or claim that storage stayed unchanged after that point.

Use a typed failure taxonomy that distinguishes validation, write, commit, reload, reconciliation, and publication for retained diagnostics while mapping them all to the generic public error. If confirmation is lost after commit, retain the previous confirmed public snapshot or expose an explicit unconfirmed state internally until a later independent reload succeeds. Never overlay the request.

### 5. Immediate API truth comes from the confirmed store

`runtime_snapshot::apply_settings_snapshot` already reloads the in-memory `NvsSnapshot` and projects `LoadedValue::Str(hostname)` into `snapshot.platform.hostname`. It should continue to consume only the confirmed store. After the transaction publishes the reloaded candidate, immediate system-info and existing settings-backed snapshot projections see the same value.

Do not add Phase 34 boot-session/global-revision semantics here. Phase 33 supplies a storage-confirmed hostname input that Phase 34 can later compose.

### 6. Live hostname application stays best effort

The current netif hostname effect is intentionally post-persistence and best effort. Preserve that boundary, but schedule it only after confirmed publication. It may affect networking behavior, yet it cannot authenticate storage truth, determine HTTP success, or mutate the confirmed snapshot.

## Exact Compatibility Semantics

| Input | Public response | Storage/publication/effects |
| --- | --- | --- |
| Malformed JSON or non-object | Existing `Invalid JSON` error | None |
| Any invalid known field, including invalid hostname in a mixed request | Existing `Wrong API input` error | None |
| Exact valid hostname-only object | Empty success only after confirmed transaction | One serialized hostname write/commit/reload/reconcile/publish, then best-effort live hostname |
| Empty object | Existing empty success | None |
| Valid unknown-only object | Existing empty success | None |
| Valid unsupported-known-only object | Existing empty success | None |
| Fully valid mixed object, including hostname plus another field | Existing empty success | None; hostname cannot be extracted |

Compatibility-only diagnostics should record stable categories and counts, never raw request bodies or values. Credential, hardware-control, mining, and self-test categories remain ineligible even when syntactically valid.

## File Touchpoints

| File | Expected role |
| --- | --- |
| `crates/bitaxe-api/src/v12_settings.rs` | Compose broad validation with exact authority so mixed invalid-known input errors while valid mixed input remains compatibility-only. |
| `crates/bitaxe-api/src/settings.rs` | Deepen the ordered executor with reload evidence, typed reconciliation, publication, and exhaustive failure-step tests; narrow or replace broad write plans on the v1.2 path. |
| `crates/bitaxe-config/src/persistence.rs` | Reuse pure snapshot/load behavior; add only narrow typed helpers if needed for fallible hostname reconciliation. |
| `firmware/bitaxe/src/settings_adapter.rs` | Own serialization, fallible independent reload, candidate construction, exact publication, and ESP-IDF errors. Remove requested-write overlay participation from the confirmed path. |
| `firmware/bitaxe/src/http_api.rs` | Route exact authority and compatibility no-ops correctly; respond only after confirmation; preserve post-response live effect and restart route. |
| `firmware/bitaxe/src/runtime_snapshot.rs` | Prove immediate system-info reads the confirmed store and does not project request overlays. |
| `scripts/serial-session-trace.sh` | Reuse stable physical-identity, readiness, ownership, trace, and cleanup primitives for Phase 33 hardware proof. |
| `scripts/phase13-monitor-capture.sh` | Reuse only the generic passive ESP32-S3 command/session pattern; do not invoke the archived Phase 28.1.1 lifecycle or revive its evidence claims. |
| `scripts/detect-ultra205.sh` | Required one-time preflight outside the exactly-one-reboot proof interval. |

## Security Threat Model Inputs

Plans should include ASVS Level 1 threat blocks and block on high-severity threats.

| Threat | Risk | Required mitigation |
| --- | --- | --- |
| T-33-01 authority confusion | Broad known settings or a mixed request reaches NVS | Closed exact hostname authority after full compatibility validation; exhaustive schema-field tests. |
| T-33-02 optimistic publication | Requested value is visible although reload failed or mismatched | Candidate reload and typed match before atomic publication; delete overlay path. |
| T-33-03 concurrent lost update | Two PATCH requests interleave writes/reloads/publication | One transaction lock through publication; concurrency regression with deterministic fake adapter. |
| T-33-04 secret disclosure | Request bodies, credentials, origins, or identifiers enter logs/evidence | Category-only logs; denylist tests; protected local traces and redacted shareable summaries. |
| T-33-05 false rollback claim | Commit succeeds but later confirmation fails | Typed post-commit uncertainty; no compensating commit and no unchanged-storage assertion. |
| T-33-06 false same-board proof | HTTP recovery or a tty path is mistaken for physical identity | Preflight detector plus stable physical-USB digest excluding enumeration fields. |
| T-33-07 extra-reset contamination | Post-reboot board-info or monitor reset adds another reset | Detector outside proof interval; full no-reset monitor flags; fail on any extra reset category. |
| T-33-08 scope escape | Evidence helper invokes raw power/reset, direct UART/pins, credentials, archived lineage, or parity promotion | Closed command/source guard and explicit non-promotion conclusion. |

## Hardware Durability Proof

CFG-12 requires real-device evidence because host tests cannot prove ESP-IDF NVS survives reboot. The approved proof should be a new Phase 33 repo-owned wrapper or a narrow reusable helper, not an invocation of archived Phase 28.1.1 workflows.

### Proof boundary

1. Run `just detect-ultra205` and accept exactly one board `205`. Board-info is reset-capable, so complete it before the proof interval.
2. Wait for the selected device to return to a settled runtime. Record the selected physical-USB identity digest and source/reference/package identities in protected local state.
3. Use fresh monitor output from this same session to derive exactly one origin-only `DEVICE_URL`; do not use stale logs, scans, mDNS, ARP, or router state.
4. Generate a non-secret hostname for the attempt, PATCH it, require confirmed success, and immediately GET `/api/system/info` to prove equality.
5. Arm a passive owner before restart using `--chip esp32s3 --before no-reset-no-sync --after no-reset --no-reset --non-interactive`.
6. Invoke the existing access-gated `POST /api/system/restart`; its response must precede `sys::esp_restart()`.
7. Observe service loss/recovery and a fresh origin only from the same current monitor session, while keeping physical identity stable across any new enumeration epoch.
8. GET `/api/system/info` and prove the hostname digest equals the immediate pre-reboot digest.
9. Reap the process tree and prove zero unexpected serial holders. Produce a redacted conclusion with no parity-row updates.

Use at least a 360-second capture budget and a shell/tool wall clock of at least 420 seconds. Fail closed on ambiguous origins, identity change, unexpected holders, incomplete cleanup, extra reset, or missing/mismatched readback.

## Common Pitfalls

1. Calling `plan_settings_patch_body` and persisting its broad `writes` without consulting `V12SettingsDecision`.
2. Classifying a mixed request before validating known values, causing an invalid known field to become a silent no-op.
3. Publishing from inside `reload` before exact reconciliation.
4. Treating a missing hostname in a silently partial snapshot as the default `bitaxe` and then reporting success.
5. Reapplying planned writes after reload, which recreates optimistic truth.
6. Locking only the NVS handle but releasing before publication, allowing a later writer's reload to race an earlier publication.
7. Skipping write/commit for same-value requests despite the locked uniform proof-chain decision.
8. Attempting compensating rollback after commit and claiming it cannot fail.
9. Using a post-reboot `just detect-ultra205`, which adds another reset inside the proof.
10. Treating tty path, IORegistry entry ID, IP address, or `DEVICE_URL` as stable board identity.
11. Reusing archived external-UART or Phase 28.1.1 workflows because they contain generic session helpers.
12. Promoting CFG rows directly from Phase 33; Phase 35 owns final correlated admission.

## Suggested Plan Decomposition

1. **Pure authority and confirmation core:** full compatibility matrix, exact hostname authority, transaction/reconciliation state machine, concurrency/failure injection, and redaction-safe diagnostics.
2. **Firmware NVS and API integration:** serialized ESP-IDF adapter, fallible candidate reload, atomic confirmed store, immediate system-info projection, and removal of requested-write overlays.
3. **Detector-gated durability harness and hardware proof:** generic passive-session reuse, exactly-one-application-reboot wrapper, redacted evidence, software simulation/source guards, then one approved Ultra 205 run.

## Validation Architecture

### Test layers

| Layer | Purpose | Fast command |
| --- | --- | --- |
| Pure authority unit | Prove every body/field-set class, exact hostname authority, public error mapping, and zero-effect compatibility outcomes. | `cargo test -p bitaxe-api v12_settings` and targeted settings tests. |
| Transaction state machine | Inject validation/write/commit/reload/reconcile/publish failures and concurrent writers; prove success ordering and no optimistic visibility. | `cargo test -p bitaxe-api settings` plus new deterministic fake-adapter tests. |
| Config persistence | Prove typed hostname reload and exact-match/mismatch behavior from snapshots. | `cargo test -p bitaxe-config persistence`. |
| Firmware source/integration | Prove handler routing, adapter serialization, candidate-before-publication, immediate system-info readback, and absence of the overlay call. | Affected firmware-host tests and source guards under repo Bazel targets. |
| Evidence-wrapper simulation | Fake detector, serial ownership, origin, HTTP, re-enumeration, timeout, redaction, and cleanup outcomes without hardware. | New direct shell test and corresponding `bazel test //scripts:<phase33-target>`. |
| Full host regression | Protect all Rust and script behavior. | `cargo test --all-features` plus affected Bazel tests. |
| Firmware build/package | Prove ESP-IDF NVS, HTTP, restart, and tool integration. | Repo-owned canonical `just build` / Bazel firmware target and package verification. |
| Hardware durability | Prove confirmed hostname survives one application restart on the same detector-gated board. | New Phase 33 command beginning with `just detect-ultra205`, using ≥360-second passive capture and ≥420-second wall clock. |

### Required regression cases

- Exact valid hostname is the only input that can invoke write, commit, reload, reconcile, publish, or live hostname effect.
- Malformed/non-object input maps to `Invalid JSON`; every invalid known value maps to `Wrong API input` with zero adapter calls.
- Empty, unknown-only, unsupported-known-only, and valid mixed requests return empty success with zero adapter calls.
- Invalid known values inside mixed input still fail generically rather than becoming compatibility-only success.
- Same-value hostname requests execute write, commit, reload, reconcile, and publication.
- Step order is exact and public success is absent after every injected failure.
- Reload mismatch and missing/wrong-typed hostname fail reconciliation and do not replace the prior confirmed snapshot.
- Commit-then-reload failure records post-commit uncertainty without a rollback claim.
- Two concurrent authorized requests cannot interleave; readers see only the old or one complete confirmed snapshot.
- Immediate `/api/system/info` reads the reloaded confirmed hostname and the source contains no requested-write overlay call.
- Diagnostics never render raw credential values, request bodies, protected origins, or device identifiers.
- Evidence simulation rejects extra resets, second detector runs, identity changes, ambiguous origins, missing readback, holder leaks, and incomplete cleanup.
- The real-device path uses the full passive command, minimum timeouts, one application restart, stable physical identity, and redacted non-promotional output.

### Per-task sampling

- After every code task, run the narrowest affected crate or script test.
- After every plan, run all affected Bazel targets and `git diff --check`.
- Before each commit in this Rust repository, run `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, and `cargo test --all-features` in that order.
- Before phase verification, run the repo-owned full check/build/package/reference gates selected by the planner.
- No three consecutive tasks may lack an automated behavior check.

### Hardware gate

Software simulation, source guards, firmware build, and package verification must pass before hardware use. Then run `just detect-ultra205`; proceed only for exactly one validated board `205`. Use only the provided USB and barrel-power interfaces, do not read local credential files unless the repo-owned command requires them, and never print or commit their contents. The proof permits exactly one application restart and no flash, raw reset/power, direct UART/pins, OTA, fault injection, mining, archived lineage work, or final promotion.

## Sources

- `.planning/phases/33-confirmed-settings-durability/33-CONTEXT.md` — locked Phase 33 decisions and evidence boundary.
- `crates/bitaxe-api/src/v12_settings.rs` and `settings.rs` — current authority, compatibility, executor, and failure contracts.
- `crates/bitaxe-config/src/persistence.rs` — pure snapshot reload and typed loaded values.
- `firmware/bitaxe/src/settings_adapter.rs`, `http_api.rs`, and `runtime_snapshot.rs` — current NVS, PATCH, restart, and immediate projection paths.
- `scripts/detect-ultra205.sh`, `serial-session-trace.sh`, and `phase13-monitor-capture.sh` — reusable detector, identity, ownership, passive-monitor, timeout, and cleanup primitives; archived lineage execution remains prohibited.
- `reference/esp-miner/main/http_server/http_server.c`, `system_api_json.c`, and `nvs_config.c` — pinned upstream response and storage breadcrumbs.
- `AGENTS.md` and the local Bright Builds standards — hardware, security, redaction, architecture, testing, verification, and frontmatter requirements.

## RESEARCH COMPLETE

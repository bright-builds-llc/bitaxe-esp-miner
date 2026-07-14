---
generated_by: gsd-discuss-phase
lifecycle_mode: yolo
phase_lifecycle_id: 33-2026-07-14T01-50-49
generated_at: 2026-07-14T01:54:30.445Z
---

# Phase 33: Confirmed Settings Durability - Context

**Gathered:** 2026-07-14
**Status:** Ready for planning
**Mode:** Yolo

<domain>
## Phase Boundary

Phase 33 makes `hostname` the only effectful v1.2 settings PATCH and makes its public success mean serialized NVS write, commit, independent reload, typed reconciliation, atomic publication, immediate readback, and preservation across one approved normal reboot on the same detector-gated Ultra 205. It preserves existing compatibility responses for every other field without granting those inputs write authority or hardware effects. Phase 34 still owns the globally revisioned coherent operator snapshot, and Phase 35 still owns final correlated evidence admission and parity promotion.

</domain>

<decisions>
## Implementation Decisions

### Storage confirmation transaction

- **D-01:** Validate the complete request before opening the effectful NVS transaction. Invalid known hostname input returns the existing stable generic error and performs no NVS open-for-write, write, commit, live-state replacement, partial change, or hardware effect.
- **D-02:** Serialize authorized hostname writers from the first storage mutation through confirmed publication. Readers continue to observe the prior confirmed snapshot until one transaction completes; concurrent PATCH requests receive an unambiguous total order.
- **D-03:** Every authorized hostname request, including a same-value request, follows one uniform proof chain: write the typed hostname, commit, independently reopen and reload NVS, parse the reloaded hostname into the same domain type, reconcile it by exact equality with the requested value, then atomically publish the complete reloaded snapshot.
- **D-04:** The publication of the independently reloaded exact match is the success linearization point. The route may schedule its empty `200` response only after publication; it must not overlay planned writes or requested values onto the runtime snapshot.
- **D-05:** Any validation, write, commit, reload, reconciliation, or publication failure returns the generic public error and never schedules the live hostname effect. Before commit, abandon the uncommitted transaction. After a commit followed by confirmation failure, do not claim rollback or unchanged storage; preserve the last confirmed public truth or represent confirmation loss explicitly until storage can be independently reloaded.

### Compatibility and immediate publication

- **D-06:** Keep the broad AxeOS compatibility parser as the response shell, but make the Phase 31 closed `V12SettingsDecision` the only persistence-authority gate. Compatibility parsing must never be converted directly into NVS write authority.
- **D-07:** Malformed or non-object JSON preserves the existing `Invalid JSON` response. Any invalid known field preserves the existing generic `Wrong API input` response and fails atomically before effects, including when that invalid field appears in a mixed patch.
- **D-08:** Valid unknown-only, unsupported-known-only, empty, or mixed field sets preserve the existing empty-success compatibility response but perform no NVS write, commit, confirmed-state publication, live hostname effect, hardware effect, or broader v1.2 promotion. A mixed request cannot extract and persist its hostname.
- **D-09:** Only an exact, validated hostname-only field set enters the storage confirmation transaction. Credential-, hardware-control-, mining-, self-test-, and other excluded categories remain category-only in retained diagnostics, with no raw values or secrets.
- **D-10:** Immediate system-info and settings-backed runtime reads must consume the atomically published reloaded snapshot. The best-effort network-interface hostname application may run only after confirmed publication and must not determine PATCH success or API-visible storage truth.

### Normal reboot durability proof

- **D-11:** Define the approved normal reboot as the existing access-gated operator restart request completing its response and then reaching the firmware `RestartAfterResponse` / `sys::esp_restart()` effect. Flashing, later board-info, USB or barrel-power cycling, OTA, panic/watchdog/fault resets, raw reset/power commands, and archived diagnostic paths are excluded from the proof interval.
- **D-12:** Run `just detect-ultra205` once as preflight. Because detector board-info resets the device, place detector setup and subsequent runtime settling outside the proof interval. Lock the selected board with a stable physical-USB identity digest that excludes tty paths and enumeration-variant identifiers.
- **D-13:** Correlate one bounded proof in this order: detector and physical-identity lock; settled baseline; generated non-secret hostname PATCH; confirmed transaction success; immediate readback; passive monitor/session ownership; one application restart; service loss and recovery; a fresh origin derived only from the same current monitor session; unchanged physical identity; matching post-reboot hostname.
- **D-14:** Passive capture must use the complete ESP32-S3 no-reset contract (`--chip esp32s3 --before no-reset-no-sync --after no-reset --no-reset --non-interactive`), at least 360 seconds of capture and 420 seconds of wall-clock budget, bounded readiness/ownership checks, process-tree cleanup, and zero unexpected serial holders before reuse.
- **D-15:** Fail closed on an extra reset, identity change, zero or multiple fresh origins, unexpected ownership, incomplete cleanup, missing immediate or post-reboot readback, or hostname mismatch. Keep raw local identifiers and traces protected and gitignored; share only redacted categories, counts, durations, booleans, and value/identity/trace digests. Phase 33 proves durability only and does not update parity rows or perform final promotion.

### the agent's Discretion

- Exact type, lock, transaction, confirmation-state, and error variant names, provided they make the ordered transaction and post-commit uncertainty explicit.
- Whether the atomic confirmed snapshot lives in the existing settings store or a narrowed confirmed-settings store, provided every immediate consumer reads the same reloaded truth and requested-write overlays are impossible.
- Exact repo-owned Phase 33 evidence helper or narrow flash-tool extension, provided it enforces the detector, passive-session, same-board, timeout, redaction, and cleanup contract without reviving archived or direct-UART/pin paths.
- Exact category-only retained log labels and digest formats, provided they are stable, deterministic, and never expose settings values beyond the intentionally generated non-secret hostname proof.

</decisions>

<canonical_refs>

## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### v1.2 scope and locked decisions

- `.planning/ROADMAP.md` — Phase 33 boundary, dependency, goal, and success criteria.
- `.planning/PROJECT.md` — v1.2 operator-ready runtime goal, safety boundary, and deferred capabilities.
- `.planning/REQUIREMENTS.md` — CFG-09 through CFG-13 and later-phase ownership.
- `.planning/STATE.md` — hostname-only allowlist and active v1.2 exclusions.
- `.planning/phases/31-operator-claim-and-telemetry-contract/31-CONTEXT.md` — closed hostname-only authority, compatibility-shell separation, and exact-claim admission.
- `.planning/phases/32-shared-i2c-and-read-only-sensor-acquisition/32-CONTEXT.md` — current runtime ownership and read-only hardware boundaries that Phase 33 must preserve.

### Settings implementation and upstream behavior

- `crates/bitaxe-api/src/v12_settings.rs` — existing exact hostname authority and compatibility-only classifications.
- `crates/bitaxe-api/src/settings.rs` — current broad parser, persistence plan, ordered executor, public error mapping, and best-effort live effect.
- `crates/bitaxe-config/src/persistence.rs` — pure NVS snapshot, reload, and persistence decision model.
- `firmware/bitaxe/src/settings_adapter.rs` — ESP-IDF NVS write, commit, reload, and current runtime snapshot adapter.
- `firmware/bitaxe/src/http_api.rs` — current PATCH route, system-info route, and application restart effect.
- `firmware/bitaxe/src/runtime_snapshot.rs` — current runtime projection consumed by operator-facing API surfaces.
- `reference/esp-miner/main/http_server/http_server.c` — upstream PATCH compatibility response and live-hostname behavior breadcrumb.
- `reference/esp-miner/main/http_server/system_api_json.c` — upstream system-info hostname projection breadcrumb.
- `reference/esp-miner/main/nvs_config.c` — upstream NVS settings cache and persistence behavior breadcrumb.

### Repository constraints and standards

- `AGENTS.md` — detector gate, hardware authorization, passive serial contract, timeout, evidence redaction, archived-lineage prohibition, and no-direct-UART/pin constraints.
- `AGENTS.bright-builds.md` — required workflow, functional-core boundary, and clean verification rules.
- `standards/core/architecture.md` — functional core, imperative shell, boundary parsing, and illegal-state modeling.
- `standards/core/code-shape.md` — early returns, optional naming, and readability expectations.
- `standards/core/testing.md` — behavior-focused Arrange/Act/Assert unit-test requirements.
- `standards/core/verification.md` — sync, repo-owned verification, and clean commit gates.
- `standards/languages/rust.md` — Rust domain modeling, error handling, and verification guidance.

</canonical_refs>

<code_context>

## Existing Code Insights

### Reusable Assets

- `crates/bitaxe-api/src/v12_settings.rs` already makes exact hostname-only authority constructible and categorizes every compatibility-only field set.
- `crates/bitaxe-api/src/settings.rs` already models ordered validation, writes, commit, reload, public responses, and post-persistence effects; Phase 33 can deepen this contract with reconciliation and confirmed publication.
- `crates/bitaxe-config/src/persistence.rs` already provides pure snapshot reload and typed loaded values suitable for deterministic reconciliation tests.
- `firmware/bitaxe/src/settings_adapter.rs` already owns ESP-IDF NVS calls and a mutex-protected current snapshot, making it the natural imperative shell for serialized confirmation.
- `firmware/bitaxe/src/http_api.rs` already exposes the operator restart route and access gate needed for the approved normal-reboot proof.

### Established Patterns

- Pure crates own authority and state-transition decisions; firmware adapters own NVS, HTTP, clocks, restart, and serial effects.
- Public settings failures are generic while internal reasons and completed steps remain typed and redaction-safe.
- The current route already orders write → commit → reload, but it accepts broad known settings, does not reconcile the reloaded hostname, and applies a requested-write overlay after reload.
- Detector board-info is reset-capable, while retained-runtime evidence requires the full passive monitor contract and exact cleanup proof.

### Integration Points

- Join the v1.2 authority decision to a narrowed persistence transaction before the firmware adapter opens NVS for writing.
- Extend the pure executor with typed reload/reconciliation/publication outcomes and exhaustive failure-order tests.
- Replace `apply_persisted_settings_writes` on the authorized path with atomic publication of the independently reloaded snapshot.
- Project storage-confirmed hostname into immediate system-info/settings reads without pulling Phase 34's global snapshot revision into this phase.
- Add a repo-owned, detector-gated Phase 33 durability evidence path around the existing restart route and passive serial/session rules.

</code_context>

<specifics>
## Specific Ideas

- Treat the confirmed snapshot publication—not commit alone—as the public success point.
- Use one transaction shape for changed and same-value hostname requests so every `200` carries the same evidence semantics.
- Preserve empty-success compatibility no-ops, but log only stable exclusion categories so a no-op cannot be mistaken for a confirmed write.
- Correlate reboot durability with one generated non-secret hostname value digest rather than promoting raw local device or network identifiers.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

***

*Phase: 33-confirmed-settings-durability*
*Context gathered: 2026-07-14*

---
generated_by: gsd-phase-researcher
lifecycle_mode: yolo
phase_lifecycle_id: 31-2026-07-13T19-47-51
generated_at: 2026-07-13T20:05:00.000Z
---

# Phase 31: Operator Claim and Telemetry Contract - Research

## Research Summary

Phase 31 should establish two small pure contracts before later phases add I/O: an independently truthful observation model for OBS-01 and a closed hostname-only v1.2 settings/admission model for CFG-08. The current code already has useful pure safety and API mappers, but its boundaries are too broad for these requirements: power and thermal status are attached to numeric records that can still carry contradictory values, API telemetry aggregates all facts behind one status, `collect_api_snapshot()` collects safety state during a request, and the settings parser authorizes every known NVS schema field.

The simplest robust implementation is to add one generic state-carrying observation type with producer-owned stamps, adapt safety/API models to project legacy numerics separately, introduce a closed hostname-only capability behind the compatibility parser, and add row/capability-scoped v1.2 admission validation. No new dependency, hardware access, sensor read, settings write, reboot, credential access, archived-lineage work, or checklist promotion is needed.

## Current Architecture And Gaps

### Observation model

- `crates/bitaxe-safety/src/power.rs` uses `PowerObservationStatus` plus independent numeric fields. Constructors are available, but the shape can represent stale/fault/unavailable status alongside arbitrary current-looking numbers.
- `crates/bitaxe-safety/src/thermal.rs` similarly separates status from readings. Its pure decisions are reusable, but each reading needs independent truth because temperature and tachometer availability can diverge.
- `crates/bitaxe-api/src/snapshot.rs` and wire mappers use one aggregate `SafetyTelemetryStatus` for all power and thermal compatibility fields. This cannot express one failed sensor while unaffected facts remain fresh.
- Existing API fixtures already preserve zero numeric compatibility for unavailable data. Phase 31 should retain that wire behavior while adding explicit per-fact truth so zero has no freshness meaning.

### Producer ownership

- `firmware/bitaxe/src/runtime_snapshot.rs::collect_api_snapshot()` calls `safety_adapter::collect_safety_report()` while serving a snapshot. A consumer request can therefore participate in collection instead of reading an immutable producer-owned observation.
- `crates/bitaxe-stratum/src/v1/telemetry_projection.rs` already demonstrates producer-sequenced event folding and repeated-request tests. Reuse its ownership principle, not its mining-specific types.
- Phase 31 needs only the stamp and state-transition contract. Phase 32 will own actual acquisition cadence; Phase 34 will own the global coherent operator-snapshot revision.

### Settings authority

- `crates/bitaxe-api/src/settings.rs` identifies known fields from the complete NVS schema and forwards them into validation/persistence planning.
- `crates/bitaxe-config/src/settings.rs` and `persistence.rs` can represent credentials, fan, voltage, frequency, display, mining, and other writes. That remains useful compatibility machinery but is too broad to serve as v1.2 authority.
- The API shell should preserve established unknown/compatibility response behavior while issuing a write capability only for a validated hostname change.

### Claim admission

- `tools/parity/src/claim_ladder.rs`, `mining_allow.rs`, `safety_allow.rs`, and the Phase 30 promotion guard already use exact claim tiers and explicit non-claims.
- Phase 31 needs a smaller closed v1.2 admission contract: eligible observation-contract and hostname-allowlist claims versus typed exclusion reasons. Excluded categories must not share the constructible eligible type.

## Recommended Domain Model

### State-carrying observation

Use one generic pure shape equivalent to:

```rust
pub enum Observation<T> {
    Fresh { sample: StampedSample<T> },
    Stale { last_good: StampedSample<T>, reason: StaleReason },
    Unavailable { reason: UnavailableReason },
    Fault { reason: FaultReason, maybe_last_good: Option<StampedSample<T>> },
}

pub struct StampedSample<T> {
    pub value: T,
    pub boot_session: BootSessionId,
    pub sequence: ObservationSequence,
    pub acquired_at: MonotonicMillis,
}
```

Exact names are discretionary. The important invariants are:

- `Fresh` always has one validated successful sample.
- `Stale` retains the original last-good stamp unchanged.
- `Unavailable` has no usable sample.
- `Fault` never exposes a failed attempt as data; it may retain the prior successful sample.
- Only a successful producer acquisition advances `sequence` and `acquired_at`.
- Sequence comparison is scoped by boot session.
- Serialization and request reads cannot mutate state or recompute acquisition metadata.
- Public reasons are typed, stable, and redaction-safe; adapter details remain logs.

Keep legacy numeric projection as an API adapter. The adapter may emit compatibility zero for non-fresh state, but it must also emit the explicit per-fact state and must not infer state from the number.

### Closed settings and admission types

Use the compatibility parser to classify input, then translate only a valid hostname into a closed authority type equivalent to `V12SettingsChange::Hostname(Hostname)`. No constructor should accept a generic schema key or `SettingsPatch`.

Model claim admission as a closed result equivalent to:

```rust
pub enum V12AdmissionDecision {
    Eligible(V12PromotableClaim),
    Ineligible(V12ExclusionReason),
}
```

The eligible enum should contain only Phase 31's exact contract claims. Typed exclusion reasons should cover active control, self-test effects, watchdog intervention/load, mining and archived Phase 28.1.1, credentials, direct UART/pins, OTA/recovery, other boards, telemetry history/UI expansion, and broad promotion. Do not allow string-to-eligible conversion or infer eligibility from the full NVS schema.

## Recommended Implementation Boundaries

1. Add shared observation/stamp/reason types to an existing pure crate, preferably `bitaxe-safety` unless a dependency-cycle check shows `bitaxe-core` is the cleaner owner.
2. Migrate power/thermal constructors and safety decisions to the authoritative observation type while keeping active effect planning unchanged and fail-closed.
3. Extend `bitaxe-api` snapshot/wire models with explicit per-fact truth, mapping compatibility numeric fields separately and adding golden/serialization tests.
4. Change firmware snapshot collection to consume stored observation state only. Phase 31 may install an explicit unavailable default store; Phase 32 adds the producer and real reads.
5. Introduce hostname-only v1.2 settings authority after compatibility parsing and before config persistence planning. Preserve public compatibility behavior but emit zero non-hostname writes/effects.
6. Add a typed Phase 31 admission validator and negative fixtures to `tools/parity`, without changing checklist statuses.

## Important Pitfalls

- Do not leave aggregate `SafetyTelemetryStatus` as the authoritative truth while adding cosmetic per-field labels.
- Do not stamp stale/fault transitions with a new acquisition time or sequence.
- Do not classify freshness independently in HTTP, WebSocket, logs, and evidence code.
- Do not use wall-clock time; stamps and age checks use one monotonic boot-local clock.
- Do not introduce Phase 34's global snapshot revision as the sensor sequence.
- Do not narrow the public parser in a way that breaks established compatibility response behavior; narrow the authority granted after parsing.
- Do not pass a generic `SettingsPatch`, schema key, map, or string into v1.2 persistence authority.
- Do not add sensor I/O, fan/voltage/control writes, settings durability, reboot, hardware evidence, credentials, archived Phase 28.1.1 work, or parity promotion.
- Do not copy upstream C expression; use it only as behavioral evidence where needed.

## Recommended Plan Structure

Use three sequential plans:

1. **Observation truth core** — add state/stamp/reason types, migrate pure power/thermal decisions, and exhaustively test legal transitions and compatibility projection inputs.
2. **API and firmware consumer boundary** — publish independent wire truth, preserve numeric compatibility, and prove repeated API/projection reads cannot advance observation metadata.
3. **Hostname authority and exact admission** — add the hostname-only capability plus typed eligibility/exclusion model, parity guard fixtures, and final OBS-01/CFG-08 repository verification.

The sequence keeps the shared type authoritative before adapters migrate and keeps settings/admission changes independent from telemetry internals.

## Validation Architecture

### Test layers

| Layer | Target behavior | Primary checks |
| --- | --- | --- |
| Pure observation unit tests | Four states are mutually exclusive; only successful production advances stamps; stale/fault retain last-good metadata | Focused tests in the owning pure crate via Cargo and Bazel |
| Safety migration tests | Existing safe decisions remain fail-closed and independent facts can diverge in state | `bitaxe-safety` unit tests and existing fixture tests |
| API contract tests | Explicit per-fact state accompanies unchanged compatibility numerics; zero never authenticates freshness | `bitaxe-api` snapshot, wire, telemetry, and golden tests |
| Consumer ownership tests | Repeated API, WebSocket, log, and evidence reads do not change sequence or acquisition time | Firmware/runtime snapshot host-testable helpers plus projection regression tests |
| Settings authority tests | Only valid hostname input constructs v1.2 write authority; all broader fields emit no v1.2 write/effect capability | `bitaxe-api` and `bitaxe-config` focused tests |
| Admission tests | Every excluded category is deterministically ineligible and cannot be converted into an eligible claim | `bitaxe-parity` unit tests plus `just parity` |
| Repository gate | Formatting, lint, build, tests, reference cleanliness, and lifecycle all pass | Mandatory Cargo sequence, `just test`, `just parity`, `just verify-reference`, lifecycle validator |

### Required regression cases

- Fresh observation requires a valid stamped sample.
- Stale retains the exact last-good boot session, sequence, acquisition time, and value.
- Fault with and without last-good data never publishes a failed attempt as a sample.
- Unavailable has no sample and maps compatibility numerics to their existing fallback without implying fresh.
- Power, bus voltage, current, chip temperature, VR temperature, and tachometer states can differ independently.
- Two or more identical consumer reads produce identical observation metadata.
- A failed producer update does not advance sequence or acquisition time; the next successful update advances exactly once.
- Valid hostname produces the sole v1.2 settings capability.
- Credentials, fan, voltage, frequency, display, mining, unknown, and mixed hostname-plus-broader input produce no broader v1.2 authority or effect.
- Every excluded claim category fails admission; exact Phase 31 eligible fixtures pass only their own claim.
- Existing safety decisions, API compatibility fixtures, settings generic public errors, and parity non-claims remain intact.

### Sampling and completion gate

- Run the focused affected-crate test after each task.
- After each plan, run its affected Bazel/Cargo targets and `git diff --check`.
- Before every commit, run in order: `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, `cargo test --all-features`.
- Before phase verification, also run `just test`, `just parity`, `just verify-reference`, requirement traceability checks for OBS-01 and CFG-08, and GSD lifecycle validation.
- All Phase 31 verification is deterministic and repository-local; hardware and network evidence are neither required nor allowed.

## Research Conclusion

Phase 31 is ready to plan with existing repository primitives. The durable solution is one authoritative typed observation contract, one read-only consumer projection boundary, and one closed hostname/admission authority. That is sufficient to satisfy OBS-01 and CFG-08 without pulling later I/O, durability, coherent-snapshot, hardware-evidence, or active-control work into the phase.

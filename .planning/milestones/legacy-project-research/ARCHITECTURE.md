# Architecture Research

**Domain:** Ultra 205 operator-ready runtime integration
**Researched:** 2026-07-13
**Confidence:** HIGH

## Scope and Constraints

Milestone v1.2 should integrate five already-modeled concerns into one truthful operator runtime: read-only INA260/EMC2101 observation, NVS-backed configuration, firmware/build identity, self-test/watchdog health, and correlated detector-gated evidence. It must preserve the existing AxeOS routes and functional-core/imperative-shell split.

The milestone must not reuse the archived Phase 28.1.1 diagnostic runtime, enable mining diagnostics, or perform active fan, voltage, reset, ASIC-enable, power-sequencing, or fault-injection work. A register-address write that is part of an I2C read transaction is read semantics; writes to configuration or actuator registers are outside the v1.2 sensor path. If a value such as fan RPM cannot be observed reliably without mutating device configuration, report it unavailable instead of initializing the actuator.

The project-local `AGENTS.md` hardware gates, credential/redaction rules, terminal archive rule, and serial-session rules materially constrain the design. `standards/core/architecture.md` supplies the functional-core/imperative-shell, parse-at-boundaries, and illegal-state-modeling rules. There is no active architecture override in `standards-overrides.md`.

## Standard Architecture

### System Overview

```text
┌─────────────────────────────────────────────────────────────────────┐
│ Host build and evidence shell                                       │
│ package manifest ─ detector/board-info ─ bounded capture ─ validator│
└───────────────────────────────┬─────────────────────────────────────┘
                                │ package/session/revision correlation
┌───────────────────────────────▼─────────────────────────────────────┐
│ Firmware imperative shell                                           │
│                                                                     │
│ boot coordinator                                                    │
│   ├── single I2C0 owner ── periodic INA260/EMC2101 read transactions│
│   ├── single NVS settings owner ── commit/reload/reconcile           │
│   ├── health supervisor ── self-test + watchdog checkpoints         │
│   └── provenance collector ── compile/package/runtime identity       │
│                  │                                                  │
│                  ▼                                                  │
│        coherent OperatorRuntimeSnapshot cell                        │
│                  │ clone only; never held across I/O                │
│          ┌───────┴────────┐                                         │
│          ▼                ▼                                         │
│     HTTP API         WebSocket cadence                              │
└──────────┬────────────────┬─────────────────────────────────────────┘
           │ typed snapshots│
┌──────────▼────────────────▼─────────────────────────────────────────┐
│ Pure Rust core                                                      │
│ sensor acquisition/freshness ─ safety classification                │
│ settings reconciliation ─ operator-health projection ─ API mapping  │
└─────────────────────────────────────────────────────────────────────┘
```

### Component Responsibilities

| Component | Ownership | v1.2 responsibility |
| --- | --- | --- |
| `bitaxe-safety` observation core | Sensor status and freshness rules | Represent never-observed, fresh, stale, and failed acquisition without sentinel zeros; separately classify unsafe values. |
| `operator_i2c_runtime` firmware shell | The only live `I2cDriver`/I2C0 handle | Perform bounded, serialized, read-semantic transactions; timestamp attempts and successes; publish typed samples. |
| `bitaxe-config` persistence core | NVS snapshot and requested-vs-reloaded reconciliation | Decide whether committed settings were actually observed after reload and expose mismatch/failure explicitly. |
| `settings_adapter` firmware shell | The only live NVS namespace owner | Serialize PATCH operations, commit, read an actual post-commit snapshot, and publish only storage-observed state. |
| `operator_runtime_state` firmware boundary | One coherent, cloneable operator snapshot | Atomically publish sensor acquisition, confirmed settings, health, provenance, session, and revision metadata. |
| `bitaxe-api` projection core | AxeOS DTO and additive operator-status mapping | Keep required upstream fields while exposing explicit status through the existing `/api/system/info` and `/api/ws/live` surfaces. |
| `safety_adapter::watchdog` / health supervisor | Runtime cadence and progress checkpoints | Publish liveness and last-progress facts; do not equate a log marker with ESP task-watchdog proof. |
| Build/provenance adapter | Compile-time and package-derived identity | Produce one typed identity used by boot logs and API projection, correlated to package evidence. |
| `tools/parity` v1.2 evidence profile | Evidence schema, inventory, redaction, correlation | Validate package, board, session, API/WebSocket, settings round-trip, health, and exact non-claims as one chain. |

## Recommended Project Structure

```text
crates/
├── bitaxe-safety/src/
│   ├── observation.rs             # Generic acquisition envelope and aging
│   ├── power.rs                   # INA260 value/safety classification
│   ├── thermal.rs                 # EMC2101 value/safety classification
│   ├── self_test.rs               # Existing typed lifecycle, projected honestly
│   └── watchdog.rs                # Existing pure step decisions
├── bitaxe-config/src/
│   └── persistence.rs             # Reload reconciliation and confirmed state
└── bitaxe-api/src/
    ├── snapshot.rs                # Operator snapshot inputs
    ├── operator_runtime.rs        # Additive status/provenance DTO projection
    ├── runtime_projection.rs      # Shared HTTP/WebSocket view construction
    └── wire.rs                    # Required AxeOS fields plus extension

firmware/bitaxe/src/
├── main.rs                        # Deliberate boot ownership handoff
├── operator_i2c_runtime.rs        # Sole I2C0 owner and sampling loop
├── operator_runtime_state.rs      # Coherent latest-state cell
├── firmware_identity.rs           # Typed compile/runtime provenance collector
├── settings_adapter.rs            # Sole NVS owner and verified reload shell
├── safety_adapter.rs              # Report projection, no v1.2 actuation
├── safety_adapter/
│   ├── i2c_bus.rs                 # Atomic register read primitive
│   ├── ina260.rs                  # Raw INA260 read adapter
│   ├── emc2101.rs                 # Read functions split from mutating init
│   └── watchdog.rs                # Health heartbeat publication
├── runtime_snapshot.rs            # API projection from coherent state
├── http_api.rs                    # Existing routes, thin request shell
└── websocket_api.rs               # Existing diff/cadence shell

tools/parity/src/
└── operator_evidence/             # New v1.2 profile and correlation validator

scripts/
└── phaseNN-operator-runtime-evidence.sh  # Repo-owned bounded orchestrator
```

### Structure Rationale

- Keep acquisition freshness and unsafe-value classification in pure Rust. ESP-IDF owns clocks and reads, but it must not decide that a cached value is fresh.
- Give I2C0 one lifetime owner. The current normal boot consumes I2C0 in the startup display adapter, while Phase 27 creates and retains another purpose-specific bus. v1.2 must replace that fork with one normal-runtime owner rather than extending `phase27_bring_up` or `power_probe`.
- Keep NVS truth separate from the requested PATCH. Current code reloads NVS and then overlays the requested writes into the in-memory snapshot; that overlay can mask a partial reload or mismatch.
- Keep API and WebSocket projections pure and fed from the same snapshot revision. Request handlers should neither sample hardware nor reload NVS.
- Extend the existing evidence validator instead of encoding truth in shell `grep` alone. The shell owns orchestration; Rust owns typed correlation and admission.

## Architectural Patterns

### Pattern 1: Acquisition State Before Safety State

**What:** Model whether a reading exists and how it was obtained separately from whether the value is safe. The adapter records `sampled_at`, `last_attempt_at`, and a monotonic sequence; a pure projection derives freshness at a supplied `now`.

```rust
enum SensorAcquisition<T, E> {
    Unavailable { reason: E, last_attempt_ms: u64 },
    Observed { sample: T, sampled_at_ms: u64, sequence: u64 },
    Failed {
        reason: E,
        failed_at_ms: u64,
        maybe_last_good: Option<TimedSample<T>>,
    },
}

enum ObservationAt<T, E> {
    Fresh(TimedSample<T>),
    Stale { last_good: TimedSample<T>, age_ms: u64 },
    Failed { reason: E },
    Unavailable { reason: E },
}
```

**When to use:** INA260 power/current/voltage, EMC2101 temperature/tach, supervisor heartbeat, and any future polled operator state.

**Trade-offs:** More types than the current `SafetyTelemetryReport`, but it prevents a last-known value, failed read, missing sensor, unsafe value, and fresh sample from collapsing into the same zero-filled DTO. `PowerObservation` already accepts age; thermal needs the equivalent age boundary.

### Pattern 2: Single Effect Owner, Latest-State Readers

**What:** One dedicated runtime owns I2C0 and performs sequential transactions. It publishes a small cloneable snapshot after completing a sample cycle. HTTP, WebSocket, evidence, and safety projection only read the snapshot.

**When to use:** Shared INA260/EMC2101 observation on the Ultra 205 bus and later optional display initialization through the same owner.

**Trade-offs:** A slow device delays the next sample on the same bus. Bounded transaction timeouts, explicit failed cycles, and watchdog checkpoints are preferable to concurrent drivers or a mutex held by arbitrary callers. Snapshot locks must never be held during I2C, NVS, HTTP, logging, or serialization.

### Pattern 3: Commit, Reload, Reconcile, Then Publish

**What:** Treat a PATCH as confirmed only when the adapter reads storage after commit and a pure reconciliation function compares the actual `NvsSnapshot` with the planned writes.

```text
parse PATCH -> pure plan -> serialized NVS writes -> commit
    -> actual reload snapshot -> pure reconcile(requested, actual)
    -> publish Confirmed / ReloadFailed / Mismatch
    -> return public success only for Confirmed
```

**When to use:** Every settings PATCH and the boot-time reload used to populate API-visible configuration.

**Trade-offs:** A commit can succeed while verification fails. In that case the runtime must expose `reload_failed` or `verification_mismatch`, retain the actual snapshot if one was read, and never synthesize success by applying requested writes to the cache. A later reload can recover the state.

### Pattern 4: Coherent Projection With Correlation Keys

**What:** Publish sensor, settings, health, and provenance facts under one snapshot revision and an opaque boot-session tag. HTTP and WebSocket derive their payloads from that same typed snapshot.

**When to use:** `/api/system/info`, `/api/ws/live`, serial evidence markers, settings round trips, and bounded health evidence.

**Trade-offs:** Dynamic platform metrics may be collected just before publication, but their revision must be explicit. The session tag must be generated locally and must not encode a MAC address, USB identity, IP, credential, or device path.

### Pattern 5: Compatible Public Extension, Exact Legacy Fields

**What:** Preserve all required AxeOS field names and numeric behavior, while adding a versioned operator-runtime object to the existing system-info/WebSocket payload. Legacy numeric sensor fields contain values only for fresh eligible observations; the extension carries status, age/sequence, configuration confirmation, health, and provenance.

**When to use:** The milestone requirement that explicit unavailable/stale/failed states be operator-visible without replacing AxeOS routes or the static UI.

**Trade-offs:** Additive JSON is safer than silently overloading zero, but compatibility tests must prove existing required fields and types remain unchanged. Do not expose internal evidence IDs, raw endpoints, NVS secret values, device identifiers, or error strings that may contain them.

## Data Flow

### Read-Only Sensor Flow

```text
Peripherals::take
  -> boot coordinator transfers i2c0 + GPIO47/48 exactly once
  -> operator I2C owner constructs one driver
  -> bounded atomic register-pointer/read transactions
  -> raw INA260 and EMC2101 samples or typed read failure
  -> timestamped SensorAcquisition values
  -> pure project_at(now) freshness decision
  -> pure power/thermal validity and safety classification
  -> coherent OperatorRuntimeSnapshot revision
  -> bitaxe-api maps legacy numbers + explicit operator status
  -> HTTP and WebSocket expose the same sequence/revision
```

The I2C owner should sample on a cadence comfortably below the stale threshold. Timeout and retry budgets must remain bounded and watchdog-aware; the current 1000 ms per operation is unsuitable for a supervisor whose pure step budget is 25 ms. Do not perform request-time I2C reads. Do not call `emc2101::init`, `set_fan_duty_percent`, DS4432U writes, ASIC-enable, or reset operations from this path.

If startup display rendering remains enabled, it must be an initialization action owned by the same I2C runtime or use a driver-borrow/release contract that returns the single driver before sampling begins. Creating a second driver or dropping the only driver after display rendering is not acceptable. Display/runtime input parity remains out of scope.

### Settings Flow

```text
HTTP PATCH body
  -> bitaxe-api parses known fields and emits SettingsPersistencePlan
  -> single settings owner serializes mutation against current confirmed snapshot
  -> adapter writes and commits NVS namespace `main`
  -> adapter reads a new NvsSnapshot from the same storage owner
  -> bitaxe-config reconciles every planned write with actual stored type/value
  -> settings state publishes Confirmed(revision, snapshot)
     or ReloadFailed / VerificationMismatch
  -> API projection reads confirmed/actual state
  -> best-effort live effects run only after confirmed persistence
```

`FirmwareSettingsAdapter::reload` should return the actual snapshot (or provide it through a typed result), not merely `Ok(())`. Remove the post-success optimistic `apply_persisted_settings_writes(plan.writes())` path. Boot load, HTTP startup, and concurrent PATCH handlers should not repeatedly `take` independent NVS ownership; one service should own the handle and serialize access.

### Provenance Flow

```text
Bazel source_commit_stamp + package manifest + Cargo build environment
  -> typed FirmwareIdentity
  -> boot marker and OperatorRuntimeSnapshot
  -> version / axeOSVersion / idfVersion plus operator provenance extension
  -> evidence validator matches API/serial identity to package source/reference
```

The current `version` field is a short source commit and `axeOSVersion` remains `safe-fixture`. v1.2 should replace placeholders with truthful, build-derived values. Use one source of truth for firmware version, source commit, static asset version, reference commit, ESP-IDF version, and target/profile. Missing values must be `Unavailable`, never a stale hand-written hash. Existing package-manifest validation remains authoritative for artifact identity.

### Health Flow

```text
health supervisor iteration
  -> records started / last_progress / step kind / decision / sequence
  -> pure projection derives running, stale, failed, or unavailable at now
  -> self-test state is projected independently
  -> coherent snapshot -> API/WebSocket -> correlated evidence
```

Keep three claims separate:

1. The safety supervisor task started and continued to publish progress.
1. Pure step supervision requested continue/yield/watchdog action.
1. The ESP-IDF task watchdog was actually configured or fed.

The current code proves only narrow start/yield log markers. v1.2 may make supervisor liveness operator-visible, but must not claim ESP task-watchdog integration unless a real adapter and evidence exist. Existing self-test types may be projected as idle/running/passed/failed/unavailable; v1.2 must not execute diagnostic ASIC work or active fan/power submodes to improve that status.

### Correlated Evidence Flow

```text
exact source package
  -> detector + board-info
  -> flash/boot session tag and provenance marker
  -> repeated sensor API + WebSocket samples sharing revisions/sequences
  -> non-secret settings PATCH
  -> storage-confirmed reload + API/WebSocket confirmation
  -> controlled reboot through an approved repo-owned path
  -> new boot session with same package and persisted setting
  -> bounded health observations
  -> redaction/inventory/correlation validator
  -> exact checklist promotion or explicit non-promotion
```

Add a dedicated v1.2 operator-readiness evidence profile rather than reusing Phase 23/25/27/28 mining profiles. The validator should require consistent board, package source/reference identity, ordered session/revision metadata, matching API/WebSocket sensor observations, a confirmed settings revision before reboot, and the same storage-observed value after reboot. Evidence may record benign values or redacted category/digest forms, but must not retain Wi-Fi/pool credentials, private URLs, IPs, MACs, USB paths, or NVS secret values.

Hardware evidence should prove normal read-only observation and persistence. Unavailable/stale/failed behavior can be proven with deterministic fake adapters and pure aging tests; do not induce electrical faults or active control merely to produce those states.

## Concurrency and Lifecycle Boundaries

| Boundary | Owner | Readers | Rule |
| --- | --- | --- | --- |
| ESP-IDF I2C0 driver | `operator_i2c_runtime` task | None directly | Construct once, transact serially, never share driver guards with HTTP/WebSocket. |
| Sensor acquisition state | Operator snapshot publisher | API, WebSocket, health/evidence projection | Clone under a short lock; derive age at a supplied monotonic `now`. |
| NVS namespace and confirmed settings | Settings owner/service | PATCH planner and operator snapshot | Serialize writes and reload; publish only actual storage observations. |
| Operator snapshot revision | `operator_runtime_state` | HTTP/WebSocket/evidence markers | One atomic/coherent revision; no I/O while locked. |
| WebSocket diff state | Existing WebSocket adapter | Cadence task | Diff already-projected public values; never resample hardware. |
| Health heartbeat | Health supervisor | Operator snapshot projector | Monotonic progress sequence; staleness is derived, not cleared by API reads. |
| Evidence destination | Host evidence orchestrator/validator | Review and parity admission | Stage, validate last, and promote atomically using existing ownership protections. |

## New Components

| Component | Why new | Inputs | Outputs |
| --- | --- | --- | --- |
| Generic sensor acquisition/freshness core | Current power has age but thermal does not; current adapters collapse missing/read failure/unsafe/stale states. | Timestamped samples, failures, monotonic `now`, stale threshold. | Fresh/stale/failed/unavailable typed projections. |
| `operator_i2c_runtime` | Normal boot has no persistent read-only I2C owner; the retained bus is diagnostic-only. | I2C0, GPIO47/48, monotonic clock. | Periodic INA260/EMC2101 acquisition updates and progress events. |
| `operator_runtime_state` | Current API collection reads several global cells sequentially and can mix generations. | Sensor, settings, health, provenance updates. | One cloneable revisioned snapshot. |
| Typed firmware identity collector | API contains a real short commit but a placeholder AxeOS value and scattered constants. | Build environment, package/static version, ESP-IDF/runtime facts. | Truthful identity for boot, API, WebSocket, and evidence. |
| v1.2 operator evidence profile | Existing profiles are mining-phase-specific and do not validate settings/reboot/sensor/health correlation. | Package, detector, capture, API/WS, settings and health artifacts. | Typed pass/fail report with exact non-claims. |

## Modified Components

| Component | Required change | Important boundary |
| --- | --- | --- |
| `firmware/bitaxe/src/main.rs` | Transfer I2C0 once to the normal operator runtime and establish state/provenance/settings owners before HTTP publication. | Do not route through archived diagnostic modes or create a second I2C driver. |
| `safety_adapter/i2c_bus.rs` | Provide bounded atomic register reads and typed failure categories. | Register-pointer writes are read semantics; no actuator/config writes in v1.2 runtime. |
| `safety_adapter/ina260.rs`, `emc2101.rs` | Return raw typed reads without deciding freshness or swallowing failures. Split mutating EMC initialization from read functions. | Fan/temp read failure is not numeric zero. |
| `bitaxe-safety::{power,thermal}` | Share acquisition/age semantics; preserve unsafe-value classification separately. | Stale and read-failed must remain representable without claiming a fresh value. |
| `settings_adapter.rs` and `bitaxe-api::settings` | Return and reconcile actual reload state; serialize NVS ownership. | Remove optimistic requested-write overlay from the success claim. |
| `runtime_snapshot.rs` | Project one coherent operator snapshot plus dynamic platform facts. | API collection performs no hardware/storage I/O. |
| `bitaxe-api::{snapshot,wire,runtime_projection}` | Add versioned operator status/provenance projection while preserving required AxeOS fields/types. | Legacy sensor numbers are populated only from eligible fresh observations. |
| `safety_adapter/watchdog.rs` | Publish recurring liveness/progress state, not a one-time log boolean. | Supervisor heartbeat is not proof of ESP task-watchdog feed/reset behavior. |
| `http_api.rs` / `websocket_api.rs` | Read the same revisioned projection and expose correlation metadata. | PATCH remains effectful shell; WebSocket cadence never samples devices. |
| `tools/parity` and repo evidence script | Add schema, redaction, ordered correlation, no-actuation guards, and atomic completion for v1.2. | A collection of individually valid files is not a valid chain unless identity/session/revisions correlate. |

## Dependency-Aware Build Order

| Order | Build item | Why this order | Primary verification |
| ---: | --- | --- | --- |
| 1 | Pure acquisition, aging, failure, health, and settings-reconciliation types. | Defines illegal-state boundaries before firmware globals or wire fields. | Unit/fixture matrix for fresh→stale, never-observed, failed read, unsafe value, reload match/mismatch/failure. |
| 2 | Pure AxeOS-compatible operator projection and provenance DTO. | Establishes what operators and evidence can observe before hardware wiring. | Required-key/type fixtures, additive extension tests, redaction denylist tests. |
| 3 | Single-owner I2C runtime with fake/read adapters. | Resolves the current I2C0 lifetime conflict and proves no-actuation call surface. | Host/fake cadence, timeout, failure recovery, sequence, and lock-boundary tests. |
| 4 | Single-owner NVS reload/reconciliation path. | Prevents API or evidence from observing optimistic values. | PATCH→commit→actual reload tests, concurrent serialization tests, reboot-load fixtures. |
| 5 | Coherent operator runtime snapshot, provenance collector, and health heartbeat. | Composes already-typed subsystems without moving decisions into firmware. | Revision/session consistency and stale-heartbeat projection tests. |
| 6 | Boot, HTTP, and WebSocket integration. | User-visible routes are wired only after ownership and truth boundaries exist. | Affected Bazel/Cargo tests and API/WebSocket correlation tests. |
| 7 | v1.2 evidence profile and bounded wrapper. | Evidence schema should validate the exact finished architecture, not drive ad hoc logging design. | Deterministic validator tests for mismatched session/revision/package, redaction, no-actuation, and interrupted runs. |
| 8 | Detector-gated Ultra 205 evidence and exact parity review. | Hardware promotion follows software gates and package identity. | Fresh sensor chain, confirmed settings reload/reboot, provenance and health correlation, redaction pass, exact non-claims. |

## Scaling Considerations

| Scale | Architecture adjustment |
| --- | --- |
| One Ultra 205, normal runtime | One I2C owner, one settings owner, and one latest-state snapshot are sufficient. |
| Longer bounded observation | Add bounded history only in host evidence; firmware should retain latest sample, last-good metadata, counters, and health revision rather than an unbounded log. |
| Future active control | Send typed commands to the existing I2C owner after a later milestone adds recovery and hardware-regression gates. Do not let active controllers acquire the bus directly. |
| Future boards | Introduce board-capability-selected sensor plans only after Ultra 205 semantics are verified; do not generalize v1.2 around unsupported devices. |

### Scaling Priorities

1. **First bottleneck:** I2C timeout/retry latency can make samples stale and starve progress. Bound each transaction, publish failures promptly, and retry on later cadence cycles.
1. **Second bottleneck:** Mixed-generation API reads can create evidence that never existed on-device. Publish a single coherent revision and project it everywhere.
1. **Third bottleneck:** Concurrent PATCH requests can reorder storage and cache state. Serialize mutations under one NVS owner and return the confirmed revision.

## Anti-Patterns

### Extending the Phase 27/28.1 Diagnostic Bus

**What people do:** Turn `phase27_bring_up` or `power_probe` into the normal sensor service.
**Why it is wrong:** Those paths initialize active hardware, are evidence-mode-specific, and belong to archived mining diagnostics.
**Do this instead:** Build a normal-runtime, read-semantic I2C owner with no dependency on diagnostic modes.

### Multiple I2C Drivers or Arbitrary Shared Mutex Access

**What people do:** Let display, sensors, HTTP, and later controllers each construct or lock the driver.
**Why it is wrong:** Ownership and transaction ordering become implicit; register-pointer/read pairs can interleave and request latency can block the bus.
**Do this instead:** One owner serializes complete transactions and publishes latest state.

### Optimistic Settings Cache

**What people do:** Apply requested writes to the API snapshot after `commit`, even when the actual reload was partial or unverified.
**Why it is wrong:** The operator sees the request, not storage truth, and reboot evidence may contradict the pre-reboot API.
**Do this instead:** Reconcile the actual post-commit snapshot and publish confirmed, mismatch, or reload-failed state.

### Zero as Status

**What people do:** Map missing, stale, failed, and unsafe sensor readings to `0` without a visible reason.
**Why it is wrong:** Zero can be a plausible numeric value and cannot support the milestone's explicit-state requirement.
**Do this instead:** Preserve legacy zero-compatible fields where necessary but add typed status/age/sequence on the existing public surfaces.

### Request-Time Sampling

**What people do:** Read I2C or NVS inside `/api/system/info` or WebSocket cadence.
**Why it is wrong:** Client traffic changes device scheduling, response time, freshness, and watchdog behavior.
**Do this instead:** Sample/persist in owned tasks; HTTP/WebSocket clone and serialize a snapshot.

### Log Markers as Health Proof

**What people do:** Treat one `safety_supervisor=started` or `yield` line as continuing liveness or actual watchdog integration.
**Why it is wrong:** A task can stop immediately after the marker, and pure yield decisions do not feed the ESP watchdog.
**Do this instead:** Publish monotonic recurring progress and retain exact non-claims for unimplemented watchdog effects.

### Uncorrelated Evidence Packs

**What people do:** Combine a package from one commit, API from one boot, WebSocket from another, and settings values from a third run.
**Why it is wrong:** Every artifact may look valid while the claimed end-to-end behavior never occurred.
**Do this instead:** Validate source/reference identity, board, ordered boot sessions, snapshot revisions, and settings revisions as one chain.

## Integration Points

### External Boundaries

| Boundary | Integration pattern | Notes |
| --- | --- | --- |
| ESP-IDF I2C0 | One `I2cDriver` inside operator task | GPIO47/48, bounded transactions, no v1.2 actuator/config writes. |
| INA260 at `0x40` | Register-pointer plus atomic read | Preserve read failure separately from last-good values. |
| EMC2101 at `0x4c` | External-temperature and tach register reads only | Do not call mutating init/fan-duty functions for operator telemetry. |
| ESP-IDF NVS namespace `main` | One serialized settings service | Commit, actual reload, pure reconcile, then publish. |
| ESP-IDF platform facts | Thin collectors for heap, reset, partition, IDF | Fold into a revisioned snapshot; missing values are explicit. |
| AxeOS HTTP/WebSocket | Existing routes and cadence | Required legacy fields remain stable; additive status is versioned. |
| Host package/detector/evidence tools | Repo-owned `just`/Bazel commands | Hardware use remains detector-gated and redacted; no credentials are required for architecture work. |

### Internal Boundaries

| Boundary | Communication | Notes |
| --- | --- | --- |
| I2C runtime → safety core | Timestamped raw sample/result | Core owns freshness and validity; adapter owns bytes and clock reads. |
| I2C runtime → operator state | Revisioned acquisition update | Publish after a complete cycle or explicit failed cycle. |
| Settings owner → config core | Planned writes + actual reloaded snapshot | Reconciliation is pure and exhaustive for planned keys. |
| Settings owner → operator state | Confirmed/mismatch/reload-failed update | Do not expose requested values as confirmed. |
| Health supervisor → operator state | Monotonic progress event | API reads cannot refresh liveness. |
| Operator state → bitaxe-api | One clone plus `now` | Projection derives stale states and public DTOs. |
| bitaxe-api → HTTP/WebSocket | Same public view/revision | WebSocket diffing occurs after projection. |
| Evidence shell → parity validator | Staged typed artifacts | Validator checks correlation and redaction before atomic promotion. |

## Sources

- `.planning/PROJECT.md` — active v1.2 goal, target features, exclusions, and current architecture. Confidence: HIGH.
- `.planning/milestones/v1.1-MILESTONE-AUDIT.md` — accepted gaps, exact non-claims, integration result, and terminal archive boundary. Confidence: HIGH.
- `.planning/RETROSPECTIVE.md` — evidence-first stopping rules and lessons from diagnostic convergence. Confidence: HIGH.
- `.planning/milestones/v1.1-research/ARCHITECTURE.md` — preceding runtime/component boundaries and evidence model. Confidence: HIGH.
- `AGENTS.md`, `AGENTS.bright-builds.md`, `standards-overrides.md`, and `standards/core/architecture.md` — local hardware/evidence restrictions and architecture rules. Confidence: HIGH.
- `firmware/bitaxe/src/main.rs`, `display_adapter.rs`, `runtime_snapshot.rs`, `settings_adapter.rs`, `http_api.rs`, and `websocket_api.rs` — current boot ownership, API projection, storage, and delivery paths. Confidence: HIGH.
- `firmware/bitaxe/src/safety_adapter/{i2c_bus,ina260,emc2101,power,thermal,watchdog,phase27_bring_up,power_probe}.rs` — current I2C lifecycle, sensor adapters, diagnostic retained bus, and supervisor behavior. Confidence: HIGH.
- `crates/bitaxe-safety/src/{power,thermal,self_test,watchdog,status,evidence}.rs` — current pure observation, safety, health, and evidence types. Confidence: HIGH.
- `crates/bitaxe-config/src/persistence.rs` and `crates/bitaxe-api/src/settings.rs` — current snapshot/reload model and ordered persistence executor. Confidence: HIGH.
- `crates/bitaxe-api/src/{snapshot,wire,runtime_projection,telemetry}.rs` — current AxeOS fields, safe-zero projection, and shared WebSocket/API payload. Confidence: HIGH.
- `firmware/bitaxe/build.rs`, `firmware/bitaxe/static/www/assets/release.json`, `tools/parity/src/release_evidence.rs`, and `docs/release/provenance-manifest.md` — current build identity and package provenance chain. Confidence: HIGH.
- `tools/parity/src/operator_evidence.rs` and `tools/parity/src/operator_evidence/{profile,inventory,generation}.rs` — current typed evidence profiles, redaction inventory, and atomic generation. Confidence: HIGH.
- `scripts/phase14-self-test-watchdog-load.sh`, `scripts/phase23-redacted-operator-evidence.sh`, and corresponding committed evidence — current marker-only health evidence and operator evidence orchestration gaps. Confidence: HIGH.
- `reference/esp-miner/main/i2c_bitaxe.c`, `power/INA260.c`, `thermal/EMC2101.c`, `nvs_config.c`, and `http_server/system_api_json.c` — pinned read-only behavioral breadcrumbs for shared bus, cached-read behavior, settings, and public fields. Confidence: HIGH.

*Architecture research for: v1.2 Ultra 205 Operator-Ready Runtime*
*Researched: 2026-07-13*

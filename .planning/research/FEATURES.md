# Feature Research

**Domain:** Ultra 205 operator-ready runtime observation, configuration, provenance, and health visibility
**Researched:** 2026-07-13
**Confidence:** HIGH for project scope, existing Rust contracts, upstream AxeOS surfaces, and evidence rules; MEDIUM for the final safe settings allowlist until requirements pin the exact keys and reboot semantics.

## Feature Landscape

v1.2 should make the already-flashable Ultra 205 understandable and predictably configurable during normal operator use. It is not a mining retry or an active hardware-control milestone. The acceptance boundary is a detector-gated, bounded session in which one physical board reports fresh read-only power and thermal observations, persists a deliberately safe subset of settings, exposes truthful firmware/build/runtime identity, and shows passive self-test and watchdog health through correlated firmware and AxeOS-compatible API evidence.

The central operator rule is that a numeric zero is not a status. Power and thermal values are usable only when accompanied by a fresh observation state. Missing sensors, failed reads, implausible values, and old samples must remain distinguishable as `unavailable`, `fault`, and `stale` rather than being flattened into plausible-looking zero values.

### Expected Operator Behavior

| Area | Expected behavior | Failure behavior |
| --- | --- | --- |
| Power telemetry | Current, input voltage, and power come from fresh read-only INA260 samples and are visible through the shared runtime/API projection. | Missing device or no sample is `unavailable`; expired data is `stale`; transport or invalid/unsafe reading is `fault`. A failure must not be presented as fresh zero telemetry. |
| Thermal telemetry | Chip temperature and available secondary/VR temperature facts come from fresh read-only thermal observations and share the same lifecycle as power telemetry. | Missing sensor is `unavailable`; invalid or failed reading is `fault`; an old retained value is explicitly `stale` if retained at all. No fan duty, reset, voltage, or power effect is emitted. |
| Settings | A published safe allowlist is validated as one request, written to NVS, committed, reloaded, and only then acknowledged. The operator sees the reloaded value immediately and again after a bounded reboot. | Malformed/invalid input performs no partial write. Write, commit, or reload failure does not return success. Public errors stay AxeOS-compatible while retained logs use redaction-safe reason categories. |
| Provenance | Firmware version, source identity, reference identity, ESP-IDF version, board/ASIC identity, running partition, and build/package identity are truthful and correlated. | Missing facts say `Unavailable` or an equivalent explicit state; synthetic values such as `safe-fixture` are never presented as device provenance. |
| Runtime health | Uptime, decoded reset reason, heap facts, passive self-test state, watchdog supervisor state/checkpoint recency, and safe runtime mode are inspectable without starting a hardware self-test. | Health collection failures remain visible and fail closed. Watchdog startup breadcrumbs alone are not reported as load or intervention proof. |
| Evidence | Detector, board-info, package/flash identity, device logs, API/WebSocket observations, persistence readback, reboot readback, and conclusion belong to one bounded session and are redacted before promotion. | Detection, target provenance, identity, capture, correlation, or redaction failure yields blocked evidence rather than a partial operator-readiness claim. |

### Table Stakes (Users Expect These)

Features users assume exist. Missing these means the runtime is not trustworthy for normal operator use.

| Feature | Why Expected | Complexity | Notes |
| --- | --- | --- | --- |
| Shared read-only I2C ownership | The Ultra 205 display, INA260, EMC2101-class thermal path, and future safety peripherals share one bus; operators should not see sporadic telemetry caused by competing bus initialization. | HIGH | One firmware-owned bus lifecycle should serialize reads and report initialization/read failure. v1.2 must not write DS4432U voltage registers or fan-control registers. |
| Fresh INA260 observation | An operator expects power, current, and input voltage to describe the device now, not a fixture or cached placeholder. | HIGH | Use the existing typed power model and its bounded freshness threshold. Preserve units and upstream-compatible numeric fields while exposing a companion observation state/reason. |
| Fresh thermal observation | Temperature is a basic device-health fact and is required before later safe actuation work. | HIGH | Read only. Distinguish missing, invalid/faulted, stale, and fresh data. Fan RPM may be reported only if it is independently observed; fan actuation is excluded. |
| Explicit telemetry states | Operators must be able to distinguish a healthy zero-like reading from missing, stale, or failed telemetry. | MEDIUM | Use typed states (`fresh`, `stale`, `unavailable`, `fault`) and stable redaction-safe reason labels. Do not let AxeOS-compatible zero defaults imply freshness. |
| Coherent API and WebSocket projection | The same device observation should not disagree across `/api/system/info`, `/api/ws/live`, retained logs, and evidence. | MEDIUM | Project from one shared snapshot/sample identity or equivalent bounded correlation marker; do not independently poll and silently combine unrelated samples. |
| Published safe settings allowlist | Operators need to know which PATCH fields v1.2 truly supports. | MEDIUM | At minimum, support `hostname`; add another field only if persistence cannot actuate hardware, alter mining, exercise display/input, trigger OTA, expose credentials, or invalidate the current evidence session. Do not imply that every field in the upstream schema is v1.2-supported. |
| Atomic PATCH validation and persistence | A successful settings response must mean every accepted write was committed and reloaded. | MEDIUM | Preserve the existing validation → write(s) → commit → reload → public-success ordering. Invalid known fields reject the request without partial writes; unknown-field behavior must be pinned to the existing compatibility contract. |
| Reload and reboot durability | Persisted configuration is meaningful only if the same value is observed after route reload and a real bounded reboot. | HIGH | Correlate pre-PATCH, post-reload, and post-reboot observations on the same detector-gated board. Do not test Wi-Fi-changing settings that would destroy the evidence target lock. |
| Truthful system and build provenance | Owners and maintainers need to identify exactly what firmware and package are running before trusting evidence or reporting a bug. | MEDIUM | Report firmware version/short source commit, reference commit or package provenance, ESP-IDF version, AxeOS/static asset version, board `205`, BM1366, running partition, and reset reason. Missing values are explicit, never synthetic. |
| Passive self-test state visibility | Operators need to know whether self-test is idle, blocked, running, passed, failed, canceled, or unavailable. | MEDIUM | v1.2 exposes state only. It does not start factory/manual self-test or execute diagnostic work, fan, power, voltage, reset, or ASIC effects. |
| Watchdog and runtime-health visibility | A responsive API is not enough to prove the runtime loops remain supervised. | MEDIUM | Expose supervisor started/available state, latest bounded checkpoint category/age, uptime, reset reason, and heap health without claiming watchdog intervention or load testing that did not occur. |
| Detector-gated correlated evidence | Hardware-sensitive parity requires a board-named, same-session proof chain. | HIGH | Begin with `just detect-ultra205`; require exactly one board `205`, board-info success, source/reference/package identity, trusted target derivation, bounded captures, redaction, and exact claim/non-claim conclusion. |

### Differentiators (Competitive Advantage)

These capabilities make the Rust firmware easier to trust and diagnose even when its observable API remains compatible with upstream AxeOS.

| Feature | Value Proposition | Complexity | Notes |
| --- | --- | --- | --- |
| Telemetry truth as a domain type | Operators and tests can tell fresh data from absence or failure without interpreting magic numbers. | MEDIUM | Make illegal combinations unrepresentable: a `fresh` observation carries usable values, while non-fresh states carry a reason and cannot silently promote numeric claims. |
| Observation-before-actuation architecture | v1.2 establishes real sensor and recovery visibility without risking voltage, fan, reset, or power effects. | MEDIUM | This provides a safer foundation for the later active-control milestone and follows the functional-core/imperative-shell project decision. |
| Transactional persistence evidence | A PATCH is proven through NVS commit, reload, API readback, and reboot continuity rather than inferred from an HTTP 200. | HIGH | The same session records the ordered boundary without committing setting values that are sensitive. |
| First-class build provenance | A support report can correlate the running firmware, package manifest, upstream reference, ESP-IDF, board, and evidence session. | MEDIUM | Present ordinary operator identity through existing compatible surfaces; keep local device/network identifiers redacted in shareable artifacts. |
| Honest passive health model | Self-test and watchdog state remain useful even when active tests are prohibited or evidence is absent. | MEDIUM | `blocked` or `unavailable` is a valid result and must not be transformed into passed health. |
| Exact-claim evidence pack | Contributors can reproduce what was observed and understand every excluded claim. | MEDIUM | Reuse the detector, target-lock, redaction, and atomic validation patterns established in v1.0/v1.1, but define a v1.2-specific operator-runtime profile rather than reusing mining evidence slots. |

### Anti-Features (Commonly Requested, Often Problematic)

| Feature | Why Requested | Why Problematic | Alternative |
| --- | --- | --- | --- |
| Active fan, voltage, reset, power sequencing, or fault injection | Live telemetry naturally invites control and failure testing. | These are safety-critical hardware effects with separate recovery and hardware-regression requirements; combining them with observation makes failures harder to isolate. | Keep v1.2 read-only. Plan active control only after observation and update/recovery foundations are independently proven. |
| Treating zero values as successful telemetry | It preserves the current upstream-compatible numeric shape with little work. | Current safe snapshots use zero-compatible placeholders; presenting them as sensor values would be a false hardware claim. | Add explicit status/freshness/reason projection and promote numeric values only from fresh samples. |
| Keeping the last value without a stale label | A stable dashboard looks better during transient read failures. | It hides sensor failure and can mislead later safety decisions. | Retain a last value only with explicit age and `stale`; otherwise expose `unavailable` or `fault`. |
| Supporting every upstream PATCH field | The schema and pure validation already contain many fields. | Wi-Fi, pool, frequency, voltage, fan, display, Stratum v2, and other fields cross v1.2 exclusions or can break target continuity and create implied behavior. | Publish a narrow safe allowlist and expand it in later evidence-backed milestones. |
| Returning PATCH success before commit/reload | It reduces route latency and mirrors optimistic UI behavior. | The operator cannot distinguish durable configuration from a failed write or reload. | Return success only after validation, every write, commit, and reload complete. Apply optional live effects only afterward. |
| Using a synthetic provenance placeholder | It keeps every API field populated. | Values such as `safe-fixture` are useful in tests but dishonest on a live device. | Use actual package/runtime facts or `Unavailable`. Test fixtures remain test-only. |
| Running self-test to prove self-test visibility | A real pass/fail result seems stronger than passive state. | Existing self-test can involve fan, power, voltage, ASIC work, and restart effects outside v1.2. | Expose idle/blocked/unavailable lifecycle and defer active self-test hardware closure. |
| Calling watchdog startup proof of resilience | Startup/yield markers already exist. | They do not prove intervention, recovery, bounded load behavior, or long-term responsiveness. | Report exactly observed supervisor/checkpoint state; retain intervention/load as explicit non-claims. |
| Network scanning or stale target reuse | It is convenient when the device URL is missing. | It violates the same-session target-source contract and can correlate evidence from the wrong device or boot. | Derive exactly one origin from the current detector-gated repo-owned monitor session or record the API portion blocked. |
| Reopening BM1366 mining diagnostics | Better telemetry might appear adjacent to the unresolved nonce blocker. | The Phase 28.1.1 lineage is terminal and repeated diagnostics had diminishing returns. | Keep v1.2 independent. A future mining milestone requires genuinely new evidence, a discriminating hypothesis, and a hard stopping rule. |
| AxeOS UI rewrite | New status presentation may tempt a frontend redesign. | The accepted compatibility boundary is API/static assets first; a UI rewrite would multiply scope without improving firmware evidence. | Preserve current AxeOS compatibility and expose additive, documented operator status through existing surfaces. |

## Feature Dependencies

```text
One firmware-owned read-only I2C lifecycle
    feeds -> INA260 + thermal sensor samples
        feeds -> Typed observation classification and freshness
            feeds -> Shared runtime snapshot
                feeds -> /api/system/info + /api/ws/live + retained logs
                    feeds -> Same-session telemetry evidence

Published safe settings allowlist
    feeds -> Pure PATCH validation
        feeds -> NVS writes -> commit -> reload -> immediate readback
            feeds -> Bounded reboot -> post-reboot readback
                feeds -> Persistence evidence

Package manifest + runtime collectors
    feeds -> Truthful provenance and runtime-health snapshot
        feeds -> API/log identity
            feeds -> Evidence identity correlation

Detector + board-info + trusted current-session target
    guards -> Flash/monitor + API/WebSocket + PATCH/reboot capture
        guards -> Redaction + exact parity promotion

Active hardware effects
    conflict with -> v1.2 read-only observation boundary
```

### Dependency Notes

- **Fresh telemetry requires shared I2C ownership:** initialization and access must be coordinated before any sample can be considered attributable and fresh.
- **API values require typed observation states:** compatibility DTOs should consume one classified runtime snapshot, not raw register values or zero-filled placeholders.
- **Telemetry evidence requires same-sample correlation:** device logs, API, and WebSocket need a bounded sample/session marker or equivalent chronology before they can support a parity claim.
- **PATCH durability requires ordered persistence:** success depends on validation, all writes, commit, reload, immediate readback, and post-reboot readback in that order.
- **Reboot evidence requires safe settings:** a Wi-Fi, credential, mining, display, or active-control change can invalidate target continuity or cross milestone boundaries, so it is excluded from the initial allowlist.
- **Provenance evidence requires package/runtime agreement:** the package manifest, flashed source/reference identity, runtime firmware fields, board identity, and evidence conclusion must describe the same build.
- **Runtime-health evidence requires passive collection:** self-test state and watchdog checkpoints may be observed, but v1.2 cannot manufacture stronger proof by running excluded hardware or fault paths.
- **Every live claim requires the detector gate:** a failed or ambiguous detector stops hardware/API work before credential access, filesystem promotion, or target inference.

## v1.2 Definition

### Launch With (v1.2)

- [ ] One shared read-only Ultra 205 I2C lifecycle for power and thermal observations, with no control-register writes.
- [ ] Fresh INA260 current/voltage/power and thermal observations with explicit `fresh`, `stale`, `unavailable`, and `fault` behavior as applicable.
- [ ] One coherent runtime projection across system info, live WebSocket telemetry, retained status logs, and evidence.
- [ ] A published safe settings PATCH allowlist, atomic validation/persistence, immediate reload readback, and bounded reboot durability proof.
- [ ] Truthful firmware, source/reference, ESP-IDF, AxeOS/static, board/ASIC, partition, reset, uptime, and heap provenance or explicit `Unavailable` states.
- [ ] Passive self-test lifecycle and watchdog/runtime-health visibility without starting active self-test or fault/load behavior.
- [ ] A detector-gated, redacted, same-session evidence pack that promotes only exact operator-runtime claims.

### Add After Validation (v1.2.x)

- [ ] Additional non-secret, non-actuating settings after each key has an explicit reload/reboot contract and cannot disrupt target continuity.
- [ ] Longer read-only telemetry continuity runs after single-session freshness, cadence, and stale/fault transitions are deterministic.
- [ ] Richer historical telemetry/statistics only after the shared snapshot producer proves bounded memory and sample ownership.
- [ ] Operator-facing support summaries derived from the same provenance and health model, without introducing a new UI framework.

### Explicit Deferrals (Later Milestones)

- [ ] OTA, rollback, interrupted update, recovery, and OTAWWW — planned for the update/recovery milestone.
- [ ] Active voltage, fan, reset, ASIC power sequencing, thermal/fan/power fault stimulus, and active self-test — planned only after observation and recovery foundations exist.
- [ ] BM1366 nonce/result/share diagnostics — terminal v1.1 lineage remains closed; reconsider only with genuinely new evidence and a bounded hypothesis.
- [ ] Gamma 601/BM1370 and every other board/ASIC family — require board-specific evidence after the Ultra 205 journey is credible.
- [ ] Runtime display/input, BAP, Stratum v2, and an Angular AxeOS rewrite — not needed to establish v1.2 operator-ready runtime behavior.

## Feature Prioritization Matrix

| Feature | User Value | Implementation Cost | Priority |
| --- | --- | --- | --- |
| Shared read-only I2C lifecycle | HIGH | HIGH | P1 |
| Typed fresh/stale/unavailable/fault telemetry | HIGH | MEDIUM | P1 |
| Live INA260 and thermal observations | HIGH | HIGH | P1 |
| Coherent API/WebSocket/log projection | HIGH | MEDIUM | P1 |
| Safe settings allowlist and atomic persistence | HIGH | MEDIUM | P1 |
| Reload and reboot durability proof | HIGH | HIGH | P1 |
| Truthful build/system provenance | HIGH | MEDIUM | P1 |
| Passive self-test/watchdog/runtime health | HIGH | MEDIUM | P1 |
| Detector-gated correlated evidence pack | HIGH | HIGH | P1 |
| Additional non-actuating settings | MEDIUM | MEDIUM | P2 |
| Longer telemetry continuity and history | MEDIUM | MEDIUM | P2 |
| Active hardware-control behavior | HIGH | HIGH | P3 |
| Mining re-entry | HIGH | HIGH | P3, trigger-gated |
| Other boards, display/BAP, Stratum v2, UI rewrite | MEDIUM | HIGH | P3 |

**Priority key:**

- P1: Must have for v1.2 operator-ready runtime.
- P2: Add only after the P1 evidence chain is stable.
- P3: Deferred to a separately scoped milestone and evidence contract.

## Reference Behavior Analysis

| Feature | Upstream ESP-Miner behavior | Rust v1.1 state | v1.2 approach |
| --- | --- | --- | --- |
| Power/thermal values | `/api/system/info` exposes power, voltage, current, temperatures, fan, and related numeric fields from global runtime state. | Typed pure power/thermal classifications exist, but live firmware projection remains zero-compatible/unavailable and lacks fresh sensor correlation. | Keep the compatible numeric surface while adding explicit observation status, freshness, and stable reason semantics backed by real read-only samples. |
| Settings PATCH | Upstream maps REST fields to NVS settings with type/range metadata. | Pure schema validation and firmware validation → write → commit → reload ordering exist; live PATCH/reboot evidence is missing. | Publish a safe v1.2 subset and prove rejection atomicity, reload readback, and reboot durability on board `205`. |
| Version/system info | Upstream reports firmware/AxeOS/ESP-IDF versions, board/ASIC, reset reason, partition, uptime, heap, and network facts. | Many fields are present, but live evidence includes a source short commit as `version`, `safe-fixture` AxeOS version, and a numeric reset reason string. | Report actual build/runtime facts with human-meaningful reset reason and explicit unavailable states; correlate them to package and reference identity. |
| Self-test/watchdog | Upstream self-test can exercise hardware and runtime tasks are monitored. | Pure self-test/watchdog models and startup/yield breadcrumbs exist; active hardware submodes and load/intervention proof remain below verified. | Expose passive lifecycle, supervisor availability, and checkpoint recency only; preserve active test/load/intervention as non-claims. |
| Evidence | Upstream is behavioral reference, not an evidence framework. | v1.0/v1.1 established detector, target-lock, redaction, typed validation, and exact-claim practices. | Reuse those practices in a dedicated operator-runtime evidence profile that correlates telemetry, configuration, provenance, and health without mining slots. |

## Sources

- `.planning/PROJECT.md` for the v1.2 goal, active requirements, terminal mining boundary, and project constraints.
- `.planning/milestones/v1.1-MILESTONE-AUDIT.md` for accepted unresolved mining debt, exact non-claims, and evidence limitations carried into v1.2.
- `.planning/RETROSPECTIVE.md` for the evidence-first, fail-closed, bounded-diagnostic, and terminal-unresolved lessons.
- `.planning/milestones/v1.1-research/FEATURES.md` for the prior feature boundary and the v1.1 capabilities that v1.2 must not reclassify as verified.
- `docs/project/gsd-new-project-brief.md` and ADRs 0001, 0010, 0012, and 0014 for device-user parity, AxeOS compatibility, evidence, and Ultra 205 priority.
- `docs/parity/checklist.md` for current `SYS-004`, `CFG-005`, `API-002`, `API-003`, `PWR-006`, `THR-001`, `IO-001`, `SELF-001`, `SAFE-12`, and `SAFE-13` evidence boundaries.
- `docs/parity/evidence/phase-14-safety-hardware-evidence-completion.md` and `docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/summary.md` for the missing fresh sensor, API correlation, self-test, and watchdog evidence.
- `docs/parity/evidence/phase-17-live-http-api-and-static-evidence/` for the current live system-info/WebSocket route evidence and zero-compatible/synthetic field gaps.
- `reference/esp-miner/main/http_server/system_api_json.c`, `reference/esp-miner/main/http_server/openapi.yaml`, `reference/esp-miner/main/nvs_config.c`, `reference/esp-miner/main/system.c`, `reference/esp-miner/main/task_monitor.c`, and `reference/esp-miner/main/self_test/self_test.c` for upstream operator-visible behavior.
- `crates/bitaxe-safety/src/power.rs`, `thermal.rs`, `watchdog.rs`, and `self_test.rs`; `crates/bitaxe-api/src/snapshot.rs`, `wire.rs`, and `settings.rs`; `crates/bitaxe-config/src/nvs.rs`; and `firmware/bitaxe/src/runtime_snapshot.rs`, `http_api.rs`, and `settings_adapter.rs` for the current Rust-owned feature contracts.
- `AGENTS.md`, `AGENTS.bright-builds.md`, `standards/core/architecture.md`, and `standards/core/verification.md` for the local read-only hardware gates, functional-core/imperative-shell design, explicit evidence, and verification boundaries that materially shape this research.

*Feature research for: v1.2 Ultra 205 Operator-Ready Runtime*
*Researched: 2026-07-13*

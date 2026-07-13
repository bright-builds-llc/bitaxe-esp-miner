# Pitfalls Research

**Domain:** Ultra 205 operator-ready read-only telemetry, configuration, provenance, and runtime-health integration
**Researched:** 2026-07-13
**Confidence:** HIGH for repository-specific safety, evidence, and architecture risks; MEDIUM-HIGH for live shared-I2C behavior until detector-gated hardware evidence exists.

## Critical Pitfalls

### Pitfall 1: Giving Each Peripheral Its Own I2C Owner

**What goes wrong:**
The display, INA260, and EMC2101 adapters independently initialize, retain, or lock I2C0. Startup order changes behavior, one task blocks another, reads intermittently fail, or a driver lifetime outlives the bus object.

**Why it happens:**
The current display path already initializes I2C0 while the safety adapters mostly expose constants and unavailable observations. Adding live reads one adapter at a time makes local progress but violates the single physical bus boundary.

**How to avoid:**
Create one firmware-owned I2C lifecycle and an explicit serialized transaction boundary. Keep device parsing and observation classification pure; keep bus ownership, timeouts, and retries in a thin adapter. Initialization failure must leave every dependent observation explicitly unavailable without panicking or reinitializing behind another task's back.

**Warning signs:**
Multiple `I2cDriver` constructors, duplicated GPIO47/GPIO48/400 kHz constants, device-specific mutexes around the same bus, or display success being treated as proof that sensor transactions work.

**Phase to address:**
Shared I2C lifecycle and read-only sensor acquisition.

### Pitfall 2: Fabricating Freshness At API Request Time

**What goes wrong:**
An old or never-observed sensor value appears fresh because `/api/system/info`, WebSocket, or statistics code stamps it when requested. Operators see plausible numbers after the producer has stalled.

**Why it happens:**
Request handlers know the current time but not when the hardware observation was acquired. Existing compatibility projections sometimes use zero-valued numerics for unavailable telemetry, which can hide the difference unless provenance remains internal and explicit.

**How to avoid:**
Represent acquisition outcome as a typed state carrying producer-owned sequence/time evidence: unavailable, fresh, stale, or failed. Only the acquisition task advances freshness. API and evidence consumers project the same immutable snapshot and never synthesize samples from reads.

**Warning signs:**
Request counters create statistics samples, timestamps advance while sensor sequence does not, stale data is refreshed by cloning it, or zero values appear without an accompanying availability/freshness status.

**Phase to address:**
Telemetry domain contract before live sensor integration.

### Pitfall 3: Treating A Sensor Read As A Safety Or Control Claim

**What goes wrong:**
Fresh INA260 or EMC2101 values are used to promote active voltage, fan, thermal-fault, reset, or mining-safety behavior even though v1.2 only proves observation.

**Why it happens:**
The pure safety crate can mint evidence tokens and plan hardware effects, so a live observation looks close to an active-control prerequisite. Existing parity rows deliberately separate implemented decision logic from hardware-verified effects.

**How to avoid:**
Enforce an observation-only capability type and evidence profile that cannot issue actuator commands. Keep DS4432U writes, fan writes, reset/power sequencing, fault stimulus, self-test hardware submodes, and mining gates outside the milestone. Promote only the exact sensor acquisition and projection rows supported by evidence.

**Warning signs:**
New code calls voltage/fan/reset setters, a read-only wrapper accepts actuator flags, a test requires fault injection, or the checklist promotes active-safety rows from telemetry alone.

**Phase to address:**
Milestone claim contract and every hardware-evidence review.

### Pitfall 4: Returning PATCH Success Before Durable Reload

**What goes wrong:**
The API reports success after validation or write intent, but NVS commit fails, a legacy mirror diverges, reload produces a different value, or reboot restores the old configuration.

**Why it happens:**
The pure persistence plan is already implemented, which can make the firmware adapter appear complete. Real success requires ordered write, commit, reload, and public snapshot replacement; hostname live-apply is a separate best-effort effect.

**How to avoid:**
Keep PATCH atomic at the public boundary. Reject invalid known fields without writes, cap and parse input before persistence, commit all primary and required compatibility keys, reload through the same parser used at boot, compare typed non-secret values, and only then return success. Prove a bounded reboot round trip for supported non-secret settings.

**Warning signs:**
HTTP 200 precedes commit/reload, API state is updated directly from the request body, tests assert emitted writes but not reloaded state, or evidence logs raw PATCH bodies and secret-bearing values.

**Phase to address:**
Settings durability and reboot evidence.

### Pitfall 5: Mixing Build, Runtime, And Reference Identity

**What goes wrong:**
The UI or evidence reports a version that cannot be tied to the running image, source commit, pinned reference commit, package manifest, and build profile. A stale package can look like current firmware.

**Why it happens:**
Identity exists across firmware logs, release manifests, parity reports, and API DTOs, but those sources have different lifetimes. Filling missing values with host checkout state during evidence capture destroys the runtime boundary.

**How to avoid:**
Embed immutable build provenance in the image, project it without host substitution, and correlate it with the flashed package manifest and current detector-gated session. Keep semantic firmware version, source commit, reference commit, build profile, board, and package digest as distinct typed fields.

**Warning signs:**
Evidence scripts call `git rev-parse` to fill a missing runtime field, API and boot log identities disagree, package digest is absent, or a reference commit is labeled as firmware source.

**Phase to address:**
Runtime identity and provenance projection.

### Pitfall 6: Exposing Health Without A Single Source Of Truth

**What goes wrong:**
Retained logs, system info, WebSocket, self-test state, and watchdog markers disagree about whether the device is healthy, stale, blocked, or unavailable.

**Why it happens:**
Each output surface can independently translate adapter errors. Repeated translation creates contradictory enums and allows a log marker to claim progress that the public snapshot never received.

**How to avoid:**
Update one typed runtime-health snapshot from bounded producer events. Project that snapshot into every public surface. Self-test visibility in v1.2 is observational only; missing hardware evidence remains explicit and cannot start hardware submodes. Watchdog health requires advancing producer checkpoints, not merely task-start logs.

**Warning signs:**
String parsing drives health state, APIs map errors separately, watchdog status remains healthy while sequence values stop, or a self-test endpoint can energize hardware.

**Phase to address:**
Runtime-health aggregation and surface correlation.

### Pitfall 7: Capturing Correct Values From The Wrong Session

**What goes wrong:**
Sensor, API, reboot, or health artifacts are individually plausible but come from different firmware builds, USB enumeration epochs, device targets, or time windows.

**Why it happens:**
The repository has many historical evidence roots and local target inputs. Copying an old artifact is easier than proving a same-session chain, especially when raw identifiers must be redacted.

**How to avoid:**
Use a new v1.2 evidence schema with one source commit, reference commit, package manifest, detector result, board 205 identity category, session tag, monotonic ordering, and redacted target-lock provenance. Require one bounded chain from detector through capture, PATCH/reload/reboot where applicable, API/WebSocket correlation, and cleanup.

**Warning signs:**
Artifacts have mismatched commits or session tags, device URLs come from scans or stale logs, timestamps overlap ambiguously, or redaction removes the correlation keys instead of replacing them with safe digests/categories.

**Phase to address:**
Evidence contract first; detector-gated closure last.

### Pitfall 8: Reopening The Archived Mining Diagnostic Lineage Indirectly

**What goes wrong:**
Telemetry or health work adds nonce, ASIC result, share, direct-UART, or pin-level diagnostics because they seem useful for observability, recreating the terminal Phase 28.1.1 loop.

**Why it happens:**
Operator observability and mining diagnosis share runtime/logging infrastructure. Without explicit exclusions, an agent may treat unresolved STR-09, ASIC-11, or CFG-07 as adjacent requirements.

**How to avoid:**
Keep the v1.2 roadmap and evidence allowlist limited to read-only power/thermal, supported settings, provenance, and runtime health. Retain the archived lineage's guards. Any mining re-entry requires a separately approved milestone, genuinely new evidence, a predicted discriminating result, and a hard stopping rule.

**Warning signs:**
Plans mention Phase 28.1.1 descendants, ASIC UART frames, nonce production, `mining.submit`, pool credentials, direct UART, pin manipulation, or checklist promotion for STR-09/ASIC-11/CFG-07.

**Phase to address:**
Milestone scope validation and roadmap review.

## Technical Debt Patterns

| Shortcut | Immediate Benefit | Long-term Cost | When Acceptable |
| --- | --- | --- | --- |
| Per-device I2C initialization | Fast first read | Bus ownership races and brittle startup order | Never in production firmware |
| Numeric zero for missing telemetry without status | Wire compatibility | Operators cannot distinguish zero from unavailable | Only when paired with explicit typed availability projection |
| Updating API state from PATCH input | Simple response | State diverges from durable NVS | Never for success responses |
| Host-derived runtime provenance | Easy evidence completion | Stale images appear current | Never for runtime identity claims |
| Self-test start as a health probe | Reuses existing model | Can cross into active hardware behavior | Never in v1.2; expose state only |
| Broad parity promotion from one hardware run | Quick closure | Unsupported control and mining claims | Never; promote rows independently |

## Integration Gotchas

| Integration | Common Mistake | Correct Approach |
| --- | --- | --- |
| I2C display ↔ sensors | Treating successful display startup as sensor-bus proof | Share one bus owner and prove each device transaction independently |
| INA260/EMC2101 ↔ safety core | Converting any number into fresh evidence | Parse status, bounds, producer time/sequence, and failure explicitly |
| Sensor snapshot ↔ API/WebSocket | Sampling on each request | Project the same producer-owned immutable snapshot |
| PATCH ↔ NVS | Returning after emitted writes | Commit, reload, compare typed state, then publish success |
| Build ↔ runtime identity | Filling API gaps from the host checkout | Embed at build time and correlate with package/session evidence |
| Watchdog ↔ health | Logging supervisor startup once | Require bounded advancing checkpoints and observable stall state |
| Evidence ↔ device target | Using scans, mDNS, router state, or stale URLs | Use explicit or same-session origin-only target lock under repo rules |

## Performance Traps

| Trap | Symptoms | Prevention | When It Breaks |
| --- | --- | --- | --- |
| Sensor polling while holding shared bus lock during sleeps/retries | Display or health updates stall | Bound each transaction and release ownership between steps | First missing or slow peripheral |
| Polling faster than operator surfaces consume | Log noise, task pressure, redundant I2C traffic | Use a documented producer cadence and coalesced projections | Normal long-running operation |
| Unbounded NVS retry/reload loop | PATCH handler stalls, watchdog risk | Typed finite attempts and visible failure | First flash/NVS error |
| Formatting large provenance/health payloads in hot tasks | Delayed sensor/watchdog checkpoints | Build compact typed snapshots; serialize at API boundary | Concurrent API/WebSocket use |

## Security Mistakes

| Mistake | Risk | Prevention |
| --- | --- | --- |
| Logging PATCH bodies or full NVS snapshots | Wi-Fi or pool secrets leak | Evidence only non-secret field categories and redacted typed outcomes |
| Committing raw device URL, IP, MAC, USB node, or session identity | Local environment disclosure | Use the repository's redacted evidence profile and safe digests/categories |
| Allowing settings routes to expose secret fields on readback | Credential disclosure | Preserve existing wire/redaction contracts and test denylisted fields |
| Reusing old evidence roots | False current-build claims | Require exact current source/package/session binding |

## UX Pitfalls

| Pitfall | User Impact | Better Approach |
| --- | --- | --- |
| Showing `0` without availability | Operator reads missing sensor as a real zero | Pair compatibility numerics with fresh/stale/unavailable/failed state |
| Reporting PATCH accepted but not durable | Settings revert after reboot | Report success only after commit and reload; show a generic failure otherwise |
| Ambiguous version strings | Operator cannot identify the running image | Distinguish semantic version, source, reference, profile, and package identity |
| A single green “healthy” flag | Hides stale sensors or watchdog stalls | Expose bounded component health and a conservative aggregate state |

## "Looks Done But Isn't" Checklist

- [ ] **Shared I2C:** One bus exists, but each peripheral has independently proved bounded transactions and failure isolation.
- [ ] **Fresh telemetry:** Values exist, but producer sequence/time advances independently of API requests and stale transition is tested.
- [ ] **Settings PATCH:** HTTP success exists, but commit, typed reload, reboot persistence, invalid atomic rejection, and secret redaction are all proved.
- [ ] **Provenance:** A version string exists, but it is bound to the running image, package, source, reference, board, and session.
- [ ] **Runtime health:** Logs exist, but API/WebSocket/self-test/watchdog views agree from one snapshot and stalls become visible.
- [ ] **Hardware evidence:** A capture exists, but detector, package, target lock, session ordering, cleanup, and exact non-claims are validated.

## Recovery Strategies

| Pitfall | Recovery Cost | Recovery Steps |
| --- | --- | --- |
| Conflicting I2C owners | MEDIUM | Stop live integration, introduce one owner, add fake-bus sequencing/failure tests, then repeat bounded hardware proof |
| False freshness | MEDIUM | Invalidate promoted evidence, move acquisition metadata to producer state, add stale/stall tests, recapture |
| Premature PATCH success | MEDIUM | Make publish conditional on commit/reload, restore prior snapshot on failure, add reboot evidence |
| Identity mismatch | LOW | Reject the evidence root, rebuild/package/flash once, and capture one exact current-session chain |
| Scope crossed into actuation/mining | HIGH | Stop the phase, revert unsupported promotion/effects, record a non-claim, and replan under a separate milestone if authorized |

## Pitfall-to-Phase Mapping

| Pitfall | Prevention Phase | Verification |
| --- | --- | --- |
| False freshness and ambiguous projection | Telemetry contract | Pure state-transition and projection tests for fresh/stale/unavailable/failed |
| I2C ownership and read failures | Shared I2C and sensor acquisition | Fake-bus ordering/failure tests plus bounded read-only hardware capture |
| Non-durable settings | Settings persistence | PATCH→commit→reload tests and detector-gated reboot round trip |
| Mixed provenance and health sources | Runtime identity and health | Cross-surface equality and advancing-checkpoint tests |
| Wrong-session or over-broad evidence | Evidence closure | Strict schema validation, redaction, same-session correlation, and row-specific parity review |

## Sources

- `.planning/PROJECT.md` — v1.2 boundary, terminal mining lineage, and exact-claim constraints.
- `.planning/milestones/v1.1-MILESTONE-AUDIT.md` — accepted gaps and integration evidence limits.
- `.planning/RETROSPECTIVE.md` — stopping rules, no-promotion discipline, and hardware-evidence lessons.
- `docs/parity/checklist.md` — current evidence boundaries for SYS-004, CFG-005, API-002/003, PWR-006, THR-001, IO-001, SELF-001, and SAFE-10–13.
- `crates/bitaxe-api`, `crates/bitaxe-config`, `crates/bitaxe-safety`, and `firmware/bitaxe` — existing pure contracts and firmware ownership boundaries.
- `AGENTS.md` and `standards/core/architecture.md` — hardware authorization, evidence redaction, functional-core/imperative-shell, and illegal-state constraints.

*Pitfalls research for: v1.2 Ultra 205 Operator-Ready Runtime*
*Researched: 2026-07-13*

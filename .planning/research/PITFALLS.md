# Pitfalls Research

**Domain:** Ultra 205 trusted production mining on existing Rust ESP-IDF Bitaxe firmware
**Researched:** 2026-07-04
**Confidence:** HIGH for repo-specific evidence, safety, redaction, and current implementation boundaries; MEDIUM-HIGH for ESP-IDF watchdog/runtime risks verified from official ESP-IDF v5.5.4 docs; MEDIUM for live-pool behavior until real accepted/rejected-share evidence exists.

## Critical Pitfalls

### Pitfall 1: Treating Controlled No-Share Evidence As Production Mining Proof

**What goes wrong:**
The milestone promotes Phase 21 controlled-runtime evidence to trusted production mining, even though Phase 21 explicitly proved a bounded controlled no-share harness and did not observe accepted shares, rejected shares, successful live nonce/result parsing, full production mining, frequency transition, or active controls.

**Why it happens:**
The existing harness already logs subscribe, authorize, notify, typed BM1366 work dispatch, runtime snapshot updates, watchdog checkpoints, API/WebSocket readiness, and safe-stop. Those markers look close to production success unless the roadmap preserves the evidence boundary between synthetic transcript progress and real socket/ASIC/share outcomes.

**How to avoid:**
Create a first milestone phase that freezes Phase 21 as the baseline and writes a trusted-mining claim ladder: controlled-no-share, live-pool-smoke, accepted-or-rejected-share, bounded production soak, and parity promotion. Require each claim tier to name exact supported subclaims and exact non-claims before implementation starts.

**Warning signs:**
Checklist rows mention "production mining" while evidence still says `controlled-no-share`, `bounded_no_result`, `bounded_no_share`, `bounded_zero_hashrate_inputs`, or `controlled_run_provenance: actual-controlled-run-or-harness`. `STR-008`, `ASIC-004`, `STAT-001`, or share-counter rows move to `verified` without a real pool response and live ASIC result artifact.

**Explicit blockers that should stop execution:**
Stop if the only observed share outcome is no-share, if nonce/result parsing is still diagnostic-only, if the evidence tree lacks a real pool response artifact, or if the redaction review cannot distinguish live values from placeholders.

**Phase to address:**
Phase 1: Baseline Evidence Contract And Claim Ladder.

### Pitfall 2: Reusing Synthetic Safety Tokens To Enable Real Hardware

**What goes wrong:**
Production mining starts from fabricated or fixture-like safety data instead of fresh Ultra 205 power, thermal, fan, voltage, and safe-stop evidence. The current controlled runtime builds a gate with fixed 5.0 V, 2.5 A, 12.5 W, 55 C, `SafetyStatus::Normal`, and `hardware_evidence_ack: true`; that is acceptable only as controlled evidence, not as proof that the connected miner is safe for sustained hashing.

**Why it happens:**
The system already has a functional safety domain model and observe-only firmware adapters, so it is tempting to pass the existing `MiningLoopGate` shape through to production. But v1.0 intentionally left active voltage, fan, thermal, fault, and full safety closure below verified.

**How to avoid:**
Add a prerequisite-safety phase before live ASIC work. It should not attempt full active safety closure unless the roadmap explicitly scopes it. It should define the minimum trusted-mining safety gate: detector success, board-info success, package identity match, fresh or deliberately bounded power/thermal/fan evidence, maximum allowed frequency/voltage, recovery steps, safe-stop proof, and no active voltage/fan writes unless separately evidenced.

**Warning signs:**
Mining code accepts `SafetyCriticalEvidence::hardware_smoke(...)` strings without fresh sensor inputs, `collect_safety_report()` still returns unavailable or zero telemetry, firmware logs `hardware_evidence_pending` while work submission is enabled, or v1.1 tries to verify active voltage/fan behavior as a side effect of mining.

**Explicit blockers that should stop execution:**
Stop if thermal telemetry is unavailable and no bounded safe-bench alternative is documented, if power telemetry is stale or out of range, if fan behavior is unknown under hashing load, if any active voltage/fan/fault command is needed but not phase-gated, or if post-action safe-state markers are missing.

**Phase to address:**
Phase 2: Mining Prerequisite Safety Gate.

### Pitfall 3: Bypassing The Mining Allow Manifest For Faster Bring-Up

**What goes wrong:**
An engineer runs a direct Stratum socket, raw BM1366, voltage, fan, erase, or flash command outside the approved wrappers. The run may produce useful logs but cannot be safely trusted, redacted, repeated, or promoted.

**Why it happens:**
Production mining is hard to reach, and direct commands are attractive when the allow manifest blocks on prerequisites. The existing allow tool already rejects non-205 boards, detector mismatches, board-info failures, package identity mismatches, unapproved surfaces, unsafe tokens, missing abort conditions, missing recovery steps, missing redaction reviewer, and missing safe-state markers. Those guardrails are exactly the point.

**How to avoid:**
Keep all hardware mining evidence behind repo-owned wrappers and extend `tools/parity` before adding new procedure shapes. New production-mining surfaces should add allow-manifest schema fields rather than bypassing the validator.

**Warning signs:**
Evidence commands do not route through `just detect-ultra205`, `tools/parity:report -- mining-allow`, `just flash-monitor`, or an approved script. The `allowed_command` contains raw `stratum`, `raw-bm1366`, `voltage-control`, `fan-control`, `erase-flash`, `write-flash`, or arbitrary `curl` against mutable endpoints.

**Explicit blockers that should stop execution:**
Stop if `just detect-ultra205` does not find exactly one likely Ultra 205, if board-info fails, if the manifest/package source and reference commits differ, if the command is not allow-manifest validated, or if recovery steps and abort conditions are incomplete.

**Phase to address:**
Phase 3: Production Mining Allowlist And Wrapper Upgrade.

### Pitfall 4: Leaking Pool Credentials, Owner Identity, Targets, Or NVS Secrets

**What goes wrong:**
Live pool URL, port, worker, BTC-address-derived username, password, device URL, IP, MAC, Wi-Fi data, API tokens, NVS secret values, or raw target data lands in committed evidence, logs, retained log buffers, WebSocket captures, summaries, or command examples.

**Why it happens:**
Trusted production mining needs real pool credentials and a live device URL. Existing scripts redact many common keys and values, but new production paths will introduce new JSON fields, logs, errors, share payloads, and target values unless redaction is designed with the feature.

**How to avoid:**
Make redaction a phase deliverable, not a final cleanup step. Every new production-mining artifact must have raw and committed forms separated, with committed evidence allowing only category labels such as `pool_config: local-owner-supplied`. Redaction tests should include pool URL, port, worker, password, BTC-address-like usernames, device URL, IP, MAC, NVS keys, Stratum target, extranonce, and common curl/socket error text.

**Warning signs:**
Evidence summaries include real endpoints, ports, workers, addresses, target hex values, `DEVICE_URL`, `/api/system/logs` output, raw `mining.authorize`, raw `mining.submit`, or unredacted `pool-credentials*.json` content. Redaction review is marked pending, blocked, or reviewer-less.

**Explicit blockers that should stop execution:**
Stop before committing or citing evidence if any raw pool value, raw endpoint, target, worker, password, token, Wi-Fi value, NVS secret, unredacted IP/MAC, or local owner address is present in the committed artifact set.

**Phase to address:**
Phase 4: Redaction And Secret-Handling Hardening.

### Pitfall 5: Inferring `DEVICE_URL` From Stale Or Ambiguous Network Evidence

**What goes wrong:**
The test harness points production-mining API/WebSocket probes at the wrong device, an old firmware instance, an unrelated host, or a stale IP from previous evidence. The run appears to pass while not proving the freshly flashed Ultra 205.

**Why it happens:**
HTTP and WebSocket capture is easier once any reachable URL is known. Repo-local rules allow deriving `DEVICE_URL` only from the same detector-gated verification session and only from a corresponding repo-owned monitor run with exactly one origin-only candidate.

**How to avoid:**
Keep `network_scan: disabled` for evidence. Require explicit `--device-url` or same-session target-lock derivation, and record target provenance without committing the raw URL. API/WebSocket mining telemetry should fail closed when the target is missing, malformed, redacted, stale, or ambiguous.

**Warning signs:**
Scripts mention mDNS, ARP, router state, network scans, stale monitor logs, or "last known" URLs. Target lock files lack a current package/source commit, detector evidence, and origin-only validation.

**Explicit blockers that should stop execution:**
Stop if zero, multiple, redacted, malformed, or stale device URL candidates exist; if the device URL came from outside the current detector-gated run; or if API/WebSocket evidence cannot be tied to the same package and port as the mining run.

**Phase to address:**
Phase 3: Production Mining Allowlist And Wrapper Upgrade.

### Pitfall 6: Starving ESP-IDF Watchdogs, Wi-Fi, HTTP, Or Telemetry During Mining

**What goes wrong:**
A live socket loop, ASIC read loop, nonce parser, share submission path, or soak harness monopolizes a task/core. The miner reboots, TWDT warns without recovery, API/WebSocket telemetry goes stale, Wi-Fi drops, or share work continues while safe-stop is delayed.

**Why it happens:**
Official ESP-IDF v5.5.4 docs state that TWDT detects tasks running without yielding and that IWDT detects blocked interrupts. IDF FreeRTOS on ESP32-S3 is SMP with core affinity, best-effort scheduling, and Core 0 timekeeping implications; generic FreeRTOS assumptions are not enough. The current safety supervisor models 25 ms step budgets, 100 ms yield intervals, and consecutive-step yield limits, but the production mining loop must actually apply those boundaries.

**How to avoid:**
Design socket, ASIC, telemetry, and safety work as bounded steps with explicit yield/feed checkpoints. Add watchdog evidence that proves no unexpected reset, panic, unsafe marker, or silence while mining is active. Keep long-running flash/OTA/recovery out of this milestone unless separately phase-gated.

**Warning signs:**
Logs show `Task watchdog got triggered`, `Interrupt wdt timeout`, missing `watchdog_yield_checkpoint`, stale `/api/ws/live` frames, stalled `/api/system/info`, no retained logs during mining, or a mining thread without a bounded timeout on socket read or UART read.

**Explicit blockers that should stop execution:**
Stop if watchdog checkpoints stop advancing, if API/WebSocket probes cannot respond during hashing, if no timeout/yield is visible around socket or UART waits, if OpenOCD/JTAG-disabled watchdogs are part of the evidence path, or if safe-stop takes longer than the documented bound.

**Phase to address:**
Phase 6: Watchdog And Runtime Responsiveness Soak.

### Pitfall 7: Conflating Diagnostic BM1366 Work With Trusted Production Work

**What goes wrong:**
The firmware treats diagnostic job `0x28`, diagnostic work fields, timeout behavior, or chip-detect UART reachability as proof that pool-derived jobs can be initialized, dispatched, mined, parsed, and submitted.

**Why it happens:**
The existing ASIC adapter can run chip-detect and work-result diagnostics and can parse result frames in pure tests. But Phase 21 evidence kept full BM1366 initialization, active reset/power sequencing, frequency transition, successful live nonce/result parsing, and accepted serial transport under mining load below verified.

**How to avoid:**
Create a dedicated ASIC production-work phase that requires an explicit transition from diagnostic command paths to pool-derived work paths. It should prove initialization state, job ID allocation, clean-jobs invalidation, work queue ownership, nonce/result validation against current jobs, timeout handling, and reset-low fail-closed behavior.

**Warning signs:**
Production work still uses `SendDiagnosticWork`, `diagnostic_job_frame`, fixed job `0x28`, synthetic fields, or only `bounded_no_result` evidence. Result parsing tests pass but live logs lack a valid nonce/result tied to the active pool job and difficulty.

**Explicit blockers that should stop execution:**
Stop if full BM1366 init is not evidenced, if only diagnostic work dispatch ran, if valid jobs are not tracked, if stale nonce results can map to current shares, if UART timeouts do not fail closed, or if reset-low recovery is not observed after a fault.

**Phase to address:**
Phase 5: Trusted BM1366 Init, Work, And Result Path.

### Pitfall 8: Shipping A Live Stratum Path That Only Handles The Happy Path

**What goes wrong:**
The miner connects to one pool once, submits a share or no-share, and appears done. Production use then fails on reconnects, fallback pool behavior, malformed frames, difficulty changes, `clean_jobs`, extranonce changes, slow/no responses, DNS/socket errors, or pool rejections.

**Why it happens:**
The pure Stratum crate already handles many typed messages and fake-pool scenarios, while the current firmware runtime uses controlled transcripts. The hard part is the imperative shell: socket lifecycle, reconnect/backoff, fallback activation, authorization failure, clean job invalidation, and redacted live logs.

**How to avoid:**
Implement the live Stratum adapter as a thin shell over existing typed protocol/state logic. Require deterministic fake-pool tests before hardware runs, then live smoke evidence for subscribe, authorize, difficulty, notify, submit, accepted/rejected response, reconnect/fallback, and fail-closed safe-stop.

**Warning signs:**
Socket code parses JSON ad hoc instead of using typed messages, credentials appear in logs, no reconnect state appears in `MiningRuntimeState`, fallback fields stay unused, or share outcome is inferred from socket success rather than a Stratum response.

**Explicit blockers that should stop execution:**
Stop if subscribe/authorize does not complete, if pool difficulty/extranonce are missing, if `clean_jobs` is ignored, if reconnect loops have no backoff/yield, if fallback pool behavior is untested, or if accepted/rejected outcomes are not tied to a parsed Stratum response.

**Phase to address:**
Phase 7: Real Stratum Socket And Share Lifecycle.

### Pitfall 9: Reporting Incorrect Hashrate, Counters, Or Scoreboard State

**What goes wrong:**
The API, WebSocket, logs, or scoreboard report misleading hashrate, accepted/rejected shares, best difficulty, uptime, pool lifecycle, or work submission state. Users may believe the miner is earning, safe, or stable when the underlying runtime has not proven that.

**Why it happens:**
Current state models contain `HashrateInputs`, share counters, pool lifecycle, fallback state, and work submission gates. Phase 21 intentionally used zero hashrate inputs and zero share counters. Production mining needs real derivation from accepted work, elapsed time, nonces, difficulty, and share responses, plus consistent projection into AxeOS-compatible surfaces.

**How to avoid:**
Add a statistics phase after real Stratum/ASIC evidence. Define source-of-truth events and derive API/WebSocket/log fields from the same `MiningRuntimeState`. Require correlation artifacts: retained log markers, `/api/system/info`, `/api/ws/live`, scoreboard/counter fields, and raw internal event summaries with secrets redacted.

**Warning signs:**
Hashrate is nonzero while no valid nonce/result exists, accepted/rejected counters change without a parsed submit response, API and WebSocket disagree, best difficulty updates on rejected shares, stale samples keep reporting active mining, or scoreboard remains an empty fixture while production claims are promoted.

**Explicit blockers that should stop execution:**
Stop if statistics are computed from expected hashrate or configured frequency instead of observed runtime inputs, if counters are not event-sourced from share responses, if API/WebSocket samples cannot be correlated to the mining window, or if stale data persists after safe-stop.

**Phase to address:**
Phase 8: Mining Telemetry, Statistics, And API Projection.

### Pitfall 10: Safe-Stop Works In Logs But Not In State Or Hardware

**What goes wrong:**
The run logs `safe_stop_status=complete`, but mining work, ASIC UART, pool socket, runtime state, API telemetry, or hardware control remains active. Recovery after failure becomes ambiguous.

**Why it happens:**
The current controlled runtime publishes safe-stop as a retained marker and replaces an API-visible snapshot. A production miner needs safe-stop to coordinate multiple effectful pieces: Stratum socket, work queue, ASIC adapter, safety gate, telemetry producer, and post-action verification.

**How to avoid:**
Make safe-stop a state transition with observable postconditions, not only a log line. Require post-stop artifacts showing mining disabled, hardware control disabled, work submission disabled, socket closed or inactive, queue drained/invalidated, watchdog still responsive, API/WebSocket projected stopped state, and reset/fail-closed behavior where applicable.

**Warning signs:**
Only retained logs mention safe-stop, but `/api/system/info` or `/api/ws/live` still shows active lifecycle; work queue still contains valid jobs; pool socket reconnects after stop; or ASIC reset/fan/voltage state is not recorded.

**Explicit blockers that should stop execution:**
Stop if any post-action safe-state marker is missing, if stop cannot be invoked after an error path, if hardware effects cannot be observed or explicitly bounded, or if API/WebSocket still advertises active mining after stop.

**Phase to address:**
Phase 9: Safe-Stop And Recovery Evidence.

### Pitfall 11: Expanding Scope Into Active Controls, OTA, Or Other Boards During Mining

**What goes wrong:**
The trusted mining milestone quietly adds active voltage/fan tuning, destructive recovery, OTA/rollback, display/input, Stratum v2, BAP, or non-205 board support. The phase becomes unsafe or impossible to prove cleanly.

**Why it happens:**
Production mining touches many adjacent systems. Active fan/thermal/voltage behavior and recovery flows are real production concerns, but v1.1 scope is trusted Ultra 205 BM1366 Stratum v1 mining with exact evidence, not full active safety or release/update closure.

**How to avoid:**
Use phase boundaries and allow manifests to keep adjacent surfaces below verified unless explicitly planned with recovery paths. Record exact non-claims for active voltage/fan/fault/self-test, OTAWWW, rollback, erase, interrupted update, non-205 boards, and Stratum v2.

**Warning signs:**
Phase plans include raw erase/write, OTA, voltage-control, fan-control, non-205 board rows, or "all boards" language. Evidence summaries claim "production ready" without a subclaim table and non-claim list.

**Explicit blockers that should stop execution:**
Stop if a procedure requires destructive or fault-injection behavior not documented in the active phase, if board is not `205`, if raw voltage/fan controls are needed, or if parity promotion depends on surfaces outside the v1.1 target.

**Phase to address:**
Phase 10: Evidence Closure, Rollout Limits, And Parity Promotion.

### Pitfall 12: Persisting Production Credentials Or Runtime State Incorrectly In NVS

**What goes wrong:**
Pool settings are stored under incompatible keys, committed without reload verification, exposed through API/logs, or left active after a failed run. NVS edge cases can cause stale credentials, wrong pool endpoints, or false evidence about what firmware consumed.

**Why it happens:**
Phase 21 uses a pool input bridge that PATCHes settings and polls retained logs for `phase21_pool_settings_consumed=true`. ESP-IDF NVS keys and namespaces have 15-character limits, NVS can recover from interrupted writes with caveats, and NVS encryption is a separate security posture. Production mining must preserve compatibility while avoiding evidence leakage.

**How to avoid:**
Keep pool-credential handling in repo-owned bridge code, verify settings PATCH -> NVS write -> commit -> reload -> runtime consumption, and scrub committed artifacts. Treat local credentials as runtime inputs only; never read or summarize their contents beyond allowed category labels.

**Warning signs:**
New keys exceed NVS length limits, production code invents new pool fields without API compare, logs print PATCH bodies, retained logs include usernames/passwords, or settings remain active after a failed safe-stop without an explicit operator decision.

**Explicit blockers that should stop execution:**
Stop if pool credential JSON is missing/invalid, if settings PATCH fails, if runtime consumption marker is missing, if NVS reload cannot prove the same values were consumed without printing them, or if redaction cannot prove credentials stayed local.

**Phase to address:**
Phase 4: Redaction And Secret-Handling Hardening, with verification in Phase 7.

## Technical Debt Patterns

Shortcuts that seem reasonable but create long-term problems.

| Shortcut | Immediate Benefit | Long-term Cost | When Acceptable |
|----------|-------------------|----------------|-----------------|
| Reusing Phase 21 controlled markers as v1.1 acceptance criteria | Faster roadmap closure | False production-mining and share-parity claims | Never for `verified`; acceptable only as baseline context |
| Keeping safety gate values as fixtures | Enables early integration | Can energize hardware without fresh safety proof | Only in controlled harnesses with explicit non-claims |
| Adding direct socket logic in firmware adapters | Quick live-pool connection | Duplicates typed protocol logic and misses tests | Only as thin I/O shell over typed Stratum domain functions |
| Recording full raw logs and redacting later | Easier diagnosis | High chance of leaked pool/user/device data | Raw local-only artifacts may exist, but committed evidence must be redacted before citation |
| Treating one public pool as the protocol test suite | Reduces setup | Reconnect, fallback, difficulty, and rejection bugs escape | Never as the only evidence; use fake-pool and live-pool layers |
| Computing hashrate from configured frequency | Produces plausible UI | Misleads operators when ASIC results or shares are absent | Only if labeled as expected/configured, not live hashrate |
| Promoting active fan/voltage controls inside mining | Appears production-like | Unsafe scope creep and hardware damage risk | Never unless a separate active-control phase documents recovery and evidence |
| Relying on JTAG/debug sessions for watchdog evidence | Easier diagnosis | OpenOCD can disable watchdogs, invalidating runtime proof | Debugging only, not acceptance evidence |

## Integration Gotchas

Common mistakes when connecting production mining surfaces.

| Integration | Common Mistake | Correct Approach |
|-------------|----------------|------------------|
| Ultra 205 hardware detection | Reusing an old port or skipping board-info | Run `just detect-ultra205`; require exactly one likely port and passing ESP32-S3 board-info for every hardware run |
| Mining allow manifest | Editing scripts to bypass validation when blocked | Extend `tools/parity` schema and tests for new approved claim tiers or wrappers |
| Pool credentials | Printing, summarizing, or committing owner-supplied pool values | Pass credential file as local runtime input; committed evidence records only `pool_config: local-owner-supplied` |
| Device URL | Discovering by scan, mDNS, router, or stale logs | Use explicit `--device-url` or same-session origin-only derivation from repo-owned monitor evidence |
| Stratum socket | Treating TCP connect as mining success | Require subscribe, authorize, difficulty/extranonce, notify, submit, and parsed accepted/rejected response evidence |
| BM1366 UART | Treating diagnostic timeout as production behavior | Separate diagnostic command paths from pool-derived production work/result paths |
| API/WebSocket | Capturing telemetry outside the mining window | Correlate samples to source commit, package, detector run, pool lifecycle, share counters, and safe-stop window |
| Safety adapter | Passing `hardware_evidence_ack` through as a boolean | Require typed evidence tokens derived from fresh or explicitly bounded safety observations |
| NVS/settings | Assuming PATCH success means runtime consumed settings | Verify commit, reload, and redacted runtime consumption markers |

## Performance Traps

Patterns that work in controlled smoke but fail during real mining.

| Trap | Symptoms | Prevention | When It Breaks |
|------|----------|------------|----------------|
| Blocking socket/UART reads without bounded timeouts | TWDT warnings, stale telemetry, delayed safe-stop | Bound every wait and emit watchdog checkpoints | First slow pool, missing ASIC result, or network stall |
| Single task owns socket, ASIC, safety, and telemetry | API/WebSocket pauses during mining | Split effectful loops or use bounded cooperative steps | Any sustained share/nonce workload |
| Heavy JSON/log work in hot path | Dropped frames, latency, heap pressure | Keep retained markers compact and move detailed evidence to host scripts | Live pool errors or verbose debug mode |
| No power-management lock review | UART/I2C timing or interrupt latency changes unexpectedly | Review ESP-IDF power management locks and peripheral clock assumptions | DFS/light-sleep enabled or Wi-Fi modem-sleep interactions |
| Long soak without safe-stop checkpoints | Ambiguous recovery after stall | Bounded soak windows with periodic watchdog, API, WebSocket, and safe-stop checks | First production soak over a few minutes |

## Security Mistakes

Domain-specific security issues beyond general firmware hygiene.

| Mistake | Risk | Prevention |
|---------|------|------------|
| Committing pool owner identity or worker strings | Exposes wallet-linked owner data and test infrastructure | Redact worker/address-like values; commit only category labels |
| Logging raw `mining.authorize` or `mining.submit` | Leaks credentials or share material | Log protocol phase and redacted status only |
| Persisting credentials without a clear local-only policy | Future evidence or API routes may expose secrets | Treat credentials as local runtime input; verify no committed NVS/API/log secret output |
| Using unredacted target/share data in evidence | Reveals private endpoints or pool-specific details | Redact raw target/endpoints before committed artifacts |
| Assuming NVS is tamper-resistant | Physical access can alter/erase data when encryption is not configured | Avoid claims about credential-at-rest security unless NVS encryption and release policy are explicitly verified |
| Running unsafe hardware commands outside wrappers | Device damage or unrecoverable state | Keep destructive and active-control flows phase-gated with documented recovery |

## UX Pitfalls

Common operator-facing mistakes in trusted mining.

| Pitfall | User Impact | Better Approach |
|---------|-------------|-----------------|
| Showing "Mining" when prerequisites are blocked | Operator believes the unit is earning or safe | Show explicit lifecycle: blocked, connecting, authorized, active, safe-stopped, error |
| Reporting expected hashrate as actual hashrate | User trusts inflated performance | Distinguish expected/configured hashrate from observed rolling hashrate |
| Hiding rejected-share reasons | Pool configuration issues are hard to diagnose | Record redacted rejection reason categories and counters |
| Safe-stop not visible through API/WebSocket | User cannot trust remote stop status | Project safe-stop state through retained logs, `/api/system/info`, and `/api/ws/live` |
| Over-broad release language | Users assume full active safety and OTA/recovery closure | Release notes must list exact supported claims and exact non-claims |

## "Looks Done But Isn't" Checklist

Things that appear complete but are missing critical pieces.

- [ ] **Live Stratum:** TCP socket connected and authorized, but no parsed accepted/rejected submit response exists.
- [ ] **ASIC work:** BM1366 diagnostic work dispatch passed, but pool-derived job dispatch and live result parsing are still missing.
- [ ] **Safety:** `hardware_evidence_ack=true` appears, but fresh/bounded power, thermal, fan, and safe-stop evidence are not present.
- [ ] **Stats:** API/WebSocket show active mining, but hashrate and counters are still zero, expected-only, stale, or uncorrelated.
- [ ] **Watchdog:** A 10-second smoke passes, but no bounded soak proves watchdog/API/WebSocket responsiveness under mining load.
- [ ] **Redaction:** Main logs are redacted, but raw pool input bridge, curl errors, retained logs, WebSocket captures, or summaries still contain secrets.
- [ ] **Device target:** HTTP probes pass, but target provenance is stale, ambiguous, or not tied to the same flash-monitor session.
- [ ] **Safe-stop:** The log marker exists, but queue/socket/API/hardware postconditions are not verified.
- [ ] **Parity:** Code exists and tests pass, but checklist rows are promoted without matching hardware-smoke, soak, or hardware-regression evidence.

## Recovery Strategies

When pitfalls occur despite prevention, how to recover.

| Pitfall | Recovery Cost | Recovery Steps |
|---------|---------------|----------------|
| False production-mining claim | MEDIUM | Revert checklist/status promotion, reclassify evidence as controlled/no-share, add non-claim note, rerun `just parity` |
| Secret leaked into committed evidence | HIGH | Stop citation, remove/redact artifact, rotate affected credentials if exposed, rerun deterministic redaction scan, document remediation |
| Unsafe mining enablement | HIGH | Safe-stop immediately, power down if necessary, preserve local raw logs for diagnosis, require new safety phase before retry |
| Watchdog starvation | MEDIUM | Reduce blocking waits, add bounded yields/feed checkpoints, rerun smoke and soak without JTAG-disabled watchdogs |
| ASIC result ambiguity | MEDIUM-HIGH | Split diagnostic and production paths, add job/result correlation tests, rerun hardware evidence from clean boot |
| Stratum reconnect/fallback failure | MEDIUM | Reproduce with fake pool, add typed state transitions and backoff, then repeat live-pool smoke |
| Stats mismatch | MEDIUM | Freeze telemetry promotion, define event source-of-truth, add correlation artifacts and API/WebSocket tests |
| Missing safe-stop postconditions | HIGH | Treat run as failed, add state-machine stop transition and post-action assertions, rerun evidence |

## Pitfall-to-Phase Mapping

How roadmap phases should address these pitfalls.

| Pitfall | Prevention Phase | Verification |
|---------|------------------|--------------|
| Controlled evidence overclaim | Phase 1: Baseline Evidence Contract And Claim Ladder | Checklist status remains below `verified` until evidence tier matches claim; `just parity` reports no validation errors |
| Synthetic safety tokens | Phase 2: Mining Prerequisite Safety Gate | Fresh or explicitly bounded safety observations exist; active-control non-claims remain visible |
| Allow-manifest bypass | Phase 3: Production Mining Allowlist And Wrapper Upgrade | `mining-allow` validates new wrapper, claim tier, allowed inputs, recovery steps, abort conditions, and safe-state markers |
| Secret/target leakage | Phase 4: Redaction And Secret-Handling Hardening | Redaction fixtures and committed artifact scan pass; raw artifacts are not committed |
| Diagnostic/production ASIC confusion | Phase 5: Trusted BM1366 Init, Work, And Result Path | Production work uses pool-derived jobs; live result parsing is tied to valid current jobs and fail-closed timeout paths |
| Watchdog/API starvation | Phase 6: Watchdog And Runtime Responsiveness Soak | Bounded smoke/soak shows watchdog checkpoints, no unexpected reboot/panic/silence, and live API/WebSocket responsiveness |
| Happy-path-only Stratum | Phase 7: Real Stratum Socket And Share Lifecycle | Fake-pool and live-pool evidence cover subscribe, authorize, difficulty, notify, submit, accepted/rejected response, reconnect, and fallback |
| Incorrect statistics | Phase 8: Mining Telemetry, Statistics, And API Projection | Counters/hashrate/scoreboard derive from runtime events and correlate across logs, API, WebSocket, and evidence window |
| Safe-stop ambiguity | Phase 9: Safe-Stop And Recovery Evidence | Post-stop artifacts prove mining disabled, hardware control disabled, work submission disabled, socket/queue inactive, and telemetry stopped |
| Rollout scope creep | Phase 10: Evidence Closure, Rollout Limits, And Parity Promotion | Summary lists exact claims/non-claims; non-205, OTA/recovery, active control, Stratum v2, BAP, and display/input rows stay below verified unless separately evidenced |

## Recommended Phase Order

1. **Baseline Evidence Contract And Claim Ladder** - prevents overclaiming before any risky work.
2. **Mining Prerequisite Safety Gate** - decides what is safe enough to attempt and what remains an exact non-claim.
3. **Production Mining Allowlist And Wrapper Upgrade** - ensures every hardware run is repeatable, scoped, and stoppable.
4. **Redaction And Secret-Handling Hardening** - must precede real pool credentials and target use.
5. **Trusted BM1366 Init, Work, And Result Path** - converts diagnostic ASIC proof into production job/result proof.
6. **Watchdog And Runtime Responsiveness Soak** - proves the runtime stays alive while mining paths run.
7. **Real Stratum Socket And Share Lifecycle** - moves from transcript-driven behavior to live pool behavior.
8. **Mining Telemetry, Statistics, And API Projection** - exposes only verified runtime facts to users and tools.
9. **Safe-Stop And Recovery Evidence** - proves failures and normal stops leave the miner bounded and observable.
10. **Evidence Closure, Rollout Limits, And Parity Promotion** - updates checklist/release language without claiming deferred surfaces.

## Sources

- Local project scope and v1.1 requirements: `.planning/PROJECT.md`
- v1.0 shipped state and carried debt: `.planning/MILESTONES.md`, `.planning/RETROSPECTIVE.md`, `.planning/milestones/v1.0-MILESTONE-AUDIT.md`
- Repo-local hardware, redaction, and evidence rules: `AGENTS.md`
- Parity source of truth and current non-claims: `docs/parity/checklist.md`
- Phase 21 controlled no-share evidence boundary: `docs/parity/evidence/phase-21-live-mining-and-soak-evidence/summary.md`
- Current controlled runtime shell: `firmware/bitaxe/src/controlled_mining_runtime.rs`
- Current ASIC diagnostic adapter: `firmware/bitaxe/src/asic_adapter.rs`
- Current observe-only safety adapter: `firmware/bitaxe/src/safety_adapter.rs`
- Current Phase 21 evidence scripts: `scripts/phase21-live-mining-evidence.sh`, `scripts/phase21-pool-input-bridge.sh`
- Mining allow-manifest validator: `tools/parity/src/mining_allow.rs`
- Stratum runtime state and controlled-runtime model: `crates/bitaxe-stratum/src/v1/controlled_runtime.rs`, `crates/bitaxe-stratum/src/v1/state.rs`
- Safety watchdog model: `crates/bitaxe-safety/src/watchdog.rs`
- ESP-IDF v5.5.4 watchdog docs: https://docs.espressif.com/projects/esp-idf/en/v5.5.4/esp32s3/api-reference/system/wdts.html
- ESP-IDF v5.5.4 FreeRTOS SMP docs: https://docs.espressif.com/projects/esp-idf/en/v5.5.4/esp32s3/api-reference/system/freertos_idf.html
- ESP-IDF v5.5.4 power management docs: https://docs.espressif.com/projects/esp-idf/en/v5.5.4/esp32s3/api-reference/system/power_management.html
- ESP-IDF v5.5.4 NVS docs: https://docs.espressif.com/projects/esp-idf/en/v5.5.4/esp32s3/api-reference/storage/nvs_flash.html

*Pitfalls research for: Ultra 205 trusted production mining*
*Researched: 2026-07-04*

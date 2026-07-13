# Feature Research

**Domain:** Ultra 205 trusted Stratum v1 production mining\
**Researched:** 2026-07-04\
**Confidence:** HIGH for v1.0 evidence boundaries and required user-visible surfaces; MEDIUM for production share acceptance details until v1.1 hardware evidence observes a real accepted or rejected share.

## Feature Landscape

v1.0 shipped a safe, evidence-governed Ultra 205 firmware with build/package/flash/monitor, config/NVS, BM1366 diagnostics, Stratum v1 pure behavior, AxeOS API/static/WebSocket routes, safety decisions, OTA/recovery boundaries, and a controlled no-share mining/soak harness. v1.1 should not rebuild those foundations. It should replace the synthesized controlled transcript with a real Stratum v1 socket path, connect that path to BM1366 work/result/share flow, and prove one bounded production mining session through redacted evidence.

The acceptance boundary is deliberately narrow: an Ultra 205 owner can configure local Wi-Fi and pool credentials, flash the firmware, start a safety-gated production mining run, observe real pool lifecycle and live mining telemetry, and see either at least one accepted or rejected share or an explicit milestone failure if safe prerequisites cannot produce that outcome. Controlled no-share behavior remains useful as regression evidence, but it is not share proof.

### Table Stakes (Users Expect These)

Features users assume exist. Missing these = trusted production mining is not credible.

| Feature | Why Expected | Complexity | Dependencies On v1.0 | Evidence Needed |
|---------|--------------|------------|----------------------|-----------------|
| Real Stratum v1 socket lifecycle | Owners expect configured pool settings to drive actual TCP Stratum I/O, not a synthesized transcript. | HIGH | NVS pool settings, Wi-Fi flash seeding, Stratum v1 message types, controlled runtime state model, AxeOS settings route. | Detector-gated Ultra 205 evidence showing connect, subscribe, authorize, difficulty/extranonce, notify/job, reconnect or safe-stop markers from real socket I/O with all pool values redacted. |
| Redacted pool credential handling | Pool URL, worker, BTC-address-derived usernames, passwords, endpoints, and tokens are sensitive local owner inputs. | MEDIUM | `wifi-credentials.json` and `pool-credentials*.json` local-input policy, redaction review scripts, retained-log discipline. | Redaction review proving no raw pool URL, port, user, worker, address, password, token, device URL, Wi-Fi secret, endpoint, or NVS secret appears in committed evidence. |
| Trusted BM1366 initialization gate | Production mining requires the BM1366 to be initialized for real work, not only chip-detect or diagnostic timeout behavior. | HIGH | BM1366 pure init plan, UART adapter, reset adapter, diagnostic chip-detect/work-result evidence, safety gate tokens. | Hardware-regression or tightly bounded hardware-smoke evidence that names board `205`, source/reference commits, init stage markers, reset/power/frequency prerequisites used, and final go/no-go state. |
| Pool-derived work dispatch | A production miner must transform live `mining.notify` data into BM1366 work instead of dispatching a fixed diagnostic job. | HIGH | Coinbase/merkle work builder, work queue, guarded dispatch planner, BM1366 work packet logic. | Evidence linking a redacted real notify/job lifecycle to typed BM1366 work dispatch markers without raw ASIC frames or raw pool job secrets in committed artifacts. |
| Live ASIC result and nonce parsing | Share submission cannot be claimed until live BM1366 results are parsed from hardware under mining load. | HIGH | BM1366 result parser, UART transport, runtime state counters, guarded mining loop. | Hardware evidence showing at least one valid result/nonce parse or a clear failure state. If no valid result occurs, accepted/rejected share remains unclaimed. |
| Share submission and accepted/rejected outcome | The milestone goal requires at least one real pool response to a submitted share, or an explicit failure if it cannot be achieved safely. | HIGH | Stratum submit message serializer, runtime share counters, API mining mapper. | Redacted evidence showing `mining.submit` was sent for a live ASIC-derived share and pool response was accepted or rejected. Do not overclaim accepted/rejected behavior from synthetic responses or no-share soak. |
| Mining prerequisite safety gate | Owners need assurance that production mining only starts when minimum safe power, thermal, fan, voltage, and evidence prerequisites are satisfied. | HIGH | Pure safety controllers, safety telemetry DTOs, watchdog status, fail-closed mining gate, Phase 20/21 safety boundaries. | Evidence that the gate blocks when prerequisites are missing and allows only the documented bounded mining mode. Claims should stay limited to mining prerequisites, not full active safety closure. |
| Live hashrate and share statistics | Owners expect API and AxeOS surfaces to show live hashrate, share counts, pool difficulty, lifecycle, work submission, and rejected reasons. | MEDIUM | `MiningRuntimeState`, `MiningStateWire`, statistics mapper, WebSocket live telemetry route. | `/api/system/info`, `/api/system/statistics`, and `/api/ws/live` captures correlated to the same mining session with redacted target data and non-zero values only when produced by the run. |
| Scoreboard population | Accepted or high-difficulty shares should appear as scoreboard entries in the upstream-compatible shape. | MEDIUM | `ScoreboardEntry`, `scoreboard_response`, share difficulty tracking. | API evidence for `/api/system/scoreboard` showing entries only when live results justify them; empty scoreboard remains acceptable for rejected-share-only evidence if documented. |
| Watchdog and safe-stop behavior | Production mining must not starve firmware services, panic, silently hang, or leave work submission enabled after stop/failure. | MEDIUM | Watchdog checkpoints, safe-stop markers, retained logs, bounded soak pattern. | Bounded session evidence with watchdog checkpoints, no unexpected reboot/panic/silence markers, and final `mining=disabled`, `hardware_control=disabled`, `work_submission=disabled` safe-stop state. |
| Evidence-governed claim promotion | v1.1 must advance only exact claims proven by artifacts. | MEDIUM | Parity checklist, release guide evidence rules, redaction review, `just parity`. | Checklist updates for STR/ASIC/STAT/API/SAFE rows cite exact v1.1 artifacts and preserve non-claims for unobserved shares, active controls, OTA/recovery, display/input, and non-205 boards. |

### Differentiators (Competitive Advantage)

Features that set the project apart. Not required for a miner to run, but valuable because they make Rust firmware trustworthy.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| Exact-claim mining evidence ledger | Owners and contributors can see exactly what was proven on real Ultra 205 hardware and what remains a non-claim. | MEDIUM | Keep one redacted v1.1 evidence root with package manifest, detector, board-info, commands, logs, API/WebSocket captures, share outcome, and conclusion. |
| Safety-gated production enablement | Production mining becomes opt-in and prerequisite-bound instead of silently enabled by compile-time or settings accidents. | HIGH | Preserve fail-closed defaults; make the runtime explain why mining is blocked. |
| Redacted observability by default | The firmware can be debugged and audited without leaking owner pool credentials or private network details into committed evidence. | MEDIUM | Retain labels and lifecycle markers, not secrets. Raw developer artifacts can exist locally but must not be promoted. |
| Pure core, thin hardware shell | Stratum, work construction, result classification, counters, statistics, and safety decisions remain testable without hardware. | MEDIUM | Build new production logic as domain types first, then connect ESP-IDF sockets/UART in adapters. |
| Share outcome honesty | A rejected share is a valid milestone outcome if the pool really rejected a live ASIC-derived submission. | LOW | Acceptance should require real pool response evidence, not "accepted-only" optimism. |
| Owner-ready operator workflow | A single documented flow can detect, flash, seed Wi-Fi/pool settings, run bounded mining, capture telemetry, safe-stop, and redact evidence. | MEDIUM | Best implemented as repo-owned `just` or script surface that follows existing detector and credential rules. |

### Anti-Features (Commonly Requested, Often Problematic)

Features that seem good but create problems.

| Feature | Why Requested | Why Problematic | Alternative |
|---------|---------------|-----------------|-------------|
| Claiming production mining from controlled no-share soak | v1.0 already has working controlled markers, so it is tempting to promote them. | Phase 21 explicitly did not observe accepted/rejected shares, live nonce parsing, or full production mining. | Treat controlled no-share as regression/preflight only; require real socket plus live ASIC-derived share evidence for v1.1 claims. |
| Hardcoded transcript or fake pool fallback as acceptance evidence | Deterministic tests are easier than real pool variability. | It bypasses the owner-visible behavior this milestone is about. | Keep fake/controlled fixtures for unit tests and preflight, then run a real pool session for acceptance. |
| Logging raw Stratum messages, pool endpoints, workers, or ASIC frames | Raw logs make debugging easier. | They can leak owner secrets, addresses, endpoints, or proprietary pool details and violate evidence policy. | Log redacted lifecycle/status markers and store raw artifacts only in ignored local paths when absolutely necessary. |
| Starting mining when safety evidence is missing | It may produce a share faster. | Unsafe hardware-control surfaces can damage hardware and invalidate the milestone. | Fail closed with a user-visible blocked reason and record missing prerequisites. |
| Full active safety closure inside v1.1 | Production mining naturally raises voltage/fan/thermal questions. | Full active control, fault stimulus, and recovery closure are broader than the milestone and require separate evidence. | Prove only the minimum mining prerequisite gate; leave full active safety parity as an exact non-claim. |
| Unbounded soak or stress mining | Longer runs feel more convincing. | It expands risk before watchdog, safe-stop, and thermal/power prerequisites are fully proven. | Use bounded sessions with documented timeout, stop condition, watchdog checkpoints, and recovery path. |
| Network scanning or stale target inference | It can find the device when logs are redacted. | Repo rules prohibit deriving `DEVICE_URL` from scans, stale logs, mDNS, ARP, router state, or unrelated evidence. | Use only fresh detector-gated flash/monitor output with exactly one origin-only target when needed, then redact before commit. |
| Treating rejected-share-only as failure | Accepted shares feel more satisfying. | A real rejected share still proves socket, ASIC result, submit, response, counters, and telemetry flow. | Accept either one real accepted or rejected share; record reason if rejected. |
| Enabling Stratum v2 or non-205 boards now | They are legitimate future parity surfaces. | They distract from the Ultra 205 BM1366 Stratum v1 production path and need separate evidence. | Keep v1.1 scoped to Ultra 205, BM1366, Stratum v1. |

## Acceptance Boundaries

| Boundary | Accepted For v1.1 | Not Accepted For v1.1 |
|----------|-------------------|-----------------------|
| Pool behavior | Real Stratum v1 socket I/O against local owner-supplied pool settings, with redacted lifecycle markers. | Controlled transcript, fake pool only, hardcoded notify only, or unverified socket connection. |
| ASIC work/result flow | Live BM1366 init gate, pool-derived work dispatch, valid nonce/result parse when claiming share submit. | Diagnostic chip-detect/work-result timeout as production proof. |
| Share outcome | At least one real accepted or rejected pool response to a live ASIC-derived submit. | Synthetic submit response, no-share soak, or API counter mutation without submit evidence. |
| Safety | Minimum documented prerequisites sufficient to enable bounded mining, plus fail-closed missing-prerequisite behavior. | Full active voltage/fan/thermal/fault/self-test closure unless separately evidenced. |
| Telemetry | Session-correlated logs, `/api/system/info`, statistics, scoreboard where applicable, and `/api/ws/live`. | Static fixtures, stale captures, or unrelated prior evidence. |
| Evidence | Redacted committed evidence plus local raw artifacts excluded from commit. | Raw pool credentials, workers, endpoints, BTC addresses, passwords, tokens, device URLs, Wi-Fi secrets, or NVS secret values. |

## Feature Dependencies

```text
Owner local inputs
    requires -> Wi-Fi credentials + pool credentials remain local and redacted

Real Stratum v1 socket lifecycle
    requires -> NVS/settings + Wi-Fi connectivity + socket adapter + Stratum v1 parser/serializer
        feeds -> Pool-derived notify/work builder

Trusted BM1366 production path
    requires -> Mining prerequisite safety gate + reset/UART/init evidence
        feeds -> Pool-derived work dispatch
            feeds -> Live nonce/result parsing
                feeds -> Share submit
                    feeds -> Accepted/rejected share counters + scoreboard

Runtime telemetry
    requires -> MiningRuntimeState + statistics samples + API/WebSocket routes
        evidences -> User-visible production mining outcome

Watchdog and safe-stop
    guards -> Socket loop + ASIC loop + telemetry capture + bounded soak

Parity checklist promotion
    requires -> Redacted evidence for each exact subclaim
```

### Dependency Notes

- **Real share outcome requires live ASIC result parsing:** a pool response only matters if the submitted share came from live BM1366 work/result flow.
- **Production work dispatch requires safety prerequisites:** the existing fail-closed gate is a feature, not a blocker to bypass.
- **Live stats require the same session as mining evidence:** API/WebSocket captures should be correlated to the run that produced socket, work, result, and share markers.
- **Scoreboard depends on share/result semantics:** populate it only from live entries that match the upstream-compatible scoreboard shape.
- **Evidence promotion depends on redaction:** a technically successful run is not commit-ready until the final redaction scan/review passes.

## MVP Definition

### Launch With (v1.1)

Minimum viable milestone: enough to decide whether the Rust firmware can be trusted to mine on one Ultra 205 under bounded conditions.

- [ ] Real Stratum v1 socket adapter and lifecycle state for subscribe, authorize, difficulty, notify, submit response, reconnect/block, and safe-stop.
- [ ] Ultra 205 BM1366 production init/work/result path gated by documented mining prerequisites.
- [ ] One real accepted or rejected share outcome from live ASIC-derived work, or an explicit milestone failure if safe prerequisites cannot reach that outcome.
- [ ] API/WebSocket/statistics/share-counter evidence from the same bounded session.
- [ ] Watchdog checkpoints and final safe-stop evidence.
- [ ] Redacted evidence pack and parity checklist updates that promote only exact proven claims.

### Add After Validation (v1.1.x)

Features to add once the first trusted production session is proven.

- [ ] Longer bounded soaks with repeatable accepted/rejected share counts after one-share proof is stable.
- [ ] Richer scoreboard history and best-difficulty persistence after live result/share semantics are proven.
- [ ] Better operator UX around blocked mining prerequisites after the first exact gate is accepted.
- [ ] More detailed pool reconnect/fallback policy after basic real socket mining works.

### Future Consideration (v1.2+)

Features to defer until this milestone has real share evidence.

- [ ] Full active voltage, fan, thermal, fault-stimulus, and self-test hardware closure.
- [ ] OTA/recovery completion, rollback, interrupted-update, large erase, and OTAWWW parity.
- [ ] Stratum v2.
- [ ] Runtime display/input parity and BAP accessory behavior.
- [ ] Non-205 boards and non-BM1366 ASIC families.
- [ ] Unbounded production soak or stress mining.

## Feature Prioritization Matrix

| Feature | User Value | Implementation Cost | Priority |
|---------|------------|---------------------|----------|
| Real Stratum v1 socket lifecycle | HIGH | HIGH | P1 |
| BM1366 production init/work/result path | HIGH | HIGH | P1 |
| Real accepted/rejected share evidence | HIGH | HIGH | P1 |
| Mining prerequisite safety gate | HIGH | HIGH | P1 |
| Redacted pool/evidence handling | HIGH | MEDIUM | P1 |
| Live mining API/WebSocket/statistics counters | HIGH | MEDIUM | P1 |
| Watchdog checkpoints and safe-stop | HIGH | MEDIUM | P1 |
| Scoreboard population | MEDIUM | MEDIUM | P2 |
| Longer bounded soak | MEDIUM | MEDIUM | P2 |
| Full active safety closure | HIGH | HIGH | P3 |
| Stratum v2 | MEDIUM | HIGH | P3 |
| Non-205 board mining evidence | MEDIUM | HIGH | P3 |

**Priority key:**
- P1: Must have for v1.1 trusted production mining
- P2: Should have once first share proof is stable
- P3: Future milestone scope

## Evidence Plan By Feature

| Feature | Testable User-Visible Outcome | Minimum Evidence |
|---------|-------------------------------|------------------|
| Real socket lifecycle | Owner pool settings lead to visible pool lifecycle and job receipt. | Redacted socket lifecycle log plus API/WebSocket lifecycle status from the same session. |
| BM1366 production path | Firmware initializes BM1366 enough to dispatch pool-derived work safely. | Hardware evidence with init gate, typed dispatch marker, and no raw frame leakage. |
| Result/share flow | Accepted or rejected share count changes after live ASIC-derived submit. | Redacted submit/response markers, runtime counters, rejected reason if applicable, and matching API capture. |
| Hashrate/statistics | Live hashrate/statistics reflect real runtime inputs rather than fixed zeros unless no result occurred. | Statistics and `/api/system/info` snapshots with session timestamp/provenance. |
| Watchdog/safe-stop | Mining session remains responsive and exits to disabled work submission. | Bounded run logs with watchdog checkpoints and final safe-stop marker. |
| Redaction | Evidence is publishable without secrets. | Final redaction review artifact and no committed raw credential/endpoint/target values. |

## Sources

- `.planning/PROJECT.md` for v1.1 goal, active requirements, out-of-scope boundaries, and architecture constraints.
- `.planning/MILESTONES.md` and `.planning/milestones/v1.0-MILESTONE-AUDIT.md` for shipped v1.0 capabilities and exact non-claims.
- `docs/parity/checklist.md` for parity rows, evidence types, safety-critical promotion rules, and current STR/ASIC/API/STAT/SAFE status.
- `docs/parity/evidence/phase-21-live-mining-and-soak-evidence/summary.md` for controlled no-share evidence and explicit below-verified claims.
- `docs/release/ultra-205.md` for operator credential, flash, evidence, and redaction rules.
- `firmware/bitaxe/src/controlled_mining_runtime.rs` for current controlled runtime shell and retained redacted markers.
- `crates/bitaxe-stratum/src/v1/controlled_runtime.rs` and `crates/bitaxe-stratum/src/v1/state.rs` for current runtime, counters, share outcome, and hashrate state contracts.
- `crates/bitaxe-api/src/mining.rs`, `crates/bitaxe-api/src/statistics.rs`, and `crates/bitaxe-api/src/scoreboard.rs` for API-visible mining, statistics, and scoreboard models.

*Feature research for: Ultra 205 trusted Stratum v1 production mining*
*Researched: 2026-07-04*

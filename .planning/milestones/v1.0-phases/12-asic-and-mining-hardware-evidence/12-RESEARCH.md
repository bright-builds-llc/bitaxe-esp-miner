# Phase 12: ASIC And Mining Hardware Evidence - Research

**Researched:** 2026-06-30  
**Domain:** Ultra 205 BM1366 hardware evidence, controlled mining smoke/soak, parity checklist promotion, and secret-safe evidence capture  
**Confidence:** HIGH for repo-local code/evidence patterns and phase gates; MEDIUM for live mining outcomes because connected hardware and controlled pool conditions determine what can be verified.

<user_constraints>
## User Constraints (from CONTEXT.md)

Source: `.planning/phases/12-asic-and-mining-hardware-evidence/12-CONTEXT.md`. [VERIFIED: .planning/phases/12-asic-and-mining-hardware-evidence/12-CONTEXT.md]

### Locked Decisions

- Start every live hardware attempt with `just detect-ultra205`; continue only when exactly one likely ESP USB serial port is found and `espflash board-info --chip esp32s3 --port <port> --non-interactive` succeeds.
- Treat Phase 11 safety evidence as a prerequisite, not a substitute for ASIC/mining evidence.
- Record a recovery path and exact allowed command set before live mining, soak, reset sequencing beyond existing wrappers, or other sustained hardware actuation.
- Use repo-owned commands and wrappers first: `just package`, `just flash-monitor board=205 port=<port> evidence-dir=<path>`, `just monitor port=<port>`, and any Phase 12 probe tooling introduced by the plans.
- Preserve the Phase 3 semantic ASIC boundary. Evidence should exercise typed `crates/bitaxe-asic` commands and observations, not raw BM1366 packets in user-facing orchestration.
- Stage evidence from safest to most active: detector gate, package identity, safe boot, chip-detect, staged initialization, diagnostic work-send/result-receive, and only then controlled mining smoke or soak.
- Prefer controlled or fake-pool conditions for first mining smoke. Real/public pool smoke is allowed only through a redacted evidence procedure.
- Keep ASIC-07, STR-06, and STR-07 below `verified` until evidence matches the exact claim.
- Every Phase 12 evidence artifact needs an explicit conclusion such as `passed for chip-detect smoke`, `passed for bounded mining smoke`, `controlled no-share condition`, `blocked by detector gate`, or `pending recovery prerequisite`.

### Deferred Ideas (OUT OF SCOPE)

- Final Ultra 205 package-to-hardware release evidence, live HTTP/static/recovery/OTA/rollback/erase/interrupted-update proof, and `DEVICE_URL` release evidence belong to Phase 13.
- Non-205 boards, BM1370/BM1368/BM1397, TPS546 hardware behavior, all-board factory image matrices, Stratum v2, BAP, and Angular UI replacement remain deferred.
- Long-term mining performance tuning, production pool optimization, and unbounded stress testing are outside Phase 12 unless a later roadmap phase defines recovery, safety, and evidence requirements.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
| --- | --- | --- |
| ASIC-07 | BM1366 initialization, work-send, and result-receive behavior have hardware-smoke evidence before release parity is claimed. | Use the existing BM1366 init/work/result pure crate plus firmware ASIC adapter, then capture board-205 hardware evidence through detector-gated wrapper/probe commands. [VERIFIED: crates/bitaxe-asic/src/bm1366/init_plan.rs] [VERIFIED: crates/bitaxe-asic/src/bm1366/work.rs] [VERIFIED: crates/bitaxe-asic/src/bm1366/result.rs] [VERIFIED: firmware/bitaxe/src/asic_adapter.rs] |
| STR-06 | The first Ultra 205 mining loop connects config, Stratum v1, BM1366 work dispatch, result parsing, and global state without bypassing safety gates. | Reuse `bitaxe-stratum` fake pool, mining-loop, queue, and runtime-state code with Phase 3 ASIC and Phase 11 safety gates. [VERIFIED: crates/bitaxe-stratum/src/v1/fake_pool.rs] [VERIFIED: crates/bitaxe-stratum/src/v1/mining_loop.rs] [VERIFIED: crates/bitaxe-stratum/src/v1/state.rs] [VERIFIED: crates/bitaxe-safety/src/evidence.rs] |
| STR-07 | Mining parity has hardware-smoke and soak criteria that record command, board, port, firmware commit, reference commit, logs, observed result, and conclusion. | Extend the Phase 9/11 wrapper-owned evidence pattern with mining-specific metadata, redaction review, bounded duration/stop conditions, and share/no-share outcomes. [VERIFIED: tools/flash/src/main.rs] [VERIFIED: docs/parity/evidence/phase-09-flash-monitor-evidence-wrapper-hardening.md] [VERIFIED: docs/parity/evidence/phase-11-safety-controller-hardware-regression-evidence.md] |
| EVD-05 | Verification layers include unit tests, golden fixtures, API comparison, hardware smoke tests, and hardware regression or soak evidence where appropriate. | Keep pure BM1366/Stratum tests green, add evidence ledger/probe validation, run detector-gated hardware smoke when available, and use `just parity` to reject overclaiming. [VERIFIED: Justfile] [VERIFIED: tools/parity/src/main.rs] [VERIFIED: docs/parity/checklist.md] |

</phase_requirements>

## Summary

Phase 12 should be planned as a hardware-evidence closure phase, not as a broad mining performance phase. The pure BM1366, Stratum, safety, API-model, flash-wrapper, and parity-validation surfaces already exist; the missing value is a strict evidence ladder that proves the exact live ASIC/mining claims or records why they remain pending. [VERIFIED: .planning/ROADMAP.md] [VERIFIED: .planning/phases/12-asic-and-mining-hardware-evidence/12-CONTEXT.md]

The planner should split the phase into a runbook/evidence contract, any minimal repo-owned probe or firmware observation hooks needed to collect bounded ASIC/mining facts, detector-gated hardware capture, and checklist/parity promotion. Live work must fail closed when detector, package identity, safety preflight, chip-detect, pool, redaction, or recovery prerequisites are missing. [VERIFIED: AGENTS.md] [VERIFIED: .planning/phases/11-safety-controller-hardware-regression-evidence/11-CONTEXT.md]

**Primary recommendation:** build a tiered Phase 12 evidence ladder: safe detector/package/boot baseline, BM1366 chip-detect/staged-init smoke, typed diagnostic work/result smoke, controlled mining smoke, optional bounded soak, then exact checklist promotion with `just parity`. [VERIFIED: .planning/phases/12-asic-and-mining-hardware-evidence/12-CONTEXT.md]

## Project Constraints

- Follow repo-local hardware guidance from `AGENTS.md`: autonomous hardware use is allowed only for connected Ultra 205 board `205` after `just detect-ultra205` succeeds exactly as defined. [VERIFIED: AGENTS.md]
- Stop and record evidence pending when there are zero likely ports, multiple likely ports, board-info failure, non-205 target, missing recovery/evidence instructions, or missing controlled pool/redaction prerequisites. [VERIFIED: AGENTS.md]
- Do not run ad hoc erase, rollback, interrupted-update, voltage/fan/mining stress, raw writes, or unbounded soak outside documented phase-gated procedures. [VERIFIED: AGENTS.md]
- Keep `reference/esp-miner` read-only and use it only as behavioral evidence with breadcrumbs and GPL guardrails. [VERIFIED: docs/adr/0005-read-only-reference-implementation.md] [VERIFIED: docs/adr/0013-mit-first-with-gpl-guardrails.md] [VERIFIED: PROVENANCE.md]
- Preserve functional core plus imperative shell: pure BM1366, Stratum, safety, and parity decisions belong in testable crates; ESP-IDF UART, GPIO, timing, networking, serial capture, and hardware effects stay in firmware or tool adapters. [VERIFIED: standards/core/architecture.md] [VERIFIED: standards/languages/rust.md]
- Before final Rust commits, run the repo-required Rust verification sequence or the repo-native equivalent required by the phase plans. [VERIFIED: AGENTS.md] [VERIFIED: standards/core/verification.md]
- Evidence must not contain pool credentials, worker secrets, Wi-Fi credentials, private endpoints, NVS secrets, or unredacted environment values. [VERIFIED: AGENTS.md]

## Existing Integration Points

| Surface | Existing Asset | Phase 12 Use |
| --- | --- | --- |
| Detector gate | `scripts/detect-ultra205.sh`, `just detect-ultra205` | Required first command before live hardware evidence. [VERIFIED: scripts/detect-ultra205.sh] [VERIFIED: Justfile] |
| Wrapper evidence | `tools/flash/src/main.rs`, `just flash-monitor` | Captures package/port/commit/log evidence and should remain the default serial proof path. [VERIFIED: tools/flash/src/main.rs] |
| BM1366 pure core | `crates/bitaxe-asic/src/bm1366/*` | Provides typed commands, init plan, work payloads, result parsing, transcript tests, and adapter gates. [VERIFIED: crates/bitaxe-asic/src/bm1366.rs] |
| Firmware ASIC adapter | `firmware/bitaxe/src/asic_adapter.rs`, `status.rs`, `uart.rs` | Owns UART/reset/status effects and visible fail-closed logs. [VERIFIED: firmware/bitaxe/src/asic_adapter.rs] |
| Stratum/mining core | `crates/bitaxe-stratum/src/v1/*` | Provides fake-pool, mining-loop, queue, mining job, share/counter, and pool lifecycle models. [VERIFIED: crates/bitaxe-stratum/src/v1/mining_loop.rs] |
| Safety evidence | `crates/bitaxe-safety/src/evidence.rs`, `firmware/bitaxe/src/safety_adapter.rs` | Provides preflight/evidence concepts that mining must not bypass. [VERIFIED: crates/bitaxe-safety/src/evidence.rs] [VERIFIED: firmware/bitaxe/src/safety_adapter.rs] |
| API/mining model | `crates/bitaxe-api/src/mining.rs`, `asic.rs`, `firmware/bitaxe/src/runtime_snapshot.rs` | Gives API/telemetry status surfaces to cite in first-loop evidence. [VERIFIED: crates/bitaxe-api/src/mining.rs] [VERIFIED: firmware/bitaxe/src/runtime_snapshot.rs] |
| Parity gate | `docs/parity/checklist.md`, `tools/parity/src/main.rs` | Exact-claim promotion and `just parity` overclaim rejection. [VERIFIED: docs/parity/checklist.md] [VERIFIED: tools/parity/src/main.rs] |

## Architecture Patterns

### Pattern 1: Tiered Evidence Ladder

**What:** Split Phase 12 evidence into ordered tiers where each tier unlocks only the next safe behavior:

1. Detector/package/safe boot baseline.
2. BM1366 chip-detect and staged initialization.
3. Typed diagnostic work-send/result-receive.
4. Controlled mining smoke.
5. Optional bounded soak.
6. Checklist/parity promotion.

**Why:** It prevents one live log from verifying unrelated claims and gives clear pending evidence when a gate blocks. [VERIFIED: .planning/phases/12-asic-and-mining-hardware-evidence/12-CONTEXT.md]

### Pattern 2: Evidence Pack Per Claim

**What:** Use one human ledger plus optional generated artifacts under `docs/parity/evidence/phase-12-asic-and-mining-hardware-evidence/`. Each claim row should name requirement IDs, command/probe, required metadata, pass criteria, fail criteria, artifact path, redaction result, evidence token, and conclusion.

**Why:** ASIC-07, STR-06, and STR-07 combine pure logic, firmware shell behavior, live UART behavior, pool lifecycle, telemetry, and soak requirements. A claim matrix keeps promotion exact. [VERIFIED: docs/parity/checklist.md] [VERIFIED: tools/parity/src/main.rs]

### Pattern 3: Diagnostic Work Before Real-Pool Work

**What:** Prefer typed, bounded BM1366 diagnostic work/result evidence before a real pool. Only move to controlled mining smoke after fake-pool tests, chip-detect/init smoke, safety preflight, and redaction/recovery gates pass.

**Why:** Diagnostic work can prove ASIC work/result plumbing without exposing pool secrets or starting unbounded mining. Real-pool smoke remains useful but should be the later, redacted evidence tier. [VERIFIED: crates/bitaxe-asic/src/bm1366/work.rs] [VERIFIED: crates/bitaxe-asic/src/bm1366/result.rs]

### Pattern 4: Secret-Safe Controlled Mining

**What:** If live pool evidence is needed, record only sanitized pool category/host where safe, lifecycle transitions, accepted/rejected shares or a controlled no-share rationale, telemetry/status fields, hashrate inputs, watchdog observations, duration, stop conditions, and safe-stop result.

**Why:** STR-07 requires live mining evidence metadata, but AGENTS.md forbids committing pool credentials, worker secrets, private endpoints, Wi-Fi credentials, or NVS secrets. [VERIFIED: AGENTS.md] [VERIFIED: .planning/REQUIREMENTS.md]

## Common Pitfalls

### Pitfall 1: Treating Phase 11 Safe Boot As Mining Evidence

Phase 11 safe boot and safety evidence can prove prerequisites and residual safety boundaries. It does not verify BM1366 work/result handling or mining-loop behavior. [VERIFIED: .planning/phases/11-safety-controller-hardware-regression-evidence/11-CONTEXT.md]

### Pitfall 2: Letting One Successful Mining Log Verify Everything

A single happy-path log should not verify chip-detect, staged init, work-send, result receive, pool lifecycle, API status, watchdog responsiveness, and soak criteria. Split claims or leave broad rows below `verified`. [VERIFIED: .planning/phases/12-asic-and-mining-hardware-evidence/12-CONTEXT.md]

### Pitfall 3: Capturing Secrets In Evidence

Pool credentials can appear in config, command lines, NVS dumps, serial logs, or URLs. Every generated artifact needs redaction review before commit. [VERIFIED: AGENTS.md]

### Pitfall 4: Bypassing Repo-Owned Tools

Direct serial writes or raw `espflash` commands can skip package identity, detector output, trusted-output checks, and evidence JSON conventions. Use repo wrappers unless a plan documents a narrow reason otherwise. [VERIFIED: tools/flash/src/main.rs] [VERIFIED: AGENTS.md]

### Pitfall 5: Running Soak Without Stop Conditions

Soak without duration, temperature/power/watchdog thresholds, reconnect limits, serial-silence handling, safe-stop behavior, and recovery notes is stress, not verification. Phase 12 should avoid unbounded stress. [VERIFIED: .planning/phases/12-asic-and-mining-hardware-evidence/12-CONTEXT.md]

### Pitfall 6: Updating Checklist Before Evidence

Checklist promotion should be last. Plans should first add or capture evidence, then update rows whose exact claims are supported, then run `just parity`. [VERIFIED: docs/parity/checklist.md] [VERIFIED: tools/parity/src/main.rs]

## Planning Guidance

Recommended plan split:

1. **Evidence contract and runbook:** Create the Phase 12 ledger, claim matrix, redaction contract, recovery/stop conditions, and checklist promotion rules. No live mining yet.
2. **Probe/firmware observation surface:** Add minimal repo-owned command or firmware status hooks only if existing `flash-monitor` evidence cannot observe chip-detect/init/work/result/mining facts. Keep pure logic in crates and effects in adapters.
3. **Detector-gated ASIC hardware smoke:** Run `just detect-ultra205`, package/flash/monitor, chip-detect/staged-init, and diagnostic work/result evidence when the board and prerequisites are available; otherwise record precise pending evidence.
4. **Controlled mining smoke/soak:** Run fake-pool or controlled pool smoke first, then optional bounded soak with redaction, watchdog/status observations, and safe-stop behavior.
5. **Checklist, parity, and verification closure:** Update only exact supported rows, add parity tests only for new machine-checkable semantics, run `just parity` and relevant Rust/Bazel tests, then verify phase goal.

The planner may merge or split these, but every plan must preserve the evidence ladder and map to ASIC-07, STR-06, STR-07, or EVD-05.

## Validation Architecture

### Test Framework

| Property | Value |
| --- | --- |
| Framework | Rust unit tests through Cargo and Bazel, shell/script tests where existing repo scripts own behavior, and detector-gated hardware evidence when available. |
| Config file | Workspace `Cargo.toml`, `MODULE.bazel`, `Justfile`, `tools/flash/BUILD.bazel`, `tools/parity/BUILD.bazel`, relevant crate `BUILD.bazel` files. |
| Quick run command | `cargo test -p bitaxe-asic -p bitaxe-stratum -p bitaxe-safety -p bitaxe-api --all-features` |
| Full suite command | `cargo fmt --all && cargo clippy --all-targets --all-features -- -D warnings && cargo build --all-targets --all-features && cargo test --all-features && just parity` |
| Hardware gate command | `just detect-ultra205` before any live board command. |
| Hardware evidence command | `just flash-monitor board=205 port=<path> evidence-dir=docs/parity/evidence/phase-12-asic-and-mining-hardware-evidence` or a Phase 12 probe command documented by the active plan. |
| Estimated runtime | Targeted host checks: ~2-5 minutes. Full Rust pre-commit checks: longer. Hardware smoke/soak: depends on detector, package build, controlled pool, duration, and recovery prerequisites. |

### Phase Requirements To Verification Map

| Requirement | Automated Verification | Hardware / Manual Verification | Evidence Artifact |
| --- | --- | --- | --- |
| ASIC-07 | BM1366 init/work/result tests stay green; any new probe parser or evidence schema has unit tests. | Detector-gated chip-detect/staged-init and diagnostic work/result run, or explicit pending/blocker record. | `docs/parity/evidence/phase-12-asic-and-mining-hardware-evidence.md` plus optional generated JSON/logs. |
| STR-06 | Stratum fake-pool/mining-loop/state tests stay green; firmware/build checks prove integration compiles. | Controlled first-loop run or safe-blocked proof showing config, Stratum, BM1366, safety gates, API/telemetry status, and watchdog responsiveness. | Phase 12 evidence ledger and generated capture artifacts. |
| STR-07 | Evidence-schema/redaction/parity checks validate required metadata and secret-free conclusions. | Bounded smoke/soak run with accepted/rejected shares or documented controlled no-share condition. | Mining smoke/soak section, redaction review, serial logs, command evidence. |
| EVD-05 | `just parity`, targeted Rust tests, and full pre-commit checks before final commit. | Hardware evidence only when detector and prerequisites pass; otherwise pending evidence is explicit. | Checklist rows and Phase 12 verification report. |

### Sampling Strategy

- After evidence/runbook edits: `git diff --check` on changed Markdown plus `just parity` if checklist/evidence rows changed.
- After Rust pure-code edits: targeted `cargo test -p ... --all-features` for affected crates.
- After firmware/tool edits: targeted Cargo tests plus `cargo build --all-targets --all-features` when feasible before hardware.
- Before live hardware: `just detect-ultra205`; record detector output in evidence.
- After live hardware: run redaction review before staging generated logs or JSON.
- Before phase completion: full Rust pre-commit sequence required by repo rules, `just parity`, phase verifier, and lifecycle validation.

### Manual-Only Verifications

| Behavior | Requirement | Why Manual | Instructions |
| --- | --- | --- | --- |
| Connected Ultra 205 detection and board-info | ASIC-07, STR-07 | Requires physical USB board. | Run `just detect-ultra205`; continue only with exactly one successful `port=<path>`. |
| BM1366 live chip-detect/staged init | ASIC-07 | Requires live ASIC/UART/reset path. | Run the plan-approved wrapper/probe after detector and recovery gates; record logs and conclusions. |
| Controlled mining smoke/soak | STR-06, STR-07 | Requires hardware, pool/fake-pool condition, safety monitoring, and redaction. | Use bounded duration and stop conditions; record accepted/rejected shares or controlled no-share rationale. |
| Redaction review | STR-07, EVD-05 | Human review is required for secrets in logs/configs. | Inspect all generated evidence before commit; document result in `redaction-review.md` or ledger section. |

### Validation Risks

- Hardware may be unavailable or ambiguous. In that case, the correct phase output is pending evidence with exact blocker language, not checklist promotion.
- Controlled mining may produce no accepted share in a short run. This can still support a controlled no-share condition only if the plan documents expected behavior, observed pool lifecycle, work dispatch, telemetry, watchdog responsiveness, and safe-stop.
- Full pre-commit checks may be expensive but are mandatory before final commit in this Rust repo when code changed.

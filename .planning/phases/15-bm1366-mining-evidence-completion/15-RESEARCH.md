# Phase 15: BM1366 Mining Evidence Completion - Research

**Researched:** 2026-07-01  
**Domain:** Ultra 205 BM1366 firmware diagnostics, mining evidence, parity gates, and hardware-run safety  
**Confidence:** HIGH for local architecture and evidence constraints; MEDIUM for live-pool execution because no pool or `DEVICE_URL` environment was present during research. [VERIFIED: .planning/phases/15-bm1366-mining-evidence-completion/15-CONTEXT.md; VERIFIED: environment audit]

<user_constraints>
## User Constraints (from CONTEXT.md)

All items in this section are copied from `.planning/phases/15-bm1366-mining-evidence-completion/15-CONTEXT.md`; this section is the locked planning boundary for Phase 15. [VERIFIED: 15-CONTEXT.md]

### Locked Decisions

## Implementation Decisions

### Trusted BM1366 Initialization Evidence

- **D-01:** Fix the Phase 12 chip-detect trust gap with a packaged diagnostic image or equivalent package-backed flow, not by weakening the wrapper trust profile. Phase 12 already observed useful chip-detect/no-mining markers, but the ELF-only capture was untrusted because `spiffs_mount=available` was absent.
- **D-02:** Preserve existing trusted wrapper markers for package identity, SPIFFS availability, Ultra 205 boot identity, firmware commit, reference commit, safe state, and route shell startup. A diagnostic-only trust profile may be used only if claims remain diagnostic-only and no checklist row is promoted to `verified`.
- **D-03:** Keep chip-detect and staged initialization evidence scoped to the exact observed behavior: chip-detect success, staged no-mining init, partial UART read, timeout, chip-count mismatch, or fail-closed status. Do not treat chip-detect evidence as proof of production mining, frequency transition, voltage behavior, work-send, result-receive, or accepted shares.
- **D-04:** If the packaged diagnostic path remains unstable, write a useful pending evidence artifact and restore the board to trusted packaged safe boot instead of overclaiming.

### Diagnostic Work-Send And Result-Receive Evidence

- **D-05:** Use a repo-owned typed firmware diagnostic over the USB serial console as the primary work-send/result-receive evidence path. The diagnostic must exercise existing typed BM1366 command, work, result, valid-job, and adapter behavior instead of raw serial writes or hand-built BM1366 frames in orchestration.
- **D-06:** Keep the diagnostic bounded and non-mining by default. It may prove `work packet dispatched`, `result frame parsed`, `bounded timeout with fail-closed state`, `invalid job rejected`, or `no result observed before timeout`, but it must not claim production mining parity by itself.
- **D-07:** Host-only replay, golden, and unit tests remain supporting regression guards only. They can protect the diagnostic runner and evidence parser, but they cannot verify live UART/ASIC behavior.
- **D-08:** A diagnostic HTTP/admin route is a fallback only if serial evidence becomes too noisy. If added, it must be compile-gated or otherwise impossible to expose accidentally in production firmware, must avoid secrets, and must have explicit redaction checks.

### Controlled Mining Smoke And Bounded Soak

- **D-09:** Use a layered evidence ladder: detector/package/safe boot, trusted chip-detect or staged init, typed diagnostic work/result, controlled mining smoke, bounded mining soak, then exact checklist promotion. Later tiers must not run when earlier tiers fail.
- **D-10:** Prefer a local deterministic Stratum harness or controlled no-share path when live pool credentials, explicit `DEVICE_URL`, or redaction prerequisites are missing. This may prove pool lifecycle, job construction, work pipeline, telemetry/status shape, watchdog responsiveness, and controlled no-share behavior, but it must be labelled as controlled evidence.
- **D-11:** Live pool micro-smoke is allowed only when disposable credentials or non-secret pool configuration, one detected Ultra 205, safety gates, recovery path, explicit `DEVICE_URL` for live API/WebSocket telemetry, safe-stop behavior, and redaction review are all available. Pool credentials, worker secrets, private endpoints, Wi-Fi credentials, API tokens, and NVS secrets must never be committed.
- **D-12:** Bounded soak may run only after a live smoke passes or the plan explicitly justifies a controlled no-share soak. It must set duration, abort conditions, thermal/power/watchdog observations, periodic status/API/WebSocket snapshots when available, reconnect/fallback scope when exercised, and final safe-stop or restore evidence.

### Checklist Promotion, Parity Guards, And Redaction

- **D-13:** Use evidence-tiered exact-claim promotion. Promote checklist rows only when the evidence class, artifact, conclusion, redaction review, and `just parity` support the exact row claim.
- **D-14:** If Phase 15 captures partial proof, use an evidence-only ledger plus conservative checklist notes. Keep broad ASIC, mining, API, WebSocket, and statistics rows below `verified` when the live evidence does not prove the whole behavior.
- **D-15:** Add or extend `tools/parity` guards only when Phase 15 attempts to promote an ASIC, Stratum, mining, API, or statistics row to `verified` and the current guard vocabulary cannot enforce the evidence/redaction prerequisite.
- **D-16:** Every generated log, JSON, Markdown evidence file, API response, WebSocket capture, and pasted output must receive redaction review before commit. Redaction review must explicitly cover pool credentials, worker names/secrets, Wi-Fi credentials, private endpoints, `DEVICE_URL`, API tokens, NVS secret values, and local terminal secrets.

### Final Verification Gate

- **D-17:** Final phase verification must run relevant targeted checks for changed code, `just test`, `just parity`, `just verify-reference`, reference diff cleanliness, redaction review, lifecycle validation, and any detector/hardware commands that the phase actually used.
- **D-18:** No wrapper-level commit or push should happen unless the phase `15-VERIFICATION.md` status is `passed` and lifecycle validation for `15-2026-07-01T02-07-59` succeeds with plans and verification present.

### the agent's Discretion

The agent may choose the exact plan count, diagnostic package target shape, evidence directory layout, JSON schema, probe command names, serial marker names, and whether additional parity checks live in `tools/parity`, a repo-owned script, or a small host tool. Those choices must keep `reference/esp-miner` read-only, use repo-owned ESP/esp-rs tooling before raw tool paths, keep diagnostics bounded, avoid broad verified claims, and avoid standalone body `---` separators in GSD artifacts.

### Deferred Ideas (OUT OF SCOPE)

## Deferred Ideas

- Same-commit package, flash, serial boot, live HTTP/static/recovery/OTA, rollback, erase, failed-update, and interrupted-update evidence belongs to Phase 16.
- Non-205 boards, BM1370/BM1368/BM1397, all-board factory images, Stratum v2, BAP, Angular AxeOS replacement, and production mining performance tuning remain deferred.
- Active voltage, fan duty, overheat/fault stimulus, self-test hardware submodes, runtime display/input parity, and broad live safety telemetry remain outside Phase 15 unless needed only as prerequisites and already covered by a bounded recovery plan.
- Long, unbounded mining stress and real-pool optimization are out of scope.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
| --- | --- | --- |
| ASIC-07 | BM1366 initialization, work-send, and result-receive behavior need hardware-smoke evidence before release parity is claimed. [VERIFIED: .planning/REQUIREMENTS.md] | Plan package-backed chip-detect/staged init first, then a bounded typed serial work/result diagnostic using `bitaxe-asic` work/result modules and firmware UART adapter. [VERIFIED: crates/bitaxe-asic/src/bm1366/work.rs; VERIFIED: crates/bitaxe-asic/src/bm1366/result.rs; VERIFIED: firmware/bitaxe/src/asic_adapter.rs] |
| STR-06 | The first Ultra 205 mining loop must connect config, Stratum v1, BM1366 dispatch, result parsing, and global state without bypassing safety gates. [VERIFIED: .planning/REQUIREMENTS.md] | Reuse `MiningLoopGate`, `MiningRuntimeState`, fake-pool/work-queue logic, and API mappers; add live or controlled evidence only after ASIC and safety gates pass. [VERIFIED: crates/bitaxe-stratum/src/v1/mining_loop.rs; VERIFIED: crates/bitaxe-stratum/src/v1/state.rs; VERIFIED: crates/bitaxe-api/src/mining.rs] |
| STR-07 | Mining parity needs hardware-smoke and soak criteria with command, board, port, commits, logs, observed result, and conclusion. [VERIFIED: .planning/REQUIREMENTS.md] | Extend the Phase 12 evidence ladder into Phase 15 component packs, including controlled no-share/live-share outcomes, telemetry snapshots, watchdog responsiveness, safe-stop, and redaction. [VERIFIED: docs/parity/evidence/phase-12-asic-and-mining-hardware-evidence.md; VERIFIED: 15-CONTEXT.md] |
| SAFE-09 | Mining/control/API/telemetry tasks must avoid watchdog starvation and preserve responsiveness under load. [VERIFIED: .planning/REQUIREMENTS.md] | Treat bounded mining smoke/soak as a watchdog and responsiveness exercise; ESP-IDF TWDT detects prolonged non-yielding tasks, so evidence should include API/WebSocket/serial responsiveness during the run. [CITED: https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/wdts.html; VERIFIED: firmware/bitaxe/src/http_api.rs] |
| EVD-05 | Verification layers must include unit tests, golden fixtures, API comparison, hardware smoke tests, and hardware regression or soak evidence where appropriate. [VERIFIED: .planning/REQUIREMENTS.md] | Keep unit/golden/API checks as regression guards and require matching hardware-smoke, hardware-regression, or soak evidence before checklist promotion. [VERIFIED: tools/parity/src/main.rs; VERIFIED: docs/parity/checklist.md] |
</phase_requirements>

## Summary

Phase 15 should be planned as evidence completion, not as a broad mining feature rewrite. [VERIFIED: 15-CONTEXT.md] The existing code already has typed BM1366 work/result logic, guarded mining-loop state, API mining projection, a trusted `tools/flash` wrapper, and parity guards that reject live ASIC/mining overclaims. [VERIFIED: crates/bitaxe-asic/src/bm1366/work.rs; VERIFIED: crates/bitaxe-asic/src/bm1366/result.rs; VERIFIED: crates/bitaxe-stratum/src/v1/mining_loop.rs; VERIFIED: tools/flash/src/main.rs; VERIFIED: tools/parity/src/main.rs]

The first plan priority is to close the exact Phase 12 root cause: chip-detect ran from an ELF-only diagnostic image and was wrapper-untrusted because packaged SPIFFS markers were absent. [VERIFIED: docs/parity/evidence/phase-12-asic-and-mining-hardware-evidence.md] Use a package-backed diagnostic build or equivalent wrapper-supported package flow that preserves `spiffs_mount=available`, route shell startup, firmware/reference commit markers, safe-state markers, and source/package identity. [VERIFIED: tools/flash/src/main.rs; VERIFIED: scripts/package-firmware.sh]

The second priority is a bounded typed serial diagnostic for work-send/result-receive, because firmware currently gates only `FailClosed` and `ChipDetectOnly` modes while pure work/result logic already exists. [VERIFIED: firmware/bitaxe/src/asic_adapter.rs; VERIFIED: crates/bitaxe-asic/src/bm1366/adapter_gate.rs; VERIFIED: crates/bitaxe-asic/src/bm1366/work.rs; VERIFIED: crates/bitaxe-asic/src/bm1366/result.rs] Controlled mining smoke should default to a deterministic no-share or fake-pool-backed path unless live pool inputs and explicit `DEVICE_URL` are available, and this environment currently has `DEVICE_URL` and pool variables unset. [VERIFIED: environment audit]

**Primary recommendation:** Build Phase 15 as five plans: package-backed diagnostic trust fix, typed work/result diagnostic, mining evidence scaffold and allow gate, controlled smoke/soak runner with telemetry/redaction, and conservative checklist/final verification closure. [VERIFIED: 15-CONTEXT.md; VERIFIED: docs/parity/evidence/phase-12-asic-and-mining-hardware-evidence.md; VERIFIED: tools/parity/src/main.rs]

## Project Constraints (from AGENTS.md)

- Do not edit the Bright Builds managed block in `AGENTS.md` or managed standards files. [VERIFIED: AGENTS.md; VERIFIED: AGENTS.bright-builds.md]
- Read repo-local guidance, `AGENTS.bright-builds.md`, `standards-overrides.md`, and relevant standards before planning or implementation. [VERIFIED: AGENTS.md; VERIFIED: AGENTS.bright-builds.md]
- Keep `reference/esp-miner` pinned and read-only; use it as behavioral evidence only. [VERIFIED: AGENTS.md; VERIFIED: .planning/PROJECT.md]
- Use ESP-IDF Rust bindings, pinned `esp-idf-sys` metadata, checked-in `.cargo/config.toml`, `espup`, `ldproxy`, and `espflash` before custom ESP-IDF tool paths. [VERIFIED: AGENTS.md]
- Treat `.embuild/` as local generated ESP tooling state; do not commit or hand-edit it, but repo automation may use managed ESP-IDF tools from it. [VERIFIED: AGENTS.md]
- Before autonomous Ultra 205 hardware use, run `just detect-ultra205`; continue only when one likely ESP USB serial port is found and `espflash board-info --chip esp32s3 --port <port> --non-interactive` succeeds. [VERIFIED: AGENTS.md; VERIFIED: scripts/detect-ultra205.sh]
- Stop or record evidence pending when zero or multiple ports, board-info failure, non-205 target, or missing recovery/evidence instructions appear. [VERIFIED: AGENTS.md]
- Do not run ad hoc erase, rollback, interrupted-update, voltage/fan/mining stress, or raw write commands outside documented phase-gated procedures. [VERIFIED: AGENTS.md]
- Every hardware run must record board `205`, selected port, source commit, reference commit, package manifest/artifacts when applicable, exact commands, board-info output, captured logs, observed behavior, and conclusion. [VERIFIED: AGENTS.md]
- Do not commit secrets, pool credentials, Wi-Fi credentials, private endpoints, or NVS secret values in evidence. [VERIFIED: AGENTS.md]
- GSD and other YAML-frontmatter Markdown artifacts must not use standalone body `---` separators after frontmatter. [VERIFIED: AGENTS.md]
- Rust commits require `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, and `cargo test --all-features` before commit. [VERIFIED: AGENTS.md]
- Pure and business logic should stay in a functional core, while ESP-IDF, serial, HTTP/WebSocket, task, and hardware effects stay in thin adapters. [VERIFIED: standards/core/architecture.md; VERIFIED: standards/languages/rust.md]
- Unit tests for pure logic should cover one concern and use Arrange, Act, Assert structure when non-trivial. [VERIFIED: standards/core/testing.md; VERIFIED: standards/languages/rust.md]
- Project skills directories `.claude/skills/` and `.agents/skills/` were absent during research. [VERIFIED: project skills audit]

## Standard Stack

### Core

| Library / Tool | Version | Purpose | Why Standard |
| --- | --- | --- | --- |
| `bitaxe-asic` | `0.1.0` local crate | Typed BM1366 command, chip-detect, init, work, result, and transcript logic. [VERIFIED: crates/bitaxe-asic/Cargo.toml; VERIFIED: crates/bitaxe-asic/src/bm1366.rs] | Use this instead of raw ASIC frames so diagnostics exercise the same typed boundary as firmware. [VERIFIED: 15-CONTEXT.md; VERIFIED: crates/bitaxe-asic/src/bm1366/command.rs] |
| `bitaxe-stratum` | `0.1.0` local crate | Stratum v1 fake pool, mining job construction, work queue, guarded mining loop, and runtime state. [VERIFIED: crates/bitaxe-stratum/Cargo.toml; VERIFIED: crates/bitaxe-stratum/src/v1/mining_loop.rs] | Use this for controlled no-share/live smoke planning so mining evidence stays aligned with existing state machines. [VERIFIED: crates/bitaxe-stratum/src/v1/fake_pool.rs; VERIFIED: crates/bitaxe-stratum/src/v1/state.rs] |
| `bitaxe-safety` | `0.1.0` local crate | Safety evidence tokens, power/thermal status, watchdog and safety decisions. [VERIFIED: crates/bitaxe-safety/Cargo.toml; VERIFIED: crates/bitaxe-safety/src/evidence.rs] | Mining gates require power, thermal, and safety evidence before work submission can be considered ready. [VERIFIED: crates/bitaxe-stratum/src/v1/mining_loop.rs; VERIFIED: crates/bitaxe-asic/src/bm1366/init_plan.rs] |
| `bitaxe-api` | `0.1.0` local crate | API mining, ASIC, statistics, and WebSocket telemetry projection models. [VERIFIED: crates/bitaxe-api/Cargo.toml; VERIFIED: crates/bitaxe-api/src/mining.rs] | Use this for HTTP/WebSocket evidence comparisons rather than inventing separate JSON models. [VERIFIED: firmware/bitaxe/src/runtime_snapshot.rs; VERIFIED: firmware/bitaxe/src/http_api.rs] |
| `tools/flash` | `0.1.0` local tool; `espflash 4.0.1` installed | Package-aware flash/monitor wrapper and trusted serial evidence JSON/log capture. [VERIFIED: tools/flash/Cargo.toml; VERIFIED: environment audit] | Preserve the wrapper trust contract rather than weakening trusted marker classification. [VERIFIED: tools/flash/src/main.rs; VERIFIED: docs/parity/evidence/phase-12-asic-and-mining-hardware-evidence.md] |
| `tools/parity` | `0.1.0` local tool | Checklist validation, live ASIC/mining overclaim guards, and Phase 14 safety allow-manifest validation. [VERIFIED: tools/parity/Cargo.toml; VERIFIED: tools/parity/src/main.rs; VERIFIED: tools/parity/src/safety_allow.rs] | Use or extend it only when checklist promotion needs machine-checkable enforcement. [VERIFIED: 15-CONTEXT.md; VERIFIED: tools/parity/src/main.rs] |
| `firmware/bitaxe` | `0.1.0`, ESP-IDF `v5.5.4`, `xtensa-esp32s3-espidf` | ESP-IDF Rust firmware shell for serial, boot, SPIFFS, HTTP/WebSocket, safety, and runtime snapshot adapters. [VERIFIED: firmware/bitaxe/Cargo.toml; VERIFIED: firmware/bitaxe/src/main.rs] | Keep hardware effects in firmware adapters and pure logic in crates. [VERIFIED: AGENTS.md; VERIFIED: standards/languages/rust.md] |

### Supporting

| Library / Tool | Version | Purpose | When to Use |
| --- | --- | --- | --- |
| `serde` / `serde_json` | `serde 1.0.228`, `serde_json 1.0.150` locked | Parse and render evidence JSON, allow manifests, API bodies, and tool output. [VERIFIED: Cargo.toml; VERIFIED: Cargo.lock] | Use for diagnostic evidence schema and redaction-safe summaries. [VERIFIED: tools/flash/src/main.rs; VERIFIED: tools/parity/src/safety_allow.rs] |
| `clap` | `4.6.1` locked | CLI parsing for host tools. [VERIFIED: Cargo.toml; VERIFIED: Cargo.lock] | Use if adding a host evidence runner under `tools/`; shell wrappers can remain for narrow orchestration. [VERIFIED: tools/parity/src/main.rs; VERIFIED: tools/flash/src/main.rs] |
| `anyhow` | `1.0.103` locked | Application/tool error context. [VERIFIED: Cargo.lock] | Use in host tools and firmware shell adapters; keep library error types typed where already established. [VERIFIED: tools/flash/src/main.rs; VERIFIED: firmware/bitaxe/src/asic_adapter.rs] |
| `camino` | `1.2.3` locked | UTF-8 path handling in host tools. [VERIFIED: Cargo.lock] | Use for evidence and manifest paths in Rust host tools. [VERIFIED: tools/parity/src/safety_allow.rs; VERIFIED: tools/flash/src/main.rs] |
| `just` | `1.48.0` installed | Human command surface. [VERIFIED: environment audit; VERIFIED: Justfile] | Keep Phase 15 commands reachable through `just` or Bazel-backed scripts where practical. [VERIFIED: AGENTS.md; VERIFIED: Justfile] |
| `bazel` | `9.1.1` installed | Canonical automation graph. [VERIFIED: environment audit; VERIFIED: .planning/STACK excerpt in AGENTS.md] | Use for firmware package targets, tool tests, wrapper tests, and `just` commands. [VERIFIED: firmware/bitaxe/BUILD.bazel; VERIFIED: scripts/BUILD.bazel] |
| Node WebSocket global | Node `v24.13.0`, `typeof WebSocket === "function"` | Fallback maintained WebSocket client for API/WebSocket telemetry capture when `websocat` is absent. [VERIFIED: environment audit] | Prefer a repo-owned Node script or Rust helper if WebSocket frames must be captured; `websocat` is not installed. [VERIFIED: environment audit] |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
| --- | --- | --- |
| Package-backed diagnostic image | ELF-only image via `--image bazel-bin/...elf` | This already produced wrapper-untrusted evidence because SPIFFS markers were absent; do not repeat it for verified claims. [VERIFIED: docs/parity/evidence/phase-12-asic-and-mining-hardware-evidence.md] |
| Typed serial diagnostic in firmware | Raw host serial writes of BM1366 frames | Raw serial writes bypass existing `bitaxe-asic` command/work/result boundaries and violate D-05. [VERIFIED: 15-CONTEXT.md] |
| Controlled no-share/local deterministic harness | Live public pool micro-smoke | Live pool can be valid only with disposable/non-secret config, explicit `DEVICE_URL`, redaction review, and safe-stop; those variables were unset during research. [VERIFIED: 15-CONTEXT.md; VERIFIED: environment audit] |
| Mining-specific allow manifest or extension | Reusing Phase 14 `safety-allow` unchanged | Current `safety-allow` allowed surfaces are safety-specific and do not include BM1366/mining surfaces. [VERIFIED: tools/parity/src/safety_allow.rs] |
| Repo-owned WebSocket helper | `websocat` | `websocat` was missing, while Node 24 with WebSocket global was present. [VERIFIED: environment audit] |

**Installation:**

```bash
# No new dependency should be required for the baseline Phase 15 plan.
# Use existing repo tools:
just doctor
just package
just detect-ultra205
```

**Version verification:** Existing crate versions were verified from `Cargo.toml` and `Cargo.lock`; installed command versions were verified with `--version`; no `npm install` package is recommended. [VERIFIED: Cargo.toml; VERIFIED: Cargo.lock; VERIFIED: environment audit]

## Architecture Patterns

### Recommended Project Structure

```text
firmware/bitaxe/src/
├── asic_adapter.rs              # Extend compile-time diagnostic modes and typed UART interpreter. [VERIFIED: firmware/bitaxe/src/asic_adapter.rs]
├── asic_adapter/
│   ├── uart.rs                  # Reuse bounded read/write/timeout methods. [VERIFIED: firmware/bitaxe/src/asic_adapter/uart.rs]
│   └── status.rs                # Add exact diagnostic status markers. [VERIFIED: firmware/bitaxe/src/asic_adapter/status.rs]
scripts/
├── phase15-*.sh                 # Narrow hardware/evidence wrappers mirroring Phase 14 patterns. [VERIFIED: scripts/phase14-power-voltage.sh]
tools/parity/src/
├── main.rs                      # Extend checklist guards only for promoted claims. [VERIFIED: tools/parity/src/main.rs]
└── mining_allow.rs              # Prefer a mining-specific allow manifest if safety_allow is too narrow. [VERIFIED: tools/parity/src/safety_allow.rs]
docs/parity/evidence/
└── phase-15-bm1366-mining-evidence-completion/
    ├── README.md
    ├── chip-detect/
    ├── work-result/
    ├── mining-smoke/
    ├── bounded-soak/
    └── redaction-review.md
```

### Pattern 1: Package-Backed Diagnostic Trust

**What:** Add a package-backed diagnostic target/flow that compiles the firmware with diagnostic env, packages the result with SPIFFS/static assets, flashes via `tools/flash`, and preserves trusted boot markers. [VERIFIED: 15-CONTEXT.md; VERIFIED: firmware/bitaxe/BUILD.bazel; VERIFIED: scripts/package-firmware.sh; VERIFIED: tools/flash/src/main.rs]

**When to use:** Use for chip-detect and staged init evidence before any work/result or mining tier runs. [VERIFIED: 15-CONTEXT.md]

**Planner note:** The existing `firmware_image` target packages the default firmware, while Phase 12 used `--action_env=BITAXE_ASIC_DIAGNOSTIC=chip-detect` plus an ELF path; the plan should create an explicit diagnostic package target or wrapper so package identity and SPIFFS evidence remain intact. [VERIFIED: firmware/bitaxe/BUILD.bazel; VERIFIED: docs/parity/evidence/phase-12-asic-and-mining-hardware-evidence.md]

### Pattern 2: Bounded Typed Work/Result Serial Diagnostic

**What:** Extend `AsicAdapterMode` beyond `ChipDetectOnly` with a compile-gated work/result diagnostic that uses `diagnostic_job_frame`, `Bm1366ValidJobIds`, and `parse_bm1366_result_frame`; publish exact serial markers for dispatched work, parsed result, timeout, invalid job, and fail-closed status. [VERIFIED: crates/bitaxe-asic/src/bm1366/adapter_gate.rs; VERIFIED: crates/bitaxe-asic/src/bm1366/work.rs; VERIFIED: crates/bitaxe-asic/src/bm1366/result.rs; VERIFIED: firmware/bitaxe/src/asic_adapter.rs]

**When to use:** Use after trusted chip-detect/staged init passes and before controlled mining smoke. [VERIFIED: 15-CONTEXT.md]

**Example:**

```rust
// Source: local pattern in crates/bitaxe-asic/src/bm1366/work.rs and result.rs. [VERIFIED: codebase grep]
let work = diagnostic_job_frame(job_id, fields)?;
uart.write_frame(work.bytes())?;
let maybe_result = uart.read_exact(BM1366_RESULT_FRAME_LEN, timeout_ms);
```

### Pattern 3: Evidence Ladder With Hard Stop Conditions

**What:** Treat each tier as a prerequisite for the next: detector/package/safe boot, trusted chip-detect/staged init, typed work/result, controlled mining smoke, bounded soak, checklist promotion. [VERIFIED: 15-CONTEXT.md]

**When to use:** Use for all Phase 15 live hardware runs. [VERIFIED: 15-CONTEXT.md]

**Stop conditions:** Stop and record pending evidence on detector mismatch, board-info failure, missing package trust markers, missing recovery plan, redaction uncertainty, unsafe temperature/power/watchdog status, serial silence after timeout, leaked secret, or missing safe-stop evidence. [VERIFIED: AGENTS.md; VERIFIED: docs/parity/evidence/phase-12-asic-and-mining-hardware-evidence.md; VERIFIED: docs/parity/evidence/phase-14-safety-hardware-evidence-completion/README.md]

### Pattern 4: Mining Evidence Allow Manifest

**What:** Mirror the Phase 14 `safety-allow` contract for mining surfaces, or add allowed mining surfaces and claim tiers to a new parity subcommand. [VERIFIED: tools/parity/src/safety_allow.rs; VERIFIED: docs/parity/evidence/phase-14-safety-hardware-evidence-completion/README.md]

**When to use:** Use before any active diagnostic work/result, controlled mining smoke, or bounded soak wrapper. [VERIFIED: 15-CONTEXT.md]

**Required fields:** Board, port, detector transcript, board-info status, package manifest, source/reference commits, surface, claim tier, evidence class, allowed command, allowed inputs, abort conditions, recovery steps, post-action safe-state markers, evidence directory, redaction reviewer, and checklist rows. [VERIFIED: tools/parity/src/safety_allow.rs; VERIFIED: docs/parity/evidence/phase-14-safety-hardware-evidence-completion/README.md]

### Pattern 5: Conservative Checklist Promotion

**What:** Only promote rows when the artifact supports the exact row claim, evidence class, conclusion, redaction review, and `just parity` outcome. [VERIFIED: 15-CONTEXT.md; VERIFIED: tools/parity/src/main.rs]

**When to use:** Use at phase closure after all evidence artifacts are reviewed. [VERIFIED: 15-CONTEXT.md]

**Current guard vocabulary:** `tools/parity` already requires hardware-smoke or soak evidence for `ASIC-002`, `ASIC-003`, `ASIC-004`, `ASIC-005`, `STR-006`, and `STR-008`, and `STR-008` requires accepted share, rejected share, or controlled no-share metadata. [VERIFIED: tools/parity/src/main.rs]

### Anti-Patterns to Avoid

- **Weakening wrapper trust:** Do not remove `spiffs_mount=available`, route shell, commit, or safe-state marker requirements to make diagnostics pass. [VERIFIED: 15-CONTEXT.md; VERIFIED: tools/flash/src/main.rs]
- **Raw BM1366 frame orchestration:** Do not construct or send raw BM1366 bytes from shell or host scripts. [VERIFIED: 15-CONTEXT.md; VERIFIED: crates/bitaxe-asic/src/bm1366/command.rs]
- **Live pool by default:** Do not run live pool smoke when `DEVICE_URL`, disposable/non-secret pool config, recovery, safe-stop, and redaction prerequisites are missing. [VERIFIED: 15-CONTEXT.md; VERIFIED: environment audit]
- **Borrowing evidence across tiers:** Do not use chip-detect, host tests, fake-pool tests, safe boot, or API fixtures as proof of production mining or live result parsing. [VERIFIED: docs/parity/evidence/phase-12-asic-and-mining-hardware-evidence.md; VERIFIED: tools/parity/src/main.rs]
- **Committing private endpoints or pool identifiers:** Do not commit pool credentials, worker names/secrets, private `DEVICE_URL`, Wi-Fi credentials, API tokens, or NVS secret values. [VERIFIED: 15-CONTEXT.md; VERIFIED: AGENTS.md]

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
| --- | --- | --- | --- |
| BM1366 frame encoding | Shell-generated hex frames or raw serial writes. [VERIFIED: 15-CONTEXT.md] | `bitaxe-asic` typed `Bm1366Command`, `diagnostic_job_frame`, `Bm1366WorkFields`, and result parser. [VERIFIED: crates/bitaxe-asic/src/bm1366/command.rs; VERIFIED: crates/bitaxe-asic/src/bm1366/work.rs; VERIFIED: crates/bitaxe-asic/src/bm1366/result.rs] | Prevents bypassing job-id, CRC, valid-job, and parser invariants already covered by tests. [VERIFIED: crates/bitaxe-asic/src/bm1366/result.rs] |
| Flash/monitor evidence | Direct ad hoc `espflash` invocations for trusted claims. [VERIFIED: docs/parity/evidence/phase-12-asic-and-mining-hardware-evidence.md] | `just flash-monitor ... evidence-dir=...` through `tools/flash`. [VERIFIED: Justfile; VERIFIED: tools/flash/src/main.rs] | Wrapper records command, board, port, commits, manifest, log path, trust status, and conclusion. [VERIFIED: tools/flash/src/main.rs] |
| Hardware run authorization | Free-form shell scripts with informal comments. [VERIFIED: docs/parity/evidence/phase-14-safety-hardware-evidence-completion/README.md] | Mining-specific allow manifest patterned after `safety-allow`. [VERIFIED: tools/parity/src/safety_allow.rs] | Machine-checks board, detector, package identity, recovery, safe-state, redaction, and checklist scope. [VERIFIED: tools/parity/src/safety_allow.rs] |
| Mining state model | Separate JSON counters in shell scripts. [VERIFIED: crates/bitaxe-api/src/mining.rs] | `MiningRuntimeState`, fake pool, work queue, and API mappers. [VERIFIED: crates/bitaxe-stratum/src/v1/state.rs; VERIFIED: crates/bitaxe-api/src/mining.rs] | Keeps accepted/rejected shares, pool lifecycle, hashrate inputs, fallback, and work gate consistent across API and telemetry. [VERIFIED: crates/bitaxe-stratum/src/v1/state.rs] |
| WebSocket capture dependency | Requiring `websocat`. [VERIFIED: environment audit] | Repo-owned Node script using Node 24 `WebSocket`, or a Rust host helper if richer evidence is needed. [VERIFIED: environment audit] | `websocat` is missing, but Node is present and has a WebSocket global. [VERIFIED: environment audit] |
| Redaction process | Grep-only proof or manual memory. [VERIFIED: docs/parity/evidence/phase-14-safety-hardware-evidence-completion/redaction-review.md] | Artifact-specific redaction review plus secret-pattern scan. [VERIFIED: docs/parity/evidence/phase-14-safety-hardware-evidence-completion/redaction-review.md] | Prior evidence policy requires every generated log, JSON, Markdown, API body, WebSocket frame, and pasted output to be reviewed before citation. [VERIFIED: 15-CONTEXT.md] |

**Key insight:** Phase 15 is about proving exact live behavior through trusted, bounded, redaction-reviewed artifacts; custom shortcuts are more likely to invalidate evidence than save implementation time. [VERIFIED: 15-CONTEXT.md; VERIFIED: docs/parity/evidence/phase-12-asic-and-mining-hardware-evidence.md]

## Common Pitfalls

### Pitfall 1: Repeating The ELF-Only Diagnostic Trust Failure

**What goes wrong:** The diagnostic image emits useful ASIC markers, but wrapper trust fails because SPIFFS/package boot markers are missing. [VERIFIED: docs/parity/evidence/phase-12-asic-and-mining-hardware-evidence.md]  
**Why it happens:** `tools/flash` trust requires package-level markers, including `spiffs_mount=available`, route shell startup, safe-state, and commit markers. [VERIFIED: tools/flash/src/main.rs]  
**How to avoid:** Build a package-backed diagnostic flow before running chip-detect/staged init evidence. [VERIFIED: 15-CONTEXT.md]  
**Warning signs:** `trusted_output=false`, `timed_out_without_trusted_output`, or `missing trusted Ultra 205 boot markers`. [VERIFIED: tools/flash/src/main.rs; VERIFIED: docs/parity/evidence/phase-12-asic-and-mining-hardware-evidence.md]

### Pitfall 2: Diagnostic Evidence Overclaims Production Mining

**What goes wrong:** A chip-detect, work packet, timeout, or parsed-result diagnostic is cited as full mining parity. [VERIFIED: 15-CONTEXT.md]  
**Why it happens:** The diagnostic exercises a narrow typed path and does not prove live pool lifecycle, accepted/rejected share behavior, hashrate, telemetry, watchdog responsiveness, or safe stop. [VERIFIED: 15-CONTEXT.md; VERIFIED: docs/parity/evidence/phase-12-asic-and-mining-hardware-evidence.md]  
**How to avoid:** Use exact claim language and keep broad ASIC/mining/API/statistics rows below `verified` unless a smoke/soak artifact proves the whole claim. [VERIFIED: tools/parity/src/main.rs]  
**Warning signs:** A checklist row says `verified` with only `unit`, `golden`, `workflow`, or diagnostic-only notes. [VERIFIED: tools/parity/src/main.rs]

### Pitfall 3: Live Pool Smoke Leaks Secrets

**What goes wrong:** Pool URL, worker name, password, private endpoint, or `DEVICE_URL` is committed in logs or API/WebSocket captures. [VERIFIED: 15-CONTEXT.md; VERIFIED: docs/parity/evidence/phase-14-safety-hardware-evidence-completion/redaction-review.md]  
**Why it happens:** Stratum logs and HTTP/WebSocket captures naturally include connection and state data. [VERIFIED: reference/esp-miner/main/tasks/stratum_v1_task.c; VERIFIED: firmware/bitaxe/src/http_api.rs]  
**How to avoid:** Prefer controlled no-share when live prerequisites are missing; when live smoke is allowed, use disposable/non-secret config and artifact-specific redaction before commit. [VERIFIED: 15-CONTEXT.md]  
**Warning signs:** Any generated artifact matches `pool`, `password`, `token`, `DEVICE_URL`, `ssid`, `http://`, `stratum`, or private IP patterns and has not been reviewed. [VERIFIED: docs/parity/evidence/phase-14-safety-hardware-evidence-completion/redaction-review.md]

### Pitfall 4: Watchdog Responsiveness Is Treated As A Log-Only Claim

**What goes wrong:** A mining smoke/soak run records shares but omits API/WebSocket/serial liveness and watchdog responsiveness. [VERIFIED: 15-CONTEXT.md]  
**Why it happens:** Mining work and UART reads can be bounded while telemetry tasks still starve. [CITED: https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/wdts.html]  
**How to avoid:** Require bounded duration, abort conditions, periodic status/API/WebSocket samples when `DEVICE_URL` is available, and final safe-stop/restore evidence. [VERIFIED: 15-CONTEXT.md]  
**Warning signs:** No duration, no samples, no safe-stop, or no final packaged safe boot restore. [VERIFIED: 15-CONTEXT.md]

### Pitfall 5: Reusing Phase 14 Safety Allow Unchanged

**What goes wrong:** A mining wrapper tries to validate against `safety-allow` and fails because allowed surfaces are only safety surfaces. [VERIFIED: tools/parity/src/safety_allow.rs]  
**Why it happens:** Phase 14 validator encodes `safe-baseline`, `power-telemetry`, `voltage-control`, `thermal-fan`, `self-test-watchdog-load`, `display-input`, `live-api-websocket-telemetry`, and `parity-redaction`, not BM1366/mining surfaces. [VERIFIED: tools/parity/src/safety_allow.rs]  
**How to avoid:** Add mining-specific allow surfaces or create a separate mining allow validator with the same structural checks. [VERIFIED: tools/parity/src/safety_allow.rs; VERIFIED: 15-CONTEXT.md]  
**Warning signs:** A plan says "use safety-allow" without adding surfaces or explaining the mapping. [VERIFIED: tools/parity/src/safety_allow.rs]

## Code Examples

### Trusted Wrapper Marker Contract

```rust
// Source: tools/flash/src/main.rs. [VERIFIED: local code]
// A trusted diagnostic capture must keep package/SPIFFS/API/safe-state/commit markers.
monitor_log_has_message(log, "bitaxe-rust boot: board=Ultra 205 asic=BM1366")
    && monitor_log_has_token(log, "spiffs_mount=available")
    && monitor_log_has_token(log, "axeos_api_route_shell=started")
```

### Diagnostic Mode Gate Shape

```rust
// Source: crates/bitaxe-asic/src/bm1366/adapter_gate.rs. [VERIFIED: local code]
// Extend this enum with explicit bounded diagnostic modes; default remains fail-closed.
pub enum AsicAdapterMode {
    FailClosed,
    ChipDetectOnly,
}
```

### Typed Work/Result Boundary

```rust
// Source: crates/bitaxe-asic/src/bm1366/work.rs and result.rs. [VERIFIED: local code]
let diagnostic = diagnostic_job_frame(Bm1366JobId::new(0x28), fields)?;
let mut valid_jobs = Bm1366ValidJobIds::empty();
valid_jobs.insert(diagnostic.job_id());
let parsed = parse_bm1366_result_frame(&frame, &valid_jobs, 16)?;
```

### Mining Loop Gate Boundary

```rust
// Source: crates/bitaxe-stratum/src/v1/mining_loop.rs. [VERIFIED: local code]
// Mining readiness requires power, thermal, safety, hardware ack, and initialized ASIC.
let decision = gate.decision();
```

### Allow-Manifest Validation Shape

```bash
# Source: Phase 14 safety evidence contract. [VERIFIED: docs/parity/evidence/phase-14-safety-hardware-evidence-completion/README.md]
bazel run //tools/parity:report -- safety-allow \
  --manifest <path> \
  --surface <surface> \
  --allowed-command <command>
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
| --- | --- | --- | --- |
| ELF-only chip-detect diagnostic and manual evidence interpretation. [VERIFIED: docs/parity/evidence/phase-12-asic-and-mining-hardware-evidence.md] | Package-backed diagnostic flow with wrapper trust markers preserved. [VERIFIED: 15-CONTEXT.md] | Phase 15 planning after Phase 12 verification. [VERIFIED: 15-CONTEXT.md] | Avoids repeating wrapper-untrusted chip-detect and enables exact chip-detect/staged-init claims. [VERIFIED: docs/parity/evidence/phase-12-asic-and-mining-hardware-evidence.md] |
| Safety wrappers without machine allow validation. [VERIFIED: .planning/phases/14-safety-hardware-evidence-completion/14-VALIDATION.md] | Phase 14 typed allow-manifest validator and component evidence packs. [VERIFIED: tools/parity/src/safety_allow.rs; VERIFIED: docs/parity/evidence/phase-14-safety-hardware-evidence-completion/README.md] | Phase 14. [VERIFIED: .planning/phases/14-safety-hardware-evidence-completion/14-VALIDATION.md] | Phase 15 should mirror this for mining because active evidence needs procedure-level gates. [VERIFIED: 15-CONTEXT.md] |
| Treating smoke/soak criteria as workflow documentation only. [VERIFIED: docs/parity/checklist.md] | `tools/parity` rejects live ASIC/mining verified rows without hardware-smoke/soak evidence and STR-008 share/no-share metadata. [VERIFIED: tools/parity/src/main.rs] | Phase 12 tooling. [VERIFIED: .planning/phases/12-asic-and-mining-hardware-evidence/12-VERIFICATION.md] | Checklist promotion can be automated for obvious overclaims but may still need additional Phase 15 guards for new row semantics. [VERIFIED: 15-CONTEXT.md] |
| WebSocket evidence blocked by missing maintained client. [VERIFIED: docs/parity/evidence/phase-14-safety-hardware-evidence-completion.md] | Use explicit `DEVICE_URL` plus repo-owned Node or Rust WebSocket helper when live telemetry is in scope. [VERIFIED: environment audit; CITED: https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/protocols/esp_http_server.html] | Phase 15 planning. [VERIFIED: environment audit] | Avoids adding a global `websocat` dependency and keeps capture code reviewable. [VERIFIED: environment audit] |

**Deprecated/outdated:**

- ELF-only diagnostic flashes are inappropriate for `verified` evidence because they can miss wrapper trust markers. [VERIFIED: docs/parity/evidence/phase-12-asic-and-mining-hardware-evidence.md]
- Unit, golden, fake-pool, API fixture, and safe-boot evidence do not verify live BM1366 work/result or production mining behavior. [VERIFIED: docs/parity/evidence/phase-12-asic-and-mining-hardware-evidence.md; VERIFIED: tools/parity/src/main.rs]
- Unbounded live-pool mining stress is out of scope for Phase 15. [VERIFIED: 15-CONTEXT.md]

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
| --- | --- | --- | --- |
| None | All planner-relevant claims in this research are sourced from local code/docs, current environment probes, or official docs. [VERIFIED: source audit] | All sections | No user confirmation is required for assumptions, but live pool credentials and `DEVICE_URL` availability remain open prerequisites. [VERIFIED: environment audit] |

## Open Questions

1. **Will Phase 15 receive an explicit reachable `DEVICE_URL`?** [VERIFIED: environment audit]
   - What we know: `DEVICE_URL` was unset during research, and Phase 14 live telemetry blocked for the same class of reason. [VERIFIED: environment audit; VERIFIED: docs/parity/evidence/phase-14-safety-hardware-evidence-completion.md]
   - What's unclear: Whether the execution environment will provide a reachable device URL after flashing. [VERIFIED: environment audit]
   - Recommendation: Plan API/WebSocket telemetry as conditional; record pending evidence if `DEVICE_URL` remains absent. [VERIFIED: 15-CONTEXT.md]

2. **Will live pool micro-smoke use disposable/non-secret config?** [VERIFIED: environment audit]
   - What we know: `BITAXE_POOL_URL`, `BITAXE_POOL_USER`, and `BITAXE_POOL_PASSWORD` were unset during research. [VERIFIED: environment audit]
   - What's unclear: Whether a live pool target will be supplied during execution. [VERIFIED: environment audit]
   - Recommendation: Default to controlled no-share/local deterministic evidence and gate live smoke behind explicit non-secret or disposable pool prerequisites. [VERIFIED: 15-CONTEXT.md]

3. **Should mining allow validation be a new `tools/parity` module or an extension of `safety_allow`?** [VERIFIED: tools/parity/src/safety_allow.rs]
   - What we know: Existing `safety_allow` fields are a strong pattern, but allowed surfaces are safety-specific. [VERIFIED: tools/parity/src/safety_allow.rs]
   - What's unclear: Whether maintainers prefer a generic allow validator or a separate mining-specific one. [VERIFIED: codebase inspection]
   - Recommendation: Prefer a small `mining_allow` module or explicit mining surfaces to avoid confusing safety and mining evidence semantics. [VERIFIED: tools/parity/src/safety_allow.rs; VERIFIED: 15-CONTEXT.md]

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
| --- | --- | --- | --- | --- |
| `just` | Human command surface and detector/package/flash/parity commands. [VERIFIED: Justfile] | Yes [VERIFIED: environment audit] | `1.48.0` [VERIFIED: environment audit] | None needed. |
| `bazel` | Firmware/package/tool/test graph. [VERIFIED: firmware/bitaxe/BUILD.bazel] | Yes [VERIFIED: environment audit] | `9.1.1` [VERIFIED: environment audit] | None needed. |
| `cargo` / `rustc` | Rust crates, host tools, and firmware build wrapper. [VERIFIED: scripts/build-firmware.sh] | Yes [VERIFIED: environment audit] | Cargo `1.88.0-nightly`; rustc `1.88.0-nightly`. [VERIFIED: environment audit] | None needed. |
| ESP export | Firmware build environment. [VERIFIED: scripts/build-firmware.sh] | Yes [VERIFIED: environment audit] | `~/export-esp.sh` present. [VERIFIED: environment audit] | Run `just bootstrap-esp` if missing. [VERIFIED: AGENTS.md] |
| Managed ESP-IDF tools | SPIFFS and factory image packaging. [VERIFIED: scripts/package-firmware.sh] | Yes [VERIFIED: environment audit] | `.embuild/espressif` with `spiffsgen.py` and `esptool.py` present. [VERIFIED: environment audit] | Run `just build`/`just package` then `just doctor` if missing. [VERIFIED: AGENTS.md; VERIFIED: scripts/esp-doctor.sh] |
| `espflash` | Board detection, board-info, flash, monitor, and evidence capture. [VERIFIED: scripts/detect-ultra205.sh; VERIFIED: tools/flash/src/main.rs] | Yes [VERIFIED: environment audit] | `4.0.1` [VERIFIED: environment audit] | None for hardware evidence; official README lists board-info, flash, list-ports, monitor, save-image, and write-bin commands. [CITED: https://github.com/esp-rs/espflash/blob/main/espflash/README.md] |
| Ultra 205 over USB | Live hardware evidence. [VERIFIED: AGENTS.md] | Yes [VERIFIED: `just detect-ultra205`] | Port `/dev/cu.usbmodem1101`; ESP32-S3 rev `v0.2`; 16MB flash. [VERIFIED: `just detect-ultra205`] | If missing later, record hardware evidence pending. [VERIFIED: AGENTS.md] |
| `DEVICE_URL` | Live HTTP/API/WebSocket telemetry capture. [VERIFIED: 15-CONTEXT.md] | No [VERIFIED: environment audit] | Unset. [VERIFIED: environment audit] | Controlled no-share evidence and serial-only status; live API/WebSocket rows remain below verified. [VERIFIED: 15-CONTEXT.md] |
| Pool env/config | Live pool micro-smoke. [VERIFIED: 15-CONTEXT.md] | No [VERIFIED: environment audit] | `BITAXE_POOL_URL`, `BITAXE_POOL_USER`, `BITAXE_POOL_PASSWORD` unset. [VERIFIED: environment audit] | Local deterministic Stratum harness or controlled no-share path. [VERIFIED: 15-CONTEXT.md] |
| `curl` | HTTP probes. [VERIFIED: scripts/phase13-http-static-smoke.sh; VERIFIED: scripts/phase14-live-telemetry.sh] | Yes [VERIFIED: environment audit] | `8.7.1` [VERIFIED: environment audit] | None for HTTP. |
| WebSocket client | `/api/ws` and `/api/ws/live` frames. [VERIFIED: firmware/bitaxe/src/http_api.rs] | Partial [VERIFIED: environment audit] | `websocat` missing; Node `v24.13.0` has `WebSocket`. [VERIFIED: environment audit] | Repo-owned Node or Rust helper. [VERIFIED: environment audit] |

**Missing dependencies with no fallback:**

- None for package-backed serial evidence, because `just`, Bazel, Cargo/Rust, ESP export, ESP-IDF managed tools, `espflash`, and one Ultra 205 USB port are available. [VERIFIED: environment audit]

**Missing dependencies with fallback:**

- `DEVICE_URL` is missing; fallback is serial-only and controlled no-share evidence with live API/WebSocket claims below `verified`. [VERIFIED: environment audit; VERIFIED: 15-CONTEXT.md]
- Live pool config is missing; fallback is local deterministic Stratum harness or controlled no-share evidence. [VERIFIED: environment audit; VERIFIED: 15-CONTEXT.md]
- `websocat` is missing; fallback is a repo-owned Node or Rust WebSocket helper. [VERIFIED: environment audit]

## Validation Architecture

### Test Framework

| Property | Value |
| --- | --- |
| Framework | Cargo/Rust tests plus Bazel tests; `nyquist_validation` is enabled. [VERIFIED: .planning/config.json; VERIFIED: Cargo.toml; VERIFIED: BUILD.bazel] |
| Config file | Workspace `Cargo.toml`, package `BUILD.bazel` files, `Justfile`. [VERIFIED: Cargo.toml; VERIFIED: Justfile] |
| Quick run command | `cargo test -p bitaxe-asic --all-features bm1366 && cargo test -p bitaxe-stratum --all-features mining && cargo test -p bitaxe-parity --all-features` [VERIFIED: Cargo.toml; VERIFIED: crates/bitaxe-asic; VERIFIED: crates/bitaxe-stratum; VERIFIED: tools/parity] |
| Full suite command | `just test && just parity && just verify-reference` plus Rust pre-commit checks before commit. [VERIFIED: Justfile; VERIFIED: AGENTS.md] |

### Phase Requirements To Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
| --- | --- | --- | --- | --- |
| ASIC-07 | Diagnostic package preserves wrapper trust and chip-detect/staged-init status markers. [VERIFIED: 15-CONTEXT.md] | unit/workflow/hardware-gated | `cargo test -p bitaxe-asic --all-features adapter_gate chip_detect init_plan && bazel test //tools/flash:tests && just detect-ultra205 && just flash-monitor board=205 port=<port> evidence-dir=<path>` [VERIFIED: crates/bitaxe-asic; VERIFIED: tools/flash; VERIFIED: Justfile] | Partial; package-backed diagnostic target is Wave 0 gap. [VERIFIED: firmware/bitaxe/BUILD.bazel] |
| ASIC-07 | Typed work-send and result-receive diagnostic emits exact result-or-timeout/fail-closed markers. [VERIFIED: 15-CONTEXT.md] | unit/integration/hardware-gated | `cargo test -p bitaxe-asic --all-features work result transcript && cargo test -p bitaxe-stratum --all-features mining_loop` [VERIFIED: crates/bitaxe-asic; VERIFIED: crates/bitaxe-stratum] | Partial; firmware diagnostic mode and wrapper are Wave 0 gaps. [VERIFIED: firmware/bitaxe/src/asic_adapter.rs] |
| STR-06 | Mining loop remains safety-gated and only reaches active state under exact evidence prerequisites. [VERIFIED: .planning/REQUIREMENTS.md] | unit/workflow/hardware-gated | `cargo test -p bitaxe-stratum --all-features mining_loop fake_pool queue && cargo test -p bitaxe-api --all-features mining` [VERIFIED: crates/bitaxe-stratum; VERIFIED: crates/bitaxe-api] | Yes for pure logic; live runner is Wave 0 gap. [VERIFIED: crates/bitaxe-stratum/src/v1/mining_loop.rs] |
| STR-07 | Smoke/soak evidence records share/no-share, duration, telemetry/watchdog, safe-stop, redaction, and conclusion. [VERIFIED: 15-CONTEXT.md] | workflow/hardware-gated | `bash -n scripts/phase15-*.sh && bazel test //scripts:phase15_*_test && just parity` after scripts exist. [VERIFIED: scripts/BUILD.bazel pattern; VERIFIED: tools/parity/src/main.rs] | No; Phase 15 wrappers are Wave 0 gaps. [VERIFIED: repository scan] |
| SAFE-09 | Bounded run preserves watchdog/API/WebSocket/serial responsiveness. [VERIFIED: .planning/REQUIREMENTS.md] | workflow/hardware-gated | `curl "$DEVICE_URL/api/system/info"` plus WebSocket helper when `DEVICE_URL` exists; otherwise record pending. [VERIFIED: firmware/bitaxe/src/http_api.rs; VERIFIED: environment audit] | Partial; helper is Wave 0 gap. [VERIFIED: repository scan] |
| EVD-05 | Checklist and parity guards reject overclaims and evidence artifacts pass redaction. [VERIFIED: .planning/REQUIREMENTS.md] | unit/workflow | `cargo test -p bitaxe-parity --all-features && just parity && rg -n "pool|password|token|DEVICE_URL|ssid|secret" docs/parity/evidence/phase-15-bm1366-mining-evidence-completion` [VERIFIED: tools/parity/src/main.rs; VERIFIED: docs/parity/evidence/phase-14-safety-hardware-evidence-completion/redaction-review.md] | Partial; Phase 15 ledger/redaction files are Wave 0 gaps. [VERIFIED: repository scan] |

### Sampling Rate

- **Per task commit:** Run targeted crate/script/tool tests for the touched surface. [VERIFIED: standards/core/verification.md; VERIFIED: AGENTS.md]
- **Per wave merge:** Run `just parity`, `just test`, and `just verify-reference`; run detector/hardware commands only when the wave owns hardware evidence. [VERIFIED: Justfile; VERIFIED: AGENTS.md]
- **Phase gate:** Full Rust pre-commit checks, `just test`, `just parity`, `just verify-reference`, reference diff cleanliness, redaction review, lifecycle validation, and hardware commands actually used. [VERIFIED: AGENTS.md; VERIFIED: 15-CONTEXT.md]

### Wave 0 Gaps

- [ ] Add package-backed diagnostic target/wrapper for BM1366 chip-detect/staged init. [VERIFIED: firmware/bitaxe/BUILD.bazel; VERIFIED: docs/parity/evidence/phase-12-asic-and-mining-hardware-evidence.md]
- [ ] Add bounded firmware diagnostic mode for typed work-send/result-receive. [VERIFIED: firmware/bitaxe/src/asic_adapter.rs; VERIFIED: crates/bitaxe-asic/src/bm1366/work.rs; VERIFIED: crates/bitaxe-asic/src/bm1366/result.rs]
- [ ] Add mining-specific allow manifest or extend parity allow surfaces for `bm1366-chip-detect`, `bm1366-work-result`, `mining-smoke`, `bounded-soak`, and `parity-redaction`. [VERIFIED: tools/parity/src/safety_allow.rs]
- [ ] Add Phase 15 evidence scaffold and artifact-specific redaction review. [VERIFIED: docs/parity/evidence/phase-14-safety-hardware-evidence-completion/README.md]
- [ ] Add controlled mining smoke/soak wrapper with conditional `DEVICE_URL`, pool, WebSocket, watchdog, and safe-stop behavior. [VERIFIED: 15-CONTEXT.md; VERIFIED: environment audit]

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
| --- | --- | --- |
| V2 Authentication | No new authentication surface is required unless a diagnostic HTTP/admin fallback is added. [VERIFIED: 15-CONTEXT.md] | Prefer serial compile-gated diagnostics; if HTTP fallback is added, make it compile-gated and impossible to expose accidentally in production. [VERIFIED: 15-CONTEXT.md] |
| V3 Session Management | No session state is introduced by the baseline serial diagnostic. [VERIFIED: firmware/bitaxe/src/asic_adapter.rs] | Do not add persistent diagnostic sessions. [VERIFIED: 15-CONTEXT.md] |
| V4 Access Control | Yes for any diagnostic route or active mining wrapper. [VERIFIED: 15-CONTEXT.md] | Gate diagnostics by compile-time mode, detector, package identity, allow manifest, and explicit command. [VERIFIED: 15-CONTEXT.md; VERIFIED: tools/parity/src/safety_allow.rs] |
| V5 Input Validation | Yes. [VERIFIED: scripts/detect-ultra205.sh; VERIFIED: tools/parity/src/safety_allow.rs] | Parse CLI args/manifests/API bodies with typed parsers and domain types; reject missing or malformed inputs before hardware action. [VERIFIED: tools/parity/src/safety_allow.rs; VERIFIED: firmware/bitaxe/src/http_api.rs] |
| V6 Cryptography | No new security cryptography should be introduced. [VERIFIED: repository scan] | Do not hand-roll crypto; keep CRC/hash behavior inside protocol modules only and treat it as protocol validation, not security. [VERIFIED: crates/bitaxe-asic/src/bm1366/crc.rs; VERIFIED: crates/bitaxe-stratum/src/v1/coinbase.rs] |

### Known Threat Patterns for Phase 15

| Pattern | STRIDE | Standard Mitigation |
| --- | --- | --- |
| Secret leakage in serial/API/WebSocket/pool evidence. [VERIFIED: 15-CONTEXT.md] | Information Disclosure | Artifact-specific redaction review, no committed credentials, no private endpoints, and controlled no-share fallback. [VERIFIED: docs/parity/evidence/phase-14-safety-hardware-evidence-completion/redaction-review.md; VERIFIED: 15-CONTEXT.md] |
| Unauthorized or accidental active mining/diagnostic mode. [VERIFIED: 15-CONTEXT.md] | Elevation of Privilege / Tampering | Compile-time diagnostic gates, detector gate, board-info, package identity, allow manifest, abort conditions, and safe-stop markers. [VERIFIED: firmware/bitaxe/src/asic_adapter.rs; VERIFIED: scripts/detect-ultra205.sh; VERIFIED: tools/parity/src/safety_allow.rs] |
| Evidence overclaim or unverifiable hardware proof. [VERIFIED: tools/parity/src/main.rs] | Repudiation | Exact claim ledger, evidence-class matching, command/log/commit metadata, and `just parity` overclaim guards. [VERIFIED: tools/parity/src/main.rs; VERIFIED: docs/parity/evidence/phase-12-asic-and-mining-hardware-evidence.md] |
| Hardware damage from unbounded mining stress or missing recovery. [VERIFIED: AGENTS.md; VERIFIED: 15-CONTEXT.md] | Tampering / Denial of Service | Bounded smoke/soak only, stop conditions, no voltage/fan stress, final safe-stop or trusted packaged restore. [VERIFIED: 15-CONTEXT.md; VERIFIED: AGENTS.md] |
| Network target discovery or private target exposure. [VERIFIED: docs/parity/evidence/phase-14-safety-hardware-evidence-completion.md] | Information Disclosure | Require explicit `DEVICE_URL`; do not scan or infer targets; redact private endpoint values. [VERIFIED: 15-CONTEXT.md; VERIFIED: scripts/phase14-live-telemetry.sh] |

## Sources

### Primary (HIGH confidence)

- `.planning/phases/15-bm1366-mining-evidence-completion/15-CONTEXT.md` - locked Phase 15 decisions, discretion, deferred scope, canonical refs, and code context. [VERIFIED: initial read]
- `.planning/ROADMAP.md` - Phase 15 goal, dependencies, success criteria, verification expectations, and research flags. [VERIFIED: initial read]
- `.planning/REQUIREMENTS.md` - ASIC-07, STR-06, STR-07, SAFE-09, and EVD-05 definitions. [VERIFIED: initial read]
- `.planning/STATE.md` - current project state, prior decisions, blockers, and Phase 15 position. [VERIFIED: initial read]
- `AGENTS.md`, `AGENTS.bright-builds.md`, `standards/core/architecture.md`, `standards/core/verification.md`, `standards/core/testing.md`, `standards/core/code-shape.md`, `standards/languages/rust.md` - repo rules and Bright Builds standards. [VERIFIED: repo instruction read]
- `docs/parity/evidence/phase-12-asic-and-mining-hardware-evidence.md` and `12-VERIFICATION.md` - Phase 12 chip-detect trust failure, safe boot restore, and residual mining gaps. [VERIFIED: local read]
- `docs/parity/evidence/phase-14-safety-hardware-evidence-completion.md`, `README.md`, `redaction-review.md`, `14-VALIDATION.md`, and `14-VERIFICATION.md` - allow-manifest/evidence-pack/redaction pattern. [VERIFIED: local read]
- `tools/flash/src/main.rs` - trusted wrapper marker classification and evidence JSON behavior. [VERIFIED: local read]
- `tools/parity/src/main.rs` and `tools/parity/src/safety_allow.rs` - overclaim guards and allow-manifest validator. [VERIFIED: local read]
- `crates/bitaxe-asic`, `crates/bitaxe-stratum`, `crates/bitaxe-api`, `crates/bitaxe-safety`, and `firmware/bitaxe` source files listed in context. [VERIFIED: local read]
- `reference/esp-miner` BM1366, ASIC common, system, and Stratum task files. [VERIFIED: local read]
- Environment probes: tool versions, `just detect-ultra205`, `DEVICE_URL`/pool env presence, Node WebSocket global, and project skill directory checks. [VERIFIED: environment audit]

### Official External Sources (HIGH confidence)

- `espflash` README - confirmed `board-info`, `flash`, `list-ports`, `monitor`, `save-image`, and `write-bin` command coverage. [CITED: https://github.com/esp-rs/espflash/blob/main/espflash/README.md]
- ESP-IDF HTTP Server docs - confirmed WebSocket support and async send API surface. [CITED: https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/protocols/esp_http_server.html]
- ESP-IDF Watchdog docs - confirmed Task Watchdog detects tasks that run too long without yielding. [CITED: https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/wdts.html]
- `esp-idf-svc` WebSocket example - confirmed Rust-side WebSocket handler example exists for the current crate family. [CITED: https://docs.rs/crate/esp-idf-svc/latest/source/examples/http_ws_server.rs]

### Secondary (MEDIUM confidence)

- None used as authoritative evidence. [VERIFIED: source audit]

### Tertiary (LOW confidence)

- None used. [VERIFIED: source audit]

## Metadata

**Confidence breakdown:**

- Standard stack: HIGH - all recommended packages/tools are existing repo dependencies or installed tools verified locally. [VERIFIED: Cargo.toml; VERIFIED: Cargo.lock; VERIFIED: environment audit]
- Architecture: HIGH - recommendations directly follow existing local modules, Phase 12/14 evidence patterns, and locked Phase 15 decisions. [VERIFIED: local code read; VERIFIED: 15-CONTEXT.md]
- Pitfalls: HIGH - major pitfalls are prior observed failures or enforced parity/checklist rules. [VERIFIED: docs/parity/evidence/phase-12-asic-and-mining-hardware-evidence.md; VERIFIED: tools/parity/src/main.rs]
- Live pool execution: MEDIUM - live pool and `DEVICE_URL` were absent during research, so plan must keep live smoke conditional. [VERIFIED: environment audit]

**Research date:** 2026-07-01  
**Valid until:** 2026-07-31 for local architecture and evidence contracts; re-audit environment and external tool versions immediately before hardware execution. [VERIFIED: environment audit]

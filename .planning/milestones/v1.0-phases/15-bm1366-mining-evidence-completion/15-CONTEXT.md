---
generated_by: gsd-discuss-phase
lifecycle_mode: yolo
phase_lifecycle_id: 15-2026-07-01T02-07-59
generated_at: 2026-07-01T02:14:24.605Z
---

# Phase 15: BM1366 Mining Evidence Completion - Context

**Gathered:** 2026-07-01
**Status:** Ready for planning
**Mode:** Yolo

<domain>
## Phase Boundary

Phase 15 closes the current Ultra 205 BM1366 mining evidence gap. It must produce trusted board-205 evidence for BM1366 chip-detect or staged initialization, typed work-send/result-receive behavior, and controlled mining smoke or bounded soak when prerequisites are present.

The phase owns ASIC/mining evidence only. It does not own same-commit release HTTP/static/recovery/OTA evidence, rollback/erase/interrupted-update recovery, non-205 boards, Stratum v2, BAP, all-board factory images, active voltage or fan stress, broad runtime display/input parity, or production mining performance tuning.

All live hardware work must begin with `just detect-ultra205` and continue only when exactly one likely ESP32-S3 serial port is found, `espflash board-info --chip esp32s3 --port <port> --non-interactive` succeeds, board `205` is confirmed, recovery and stop conditions are documented, and generated artifacts can pass redaction review.

</domain>

<decisions>
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

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase Scope And Governance

- `.planning/ROADMAP.md` - Phase 15 goal, gap closure, requirements, success criteria, verification expectations, and controlled pool/recovery research flag.
- `.planning/REQUIREMENTS.md` - ASIC-07, STR-06, STR-07, SAFE-09, and EVD-05 traceability for trusted BM1366 and mining evidence.
- `.planning/PROJECT.md` - Ultra 205 BM1366 first target, ESP-IDF Rust stack, read-only reference policy, evidence policy, architecture, licensing, and safety constraints.
- `.planning/STATE.md` - Current project state after Phase 14 and accumulated ASIC/mining/safety evidence blockers.
- `AGENTS.md` - Repo-local autonomous Ultra 205 detector gate, stop conditions, evidence metadata, destructive/fault-injection limits, and frontmatter separator rule.
- `standards/core/verification.md` - Repo-native verification and pre-commit expectations.
- `standards/core/testing.md` - Unit-test expectations for changed pure logic.
- `standards/languages/rust.md` - Rust module, naming, invariant, test, and verification guidance.

### Prior Phase Decisions And Evidence

- `.planning/phases/03-bm1366-asic-protocol-and-safe-initialization/03-CONTEXT.md` - BM1366 semantic command/observation boundary, staged init gate, and hardware evidence rules.
- `.planning/phases/04-stratum-v1-and-first-mining-loop/04-CONTEXT.md` - Stratum v1, fake-pool, mining loop, runtime state, and first-loop evidence boundaries.
- `.planning/phases/12-asic-and-mining-hardware-evidence/12-CONTEXT.md` - Phase 12 evidence ladder, chip-detect trust failure, work/result pending status, mining preflight, and redaction policy.
- `.planning/phases/12-asic-and-mining-hardware-evidence/12-VERIFICATION.md` - Passed Phase 12 evidence-governance result and residual live ASIC/mining gaps.
- `.planning/phases/14-safety-hardware-evidence-completion/14-CONTEXT.md` - Safety allow-manifest pattern, exact-claim checklist promotion, and final verification expectations.
- `.planning/phases/14-safety-hardware-evidence-completion/14-VERIFICATION.md` - Current safety evidence boundary and residual active-control/live-telemetry blockers.
- `docs/parity/evidence/phase-12-asic-and-mining-hardware-evidence.md` - Phase 12 ledger with untrusted chip-detect capture, blocked mining smoke, restore evidence, and claim matrix.
- `docs/parity/evidence/phase-14-safety-hardware-evidence-completion.md` - Phase 14 ledger with allow manifests, redaction review, and live telemetry blockers.
- `docs/parity/evidence/phase-12-asic-and-mining-hardware-evidence/chip-detect/flash-command-evidence.json` - Prior untrusted chip-detect machine evidence shape.
- `docs/parity/evidence/phase-12-asic-and-mining-hardware-evidence/chip-detect/flash-monitor.log` - Prior chip-detect markers and partial UART read failure.
- `docs/parity/evidence/phase-12-asic-and-mining-hardware-evidence/safe-boot-restore/flash-command-evidence.json` - Trusted restore package evidence shape.
- `docs/parity/evidence/phase-12-asic-and-mining-hardware-evidence/safe-boot-restore/flash-monitor.log` - Trusted restore serial markers.

### Current Tooling And Implementation Surfaces

- `Justfile` - Human command surface for `detect-ultra205`, `package`, `flash-monitor`, `monitor`, `parity`, `verify-reference`, `build`, and `test`.
- `scripts/detect-ultra205.sh` - Required detector and board-info preflight before autonomous hardware use.
- `scripts/phase12-mining-smoke-preflight.sh` - Existing fail-closed mining smoke preflight pattern.
- `tools/flash/src/main.rs` - Flash/monitor evidence capture, trusted output classification, package manifest lookup, and JSON/log artifact behavior.
- `tools/parity/src/main.rs` - Checklist validation, evidence-token guard, and release-sensitive overclaim checks.
- `tools/parity/src/safety_allow.rs` - Phase 14 allow-manifest validation pattern to reuse or mirror for BM1366/mining probes when useful.
- `crates/bitaxe-asic/src/bm1366.rs` - BM1366 module entrypoint.
- `crates/bitaxe-asic/src/bm1366/adapter_gate.rs` - Hardware evidence acknowledgement and diagnostic gate model.
- `crates/bitaxe-asic/src/bm1366/chip_detect.rs` - Pure chip-detect validation and follow-up actions.
- `crates/bitaxe-asic/src/bm1366/command.rs` - Typed BM1366 command boundary.
- `crates/bitaxe-asic/src/bm1366/init_plan.rs` - Staged initialization plan and no-mining states.
- `crates/bitaxe-asic/src/bm1366/observation.rs` - Typed BM1366 observations.
- `crates/bitaxe-asic/src/bm1366/result.rs` - Result parsing and valid-job tracking.
- `crates/bitaxe-asic/src/bm1366/transcript.rs` - Fake UART transcript seam.
- `crates/bitaxe-asic/src/bm1366/work.rs` - Work payload and diagnostic job modeling.
- `crates/bitaxe-stratum/src/v1/fake_pool.rs` - Deterministic fake-pool harness.
- `crates/bitaxe-stratum/src/v1/mining.rs` - Mining job construction and share decision logic.
- `crates/bitaxe-stratum/src/v1/mining_loop.rs` - First mining-loop state machine and safety evidence gates.
- `crates/bitaxe-stratum/src/v1/queue.rs` - Work queue behavior.
- `crates/bitaxe-stratum/src/v1/state.rs` - Pool lifecycle, counters, and runtime state.
- `crates/bitaxe-api/src/mining.rs` - API-visible mining status model.
- `crates/bitaxe-api/src/asic.rs` - API-visible ASIC status/settings model.
- `crates/bitaxe-safety/src/evidence.rs` - Safety evidence token model.
- `firmware/bitaxe/src/asic_adapter.rs` - Firmware interpreter for typed BM1366 actions.
- `firmware/bitaxe/src/asic_adapter/status.rs` - ASIC and mining status logging.
- `firmware/bitaxe/src/asic_adapter/uart.rs` - ESP-IDF UART adapter.
- `firmware/bitaxe/src/safety_adapter.rs` - Firmware safety facade and preflight inputs.
- `firmware/bitaxe/src/runtime_snapshot.rs` - Runtime status and telemetry integration point.
- `firmware/bitaxe/src/http_api.rs` - ESP-IDF HTTP and WebSocket route shell for live telemetry probes when `DEVICE_URL` is available.

### Upstream Reference And Policy

- `reference/esp-miner/config-205.cvs` - Ultra 205 BM1366 defaults and pool-related config.
- `reference/esp-miner/main/device_config.h` - Ultra 205 board profile, capabilities, and expected ASIC count.
- `reference/esp-miner/components/asic/asic.c` - Shared ASIC dispatch behavior.
- `reference/esp-miner/components/asic/bm1366.c` - BM1366 initialization, packet, work, result, and nonce behavior.
- `reference/esp-miner/components/asic/asic_common.c` - Chip counting, receive-work validation, difficulty mask, and shared result behavior.
- `reference/esp-miner/main/power/asic_init.c` - Reset, UART, chip-detect, and max-baud initialization shell.
- `reference/esp-miner/main/power/asic_reset.c` - ASIC reset GPIO timing behavior.
- `reference/esp-miner/components/stratum/stratum_api.c` - Stratum v1 message handling.
- `reference/esp-miner/components/stratum/stratum_socket.c` - Pool socket lifecycle, reconnect, and fallback behavior.
- `reference/esp-miner/components/stratum/mining.c` - Coinbase hashing, merkle root, BM job construction, extranonce, and difficulty behavior.
- `reference/esp-miner/main/work_queue.c` - Upstream work queue behavior.
- `reference/esp-miner/main/tasks/protocol_coordinator.c` - Protocol lifecycle coordination.
- `reference/esp-miner/main/system.c` - Accepted/rejected share counters and clean-jobs handling.
- `reference/esp-miner/main/global_state.h` - Pool, share, queue, and mining status fields.
- `docs/adr/0001-device-user-parity.md` - Observable device-user parity definition.
- `docs/adr/0003-esp-idf-rust-production-stack.md` - ESP-IDF Rust stack and adapter boundary.
- `docs/adr/0005-read-only-reference-implementation.md` - Read-only upstream reference policy.
- `docs/adr/0006-parity-checklist-as-audit-evidence.md` - Checklist evidence policy.
- `docs/adr/0012-parity-verification-evidence.md` - Evidence requirements and hardware-control verification gate.
- `docs/adr/0013-mit-first-with-gpl-guardrails.md` - GPL guardrails for upstream-derived behavior and fixtures.
- `docs/adr/0014-pivot-to-ultra-205-bm1366-first-parity.md` - Ultra 205/BM1366 first parity target and deferred non-205 scope.
- `PROVENANCE.md` - Reference, GPL, fixture, and release artifact provenance policy.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets

- `just detect-ultra205` already implements the required board-info preflight and prints `port=<path>` only after one likely Ultra 205 serial port passes.
- `just flash-monitor board=205 port=<path> evidence-dir=<path>` already records wrapper-owned JSON and serial logs with board, port, source commit, reference commit, manifest, trusted-output status, and conclusion.
- Phase 12 evidence shows the exact root cause to avoid: ELF-only diagnostic flash observed chip-detect markers but failed wrapper trust because packaged SPIFFS markers were absent.
- `crates/bitaxe-asic` already owns typed BM1366 protocol, init, work, result, transcript, and adapter-gate logic.
- `crates/bitaxe-stratum` already owns Stratum v1 fake-pool, mining job, work queue, mining loop, and runtime state logic.
- `crates/bitaxe-safety` and `firmware/bitaxe/src/safety_adapter.rs` provide safety evidence and preflight concepts Phase 15 must not bypass.
- `tools/parity` already rejects unsupported verified claims; Phase 15 should extend this only for new machine-checkable ASIC/mining evidence semantics.

### Established Patterns

- Pure ASIC, Stratum, safety, and parity decisions live in host-testable Rust crates; ESP-IDF UART, GPIO, timing, serial monitoring, network, WebSocket, and task effects stay in firmware or tool adapters.
- Hardware evidence uses conservative, scoped conclusions and names board, port, source commit, reference commit, package identity, command, logs, observed behavior, and redaction result.
- Checklist status follows evidence, not implementation. Broad mixed claims stay below `verified` unless the artifact covers the exact behavior.
- GSD artifacts and evidence Markdown avoid standalone body `---` separators after YAML frontmatter.

### Integration Points

- Add Phase 15 evidence under `docs/parity/evidence/phase-15-bm1366-mining-evidence-completion.md` and component-scoped generated artifacts where useful.
- Add a package-backed BM1366 diagnostic evidence path or wrapper support so chip-detect diagnostics keep trusted SPIFFS/package markers.
- Add a bounded serial diagnostic runner only if needed to exercise typed work-send/result-receive without raw serial writes.
- Add controlled mining smoke/soak wrappers only behind detector, safety, ASIC, recovery, telemetry, safe-stop, and redaction gates.
- Update `docs/parity/checklist.md` only after evidence exists, using exact claim notes and leaving unsupported broad claims below `verified`.

</code_context>

<specifics>
## Specific Ideas

- Preferred first fix: packaged diagnostic image through the existing wrapper to close the Phase 12 chip-detect trust root cause.
- Preferred work/result path: typed firmware diagnostic over USB console with bounded result-or-timeout evidence and no pool credentials.
- Preferred mining path: layered evidence ladder with deterministic controlled no-share fallback; live pool micro-smoke only when disposable credentials, explicit `DEVICE_URL`, safe-stop, and redaction are available.
- Preferred promotion path: evidence-tiered exact-claim checklist updates, parity guard extensions only when rows are promoted, and evidence-only ledgers when proof remains partial.
- Every live run should restore or confirm final trusted packaged safe boot after active diagnostics.

</specifics>

<deferred>
## Deferred Ideas

- Same-commit package, flash, serial boot, live HTTP/static/recovery/OTA, rollback, erase, failed-update, and interrupted-update evidence belongs to Phase 16.
- Non-205 boards, BM1370/BM1368/BM1397, all-board factory images, Stratum v2, BAP, Angular AxeOS replacement, and production mining performance tuning remain deferred.
- Active voltage, fan duty, overheat/fault stimulus, self-test hardware submodes, runtime display/input parity, and broad live safety telemetry remain outside Phase 15 unless needed only as prerequisites and already covered by a bounded recovery plan.
- Long, unbounded mining stress and real-pool optimization are out of scope.

</deferred>

*Phase: 15-bm1366-mining-evidence-completion*
*Context gathered: 2026-07-01*

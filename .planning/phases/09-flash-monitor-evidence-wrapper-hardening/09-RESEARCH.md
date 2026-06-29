# Phase 09: Flash-Monitor Evidence Wrapper Hardening - Research

**Researched:** 2026-06-29
**Domain:** Rust CLI evidence wrapper, espflash serial monitor capture, Ultra 205 parity evidence
**Confidence:** HIGH

<user_constraints>
## User Constraints (from CONTEXT.md)

Source: `.planning/phases/09-flash-monitor-evidence-wrapper-hardening/09-CONTEXT.md`. [VERIFIED: 09-CONTEXT.md]

### Locked Decisions

## Implementation Decisions

### Noninteractive Monitor Capture

- **D-01:** Add a first-class wrapper-owned noninteractive evidence path for `flash-monitor --evidence-dir` that invokes `espflash monitor --chip esp32s3 --port <port> --non-interactive` after the flash command.
- **D-02:** Preserve normal interactive `just monitor` behavior for operator/manual use. Evidence capture should not depend on an interactive input reader or PTY-only behavior.
- **D-03:** Keep using esp-rs tooling. Do not add a custom serial backend or PTY dependency unless `espflash monitor --non-interactive` proves unable to capture the required Ultra 205 boot log.
- **D-04:** Define a bounded capture strategy for evidence mode so the wrapper does not hang indefinitely. The captured log must still fail closed if the process exits unsuccessfully, times out without trusted output, or cannot create the evidence log.

### Evidence Record Contract

- **D-05:** Enrich the existing `flash-command-evidence.json` as the canonical machine-readable evidence record instead of relying on manual prose transcription.
- **D-06:** The evidence record for `flash-monitor` must include command kind, board, selected port, source/firmware commit, reference commit, package manifest path, flash image path, exact flash command, exact monitor command, monitor log path, capture mode, capture status, timestamp, and a conclusion/status field.
- **D-07:** It is acceptable to add a generated Markdown evidence summary if the implementation can render it from the same structured record, but the JSON record remains the source of truth.
- **D-08:** Avoid manual evidence-only docs as the primary Phase 9 fix. Manual prose may summarize results, but it must cite wrapper-produced artifacts.

### Failure Handling And Recovery Guidance

- **D-09:** Monitor startup failures must fail visibly and explain that evidence capture is not trusted. The wrapper must not silently retry, silently switch modes, or leave a partial log presented as valid proof.
- **D-10:** Recovery guidance should point to repo-owned steps: rerun `just detect-ultra205`, confirm exactly one port, rerun `just flash-monitor board=205 port=<port> evidence-dir=<path>`, and use `just monitor port=<port>` or PTY/manual monitor only as diagnostic follow-up.
- **D-11:** If an input-reader failure is still observed, the recovery text should specifically steer operators to the wrapper's noninteractive evidence path rather than asking them to run raw `espflash monitor` directly.

### Hardware Evidence And Checklist Boundary

- **D-12:** Capture fresh wrapper-based Ultra 205 serial evidence after implementation when `just detect-ultra205` succeeds with exactly one ESP32-S3 candidate.
- **D-13:** Update workflow/release evidence so raw-monitor fallback is no longer the only proof for the serial boot evidence path. Prefer refreshing `WF-005` and relevant release docs over adding a duplicate checklist row unless the existing row cannot express the closure clearly.
- **D-14:** Do not promote `FS-001`, `OTA-001`, `OTA-002`, `REL-001`, `REL-002`, or `REL-003` to verified from Phase 9 serial evidence alone. Those rows still require the live HTTP/static/recovery/OTA/rollback evidence planned for later phases.
- **D-15:** Keep all evidence records free of secrets, private endpoints, pool credentials, Wi-Fi credentials, and NVS secret values.

### the agent's Discretion

The agent may choose exact CLI flag names, capture timeout defaults, evidence schema field names, helper function boundaries, tests, and whether to emit a Markdown summary in addition to JSON. Those choices must preserve the existing `tools/flash` command surface, keep `just` as the user entrypoint, keep `espflash` as the backend, use typed Rust structs for evidence records, and keep `reference/esp-miner` read-only.

### Deferred Ideas (OUT OF SCOPE)

- Live HTTP/static/recovery/OTA/rollback/large-erase/failed-update/interrupted-update evidence remains Phase 13 release evidence scope unless a later roadmap change says otherwise.
- A custom serial monitor backend remains deferred unless `espflash monitor --non-interactive` cannot produce reliable evidence.
- A dedicated new parity checklist row for wrapper evidence capture remains optional; start by refreshing existing workflow evidence unless `WF-005` proves too broad.
- Non-205 board verification, ASIC/mining hardware evidence, and safety-controller hardware regression evidence remain later phases.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
| --- | --- | --- |
| FND-07 | `just build`, `just test`, `just package`, `just flash`, `just monitor`, `just flash-monitor`, `just verify-reference`, and `just parity` must exist and route through Bazel or repo-owned scripts represented in the automation graph. [VERIFIED: .planning/REQUIREMENTS.md] | Keep `Justfile` unchanged as the human command surface and implement the evidence behavior inside `tools/flash`, which is already invoked by `just flash-monitor`. [VERIFIED: Justfile] [VERIFIED: tools/flash/src/main.rs] |
| FND-08 | USB flashing ergonomics must support `board=205`, optional `port=...`, likely-port discovery, clear ambiguous-port errors, build-before-flash by default, and printing the underlying flashing command. [VERIFIED: .planning/REQUIREMENTS.md] | Reuse existing `resolve_port`, `ensure_ultra_205`, `prepare_flash`, and command emission behavior; add only the evidence-mode monitor command shape. [VERIFIED: tools/flash/src/main.rs] |
| REL-07 | Build, flash, monitor, OTA, and recovery documentation must be sufficient for a developer with a connected Ultra 205 to operate safely. [VERIFIED: .planning/REQUIREMENTS.md] | Update `docs/release/ultra-205.md` to document wrapper-owned noninteractive evidence capture and fail-closed recovery guidance without changing OTA/recovery claims. [VERIFIED: docs/release/ultra-205.md] [VERIFIED: 09-CONTEXT.md] |
| EVD-05 | Verification layers include unit tests, golden fixtures, API comparison, hardware smoke tests, and hardware regression or soak evidence where appropriate. [VERIFIED: .planning/REQUIREMENTS.md] | Add unit tests for command construction, capture status, evidence schema, and failure handling; capture fresh Ultra 205 wrapper evidence only after `just detect-ultra205` passes. [VERIFIED: tools/flash/src/main.rs] [VERIFIED: scripts/detect-ultra205.sh] |
</phase_requirements>

## Summary

Phase 9 should be planned as a narrow hardening pass inside the existing Rust `tools/flash` CLI. The current wrapper already builds/packages, resolves the Ultra 205 port, flashes a manifest-selected image, captures stdout/stderr to `flash-monitor.log`, and writes `flash-command-evidence.json`, but evidence-mode `flash-monitor` still uses the same interactive monitor command as `just monitor`: `espflash monitor --port <port>`. [VERIFIED: tools/flash/src/main.rs]

The safe implementation model is to keep ordinary `just monitor` interactive and add a separate evidence-mode monitor command for `flash-monitor --evidence-dir`: `espflash monitor --chip esp32s3 --port <port> --non-interactive`. Local `espflash 4.0.1` help confirms `monitor` supports `--chip esp32s3`, `--port`, and `--non-interactive`, and Phase 8 proves the same command shape captured the missing Ultra 205 serial boot evidence when wrapped with a 25 second timeout. [VERIFIED: local `espflash monitor --help`] [VERIFIED: docs/parity/evidence/phase-08-ultra-205-release-gate.md]

**Primary recommendation:** Add a typed noninteractive evidence capture path in `tools/flash`, bounded by a Rust-owned timeout and a pure trusted-output classifier, then update `WF-005` and release evidence docs to cite wrapper-produced JSON/log artifacts without promoting HTTP, OTA, recovery, rollback, erase, ASIC, mining, or safety rows. [VERIFIED: 09-CONTEXT.md] [VERIFIED: docs/parity/checklist.md]

## Project Constraints (from AGENTS.md)

- Prefer `AGENTS.md`; also read `AGENTS.bright-builds.md`, `standards-overrides.md`, and relevant standards before planning or implementation. [VERIFIED: AGENTS.md] [VERIFIED: AGENTS.bright-builds.md]
- Do not modify the managed Bright Builds block in `AGENTS.md` or the managed `AGENTS.bright-builds.md` file. [VERIFIED: AGENTS.md] [VERIFIED: AGENTS.bright-builds.md]
- Before repo edits through normal development work, use a GSD workflow entrypoint; Phase 9 is already inside the GSD phase workflow. [VERIFIED: AGENTS.md] [VERIFIED: gsd init phase-op 09]
- Keep `reference/esp-miner` pinned, read-only, and unmodified. [VERIFIED: AGENTS.md] [VERIFIED: .planning/PROJECT.md]
- Prefer ESP-IDF and esp-rs tooling, including `espflash`, before custom CMake, PlatformIO, manual ESP-IDF installs, or custom binary/serial tooling. [VERIFIED: AGENTS.md]
- Treat `.embuild/` as generated local state; do not commit or hand-edit it. [VERIFIED: AGENTS.md]
- Use `just detect-ultra205` before autonomous Ultra 205 hardware use; continue only when exactly one likely ESP USB serial port is found and `espflash board-info --chip esp32s3 --port <port> --non-interactive` succeeds. [VERIFIED: AGENTS.md] [VERIFIED: scripts/detect-ultra205.sh]
- Every hardware run must record board `205`, selected port, source commit, reference commit, package manifest/artifacts when applicable, exact commands, board-info output, captured logs, observed behavior, and conclusion. [VERIFIED: AGENTS.md]
- Do not commit secrets, pool credentials, Wi-Fi credentials, private endpoints, or NVS secret values in evidence. [VERIFIED: AGENTS.md]
- For Rust commits, run `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, and `cargo test --all-features` before committing. [VERIFIED: AGENTS.md]
- Prefer functional core and imperative shell; pure decisions should be testable, and I/O should stay in thin adapters. [VERIFIED: standards/core/architecture.md] [VERIFIED: standards/languages/rust.md]
- Prefer early returns, `let...else`, typed invariants, and `maybe_` names for optional internal values. [VERIFIED: standards/core/code-shape.md] [VERIFIED: standards/languages/rust.md]
- Unit tests for pure/business logic are required, should test one concern, and should usually show Arrange, Act, Assert sections. [VERIFIED: standards/core/testing.md]
- In GSD/frontmatter-parsed Markdown, use standalone `---` only as top frontmatter delimiters; do not use body `---` separators. [VERIFIED: AGENTS.md]
- Project skill directories `.claude/skills/` and `.agents/skills/` were not present or contained no `SKILL.md` files in this checkout. [VERIFIED: `find .claude/skills .agents/skills -maxdepth 2 -name SKILL.md`]

## Standard Stack

### Core

| Library / Tool | Version | Purpose | Why Standard |
| --- | --- | --- | --- |
| `tools/flash` / `bitaxe-flash` | `0.1.0` | Repo-owned flash, monitor, flash-monitor, port resolution, and evidence capture CLI. | It is already the `Justfile` backend for `just flash`, `just monitor`, and `just flash-monitor`. [VERIFIED: tools/flash/Cargo.toml] [VERIFIED: Justfile] |
| `espflash` | local `4.0.1` | Flashing, board-info, and serial monitoring backend. | Repo-local guidance says prefer esp-rs tooling; local help confirms `monitor --chip esp32s3 --port <port> --non-interactive`. [VERIFIED: `espflash --version`] [VERIFIED: local `espflash monitor --help`] [VERIFIED: AGENTS.md] |
| Bazel | local `9.1.1` | Canonical build/test/package automation graph. | `just` routes `flash-monitor` through `bazel run //tools/flash:flash -- flash-monitor`. [VERIFIED: `bazel --version`] [VERIFIED: Justfile] |
| Rust | workspace edition `2021`; local `rustc 1.88.0-nightly` | CLI implementation and tests. | The workspace uses Rust 2021, and `tools/flash` is a Rust binary/test target. [VERIFIED: Cargo.toml] [VERIFIED: `rustc --version`] [VERIFIED: tools/flash/BUILD.bazel] |
| `clap` | workspace `4.6.1`; lock `4.6.1` | Typed CLI parsing and key-value alias normalization. | Existing CLI uses `Parser`, `Args`, and `Subcommand`; keep using it for any added flags such as capture timeout. [VERIFIED: Cargo.toml] [VERIFIED: Cargo.lock] [VERIFIED: tools/flash/src/main.rs] |
| `serde` / `serde_json` | `serde` workspace `1.0.228`; `serde_json` workspace `1.0.150` | Typed evidence JSON and package manifest parsing. | Existing `EvidenceRecord` and `PackageManifest` already use typed serde structs. [VERIFIED: Cargo.toml] [VERIFIED: tools/flash/src/main.rs] |
| `camino` | workspace `1.2.3`; lock `1.2.3` | UTF-8 paths for CLI args, manifests, and evidence paths. | Existing CLI path fields use `Utf8PathBuf`; preserve this boundary. [VERIFIED: Cargo.toml] [VERIFIED: Cargo.lock] [VERIFIED: tools/flash/src/main.rs] |
| `anyhow` | workspace `1.0.102`; lock `1.0.103` | CLI error propagation and context. | Existing code uses `bail!`, `Context`, and `Result`; use contextual fail-closed errors. [VERIFIED: Cargo.toml] [VERIFIED: Cargo.lock] [VERIFIED: tools/flash/src/main.rs] |

### Supporting

| Library / Tool | Version | Purpose | When to Use |
| --- | --- | --- | --- |
| `tempfile` | workspace `3.27.0`; lock `3.27.0` | Unit-test temporary evidence directories and manifests. | Existing `tools/flash` tests already use `tempdir`; use it for new evidence JSON/log tests. [VERIFIED: Cargo.toml] [VERIFIED: tools/flash/src/main.rs] |
| `just` | local `1.48.0` | Human command surface. | Use for operator and hardware verification commands, not as the implementation locus. [VERIFIED: `just --version`] [VERIFIED: Justfile] |
| `git` | local `2.53.0` | Firmware and reference commit recording. | Existing `LocalFlashEnvironment` records source and reference commits with `git rev-parse`. [VERIFIED: `git --version`] [VERIFIED: tools/flash/src/main.rs] |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
| --- | --- | --- |
| Wrapper-owned `espflash monitor --chip esp32s3 --port <port> --non-interactive` | Custom serial backend | Do not use unless `espflash` cannot capture reliable Ultra 205 boot logs; custom serial handling would reimplement esp-rs behavior. [VERIFIED: 09-CONTEXT.md] |
| Separate flash then evidence monitor commands | Single `espflash flash/write-bin --monitor --non-interactive` invocation | Do not use as the primary Phase 9 fix because separate commands preserve clearer flash versus monitor error attribution. [VERIFIED: 09-DISCUSSION-LOG.md via rg] [VERIFIED: local `espflash flash --help`] |
| Rust-owned timeout around `Command::spawn` | Shell `timeout 25 ...` | Do not depend on shell `timeout`; it was useful in Phase 8 but is not portable and is not part of the Rust CLI contract. [VERIFIED: docs/parity/evidence/phase-08-ultra-205-release-gate.md] [VERIFIED: `timeout --version`] |
| JSON source of truth | Manual Markdown-only evidence | Do not make manual prose primary proof; Phase 9 decisions require enriched `flash-command-evidence.json`. [VERIFIED: 09-CONTEXT.md] |

**Installation:**

```bash
# No new Rust or npm packages are recommended for Phase 9.
cargo install espflash --locked
```

`espflash` installation is already satisfied locally at `4.0.1`; the install line is documentation for missing environments only. [VERIFIED: `espflash --version`] [CITED: https://raw.githubusercontent.com/esp-rs/espflash/main/espflash/README.md]

**Version verification:** Phase 9 should not add dependencies. Existing recommended versions were verified from `Cargo.toml`, `Cargo.lock`, and local CLI `--version` output instead of `npm view`, because this phase has no npm packages. [VERIFIED: Cargo.toml] [VERIFIED: Cargo.lock] [VERIFIED: local tool version commands]

## Architecture Patterns

### Recommended Project Structure

```text
tools/flash/
|-- src/main.rs       # Existing CLI, command construction, capture, evidence JSON, and tests.
|-- BUILD.bazel       # Existing rust_binary and rust_test target.
`-- Cargo.toml        # Existing Rust dependencies; no new dependency needed.

docs/parity/evidence/
`-- phase-09-.../     # Fresh wrapper-produced JSON/log and optional Markdown summary.

docs/parity/checklist.md      # Refresh WF-005 only; avoid release-row overclaim.
docs/release/ultra-205.md     # Document wrapper evidence flow and recovery guidance.
```

The implementation can stay in `tools/flash/src/main.rs`; the file is currently large enough that the planner should consider small named helpers, but a broad module split is not required for this narrow phase. [VERIFIED: tools/flash/src/main.rs] [VERIFIED: standards/core/code-shape.md]

### Pattern 1: Separate Interactive Monitor From Evidence Monitor

**What:** Keep `prepare_monitor_command` for normal `just monitor`, and add a focused evidence-mode builder used only when `flash-monitor` has `--evidence-dir`. [VERIFIED: tools/flash/src/main.rs] [VERIFIED: 09-CONTEXT.md]

**When to use:** Use the evidence command after a successful flash and before writing the final evidence record. [VERIFIED: tools/flash/src/main.rs]

**Example:**

```rust
fn prepare_evidence_monitor_command(
    common: &CommonArgs,
    environment: &impl FlashEnvironment,
) -> Result<CommandSpec> {
    ensure_ultra_205(common.board)?;
    let port = resolve_port(common.port.as_deref(), environment)?;
    Ok(CommandSpec::new(
        "espflash",
        ["monitor", "--chip", "esp32s3", "--port", port.as_str(), "--non-interactive"],
    ))
}
```

Source: existing command-builder pattern plus local `espflash monitor --help`. [VERIFIED: tools/flash/src/main.rs] [VERIFIED: local `espflash monitor --help`]

### Pattern 2: Bounded Capture Returns Structured Status

**What:** Change capture from `Result<()>` to a typed result such as `MonitorCaptureOutcome` with fields for mode, timeout seconds, process status, trusted-output status, and conclusion. [VERIFIED: tools/flash/src/main.rs] [VERIFIED: 09-CONTEXT.md]

**When to use:** Use for `flash-monitor --evidence-dir` only; ordinary `monitor` should still attach interactively and not time out. [VERIFIED: 09-CONTEXT.md]

**Example:**

```rust
#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
enum CaptureStatus {
    Completed,
    TimedOutAfterTrustedOutput,
    TimedOutWithoutTrustedOutput,
    Failed,
}
```

The process should fail closed on spawn failure, log creation failure, nonzero exit, or timeout without trusted boot markers. [VERIFIED: 09-CONTEXT.md]

### Pattern 3: Pure Trusted-Output Classification

**What:** Read the captured log after the bounded process stops and classify it with a pure helper before writing a passing evidence conclusion. [VERIFIED: standards/core/architecture.md] [VERIFIED: docs/parity/evidence/phase-08-ultra-205-release-gate/flash-monitor-noninteractive.log]

**Recommended minimum serial markers:** `bitaxe-rust boot: board=Ultra 205 asic=BM1366`, `safe_state:`, `ota_boot_validation=`, `spiffs_mount=available`, `axeos_api_route_shell=started`, `firmware_commit=`, and `reference_commit=`. These markers are present in the Phase 8 fallback log and support a serial-scope conclusion only. [VERIFIED: docs/parity/evidence/phase-08-ultra-205-release-gate/flash-monitor-noninteractive.log]

**When to use:** Use this helper before setting `capture_status` to a trusted passing state; do not infer trust from file existence alone. [VERIFIED: 09-CONTEXT.md]

### Pattern 4: Enriched Evidence Record

**What:** Replace the current single `command` string with separate fields for `flash_command` and `monitor_command`, while retaining a compatibility-friendly `command_kind`. Add `capture_mode`, `capture_status`, `capture_timeout_seconds`, `monitor_log_path`, and `conclusion`. [VERIFIED: tools/flash/src/main.rs] [VERIFIED: 09-CONTEXT.md]

**When to use:** Write after monitor capture has either succeeded with trusted output or failed with an explicit untrusted conclusion. The command should still return nonzero on untrusted evidence. [VERIFIED: 09-CONTEXT.md]

**Example JSON shape:**

```json
{
  "command_kind": "flash-monitor",
  "board": "205",
  "port": "/dev/cu.usbmodem1101",
  "firmware_commit": "source-commit",
  "reference_commit": "reference-commit",
  "manifest_path": "bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json",
  "flash_image_path": "bazel-bin/firmware/bitaxe/bitaxe-ultra205-factory.bin",
  "flash_command": "espflash write-bin --chip esp32s3 --port /dev/cu.usbmodem1101 0x0 ...",
  "monitor_command": "espflash monitor --chip esp32s3 --port /dev/cu.usbmodem1101 --non-interactive",
  "monitor_log_path": "docs/parity/evidence/phase-09.../flash-monitor.log",
  "capture_mode": "noninteractive",
  "capture_status": "timed_out_after_trusted_output",
  "capture_timeout_seconds": 25,
  "timestamp": "1782770000",
  "conclusion": "passed - wrapper-owned serial boot evidence captured; HTTP/OTA/recovery parity not claimed"
}
```

The JSON field names are recommended planning inputs; exact names are under agent discretion as long as D-06 is satisfied. [VERIFIED: 09-CONTEXT.md]

### Anti-Patterns To Avoid

- **Changing normal `just monitor` to noninteractive:** This would violate the decision to preserve manual/operator interactive behavior. [VERIFIED: 09-CONTEXT.md]
- **Accepting a partial log as proof:** File existence is not trusted evidence; classify required boot markers before a passing conclusion. [VERIFIED: 09-CONTEXT.md]
- **Relying on raw `espflash monitor` in docs as the fix:** Phase 9 exists to make the repo wrapper own the evidence path. [VERIFIED: docs/parity/evidence/phase-08-ultra-205-release-gate.md] [VERIFIED: 09-CONTEXT.md]
- **Using `Command::status` for monitor capture:** `espflash monitor` is a long-running command, so an unbounded status wait can hang the wrapper. [VERIFIED: tools/flash/src/main.rs] [VERIFIED: docs/parity/evidence/phase-08-ultra-205-release-gate.md]
- **Promoting release rows from serial logs:** Phase 9 serial evidence cannot verify live HTTP/static/recovery/OTA/rollback behavior. [VERIFIED: 09-CONTEXT.md] [VERIFIED: docs/parity/checklist.md]

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
| --- | --- | --- | --- |
| ESP serial monitor protocol | Custom serial reader, PTY adapter, or reset-key emulator | `espflash monitor --chip esp32s3 --port <port> --non-interactive` | esp-rs tooling is the repo-preferred backend and local `espflash` supports the needed noninteractive command. [VERIFIED: AGENTS.md] [VERIFIED: local `espflash monitor --help`] |
| Timeout process control | Shelling through `timeout` | Rust `Command::spawn`, `try_wait`, bounded sleep loop, and `kill` on timeout | This keeps the wrapper portable and testable without depending on GNU coreutils. [VERIFIED: docs/parity/evidence/phase-08-ultra-205-release-gate.md] |
| Evidence JSON | Manual string concatenation | `serde::Serialize` typed structs | Existing evidence and package parsing already use serde, and typed structs make required fields testable. [VERIFIED: tools/flash/src/main.rs] |
| Board detection gate | Ad hoc port probe in `tools/flash` | Existing `just detect-ultra205` before hardware evidence | Repo-local guidance already defines the hardware gate and required `board-info` command. [VERIFIED: AGENTS.md] [VERIFIED: scripts/detect-ultra205.sh] |
| Release parity proof | Manual prose or checklist edits alone | Wrapper-produced JSON/log plus narrow docs/checklist updates | Phase 9 decisions make JSON the source of truth and forbid overclaiming release parity. [VERIFIED: 09-CONTEXT.md] |

**Key insight:** Phase 9 should harden the wrapper around standard esp-rs tools, not create a new flashing or monitoring stack. [VERIFIED: AGENTS.md] [VERIFIED: 09-CONTEXT.md]

## Common Pitfalls

### Pitfall 1: Interactive Monitor In Evidence Mode

**What goes wrong:** The wrapper reaches monitor startup and fails with `Failed to initialize input reader` in a noninteractive execution environment. [VERIFIED: docs/parity/evidence/phase-08-ultra-205-release-gate.md]

**Why it happens:** Current `flash-monitor --evidence-dir` calls `prepare_monitor_command`, which returns `espflash monitor --port <port>` without `--non-interactive`. [VERIFIED: tools/flash/src/main.rs]

**How to avoid:** Use a separate evidence-mode command builder that includes `--chip esp32s3` and `--non-interactive`. [VERIFIED: 09-CONTEXT.md] [VERIFIED: local `espflash monitor --help`]

**Warning signs:** Evidence docs mention raw fallback `timeout 25 espflash monitor ... --non-interactive`, or the generated JSON contains `monitor: espflash monitor --port ...`. [VERIFIED: docs/parity/evidence/phase-08-ultra-205-release-gate.md] [VERIFIED: docs/parity/evidence/phase-07-ultra-205-ota-hardware-smoke.md]

### Pitfall 2: Unbounded Monitor Capture

**What goes wrong:** The wrapper hangs because serial monitor commands are intended to stay attached. [VERIFIED: docs/parity/evidence/phase-08-ultra-205-release-gate.md]

**Why it happens:** Existing `execute_capturing` waits on `Command::status()` and has no timeout or trusted-output cutoff. [VERIFIED: tools/flash/src/main.rs]

**How to avoid:** Spawn the monitor process, capture stdout/stderr to the log, poll with a timeout, kill on timeout, and classify the completed log. [VERIFIED: standards/core/architecture.md]

**Warning signs:** A test fake only verifies that `flash-monitor.log` exists and does not inspect capture status or trusted markers. [VERIFIED: tools/flash/src/main.rs]

### Pitfall 3: Treating Timeout As Automatic Failure

**What goes wrong:** Evidence mode fails even though it captured all required boot markers, because `espflash monitor` was killed by the bounded capture timeout. [VERIFIED: docs/parity/evidence/phase-08-ultra-205-release-gate.md]

**Why it happens:** The monitor is a continuous process; Phase 8 used timeout as the expected stop condition after log capture. [VERIFIED: docs/parity/evidence/phase-08-ultra-205-release-gate.md]

**How to avoid:** Represent `timed_out_after_trusted_output` separately from `timed_out_without_trusted_output`. [VERIFIED: 09-CONTEXT.md]

**Warning signs:** Evidence only records a process exit code and lacks a capture status field. [VERIFIED: tools/flash/src/main.rs]

### Pitfall 4: Evidence Written Outside The Requested Directory

**What goes wrong:** Bazel runfiles or process working-directory behavior causes logs to appear outside the repo evidence path. [VERIFIED: .planning/phases/08-parity-evidence-and-ultra-205-release-gate/08-02-SUMMARY.md]

**Why it happens:** Bazel-launched tools may run from execroot/output directories unless paths and workspace roots are handled explicitly. [VERIFIED: .planning/STATE.md] [VERIFIED: tools/flash/src/main.rs]

**How to avoid:** Resolve evidence paths exactly from the CLI argument, create parent directories before spawning, and write JSON/log paths as the requested paths. [VERIFIED: tools/flash/src/main.rs]

**Warning signs:** `flash-command-evidence.json` points to a log path that was not created in the requested `evidence-dir`. [VERIFIED: tools/flash/src/main.rs]

### Pitfall 5: Checklist Overclaim

**What goes wrong:** Serial wrapper evidence gets used to promote `FS-001`, `OTA-001`, `OTA-002`, `REL-001`, `REL-002`, or `REL-003`. [VERIFIED: 09-CONTEXT.md]

**Why it happens:** Phase 7 and Phase 8 serial logs include SPIFFS mount and route registration, but no reachable `DEVICE_URL` and no live HTTP/OTA/recovery requests. [VERIFIED: docs/parity/evidence/phase-07-ultra-205-ota-hardware-smoke.md] [VERIFIED: docs/parity/evidence/phase-08-ultra-205-release-gate.md]

**How to avoid:** Refresh `WF-005` and release guide text only; leave live HTTP/static/OTA/recovery rows below `verified`. [VERIFIED: 09-CONTEXT.md] [VERIFIED: docs/parity/checklist.md]

**Warning signs:** A docs diff changes release rows from `implemented` or `deferred` to `verified` without Phase 13 evidence. [VERIFIED: .planning/ROADMAP.md]

## Code Examples

Verified patterns from existing sources and local command help. [VERIFIED: tools/flash/src/main.rs] [VERIFIED: local `espflash monitor --help`]

### Evidence Command Construction

```rust
fn prepare_evidence_monitor_command(
    common: &CommonArgs,
    environment: &impl FlashEnvironment,
) -> Result<CommandSpec> {
    ensure_ultra_205(common.board)?;
    let port = resolve_port(common.port.as_deref(), environment)?;

    Ok(CommandSpec::new(
        "espflash",
        [
            "monitor",
            "--chip",
            "esp32s3",
            "--port",
            port.as_str(),
            "--non-interactive",
        ],
    ))
}
```

### Pure Trusted Marker Check

```rust
fn monitor_log_has_trusted_boot_markers(log: &str) -> bool {
    [
        "bitaxe-rust boot: board=Ultra 205 asic=BM1366",
        "safe_state:",
        "ota_boot_validation=",
        "spiffs_mount=available",
        "axeos_api_route_shell=started",
        "firmware_commit=",
        "reference_commit=",
    ]
    .iter()
    .all(|marker| log.contains(marker))
}
```

The marker list is derived from the Phase 8 noninteractive serial log and should support only the serial-scope Phase 9 conclusion. [VERIFIED: docs/parity/evidence/phase-08-ultra-205-release-gate/flash-monitor-noninteractive.log]

### Capture Outcome Contract

```rust
#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
struct MonitorCaptureOutcome {
    mode: String,
    status: CaptureStatus,
    timeout_seconds: u64,
    trusted_output: bool,
    conclusion: String,
}
```

This keeps illegal states easier to review because `status`, `trusted_output`, and `conclusion` must be written together. [VERIFIED: standards/core/architecture.md] [VERIFIED: standards/languages/rust.md]

## State Of The Art

| Old Approach | Current Approach | When Changed | Impact |
| --- | --- | --- | --- |
| Manual/PTY interactive monitor with `CTRL+R` and `CTRL+C` for clean boot logs. | Wrapper-owned noninteractive evidence mode should invoke `espflash monitor --chip esp32s3 --port <port> --non-interactive`. | Phase 8 documented the wrapper startup failure and raw fallback gap on 2026-06-29. [VERIFIED: docs/parity/evidence/phase-08-ultra-205-release-gate.md] | Plans should remove human input and PTY dependencies from trusted evidence capture. [VERIFIED: 09-CONTEXT.md] |
| Raw shell fallback `timeout 25 espflash monitor ... --non-interactive`. | Rust wrapper should own timeout, log capture, JSON status, and recovery guidance. | Phase 9 was added to close this gap after Phase 8. [VERIFIED: .planning/ROADMAP.md] | Evidence becomes repeatable through `just flash-monitor ... evidence-dir=...`. [VERIFIED: 09-CONTEXT.md] |
| Evidence JSON records one combined command string and `log_path`. | Evidence JSON should record flash command, monitor command, monitor log path, capture mode/status, and conclusion. | Phase 9 evidence contract D-05 through D-08. [VERIFIED: 09-CONTEXT.md] | Reviewers no longer infer evidence trust from prose or file existence. [VERIFIED: 09-CONTEXT.md] |
| Serial boot evidence risked being used to imply release parity. | Phase 9 docs should explicitly preserve HTTP/OTA/recovery/rollback blockers. | Phase 8 and Phase 9 roadmap boundaries. [VERIFIED: .planning/ROADMAP.md] [VERIFIED: docs/parity/checklist.md] | Avoids false verification of later release rows. [VERIFIED: 09-CONTEXT.md] |

**Deprecated/outdated:**

- Treating `espflash monitor --port <port>` as sufficient for noninteractive evidence is outdated for this repo because Phase 8 reproduced an input-reader failure. [VERIFIED: docs/parity/evidence/phase-08-ultra-205-release-gate.md]
- Treating `timeout 25 espflash monitor ...` as accepted release evidence is outdated after Phase 9 because the wrapper must own capture and evidence metadata. [VERIFIED: 09-CONTEXT.md]

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
| --- | --- | --- | --- |
| A1 | A 25 second default capture window is enough for Phase 9 serial-scope boot evidence because the Phase 8 fallback captured all required boot markers within that bounded command. [ASSUMED] | Architecture Patterns, Common Pitfalls | If slow boots exceed 25 seconds, evidence mode could fail closed until the planner makes the timeout configurable or chooses a longer default. |

The planner can eliminate A1 by making the timeout configurable, documenting the default, and requiring tests for both timeout-success and timeout-failure classifications. [VERIFIED: 09-CONTEXT.md]

## Open Questions

1. **Should the generated Markdown summary be added in Phase 9?**
   - What we know: The JSON record is required and is the source of truth. [VERIFIED: 09-CONTEXT.md]
   - What's unclear: A Markdown summary is optional and only useful if rendered from the same structured record. [VERIFIED: 09-CONTEXT.md]
   - Recommendation: Start with JSON plus a concise hand-maintained evidence doc that cites the JSON/log; add generated Markdown only if it is a small helper around the same struct. [VERIFIED: 09-CONTEXT.md]

2. **How strict should trusted marker classification be?**
   - What we know: Phase 8 serial logs include boot identity, safe state, boot validation, SPIFFS mount, route registration, firmware commit, and reference commit. [VERIFIED: docs/parity/evidence/phase-08-ultra-205-release-gate/flash-monitor-noninteractive.log]
   - What's unclear: Future firmware log wording may change before Phase 13. [ASSUMED]
   - Recommendation: Use a minimal serial-scope marker set and keep failures explicit so log wording drift is visible in tests/evidence. [VERIFIED: standards/core/testing.md]

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
| --- | --- | --- | --- | --- |
| `espflash` | Flash, board-info, noninteractive monitor capture | yes | `4.0.1` | Blocking if missing; install with `cargo install espflash --locked`. [VERIFIED: `espflash --version`] [CITED: https://raw.githubusercontent.com/esp-rs/espflash/main/espflash/README.md] |
| `bazel` | `just flash-monitor` backend and test target | yes | `9.1.1` | Blocking if missing for repo command surface. [VERIFIED: `bazel --version`] |
| `just` | Human command surface and hardware detector | yes | `1.48.0` | Use direct Bazel/script commands only for diagnostics; plans should keep `just` as the user entrypoint. [VERIFIED: `just --version`] [VERIFIED: Justfile] |
| `cargo` | Rust verification and optional espflash install | yes | `1.88.0-nightly` | Blocking for full Rust pre-commit verification. [VERIFIED: `cargo --version`] |
| `rustc` | Rust build/test implementation | yes | `1.88.0-nightly` | Blocking for Rust checks. [VERIFIED: `rustc --version`] |
| `git` | Source/reference commit metadata | yes | `2.53.0` | Evidence records fall back to `Unavailable` today, but trusted hardware evidence should record real commits. [VERIFIED: `git --version`] [VERIFIED: tools/flash/src/main.rs] |
| GNU `timeout` | Phase 8 raw fallback only | yes | `9.10` | Do not use in Phase 9 implementation; use Rust-owned process timeout. [VERIFIED: `timeout --version`] |
| Ultra 205 USB hardware | Fresh wrapper hardware evidence after implementation | yes, currently detected | ESP32-S3 rev v0.2, 16MB flash, `port=/dev/cu.usbmodem1101` | If detection later fails, record hardware evidence pending and do not flash. [VERIFIED: `just detect-ultra205`] |

**Missing dependencies with no fallback:**

- None found during research. [VERIFIED: local environment probes]

**Missing dependencies with fallback:**

- None found during research. [VERIFIED: local environment probes]

## Validation Architecture

### Test Framework

| Property | Value |
| --- | --- |
| Framework | Bazel `rust_test` target for `tools/flash`; Rust unit tests live in `tools/flash/src/main.rs`. [VERIFIED: tools/flash/BUILD.bazel] [VERIFIED: tools/flash/src/main.rs] |
| Config file | `tools/flash/BUILD.bazel`; package deps in `tools/flash/Cargo.toml`. [VERIFIED: tools/flash/BUILD.bazel] [VERIFIED: tools/flash/Cargo.toml] |
| Quick run command | `bazel test //tools/flash:tests` [VERIFIED: command passed during research] |
| Full suite command | `just test`; before commit in this Rust repo also run `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, and `cargo test --all-features`. [VERIFIED: Justfile] [VERIFIED: AGENTS.md] |

### Phase Requirements To Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
| --- | --- | --- | --- | --- |
| FND-07 | `just flash-monitor` still routes through `bazel run //tools/flash:flash -- flash-monitor` and normal command surface stays intact. [VERIFIED: Justfile] | unit/smoke | `bazel test //tools/flash:tests` | yes, existing `tools/flash/src/main.rs` tests. [VERIFIED: tools/flash/src/main.rs] |
| FND-08 | Evidence-mode monitor command includes `monitor --chip esp32s3 --port <port> --non-interactive`, while ordinary monitor remains interactive. [VERIFIED: local `espflash monitor --help`] | unit | `bazel test //tools/flash:tests` | add tests in existing file. [VERIFIED: tools/flash/src/main.rs] |
| REL-07 | Failure guidance tells operators evidence is untrusted and points to `just detect-ultra205`, wrapper `just flash-monitor`, and diagnostic `just monitor`, not raw fallback. [VERIFIED: 09-CONTEXT.md] | unit/docs review | `bazel test //tools/flash:tests` plus docs diff review | add tests in existing file and update docs. [VERIFIED: tools/flash/src/main.rs] [VERIFIED: docs/release/ultra-205.md] |
| EVD-05 | Evidence JSON includes required metadata and capture status; untrusted capture returns nonzero and does not present partial logs as proof. [VERIFIED: 09-CONTEXT.md] | unit + hardware smoke | `bazel test //tools/flash:tests`; after implementation `just detect-ultra205` then `just flash-monitor board=205 port=<port> evidence-dir=<path>` | add tests in existing file and capture hardware evidence after implementation. [VERIFIED: scripts/detect-ultra205.sh] |

### Sampling Rate

- **Per task commit:** `bazel test //tools/flash:tests` for code changes, plus docs diff review for Markdown evidence changes. [VERIFIED: tools/flash/BUILD.bazel] [VERIFIED: standards/core/verification.md]
- **Per wave merge:** `just test` when the phase changes docs and Rust wrapper behavior. [VERIFIED: Justfile]
- **Phase gate:** Full suite and hardware evidence if `just detect-ultra205` passes exactly one port; otherwise record hardware evidence pending. [VERIFIED: AGENTS.md] [VERIFIED: scripts/detect-ultra205.sh]

### Wave 0 Gaps

- [ ] Add unit tests in `tools/flash/src/main.rs` for evidence monitor command construction. [VERIFIED: tools/flash/src/main.rs]
- [ ] Add unit tests in `tools/flash/src/main.rs` for bounded capture statuses: success, timed out with trusted output, timed out without trusted output, nonzero exit, and log creation failure. [VERIFIED: 09-CONTEXT.md]
- [ ] Add unit tests in `tools/flash/src/main.rs` for enriched `flash-command-evidence.json` fields. [VERIFIED: 09-CONTEXT.md]
- [ ] Add failure-guidance assertion tests so monitor startup failures state evidence is not trusted and include repo-owned recovery commands. [VERIFIED: 09-CONTEXT.md]

Existing test infrastructure covers the target; no new test framework is needed. [VERIFIED: `bazel test //tools/flash:tests`]

## Security Domain

Security enforcement is enabled because `.planning/config.json` does not set `security_enforcement` to `false`. [VERIFIED: .planning/config.json]

OWASP ASVS is the referenced application-security verification standard; the latest stable version is 5.0.0 according to the OWASP ASVS project page. [CITED: https://github.com/OWASP/ASVS] [CITED: https://owasp.org/www-project-application-security-verification-standard/]

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
| --- | --- | --- |
| V2 Authentication | no | No authentication surface is added by a local CLI evidence wrapper. [VERIFIED: tools/flash/src/main.rs] |
| V3 Session Management | no | No web session or token lifecycle is touched by Phase 9. [VERIFIED: 09-CONTEXT.md] |
| V4 Access Control | no | Local hardware access is gated procedurally by `just detect-ultra205`; no app authorization model changes. [VERIFIED: AGENTS.md] [VERIFIED: scripts/detect-ultra205.sh] |
| V5 Input Validation | yes | Keep typed `board=205` parsing, UTF-8 evidence paths, explicit port resolution, command argument vectors, and trusted log marker checks. [VERIFIED: tools/flash/src/main.rs] |
| V6 Cryptography | no | No cryptographic primitive or key management changes are in scope. [VERIFIED: 09-CONTEXT.md] |

### Known Threat Patterns For This Stack

| Pattern | STRIDE | Standard Mitigation |
| --- | --- | --- |
| Command injection through `port`, `image`, or `evidence-dir` | Tampering | Continue using `std::process::Command` with argument vectors; do not build shell command strings for execution. [VERIFIED: tools/flash/src/main.rs] |
| Evidence spoofing through partial logs | Tampering | Require structured capture status plus trusted boot marker classification before a passing conclusion. [VERIFIED: 09-CONTEXT.md] |
| Secret leakage in evidence logs/docs | Information Disclosure | Redact or avoid Wi-Fi credentials, pool credentials, private endpoints, and NVS secret values; Phase 9 serial evidence should not add network probes. [VERIFIED: AGENTS.md] [VERIFIED: 09-CONTEXT.md] |
| Indefinite monitor hang | Denial of Service | Use Rust-owned bounded capture and fail closed on timeout without trusted output. [VERIFIED: 09-CONTEXT.md] |
| Wrong-device evidence | Spoofing | Run `just detect-ultra205` first and record `espflash board-info --chip esp32s3 --port <port> --non-interactive` output. [VERIFIED: AGENTS.md] [VERIFIED: scripts/detect-ultra205.sh] |
| Release overclaim | Elevation of Privilege in governance | Update only serial workflow evidence; leave HTTP/OTA/recovery/rollback rows below verified. [VERIFIED: 09-CONTEXT.md] [VERIFIED: docs/parity/checklist.md] |

## Sources

### Primary (HIGH confidence)

- `.planning/phases/09-flash-monitor-evidence-wrapper-hardening/09-CONTEXT.md` - locked decisions, evidence contract, recovery guidance, hardware boundary. [VERIFIED: 09-CONTEXT.md]
- `.planning/ROADMAP.md` - Phase 9 gap closure, success criteria, verification expectations, dependencies. [VERIFIED: .planning/ROADMAP.md]
- `.planning/REQUIREMENTS.md` - FND-07, FND-08, REL-07, EVD-05. [VERIFIED: .planning/REQUIREMENTS.md]
- `AGENTS.md`, `AGENTS.bright-builds.md`, `standards/*` - repo constraints, Rust standards, verification and testing rules. [VERIFIED: AGENTS.md] [VERIFIED: AGENTS.bright-builds.md]
- `tools/flash/src/main.rs`, `tools/flash/BUILD.bazel`, `tools/flash/Cargo.toml`, `Justfile` - current implementation and test surface. [VERIFIED: tools/flash/src/main.rs] [VERIFIED: tools/flash/BUILD.bazel]
- `scripts/detect-ultra205.sh` - hardware detection gate. [VERIFIED: scripts/detect-ultra205.sh]
- `docs/parity/evidence/phase-08-ultra-205-release-gate.md` and `flash-monitor-noninteractive.log` - raw fallback gap and trusted serial markers. [VERIFIED: docs/parity/evidence/phase-08-ultra-205-release-gate.md]
- `docs/parity/checklist.md` and `docs/release/ultra-205.md` - docs/checklist boundaries. [VERIFIED: docs/parity/checklist.md]
- Local CLI probes: `espflash --version`, `espflash monitor --help`, `just detect-ultra205`, `bazel test //tools/flash:tests`. [VERIFIED: local commands]

### Secondary (MEDIUM confidence)

- `https://raw.githubusercontent.com/esp-rs/espflash/main/espflash/README.md` - espflash support, installation, configuration, and command overview. [CITED: esp-rs espflash README]
- `https://github.com/esp-rs/espflash/releases/tag/v4.0.0` and `v4.0.1` - release-note context for noninteractive/no-reset related CLI capability and local version. [CITED: esp-rs GitHub releases]
- `https://owasp.org/www-project-application-security-verification-standard/` and `https://github.com/OWASP/ASVS` - ASVS project and latest stable version context. [CITED: OWASP ASVS]

### Tertiary (LOW confidence)

- None. [VERIFIED: research source log]

## Metadata

**Confidence breakdown:**

- Standard stack: HIGH - no new stack is needed; versions and command capabilities were verified from local files and local CLI help. [VERIFIED: Cargo.toml] [VERIFIED: local `espflash monitor --help`]
- Architecture: HIGH - the current code already has the correct command-spec/evidence abstractions and fake environment tests; the phase is a narrow extension. [VERIFIED: tools/flash/src/main.rs]
- Pitfalls: HIGH - the triggering failure and fallback command are recorded in Phase 8 evidence, and local help confirms the noninteractive command support. [VERIFIED: docs/parity/evidence/phase-08-ultra-205-release-gate.md] [VERIFIED: local `espflash monitor --help`]
- Hardware availability: HIGH for current research moment - `just detect-ultra205` passed with exactly one ESP32-S3 candidate on `/dev/cu.usbmodem1101`; re-run immediately before implementation hardware evidence. [VERIFIED: `just detect-ultra205`]

**Research date:** 2026-06-29
**Valid until:** 2026-07-06 for espflash/hardware availability; 2026-07-29 for repo-local architecture guidance. [ASSUMED]

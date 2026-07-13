---
phase: 01-foundation-and-gamma-601-boot-log
reviewed: "2026-06-21T04:21:34Z"
depth: standard
files_reviewed: 51
files_reviewed_list:
  - .bazelversion
  - .cargo/config.toml
  - .gitignore
  - .gitmodules
  - BUILD.bazel
  - Cargo.lock
  - Cargo.toml
  - Justfile
  - MODULE.bazel
  - MODULE.bazel.lock
  - crates/bitaxe-api/BUILD.bazel
  - crates/bitaxe-api/Cargo.toml
  - crates/bitaxe-api/src/lib.rs
  - crates/bitaxe-asic/BUILD.bazel
  - crates/bitaxe-asic/Cargo.toml
  - crates/bitaxe-asic/src/lib.rs
  - crates/bitaxe-config/BUILD.bazel
  - crates/bitaxe-config/Cargo.toml
  - crates/bitaxe-config/src/lib.rs
  - crates/bitaxe-core/BUILD.bazel
  - crates/bitaxe-core/Cargo.toml
  - crates/bitaxe-core/src/lib.rs
  - crates/bitaxe-stratum/BUILD.bazel
  - crates/bitaxe-stratum/Cargo.toml
  - crates/bitaxe-stratum/src/lib.rs
  - crates/bitaxe-test-support/BUILD.bazel
  - crates/bitaxe-test-support/Cargo.toml
  - crates/bitaxe-test-support/src/lib.rs
  - docs/parity/checklist.md
  - docs/parity/evidence/phase-01-gamma-601-boot-log.md
  - firmware/bitaxe/BUILD.bazel
  - firmware/bitaxe/Cargo.toml
  - firmware/bitaxe/build.rs
  - firmware/bitaxe/sdkconfig.defaults
  - firmware/bitaxe/src/main.rs
  - reference/esp-miner
  - rust-toolchain.toml
  - scripts/BUILD.bazel
  - scripts/build-firmware.sh
  - scripts/package-firmware.sh
  - scripts/verify-reference-clean-test.sh
  - scripts/verify-reference-clean.sh
  - tools/flash/BUILD.bazel
  - tools/flash/Cargo.toml
  - tools/flash/src/main.rs
  - tools/parity/BUILD.bazel
  - tools/parity/Cargo.toml
  - tools/parity/src/main.rs
  - tools/xtask/BUILD.bazel
  - tools/xtask/Cargo.toml
  - tools/xtask/src/main.rs
findings:
  critical: 0
  warning: 4
  info: 0
  total: 4
status: issues_found
---

# Phase 01: Code Review Report

**Reviewed:** 2026-06-21T04:21:34Z
**Depth:** standard
**Files Reviewed:** 51
**Status:** issues_found

## Summary

Reviewed the Phase 01 foundation, firmware boot/log, Bazel/Cargo wiring, flash workflow, parity tooling, and evidence documents. The small phase-contract crates are straightforward and unit-tested. No hardcoded secrets, shell-eval injection patterns, empty catches, or debug artifacts were found in the reviewed source.

Material guidance applied: `AGENTS.md`, `AGENTS.bright-builds.md`, `standards-overrides.md`, `standards/core/architecture.md`, `standards/core/code-shape.md`, `standards/core/verification.md`, `standards/core/testing.md`, and `standards/languages/rust.md`.

Verification run during review: `cargo test --workspace --exclude bitaxe-firmware` passed for host-side crates and tools.

## Warnings

### WR-01: Firmware commit log can become stale across Git-only changes

**File:** `firmware/bitaxe/build.rs:3`

**Issue:** The build script reads `git rev-parse --short=12 HEAD` and emits `BITAXE_FIRMWARE_COMMIT`, but it does not emit Cargo rerun hints for Git metadata. Cargo can reuse the previous build-script output when only `.git/HEAD` or the current ref changes, so firmware logs can report an old commit. That weakens the Phase 01 provenance evidence in `docs/parity/evidence/phase-01-gamma-601-boot-log.md`.

**Fix:**

```rust
fn main() {
    embuild::espidf::sysenv::output();
    emit_git_rerun_hints();

    let Some(commit) = maybe_git_commit() else {
        return;
    };

    println!("cargo:rustc-env=BITAXE_FIRMWARE_COMMIT={commit}");
}

fn emit_git_rerun_hints() {
    if let Some(head_path) = git_path("HEAD") {
        println!("cargo:rerun-if-changed={head_path}");
    }

    if let Some(ref_name) = git_stdout(["symbolic-ref", "-q", "HEAD"]) {
        if let Some(ref_path) = git_path(ref_name.trim()) {
            println!("cargo:rerun-if-changed={ref_path}");
        }
    }
}
```

Add a build-script focused regression check or package-level verification that proves a changed Git ref refreshes the emitted firmware commit.

### WR-02: `flash-monitor` evidence points to a log file that is never created

**File:** `tools/flash/src/main.rs:361`

**Issue:** `run_flash_monitor` calls `run_flash` and then `run_monitor`, but only `run_flash` writes evidence. `write_evidence_if_requested` always records `flash-monitor.log` at lines 545-563, while `run_monitor` streams `espflash monitor` without capturing or writing that log. A user running the documented `just flash-monitor ... evidence-dir=...` path will get command metadata that points at a missing log file, so the evidence bundle cannot prove the required boot log lines.

**Fix:**

```rust
fn run_flash_monitor(
    command: &FlashMonitorCommand,
    environment: &impl FlashEnvironment,
) -> Result<()> {
    let flash_outcome = run_flash_without_evidence(command, environment)?;
    let monitor_command = prepare_monitor_command(&command.common, environment)?;

    if let Some(evidence_dir) = &command.common.evidence_dir {
        let log_path = evidence_dir.join("flash-monitor.log");
        environment.execute_capturing(&monitor_command, &log_path)?;
        write_flash_monitor_evidence(&command.common, &flash_outcome, &log_path, environment)?;
        return Ok(());
    }

    environment.execute(&monitor_command)
}
```

Keep the fake environment testable by adding a capture method to the trait, then add a unit test that `flash-monitor` writes evidence with `command_kind=flash-monitor` and creates the referenced log.

### WR-03: Safety-critical parity validation misses rows marked only as safety-critical

**File:** `tools/parity/src/main.rs:357`

**Issue:** The checklist states that safety-critical and hardware-control surfaces require hardware evidence before `verified`, and rows such as `PWR-001` are explicitly marked "Safety-critical; requires hardware evidence." The validator only searches for a short term list plus ASIC initialization. If `PWR-001 | ASIC reset behavior` is changed to `verified | unit`, `is_safety_critical` does not match "safety-critical" and the invalid claim passes.

**Fix:**

```rust
fn is_safety_critical(row: &ChecklistRow) -> bool {
    let haystack = format!(
        "{} {} {} {}",
        row.id, row.surface, row.rust_owned_target, row.notes
    )
    .to_ascii_lowercase();

    haystack.contains("safety-critical")
        || row.id.starts_with("PWR-")
        || row.id.starts_with("THR-")
        || ["voltage", "fan", "thermal", "power"]
            .iter()
            .any(|term| haystack.contains(term))
        || haystack.contains("asic initialization")
        || (row.id.starts_with("ASIC") && haystack.contains("initialization"))
}
```

Add a regression test for `PWR-001 | ASIC reset behavior | verified | unit | Safety-critical; requires hardware evidence.` so the validator blocks that false parity claim.

### WR-04: Bare `COM` is accepted as a likely serial port

**File:** `tools/flash/src/main.rs:499`

**Issue:** The Windows port matcher accepts any token that starts with `COM` and whose suffix is all digits. For the bare token `COM`, the suffix is empty and `.all(...)` returns true. If `espflash list-ports` output contains `COM` as a header or label token, auto-detection can select an invalid port instead of failing with the actionable "No serial ports found" or "Ambiguous serial ports" message.

**Fix:**

```rust
fn is_likely_port(port: &str) -> bool {
    if port.starts_with("/dev/cu.usbmodem")
        || port.starts_with("/dev/cu.usbserial")
        || port.starts_with("/dev/ttyUSB")
        || port.starts_with("/dev/ttyACM")
    {
        return true;
    }

    let Some(suffix) = port.strip_prefix("COM") else {
        return false;
    };

    !suffix.is_empty() && suffix.chars().all(|character| character.is_ascii_digit())
}
```

Add unit coverage for `COM` rejecting and `COM3` accepting.

---

_Reviewed: 2026-06-21T04:21:34Z_
_Reviewer: the agent (gsd-code-reviewer)_
_Depth: standard_

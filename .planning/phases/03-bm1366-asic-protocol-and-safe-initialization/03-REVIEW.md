---
phase: 03-bm1366-asic-protocol-and-safe-initialization
reviewed: 2026-06-27T01:35:51Z
depth: standard
files_reviewed: 32
files_reviewed_list:
  - Cargo.lock
  - MODULE.bazel.lock
  - crates/bitaxe-asic/BUILD.bazel
  - crates/bitaxe-asic/Cargo.toml
  - crates/bitaxe-asic/fixtures/bm1366/init-plan-cases.json
  - crates/bitaxe-asic/fixtures/bm1366/protocol-cases.json
  - crates/bitaxe-asic/fixtures/bm1366/transcript-cases.json
  - crates/bitaxe-asic/fixtures/bm1366/work-result-cases.json
  - crates/bitaxe-asic/src/bm1366.rs
  - crates/bitaxe-asic/src/bm1366/adapter_gate.rs
  - crates/bitaxe-asic/src/bm1366/command.rs
  - crates/bitaxe-asic/src/bm1366/crc.rs
  - crates/bitaxe-asic/src/bm1366/frequency_voltage.rs
  - crates/bitaxe-asic/src/bm1366/init_plan.rs
  - crates/bitaxe-asic/src/bm1366/observation.rs
  - crates/bitaxe-asic/src/bm1366/packet.rs
  - crates/bitaxe-asic/src/bm1366/registers.rs
  - crates/bitaxe-asic/src/bm1366/result.rs
  - crates/bitaxe-asic/src/bm1366/transcript.rs
  - crates/bitaxe-asic/src/bm1366/work.rs
  - crates/bitaxe-asic/src/dispatch.rs
  - crates/bitaxe-asic/src/error.rs
  - crates/bitaxe-asic/src/lib.rs
  - docs/parity/checklist.md
  - docs/parity/evidence/phase-03-ultra-205-bm1366-chip-detect.md
  - firmware/bitaxe/Cargo.toml
  - firmware/bitaxe/sdkconfig.defaults
  - firmware/bitaxe/src/asic_adapter.rs
  - firmware/bitaxe/src/asic_adapter/reset.rs
  - firmware/bitaxe/src/asic_adapter/status.rs
  - firmware/bitaxe/src/asic_adapter/uart.rs
  - firmware/bitaxe/src/main.rs
findings:
  critical: 0
  warning: 2
  info: 1
  total: 3
status: issues_found
---

# Phase 03: Code Review Report

**Reviewed:** 2026-06-27T01:35:51Z
**Depth:** standard
**Files Reviewed:** 32
**Status:** issues_found

## Summary

Reviewed the Phase 03 BM1366 protocol crate, firmware ASIC adapter, fixtures, parity checklist/evidence, and build metadata against the repo instructions, Bright Builds rules, and Phase 03 context/validation artifacts. The pure protocol boundary is mostly kept in `bitaxe-asic`, firmware does not construct raw BM1366 frames directly, and the evidence documents avoid marking live hardware behavior as verified.

I found two warning-level fail-closed/status issues in the live diagnostic chip-detect adapter path and one info-level stale placeholder that can confuse the Phase 03 API surface.

## Warnings

### WR-01: Chip-detect success can be published without validating the chip-id response

**Files:**
- `crates/bitaxe-asic/src/bm1366/init_plan.rs:51`
- `firmware/bitaxe/src/asic_adapter.rs:79`
- `firmware/bitaxe/src/asic_adapter/status.rs:23`

**Issue:** `Bm1366InitPlan::chip_detect_only` queues `ReadExact` and then unconditionally queues `PublishStatus(ChipDetectedNoMining { chips: preflight.expected_chips() })`. The firmware adapter performs the exact-length UART read at `firmware/bitaxe/src/asic_adapter.rs:79` but discards the returned bytes, so any exact-length 11-byte response, including a bad preamble, bad CRC, wrong chip id, or wrong chip count, can still lead to `asic_status=chip_detected` at `status.rs:23`. The host transcript parser validates these cases, but that parser is not part of the live adapter path. This violates the Phase 03 fail-closed requirement and can create false chip-detect evidence.

**Fix:** Make chip-detect a semantic adapter operation instead of `ReadExact` followed by unconditional success. Reuse or expose the existing chip-id response validation from `bitaxe-asic`, and publish `ChipDetectedNoMining` only after all expected responses parse and count correctly. On any parse fault, publish `FailClosed` and keep/reset the ASIC in a safe state.

```rust
let response = uart.read_exact(*len, *timeout_ms)?;
let observation = bm1366::parse_chip_id_response(&response)?;
chip_detect.record(observation)?;

if chip_detect.observed_chips() == preflight.expected_chips() {
    status::publish_status(AsicInitStatus::ChipDetectedNoMining {
        chips: chip_detect.observed_chips(),
    });
} else {
    reset.hold_reset_low()?;
    status::publish_status(AsicInitStatus::FailClosed {
        reason: "chip_detect_response_invalid",
    });
}
```

Add a firmware-adapter or host seam test proving an exact-length response with an invalid preamble or CRC does not publish `chip_detected`.

### WR-02: Adapter I/O failures escape without visible fail-closed status after reset release

**Files:**
- `firmware/bitaxe/src/asic_adapter.rs:60`
- `firmware/bitaxe/src/asic_adapter/uart.rs:76`
- `firmware/bitaxe/src/asic_adapter/reset.rs:29`

**Issue:** `run_chip_detect_only` executes each action with `interpret_action(...)?`, so UART write/read failures or partial reads return immediately. `Bm1366Uart::read_exact` clears RX and bails on a partial read at `uart.rs:76`, but the adapter does not then publish a fail-closed status or attempt to hold reset low. Because `PulseReset` releases reset high at `reset.rs:29` before the read path, an adapter error after reset release can exit through `main.rs` without the Phase 03-required visible fail-closed reason and safe reset posture.

**Fix:** Convert action-loop errors into an explicit fail-closed transition before returning. Best effort reset-hold and status publication should happen for every adapter I/O error after peripherals are acquired.

```rust
for action in decision.actions() {
    if let Err(error) = interpret_action(action, &mut reset, &mut uart) {
        let _ = reset.hold_reset_low();
        status::publish_status(AsicInitStatus::FailClosed {
            reason: "chip_detect_adapter_error",
        });
        log::warn!("asic_status=fail_closed reason=chip_detect_adapter_error error={error:#}");
        return Ok(());
    }
}
```

Add a test seam for a partial read or UART write failure that verifies the adapter publishes `fail_closed` and attempts to hold reset low.

## Info

### IN-01: Phase 3 API still exposes a stale deferred runtime status

**File:** `crates/bitaxe-asic/src/lib.rs:8`

**Issue:** `AsicRuntimeStatus::DeferredUntilPhase3` and its test still describe active ASIC behavior as deferred until Phase 3, even though this phase now introduces BM1366 protocol and diagnostic adapter gates. This is not a runtime bug, but it leaves a stale public placeholder in the crate surface and can confuse later callers or evidence readers.

**Fix:** Remove the placeholder if no caller needs it, or replace it with a current fail-closed/gated status name that matches Phase 03 behavior, such as a status that explicitly points callers to `bm1366::adapter_gate` or `AsicInitStatus`.

## Verification

- `cargo test -p bitaxe-asic --all-features` passed: 44 unit tests, 0 failures; doc tests passed.

---

_Reviewed: 2026-06-27T01:35:51Z_
_Reviewer: the agent (gsd-code-reviewer)_
_Depth: standard_

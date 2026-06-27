---
phase: 03-bm1366-asic-protocol-and-safe-initialization
reviewed: 2026-06-27T01:51:44Z
depth: standard
files_reviewed: 12
files_reviewed_list:
  - crates/bitaxe-asic/BUILD.bazel
  - crates/bitaxe-asic/src/lib.rs
  - crates/bitaxe-asic/src/bm1366.rs
  - crates/bitaxe-asic/src/bm1366/chip_detect.rs
  - crates/bitaxe-asic/src/bm1366/command.rs
  - crates/bitaxe-asic/src/bm1366/init_plan.rs
  - crates/bitaxe-asic/src/bm1366/transcript.rs
  - firmware/bitaxe/src/asic_adapter.rs
  - firmware/bitaxe/src/asic_adapter/reset.rs
  - firmware/bitaxe/src/asic_adapter/status.rs
  - firmware/bitaxe/src/asic_adapter/uart.rs
  - firmware/bitaxe/src/main.rs
findings:
  critical: 0
  warning: 1
  info: 1
  total: 2
status: issues_found
---

# Phase 03: Code Review Report

**Reviewed:** 2026-06-27T01:51:44Z
**Depth:** standard
**Files Reviewed:** 12
**Status:** issues_found

## Summary

Re-reviewed the Phase 03 warning fixes from `d11667f` and `5e5a0ed` against `AGENTS.md`, `AGENTS.bright-builds.md`, `standards-overrides.md`, the Rust/core Bright Builds standards, the Phase 03 context/validation artifacts, and the prior `03-REVIEW.md` / `03-REVIEW-FIX.md`.

Previous WR-01 is resolved: `Bm1366InitPlan::chip_detect_only` no longer publishes `ChipDetectedNoMining` unconditionally, and the live firmware path validates chip-id response length, preamble, CRC, chip id, and expected chip count before publishing success.

Previous WR-02 is resolved: action-loop errors now transition through `chip_detect_adapter_error`, attempt reset-low, publish `asic_status=fail_closed`, and return without escaping through `main`.

No regressions were found in those fixed paths. One separate fail-closed setup edge remains in the firmware adapter before the action-loop handler is available, and the stale Phase 3 runtime placeholder remains.

## Warnings

### WR-01: Adapter setup failures can bypass fail-closed reset/status before the action loop

**File:** `firmware/bitaxe/src/asic_adapter.rs:55`
**Issue:** `AsicUart::new(...)?` and `AsicReset::new(...)?` still propagate errors before the action-loop error handler at `firmware/bitaxe/src/asic_adapter.rs:64` exists. A UART setup failure exits without acquiring reset or publishing `asic_status=fail_closed`, and a reset setup failure exits without a visible fail-closed status. That leaves a Phase 03 adapter stage error outside the fail-closed/status contract.
**Fix:** Initialize reset first, convert adapter setup errors into explicit fail-closed statuses, and hold reset low on UART setup failure once reset is available.

```rust
let mut reset = match reset::AsicReset::new(peripherals.pins.gpio1) {
    Ok(reset) => reset,
    Err(error) => {
        log::warn!("asic_status=fail_closed reason=reset_adapter_unavailable error={error}");
        status::publish_status(AsicInitStatus::FailClosed {
            reason: "reset_adapter_unavailable",
        });
        return Ok(());
    }
};

let mut uart = match uart::AsicUart::new(
    peripherals.uart1,
    peripherals.pins.gpio17,
    peripherals.pins.gpio18,
) {
    Ok(uart) => uart,
    Err(error) => {
        best_effort_hold_reset_low(&mut reset, "uart_adapter_unavailable");
        log::warn!("asic_status=fail_closed reason=uart_adapter_unavailable error={error}");
        status::publish_status(AsicInitStatus::FailClosed {
            reason: "uart_adapter_unavailable",
        });
        return Ok(());
    }
};
```

## Info

### IN-01: Phase 3 API still exposes a stale deferred runtime status

**File:** `crates/bitaxe-asic/src/lib.rs:8`
**Issue:** `AsicRuntimeStatus::DeferredUntilPhase3` and its test still describe active ASIC behavior as deferred until Phase 3, even though Phase 03 now includes BM1366 protocol logic and diagnostic chip-detect gates. This is not a runtime bug, but it leaves a stale public placeholder in the crate surface.
**Fix:** Remove the placeholder if no caller needs it, or rename it to match the current fail-closed/gated Phase 03 behavior.

## Verification

- Targeted anti-pattern scan for secrets, dangerous functions, debug artifacts, and empty catches found no matches in reviewed files.
- `git check-ignore -v` produced no ignored-file matches for reviewed source files.
- `cargo test -p bitaxe-asic --all-features` passed: 47 unit tests, 0 failures; doc tests passed.
- `source "$HOME/export-esp.sh" && cargo check -p bitaxe-firmware --target xtensa-esp32s3-espidf` passed.
- `bazel test //crates/bitaxe-asic:tests` passed.

---

_Reviewed: 2026-06-27T01:51:44Z_
_Reviewer: the agent (gsd-code-reviewer)_
_Depth: standard_

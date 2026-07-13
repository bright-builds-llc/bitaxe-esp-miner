---
status: resolved
trigger: "Ultra 205 fans spin but the LCD remains blank after a physical replug while the Phase 28.1.1 accepted-state diagnostic Rust package is retained."
created: 2026-07-10T00:05:26Z
updated: 2026-07-10T00:44:23Z
---

## Current Focus

hypothesis: Confirmed and resolved - the Phase 27 bridge worker overflowed ESP-IDF's inherited 3072-byte pthread stack; a dedicated 16 KiB worker stack removes the deterministic reboot loop.
test: Detector-gated flash of the corrected accepted-state package followed by a redacted 360-second hardware capture and post-run board-info check.
expecting: Satisfied - one boot, one `h4_continuous_result=listener_armed`, five accepted-state markers, and no stack overflow, panic, software reset, or repeated boot.
next_action: Preserve the corrected package on the device, commit the hardware verification update, and push the verified fix. Do not erase NVS.

## Symptoms

expected: After reconnecting or power-cycling the Ultra 205, its LCD should show normal device text while firmware boots and runs.
actual: Fans spin, but the LCD is blank.
errors: No visible LCD error. A no-reset serial attachment made after boot has so far shown only the monitor header, with no ROM boot, panic, watchdog, I2C/display error, or accepted-state markers; this was not a synchronized boot capture.
reproduction: Unplug and replug USB power while the Phase 28.1.1 accepted-state diagnostic Rust package remains installed.
started: Observed immediately after the physical replug requested for Phase 28.1.1 cold-start lifecycle verification.

## Eliminated

- Factory-reset requirement: eliminated as a response to the blank LCD alone. The installed diagnostic mode deliberately skips display initialization; erasing NVS or firmware would destroy useful state without testing the leading hypothesis.
- LCD blankness as boot-loop evidence: eliminated. The same visible symptom follows deterministically from the Phase 27 display-deferred branch even during a healthy boot.
- Missing diagnostic package identity: eliminated. The current local enablement ledger and package manifest both identify source commit `decd3b382a1d042134d05a2e941649e60b222f9d`; the promoted baseline records that same package as retained.
- Expected normal runtime display parity: eliminated for this package. The display adapter itself declares a startup-only runtime gap, and the Phase 27 branch does not call even that startup renderer.

## Evidence

- timestamp: 2026-07-10T00:05:26Z
  checked: `scripts/phase28.1.1-accepted-state-diagnostic.sh:90-106`
  found: The accepted-state attempt adds `--investigation accepted_state_snapshot` and delegates package construction to `scripts/phase27-live-hardware-bridge-package.sh`.
  implication: The retained Phase 28.1.1 diagnostic is not the default display-rendering firmware; it is built on the Phase 27 bridge package.
- timestamp: 2026-07-10T00:05:26Z
  checked: `scripts/phase27-live-hardware-bridge-package.sh:4-6,61-75`
  found: The package compiles with `BITAXE_MINING_EVIDENCE_MODE=phase27-live-hardware-asic-stratum-bridge` plus the matching hardware-evidence acknowledgement.
  implication: `MiningEvidenceMode::current()` resolves to `Phase27LiveHardwareBridge` for the installed diagnostic.
- timestamp: 2026-07-10T00:05:26Z
  checked: `firmware/bitaxe/src/mining_evidence_mode.rs:43-53,67-70`
  found: The exact compile-time mode and acknowledgement pair selects `Phase27LiveHardwareBridge` and makes `is_phase27_live_hardware_bridge()` true.
  implication: The Phase 27 startup branch is selected deterministically, not inferred from the blank screen.
- timestamp: 2026-07-10T00:05:26Z
  checked: `firmware/bitaxe/src/main.rs:60-95`
  found: When Phase 27 mode is active, startup logs `display_status=deferred reason=phase27_safety_i2c0_in_use` and passes I2C0/GPIO47/GPIO48 to safety bring-up. Only the `else` branch calls `display_adapter::render_startup_debug_text()`.
  implication: A power-cycled OLED remains blank by design under this diagnostic package because firmware never initializes or flushes text to it.
- timestamp: 2026-07-10T00:05:26Z
  checked: `firmware/bitaxe/src/display_adapter.rs:1,29-39,53-78`
  found: The available display implementation is explicitly startup-only; rendering requires creating an I2C0 driver, initializing the SSD1306, drawing text, and flushing it.
  implication: Skipping the renderer leaves no independent runtime display path that could populate the screen later.
- timestamp: 2026-07-10T00:05:26Z
  checked: `firmware/bitaxe/src/safety_adapter/phase27_bring_up.rs:91-120,153-176`
  found: Phase 27 initializes the shared I2C bus, initializes the EMC2101, sets startup fan duty, records the fan marker, and retains the I2C bus for power sampling.
  implication: Fans spinning while the screen stays blank is the expected combined signature of this mode, not contradictory evidence of a boot loop.
- timestamp: 2026-07-10T00:05:26Z
  checked: `bazel-bin/firmware/bitaxe/phase27-live-hardware-bridge-enablement.md` and `bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json`
  found: The local package ledger says Phase 27 live bridge, `work_result_investigation: accepted_state_snapshot`, source commit `decd3b382a1d042134d05a2e941649e60b222f9d`; the manifest has the same source commit.
  implication: The concrete package associated with the hardware capture has exactly the mode that intentionally suppresses the LCD.
- timestamp: 2026-07-10T00:05:26Z
  checked: `.planning/phases/28.1.1-bm1366-nonce-production-wire-parity/28.1.1-accepted-state-baseline-redacted.md` and `28.1.1-accepted-state-lifecycle-redacted.md`
  found: The promoted baseline identifies source commit `decd3b382a1d042134d05a2e941649e60b222f9d`, records readable diagnostic observations and completed work dispatch, and says the Rust phase package was retained; the lifecycle ledger repeats `rust_phase_package_status: retained` and post-capture board-info passed.
  implication: Repository evidence supports a functioning retained diagnostic package and does not support spontaneous replacement by a different firmware image.
- timestamp: 2026-07-10T00:05:26Z
  checked: git history for `display_status=deferred reason=phase27_safety_i2c0_in_use`
  found: Commit `255d495d7ec39251d76c52b6d37a72e0e69fdd58` deliberately introduced the Phase 27 branch, removed display rendering from that branch, and added shared-I2C safety/fan bring-up.
  implication: The blank Phase 27 display is an intentional architecture boundary introduced with live hardware safety, not an accidental change made by the accepted-state instrumentation.
- timestamp: 2026-07-10T00:09:30Z
  checked: `scratch/phase28.1.1-accepted-state/blank-lcd-diagnosis/controlled-reset-monitor.raw.log` using a closed safe-marker extraction only
  found: The synchronized capture contains five `bitaxe-rust boot` markers and four complete failure cycles. In each complete cycle, `h4_continuous_result=listener_armed` is followed immediately by `***ERROR*** A stack overflow in task pthread has been detected.`, a corrupted backtrace, `Rebooting...`, and `rst:0xc (RTC_SW_CPU_RST)`; the intervals are approximately 17 seconds.
  implication: A real deterministic software reset loop is independently confirmed. The earlier late-attach capture could not see it, and the blank LCD remains nondiagnostic by itself.
- timestamp: 2026-07-10T00:09:30Z
  checked: first complete controlled-reset cycle at raw capture lines 519-536
  found: The safe sequence is `phase25_live_stratum_status=connecting`, socket watchdog checkpoint, `h4_continuous_result=listener_armed`, pthread stack-overflow error, corrupted backtrace, reboot, and RTC software CPU reset. There is no intervening display, NVS, I2C, watchdog, or credential error.
  implication: The failure boundary is the active live bridge worker, not display initialization or factory configuration.
- timestamp: 2026-07-10T00:09:30Z
  checked: `firmware/bitaxe/src/live_stratum_runtime.rs:123-141,179-235`
  found: Phase 27 creates one `phase27-bridge` Rust thread with `thread::Builder::new().name(...).spawn(...)` but no `.stack_size(...)`; that same worker waits for settings and then owns connection, runtime construction, socket pumping, JSON handling, ASIC orchestration, and accepted-state instrumentation.
  implication: The bridge worker inherits the ESP-IDF pthread default instead of a workload-sized stack and is the leading source of the `pthread` overflow.
- timestamp: 2026-07-10T00:09:30Z
  checked: `firmware/bitaxe/src/live_stratum_runtime.rs:566-666,674-860,1121-1164`
  found: The bridge worker keeps `LiveStratumRuntime`, pending actions, pending submit state, and expanded `AsicBridgeState` on one call chain. Arming the listener makes the continuous poll path live; the accepted-state package also adds observation/deadline/category fields and read handling to this same worker, but does not create another thread.
  implication: The current live-bridge call chain plausibly exceeds 3072 bytes. The exact accepted-state contribution is not isolated by an A/B capture, so diagnosis should target the worker stack contract rather than blame one field or one log statement.
- timestamp: 2026-07-10T00:09:30Z
  checked: `bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json:4-26` and `target/xtensa-esp32s3-espidf/release/build/esp-idf-sys-2a443813359d6cae/out/sdkconfig:1322,2063-2074`
  found: The installed package manifest identifies source commit `decd3b382a1d042134d05a2e941649e60b222f9d` and points at the `2a443813359d6cae` ESP-IDF build. That exact generated configuration gives the main task 8192 bytes but leaves `CONFIG_PTHREAD_TASK_STACK_SIZE_DEFAULT=3072` and `CONFIG_PTHREAD_TASK_NAME_DEFAULT="pthread"`.
  implication: The panic's task name and the bridge worker's inherited stack size match the exact installed package, rather than a generic build assumption.
- timestamp: 2026-07-10T00:09:30Z
  checked: `firmware/bitaxe/sdkconfig.defaults:1-25` and ESP-IDF `components/pthread/Kconfig:10-20,43-47`
  found: Repository defaults explicitly raise only the main task to 8192 bytes and do not override pthread stack size; ESP-IDF therefore supplies its 3072-byte pthread default and default name `pthread`.
  implication: Increasing the main task stack cannot cure this crash; the failing worker needs an explicit per-thread stack contract or an intentionally justified pthread-default change.
- timestamp: 2026-07-10T00:09:30Z
  checked: `firmware/bitaxe/src/http_api.rs:56-57,65-71,95-101,984-996`
  found: Comparable nontrivial firmware workers already avoid the 3072-byte default: HTTP uses an 8192-byte server stack, live telemetry requests 16384 bytes, and settings effects requests 8192 bytes.
  implication: A deliberate stack size on the Phase 27 bridge worker is consistent with existing project practice; blindly raising every pthread is unnecessary.
- timestamp: 2026-07-10T00:09:30Z
  checked: `firmware/bitaxe/src/safety_adapter/watchdog.rs:13-28`
  found: The safety supervisor is another default-stack Rust pthread, but its loop is shallow and had already been running for roughly five seconds before each bridge listener marker and immediate crash.
  implication: A corrupted backtrace prevents absolute task attribution, but timing and workload make the Phase 27 bridge worker substantially more likely than the safety supervisor.
- timestamp: 2026-07-10T00:31:47Z
  checked: `firmware/bitaxe/src/live_stratum_runtime.rs` Phase 27 worker scheduling path
  found: The worker now requests a dedicated 16 KiB stack, matching the existing heavy live-telemetry worker while leaving the global ESP-IDF pthread default unchanged. Spawn failure is no longer discarded: it logs `reason=spawn_failed`, publishes a redacted blocked/safe-stop state, and marks the bridge complete.
  implication: The correction targets only the confirmed heavy worker and fails visibly if the bounded task-stack allocation cannot be created.
- timestamp: 2026-07-10T00:31:47Z
  checked: `cargo check -p bitaxe-firmware --all-features --target xtensa-esp32s3-espidf` and `cargo clippy -p bitaxe-firmware --all-features --target xtensa-esp32s3-espidf`
  found: Both focused cross-target production checks pass. They report existing warnings outside this change but no new compile or lint failure in the corrected scheduling path.
  implication: The corrected firmware production path compiles for ESP32-S3; hardware remains the required behavioral proof for the stack budget.
- timestamp: 2026-07-10T00:31:47Z
  checked: `cargo test -p bitaxe-firmware --all-features --target xtensa-esp32s3-espidf --no-run` and `cargo fmt --all -- --check`
  found: Firmware test compilation reaches two pre-existing `assert!(SocketLineOutcome)` errors at lines 2093 and 2188 plus an unrelated warning; format check reports pre-existing diffs in `crates/bitaxe-stratum/src/v1/live_runtime.rs` and an unchanged line in this firmware file. No source-text stack-size test was added because it would not verify runtime stack sufficiency.
  implication: The production fix is ready for the main agent's full gate/format pass, but only the detector-gated hardware capture can close this resource-budget regression.
- timestamp: 2026-07-10T00:44:23Z
  checked: mandatory Rust gates, exact Phase 27 accepted-state package build from commit `abf9db49949dd9e2f6ed6ad6666379cb816d2d30`, detector-gated flash, redacted 360-second capture, and post-run `just detect-ultra205`
  found: Formatting, strict Clippy, all-target build, full tests, reference cleanliness, firmware packaging, pre/post ESP32-S3 board-info, and the hardware workflow all passed. The corrected capture contains one firmware boot, one listener-arm marker, five accepted-state markers, zero pthread stack overflows, zero `RTC_SW_CPU_RST` markers, and zero panic/backtrace markers.
  implication: The former approximately 17-second deterministic reboot loop is resolved on the Ultra 205 without erasing NVS or changing the global pthread default.

## Resolution

root_cause: Two independent behaviors were present. First, the retained Phase 28.1.1 accepted-state diagnostic intentionally leaves the LCD blank while Phase 27 safety bring-up starts the fan; this remains expected. Second, the original installed image entered an approximately 17-second boot loop because a Rust/ESP-IDF pthread overflowed and the panic policy rebooted. The overflowing pthread was the Phase 27 bridge worker: it reached `h4_continuous_result=listener_armed`, executed the heavy socket/JSON/Stratum/ASIC path, and inherited the exact package configuration's 3072-byte pthread default because it lacked `.stack_size(...)`. The corrected hardware capture confirms that assigning this worker a dedicated 16 KiB stack removes the loop.
confidence: high for intentional LCD deferral, the original stack-overflow reboot loop, attribution to the undersized `phase27-bridge` worker, and the hardware-verified correction.
boot_loop_assessment: Resolved. The faulty image produced four complete approximately 17-second listener-arm/overflow/reboot cycles; the corrected image produced one boot and remained stable for the full 360-second capture with no overflow, panic, or software reset.
fix: Applied the smallest targeted correction in `firmware/bitaxe/src/live_stratum_runtime.rs`: the `phase27-bridge` worker now requests 16 KiB, matching the comparable live-telemetry worker, without globally increasing pthread stacks. A spawn failure is visible and converges to blocked safe-stop state instead of being silently ignored. Do not erase NVS or factory-reset; normal repo-owned reflash of this corrected diagnostic image is the recovery path.
verification: Passed. Mandatory Rust gates, reference cleanliness, exact-commit ESP32-S3 packaging, detector-gated flash, a redacted 360-second capture, and post-run board-info all passed. Hardware markers show one boot, one listener arm, five accepted-state stages, and zero stack overflow, panic, or software-reset markers.
files_changed:
  - `firmware/bitaxe/src/live_stratum_runtime.rs`
  - `.planning/debug/ultra205-blank-lcd-after-diagnostic-replug.md`

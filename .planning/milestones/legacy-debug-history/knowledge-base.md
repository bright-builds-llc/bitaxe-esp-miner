# GSD Debug Knowledge Base

Resolved debug sessions. Used by `gsd-debugger` to surface known-pattern hypotheses at the start of new investigations.

***

## phase35-http-diagnostic-invalid-attempt12 — Positive curl duration collapsed into the zero sentinel

- **Date:** 2026-07-21
- **Error patterns:** http_diagnostic_invalid, original settings, sub-millisecond duration, curl metrics, classifier boundary
- **Root cause:** Nearest-integer millisecond conversion mapped a positive curl duration below half a millisecond to `0`, which the strict schema reserves for an absent boundary; later request facts then made the observation inconsistent before Rust classification.
- **Fix:** Preserve exact zero as absence while mapping every positive duration to at least one millisecond, including the derived TLS handshake duration, with direct and Bazel/runfiles fake-curl regressions.
- **Files changed:** scripts/phase35-http-boundary-read.sh, scripts/phase35-http-boundary-read-test.sh

***

## phase35-request-transmission-incomplete-attempt14 — Receive failure misclassified from a zero curl request counter

- **Date:** 2026-07-21
- **Error patterns:** request_transmission_incomplete, size_request zero, curl exit 56, response receive failure, request bytes zero
- **Root cause:** Curl exit 56 is a receive-side failure and can occur after a complete bodyless GET even while `%{size_request}` remains zero. The classifier treated that raw counter as the sole proof of request transmission and emitted the earlier inaccurate category.
- **Fix:** Preserve raw request bytes, but derive bodyless-GET transmission completion from positive request bytes or curl receive error 56 in both shell and Rust. Keep exit 55 as the send-failure boundary and cover missing-response plus partial-response receive failures.
- **Files changed:** scripts/phase35-http-boundary-read.sh, scripts/phase35-http-boundary-read-test.sh, tools/parity/src/phase35_http.rs, tools/parity/src/phase35_http/tests.rs

***

## phase35-attempt15-timeout — Host curl request-size counter stayed zero after complete GETs

- **Date:** 2026-07-21
- **Error patterns:** request_transmission_incomplete, response timeout, curl exit 28, size_request zero, successful GET counterexample
- **Root cause:** The host curl build reported `%{size_request}=0` after peers received the complete bodyless request, including on successful responses. Fake fixtures supplied positive values and concealed that the classifier was using an unusable counter as its primary send boundary.
- **Fix:** Replace curl with a repo-owned schema-v2 Rust probe. Mark transmission complete only after the full request write and transport flush succeed; retain partial-byte counts and typed transport outcomes without persisting raw request material.
- **Files changed:** scripts/phase35-http-boundary-read.sh, scripts/phase35-http-boundary-read-test.sh, tools/parity/src/main.rs, tools/parity/src/phase35_http.rs, tools/parity/src/phase35_http/tests.rs, tools/parity/src/phase35_http_probe.rs, tools/parity/src/phase35_http_probe/tests.rs

***

## macos-rust-launch-and-cache-stalls — Separate policy assessment from ignored-cache enumeration

- **Date:** 2026-07-24
- **Error patterns:** unrelated Macroquad/Miniquad abort popups, `_dyld_start`, AppleSystemPolicy first launch, `target/debug/deps` enumeration stall
- **Root cause:** The popups came from unrelated GUI binaries. Fresh Rust executables separately encountered macOS AMFI/AppleSystemPolicy assessment in an unhealthy long-running host session, while a later Rust gate blocked on unreadable enumeration of the pre-existing ignored Cargo cache.
- **Fix:** Perform a full reboot, allow a bounded first-launch policy assessment, run complete Rust gates in clean isolated targets, recoverably quarantine the stalled ignored cache, and verify normal target recreation.
- **Non-workaround:** No repository source, signing, xattr, provenance, AMFI, or security-policy change was justified.

***

## phase36-plan08-flash-and-recovery-failed — Conflicting flash evidence options rejected before device access

- **Date:** 2026-07-25
- **Error patterns:** flash_failed, recovery_failed, sealed_non_promotion, exact flash, typed recovery, CLI validation
- **Root cause:** `scripts/phase36-hardware-effect.sh` passed `--evidence-mode dual` and `--redact-evidence` to the `tools/flash flash` subcommand for both exact-package flash and typed recovery. The real parser rejects dual outside `flash-monitor` and declares the two flags conflicting, so both operations exited before environment or device execution and the broker normalized those exits to `flash_failed` and `recovery_failed`.
- **Fix:** Removed the flash-monitor-only dual evidence option from Phase 36 exact flash and typed recovery while retaining redacted evidence. Added deployed-adapter OS-boundary and real flash-parser regressions for the corrected command shape. Updated the process test's stale public incomplete-plan expectation after Plan 36-08 completed.
- **Files changed:** scripts/phase36-hardware-effect.sh, scripts/phase36-substantive-evidence-test.sh, tools/flash/src/main.rs, .planning/debug/resolved/phase36-plan08-flash-and-recovery-failed.md

***

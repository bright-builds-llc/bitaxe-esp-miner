---
status: resolved
trigger: "Phase 33 stops with physical_identity_unavailable after its detector-approved Ultra 205 preflight on macOS."
created: 2026-07-14T15:13:25Z
updated: 2026-07-14T15:33:53Z
---

## Current Focus

hypothesis: Confirmed and fixed - the ancestor parser found the correct fields but returned before a large real `ioreg` producer reached EOF. Under the wrapper's `set -o pipefail`, that early close gave `ioreg` SIGPIPE status 141 and rejected the otherwise valid command substitution.
test: Complete. The physical identity helper now runs under pipefail against an ioreg fixture with a deterministic SIGPIPE trap and enough trailing tree data to expose an early close.
expecting: Met. The parser captures exactly one target identity, drains through EOF, then validates and emits it without changing field eligibility.
next_action: Return the resolved continuation so the parent Phase 33 workflow can begin a new detector-gated attempt.

## Symptoms

expected: `serial_session_usb_physical_identity(port)` returns a stable digest for the detector-approved Ultra 205 by associating the IOSerialBSDClient leaf with its USB ancestors, requiring idVendor, idProduct, and USB Serial Number or locationID, while excluding enumeration-only tty and registry fields.
actual: Detector and board-info pass, but Phase 33 stops before flash because the identity helper cannot construct the physical identity.
errors: Exact category `physical_identity_unavailable`.
reproduction: The repo-owned Phase 33 wrapper calls the helper after its sole detector preflight on macOS native USB.
started: First eligible post-replug attempt. The existing fake Darwin test had all USB fields incorrectly on the IOSerialBSDClient leaf.

## Eliminated

- The Phase 33 detector and ESP32-S3 board-info checks are not the failing boundary; both completed before the helper returned unavailable.
- Missing USB identity properties are not a property of the approved device; read-only IOService evidence shows the required fields on its USB ancestors.
- Enumeration-only fields are not eligible as a fallback because the physical identity must remain stable across re-enumeration.

## Evidence

- timestamp: 2026-07-14T15:13:25Z
  checked: Existing `serial_session_usb_physical_identity` Darwin branch and focused fake test.
  found: The helper queries `ioreg -r -c IOSerialBSDClient -l -w 0`, extracts the matching leaf block, and requires all stable fields in that block. The fake test places USB Serial Number, idVendor, idProduct, and locationID directly on that leaf.
  implication: The implementation and test share the same false topology assumption.
- timestamp: 2026-07-14T15:13:25Z
  checked: Confirmed redacted read-only macOS IORegistry evidence from the failed Phase 33 attempt.
  found: The target IOSerialBSDClient leaf has no stable USB fields, while its IOUSBHostDevice and IOUSBHostInterface ancestors provide the required vendor, product, and serial-or-location properties; exactly one Espressif USB candidate is present.
  implication: Physical identity must be derived from the target leaf's active IOService ancestor path, not from the leaf property block or a global USB candidate search.
- timestamp: 2026-07-14T15:14:55Z
  checked: Direct serial-session test after replacing the Darwin fixture with realistic leaf-versus-ancestor output and an unrelated serial tree.
  found: `bash scripts/serial-session-trace-test.sh` exited 1 before the fix because the old physical-identity query received a target leaf with no stable USB properties.
  implication: The fixture reproduces the exact unavailable boundary without using the connected board and confirms the topology mismatch as root cause.
- timestamp: 2026-07-14T15:19:30Z
  checked: Focused direct tests, shell static analysis/formatting, Bazel serial/detector/Phase 33/parity tests, and diff/redaction review after the fix.
  found: Direct serial-session and Phase 33 simulation tests passed; ShellCheck and `shfmt -d` passed for the affected scripts; Bazel passed `//scripts:serial_session_trace_test`, `//scripts:detect_ultra205_test`, `//scripts:phase33_confirmed_settings_durability_test`, and `//tools/parity:tests`; `git diff --check` passed. No board, detector, board-info, flash, monitor, HTTP mutation, or restart command ran during debugging.
  implication: The macOS fix is regression guarded across the direct and canonical Bazel surfaces without weakening the hardware gate or consuming a hardware attempt.
- timestamp: 2026-07-14T15:31:34Z
  checked: Confirmed wrapper-shell comparison using the latest protected detector session without a new hardware operation.
  found: Both identity helpers return the expected digest/fields from Bash without pipefail, while the Phase 33 wrapper still classifies the same pre-flash call as `physical_identity_unavailable` under `set -euo pipefail`. The parser returns at the target leaf before the large real ioreg stream completes, causing upstream SIGPIPE status 141.
  implication: The ancestor association fix is correct but incomplete at the shell pipeline boundary; the consumer must drain input to EOF before emitting.
- timestamp: 2026-07-14T15:33:53Z
  checked: Large-producer regression before and after moving parser emission to EOF.
  found: Before the continuation fix, the fixture's producer reported a broken pipe and the direct suite failed with `Darwin physical identity did not drain ioreg under pipefail`. After the fix, the direct serial-session and Phase 33 simulation suites passed, ShellCheck and `shfmt -d` passed, and Bazel passed the serial-session, detector, Phase 33, and parity targets.
  implication: The regression exercises the wrapper's actual pipefail boundary and proves the parser no longer converts a valid target into producer status 141.

## Resolution

root_cause: Two defects combined. First, the original Darwin helper looked only at IOSerialBSDClient leaf properties even though native macOS keeps stable identity fields on USB ancestors. Second, the first ancestor parser returned immediately after finding the target; a large real ioreg producer then received SIGPIPE, making the pipeline fail under the wrapper's pipefail semantics. The small fake producer completed before the close and masked the second defect.
fix: Preserve exact ancestor selection, capture exactly one target identity, continue consuming the producer through EOF, then validate and emit the stable fields. The pipefail boundary no longer converts an accepted target into SIGPIPE failure; missing fields, unrelated trees, duplicate targets, and Linux behavior remain fail-closed or unchanged.
verification: The large-producer pipefail regression, direct serial-session and Phase 33 simulation tests, ShellCheck, `shfmt -d`, affected Bazel targets, and diff/redaction review passed. The repository-mandated ordered Rust gate is recorded in the commit handoff.
files_changed: [`scripts/serial-session-trace.sh`, `scripts/serial-session-trace-test.sh`, `.planning/debug/phase33-macos-usb-physical-identity.md`]

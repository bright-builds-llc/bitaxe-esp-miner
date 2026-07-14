---
status: resolved
trigger: "Phase 33 stops with physical_identity_unavailable after its detector-approved Ultra 205 preflight on macOS."
created: 2026-07-14T15:13:25Z
updated: 2026-07-14T15:19:30Z
---

## Current Focus

hypothesis: Confirmed - the Darwin physical-identity helper inspected only the IOSerialBSDClient leaf, but macOS publishes the required stable USB vendor, product, serial, and location properties on IOService ancestors. The former fake test incorrectly placed every field on the leaf and masked the production topology mismatch.
test: Complete. The pure Bash parser selects stable fields only from IOUSBHostDevice or IOUSBHostInterface nodes on the exact target leaf's active path.
expecting: Met. The fixed helper returns one stable digest for the target across enumeration-only leaf changes, rejects unmatched or insufficient trees, ignores the unrelated serial tree, and leaves Linux behavior unchanged.
next_action: Return the resolved debug result so the parent Phase 33 workflow can start a fresh detector-gated hardware attempt.

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

## Resolution

root_cause: The Darwin helper queries a property-only IOSerialBSDClient leaf archive and requires stable USB properties there. Native macOS publishes those properties on IOUSBHostDevice and IOUSBHostInterface ancestors, so the target leaf matches the port but cannot satisfy the field gate. The old fake test copied ancestor fields onto the leaf and could not expose the failure.
fix: Query the IOService tree with ancestry, parse the exact target leaf's current ancestor path in a pure Bash helper, accept stable fields only from IOUSBHostDevice and IOUSBHostInterface nodes, and hash a canonical vendor/product plus serial-or-location projection. Enumeration-only leaf names, device nodes, registry IDs, process IDs, and unrelated serial trees never enter the projection. Realistic fixtures now keep USB fields on ancestors, vary unrelated and enumeration-only data, and cover missing required fields. Linux behavior is unchanged.
verification: Focused direct tests, ShellCheck, `shfmt -d`, affected Bazel tests, and diff/redaction review passed. The repository-mandated ordered Rust gate is recorded in the commit handoff after it completes.
files_changed: [`scripts/serial-session-trace.sh`, `scripts/serial-session-trace-test.sh`, `.planning/debug/phase33-macos-usb-physical-identity.md`]

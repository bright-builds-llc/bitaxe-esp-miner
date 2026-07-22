---
status: verifying
trigger: "Attempt 24 stopped at the read-only checksum probe with flash_boundary_invalid before credential access or writes."
created: 2026-07-22T15:10:01Z
updated: 2026-07-22T15:10:01Z
---

## Current Focus

hypothesis: The probe completed enough to print its checksum, but the adapter required a fixed-width digest while espflash 4.5.0 prints an unpadded `u128`; the classifier then serialized its post-info boundary with a spelling different from the shell's canonical category.
test: Compare only the protected checksum shape with the installed espflash 4.5.0 source contract, replay the immutable metrics/log through the repaired classifier, and run a real-process fake espflash that emits the same leading-zero-elided shape.
expecting: The real-process probe accepts exactly one bounded unpadded lowercase hexadecimal checksum line, the projection uses `post_info_pre_transfer_failed`, and malformed, overlong, uppercase, embedded, or multiple candidates remain rejected.
next_action: Commit this redacted checkpoint, run the complete clean software gate and exact-current-HEAD preflight, then use fresh Attempt 25 under `continue_after_verified_fix`.

## Symptoms

expected: The read-only 4 KiB checksum probe either reaches `ready` or emits one canonical typed flash boundary before any sensitive input or write.
actual: The child connected, completed device information, and emitted one checksum-shaped line, but the adapter recorded no transfer start. The classifier wrote a projection whose enum spelling the shell rejected, so the supervisor preserved the coarser `flash_boundary_invalid` category.
errors: Shareable signature is `flash_boundary_invalid` with no admitted flash stage or boundary, no credential access, no writes, no mutation, and clean secondary outcomes.
reproduction: Attempt 24 is sealed and must not be reused. Its immutable protected log has exactly one line with the official `0x` prefix and a 31-digit checksum payload; espflash 4.5.0 source prints `u128` with `{checksum:x}`, which intentionally omits leading zeroes.
started: Attempt 24 at exact source `dec8b8a6bef8f504ec83a7eebe03b69a08be5064` after doctor and exact-head preflight passed.

## Eliminated

- Detector or pre-connect failure: the typed metrics record connection and complete device information.
- Credential or write side effects: the probe runs before credential validation and every write stage; neither began.
- Secret sanitization or malformed private input: the private log is mode `0600`, bounded, valid text, and contains one sanitizer-safe checksum shape.
- espflash version drift: the exact 4.5.0 provenance contract passed.
- Hardware retry as diagnosis: the sealed inputs reproduce the classifier spelling defect offline, and installed espflash source proves the variable-width output contract without another device request.

## Evidence

- timestamp: 2026-07-22T15:10:01Z
  checked: Attempt 24's sealed typed metadata and protected artifact shapes without printing the checksum, device identifiers, commands, or paths.
  found: The probe connected and completed device information; the sole output line contains the official prefix and 31 hexadecimal checksum digits.
  implication: The fixed 32-digit parser rejected a valid leading-zero-elided espflash result.
- timestamp: 2026-07-22T15:10:01Z
  checked: Installed espflash 4.5.0 source for the `checksum-md5` command.
  found: The command converts the digest to `u128` and prints it with lowercase unpadded hexadecimal formatting.
  implication: Valid output is one `0x`-prefixed lowercase value containing between 1 and 32 hexadecimal digits.
- timestamp: 2026-07-22T15:10:01Z
  checked: The immutable Attempt 24 metrics and private log through the rebuilt parity classifier.
  found: The replay writes schema `phase35-flash-boundary-v1`, stage `probe`, and canonical terminal boundary `post_info_pre_transfer_failed`; a non-ready classifier exit is expected.
  implication: The enum serialization mismatch is fixed without modifying the sealed attempt.
- timestamp: 2026-07-22T15:10:01Z
  checked: Focused flash, parity, Phase 35 supervisor, promotion, and Phase 30 non-promotion suites plus the mandatory Rust gate.
  found: All pass, including a real-process variable-width checksum regression and exact projection-spelling regression.
  implication: The repair covers both observed software mismatches and preserves fail-closed surrounding contracts.

## Resolution

root_cause: The read-only probe assumed a fixed-width 32-digit MD5 line, but espflash 4.5.0 prints an unpadded hexadecimal `u128`; the valid leading-zero-elided result was therefore marked incomplete. The resulting non-ready projection used serde's default `failure` spelling while the shell contract requires `failed`, replacing the discriminating boundary with `flash_boundary_invalid`.
fix: Accept exactly one official lowercase `0x`-prefixed checksum containing 1 through 32 hexadecimal digits, reject ambiguous or malformed candidates, and explicitly serialize the canonical `post_info_pre_transfer_failed` boundary.
verification: Code commit `671011a7` passes focused real-process and classifier suites plus the mandatory Rust gate. The checkpoint commit and exact-head preflight remain pending before Attempt 25.
files_changed:

- tools/flash/src/main.rs
- tools/parity/src/phase35_flash.rs
- .planning/debug/phase35-attempt24-probe-checksum-shape.md

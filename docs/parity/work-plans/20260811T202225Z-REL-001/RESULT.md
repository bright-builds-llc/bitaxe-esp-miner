# Parity work result

- Parity row: `REL-001`
- Final status: `verified`
- Implementation commit: `9d88a6454ae9171c91516d3842581f8188633b6d`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Board: Ultra 205
- Hardware attempts under this plan: one

## Evidence and verification

The exact clean schema-v3 package built from pushed implementation commit
`9d88a6454ae9171c91516d3842581f8188633b6d` contains the six required artifact
kinds and is bound to the pinned reference. The sole detector admitted one
Ultra 205. The sole conditional capture flashed that exact factory package,
proved passive factory safe state, admitted one trusted same-session origin,
prearmed the same-device reader, uploaded the exact OTA application once, and
observed the application recover from `factory` in `ota_0`.

The committed
[`bitaxe-partition-layout-evidence-v1`](../../evidence/rel001-ota-slot/partition-layout-projection.json)
projection has SHA-256
`a9c79eecfc8ad75859d676d7e4b6ea0a6047be6710a808f0bab98ab752ccb10a`.
The repository-owned Rust validator independently accepts it. Its closed facts
prove:

- exact source, pinned reference, package-manifest identity, and board 205
  admission;
- an eight-row canonical Ultra 205 table with the exact partition-table and
  OTA-image digests recorded by the package;
- a safe factory baseline and completed upload of the exact OTA image;
- reader admission before exactly one request, correlated serial delivery,
  preserved trusted origin, service loss, and application recovery on the
  same physical device;
- exact recovered build identity, a changed boot session, ordinal `N+1`, a
  software reset, complete OTA boot validation, and `ota_0` execution;
- mining and hardware control remained disabled, cleanup completed, every
  private file/directory used its required protected mode, and semantic
  redaction passed.

Independent digest checks reproduce the projection's package-manifest,
partition-table, and OTA-image SHA-256 values. The focused actual-table and
negative-drift regressions passed, followed by the complete ordered software
gate. The exact effect-capable commands were the immutable plan's `just
package`, private `just detect-ultra205`, and conditional `just
capture-partition-layout-evidence` commands.

## Conclusion

The exact pushed Rust firmware uses the canonical Ultra 205 partition layout,
boots safely from `factory`, accepts one complete application OTA update, and
returns on the same device in `ota_0` with exact build and next-boot identity.
This satisfies `REL-001` with unit, API-comparison, workflow, and
hardware-smoke evidence.

## Non-claims and residual risks

Raw detector, USB, serial, HTTP, origin, network, credential, OTA, and process
material remains only in ignored protected roots. The Wi-Fi credential file
was an opaque local input, and no pool credential was read. Stable USB
enumeration remained present across the software restart, so this result does
not claim a physical USB disappearance/re-enumeration event. It also does not
claim rollback, interrupted-update behavior, OTAWWW/static-partition updates,
recovery upload, erase-flash, arbitrary writes, mining, ASIC work, hardware
controls, other-board behavior, or release readiness.

# Parity work closure

- Parity row: `REL-001`
- Final status: `implemented`
- Outcome: `blocked`
- Verification claimed: `no`
- Plan SHA-256: `5f63933e7e7390553cfd4f7f8c15fdccdf4810c081c94f0d55269d946e03a187`
- Active task: `task-parity-rel001-live-ota-slot-transition`

## Closure reason

The one detector admitted exactly one Ultra 205, but the one conditional
capture stopped with the closed category `evidence_invalid` before any flash or
OTA effect. The package artifact digests were valid. The capture's canonical
partition comparison expected the `otadata` size token as `8K`, while the
checked-in, package-hashed partition table uses the ESP-IDF-equivalent spelling
`8k`. No public projection was created, and the immutable plan's retry bound
prohibits another capture in this attempt.

## Next safe action

Create a fresh REL-001 task and immutable plan. Normalize accepted ESP-IDF size
suffix spelling without weakening partition name, type, subtype, offset, size,
order, count, or digest binding; add a regression using the actual checked-in
partition table; pass the complete software gate; then use a fresh detector and
attempt ordinal. Standing task authorization remains sufficient once that new
contract is complete.

## Non-claims

This closure does not verify a factory boot, OTA upload, slot transition,
same-device restart, boot validation, rollback, interrupted update, OTAWWW,
recovery, mining, hardware control, other-board behavior, or release readiness.

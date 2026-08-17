# Parity work result

- Parity row: `REL-003`
- Final status: `verified`
- Implementation commit: `70493a51249df2f82eb5b046be7dc95b137c7e97`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`

## Evidence and verification

The exact clean package at implementation commit `70493a51` passed
`just package` and the manifest-backed release gate. The release gate proves
schema-v3 package inventory, canonical artifact paths and checksums, clean
provenance, license inventory, cargo-about review, install notes, Ultra 205
metadata, and the package workflow. The corrected partition-table artifact is
the canonical repository-relative
`firmware/bitaxe/partitions-ultra205.csv`; its red regression and the real
release gate prevent absolute host-path recurrence.

Accepted prior release evidence remains at:

- `docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/summary.md`
  and the Phase 19 summary, which prove package/release-gate provenance,
  detector and exact-package flash-monitor behavior, invalid-image rejection,
  a valid OTA response, recovery-route availability, and redaction;
- `docs/parity/work-plans/20260811T231354Z-REL-002/RESULT.md` and
  `docs/parity/evidence/rel002-sdkconfig-rollback/sdkconfig-rollback-projection.json`,
  which independently prove one reset-aborted interrupted application update
  with unchanged exact baseline, one pending-validation probe in `ota_0`,
  native rollback to the exact factory build, passive safe state, exact-build
  restoration, cleanup, protected modes, and redaction.

Together those accepted artifacts satisfy the release verifier's rollback,
recovery, failed-update, and interrupted-update terms. They explicitly left
large erase as a non-claim.

The new committed
`docs/parity/evidence/rel003-large-erase/release-recovery-projection.json` has
SHA-256
`6c712fd14e2dfc666a78602855efa825b9c104178efeee889e05b6f6b76f5b12`.
The independent `bitaxe-release-recovery-evidence-v1` validator proves board
205, attempt ordinal 1, exact source `70493a51`, the pinned reference, package
manifest SHA-256, immutable plan SHA-256, one admitted detector, one completed
full-flash erase, one exact factory restore, owner Wi-Fi/default NVS restore,
`mineonboot=false`, trusted runtime identity, SPIFFS readiness, passive safe
state, complete cleanup, no recovery reflash, and redaction.

The effectful commands were exactly the plan-frozen protected detector and
single conditional `just capture-release-recovery-evidence` invocation. The
wrapper and attempt roots are mode `0700`; their files are mode `0600`; the
public projection is mode `0644`. No private credential, device, USB, network,
process, origin, setting, command, log, PID, trace, or raw firmware value was
published. The standalone validator, `just verify-redaction`,
`just verify-reference`, package/release gate, focused Rust/Bazel suites, full
Cargo suite, Bright Builds, all Bazel tests, parity validation, immutable-plan,
and diff checks passed.

## Conclusion

REL-003 now has accepted release-gate, provenance, package-workflow, failed-
update, interrupted-update, native rollback, recovery, and full large-erase
recovery evidence on one Ultra 205. The new regression proves that the release
factory image reconstructs a completely erased board into the exact packaged
runtime with safe defaults, network access, static filesystem readiness, and
no mining/control activity. This satisfies the row's complete verified
release-image behavior boundary.

## Non-claims and residual risks

Large erase intentionally reset onboard settings; only owner Wi-Fi plus
package defaults were restored, and local ignored credential files remain
available. Pool values were not reseeded and mining remains disabled. This
result does not claim OTAWWW whole-www update parity, power-loss interruption,
eFuse anti-rollback, arbitrary raw writes, repeated erases, direct-UART or pin
behavior, release signing, factory provisioning at scale, other boards/ASICs,
production mining, electrical calibration, profitability, or commercial
release readiness. Those surfaces remain separately gated.

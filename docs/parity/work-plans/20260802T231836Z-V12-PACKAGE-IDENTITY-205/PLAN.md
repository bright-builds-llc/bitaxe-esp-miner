# Parity work plan

- Run ID: `20260802T231836Z-V12-PACKAGE-IDENTITY-205`
- Parity row: `V12-PACKAGE-IDENTITY-205`
- Initial status: `implemented`
- Source commit: `71e815e6a7bf4740c61d1885b3f04d9dabd7fc8e`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-v12-package-identity-205`

## Selection

`bazel run //tools/parity:report -- next-item --format json` reported no open
plan. The candidates before `V12-PACKAGE-IDENTITY-205` are not currently
promotable:

- `CFG-001` still needs a fresh, purpose-bound 485 MHz/1200 mV hardware
  actuation. The completed upstream-default soak artifacts are explicitly
  non-promotable, and the later attempt closed at a repeated continuity
  boundary; this work will not relabel them or authorize another soak.
- `CFG-005` remains broader than the implemented firmware persistence path.
  Hostname and `mineonboot` are confirmed, while the other upstream PATCH
  fields are compatibility-only; another hostname proof would not verify the
  row.
- `NET-001`, `ASIC-002` through `ASIC-005`, `ASIC-007`, `STR-001`, `STR-006`,
  and `STR-007` retain live reconnect, initialization, work/result, serial,
  frequency-transition, coordinator, or complete soak gaps.
- `API-002`, `API-003`, and `API-009` retain broader live response,
  persistence, or command-effect gaps than the current evidence closes.
- `PWR-001` through `PWR-003`, `PWR-005`, `PWR-006`, `THR-001` through
  `THR-003`, and `SELF-001` require their own safety-critical hardware
  regression contracts.
- `LOG-001` retains live initialization, soft-reboot retention, and lifecycle
  gaps. `REL-001` through `REL-003` retain selected-partition, SDK rollback,
  recovery, interrupted-update, and release-image gaps.
- `SAFE-10`, `SAFE-11`, `CFG-07`, `ASIC-09` through `ASIC-12`, `STR-08`,
  `STR-09`, `SAFE-12`, and `SAFE-13` retain live mining prerequisite,
  credential, ASIC correlation, transport, safe-stop, or watchdog gaps. The
  previous soak's repeated boundary is not a promotion source for them.

`V12-PACKAGE-IDENTITY-205` is now actionable. Its Phase 36 correction was
`runtime_identity_observation_insufficient`. The subsequently completed
bounded OTA result directly binds a clean manifest and OTA artifact to source
commit `2541818aa23120dd85c711386efadb69a1415ad3`, the pinned reference, one
detector-admitted Ultra 205, wrapper flash evidence, and exact post-reboot
firmware/reference identities. This is the missing observation for this row.

## Scope and non-scope

Reconcile only the exact source, reference, package, and runtime identity claim
for board 205. Create a row-specific result that cites the immutable OTA result
and transition receipt, rerun the deterministic package/runtime identity tests,
and promote only `V12-PACKAGE-IDENTITY-205` if every identity binding passes.

No firmware, reference, package, raw hardware evidence, or credential content
will change. No detector, flash, reset, OTA, HTTP, serial, mining, network,
voltage, fan, power, direct-UART, or pin effect is authorized or required by
this plan. The OTA attempt is not repeated.

This plan does not claim hostname durability, operator-snapshot substance,
runtime-health substance, selected-partition internals, rollback, OTAWWW,
network longevity, mining, safety controls, other boards, or release readiness.

## Implementation

- [ ] Confirm the immutable OTA result, transition receipt, checklist row, and
      runtime identity implementation agree on the exact package source and
      pinned reference commits.
- [ ] Run focused package-manifest and runtime-boot-attestation regressions;
      add code only if they expose a row-specific defect.
- [ ] Add an append-only worklog and a row-specific `RESULT.md` that records
      the direct evidence, conclusion, and non-claims without copying raw
      device artifacts.
- [ ] Commit the result before changing the checklist, then transition only
      `V12-PACKAGE-IDENTITY-205` and synchronize deterministic progress.

## Verification and promotion

Focused verification:

- `cargo test -p bitaxe-api runtime_boot_attestation --all-features`
- `cargo test -p xtask package_manifest --all-features`
- `bazel test //tools/xtask:tests //crates/bitaxe-api:tests`
- validate the committed OTA result and receipt paths and their recorded exact
  source/reference identities.

Mandatory repository verification is the ordered Rust sequence, managed Bright
Builds checks, `just test`, `just parity`, `just parity-progress`, redaction,
reference cleanliness, and `git diff --check`.

Promotion requires the row-specific result to bind all four identities: clean
package source commit, pinned reference commit, manifest-admitted package/OTA
artifact, and exact post-reboot runtime identities on the detector-admitted
Ultra 205. Evidence remains `workflow,hardware-smoke`. Any mismatch, stale or
indirect observation, privacy failure, invalid transition, or repository gate
failure leaves the row `implemented`. No hardware recovery path or retry exists
because this plan performs no hardware action.

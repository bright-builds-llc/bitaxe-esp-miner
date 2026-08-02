# V12-PACKAGE-IDENTITY-205 work result

- Parity row: `V12-PACKAGE-IDENTITY-205`
- Final status: `verified`
- Implementation commit: `2541818aa23120dd85c711386efadb69a1415ad3`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`

## Evidence and verification

The immutable [bounded OTA result](../20260802T223139Z-OTA-001-RETRY/RESULT.md)
binds the four identities required by this row:

- clean implementation and package source commit
  `2541818aa23120dd85c711386efadb69a1415ad3`;
- pinned reference commit
  `c1915b0a63bfabebdb95a515cedfee05146c1d50`;
- manifest-admitted package and `esp-miner.bin` SHA-256 values
  `ab63ff31c192cf81fea451d8b1fbe4f03068c8c004254a83d0c1cf7f5e6fdf0f`
  and `5317896017a0039ca08e6c4437dde81ba0ab5a9b25bda643e9925376551377de`;
- exact post-reboot implementation and pinned-reference identities observed
  through a qualified OS-native passive capture on the one detector-admitted
  Ultra 205.

The same result records successful wrapper flash admission, safe-state boot,
boot validation, cleanup detection, and redaction. Transition receipt
[`20260802T230503Z-OTA-001`](../../checklist-transitions/20260802T230503Z-OTA-001.json)
cryptographically binds that result to its immutable plan and checklist
predecessor. Commit `2541818aa23120dd85c711386efadb69a1415ad3` is an ancestor
of this result's repository state.

Focused verification passed:

- `cargo test -p bitaxe-api runtime_boot_attestation --all-features`: 11 tests;
- `cargo test -p xtask package_manifest --all-features`: 8 tests;
- `bazel test //tools/xtask:tests //crates/bitaxe-api:tests`: both targets.

These regressions reject stale package identity, wrong reference identity,
wrong ELF digest, mixed boot sessions/ordinals, non-monotonic observations,
incomplete readiness, malformed manifests, missing release artifacts, and
duplicate package artifacts.

## Conclusion

The admitted Ultra 205 ran the exact clean package whose manifest names the
same source and pinned reference commits, and qualified post-reboot evidence
observed those exact identities from the device. This directly resolves the
Phase 36 `runtime_identity_observation_insufficient` correction and supports
`workflow,hardware-smoke` verification for
`V12-PACKAGE-IDENTITY-205`.

## Non-claims and residual risks

This result does not promote or claim hostname durability, operator-snapshot
substance, runtime-health substance, selected-partition internals, rollback,
destructive or interrupted-update recovery, OTAWWW, network longevity, mining,
pool behavior, voltage/fan/power effects, other boards, direct UART, pin
manipulation, or release readiness. Private USB, serial, HTTP, device-origin,
network, and credential material remains under ignored roots and was not read
or copied for this reconciliation.

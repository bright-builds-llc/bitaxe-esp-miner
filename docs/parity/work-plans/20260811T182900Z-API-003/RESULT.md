# Parity work result

- Parity row: `API-003`
- Final status: `verified`
- Implementation commit: `3dea210228722634360daeda1327f2676e78db3a`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Board: Ultra 205
- Hardware attempts under this plan: one

## Evidence and verification

The sole detector admitted one Ultra 205 before the exact schema-v3 package
built from clean pushed implementation commit
`3dea210228722634360daeda1327f2676e78db3a` was flashed. The capture derived
one trusted same-session origin, read the baseline hostname and rotation,
submitted one atomic two-field system-settings PATCH, confirmed both values in
one immediate readback, restored both baseline values in one atomic PATCH, and
confirmed both together.

The committed
[`bitaxe-settings-patch-evidence-v1`](../../evidence/api003-settings-patch/settings-patch-projection.json)
projection has SHA-256
`6cad3810a4f0f5573c055141ff1ede6c6c629092816bd5d2147c6130a5f8c2d8`.
The repository-owned Rust validator independently accepts it. Its closed facts
prove:

- exact clean source, pinned reference, package-manifest identity, app ELF
  identity, and board 205 admission;
- one combined mutation request containing exactly hostname and rotation, with
  both generated candidate values confirmed together immediately;
- one combined restoration request containing exactly the two baseline values,
  with both confirmed together;
- mining and hardware control remained disabled, cleanup completed, all private
  roots and files used the required modes, and the admitted serial port has no
  remaining holder; and
- the public projection contains only hashes, counts, booleans, closed labels,
  and build/workflow identities and passes semantic redaction.

The first independent redaction scan exposed a naming-only collision: two
digest field names used `original`, which contains the denylisted substring
`origin`. No raw value was present. The public contract and projection were
renamed to `baseline` digest fields and a production-shaped regression was
added. The hardware capture was not repeated.

The exact effect-capable commands were the immutable plan's `just package`,
private `just detect-ultra205`, and conditional `just
capture-settings-patch-evidence` commands. Focused contract, orchestration,
failure-category, recovery, real-child-process, validator, and redaction tests
passed, followed by the complete ordered repository gate and the package,
selector, immutable-plan, task-uniqueness, privacy, mode, holder, reference,
and diff checks.

## Conclusion

The exact pushed Rust firmware accepts one production `/api/system` PATCH that
atomically changes two real benign settings on a detector-admitted Ultra 205,
projects both values together immediately, and atomically restores and confirms
both baselines while remaining in the required passive safety state. This
satisfies `API-003` with typed workflow, API-comparison, and hardware-smoke
evidence.

## Non-claims and residual risks

Raw detector, USB, serial, HTTP, configuration, hostname, rotation, origin,
network, credential, and process material remains only in ignored private
roots. The Wi-Fi credential file was an opaque local input, and no pool
credential was read. This result does not claim arbitrary settings, secret or
safety-control mutations, reboot or power-loss durability, repeated or
concurrent PATCH behavior, mining, ASIC work, hardware controls, network
reconnect longevity, OTA/recovery, other boards, or release readiness.

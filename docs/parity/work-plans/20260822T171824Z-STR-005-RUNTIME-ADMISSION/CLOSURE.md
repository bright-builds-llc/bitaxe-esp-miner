# Parity work closure

- Parity row: `STR-005`
- Final status: `implemented`
- Outcome: `blocked`
- Verification claimed: `no`
- Plan SHA-256: `0e7d4139b0186870509b0bb380cf35256b2f037a76c4c298966f2f8924b8c98e`
- Active task: `task-parity-str005-stratum-v2`

## Closure reason

Clean pushed source and exact package `24180a94` passed every mandatory
software gate. The no-effect preflight returned `pre_effect_ready` with both
effect/root booleans false and absent private/public targets. A fresh detector
then admitted exactly one Ultra 205.

The task-gated read-only runtime admission passed passive monitor process
completion, exactly one current runtime origin, same-origin system/theme reads,
and exact restoration-input reconstruction. It stopped at earliest category
`hardware_blocked`, checkpoint `restore_package`, because one exact restorable
package for the firmware currently running could not be selected from the
bounded local inventory. That inventory contains 71 retained package manifests,
but it does not establish one usable matching identity.

Attempt-004 was not consumed. Its private root and the public projection remain
absent. The command did not create a fixture, write settings/NVS, flash, contact
a pool, mine, initialize or control the ASIC, change voltage/fan/power, submit a
share, or publish evidence.

## Next safe action

Keep `STR-005` at `implemented`. Hardware testing requires recovery of the exact
currently installed package bytes and manifest, with its identity independently
matched before any device mutation. A deliberate new-baseline flash is not an
equivalent restoration proof and needs a separate safety/recovery decision; it
must not be inferred from the present authorization. Do not retry runtime
admission or attempt-004 unchanged.

## Non-claims

This closure does not verify a Noise handshake, V2 channel, ASIC work, target-
qualified nonce, encrypted share, accepted response, terminal safe stop,
settings/package restoration, external-pool interoperability, mixed-protocol
fallback, other boards, unbounded mining, OTA, or release readiness. It does
not create `RESULT.md`, hardware-regression evidence, or `verified` status.

# Noise v2 operator handoff

This implementation uses the frozen v1 contract and v2 parity amendment in
`docs/hardware/`. No hardware run is implied by the software tests. Preflight
requires completed archived runtime and fixture tasks, clean published sources,
the exact accepted cadence predecessor, and the derived native audit.

## Canonical preparation

Build the clean native package and
`//tools/stratum-v2-fixture:noise_serial_build_identity`. The sidecar binds the
canonical fixture binary to Bazel's source status. Then run the existing recipe:

```sh
just stratum-v2-noise-serial preflight \
  --private-root "$ATTEMPT_ROOT" \
  --firmware-root "$FIRMWARE_ROOT" \
  --gate-root "$GATE_ROOT" \
  --package-manifest "$FIRMWARE_ROOT/bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json" \
  --fixture-binary "$FIRMWARE_ROOT/bazel-bin/tools/stratum-v2-fixture/stratum_v2_fixture" \
  --attempt-ordinal 1 \
  --predecessor-receipt "$ACCEPTED_CADENCE_RESULT"
```

The parent directory must already be ignored and mode0700; the attempt child
must be absent. Preflight consumes the exclusive ordinal assignment before
creating the child. It snapshots thirteen runtime artifacts, evaluator sources,
fixture identity, contracts and native inputs. A partial creation never makes
its ordinal reusable. Initial positive admission is implemented; later ordinals
fail with `noise_retry_progress_unverified` until concrete failed evidence and
verified progress have a reviewed continuation. This is not a policy attempt
cap.

`serve` requires mode0600 redirected stdout because its URL is operational data.
Its output/log paths belong outside the initially absent attempt child during
preflight. The parent should launch an isolated process and retain its actual
exit event rather than infer exit from listener disappearance.

```sh
just stratum-v2-noise-serial serve --private-root "$ATTEMPT_ROOT" > "$PRIVATE_STDOUT"
```

## Same-page workflow

Keep the same loaded Gate page and private preservation baseline throughout. Use
native browser interaction for permission/Connect; do not synthesize a
permission gesture or adopt a new page baseline.

1. Connect with the initial before identity, then call
   `noiseSupervisor.recordAccounting("before-install")`.
1. Close the Worker and await `noiseSupervisor.flush()`.
1. The repo-owned `installCandidate(root, 0)` export in `operator.mjs` owns
   fresh detection, a prearmed process observer, server claim consumption, and
   the exact `just flash-monitor` invocation. It accepts only root/index,
   derives the target from detector output, and binds the child to the parent's
   original context and claim digests. It never accepts an arbitrary command or
   port.
1. Call `noiseSupervisor.configureCandidate()`, then natively Connect again.
1. For each index1–4: close/flush, await `installCandidate(root, index)`,
   reconfigure the same candidate, natively Connect, then await
   `noiseSupervisor.recordCycle(index)`. This creates a fresh probe ticket,
   awaits the actual maximum Gate probe and joins its result to the new session
   and published journal rows. Retained probe counts alone cannot pass.
1. Call `noiseSupervisor.run()` once. It records pre-job accounting, starts the
   fixture, obtains fresh possession/network evidence, consumes Start once,
   records the actual terminal, restores and closes the Worker. A lost or
   ambiguous Start is never resent.
1. Natively reconnect the same page; call `noiseSupervisor.restoreAndRecord()`
   for fresh retained-job/preservation proof and after accounting. This helper
   can record safe restoration of a failed retained job without promoting that
   job's outcome.
1. Close the Worker, await `noiseSupervisor.flush()`, close the actual browser
   tab, and stop the supervisor. Preserve every actual nonzero exit/failure.

The diagnostic uses no Work Lease signer, pool credentials, mining allowance or
mining route. `recover` is unsupported and returns `noise_recovery_unavailable`;
it never resets or reflashes a device.

## Parent cleanup and finalization

`observeOwnedExit(child, contextSha256, owner, "supervisor")` records an actual
owned child close event. Call `markStopRequested()` immediately before the
parent's normal stop signal; `receipt()` requires observed exit and a
same-parent hrtime interval no longer than five seconds. The owner must identify
that exact child. Do not label an unrelated wrapper's exit as the server's exit.

After actual browser closure, construct the closed parent browser witness from
that observation and the exact last journal row; this is the one external UI
fact the process observer cannot invent. Pass `{browser, supervisor}` to
`recordCleanup(root, context, input)` from `cleanup.mjs`. It derives the fixture
witness from the fixture owner's real close-event receipt, checks all observed
serial node paths and owned process/listener absence, and writes the exclusive
sibling `.cleanup` records. Natural fixture exit has no invented stop request.

Finalization requires stopped host writers even when the result is unverified.
It copies parent cleanup inputs before classification, preserves available
partial input digests, seals once, and only publishes a strictly admitted v2
projection on a complete pass. Existing public projections are never
overwritten.

```sh
just stratum-v2-noise-serial finalize \
  --private-root "$ATTEMPT_ROOT" \
  --cleanup-receipt "$ATTEMPT_ROOT.cleanup/receipt.json"
just stratum-v2-noise-serial review --private-root "$ATTEMPT_ROOT"
```

The reader revalidates frozen membership, protocol/cycle/accounting/restoration
joins and the public projection. Old v1 schemas and retired historical effect
commands keep their previous meanings. Software-only fixtures, borrowed-worker
native stack checks and package fit do not establish hardware qualification.

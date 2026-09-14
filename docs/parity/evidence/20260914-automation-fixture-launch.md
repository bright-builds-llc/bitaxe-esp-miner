# Automation fixture launch investigation

The Stage-B verification run exposed another host execution-policy wait. Five
automation tests timed out while launching freshly written executable fixtures.
An isolated canonical rerun also failed. These failures occurred before any
Stage-B installation, restart or mining.

## Evidence

Two sampled children remained at `_dyld_start` with no JavaScript frames. Kernel
logs join their exact PIDs, fixture path and timeout events to AppleSystemPolicy
sleep interruptions (references 1338509 and 1338510). The subsequent non-allow
lines followed cancellation; they are not evidence of a completed earlier
denial. Unrelated numeric and provenance-token matches were excluded.

The same fixture bytes completed through explicit Node invocation. With the
Bazel-pinned Node binary, spawn-to-exit was 20.966 ms and owned cleanup completed.
The original sandbox interpreter alias had expired after the test, so this
comparison used the same pinned toolchain through its stable runfiles alias.

| Artifact                        | SHA256                                                             |
| ------------------------------- | ------------------------------------------------------------------ |
| Protected investigation receipt | `e1d95ca65a91130ed269c5e8be43cd6ed0b46af7a1b3abbfd7127b48098761ca` |
| Unchanged fixture source        | `09eb6c0197b9dd80a2bcf1e2b70904b2c1c337cccd1be766c07257757882fdbc` |
| Pinned Node executable          | `ee6fb0e015284d83a91e8ec5213f43a157f8a392b58555301682892ba928c04a` |

Private evidence is under `scratch/reset-origin-20260914`. The receipt binds
samples, narrowly reviewed host logs, failed canonical output and the explicit
interpreter comparison. No security configuration, quarantine, signatures,
credentials or acceptance timeout was changed.

## Correction and limits

The test-only `createFixtureProcessPort` maps each registered fixture path to
its explicitly selected Node or shell interpreter, then delegates to the
unchanged production process adapter. File extensions do not choose the
interpreter: one existing `.mjs` fixture contains shell source. Unregistered
native programs retain their original launch behavior.

Real child execution, arguments, environment, result parsing, exit status,
timeout and operator-gated lifetime remain covered. This removes an unnecessary
fresh executable-script launch from synthetic tests. It does not repair macOS
or establish reliable execution of every future native binary. Clean package
and hardware admission checks remain required.

Strict production/test TypeScript compilation and all 507 canonical automation
tests pass, including six explicit-interpreter regressions. No tests were skipped
and no timeout was increased. The canonical test execution completed in 63.9
seconds. No hardware qualification result is inferred from these host tests.

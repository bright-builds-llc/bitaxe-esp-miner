# Prove Ultra 205 Noise authentication

- Run ID: `20260829T143226Z-STR-005-NOISE-AUTH`
- Parity row: `STR-005`
- Initial status: `implemented | unit,golden,workflow`
- Source base: `607336a60bc22180e5eb636b222be932b1bfd2df`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-str005-noise-auth-205`
- Prerequisite: accepted TCP projection
  `docs/parity/evidence/str005-tcp-payload/tcp-payload-projection-009.json`

## Objective

Re-prove the exact Ultra 205 TCP connection, complete authenticated Noise NX
against the configured local authority, and deliver one encrypted
diagnostic-only frame. Restore recovery-006 exactly and publish only an
independently validated closed projection.

This child does not open a channel, receive a target or job, touch the ASIC,
submit a share, mine, change fan or voltage control, contact an external pool,
or promote STR-005.

## Interfaces and evidence

Add `just stratum-v2-noise-auth preflight|start|recover|finalize`. Diagnostic
ordinal `N` maps to `scratch/str005-noise-auth/diagnostic-NNN` and
`docs/parity/evidence/str005-noise-auth/noise-auth-projection-NNN.json`;
recovery ordinal `N` maps to `scratch/str005-noise-auth/recovery-NNN`.
`preflight` creates no private root and performs no diagnostic network or flash
effect. `finalize` performs no hardware effect and may join a complete
diagnostic to a separately accepted recovery result.

Use a distinct consume-once NVS case `noise_auth_v1`. The sole admitted owner
must run before production Stratum owners and exclude every mining and
hardware-control owner.

The public schema is `bitaxe-stratum-v2-noise-auth-projection-v1`. Build it from
an explicit allowlist of safe provenance/evaluator digests, closed stages,
bounded timings and counts, tuple identity, act-one transfer, authenticated
Noise, exact encrypted proof, restoration, cleanup, non-effects, and
`redaction_status`. Raw ports, addresses, endpoints, credentials, authority
material, certificates, Noise acts, encrypted bytes, timestamps, and device
identifiers remain in memory or protected mode-`0600` artifacts only.

## Firmware and fixture behavior

Reuse the prepared-Noise order: resolve, prepare Noise and exact 64-byte act
one, connect, configure, send, read act two, authenticate, then send one
reserved diagnostic frame with extension `0xffff`, message `0xff`, and an empty
payload. Require a configured authority key. Use the standard Rust TCP stream,
`TCP_NODELAY`, bounded read/write timeouts, exact byte accounting, flush, and
closed socket-error categories.

Record and replay the private local ephemeral port plus every closed stage,
timing, byte count, socket category, and terminal state every five seconds for
the existing 120-second observation window. No raw cryptographic value is
logged.

Add a dedicated fixture mode that accepts at most three exact-peer candidates,
polls them concurrently for at most ten seconds, rejects a fourth as
`candidate_overflow`, selects only a candidate delivering exact 64-byte act
one, produces signed act two, decrypts the client proof, and requires the exact
reserved diagnostic frame. More than one exact-peer connection is a failure
even when one authenticates.

The supervisor privately joins firmware local port to fixture remote port.
Acceptance requires one exact-peer connection, consistent tuple match, exact
act one, act two sent, configured-authority authentication, exact encrypted
proof, accepted firmware and fixture terminals, exact recovery-006 restoration,
and cleanup.

## Recovery and hardware contract

Before any effect, require the committed public recovery projection at mode
`0644`; private inputs at `0600` and roots at `0700`; exact restore bundle,
receipt, and source lineage; a contained non-symlinked `esptool.py`; importable
managed NVS Python; and exact `restore-installed --admission-only` acceptance.

After separate clean plan and implementation commits pass every gate and are
pushed, package exact HEAD, run fresh `just detect-ultra205`, run the exact
ordinal-001 no-effect preflight, then run ordinal-001 once:

`just stratum-v2-noise-auth preflight --board 205 --port <detector-port> --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --restore-bundle scratch/str005-installed-package-recovery/recovery-006/restore-bundle.private.json --private-root scratch/str005-noise-auth/diagnostic-001 --projection docs/parity/evidence/str005-noise-auth/noise-auth-projection-001.json --plan docs/parity/work-plans/20260829T143226Z-STR-005-NOISE-AUTH/PLAN.md --diagnostic-ordinal 1 --redact-evidence`

`just stratum-v2-noise-auth start --board 205 --port <detector-port> --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --restore-bundle scratch/str005-installed-package-recovery/recovery-006/restore-bundle.private.json --private-root scratch/str005-noise-auth/diagnostic-001 --projection docs/parity/evidence/str005-noise-auth/noise-auth-projection-001.json --plan docs/parity/work-plans/20260829T143226Z-STR-005-NOISE-AUTH/PLAN.md --diagnostic-ordinal 1 --redact-evidence`

If inline restoration is incomplete, run fresh recovery-only root
`scratch/str005-noise-auth/recovery-001`, prove exact recovery-006 identity and
settings, `mineonboot=false`, inactive zero-work/share state, fresh admission,
and zero owned processes, then finalize without rerunning the diagnostic.

Continuation is signature-bounded. A new ordinal requires a real-boundary
regression, targeted fix, every verification gate, clean commit/push, rebuilt
package, fresh root, and fresh detector. Fix connection ownership for tuple
mismatch/multiple connections; the demonstrated standard-stream/fixture seam
for partial act one; responder/read ownership for missing act two; only the
typed Noise completion boundary for authentication failures; and encrypted
frame handling for proof failure. Do not introduce direct lwIP speculatively.
Any incomplete restoration stops diagnostic hardware until recovery is exact.
Repeating the same authoritative post-fix signature is terminal.

## Verification and completion

Use red-to-green vertical slices at the NVS admission, firmware transcript and
transport, real fixture process, supervisor projection/validator, and
recovery/finalizer seams. Cover one valid connection, silent and multiple
candidates, unexpected peer, overflow, partial/EOF/timeout/I/O act one,
malformed act one, act-two failure, proof mismatch, tuple joins, duplicate
markers, raw-field exclusion, evaluator drift, and recovery/finalizer joins.

Before each commit or hardware ordinal, run `cargo fmt --all`, strict
all-target/all-feature Clippy, all-target/all-feature build, all-feature Cargo
tests, Bright Builds, all Bazel tests, canonical package, parity, progress,
redaction, reference cleanliness, whitespace, and final diff review.

On acceptance create `RESULT.md`, archive only
`task-str005-noise-auth-205`, and leave STR-005 at
`implemented | unit,golden,workflow`. The channel/job child becomes eligible
but remains separate.

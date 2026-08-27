# Diagnose STR-005 Noise interoperability without mining

- Run ID: `20260826T210025Z-STR-005-NOISE-DIAGNOSTIC`
- Parity row: `STR-005`
- Initial status: `implemented`
- Source base: `ba096f51116858ae34aee0d83c6a64cfccb392fa`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-str005-noise-handshake-diagnostic`
- Continues: `docs/parity/work-plans/20260826T135721Z-STR-005-INACTIVE-RESTORATION/CLOSURE.md`

## Objective

Replace the ambiguous repeated `transport/handshake` plus fixture `noise`
boundary with one authoritative closed Noise signature while the Ultra 205
performs no ASIC, voltage, fan, or mining work. Prove authenticated local Noise
or identify the exact failing completion boundary, then restore the exact
recovery-006 firmware and attempt-004 settings.

This is diagnostic work only. It is not campaign attempt-008 and cannot promote
STR-005.

## Typed Noise diagnostic core

Extend the initiator with a diagnostic completion result whose closed failures
are `message_length`, `decrypt`, `public_key`, `certificate_time`,
`certificate_signature`, `state`, and `other`. Preserve the stable production
error surface by mapping diagnostic failures back to its existing public error,
while the diagnostic owner retains the exact closed category.

Use the signed certificate interval and the supplied device timestamp to
distinguish certificate time from certificate signature without emitting key,
certificate, frame, clock, endpoint, or decrypted values. Cover every category
with deterministic pure tests where the dependency permits construction.

## Boot-time no-mining owner

Admit one run only when private NVS keys `sv2diagkind`, `sv2diaglease`, and
`sv2diagcase` form the exact task marker. Consume all three before use and reject
missing, malformed, partial, or replayed markers. When admitted, a sole
diagnostic owner replaces production Stratum V1/V2 and the normal fan owner,
waits for configured Wi-Fi, then performs only TCP connect, Noise act one, Noise
act two, initiator completion, and one encrypted client-authentication proof.

Retain closed runtime stages `tcp_connected`, `act_one_created`, `act_one_sent`,
`act_two_received`, `time_sampled`, and `authenticated`. Retain one terminal
category from `accepted`, `resolve`, `connect`, `configure`, `rng`, `act_one`,
`act_one_write`, `act_two_read`, `clock_before_epoch`, `clock_overflow`, or one
typed completion failure. Do not add an HTTP mutation API.

The diagnostic path must not initialize, lease, configure, reset, or work the
ASIC; enable or change core voltage; start production mining; start the normal
fan controller; open a channel; consume a job; search a nonce; or submit a
share. Regression tests prove ordinary boots are unchanged when the marker is
absent or consumed.

## Handshake-only fixture and workflow

Add a handshake-only fixture mode with protected progress facts
`listener_ready`, `connection_accepted`, `act_one_received`,
`responder_created`, `act_two_created`, `act_two_sent`, and
`client_authenticated`. Mark `client_authenticated` only after decrypting the
post-handshake client proof. Preserve child deadline, output-limit,
process-group termination, and cleanup as typed terminal facts.

Add `just stratum-v2-noise-diagnostic preflight|start`. The first ordinal uses:

- private root `scratch/str005-noise-diagnostic/diagnostic-001`;
- public projection
  `docs/parity/evidence/str005-noise-diagnostic/noise-diagnostic-projection.json`;
- the local same-subnet authenticated fixture;
- the canonical exact package from clean pushed HEAD;
- the existing ignored Wi-Fi input; and
- recovery-006 plus the attempt-004 backup for exact final restoration.

`preflight` is no-effect and proves clean pushed implementation, canonical
package and plan identity, accepted recovery readiness, exact local credential
inventory, fresh one-board detection, safe current runtime, fresh protected
root absence, and executable child contracts. `start` repeats admission, starts
the fixture, writes the exact package plus one-shot diagnostic marker, observes
the terminal receipt, stops the fixture, restores recovery-006 once through the
managed typed executor, seeds Wi-Fi separately, restores settings/theme in
memory, and proves exact original identity/configuration, `mineonboot=false`,
inactive `paused` or `safe_blocked`, zero work/shares, USB cleanup, and no owned
process.

## Evidence, continuation, and stop policy

Private roots are mode `0700`; private files are mode `0600`, regular,
non-symlinked, contained, and secret-sanitized before first write. Public
evidence contains only closed categories, booleans, bounded counts/durations,
safe provenance, artifact digests, restoration, cleanup, and redaction status.
It never contains USB or network identifiers, credentials, settings values,
logs, keys, certificates, frames, timestamps, flash bytes, or endpoints.

The first eligible hardware run is diagnostic ordinal 1 after focused tests,
ordered Cargo gates, Bright Builds, all Bazel tests, canonical build/package,
parity/progress, redaction, reference cleanliness, selector lineage,
sensitive-value review, final diff review, clean push, no-effect preflight, and
fresh detection.

If a new authoritative signature identifies a correctable repository defect,
add a real-boundary red/green regression, fix it, repeat every gate, commit and
push, rebuild, exactly restore if necessary, and use a fresh root and ordinal
under this same plan. Never retry unchanged or reuse a sealed root. Stop on a
repeated post-fix signature, unresolved partial transfer, hardware blocker,
authority boundary, impossible evidence contract, or inability to restore the
exact original state.

On authenticated Noise plus exact restoration, independently validate the
projection, create `RESULT.md`, archive only this diagnostic task, final-verify,
commit, and push. STR-005 remains `implemented`; channel/job/share verification
requires a separate formal campaign plan.

## Non-claims

This plan makes no claim for channel opening, target/job receipt, BM1366 work,
nonce qualification, accepted shares, production pool interoperability,
external pools, mixed-protocol fallback, other boards, direct UART/pins, raw
NVS/coredump access, fault injection, OTA, erase, arbitrary writes, unbounded
mining, STR-005 verification, or release readiness.

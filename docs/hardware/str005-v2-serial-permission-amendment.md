# V2 pre-open permission failure amendment

Contract ID: `str005-v2-serial-permission-v1`.

This prospective amendment supplements the immutable
[base contract](str005-v2-serial-qualification.md) and
[reservation-clock amendment](str005-v2-serial-clock-amendment.md). It defines
one guarded closure and successor for the observed Channel001 preparation. It
changes no firmware protocol, mining allowance, safety limit, positive judge or
accepted historical result.

## Observed boundary and non-claims

Channel001 bound the uninstalled candidate firmware source
`979f7ba23881ba0921ebf726dd34f08cf4405acf`, Gate
`9251a1f1fbacb35093db0dc0a75a1a87206abcb0`, and context digest
`1eca9c340cec2786933552b2e549eff5536ca751c3192b17f431e98bdded1aae`. The first
recorded failure is `v2_browser_failed`, referencing state four:
`connect_failed`, serial category `operation_failed`, admission stage
`permission`. Six states remain in the before phase; the final state is closed
with released browser ownership. An earlier configured state optimistically
marks ownership unreleased while requesting permission; that is not proof of an
opened port.

The exact published Gate source maps both rejected selection and selection
followed by foreground/generation loss to this permission-stage outcome. Neither
path calls Serial open, creates a channel, sends Hello or activates the
qualification scope. Therefore closure may prove no application admission and no
device-control effects from this preparation. It must not claim that no port was
selected, no browser grant existed, or that a fresh device baseline, identity,
settings or ledger was measured. Read-only USB detector inspection did occur.
The native foreground window observed after rejection was not the qualification
window; the precise browser exception was intentionally not retained.

Preserve this failure and all existing bytes. Never reinterpret it as a passed
channel exchange, collected device summary or mining attempt.

## Targeted Connect correction

For the V2 qualification page only, the Connect DOM handler synchronously
requires a trusted click, visible document, focused document and active user
activation before invoking existing Connect in that same event task. A missing
precondition makes no permission request, opens no port, changes no controller
state, and leaves only a local allowlisted precondition notice. It is not a new
attempt failure or admission, and cannot extend any device authority.

The operator must bring the owned qualification page into the actual native
foreground, verify that window/page, and use a native Chrome gesture for Connect
and chooser selection. DOM focus is an additional signal, not proof of OS window
identity. Do not synthesize activation, enable focus emulation, add sleeps
inside the click path, wrap or weaken requestPort, use a getPorts fallback,
automatically select permission, or change the ordinary controller/SDK API.

Software tests cover each rejected precondition, immediate valid invocation,
ordinary behavior, and both indistinguishable permission-failure branches with
zero opens, frames and scope activations.

## Effect-free closure and review

Add these actions to the repo-owned `just stratum-v2-serial` family:

- `close-permission --private-root FAILED`: verify the exact failed preparation
  and actual host release, then exclusively create the protected deterministic
  sibling `FAILED.permission-closure.json`.
- `review-permission --private-root FAILED`: independently revalidate that
  receipt and all bound bytes, read-only.

Neither action opens hardware, obtains credentials, invokes a signer, modifies
an existing result or acquires continuation authority. Output remains outside
the failed preparation. Never rewrite its source snapshot, journal, failure,
artifacts or consumed host marker. Reject aliases, symlinks, unsafe modes,
noncanonical paths and existing/conflicting closure output. A failed write may
not be represented as a completed receipt.

The classifier must independently verify:

1. The v1 context, consumed Channel001 assignment, exact published failed pair,
   all thirteen artifact snapshots, native/source/preflight inventories and the
   accepted Noise predecessor chain.
1. Exact before-phase chronology: configured permission preparation, the sole
   permission/operation_failed admission rejection, immutable first failure
   pointing to state four, closing and closed states. No connected/running
   state, device baseline, restoration confirmation, inactive-lease observation,
   renewal or heartbeat suppression may have been observed.
1. No device journal, installation/flash/cycle/probe record, accounting review,
   fixture/observer owner, Start claim, issuance, grant delivery, work or later
   phase evidence. Reject unknown inventory membership and conflicting causes.
1. The actual browser-closure witness joined to the final state, and actual
   parent-observed supervisor identity/zero exit within five seconds. The saved
   parent script's baseline-requiring browser branch did not emit this witness;
   preserve the separate parent/CUA observation provenance.
1. Host resource absence: stopped owned supervisor, no supervisor listener and
   no holders of either native serial node. Verify the protected recorded
   observations and repeat read-only process/listener/holder inspection while
   creating closure. Read-only review verifies original observations and sealed
   hashes without obtaining fresh device authority.
1. Exact failed-root inventory and every supporting sibling/operator artifact,
   including the initial detector observation, launch logs and host assignment.
   Seal hashes and membership within the failed tree and operator tree, plus the
   individually named supporting siblings. Do not inventory the shared parent
   namespace: new successor files there are legitimate. Altered or added bytes
   within the sealed evidence scope invalidate review.

The receipt records an **unverified preparation**, its exact first observation
and supported boundary, hashes, cleanup provenance and explicit non-claims.
Device accounting is **not collected**. The accepted predecessor supplies only
the expected next ordinal 18 and 1560000-ms charged total for fresh admission.
There is no charge, refund, reset or accounting-success claim from this closure.

## Exclusive successor admission

Add optional `preflight --supersede-permission CLOSURE` for Channel only. Accept
exactly Channel002 superseding the observed Channel001. Reject Channel003,
recursive lineage, Share supersession, generic force/retry flags, duplicate or
conflicting assignments, altered evidence, unchanged pairs, missing correction
or archived-task authority.

Require independently reviewed closure, a changed clean published pair and
source-bound verification of the targeted Gate correction. Retain the same
accepted Noise predecessor as the protocol ancestor: the failed preparation is
only supersession evidence. Bind the amendment, firmware, Gate, clean package,
fixture, observer, validator and correction identities before assignment.
Reserve Channel002 exclusively before any effects; interrupted assignment stays
consumed. Closure alone is not a bearer permission or effect permit.

Preserve the exact closed v1 context reader for historical inspection. New
contexts use `str005-v2-serial-context-v2`: the existing context fields plus
`permissionSupersession`. Channel002 requires that field to be the closed object
`{failedRoot, failedContextSha256, closurePath, closureSha256}`. Both paths are
canonical and `closurePath` is the deterministic sibling of `failedRoot`; both
digests are SHA256. Share001 requires `permissionSupersession: null`. No other
new live Channel ordinal or nullable supersession variant is admitted.

The contract binding retains `base` and `amendment` (the clock amendment),
adding `permission` for this exact published document. Before exclusive
assignment, the repo-owned preflight runs the fixed effect-free Gate command
`bun test ./web/worker-qualification-gesture.test.ts ./web/worker-serial-admission.test.ts`
against the clean published Gate checkout, with a 30-second timeout and bounded
output. It never accepts a caller-supplied test verdict. Only zero exit with no
signal passes. Bind the observed correction result as
`permission-correction.json` in the new preparation's preflight inventory, using
the closed fields `schema`, `firmwareCommit`, `gateCommit`, `gateBundleSha256`,
`checkerSha256`, `command`, `exitCode`, `signal` and `elapsedMs`. The schema is
`str005-v2-permission-correction-v1`; the command is the exact argument array
above, exit code is zero, signal is null and elapsed milliseconds are a
nonnegative integer no greater than 30000. Do not retain raw test output. Share
inherits this proof through its accepted same-pair Channel002 ancestor; it gains
no separate supersession. The live workflow retains task, source/publication,
failure/seal and artifact gates. Historical preparations remain unavailable for
effects.

Share remains host ordinal one, with no permission-supersession authority of its
own. It derives the accepted Channel002 pair through the normal predecessor
path, performs its own four fresh cycles and uses the existing normal
reservation at expected device ordinal 18. Fresh authenticated accounting before
installation and before issuance must prove the expected state; discrepancies
stop admission. No ledger reset, refund or prior cycle/baseline transfer is
permitted.

## Verification and continuation

Test realistic immutable failure fixtures, both permission-stage branches,
partial/interrupted closure and assignment, altered artifacts or membership,
unsafe paths/modes, conflicting failures, any actual serial/effect evidence,
incomplete cleanup, wrong predecessor, unchanged pair, missing correction,
archived-task rejection and all duplicate/recursive/Share retry variants.
Preserve positive judges and existing historical readers unchanged.

Publish this amendment before implementation. Publish the verified correction
before creating/reviewing closure and admitting the new context. Run ordered
Cargo, Gate/browser, canonical, native package/resource, ownership, reference,
redaction, task-boundary and parity checks. On complete implementation, archive
only this amendment task; the main qualification task retains both live scopes.

The successor uses every original channel/share acceptance limit and cleanup
requirement. Another failure remains unverified and cannot reuse a consumed
context. This amendment creates no general retry class. Parity remains 90/95
until the separately governed full evidence-promotion task passes.

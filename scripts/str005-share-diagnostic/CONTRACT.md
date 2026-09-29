# Focused Share001 diagnostic image qualification

Owner: `task-str005-v2-accepted-share-probe`. This new owner has no authority until
its individual compiled stage flag and exact active-task stage line are enabled
in clean committed and pushed source. Existing consumed panic, Renew, startup and
share gates stay disabled. No grant, signer, mining fixture, Start, renewal or
heartbeat fault is available here. Parity remains 90/95 and earlier results remain
unchanged.

## Fixed evidence and current measurements

Independently verify all original inventories and producer artifacts:

- Share001: `scratch/str005-share/share001/attempt`, seal
  `1125f5d0dea1fa310ad3001cfa3c2aaa775924aacd1240a68555021ce559261e`.
- Recovery005: `scratch/str005-share-recovery/recovery005/attempt`, seal
  `49abc244d0efe12af30a7265cd5661dea0e735fa78baa1a7ff30b279cf3f39a6`.
  Rejudge actual state, diagnostics, status, ledgers and original collection time;
  its historical timestamp never becomes fresh authority.
- Original full core: `scratch/str005-share-crash/acquisition001/attempt`, seal
  `3ec746b255da28290d8f1944e1cc375f7deea559f8d936bd49dea8e5ed6ad39b`, SHA-256
  `b665c154d35f43bf7a0ab9acfe0aab207ac9e36c6a1a10ce649f3863f880f29c`.
  Verify acquisition result, full partition table integrity and all 974,848 bytes.

Before-image is `f000872f2e436aa7cdaa8cbfa41eee965a27731e`, ELF
`a3e257418d625aef5e9423fe28f598377f92b092b634264d3cc1e86b4e83e3c2`;
Gate is `9643e87664397a321c715a3a1b1bb6c1183b83ea`. The package for a new
installation must identify the current clean pushed HEAD. Native cutoff, store,
Start, Renew and new provenance audits must all identify that exact ELF. No
candidate ELF hash is guessed or copied from a development build.

## Publication and command surface

Compiled stage flags default false. For each explicitly admitted stage, publish
its own exact line in the single active accepted-share task:

```text
Share diagnostic recovery hardware: enabled.
Share diagnostic clear hardware: enabled.
Share diagnostic installation hardware: enabled.
Share diagnostic capture hardware: enabled.
Share diagnostic archive-clear hardware: enabled.
```

Only enable stages covered by the current published attempt. Recovery and clear
may need to be published together so a fresh proof retains the same host-source
identity. Generic core tools still require their separate accepted-share task
markers; enabling this owner does not enable those tools implicitly.

All roots are fresh absolute ignored paths under a private 0700 parent. Logs and
artifacts are 0600. `serve` requires separate protected stdout/stderr files.
Freeze source, contract, pinned Gate assets and package while a device owner is
active. Every effect rechecks those bindings. Historical artifact validation never
runs holder checks against old serial node names; current admission and cleanup
check the actual freshly detected device.

## Stage 1: recover, preserve and clear the old core

```text
just str005-share-diagnostic preflight --stage recovery --private-root <new-recovery> --gate-root <Gate>
just str005-share-diagnostic serve --private-root <new-recovery>
just str005-share-diagnostic finish --private-root <new-recovery>
```

Save a \<=60-second same-physical runtime detector as the parent's
`detector.stdout.log`; require no serial holder. Use the shared direct-browser
collector: authenticate the exact before image, measure current idle status,
ledger/original budget, diagnostics and preservation, then independently Stop and
Close. Ready and restored `baseline_confirmed` states are admitted only with the
same full preservation/inactive-authority checks. Proof time is actual collection
begin and its maximum age is 120 seconds. No ordinal is assumed.

Terminate the browser/server owner and save fresh `cleanup-detector.stdout.log`
before finalization. Seal only actual complete recovery; missing restoration,
resources or evidence remains a blocker.

```text
just str005-share-diagnostic preflight --stage clear --private-root <new-clear> --gate-root <Gate> --recovery-root <fresh-sealed-recovery>
just str005-share-diagnostic clear --private-root <new-clear>
just str005-share-diagnostic finish --private-root <new-clear>
```

The clear wrapper invokes only official `core-dump-clear`, with exact old image,
physical identity, fresh rederived proof and immutable original archive digest.
Save fresh `clear-detector.stdout.log` in its parent. The generic tool rereads and
compares the real core bytes, erases only the admitted core partition, checks
all-FF readback, returns to the exact installed image, and releases its session.
The proof is consumed once even on failure. No other partition, NVS value,
identity or replay state may be erased. A differing core blocks the effect.
The core child runs inside an independently identified process group. Its small
repository supervisor waits for the durable PID/PGID/start-time claim before
launching the core tool; actual worker identity is retained separately. The
supervisor stays alive until the parent terminates the entire owned group, records
the worker exit and verifies group absence. Private logs cannot outlive that
managed group unnoticed.

Finalization cannot seal while the parent, worker or group is alive or unproved
gone. It also requires a new same-physical runtime `cleanup-detector.stdout.log`
and independent holder absence on both admitted and returned serial nodes.
Failure to prove either boundary returns an unsealed blocker; no writer is sealed
out from under a live operation. A failed producer retains its original first
failure separately from any current cleanup failure. Later observed group absence
can permit a failed partial seal while the earlier failed reap result remains
explicitly recorded; it never turns that operation into a pass. Failed
producer/return/readback outcomes remain failed and cannot admit installation.

## Stage 2: state-preserving installation

```text
just str005-share-diagnostic preflight --stage installation --private-root <new-install> --gate-root <Gate> --manifest <canonical-package> --clear-root <verified-clear>
just str005-share-diagnostic serve --private-root <new-install>
just str005-share-diagnostic install --private-root <new-install>
just str005-share-diagnostic finish --private-root <new-install>
```

Establish a **new before baseline after the clear/reset**, then preserve that same
page baseline across this installation. Do not claim preservation across the
separate earlier page. Installation requires a fresh proof observed strictly
after the verified clear exit. Save another \<=60-second detector as
`install-detector.stdout.log` immediately before effect admission.

For installation, `--manifest` must use the canonical repository output path
`<repo>/bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json`. A private copied
manifest is rejected even when byte-identical. Private package copies are retained
archives for historical identity, decoder inputs and later read-only evidence;
they do not replace the canonical installation manifest path.

This collector records `ready` **before** Stop. A fresh boot may truthfully report
`deviceRestorationConfirmed=false` because restoration is `not_required` while
its safe baseline, inactive lease and preservation are confirmed. That pre-Stop
receipt never claims a completed restoration. Actual Stop and Close must produce
the later confirmed-restoration/closed receipt before any current proof or
installation admission. The separate Stop-first recovery collector keeps its
stricter `baseline_confirmed` policy unchanged.

The shared installer uses the canonical manifest, exact physical lease, guarded
ROM admission, disjoint state-preserving segments, 360-second capture and the
existing 1,200-second outer bound. No factory reset, credentials provisioning,
rollback or implicit retry. Candidate configuration requires real installation
receipts; current authenticated candidate recovery must verify exact runtime
identity, unchanged ledgers/budget and same-page preservation before sealing.
Capture is independently disabled during installation.

## Stage 3: separately admitted capture

After a successful sealed installation, recovery preflight can select it with
`--installation-root <sealed-installation>`. Its exact producer artifacts and
candidate recovery are rejudged; the new image is never inferred from a success
flag alone. Collect a new current recovery, acquire the actual core into its
`installed-core` child with the official separately admitted reader, and preserve
an empty region before sealing it. A nonempty region requires diagnosis and a
separate archive-bound disposition; never overwrite it with a self-test.

```text
just str005-share-diagnostic preflight --stage capture --private-root <new-capture> --gate-root <Gate> --installation-root <sealed-installation> --recovery-root <sealed-empty-core-recovery> --retained-manifest <private-retained-manifest>
just str005-share-diagnostic serve --private-root <new-capture>
just str005-share-diagnostic finish --private-root <new-capture>
```

Capture requires `--retained-manifest`, pointing to the private archive of the
exact installed package. Its manifest digest, installed source/reference identity,
all artifact digests and full ELF digest must match the sealed installation.
The ELF path is derived from that validated archive and all five audits run on
those bytes. A later host commit does not require rebuilding or reinstalling the
firmware; mutable `bazel-bin` outputs are not capture inputs. This option is
rejected for installation, which still requires the canonical manifest path.

The shared capture owner revalidates actual empty-core lineage, fresh paired
baseline diagnostics, all exact-ELF audits, bounded current store readiness and
one ASIC-off self-test. Claim, fault and reads remain bounded; Stop/Close remain
independent. Preserve immediate panic/store observations before any later ROM
entry. Core acquisition, offline checksum/full-ELF/cutoff/provenance inspection
and subsequent clearing remain separate explicitly published operations. This
initial owner does not enable a later bounded mining reproduction.

The capture finalizer rechecks the official read claim, full partition geometry,
store receipt, panic source and boot, and the exact archived ELF. It reruns the
bounded provenance decoder and batch debugger in new private child roots before
sealing. Both must succeed, including a meaningful original panic frame. A
decoder whose process group cannot be proved released leaves an unsealed result;
other missing evidence seals an explicitly incomplete capture. Raw dumps,
registers, stack and debugger text remain private.

## Stage 4: archive-bound captured-core clear

The independently published `archive-clear` stage consumes a successful sealed
capture and a new current installed-image recovery. Its arguments are exactly
`--private-root`, `--stage archive-clear`, `--gate-root`, `--capture-root` and
`--recovery-root`. The adapter rejudges the captured raw partition, decoded
original frame, official read, full ELF, current identity and preserved digest.
It uses the existing bounded clear supervisor and official `core-dump-clear`
command. The old `clear` stage remains disabled and continues to require the
original dump hash. Before the effect, both the stage flag and the generic
archive-bound core-clear marker must be enabled in the active task. Verify the
exact-image return, full erased readback, child/group release and fresh detector
and holder absence before sealing. A separate fresh post-clear recovery is
required before any Start attempt. Capture, mining and grants remain disabled.

## Stop conditions, retries and verification

One command root and immutable claim per admitted attempt. Any changed source,
identity, archive, pending ledger, missing restoration, ambiguous owner or failed
audit stops dependent effects. Close/recovery evidence remains collectible after
failure. A retry requires demonstrated progress and a new published attempt;
never mutate a sealed root or relabel old failures. No historical release or
cause is inferred from current recovery.

Run the new stage/gate/argument/clear-boundary tests and all shared panic-owner
production-boundary tests before enabling. Native build and all five exact-image
audits must pass on the clean published candidate. Hardware capture alone never
establishes panic cause: require independently decoded original frame and bounded
allocation history, preserving any unavailable or invalid fields explicitly.

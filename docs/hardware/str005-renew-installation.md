# State-preserving Renew correction qualification

Owner: `task-str005-v2-accepted-share-probe`. Software preparation only until the
compiled successor gate and exact task stage are enabled in clean published
source. This prospective contract does not reopen consumed panic/startup trials,
replay Share002, or assert a cause for its panic. Parity remains 90/95.

## Scope and exact inputs

The sole predecessor is `scratch/str005-startup/startup003/attempt`, with sealed
inventory SHA-256 `cc5bab7bd34da6f3b50388c31f5e1fc4aefb6a7faf12bd14678a6dc08593e7a1`.
Verify every sealed file before using its actual installed source/ELF, known
attempt ID and worker generation. Its observed boot was 9 and its completed
ledger was next20/last19/1,920,000 ms/pending=false. These are comparison inputs;
every effect requires fresh authenticated measurements, not copied observations.

The candidate is the clean published package supplied to `renew-preflight`.
Preflight binds its complete manifest, exact source/ELF and flash binary; audits
both selected signed-Start and signed-Renew paths on those ELF bytes and requires
`selected_path_with_headroom`. Native panic-cutoff and core-store audits run on
that same ELF. Candidate source and ELF are frozen in the private context before
any device effect; a changed source, manifest, binary, Gate pin or contract
invalidates admission. The owning task publishes this canonical build selection;
preflight records and verifies the actual pair before device access.

No grant, reservation, Start, renewal, mining, heartbeat suppression, external
pool, factory reset, NVS provisioning, historical replay or parity promotion is
available in this command. The separate share command needs its own contract.

## Stages and publication

The shared `str005-panic-probe` command has explicit `renew-preflight`,
`renew-serve`, `renew-install` and `renew-finish` actions. Original actions retain
the original task gate. There is no force flag or automatic fallback to another
owner. `RENEW_ENABLED` and exactly one active accepted-share task are required.
Task sentinels are exact full lines:

- `Renew image qualification: baseline enabled.`
- `Renew image qualification: installation enabled.`
- `Renew image qualification: self-test enabled.`
- `Renew image qualification: core clear enabled.`

Installation, self-test and core clear are mutually exclusive. Default source keeps the
successor disabled. Publish installation-only first, then independently review its
result and publish capture-only. Acquisition and archive-bound clear retain their
separate Rust command admission under this same task:

- `Renew-image core-dump acquisition: enabled (fresh recovery required).`
- `Renew-image core-dump clearing: enabled (private archive verified).`

Disable consumed stages after recording their outcome. Each stage uses a fresh
ignored 0700 parent/child, immutable 0600 artifacts and distinct private stdout
and stderr. No raw dumps, serial diagnostics, credentials or local network facts
may be copied into public evidence.

## Installation, preserving the known retained attempt

Example command shape, using absolute paths frozen in the private context:

```text
just str005-panic-probe renew-preflight --private-root <new-install-child> --gate-root <pinned-clean-gate> --manifest <new-package-manifest>
just detect-ultra205
just str005-panic-probe renew-serve --private-root <new-install-child>
just str005-panic-probe renew-install --private-root <new-install-child>
just str005-panic-probe renew-finish --private-root <new-install-child>
```

Detector output belongs in the private parent's `detector.stdout.log`, and a
fresh installation detector in `install-detector.stdout.log`. The printed URL
opens the sole direct browser serial owner. In that page, connect the exact
installed image and collect the before baseline. An immutable one-use
`baseline-begin.json` marks actual collection start before observations; its
120-second freshness window cannot be renewed by finalizing later.

Before any ROM entry/reset, collect authenticated ledger/original budget,
settings/identity/authorization preservation, current diagnostics, and status for
startup003's **known attempt ID**. Never query null while its retained record
exists. Admission requires the same boot/generation/attempt, terminal state,
closed socket, quiescent worker, no retained fence, no pending accounting, and
exact measured ledger/budget comparison. Stop/Close run independently of collection
success. Both current restoration and serial release must be observed.

The truthful reset prerequisite is `str005-current-recovery-proof-v2` with
`current_v2_released=true`, the known `retained_attempt_id` and
`retained_status_sha256`; it contains no `current_v2_idle` claim. Before installing,
the owner rereads the actual saved status and collection begin, revalidates release
and all evidence, and binds the digest and timestamp to this proof.

The existing installation adapter launches only admitted state-preserving
`flash-monitor` with exact physical identity/package, dual private evidence and
360-second capture. Its 1,200-second outer timeout retains failure and kills
owned descendants; it never retries a reset/write implicitly. Same-page
preservation survives the browser disconnect/reconnect. Candidate configuration
requires the actual install receipt; collect fresh candidate baseline, exact
running ELF and unchanged ledger/budget, then Stop/Close and host cleanup. Seal
installation independently. A returned image is not proof of original panic cause.

## Existing-core preservation, capture and cutoff

After a verified sealed installation, publish capture stage and use retained
package bytes with `renew-preflight --recover-installed-root <sealed-install> --retained-manifest <frozen-manifest> --gate-root <gate> --private-root <new-recovery>`.
`renew-serve` collects a fresh authenticated idle recovery after the intentional
installation reset. V1 idle proofs are appropriate here because the old retained
record was preserved before the reset. This recovery-only mode cannot install or
self-test.

Before sealing that recovery, acquire the actual core partition through
`just core-dump-read` into its `installed-core` child using its fresh
`current-recovery.json`, physical identity and exact installed source/ELF. Preserve
the result, partition table and all dump bytes. It must be empty for the following
capture admission. A nonempty dump blocks the self-test: preserve and diagnose it;
clearing it requires a separately published archive-bound scope, never automatic
erasure. Return to the application and independently collect fresh baseline and
release evidence after acquisition.

A capture-only preflight uses `--capture-recovery-root <sealed-recovery>` and
`--retained-manifest <same-frozen-manifest>`, plus the Gate/private-root arguments.
It verifies the sealed installation lineage, preserved empty partition, exact
cutoff/store/Start/Renew audits, and source/boot-bound store readiness. The owner
collects two current diagnostics snapshots and fresh state/accounting, then exposes
one ASIC-off self-test. Retain the panic acknowledgement, boot transition and
immediate store diagnostics before any later ROM reset. Stop/Close and independent
candidate recovery remain available when the self-test fails.

Acquire into `self-test-core` through the same repository core reader. Offline:

```text
just core-dump verify-cutoff --dump <self-test-core/core-dump.private.bin> --elf <exact-installed-elf> --elf-sha256 <exact-sha256> --private-root <cutoff-review>
```

Require valid captured length/checksum, full matching ELF identity and actual
captured cutoff receipt: ASIC outputs disabled and generation revoked. Native
placement alone and SDK store success are insufficient. Preserve partial or failed
outcomes; the generic observer never promotes them to verified capture.

## Archive-bound clear and terminal criteria

Only after immutable archive/checksum/full-ELF/captured-cutoff review may a new
fresh authenticated recovery admit the thin successor clear wrapper. Publish
`Renew image capture seal: <64 lowercase hex>.` in the owning task and the separate
core-clear sentinel. The sealed capture root supplies independently revalidated
actual dump, checksum, full ELF, captured cutoff and immediate store observations.
Use a newly completed/sealed recovery-only successor root with a fresh V1 proof:

```text
just str005-panic-probe renew-clear-preflight --private-root <new-clear-root> --capture-root <sealed-capture> --recovery-root <fresh-sealed-recovery>
just str005-panic-probe renew-clear --private-root <new-clear-root>
just str005-panic-probe renew-clear-finish --private-root <new-clear-root>
```

The wrapper reuses `runClear`/`verifyClear`, preserving their launch, detector,
exit, archive comparison, actual child result and sealed child inventory. Fresh
`clear-detector.stdout.log` belongs in the parent of the new clear root. Finalization
seals the wrapper only after verified success; failure artifacts remain inspectable
and cannot qualify the successor. Its result explicitly requires a fresh post-clear
baseline. Underneath it invokes only the repository `core-dump-clear` command. Bind physical/source/
ELF, recovery proof, preserved dump and SHA-256, and a fresh private child. The
existing command rereads/compares the actual bytes, erases only the actual admitted
core partition, verifies all-FF readback, returns to the exact application and
releases ownership. No proof authorizes a different partition, dump or image.
Every read or clear consumes the semantic recovery proof exactly once in the
ignored 0700 `scratch/core-dump-proof-claims` registry before USB ownership.
A copied or reformatted proof and a different output directory cannot renew it;
claims persist after failures. A later ROM operation requires fresh collection.
Collect fresh post-clear identity, unchanged ledger/budget, current restoration and
preservation, Stop/Close and host release. Seal the clear and qualification result.

One installation and one capture self-test are admitted per published attempt;
read/clear operations each require fresh proof and unused output roots. A failure
stops dependent effects. Recovery is read/Stop/Close and guarded restoration only;
no grant issuance or implicit reinstall. A subsequent attempt requires demonstrated
progress and a new published contract, not a fresh ordinal guess.

Qualification requires all independent install, preservation, capture/cutoff,
clear-readback and cleanup results. Otherwise record the exact blocker. This does
not claim new-image mining startup, renewal success or accepted-share proof.

## Applicability and verification

Renew dispatch isolation requires exact-ELF Start/Renew path audits and affected
authorization/renewal acceptance and rejection regressions. Reassess USB ownership,
startup, packaging and preservation explicitly. Unchanged channel/protocol evidence
may be reused with a stated impact judgment under ADR-0029; Channel006 and
startup003 retain their real tested identities. Unknown impact requires repeating
that check; four installations are not an automatic prerequisite.

Run panic-probe tests, new successor producer/consumer tests, core Rust v1/v2
admission tests, canonical build/tests and native audits. Test wrong attempts,
active/fenced records, changed ledgers, failed restoration/cleanup, stale or reused
collection proofs, old consumed gates, independent install/capture scopes and
retained-status digests. No tests contact a device.

## Recovery from installation001 candidate-history rejection

Installation001's sealed record
`8d6a89683e92b6d798b92a01b38882138b5db5f177058c3c0ed8f9a04b85a5ad`
contains a successful write and installation receipt but no candidate recovery
round. It remains incomplete with `candidate_recovery_missing`. Its exact source
`8d4e470e4cd3e18be510d3912586aeda65960275` and ELF
`d0dd775335e55e9d009a0a779bc938bebf64e76fe78d8d7c8144a06ef1c9d88e`
form a **partial write identity anchor**, not qualified preservation or recovery.

The strict read-only anchor validator accepts only that seal, image and recorded
failure boundary, verifies the write/monitor receipts and exact native audits,
and preserves all original non-claims. `renew-preflight --recover-install-root <installation001> --retained-manifest <frozen-package> --gate-root <new-pinned-gate> --private-root <new-recovery>` may use it to prepare an independently authorized
fresh current-state recovery. No installation, self-test, grant or mining is
available in this recovery-only mode. Root task publication still owns effects.

A completed and sealed fresh recovery from that anchor can supply
`--before-recovery-root` to a separately published new installation. Fresh before
measurements and same-page preservation are mandatory; do not reuse startup003's
retained status after installation001 reset it. This lineage adapter never changes
installation001's status, infers historical preservation, or enables an unchanged
retry without demonstrated progress and a successor contract.

Candidate recovery now owns its entire prelude: possession, status and HTTP begin
are bounded. A status-history rejection before any HTTP round still triggers
independently bounded Stop and Close. A typed first failure and actual closure
assessment are persisted as `candidate-recovery-failure.json`, included in the
immutable partial result, and disable further candidate recovery attempts in that
page. Missing or malformed closure evidence cannot become a cleanup pass.

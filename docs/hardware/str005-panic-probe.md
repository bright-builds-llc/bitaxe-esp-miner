# Staged STR-005 panic capture contract

The active `task-str005-start-panic-diagnosis` and [ADR-0031](../adr/0031-prospective-panic-diagnostics.md)
authorize this prospective diagnostic sequence. Share002 remains sealed and
unverified; its missing historical retained resource record is not replaced by
new current-state observations. Parity remains 90/95.

## Published scope

This revision admits authenticated baseline collection, preservation of the
existing core region, one state-preserving installation, one ASIC-off capture
self-test, independent recovery, private dump analysis and explicit core-only
clearing after verified archival. It does not implement or admit Start, work
grants, mining, renewals, an external pool, factory reset, NVS erase, direct pin
operations or replay of Share002. A later published command must independently
verify the capture/cutoff prerequisites before the single Start stage in ADR-0031.

Publish clean host source and the pinned Gate source before device access. Build
`just package` and `bazel build //tools/flash:flash` from that clean pushed source.
Preflight verifies canonical package/source identity, exact retained Gate assets,
the immutable Share002 inventory, and the compiled native cutoff. It also runs
the generated installation command through the pinned real CLI with `--dry-run`
in a separate protected directory. Both the exit and dry-run receipt must prove
that no hardware branch executed; argument conflicts block preflight.
`just test-panic-probe-command` exercises this actual CLI boundary, including the
historical conflicting-flag rejection, with a nonexistent device node. This is
a post-publication local integration check: run it after `just package` on clean
pushed source; a dirty package is correctly refused even with `--dry-run`. Its audit
checks the fatal-handler route, IRAM code/literals, internal-DRAM state, bounded
straight-line instructions, safe GPIO latch writes and generation revocation
before delegation to ESP-IDF. It is software evidence, not hardware proof.

## Protected output and commands

Use a new parent under `scratch/str005-panic/` with mode 0700 and an absent child
named `attempt`. All raw browser diagnostics, manifests, account identifiers,
core bytes and vendor output stay in ignored private files (0600). Keep each
wrapper's stdout/stderr in distinct files outside an output child that must not
exist yet. Never print raw memory or sensitive settings into public evidence.
All paths below are absolute. Preflight requires the canonical manifest path
`<repo>/bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json`; offline dump
analysis requires a resolved regular ELF path or a byte-identical private copy.

```sh
just str005-panic-probe preflight --private-root <parent/attempt> --gate-root <clean-Gate-root> --manifest <canonical-package-manifest>
just detect-ultra205
just str005-panic-probe serve --private-root <parent/attempt>
```

Save the first detector output as `parent/detector.stdout.log` and stderr as a
separate sibling. Serve requires this output to be at most 60 seconds old,
exactly one admitted runtime device with the same physical identity, and no
existing serial holder. Open only the printed loopback page. Use native Chrome
Connect Worker to obtain fresh authenticated identity; USB names/descriptors
provide no application authority. Click **Capture baseline and accounting, then
close**. The collector independently attempts status, both accounting ledgers,
validated diagnostics, Stop and Close, including after an earlier failure.

The proof requires safe baseline, disabled mining, unchanged settings/identity/
authorization high-water, idle current V2 status, no pending reservation, matched
boot observations, confirmed restoration, inactive authority and actual serial
release. It records the measured next ordinal; no ordinal is assumed. Proof is
valid for 120 seconds, rejects future timestamps and is never refreshed by
editing its timestamp. Old idle status is not historical resource evidence.

Preserve the installed core region before installation, using a separate absent
private child and the current proof:

```sh
just core-dump-read --board 205 --port <fresh-port> --expected-physical-sha256 <physical-sha256> --expected-installed-source <installed-commit> --expected-installed-elf <installed-ELF-sha256> --recovery-proof <attempt/current-recovery.json> --private-root <new-read-child>
```

This owns the existing physical lease, admits ROM using bounded board-info,
reads the actual partition table and only the admitted core range, returns the
same device to the expected application and releases resources even after read
failure. Each read is bounded to 360 seconds. Keep a supervisor budget larger
than all composed admission/read/return limits; terminate and retain the whole
process group on expiry. Never run another USB owner concurrently.

Seal the first baseline with `finish` after closing its page/server and saving a
fresh detector output as `parent/cleanup-detector.stdout.log`. Create a new
preflight/root/page for installation so the baseline proof is freshly collected.
Preserve the initial detector log. After baseline collection, save a new detector
output as `parent/install-detector.stdout.log`, then run:

```sh
just str005-panic-probe install --private-root <parent/attempt>
```

The installer checks proof freshness, exact physical identity, binary/package
hashes and serial release. It invokes the prebuilt repo flash owner once with
state-preserving segments, `--expected-physical-sha256`, dual private/redacted
evidence and a 360-second capture limit. The child group has a 1,200-second outer limit;
its claim, stdout/stderr, exit and cleanup results are retained independently.
No Wi-Fi/pool credential input or seed/reset operation is accepted. Keep the
original page open so it can compare its private preservation baseline after
installation. **Verify installation and configure candidate** requires the real
successful flash receipt, then native Connect Worker must match the exact
candidate source and ELF before any self-test.

## One ASIC-off capture self-test

Click **Run one off-only core capture self-test** only after candidate identity
and preservation checks pass. A one-use host claim and fresh authenticated
request bind the current boot ordinal. Firmware requires disabled ASIC power,
held reset, no worker authority and fresh session state before and after its
generation claim. It acknowledges before deliberately aborting. The native
panic wrapper first writes the established safe latch values, revokes the
actual generation, records a fixed cutoff receipt and then delegates to IDF.
The self-test marker is set only on this authenticated off-only path.

Retain the Gate observer result even on failure. Stop/Close run independently;
use a fresh native connection and **Collect candidate recovery and close** to
obtain a new current proof. Every recovery round gets an immutable new directory
and distinct boot/transport session, with at most eight rounds per page. The UI
reports the exact proof path. No self-test retry is allowed in this contract.

Read the candidate core using that fresh proof and exact candidate identity.
Run the protected offline commands with the matching retained ELF:

```sh
just core-dump inspect --dump <private-dump> --elf <exact-ELF> --elf-sha256 <ELF-sha256> --private-root <new-inspection-child>
just core-dump analyze --dump <private-dump> --elf <exact-ELF> --elf-sha256 <ELF-sha256> --private-root <new-analysis-child>
just core-dump verify-cutoff --dump <private-dump> --elf <exact-ELF> --elf-sha256 <ELF-sha256> --private-root <new-cutoff-child>
```

Require vendor checksum/full ELF identity, expected deliberate panic evidence,
and an actual captured internal-DRAM cutoff receipt proving configured/output-
enabled safe pin state, generation revocation and `self_test_marked: true`.
Executable BSS defaults are never a substitute for captured bytes. Missing,
truncated, corrupt or mismatched evidence is a blocker, not capture success.

The first dump is preserved on device. Only after private archival and matching
full partition digest may a fresh recovery proof admit separate clearing:

```sh
just core-dump-clear --board 205 --port <fresh-port> --expected-physical-sha256 <physical-sha256> --expected-installed-source <candidate-commit> --expected-installed-elf <candidate-ELF-sha256> --recovery-proof <fresh-candidate-proof> --preserved-dump <archived-full-partition> --preserved-sha256 <full-partition-sha256> --private-root <new-clear-child>
```

Clearing rereads and compares the current full region, rechecks same-lease ROM
identity without resetting again, erases only the admitted core range and
verifies all bytes erased. It has no automatic retry. It cannot erase NVS or
reset qualification accounting. Fresh recovery is required after return.

## Cleanup and stop conditions

Close the owned page and server, collect a fresh cleanup detector log, then:

```sh
just str005-panic-probe finish --private-root <parent/attempt>
```

Finish verifies server/process/listener and observed serial-node release,
revalidates collected parts and seals the new inventory. A main-operation failure
does not suppress recovery or cleanup; retain first failure and every independent
cleanup result. The host result alone never claims a valid captured core.

Stop further effects on identity/settings drift, pending accounting, stale or
missing proof, failed native audit, failed installation, unexpected reset,
missing dump/cutoff evidence, unsuccessful application return or remaining
resource ownership. Preserve partial evidence and record the exact blocker.
No blind retries; a demonstrated boundary fix needs verification and a new
published contract/root. Do not archive the diagnosis until its actual criteria
pass, and never amend historical sealed evidence to permit continuation.

## Recovery after the observed HTTP startup failure

The sealed installation002 write completed but its runtime capture reported
`http_server` / `http_task`, stable boot and safe baseline, with incomplete
startup. Its ordinary installation gate remains failed. A separate read-only
mode binds that exact failure and its retained clean package before fresh USB
admission; it cannot configure installation or bind/invoke the self-test API.

```sh
just str005-panic-probe preflight --private-root <new-recovery-parent/attempt> --gate-root <clean-Gate-root> --recover-install-root <sealed-installation002-attempt> --retained-manifest <private-build-663d5314/bitaxe-ultra205-package.json>
```

Use the usual fresh detector, serve, native Connect Worker, baseline collector,
Stop/Close and finish sequence. The expected installed source/ELF come from the
verified failed-installation lineage, while the host source is the current clean
published revision. Full inventory, command, exit, package, native audit and
private log digests must match. Only the observed HTTP task failure is admitted;
identity drift, partial writes, missing cleanup, unsafe baseline and unrelated
errors fail closed. Fresh authentication supplies the exact ELF and accounting.

This page proves current-session continuity only. It does not manufacture the
original page's lost before/after comparison or turn installation002 into a
success. The result keeps `installation_complete: false` and labels that narrower
continuity basis even when current recovery succeeds.

A subsequent correction preflight may use its sealed current recovery as an
identity anchor, together with the new canonical clean package:

```sh
just str005-panic-probe preflight --private-root <new-install-parent/attempt> --gate-root <clean-Gate-root> --manifest <repo/bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json> --before-recovery-root <sealed-current-recovery-attempt>
```

That historical anchor is not effect authority. A new live baseline and fresh
120-second proof are still mandatory before the ordinary state-preserving
installation command. Preserve/read the current core region before the trial.
The correction pins IDF PERF and moves ordinary allocations larger than 2 KiB
toward PSRAM through the standard IDF policy. Internal reserve, HTTP/task and
core stacks, DMA requirements, partition/NVS policy and all effect gates remain
unchanged. One no-mining correction trial follows verified software progress;
failed startup again stops the sequence for more specific allocation evidence.

Policy references: [ESP-IDF 5.5.4 external RAM allocation and reserve](https://docs.espressif.com/projects/esp-idf/en/v5.5.4/esp32s3/api-guides/external-ram.html)
and [dedicated core-dump stack guidance](https://docs.espressif.com/projects/esp-idf/en/v5.5.4/esp32s3/api-guides/core_dump.html).
Neither document establishes this firmware's measured headroom or permits
moving cache-sensitive task stacks into PSRAM.

The 2 KiB threshold targets known default-allocation sites in the pinned source:
HTTP's 2,328-byte `httpd_data` and the 4,096-byte USB RX ring storage. The USB
control structure uses explicit internal capabilities; its interrupt is registered
with flags zero, so it is disabled during cache-disabled flash critical sections.
The 2,048-byte TX storage remains internal-first at the threshold. Default
allocations already permitted PSRAM fallback; this changes preference, not an
explicit capability requirement. These eligible sizes are not measured reclaimed
bytes—actual startup and capture must still pass. Source review after the
failed self-test corrected the earlier scope assumption: pinned IDF also walks
used PSRAM heap blocks. Account for them in capacity; external static sections
are a different case. The native task stacks and cutoff receipt stay internal.

## Recovery from the sealed installation timeout

Installation003 exhausted its 480-second supervisor after the program reported
application return, before the final capture receipt was persisted. That marker
permits only fresh read-only admission, never an installation-success claim.
The recovery predecessor validator must bind the sealed command/context, pinned
binary, timeout/cleanup result, protected stdout and retained package, and reject
interruption, spawn failure, ambiguous markers or incomplete cleanup. Fresh Gate
authentication must still establish the actual installed source and full ELF.

Use recovery-only preflight with installation003 and its retained `build-9be53f69`
manifest. Preserve the failed installation and current-session-only continuity
labels. A missing capture receipt never authorizes another flash.

After that recovery is sealed, capture qualification of the existing image uses:

```sh
just str005-panic-probe preflight --private-root <new-capture-parent/attempt> --gate-root <clean-Gate-root> --capture-recovery-root <sealed-current-recovery-attempt> --retained-manifest <private-build-9be53f69/bitaxe-ultra205-package.json>
```

This mode cannot install firmware. Its same-image before/candidate transition
requires a new live baseline, fresh authenticated identity/accounting, current-
boot healthy startup diagnostics, unchanged settings/identity, safe inactive
state and actual Close. It records a separate capture-admission review, leaving
installation completeness false. Failed, stale, missing or mixed-boot readiness
blocks the self-test. The usual one-use off-only self-test, independent recovery,
private dump inspection/cutoff verification and cleanup bounds then apply.
No Start or work grant is available through this mode.

Future installation supervisors use a 1,200-second outer bound while retaining
the 360-second observation window. New context, claim and runner records bind
that bound; legacy records without it retain their original 480-second meaning.
The longer supervisor permits the listed stage budget with additional margin,
but remains a deliberate outer stop if nested operations take longer. It does
not change or authorize replay of installation003.

Existing-image capture also requires `installed-core` evidence inside the sealed
current-recovery root: a successful same-image core read, application return,
cleanup and a full erased 952 KiB region. Preserve that region before sealing
recovery. Admission binds those files and expects the next live boot ordinal to
be exactly one beyond the recovery session observed before the managed ROM
round trip. Any intervening boot, nonempty region or failed read/return blocks
capture and requires inspecting the preserved evidence first.

The firmware collapses several reset causes into `other`; do not relabel that
value as a proven USB or software reset. For this off-only diagnostic, `other`
is admissible only with the bound managed-return/empty-region evidence and
fresh healthy, inactive current state. Record the raw-cause uncertainty. Panic,
watchdog and brownout observations remain blockers here. This does not admit
Start: the controlled self-test must independently yield the expected explicit
panic observation, exact valid core and captured cutoff proof.

Readiness uses two separately retained Gate exports from the same live session,
with the same current boot and exact identity and strictly advancing
`runtime_ready` uptime. Gate coalesces observations by category/stage; one export
contains only the latest ready sample and its array order is not wire chronology.
Capture-only baseline collection therefore obtains a second export within the
bounded observation window before Stop/Close. Both snapshot hashes are bound
into the review. Missing, repeated, failed or mixed-boot observations block
admission; failed collection still closes the session and retains partial files.
The ordered native panic-observer journal remains a separate evidence source.

## Terminal result of this admitted self-test

The one controlled panic/reset completed, but the full core partition remained
erased. [Capture qualification failed](../parity/evidence/20260927-str005-core-self-test-no-dump.md);
Start was not admitted. The active task now disables installation, further
self-test, reset-capable acquisition and clearing. The historical commands and
sealed outcomes above are preserved; they do not authorize another fault.
Only read-only baseline collection remains enabled until a new contract binds
a verified store-result/required-length discriminator.

## No-mining stack-correction installation continuation

Owner-authorized 2026-09-27 after the static signed-Start stack analysis. This
prospective continuation admits a verified stack-frame correction and one
state-preserving installation without a capture self-test. It supersedes the
preceding installation/acquisition disablement only for this narrow sequence;
the core-store failure and live Start prerequisite remain unresolved. It changes
no task stack size, internal reserve, allocation policy, partition layout,
USB ownership, heartbeat deadline, signed-authority or accounting rule.

First reject the original Share002 signed-Start selected native chain against
its 16 KiB budget with a 2 KiB minimum unclaimed margin, and require the outlined
candidate to pass the same audit. The margin is a conservative selected-path
engineering gate, not a proven whole-task maximum or runtime high-water result.
Run `just audit-signed-start-stack --elf <exact-ELF> --output <new-audit-json>`
and retain the report bound to the package ELF. Run behavioral regressions,
canonical native package/resource checks and the
actual flash CLI dry-run before device effects. Publish clean source and this
contract before preflight/device access.

Use fresh protected parents `scratch/str005-panic/current-recovery-003` and
`scratch/str005-panic/installation004`, each with an absent `attempt` child and
distinct protected sibling logs. The existing sealed installation003 and retained
`build-9be53f69` package anchor installed identity only; their failed outcomes
remain unchanged. Follow the recovery-only preflight above with that pair,
collect fresh authenticated current baseline, then preserve the full existing
core partition as `attempt/installed-core` using `just core-dump-read`. It must
retain the same physical lease, current proof, exact installed identity, bounded
ROM admission/read/application return and cleanup. Seal that recovery only after
fresh cleanup detection and page/server release. Preserve nonempty contents;
never clear the core region for this installation.

Preflight installation004 with the canonical clean candidate manifest and
`--before-recovery-root <current-recovery-003/attempt>`. The sealed historical
proof is only an identity anchor. Collect a new live baseline in the installation
page, close its serial owner, save fresh install detector evidence, and invoke
`just str005-panic-probe install --private-root <installation004/attempt>` once
within the proof's 120-second lifetime. Preserve the page's in-memory before
state across installation. The existing 360-second runtime capture and
1,200-second installer supervisor bounds apply; failed receipt, unexpected reset,
identity/settings drift, pending ledger, unsafe state or unreleased ownership
stops further effects.

On a successful receipt, use **Verify installation and configure candidate**,
connect freshly to the exact candidate and **Collect candidate recovery and
close**. Require healthy startup, exact identity, unchanged settings/Device
Identity/replay high-water and both measured ledgers, confirmed restoration,
inactive authority and actual serial/page/process/listener release. Finish and
seal the attempt. A no-self-test installation result must explicitly keep
`core_capture_verified: false`; it cannot satisfy panic-capture qualification or
establish the original panic cause. A missing candidate recovery is failure.

Self-test, core clearing, signed Start, grants, mining, renewals, external pools,
factory reset, NVS erasure, direct pins and Share002 replay remain unavailable.
No qualification ordinal is consumed by this sequence. Report the next ordinal
from fresh accounting without assuming its value. Four historical continuity
cycles remain observations on their original package; this single update checks
its own before/after preservation and does not claim new four-cycle/live parity.

Failure collection and Stop/Close remain independent of the main operation.
A failed install retains its typed earliest boundary and independent cleanup;
only an already supported exact failed-install recovery mode may reopen the
same device. A newly unsupported failure is a blocker requiring its own verified
recovery implementation, not a reason to broaden an existing validator. No
unchanged retry or second flash is admitted. A targeted verified fix may proceed
under the repository progress policy with a fresh published ordinal/root.

## Recovery after reference-observation rejection

Installation004 completed its single write and trusted healthy startup capture,
but the wrapper rejected the receipt before candidate configuration because the
fixed-serial producer intentionally reports `observed_reference_commit` as
`Unavailable`. The package reference is known; that does not make it an observed
serial field. The corrected validator accepts only the exact unavailable marker
or an exact reference match, retaining all other package, time, private-log,
source and healthy-capture requirements. Mismatched observed references reject.

The original installation004 remains sealed with its failed wrapper result and
missing before/after candidate-preservation proof. It is not retroactively made
successful. A new recovery predecessor case requires its exact zero-exit child,
complete write, trusted healthy stable capture, unavailable reference marker,
command/context/package/log digests and full release. It admits observation only,
with `installation_complete: false` and `continuity_basis: current-session-only`.

Publish and verify this host-only correction, then use a new mode-0700 parent
`scratch/str005-panic/current-recovery-004` and absent `attempt` child. Invoke the
existing recovery-only preflight with `--recover-install-root <installation004/attempt>`
and `--retained-manifest <build-2d81a9cd/bitaxe-ultra205-package.json>`; use the
pinned clean Gate. Collect fresh detector output, serve, native Connect Worker,
**Capture baseline and accounting, then close**, close the page/server, collect
cleanup detection, and finish/seal. Existing operation/cleanup bounds apply.

Require fresh exact installed source/full ELF, safe inactive baseline, both
measured ledgers without pending reservation, authenticated current idle status,
healthy startup and restoration/release. No old proof timestamp authorizes this
session. Any mismatch, unsafe/pending state, failed collection or unreleased
owner is a blocker; retain earliest failure and partial evidence. No reset,
reflash, acquisition, clearing, self-test, grant or mining is admitted, and all
corresponding task sentinels remain disabled. No unchanged recovery retry.

## Receipt-corrected preservation trial

The owner requested a fresh reflash after explanation of the receipt defect.
The verified f4d6ee39 adapter fix is the progress basis. This prospectively
re-enables installation and core preservation only for current-recovery-005 and
installation005, under the existing no-mining stack-correction bounds. The
reference marker remains unavailable unless actually observed; no old failed
result is relabeled or edited.

Publish this contract and exact clean source, build `just package` and the flash
binary, pass `just test-panic-probe-command`, and retain a passing exact-ELF
`just audit-signed-start-stack` report. Create fresh protected parents with absent
`attempt` children and separate private stdout/stderr logs.

For current-recovery-005, use recovery-only preflight anchored to sealed
installation004 and retained build-2d81a9cd. Follow fresh detection, serve,
native Connect, authenticated baseline/accounting and Close. Use its fresh
120-second proof to preserve the full current core region with `core-dump-read`
into `attempt/installed-core`. Require ROM admission, exact application return
and cleanup; preserve any bytes without clearing. Close the page/server, obtain
fresh cleanup detection and finish/seal. No effect proceeds on failed recovery.

For installation005, use the new canonical package and
`--before-recovery-root <current-recovery-005/attempt>`. Obtain a NEW live baseline
on its page, release serial ownership, save fresh install detector output, and
invoke `just str005-panic-probe install --private-root <installation005/attempt>`
once. Keep the page and its private before-state in memory across the update.
The 360-second capture and 1,200-second installer bound remain unchanged.
Require the corrected successful receipt, then configure the candidate, connect
freshly to the exact source/full ELF, and collect candidate recovery/Close on
that same page. Verify before/after settings, Device Identity, replay high-water,
mining-disabled state, both unchanged ledgers, restoration and actual release.
Close page/server, collect cleanup detection and finish/seal the attempt.

Stop on the same reference-validation boundary recurring after its verified fix,
new identity/preservation/accounting/startup failure, unexpected reset, failed
application return or incomplete cleanup. Preserve earliest failure; no second
write is admitted. Existing supported read-only recovery may collect evidence
independently; a new unsupported boundary needs its own verified recovery fix.
No Start, grant, mining, self-test, clearing, factory reset, NVS erase, external
pool or direct pins are authorized. Completion of this update proves only its
preservation/current-health boundary, not Share002's cause or panic capture.
Disable effectful sentinels again after the terminal outcome; parity stays 90/95.

## Numeric store-diagnostic self-test continuation

Owner-approved 2026-09-27. This prospective continuation admits one newly
instrumented ASIC-off self-test after native/software verification, using
current-recovery-006 and installation006 as fresh protected parents. It does
not replay the erased capture001 or any prior fault. Every prior seal and
non-claim remains unchanged. A later targeted correction requires its own
verified progress and fresh root; no unchanged second fault is admitted.

The pinned SDK's cross-object store, write-init, prepare, start and end calls
are linker-wrapped without editing generated SDK files. Numeric RTC records
bind source fingerprint and boot ordinal, commit integrity last, and expose
requested/prepared lengths, measured partition capacity and separate SDK return
codes. `init_result` is `esp_core_dump_write_init` validation, not a claimed
success result from the void boot initializer. Capacity is queried during normal
boot. Invalid/torn/wrong-source/wrong-boot records remain explicitly invalid.

The native store audit must prove SDK-to-wrapper-to-real routes, IRAM code and
literal closure, bounded writes confined to the receipt, RTC NOLOAD and internal
metadata placement, and at most 256 bytes added stack. The compiled initial
DROM jump table failed this audit; table-free arithmetic must pass it. Existing
panic-cutoff and signed-Start stack audits remain mandatory. Native checks are
software evidence, not proof of a captured dump or electrical behavior.

Publish clean firmware/host and Gate sources, update the archive pin, build the
canonical package/flash tool, and pass the real CLI dry-run before device access.
New candidate preflight runs both native audits and binds their digests. Read-only
recovery of older installed images does not require nonexistent new wrappers and
cannot self-test or install. An explicit successful-installation lineage selector
keeps it distinct from failed-installation recovery:

```sh
just str005-panic-probe preflight --private-root <current-recovery-006/attempt> --gate-root <clean-pinned-Gate> --recover-installed-root <installation005/attempt> --retained-manifest <build-02197010/bitaxe-ultra205-package.json>
```

The successful lineage checks the immutable inventory, actual install receipt,
complete candidate recovery, before/after preservation and unchanged ledgers.
It binds the post-install candidate proof, never the root pre-update proof. Its
historical proof supplies identity only; every effect still requires a new live
120-second current-state proof. Use fresh detector/serve/native Connect/baseline/
Close, preserve the entire current core region into `attempt/installed-core`,
verify exact application return and release, then finish/seal recovery006.

Preflight installation006 against the new canonical package and
`--before-recovery-root <current-recovery-006/attempt>`. Obtain its own fresh
same-page baseline and detector, flash exactly once, validate the complete
360-second capture/1,200-second supervisor receipt, configure candidate and
connect freshly to the exact source/full ELF. Keep the page's private before-state
for the preservation comparison. Do not start a work grant or fixture.

Before pressing the one-use self-test control, require a current-boot valid
`core_dump_store_receipt` at `ready`, correct source fingerprint, measured
974,848-byte capacity, unmarked self-test and all store results/lengths unavailable.
The claim requires a newly exported, source/boot-matched diagnostic snapshot at
most 30 seconds old, alongside the existing fresh state/status/ledger checks.
Wait for the normal bounded replay to supply the marker before claiming the
fault; an absent marker is a blocker, not permission to guess readiness.

Run **one** authenticated ASIC-off self-test. Require matching acknowledgement,
explicit panic observation, changed boot and exact healthy candidate identity.
The native cutoff must remain ahead of IDF fatal processing. Independently
reconnect and collect candidate recovery/Close BEFORE a ROM core read. This saves
the previous-boot store receipt while its boot binding still matches the admitted
fault. Store classification is retained separately from current recovery success;
SDK success alone does not verify a dump. Later recovery after a managed reset is
labeled outside the panic boot and cannot replace that earlier receipt.

With fresh candidate recovery proof and released serial ownership, read the full
core region into `attempt/self-test-core` using the existing managed physical
lease/ROM/read/application-return command. Retain all bytes privately. Inspect
with the exact retained ELF, require checksum/full image identity, and verify the
actual captured cutoff receipt. Keep vendor output and raw memory protected.
Collect another fresh candidate recovery if needed after the ROM round trip,
then close page/server, obtain cleanup detection and finish/seal all evidence.

Classify measured boundaries without converting SDK status into capture success:
initialization rejection; capacity rejection when the pinned 32-byte alignment
plus 32-byte checksum exceeds measured capacity and prepare returns NO_MEM without
changing its input length; later store failure; incomplete progress; or store
reported success pending actual dump inspection. Missing/corrupt evidence, stale
identity, pending ledger, unsafe state, failed return or incomplete cleanup blocks
further effects. A size result must precede any capture-profile/partition change.

No clearing, Start, grant, mining, renewal, external pool, factory reset, NVS erase,
direct pins or Share002 replay is admitted by this stage. If valid dump and cutoff
proof pass, publish/test the separate archival/clear/Start contract before its
first effect, select a new allowance from fresh accounting, and retain the full
180,000-ms reservation rule. If they fail, record the precise measured blocker
and verify a targeted correction before any fresh attempt. Parity stays 90/95.

## Measured-capacity correction trial

Installation006 produced an integrity-valid immediate previous-boot receipt:
write initialization succeeded, raw requested/prepared length was 1,195,632 bytes,
prepare and store returned NO_MEM (257), and capacity was 974,848 bytes. Pinned
32-byte alignment plus SHA-256 requires 1,195,680 bytes. The full preserved
partition remained erased. Startup, acknowledgement/panic/recovery and cleanup
passed independently. This is a measured pre-write capacity boundary.

The verified-fix continuation uses the official task-stack/register profile with
`CONFIG_ESP_COREDUMP_CAPTURE_DRAM=n`. Bulk heap/PSRAM and ordinary DRAM data are
excluded; eligible task stacks, TCBs, registers and the explicit 28-byte cutoff
receipt remain. The receipt uses the SDK `.dram2.coredump.*` selected-user region.
No partition, stack size, reserve, authority or hardware-safety limit changes.
Native cutoff audit v2 must prove both linker containment and actual SDK memory-
section table selection; a program-image default can never replace captured
receipt bytes. Legacy v1 audits remain readable for recovery but cannot admit
this profile's self-test.

Publish/test the configuration, native inclusion/decoder regressions, stack/store
and cutoff audits, exact package and real CLI dry-run. Then use fresh protected
parents current-recovery-007 and installation007, each with an absent `attempt`
child and distinct private sibling logs. The recovery-only selector is
`--recover-installed-root <installation006/attempt>` with retained build-bfc2cbb0.
That lineage supplies installed identity only; capture006 remains unverified.
Freshly authenticate, measure accounting, Stop/Close and preserve the actual core
region; verify application return/release and seal recovery007.

Installation007 uses the new canonical package and
`--before-recovery-root <current-recovery-007/attempt>`, with a new live same-page
baseline, fresh detector, and one state-preserving flash. Retain the existing
360-second capture and 1,200-second outer bound. After exact candidate identity,
preservation and current ready-receipt checks, run one new ASIC-off self-test.
This is progress-backed continuation after the demonstrated capacity correction,
not an unchanged retry. Preserve the immediate post-panic store observation and
fresh recovery proof before managed ROM acquisition of the full core region.

Require actual vendor checksum, matching full ELF identity, private inspection
and actual captured cutoff proof. Reconnect and collect independent current
recovery after the managed read, then release page/server/process/listener/serial
ownership and seal. SDK success or smaller requested length alone is insufficient.
The same authoritative capacity signature recurring after this fix stops further
faults; any different failure needs its own measured discriminator and verified
correction. Preserve every earlier result and earliest failure.

Clearing, Start, grants, mining, renewals, external pools, factory reset, NVS erase,
direct pins and Share002 replay remain disabled in this stage. A separate verified
contract must bind successful archive/checksum/cutoff proof before any clearing
or fresh bounded Start. Full private dumps remain development-authorized, and
all raw memory stays protected. Parity remains 90/95.

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
bytes—actual startup and capture must still pass. PSRAM heap contents remain
outside the supported DRAM dump; the native task stacks and cutoff receipt stay
internal. A valid core does not imply that every external heap object is captured.

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

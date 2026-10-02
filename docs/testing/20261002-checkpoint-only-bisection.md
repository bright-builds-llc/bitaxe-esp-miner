# Checkpoint-only emulator bisection

The owner requested continued bisection using ordinary application checkpoints.
The active runner uses no debugger, memory capture, core-dump acquisition or
inspection, and does not examine panic output. Earlier evidence and claims remain
immutable. Physical-effect gates stay disabled and parity remains **90/95**.

## Bounded comparison

Campaign002 runs one clean, marked virtual ELF with SDK 5.5.4, the pinned QEMU
backend and seed 1. Cutoffs 101, 102, 103, 105, 106, 107, 109 and 110 are admitted
once each. These stop before certificate completion and cannot qualify a full
handshake or physical firmware. The first cutoff is 106; choose the next half
using measured application records.

The process owner persists only complete stdout lines beginning with
`VIRTUAL_U205 `. It discards other stdout and all stderr without interpreting
their content. All raw bytes still count toward the 2 MiB limit. Partial lines
are bounded at 64 KiB; overflow terminates the owned process group visibly.
QEMU collection is bounded at 35 seconds. Termination, descendant release and
descriptor closure are required before result finalization.

Results bind the source, full ELF, configuration, package, native cutoff audit
and validator. Version 2 separates application checkpoint checks from unsupported
independent task-bound observations. Missing or unsafe records stop inference.
An expected collection cutoff is distinct from natural emulator completion.
Guest resource-release reports are distinct from verified host-owner release.

## Commands

Publish the contract and source before running a cutoff. Build the frozen package:

```sh
just virtual-emulator build --manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --evidence-dir scratch/virtual-noise-diagnostic/build006-clean-telemetry
```

Generate a fresh native receipt through the Bazel target:

```sh
bazel run //scripts:virtual_noise_prefix_audit -- --elf ELF --sdkconfig CONFIG --compiled-source-sha256 SHA --cutoff-phase CUTOFF --output AUDIT
```

`ELF`, `CONFIG` and `SHA` must come from that exact package. `AUDIT` is a fresh
private file. The runner checks its actual native instructions and bindings
before consuming the cutoff's exclusive claim:

```sh
just virtual-emulator noise-prefix --manifest scratch/virtual-noise-diagnostic/build006-clean-telemetry/virtual-package.json --audit AUDIT --stop CUTOFF --seed 1 --evidence-dir ROOT
```

`ROOT` is a fresh ignored private directory. No physical discovery, USB, flashing,
Start, grants, mining, real credentials or old-attempt replay is permitted.
Campaign001's missing debugger snapshots remain an unresolved historical gap;
the new collection mode does not change that result.

## Verification

Use synthetic ordinary text for process projection tests, plus real process
release tests, application-record checks, native cutoff audits and shared scenario
tests. Run the ordered Cargo checks and applicable repository checks before each
commit. Do not read retained crash or core evidence as part of this continuation.

Full virtual-board qualification remains incomplete. Independent task bounds,
complete-handshake behavior and the parent task's other required profiles remain
separate requirements; partial diagnostic success cannot archive the task.

## Measured campaign002 result

Published source `17461e37` produced clean virtual ELF
`6d07737269d24d3749adad65150d8285a2d7ce948469a318965f23236f9e758b`.
Both cutoffs passed all six application checks, including recorded heap integrity,
the unchanged 2,048-byte margin and guest resource release. Host process groups,
descriptors and writer leases were independently released.

| Cutoff | Last preparation boundary     | Minimum free main stack | Retained application bytes |
| ------ | ----------------------------- | ----------------------: | -------------------------: |
| 106    | Before responder construction |                   9,672 |                      2,342 |
| 110    | Before certificate completion |                   2,296 |                      3,174 |

No debugger or post-run partition inspection was used. Non-application output was
discarded; target stderr files are empty. The campaign is closed and its claims
remain immutable. Both results are diagnostic subsets with full qualification
false and independent task bounds unsupported.

These observations focus the next source investigation on completion and its
live caller frame. The shortened helper's native frame differs from the original
full probe, so this does not prove an exact faulty statement or an unchanged-image
bisection boundary. The next checkable plan extracts preparation's native frame
before completion, preserves real crypto behavior and tests its exact native
resource paths before any separate full-boundary contract is admitted.

## Static resource measurement of the correction

These figures come from dev builds that were never executed. They are static
lower bounds over named native paths plus the longest resolved crypto-family
descent. As a calibration point, campaign002's cutoff-110 ECDH descent measured
14,272 bytes statically, while runtime used about 14,088 bytes.

| Path                       | Original full probe | First correction | Responder split |
| -------------------------- | ------------------: | ---------------: | --------------: |
| Responder ECDH and sign    |              15,664 |           15,104 |          13,552 |
| Completion and certificate |              16,160 |           12,160 |          12,288 |
| Encrypted frame            |               5,664 |            1,904 |           2,032 |

The unchanged budget is 14,336 bytes (16,384 main stack minus the 2,048-byte
margin). The first correction fixed completion but pushed responder ECDH over
budget, because its combined preparation frame was 3,184 bytes. Splitting the
responder into construction and step frames, and consuming the act-two result
in place, brings every required path within budget. The responder path has the
least headroom: 784 static bytes.

Auditor v2 now requires these corrected paths, including a named
certificate-verification descent. Indirect, outside-family, cyclic and drop-glue
edges remain unbounded and are reported. A passing static receipt is necessary
but not sufficient; runtime margin and heap integrity still need a separately
published checkpoint-only run.

A clean build of source `7b2fe9e6` produced virtual ELF
`73b3f2ccded8958dc47ecf7325c70389a3bce5546a13231f523b470ef49f3dec`. Its bound v2
receipt reproduces these figures exactly, and read-only package admission accepts
it. That is static admission only: nothing was executed, and runtime margin
remains unproven.

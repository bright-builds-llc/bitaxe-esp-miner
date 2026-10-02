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

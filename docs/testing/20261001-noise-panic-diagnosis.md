# Virtual Noise crash diagnosis

This continuation belongs to `task-ultra205-virtual-board-validation`.
Physical hardware gates remain disabled, accepted-share remains unresolved, and
parity stays **90/95**. Raw cores, registers and debugger output remain private.
No original seal or diagnostic record is rewritten.

## Retained evidence

Two independent offline inspections verify the run017 dump checksum and full
virtual ELF identity:

- Dump SHA-256: `0f7c3e60e841c6d4f1277135069e1adfb641e7fc3a296f6b2a6c953ed7d32e34`.
- ELF SHA-256: `579fdd34473fe17245f36d7e026fbd68e1041b0a7408a1cb3f958d4b030eb98e`.
- Configured main stack:16,384 bytes; pinned SDK overhead:512 bytes.
- TCB bound distance:16,888 bytes, or16,889 inclusive. The saved task top and
  fault stack pointer are outside those bounds. The fault pointer lies10,168
  bytes below the stack base.
- Deduplicated observed native frames total25,824 bytes. Inlined frames sharing
  a CFA are not added twice. This is observed coverage, not a global upper bound.
- The virtual fixture caller reserves8,464 bytes and its exchange constructor
  reserves7,136 bytes. Their15,600 bytes overlap the6,080-byte Noise completion
  chain and outer controller frames.

The direct fault is a heap-registry node read in SDK `find_containing_heap`.
That node is unaligned and outside configured memory. The vector passed to free
is consistent across SDK and Rust frames:96 bytes, alignment1, word-aligned and
within configured PSRAM. This does not prove registered-heap membership: the
PSRAM payload/allocation header and heap descriptor were not captured. Vector
length/capacity locals were optimized out; capacity96 is inferred from the
verified deallocation layout, not a directly inspected local.

The virtual call site violates its stack contract. The earliest corrupting write
is not observed; neither double-free nor a Noise algorithm defect is established.
The physical status001 failure cause remains unproved.

## Minimal feedback loop

Use the real production Noise initiator, responder and transport with the same
synthetic seed/trust/time and packet shape. Remove controller, board, grants,
ASIC/work and fixture lifecycle context. Complete one authenticated handshake and
one32-byte opaque encrypted frame round trip. Typed phase callbacks preserve
stack observations before potentially failing heap inspection and serialize
results only after resource release.

Local regressions require valid authentication, exact payload round trip, phase
ordering and release after success or ordinary rejection. Wrong authority,
modified/truncated act-two and modified/truncated ciphertext must be rejected by
the real protocol path. No negative fixture may manufacture an ACK or control
success.

Before any target execution, bind the exact marked package/ELF/configuration,
verify the pinned emulator and SDK routing, audit caller frames against the
unchanged16KiB stack and existing2KiB margin, and publish the bounded execution
contract. The initial target gate remains disabled until these checks pass.

A green minimized result identifies context dependence; it does not repair or
qualify the retained full lifecycle. Further experiments must state falsifiable
predictions and change one variable at a time. An evidence-backed correction
must preserve cryptographic decisions, allocation classes and stack/safety limits,
then pass the native audit and the original composed boundary through a separate
published gate.

## Baseline result and corrected boundary

The one-use baseline was published at `df639c99` and ran on a clean marked
virtual image, ELF SHA-256
`80875ff9aaef090bc3f9016693c60de2fc99161771e66f9c87c2959b7750a94d`.
It panicked before a result; all owned processes released. Its virtual core
partition was empty. The first fault is a TLSF integrity-check read; the second
fault occurs during panic/core-storage preparation, not another baseline attempt.
Both the consumed claim and partial evidence remain unchanged.

An optimized DWARF line initially suggested BeforeCompletion. Exact caller
instructions instead load phase111AfterCompletion and then112. Therefore valid
certificate verification had already returned before the failing heap check.
The corrected instruction proof is retained separately from the earlier finding.

The prior selected `mix_hash` stack audit omitted a required valid certificate
branch. Its prefix measures15,040 bytes, rising to17,264 through nested Strauss
and odd-multiple-table crypto before additional coverage. That exceeds even the
nominal16,896-byte allocation. The observed first-fault backtrace span is8,080
bytes after crypto returned; without current task bounds or checkpoint RAM, it
cannot establish where the earliest overwrite happened. Keep this distinct from
the earlier run017 task-bound proof.

## Controlled bisection

The owner approved iterative emulated code bisection. Runtime-prefix comparisons
use one fixed ELF/configuration/seed, preserving native frames. Prefixes101..110
stop before the known oversized completion branch. A reached boundary while
owners remain live and later resource release are separate conclusions. A partial
prefix never credits authentication, a full lifecycle or hardware qualification.
Any compiled code removal is corroboration with explicitly changed native frames.

Pause an owned private debugger at reached-prefix, release or first panic/assert
to capture checkpoint RAM and task stack state before secondary panic handling.
Verify debugger grammar and ABI offline first. Each cutoff has an exclusive claim,
fresh protected root, finite process/output bounds and mandatory owner cleanup.
The active task records the exact execution gate; no unchanged baseline replay.

Build the frozen campaign package after its source and contract are pushed:

```sh
just virtual-emulator build --manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --evidence-dir scratch/virtual-noise-diagnostic/build005-clean-prefix
```

For each admitted cutoff, generate its native receipt with
`bazel run //scripts:virtual_noise_prefix_audit -- --elf ELF --sdkconfig CONFIG --compiled-source-sha256 SHA --cutoff-phase CUTOFF --output AUDIT`.
ELF, CONFIG and SHA must come from that exact package. Use a fresh private audit
file and evidence root for each cutoff. The bounded execution command is:

```sh
just virtual-emulator noise-prefix --manifest scratch/virtual-noise-diagnostic/build005-clean-prefix/virtual-package.json --audit AUDIT --stop CUTOFF --seed 1 --evidence-dir ROOT
```

Only cutoffs 101, 102, 103, 105, 106, 107, 109 and 110 are admitted. Start with
106 when its native receipt passes; select the next half from measured results.
A blocked native receipt selects an earlier admitted prefix without executing
the oversized path. Unknown runtime facts stop the campaign pending a separate
diagnostic repair. These commands cannot enter certificate completion or access
physical hardware.

Ranked predictions: caller/crypto stack overlap is strongest; retiring those
scratch frames must lower native peak and restore after-completion integrity at
unchanged limits. Earlier heap damage would fail a pre-completion prefix or its
entry check. Ownership/emulator behavior remains unproved and needs an independent
discriminator after valid stack and heap observations. No evidence currently
establishes double-free or a Noise algorithm defect.

## Current conclusion

Offline evidence proves the earlier virtual call-site stack overrun and a required
completion path exceeding the current minimal caller budget. The staged bisection runner,
selected native audits and offline debugger preflight pass local checks. Prefix
execution remains pending publication and a clean campaign rebuild. Physical gates remain disabled and
full qualification remains blocked; no shortened run is a substitute for missing
full-boundary proof.

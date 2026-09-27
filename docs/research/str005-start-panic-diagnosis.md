# STR-005 Start panic: static diagnosis

Date: 2026-09-27. Owner: `task-str005-start-panic-diagnosis`.
Disposition: **cause unresolved; `stop_hardware_blocker`**. Static investigation
and primary-source research are complete. No corrective firmware change or new
device action is justified by the available trace. The task remains unarchived.

Subsequent owner decision: [ADR-0030](../adr/0030-development-core-dumps.md)
permits full private development dumps. The resulting
[capture implementation](../hardware/development-core-dumps.md) uses ESP-IDF flash
dumps and exact debug artifacts instead of the proposed custom minimal native
fatal recorder. This does not change the historical image audit or establish
Share002's cause.

## What the evidence establishes

Share002 attempted Start, did not receive a successful Start response, and the
native page observed boot12 followed by boot13 with reset category `panic`.
That category identifies a fatal reset class, not its cause. Neither the startup
heap checkpoint nor the `corrupt` previous-boot preparation receipt identifies
an allocation failure or failed preparation step.
[Original failure report](../parity/evidence/20260927-str005-v2-share-start-unverified.md)

The later independent recovery measured qualification next18, last completed17,
1,560,000 ms charged and pending=false. The original campaign remained exhausted.
It confirmed the current safe baseline and actual release, but current V2 status
was idle with no retained Share002 record. These measurements are not a pre-crash
trace and do not authorize another Start or ordinal reuse.
[Recovery report](../parity/evidence/20260927-str005-failure-recovery-accounting.md)

The exact installed firmware source is
`cf7a3f038dabdd5383083734336aeb64a22f837c`.

| Artifact                          | Verified identity                                                  |
| --------------------------------- | ------------------------------------------------------------------ |
| Installed ELF SHA-256             | `66a77d2cb699e2064469c7b124f482a57a7fd6f6b13ef737554f53f721e78590` |
| Sealed resolved sdkconfig SHA-256 | `c02b9282fbdf61620eaf9186bfd8fb025fa32269b0528bf6b2615e7d7eb39ce3` |
| Share002 seal SHA-256             | `14d2122208b2040f1482074c77328cd3a59c651e45bc17e7be7af2648d8f5950` |
| Gate source                       | `e20c0fd52d2216596f904992ffa54fda33be9025`                         |

Every Share002 sealed inventory entry was rechecked by length and SHA-256 without
mutation. Presence-only examination of its runtime JSON, JSONL and log artifacts
found no native exception PC, register report, abort location, Guru Meditation
trace or core dump. Backtrace text in static policy is not an observed backtrace.
The ELF retains function symbols but contains no `.debug*` sections, and the seal
contains no linker map. Symbol lookup could resolve a function if a valid code
address existed; source-line resolution is limited, and there is no observed fault
address to decode. Handler symbol addresses must never be reported as fault PCs.

## The demonstrated capture gap

The installed resolved configuration selects:

| Setting                                  | Installed behavior  |
| ---------------------------------------- | ------------------- |
| `CONFIG_ESP_SYSTEM_PANIC_PRINT_REBOOT`   | enabled             |
| Panic reboot delay                       | zero seconds        |
| Primary console                          | UART0 default       |
| Secondary console                        | disabled            |
| USB Serial/JTAG console                  | disabled            |
| Core dump                                | disabled            |
| FreeRTOS stack overflow checking         | canary              |
| Compiler stack checking / heap poisoning | disabled / disabled |
| Abort on failed allocation               | disabled            |
| Main-task stack                          | 16,384 bytes        |

Application USB diagnostic replay is a different output path from the native
panic console. IDF 5.5.4 only compiles native USB panic printing when primary or
secondary USB console routing is selected. The installed UART-only routing thus
explains why connecting the browser does not capture native panic details. It
does not explain what triggered the panic. The no-secondary policy is enforced
by the existing build checks, so changing it is an ownership/framing decision,
not a harmless logging toggle.
[IDF panic implementation](https://github.com/espressif/esp-idf/blob/v5.5.4/components/esp_system/panic.c),
[local defaults](../../firmware/bitaxe/sdkconfig.defaults),
[build contract](../../firmware/bitaxe/build.rs)

The local std Rust panic hook records a file fingerprint and line before chaining
the previous hook. It does not intercept all CPU exceptions, native assertions,
watchdogs or allocator aborts. Invalid/missing receipts do not rule out any of
those causes. Preparation progress and the first allocation-failure receipt are
independent observations; an earlier recoverable allocation failure would not
by itself explain a later fatal reset.
[Rust hook](../../firmware/bitaxe/src/panic_evidence.rs),
[preparation writer](../../firmware/bitaxe/src/preparation_evidence.rs),
[IDF fatal-error classes](https://docs.espressif.com/projects/esp-idf/en/v5.5.4/esp32s3/api-guides/fatal-errors.html)

## Start boundary and diagnosis limits

The relevant source files are unchanged from the installed firmware commit.
The audited sequence is request/context validation and signature verification,
effect-pending persistence, session admission, durable qualification reservation,
then V2 share seeding and production-owner preparation, followed by the reply.
Reservation persistence precedes the owner-preparation call.
[Controller Start](../../crates/bitaxe-worker-control/src/controller.rs),
[session admission](../../firmware/bitaxe/src/bwg_worker_session.rs),
[qualification accounting](../../firmware/bitaxe/src/worker_qualification_budget.rs)

The fresh ledger has no additional durable charge. This constrains the observed
accounting result, but cannot locate the last executed instruction or prove that
no transient earlier work occurred. In particular, Start failure plus a panic
reset does not establish that ASIC preparation was reached.

| Possible fatal class                 | Missing discriminator                                                             |
| ------------------------------------ | --------------------------------------------------------------------------------- |
| Rust panic/assertion                 | valid build-bound Rust location receipt or decoded panic location                 |
| Native exception                     | exception class and faulting code address                                         |
| Explicit native abort / failed check | closed abort reason or source location                                            |
| Stack failure                        | watchpoint/canary cause or captured fault, correlated with the actual task        |
| Allocation failure                   | failed allocation at the fatal boundary, not merely the first failure in the boot |

These are unresolved classes, not ranked causal claims. No agent-runnable loop
currently reproduces the actual Share002 panic. Existing host tests do not
exercise the same ESP-IDF task, stack and hardware runtime, and the historical
private Start payload was not retained for replay. A synthetic failing test would
not prove this bug. Following the diagnosis workflow, no speculative stack,
heap-reserve, timing or ASIC change was made.

## Recommended next engineering work

1. Retain a matching diagnostic ELF with line information and linker map for each
   future candidate. Bind them to the exact image/build; never symbolize an old
   crash using newly rebuilt addresses. Verify image/segment effects before
   claiming a debug-metadata change leaves runtime bytes unchanged.
1. Extend the fixed-size reset-retained diagnostic design to distinguish native
   fatal classes as well as Rust panics. Candidate fields are build/boot identity,
   closed reason, CPU, validated code offsets and a closed Start-stage identifier.
   Do not retain arbitrary panic messages, task names, raw registers, stack or heap
   bytes. A pinned native-handler integration must be source-reviewed and native-
   tested; the Rust hook alone is insufficient. This is a design recommendation,
   not a claim that ESP-IDF provides a ready-made safe hook.
1. Add minimal checkpoints around receipt/validation, signature verification,
   effect-pending persistence, reservation commit and owner preparation. Require
   allocation-free bounded writers, first-failure retention, boot/build checks,
   torn-write tests and non-consuming replay through the existing USB owner.
1. Preserve those typed diagnostic categories in a dedicated panic collector.
   The current recovery projection deliberately retains only boot observations;
   reusing it unchanged would omit valid panic/allocation/preparation receipts.
   Validate before any write, and decode only admitted code locations offline.
1. Before any installed reproduction, resolve the recovery dependency
   prospectively, publish the exact capture/effect contract and verify its
   implementation. Use fresh accounting, the smallest necessary bounded effect,
   independent safety enforcement and unconditional recovery/cleanup. This task
   does not authorize replay, new mining authority, direct pins or another owner.

Full core dumps and debugger halts are useful vendor mechanisms but unsuitable
as the immediate default here: dumps include uncontrolled stack contents, and
debugging changes execution/safety behavior. Simply enabling secondary USB output
also risks mixing unframed or sensitive text into the control stream. See the
[primary-source comparison](str005-esp32-panic-capture.md) for alternatives and
their specific tradeoffs.

## Verification and disposition

The existing API/receipt tests and actual preparation-writer host test passed via
`bazel test //crates/bitaxe-api:tests //firmware/bitaxe:preparation_evidence_tests`.
Gate diagnostic parsing/export tests passed: 33 tests. These verify existing
software invariants, not panic reproduction or cause. The installed ELF/config
and all sealed inputs were checked without mutation; primary-source research
used versioned Espressif, esp-rs, Rust and Bitaxe sources. Ordered Cargo checks
passed (2,366 tests, three existing ignores), as did Bright Builds, redaction,
Markdown, diff and parity checks. Independent review confirmed the identities,
source ordering and conservative conclusions.

Blockers are missing native panic-location evidence and the unresolved recovery
prerequisite for new device evidence. No panic correction or live reproduction is
claimed. No device was opened, reset or flashed; no Start, grant or mining effect
occurred. Share002 and recovery evidence remain sealed, and parity remains 90/95.

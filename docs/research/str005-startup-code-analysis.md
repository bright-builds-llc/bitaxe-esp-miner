# STR-005 startup code analysis

Date: 2026-09-27. Owner: `task-str005-start-panic-diagnosis`.
Scope: offline source/history and exact original ELF analysis, before further
instrumentation. **No demonstrated cause or firmware fix for Share002.**

## Separate the symptoms

| Symptom                                        | Evidence and conclusion                                                                                                                                                                                    |
| ---------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Original mining Start failure                  | `cf7a3f03`, boot12→13, reset category panic, no successful Start response. A fatal reset, not merely an indefinitely blocked startup.                                                                      |
| Later diagnostic HTTP boot failure             | Diagnostic debug metadata changed IDF optimization; internal allocation pressure prevented HTTP task creation. The corrected `9be53f69` image subsequently reported complete startup and no first failure. |
| Installation supervisor timeout                | A 480-second host deadline expired. Later authenticated recovery established the installed image and healthy startup; the timeout itself is not proof of a firmware hang.                                  |
| Controlled panic produced empty core partition | A separate capture blocker. It supplies no fault location for the earlier Start panic.                                                                                                                     |

Sources: [original failure](../parity/evidence/20260927-str005-v2-share-start-unverified.md),
[diagnostic outcomes](../parity/evidence/20260927-str005-core-self-test-no-dump.md).

## Which additions can be responsible?

`git diff 77e2e4a4 cf7a3f03 -- firmware crates` is empty. The final guarded
successor workflow changes before Share002 did not change these firmware paths.
The significant earlier addition is `979f7ba2`, Standard V2 support in the
existing controller, serial runtime and production worker. It introduced V2
observation validation and expanded the protocol state carried by that path.
The qualified pre-V2 Noise image `ad629679` provides a compiled comparison:
three nested control frames grew by 1,088 bytes in the original failing image.
That baseline is not evidence of a successful earlier V2 mining Start.

Core dumps and debug artifacts arrived in `8f69978b`, after the original failure.
The subsequent IDF optimization/allocation-policy correction in `9be53f69`
addresses the diagnostic boot regression; neither addition can have caused the
original `cf7a3f03` panic. Earlier thread-reservation, HTTP ordering and statistics
allocation changes affect available memory, but the successful authenticated
channel shows that unconditional boot initialization had already progressed.

## Ranked inspection targets

### 1. Signed Start stack pressure — strongest concrete static lead

The USB control task has a 16,384-byte stack, separate from the production mining
owner. Its controller dispatch frame stays live while Start verifies the grant.
Ed25519 verification nests substantial native stack frames. See the compiled
measurements below. This is compatible with failure before durable qualification
reservation and with a successful channel-only test: channel setup does not run
this signed Start verification chain.

A large frame is not a proven overflow. There is no native fault PC, canary report
or task high-water measurement at the failing boundary. The unchanged ledger is
an accounting constraint, not an instruction trace.

### 2. Pre-reservation allocation or NVS failure — plausible, unlocalized

Before reservation, Start performs JSON parsing/canonicalization, signature
verification, replay-sequence persistence and effect-pending persistence.
These paths allocate strings/buffers and enter native NVS code. Low internal RAM
at boot is relevant context but cannot establish which allocation, if any,
failed at Start. Default allocation policy also distinguishes internal-capability
requests from ordinary requests that can use PSRAM.

No obvious recursive serialization was found: the grant's authorizationless view
is a distinct serializable structure; V2 parsing uses a bounded field visitor.
No confirmed repository mutex cycle was found in this pre-reservation path.

### 3. Bounded owner waits — can look hung, do not explain the panic alone

Production Start sends through a bounded queue and waits up to the remaining
lease time, capped at 65 seconds. Cleanup can wait up to 140 seconds. These can
outlast a 30-second browser Start deadline, but they are bounded waits and occur
after qualification reservation. The recorded panic and unchanged durable
accounting make a simple browser timeout an incomplete explanation.

Lock review found fail-fast atomic budget admission, a reservation mutex released
before ledger persistence, Wi-Fi/settings `try_lock` paths, atomic serial epoch
and Noise cancellation, and no repository mutex held across the owner response
wait. This reduces confidence in a simple self-deadlock without proving all
native library behavior deadlock-free.

## Compiled comparison

Xtensa `entry a1, N` frame sizes from retained native ELFs. The baseline is
`ad629679de9dcae33fdc1836372f73d20f4e0070`, retained at
`scratch/str005-noise-serial/attempt-001/qualified-artifacts/firmware/bitaxe-ultra205.elf`,
SHA-256 `1427aa29d9b563041fe8b9833b178df833cb724264b3719df2d0ce71570c5fef`.
The installed comparison ELF SHA-256 is
`075be768fc0c3789f8577917e2aadbbb28ef62d8ee1388f5cd46a54079135fde`.

| Function             | Pre-V2 Noise `ad629679` | Share002 `cf7a3f03` | Installed `9be53f69` |
| -------------------- | ----------------------: | ------------------: | -------------------: |
| `run_owner`          |                     480 |                 688 |                  688 |
| `prepare_frame`      |                     736 |                 432 |                  432 |
| `prepare_controller` |                   3,648 |               4,832 |                4,832 |
| Nested subtotal      |                   4,864 |               5,952 |                5,952 |

This is a measured 1,088-byte increase in the shared control chain. The existing
production-owner and selected Noise/V2 crypto stack audits do not bound the
complete signed Start path on this distinct USB control task.

A separate existing selected V2 Share crypto audit reports 11,680 of 12,288
bytes, only 608 bytes remaining and explicitly not a complete callgraph bound.
That is another narrow margin, but the path normally follows durable reservation,
so it ranks below the control-path candidate for this specific failure.

### Exact original signed Start chain

Original ELF SHA-256:
`66a77d2cb699e2064469c7b124f482a57a7fd6f6b13ef737554f53f721e78590`.
The table contains compiled symbol addresses and unconditional Xtensa entry
allocations, **not fault addresses or a captured backtrace**.

| Function                                            | Symbol address | Frame bytes |
| --------------------------------------------------- | -------------- | ----------: |
| `pthread_task_func`                                 | `421bb45c`     |          32 |
| `Thread::thread_start`                              | `4217e6a0`     |          32 |
| Control closure `FnOnce` vtable shim                | `420533c0`     |       1,856 |
| `__rust_begin_short_backtrace`                      | `4205085c`     |          32 |
| `run_owner`                                         | `42021e1c`     |         688 |
| `prepare_frame`                                     | `4200a1d8`     |         432 |
| `prepare_controller`                                | `420055d0`     |       4,832 |
| `verify_start`                                      | `4201274c`     |         640 |
| `verify_strict`                                     | `420bb83c`     |         624 |
| `RCompute::compute`                                 | `420bbca4`     |       1,472 |
| `EdwardsPoint::vartime_double_scalar_mul_basepoint` | `420bc738`     |          32 |
| Backend `vartime_double_base_mul`                   | `420c263c`     |       2,688 |
| `NafLookupTable5::from`                             | `420bfc44`     |       2,096 |
| `ProjectivePoint::double`                           | `420c1a10`     |         544 |
| `FieldElement2625::square_inner`                    | `42244408`     |         384 |
| Total consistent chain                              |                |  **16,384** |

The shim-through-leaf subtotal is **16,320 bytes**, with actual literal/direct
call targets resolved in the exact ELF. Successive call sites are
`42053446`, `4205086a`, `42021fc1`, `4200a344`, `420089eb`, `42012f1b`,
`420bbc5b`, `420bc01d`, `420bc746`, `420c2694`, `420bfded`, `420c1a26`.
The two platform edges are indirect: `pthread_task_func` invokes its supplied
function at `421bb484`; `Thread::thread_start` invokes its closure vtable at
`4217e6ab`. Their inclusion follows the source/ABI thread-entry path; the full
creation-site vtable assignment was not independently recovered. They add 64
bytes, reaching the entire configured stack before RTOS/TLS overhead or any
additional headroom. Even the resolved subtotal leaves only 64 bytes.

This is a specific, serious static stack-budget concern, not a complete global
stack bound or proof that Share002 executed every listed edge before its reset.
The arithmetic includes only simultaneously nested frames; unrelated large
closure clones and sequential helper calls are excluded. Reproduce by checking
each symbol's `entry a1, N` and resolving its call target in the exact ELF rather
than adding the largest functions from unrelated paths.

## Smallest next checks

1. First resolve the remaining platform edge inference and extend offline native
   stack inspection to the complete signed Start path. Compare a minimal
   outlined Start handler against the original
   controller dispatch frame. A separate non-inlined handler may reduce live
   frame overlap without enlarging every internal task stack. Measure the exact
   compiled result; do not assume that extracting a Rust function prevents
   inlining or reduces the frame.
1. If that establishes a useful reduction, preserve signature/replay/accounting
   behavior with existing host tests and native frame checks before considering
   installation. This review makes no runtime changes and is not a fix claim.
1. For a later authorized observation, capture minimal numeric Start-stage and
   actual control-task stack headroom around signature verification and NVS
   persistence. Resolve the capture blocker before relying on a new panic to
   provide a dump. No Share002 replay is needed to analyze the compiled path.

Simply increasing stack sizes is not the first recommendation: internal memory
is already constrained, and a larger stack can recreate the HTTP startup problem.

## Source navigation

- Control stack: `firmware/bitaxe/src/bwg_worker_usb.rs`, `OWNER_STACK_BYTES`.
- Dispatch/Start: `crates/bitaxe-worker-control/src/controller.rs`,
  `prepare_controller` and `start`; `controller/frame.rs`, `prepare_frame`.
- Signature/replay: `crates/bitaxe-worker-control/src/authorization.rs`,
  `verify_start`, `verify` and `verify_jws`.
- Native persistence: `firmware/bitaxe/src/bwg_worker_nvs.rs`.
- Reservation ordering: `firmware/bitaxe/src/bwg_worker_session.rs` and
  `worker_qualification_budget.rs`.
- Owner wait: `firmware/bitaxe/src/production_mining_session/bwg.rs`.
- V2 state/locks: `firmware/bitaxe/src/v2_serial_runtime.rs` and
  `crates/bitaxe-worker-control/src/controller/v2.rs`.

These are source locations, not an observed crash backtrace. Historical claims
use `cf7a3f03`; current line positions can differ after diagnostic additions.

## Disposition

Analytic review complete; original panic cause and capture qualification remain
blocked. No device access, firmware edits or new hardware evidence. Sealed inputs
were read only; parity remains 90/95. The active task remains unarchived.

Verification: ordered Cargo formatting, clippy, build and tests passed (2,403
passed, three existing ignores); Bright Builds, redaction, reference, parity and
diff checks passed. Markdown checks cover this report and the appended task
block; whole-file `TASKS.md` formatting already fails at HEAD and unrelated
historical blocks were preserved. Host tests do not reproduce the native panic.

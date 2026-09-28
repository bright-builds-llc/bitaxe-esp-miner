# STR-005 Start stack correction: installed, panic cause unverified

The signed Start path now avoids the large general dispatch frame. The native
selected signature chain decreased from 16,384 to **13,264 bytes**, leaving
**3,120 bytes** unclaimed on the unchanged 16 KiB control stack. One update was
written, and independent fresh authentication confirms the exact candidate is
installed with healthy idle startup. No mining Start or grant was issued.

The diagnosis task remains **blocked and unarchived**. This verifies a compiled
stack-budget improvement and current installed health, not Share002's fault
location, live Start success, core capture or parity. Parity remains **90/95**.

## Change and native regression

`prepare_frame` dispatches Start directly to the non-inlined
`prepare_start_controller`. The general `prepare_controller` is also explicitly
non-inlined. Validation, authorization, replay persistence, reservation ordering,
response evidence and request correlation retain their existing behavior.

The first outline alone allowed LLVM to inline the general dispatcher into
`prepare_frame`: the selected chain grew to 17,808 bytes and was rejected. The
second non-inlining boundary removed that overlap. No stack, internal reserve,
allocation preference, partition or safety-deadline increase was used.

| Exact native artifact                      | Selected fixed frames | Unclaimed bytes | Audit outcome |
| ------------------------------------------ | --------------------: | --------------: | ------------- |
| Original Share002 `cf7a3f03`               |                16,384 |               0 | rejected      |
| Initial outline, private exploratory build |                17,808 |          -1,424 | rejected      |
| Clean installed candidate `2d81a9cd`       |                13,264 |           3,120 | passed        |

`just audit-signed-start-stack` resolves the selected compiled call edges and
requires at least 2,048 unclaimed bytes. Two platform entry edges remain
source/ABI-supported inferences. This is not a complete callgraph maximum,
runtime high-water measurement or captured crash stack. Regression fixtures
reject missing edges, dynamic stack adjustment, historical exhaustion and the
inlined-caller regression. The original exact ELF was also actually audited and
rejected; the clean published candidate passed the same command.

## Installed identities

| Item                          | Identity                                                           |
| ----------------------------- | ------------------------------------------------------------------ |
| Firmware source               | `2d81a9cdd9e343eb1c6fd28626d65ec1d1459c2d`                         |
| ELF SHA-256                   | `7e22bdfe2bb439298ac334c12db205373d4e9cd0d873da2f05f923abd2b2e631` |
| Package manifest SHA-256      | `c29ebb304c3b0ab76ccd11610aedfa6e1c70a03fc29c51f8d01335c40a64d8c5` |
| Gate                          | `14d0e5b37081b78d0e5c44992010f83a1cf73e95`                         |
| Final read-only recovery host | `f4d6ee39`                                                         |
| Application image             | 4,119,712 bytes; 74,592 bytes below the 4 MiB slot limit           |

Exact artifacts, debug metadata and native audit outputs are retained privately.
The later host-only receipt correction did not cause another firmware build or
write and is not relabeled as the installed firmware.

## Hardware sequence and independent outcomes

1. Current-recovery-003 authenticated the old installed `9be53f69` image and
   measured both ledgers, inactive state, restoration and actual release. The
   managed ROM read preserved all 974,848 core bytes, restored the exact
   application and completed cleanup. The region was entirely erased, matching
   SHA-256 `94a21164829c644f15d62317c52d9f42a0ef66bd084d5ffdeb007b375e210951`.
1. Installation004 obtained a new fresh before-state proof, then invoked the
   published state-preserving installer once. The child exited zero without
   supervisor timeout or interruption and released serial ownership. Its full
   360-second capture was trusted: exact execution, safe baseline, complete
   startup, stable boot, no startup failure and no assessment issues.
1. The installation wrapper nevertheless failed at `panic_install_receipt`.
   Its validator required an observed reference revision, but the fixed-serial
   producer deliberately emits `Unavailable` for that field. The outcome was
   sealed as `installation_review_missing`; candidate configuration and the
   same-page before/after preservation comparison were not completed.
1. A regression-tested host correction accepts the producer's exact unavailable
   marker without inventing an observation or allowing mismatched references.
   A narrow new predecessor adapter requires the sealed zero-exit, trusted,
   healthy, released result and that exact finalization blocker. It admits
   read-only current recovery and never retroactively passes installation004.
1. Current-recovery-004 freshly authenticated the installed candidate, observed
   healthy startup and collected accounting/status/diagnostics independently.
   Stop/Close, browser closure, server/process/listener release and fresh serial
   cleanup passed. Its result is explicitly current-session-only and retains
   `installation_complete: false` and `core_capture_verified: false`.

Final observed boot ordinal was 13, reset category `other` (not relabeled as a
proven software reset). Runtime startup was complete with first failure none;
HTTP and SPIFFS were ready. The post-statistics internal-memory checkpoint had
7,595 free bytes and a 4,352-byte largest block. These are startup observations,
not Start-time heap or stack measurements.

Fresh final accounting: next ordinal **18**, last completed **17**, total charged
**1,560,000 ms**, pending **false**. The original campaign remains exhausted at
240,000 ms. No qualification allowance was issued, reserved, refunded or reset.

## Evidence integrity

Private roots below `scratch/str005-panic/` use protected storage. Their original
sealed inventories remain immutable:

| Root                           | Inventory SHA-256                                                  |
| ------------------------------ | ------------------------------------------------------------------ |
| `current-recovery-003/attempt` | `aad005919935455e07434440be988d78bee30379e03e254103bd5f8c8992e2ca` |
| `installation004/attempt`      | `963627e1463da3663433b89aa6c59b4f505f1c688d1eee99b7fc33956c1dcf8f` |
| `current-recovery-004/attempt` | `36073324035e96edfe1ab21319a65e8f2e81963b27fff62ca0372147f900b898` |

Final current-recovery proof SHA-256:
`a0a47ff26c74887e3d5f3ac9851df588d51b675463e05b193c8918ec265eddf9`.
Share002 remains sealed and unverified; no historical resource gap is filled by
these observations. No old evidence was rewritten or promoted.

## Verification and remaining blocker

Ordered Cargo formatting, clippy, build and tests passed: 2,405 tests, three
existing ignores. The affected controller/authorization/preservation/restoration
Bazel targets, five stack-audit tests, 51 final probe tests, actual pinned flash
CLI dry-run, clean native package, USB ownership/symbol checks, standards,
reference, redaction and parity checks passed. Markdown checks cover the changed
document/task blocks; unrelated pre-existing tracker formatting is preserved.

No self-test, dump clearing, Start, mining, renewal, external pool, factory reset,
NVS erase, direct pin operation or Share002 replay occurred. Installation and
reset-capable acquisition sentinels are disabled again. A future live Start still
requires resolving the empty-core/cutoff capture prerequisite and its own
published bounded contract. Same-page update preservation remains unverified for
installation004. Healthy current recovery does not erase either blocker.

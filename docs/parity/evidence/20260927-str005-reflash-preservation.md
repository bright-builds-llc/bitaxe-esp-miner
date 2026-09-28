# STR-005 receipt-corrected reflash: preservation verified

Outcome: **complete for this no-mining update and preservation trial**.
Installation005 and its same-page candidate recovery passed with no blockers.
The overarching panic diagnosis remains unresolved/unarchived; parity is 90/95.

## What the receipt bug was

The fixed-serial monitor deliberately sets `observed_reference_commit` to
`Unavailable`. That field concerns the pinned upstream ESP-Miner reference,
not the separately verified firmware source/full ELF identity. The package's
`reference_commit` is known, but it is not thereby observed from serial output.
The old wrapper incorrectly required the two reference fields to match.

Installation004 had already written the image and collected trusted healthy
startup when that check rejected its receipt. The f4d6ee39 correction accepts
only the exact unavailable marker or an exact observed reference match, while
retaining package-reference, firmware identity, healthy startup, time, log-digest
and cleanup checks. A conflicting reference still rejects. Tests also bind the
actual sealed failure to read-only recovery without rewriting its outcome.

The fresh installation005 exercised the corrected validator end to end. Its
receipt still truthfully says `Unavailable`; no reference observation was
invented. The original failed installation004 remains sealed and unchanged.

## Exact candidate and pre-effect checks

| Item                                            | Identity/result                                                    |
| ----------------------------------------------- | ------------------------------------------------------------------ |
| Published firmware/host source                  | `021970103d5a2c8c18852e6d937433ac685a06fc`                         |
| Full ELF SHA-256                                | `9d27de81ba1a535242d67fa77dbbb3cb7651658bf2b1879023f6cb31620f2377` |
| Package manifest SHA-256                        | `2dc3981123370aa7f3c25af8800772dda24dca69338be42a84f5c59ffa8b9649` |
| Gate source                                     | `14d0e5b37081b78d0e5c44992010f83a1cf73e95`                         |
| Selected signed-Start fixed frame chain         | 13,264 bytes; 3,120 bytes unclaimed on the unchanged 16 KiB stack  |
| Native package and actual flash-command dry-run | passed                                                             |

No new functional firmware change was introduced in this trial beyond build
identity; it retains the [outlined Start correction](20260927-str005-start-stack-correction.md).
The selected stack audit is not a complete callgraph bound or a measured runtime
high-water result. Platform edge inference and hardware non-claims remain.

## Executed sequence

Current-recovery-005 used a fresh authenticated session against installed
2d81a9cd, measured both ledgers, confirmed inactive safe state and released
serial ownership. The managed same-device ROM read preserved all 974,848 core
bytes, returned to the exact application and completed cleanup. The full region
was erased; its SHA-256 remains
`94a21164829c644f15d62317c52d9f42a0ef66bd084d5ffdeb007b375e210951`.
The page/server and serial resources were released before sealing recovery.

Installation005 obtained another fresh before-state proof and invoked the
state-preserving flash owner exactly once. The child exited zero, without
supervisor timeout or interruption, and released serial ownership. Its
360-second capture was trusted: exact execution, safe baseline, complete
startup, stable boot and no assessment issues. `timed_out_after_trusted_output`
is the bounded monitor window ending after qualified output; the 1,200-second
installer supervisor did not expire.

The corrected wrapper accepted the receipt. The original page retained its
private before-state, configured the exact candidate, and reconnected through
fresh native Web Serial permission and authenticated possession. Settings,
Device Identity and authorization high-water matched across the update;
`mine_on_boot` remained false. Candidate recovery collected fresh status,
diagnostics and both ledgers, then independently ran Stop and Close.

Final observations: boot15, reset category `other` (not relabeled as a proven
software reset), runtime startup complete with first failure none, and HTTP and
SPIFFS ready. Qualification accounting remained next18, last completed17,
1,560,000 ms charged and pending=false. The original campaign remained exhausted
at 240,000 ms. Both before/after ledger objects compare equal. No allowance was
issued, reserved, refunded or reset.

The page and server were closed, fresh cleanup detection passed, and finish
verified process/listener/serial release. Terminal result:
`complete: true`, `installation_complete: true`, `candidate_install_reviewed: true`,
complete candidate recovery, no blockers, `self_test_not_requested: true`,
`core_capture_verified: false`, `historical_resource_proof: false` and
`parity_promotion: false`.

## Sealed evidence

| Protected root/artifact                  | SHA-256                                                            |
| ---------------------------------------- | ------------------------------------------------------------------ |
| `current-recovery-005/attempt` inventory | `9e0629762fef687410c6ad65f9e52067f262af11b841a3432f08c81a8f5d3b2d` |
| `installation005/attempt` inventory      | `86589048c7b866f56e0ecc50edc77009ede414da68b617b80a967830c3b38001` |
| Installation005 result                   | `5d95d53eacf10927ce93914d4aabf1e9c1b0c06ed72f7e3503497c664b9e4162` |
| Candidate current-recovery proof         | `f07b16c794b1d841a72e17370582adcde9d4ed1a537d8994e831d41281517f21` |
| Flash receipt                            | `40f5a414cb5180385fd87f866db690a81cbfc54ebb5177ced74e8ef6bb1df388` |

Roots live below ignored `scratch/str005-panic/` with protected raw artifacts.
The exact clean package/debug artifacts are retained in protected
`scratch/development-core-dumps/build-02197010`. No raw settings, credentials,
device identifiers or core data are promoted here. Prior seals remain unchanged.

## Verification and limits

Ordered Cargo formatting, clippy, build and tests passed, including 2,405 tests
and three existing ignores. All 56 probe/stack-audit tests passed, along with
clean native packaging, exact-ELF stack audit, actual CLI dry-run, standards,
redaction, reference/parity and changed Markdown/diff checks. A host test launch
was delayed in macOS dyld before test execution; a private process sample was
retained, and the suite subsequently completed without a test failure.

No mining Start, grant, renewal, self-test, dump clearing, external pool, factory
reset, NVS erase, direct pins or Share002 replay occurred. Effectful task gates
are disabled after completion. This successful fresh trial closes the update
preservation gap for its exact package; it does not alter installation004 or
prove the original Start panic cause, working core capture, live Start, four
new continuity cycles or additional parity.

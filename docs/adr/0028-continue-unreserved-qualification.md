# ADR-0028: Continue one rejected, unreserved foreground allowance

Accepted 2026-09-08 under the authorized iterative qualification task.

## Evidence

Firmware f3bbfd6a / Gate 2106f1c passed four no-mining cycles and normal ordinal
11 with five accepted shares and three renewals. Foreground ordinal 12 was
issued and delivered, but the operator spent longer than 60 seconds arranging
the observer before Start. Firmware correctly rejected expired possession with
`admission_required` before authorization-sequence persistence or reservation.
The existing expiry boundary test confirms this ordering.

Fresh same-image recovery established safe idle state and an unchanged ledger:
1080000 ms charged, last completed ordinal 11, next ordinal 12, pending false.
No foreground fault occurred. The old campaign remains exhausted and unchanged.
The issued delivery is failed evidence, not a charged or completed allowance.

## Bounded continuation

Permit one immutable continuation of this foreground allowance only after a
passed normal result on the exact same firmware/browser pair and hint policy.
Preserve the original context, issuance, consumption, samples, failure and
diagnostic records byte-for-byte. Seal a separate unreserved-rejection record
linking those bytes, the unchanged recovered ledger and closed ownership proof.
An exclusive child link prevents a second continuation or nested continuation.

The child retains the exact allowance ID, ordinal, purpose, active-time limit
and signed hint. It receives a new logical connection, possession proof,
challenge, Work Lease ID and authorization signature. No old private binding or
signature is revived. Old private bindings were intentionally not persisted;
freshness is enforced by the closed original session, new protocol admission
and existing firmware signature/session checks.

Require fresh fan proof and a fresh authenticated idle-ledger review again
before issuing the child authorization. Any pending or advanced reservation,
work for ordinal 12 in the original history, changed evidence, wrong identity,
unconfirmed restoration or unresolved ownership prevents continuation. Charge
the original allowance exactly once, when its valid Start is admitted. Do not
refund, synthesize a charge, skip an ordinal or mark the rejected delivery passed.

## Separate driver and runtime identity

Context version 4 identifies the updated qualification driver separately from
the already-qualified firmware source/ELF and Gate source/bundle. The driver
must be an exact clean pushed descendant with only an explicit allowlist of
qualification-host and decision files changed. Firmware, crates, Gate, trust,
the browser recording client, mining policy and timing code cannot change.

Verify the retained 13-file artifact snapshot and four cycle receipts, and copy
them exclusively into the new child context. Use the retained manifest, never a
newly generated Bazel image bearing the driver commit. No flash, reset, settings
write or additional mining is authorized merely by creating the continuation.

The existing signed normal result remains a dependency. Only a completed child
foreground result can admit the heartbeat successor, using the same retained
runtime, driver, hint and cycle lineage. The final campaign remains 180/30/30
seconds, and both fault tests retain the three-second shutdown-initiation limit.

## Commands and operation

Use `just fixed-usb-qualification iterative-continue-unreserved` with the
original `--predecessor-root`, new `--private-root`, protected recovery `--input`,
existing `--authority-directory` and exact pushed `--qualification-source-commit`.
This creates evidence and
context only. Existing `serve` then requires fresh cooling and ledger checks
before one in-memory signed delivery.

For the heartbeat successor, use `iterative-preflight` with
`--retained-runtime-from` and `--cycles-from` set to the completed foreground child, the same
`--qualification-source-commit`, retained manifest, ordinary exact identity and
previous-receipt arguments, and `--suggested-difficulty 1000`.

Prepare and verify the allowed HTTP(S) observer control while disconnected.
Disable debugger focus emulation for the qualification tab and prove real
visible/hidden/visible transitions before admission; never synthesize a hidden
event or override the document property. Then run
fresh connection, cooling, ledger review, signing, load and Start contiguously,
comfortably inside the 60-second possession lifetime. Never extend that lifetime
or refresh a binding after signing without a new authorization.

## Verification and non-claims

Test unchanged/advanced/pending ledgers, wrong tuples and generations, stale or
modified evidence, duplicate and nested children, exclusive-write failures,
forbidden source changes, artifact drift, and successor ordering. Existing
firmware accounting, ownership, privacy, replay and stop guards remain binding.
Publish the driver contract before effects. Retain the original failed delivery
as failed evidence even if the continuation succeeds; no unrelated parity
criterion is promoted.

# ADR-0027: Bind advisory pool difficulty hints to signed Work Leases

Status: Accepted for implementation; hardware acceptance remains unverified.

## Evidence and scope

Diagnostic 8 and normal 9 on firmware 553e4df8 reconstructed 59 nonce results
that all met the software-expected ASIC filter. None met the pool target.
Normal operation stopped safely with eight renewals and no submitted share.
This supports examining pool difficulty negotiation; it does not prove that a
pool will honor a hint or return an accepted share within a bounded allowance.

The existing encoder supports `mining.suggest_difficulty`, but production
authorization does not emit it. The pinned reference sends a nonzero configured
hint after authorization (`main/tasks/stratum_v1_task.c`).

## Decision

Controller 0.4 adds optional signed `stratum.suggestedDifficulty`, an integer
0..65535. Absence or zero disables the Worker hint. Preserve explicit zero in
canonical signatures; omit only absence. Null, fractional, negative and
out-of-range values fail validation. The signed application manifest advertises
`poolDifficultyHintProfile: worker-stratum-difficulty-hint-v1`, changing its
digest and requiring the existing Update Authority to sign the new capability.

Worker execution uses only the signed value, with no implicit fallback to
another configured endpoint. Ordinary configured pools carry their own existing
NVS hints. Neither path writes NVS or changes credentials.

After matched successful authorization, queue one advisory with a fresh request
ID through the existing owned transport. Reconnection requires new authorization
before another hint. Advisory replies neither authorize work nor count as
shares. Only pool `mining.set_difficulty` establishes the share target. ASIC
filtering, admission, renewal, cancellation, dispatch/submission revocation and
ordered shutdown remain unchanged.

## Qualification policy

The next qualification explicitly uses `--suggested-difficulty 1000`; 1000 is
the repository's public default selected as test policy, not an observation of
installed NVS. Context version 3 freezes this value before signing and retains
it across all windows on the same exact package. Historical contexts remain
readable and immutable, without new effect authority.

Use `just fixed-usb-qualification iterative-preflight` with all existing
exact-source, package, protected-input and previous-receipt arguments, plus this
explicit hint option. The supervisor carries the frozen value into the
memory-only signed Start grant. No credential-file extension or private payload
export is needed.

All ADR-0026 accounting and safety limits remain binding: 30-second diagnostics,
final 180/30/30-second windows, charge before preparation without refund, four
cycles for each new package, verified progress before retry and proven cleanup.
The original exhausted campaign remains untouched.

## Verification

Require strict cross-language signature/canonical vectors, tamper rejection,
authorization ordering, absent/zero behavior, reconnect IDs, advisory-response
isolation and cancellation at queued-write boundaries. Verify the frozen hint
reaches the signer without credential-file changes. Run both repositories'
required checks and real ESP32-S3 packaging before publication and hardware.
A pool ignoring the hint leaves accepted-share evidence unverified and cannot
justify an unchanged retry.

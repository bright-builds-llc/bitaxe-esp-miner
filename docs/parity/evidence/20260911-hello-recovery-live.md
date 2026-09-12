# Hello recovery with live mining — 2026-09-11

The Ultra 205 passed the prospectively authorized
[version-5 recovery contract](../../hardware/hello-recovery-qualification.md):
four exact-pair update/reconnect cycles, actual mining followed by abrupt
transport loss, fresh authenticated recovery without a CLI drain, and separately
authorized resumed mining. The [attempt-003](20260910-hello-recovery-attempt-003.md)
missing-reply judgment remains unchanged; this result does not assign its cause.

## Exact runtime

| Component              | Identity                                                           |
| ---------------------- | ------------------------------------------------------------------ |
| Firmware source        | `6b6aa29e2d8ce4c788d0d6b3a41a86fc06f77757`                         |
| Gate source            | `632db8110bc74d4a7e3dac60fd915a5900b81f6a`                         |
| Firmware ELF SHA-256   | `cfeccbc51d671feab0ed7423e8f3c2c3a622d601a5194ac1202ee1d4fb6ca43c` |
| Gate archive SHA-256   | `9025c151fb18d2fa3ef27c7af03dfc754855c04e5dd2f62c14e2930333f0510c` |
| Browser bundle SHA-256 | `9cafd6ab90f23f440a6556648cac51bf205e6454c361d271499c1f2c87fb84c5` |
| Read-only reference    | `c1915b0a63bfabebdb95a515cedfee05146c1d50`                         |

Both sources were clean and pushed before effects. The clean package and its
13 runtime artifacts were preserved independently for each allowance. The final
documentation commit does not replace the qualified firmware installed above.

## Observed hardware results

Initial installation and four subsequent ordinary updates passed detector/ROM
admission, complete stable startup, exact runtime identity and process/USB
cleanup. Every cycle completed a fresh 65536-byte request and response, with
65376 bytes of probe padding. Device Identity, settings and the original
pre-mining authorization baseline survived all four updates. No NVS seed,
factory reset, external UART, pin manipulation or power-removal test occurred.

| Diagnostic phase     | Allowance | Worker generation | Work dispatched | ASIC nonce results | Active time | Gate closure | Shutdown initiation |
| -------------------- | --------- | ----------------- | --------------- | ------------------ | ----------- | ------------ | ------------------- |
| Mining loss/recovery | 14        | 1                 | 3               | 2                  | 11.501 s    | 2.651 s      | 2.676 s             |
| Reauthorized resume  | 15        | 3                 | 2               | 1                  | 9.685 s     | 0.419 s      | 0.459 s             |

Gate/shutdown timings are device-local offsets from the last valid heartbeat.
The loss hook cut after two observed work dispatches, with 11354 ms remaining at
the work gate. Its receipt confirmed complete native ownership release and zero
Restore/Close control records sent during the cut. Three work dispatches were
ultimately retained before shutdown. The three ASIC nonce results across both
phases did not meet the pool share threshold; no accepted share is claimed or required by
this short diagnostic contract.

After the full recovery observation interval, fresh Hello and possession
succeeded on the first attempted reconnect, without a drain, reset or reflash.
The actual wait after the recorded post-cut observation was 162502 ms, exceeding
the declared 145000-ms minimum. That wait accommodated the separate cooling
bound; it was not used as proof of shutdown timing.

The retained reason was `link_closed`. The previous firmware epoch contained
both native TX abandonment and epoch revocation, supporting that allowed stop
path. Queue acceptance is not host delivery, and the trace does not require a
single causal ordering between abandonment and revocation. Fresh recovery
reported one skipped record, 2136 skipped bytes and zero skipped control replies.
**Stale-control-reply hardware coverage remains false.** The production-channel
software regressions separately cover valid stale replies and an already-read
response whose integrity validation is still pending.

Accepted Start/Renew legitimately advances durable authorization marks. The
original baseline was retained, and a separate private post-work checkpoint
matched across fresh admission. Only its unrelated identifier, generation and
match result entered private evidence; the authorization digest stayed in
browser memory. Resume used fresh possession and its own signed allowance,
then produced work in generation 3 and completed the ordinary diagnostic stop.

| Sampled safety/resource observation | Loss            | Resume           |
| ----------------------------------- | --------------- | ---------------- |
| Input voltage                       | 5.330–5.35125 V | 5.3475–5.36125 V |
| Maximum power                       | 8.44 W          | 8.31 W           |
| Maximum active temperature          | 35°C            | 34°C             |
| Minimum owner stack headroom        | 8436 bytes      | 8436 bytes       |
| Terminal cooling temperature        | 29°C            | 35°C             |

Both terminal records reached `fan_paused`, confirmed restoration and an inactive
lease. The existing conservative profile and safety limits were unchanged.
Sampled extrema do not describe unobserved transients. The exact native owner
entry audit passed at 4512 bytes against an 8192-byte ceiling; its allocation
remained 24576 bytes. Native retained trace storage measured 4168 bytes.

## Accounting, cleanup and verification

Each diagnostic reserved and charged its full 30000-ms allowance. The existing
qualification ledger advanced from ordinal 14 / 1140000 ms to ordinal 16 /
1200000 ms, with last completed ordinal 15 and no pending reservation. The
original campaign remained exhausted at 240000 ms. No charge was refunded and
no replacement campaign was created. Pool configuration was local-owner-supplied;
no pool values, credentials or signing payloads were retained in evidence.

The loss supervisor exited before the resume supervisor started at the same
local origin. The page retained its original continuity baseline through resume,
then closed explicitly. Both supervisors exited successfully; owned process,
listener and serial-holder checks passed. The device was left in its restored
baseline with mining disabled on boot and no active lease.

Software verification passed: 2170 Rust tests with one existing ignored test,
497 Gate web/crypto tests, 352 Gate Rust tests with two existing opt-in ignores,
200 supervisor tests and 103 canonical targets, plus native compilation,
USB ownership/symbols, stack audit, reference, redaction, parity/progress,
standards and scoped formatting checks. Quiet compiler runs were preserved and
canceled with cleanup before verification succeeded using fresh Cargo output
directories; no OS/cache-corruption cause is claimed. Two test fixtures were
corrected: handshake setup no longer inherits a two-millisecond idle-read bound,
and the negative file-permission fixture sets its intended mode explicitly.

Private operator helpers were reviewed and exercised before device use. Their
process observer was armed before each detector/flash command and never signaled
a hardware owner. The independent cycle audit bound actual flash receipts,
closed/ready journal rows, fresh probe returns, source hashes and cleanup facts.
Historical reference identity and redacted receipt paths retained their original
non-claims. No parity checklist status was promoted by this qualification.

## Evidence bindings

Closed outcome: `complete`. Independent verification checked both passed
results, all 26 preserved runtime-artifact copies, immutable sample journals,
post-work authorization continuity, four observed cycles and actual cleanup.
The loss inventory contains 209 files and was sealed before the resume context
was created; the resume inventory contains 72 files. Both roots are immutable.

| Record                   | SHA-256                                                            |
| ------------------------ | ------------------------------------------------------------------ |
| Loss result              | `f0318511a10b3d984988ed4e04a5f1a45a7caeb282e1109388c58b5da37c709f` |
| Loss artifact snapshot   | `f9d542d0145ec128d09ecb209d58c170590ff90951ffcf65a556df2cbef40485` |
| Loss inventory           | `b96759cf5151af90f5e09be7466cc87d63ecf3a16f5f00cc9785204b4bba7346` |
| Resume result            | `db61a27693f988c769c29ba4b8a2721855f8832bd5a7fa1722100a762f661ac4` |
| Resume artifact snapshot | `af6f474794e07a546729a45f1daebdefe9fc0acc621eeaf73090fd317766d332` |
| Resume inventory         | `9169e128d0931852e414a8932744735d6252cb2abe832d4ace8035c842b83541` |
| Independent auditor      | `643a4a4f65802cc8514bd5965e64ef0171fef8996f6a0305e8225ce560be7ec4` |

# STR-005 Noise runtime readiness

## Result and scope

The initial software readiness audit is blocked on a cancellable cryptographic
operation boundary. The currently pinned Noise implementation cannot establish
the frozen contract's requirement that the worker actually become quiescent
within five seconds of cancellation during every phase. Independent authority
revocation is achievable, but is not thread completion or secret disposal.

This is a source/API limitation, not evidence that ESP hardware crypto has
exceeded five seconds. No device access, native performance measurement, reset,
flash, signing or mining occurred. Controller/Gate implementation alone cannot
resolve this boundary, so no new firmware diagnostic commands or live admission
were implemented by this audit. The runtime task remains incomplete.

The authoritative contract is
[STR-005 Noise serial qualification](str005-noise-serial-qualification.md),
SHA-256 `0da417fc198a89042eb62902999ac822be5365a5f0333df8220f15afe3447a62`. The
inspected published firmware source is
`90c13b1a4b774e4e8610a8bfe4dad4c05db80eb7`. The firmware portion of this audit
adds this report and host characterization tests plus their module/build wiring.
It does not change the frozen interface, time limits or production cryptographic
behavior.

## Exact source chain

The existing production adapter calls
[`NoiseInitiator::prepare_with_observer`](../../crates/bitaxe-stratum/src/v2/noise/preparation.rs),
which first completes `NoiseInitiator::new`, calls the unit-returning observer,
then completes `act_one` before calling the observer again. There is no
cancellation predicate, fallible observer or resumable operation in this API.

The lockfile pins `noise_sv2 1.4.2`, `secp256k1 0.28.2` and
`secp256k1-sys 0.9.2`. The relevant dependency paths below are relative to their
exact downloaded crate packages, not a different version in the local cache.

| Boundary                 | Exact implementation                                                  | Consequence                                                                                                                                             |
| ------------------------ | --------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Initiator construction   | `noise_sv2/src/initiator.rs:183`, `handshake.rs:111`                  | Synchronously creates a secp context, generates a keypair, normalizes parity and builds the initiator before returning                                  |
| Secret scalar generation | `secp256k1/src/key.rs`, `SecretKey::new`                              | Repeatedly requests random bytes until the scalar validates; no attempt maximum or cancellation result                                                  |
| Act one                  | `noise_sv2/src/initiator.rs:266`, `step_0`                            | Calls `ElligatorSwift::from_pubkey` before hashing/encryption; no observer or cancellation hook inside the operation                                    |
| ElligatorSwift encoding  | `secp256k1/src/ellswift.rs:233`, `encode`                             | Enters one synchronous native `secp256k1_ellswift_encode` call with fixed zero encoding randomness; exposes no cancellation callback or candidate limit |
| Native encoding search   | `secp256k1-sys/depend/secp256k1/src/modules/ellswift/main_impl.h:335` | `ellswift_xelligatorswift_var` has an unconditional candidate-search loop; the source states average four iterations but supplies no maximum            |
| Act-two authentication   | `noise_sv2/src/initiator.rs:313`, `step_2_with_now`                   | Repeats local ElligatorSwift encoding, then performs two ECDH operations, decryption and certificate verification synchronously; no cancellation hook   |

The native search is algorithmic, not a network/DNS wait. A deterministic
counter-derived candidate is tested each iteration until an inverse encoding
exists. Typical short execution and average iteration count are not a hard upper
bound. Fixed-size hashes, curve multiplication and verification are different:
their finite computations still need target-specific runtime/stack
qualification, but this audit does not classify them as external indefinite
waits or claim that they exceed the cleanup limit.

### Source byte bindings

These SHA-256 values identify the inspected source bytes. The lockfile digest is
from the published source commit named above. Concurrent fixture dependency
additions in the working tree do not change the audited Noise/secp pins. Crate
archive checksums and selections remain bound by that published `Cargo.lock`.

| Source                                                                  | SHA-256                                                            |
| ----------------------------------------------------------------------- | ------------------------------------------------------------------ |
| Published `Cargo.lock`                                                  | `1d7155c41b514bd583292fc072be94694f42cc221ba562243478796e7d8a221b` |
| `crates/bitaxe-stratum/src/v2/noise/preparation.rs`                     | `dc10d5f606ab6070340f0d8149cca2f0f417da97e0a617def2f5d2af3b2cc8b1` |
| `firmware/bitaxe/src/stratum_v2_session/transport.rs`                   | `a33edc253a8252282d71d4286d3057ca2790a8e6a31b6baf52fa149fc5f6e126` |
| `noise_sv2-1.4.2/src/initiator.rs`                                      | `03c2e85bca751e6d8346388721af265e993ae6b89ffb7d8405b82f1490971d34` |
| `noise_sv2-1.4.2/src/handshake.rs`                                      | `a46eb8a0f1a0967ec25fda60a5d54a8cf8c38606f99c061db124c75d70fd468d` |
| `secp256k1-0.28.2/src/ellswift.rs`                                      | `605a8579b1ff96604f683d0ca275dcd5f268d0bb1c8ef3c7252b3cf50200d2bc` |
| `secp256k1-0.28.2/src/key.rs`                                           | `887ee0b1ef45a2b42617ed66802824823206afaa1d143676d56f8625f3b0f4bc` |
| `secp256k1-sys-0.9.2/depend/secp256k1/src/modules/ellswift/main_impl.h` | `776b1657c78e07f9c7bccdc46ff1000dc951515248c3c3a75e099fa826a7f920` |

## Why wrapper changes are insufficient

- Checking elapsed time after preparation returns detects an overrun; it does
  not stop preparation at its deadline or establish timely quiescence.
- A cancellation-aware RNG can control its own requests. A bounded, validated
  scalar feeder could eliminate random scalar rejection retries without using
  biased fallback scalars. Neither approach intercepts the later native
  ElligatorSwift search: that search does not call the supplied RNG.
- A fallible observer between keypair creation and act one would improve the
  between-operation boundary but cannot interrupt an operation already entered.
  The current observer returns unit, so even that early exit is not available.
- `step_2_with_now` also performs ElligatorSwift encoding; fixing only initial
  preparation leaves authentication cancellation unresolved. The supported API
  cannot accept a cached local encoding to omit that repeated call.
- Moving crypto to a thread lets the controller independently revoke authority,
  but dropping its join handle merely detaches it. A timeout cannot prove the
  worker has stopped; it must retain `incomplete` and keep effect owners fenced.
- Killing or unwinding across native code, resetting the device, substituting
  known secret material, skipping validation or weakening deadlines is not an
  acceptable implementation of this contract.

### Disposal boundary

The pinned Noise initiator implements `Drop`, invoking its internal erase
routine after normal ownership release. That code uses volatile writes for some
fields and `non_secure_erase` for the ephemeral keypair. This is not a guarantee
that every opaque internal or stack copy is erased. More importantly, `Drop`
cannot run while the worker still owns a live synchronous call.

The successor must retain the contract's narrower guarantee: repo-owned secret
buffers use zeroizing wrappers, ownership is released, and the supervisor
observes actual completion/join. Cancellation notification alone cannot set
`volatileInputsDisposed` or `workerState: "quiescent"`.

## Production-seam characterization

The tests in
[`cancellation_tests.rs`](../../crates/bitaxe-stratum/src/v2/noise/cancellation_tests.rs)
call the actual production preparation helper and pinned Noise dependency:

1. A synthetic RNG signals when the call reaches its controlled dependency, then
   waits on a bounded release channel. The supervisor requests cancellation and
   observes no callback or worker completion for 5050 ms. The test always
   releases and joins the worker before asserting. Both observer notifications
   arrive only after the dependency returns.
1. Recording cancellation at `KeypairReady` does not prevent the production
   helper from entering act-one construction and reporting `ActOneReady`.

These tests characterize missing API control points. The first deliberately
holds a synthetic entropy source; it does not reproduce an ESP RNG stall,
measure a native ElligatorSwift duration or establish a five-second hardware
overrun. The native search limitation is separately established by the pinned
source chain above. A host benchmark would not supply that missing bound.

Focused command:

```sh
CARGO_TARGET_DIR=scratch/cpu0-cadence-20260913/cargo-target \
CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 \
cargo test -p bitaxe-stratum cancellation_tests -- --test-threads=1
```

The focused run passed both tests, with 325 tests filtered out, in 5.06 seconds.
Workspace and canonical verification remain the parent task's coordinated
responsibility.

## Required continuation

Resolve this as a separately reviewed bounded/cancellable crypto implementation
and dependency-source qualification, preserving the frozen wire semantics and
deadlines. A narrow pinned upstream change or reviewed dependency patch is
preferable to duplicating the handshake or silently replacing cryptography. No
downloaded registry source was modified by this audit.

The necessary boundary must include:

1. A fallible bounded scalar-generation path and cancellable/bounded native
   ElligatorSwift candidate search, with no partial encoding returned as valid.
1. Propagated cancellation through both act-one construction and act-two
   authentication; checks around remaining finite cryptographic operations and
   target-specific proof that any indivisible portion fits the cleanup budget.
1. Ordinary RAII cleanup of every early-exit path, zeroizing repo-owned secret
   buffers and truthful parent-observed completion rather than detached work.
1. Deterministic cancellation during each real operation and adversarial
   candidate/entropy behavior, independent transcript interoperability tests,
   and source/provenance review of any patched dependency.
1. Only then the asynchronous controller lifecycle and native transport, native
   resource review with ordinary owners, and the contract's later live startup
   and hardware gates. None of these obligations is satisfied by this report.

A cooperative dependency seam could satisfy the existing contract without
changing its public interface. Its exact implementation and native feasibility
have not been established here. Until that work passes, runtime readiness and
the live task remain blocked; fixture and Gate software work can retain their
bounded independent value without gaining hardware authority.

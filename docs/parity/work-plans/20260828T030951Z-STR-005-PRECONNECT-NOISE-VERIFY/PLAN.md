# Precompute Noise act one and verify STR-005

- Run ID: `20260828T030951Z-STR-005-PRECONNECT-NOISE-VERIFY`
- Parity row: `STR-005`
- Initial status: `implemented`
- Source base: `c46069a1cd78a302b57f9e7893c1a0a01e891d38`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-str005-preconnect-noise-and-verification`
- Continues: `docs/parity/work-plans/20260826T210025Z-STR-005-NOISE-DIAGNOSTIC/CLOSURE.md`

## Objective

Move expensive Rust Noise act-one preparation before TCP connection, prove an
authenticated no-mining local handshake, then run one bounded local-fixture
campaign and promote STR-005 only after an accepted share plus exact original
restoration.

## Pre-connect transport fix

Add a non-debuggable `PreparedNoiseInitiator` holding the initialized initiator
and exact 64-byte act one. Both diagnostic and production V2 paths must order
effects as resolve, prepare Noise/act one, connect, configure, write act one,
read act two, and complete Noise.

First add the red-capable real-TCP regression:

`cargo test -p bitaxe-stratum-v2-fixture preconnect_act_one_avoids_accept_deadline -- --nocapture`

It must reproduce the legacy split under a pinned short server deadline, then
pass only when preparation precedes connection. Firmware ownership coverage must
prove the connector is unreachable before successful preparation.

Retain closed monotonic timings for keypair preparation, act-one construction,
connect, write, and act-two read. Durations are bounded and value-free; measured
preparation above 60 seconds stops before connect as `preparation_slow`.

## Diagnostic ordinal 4

Rebind `stratum-v2-noise-diagnostic preflight|start` to:

- root `scratch/str005-noise-diagnostic/diagnostic-004`;
- projection `docs/parity/evidence/str005-noise-diagnostic/noise-diagnostic-projection-004.json`;
- ordinal 4 and this plan; and
- restore root `diagnostic-004/restoration`.

Require exact peer admission, all 64 act-one bytes within the existing ten-second
post-accept deadline, authenticated Noise, encrypted client proof, independent
projection validation, and exact recovery-006 firmware/settings restoration.

`preparation_slow` or a typed completion failure may continue only after a
real-boundary regression and targeted fix. A repeated zero-byte timeout after
precomputation is terminal. Never retry unchanged or reuse a sealed root.

## Campaign attempt 008

Only after accepted diagnostic evidence is committed, pushed, and repackaged,
rebind campaign preflight, runtime admission, campaign, and restoration to:

- root `scratch/str005-stratum-v2/attempt-008`;
- campaign ordinal 8 and this plan;
- exact-peer authenticated local fixture;
- Ultra 205 at 400 MHz, 1100 mV, and 100% fan; and
- the existing 180-second safety ceiling.

Acceptance requires Noise authentication, channel opening, target/job receipt,
BM1366 work, qualified nonce, encrypted accepted share, complete safe-stop,
USB/process cleanup, and exact recovery-006 identity/settings/theme restoration.
A later ordinal is eligible only after a new closed signature and verified fix;
stop immediately on a repeated signature or unresolved partial transfer.

## Evidence and completion

Private roots are mode `0700` and files mode `0600`. Public evidence contains
only closed categories, bounded timings/counts, booleans, digests, safe
provenance, cleanup, and redaction status.

Before every hardware ordinal run ordered Cargo gates, Bright Builds, all Bazel
tests, canonical ESP32-S3 build/package, real-child tests, parity/progress,
reference cleanliness, selector lineage, sensitive-value review, redaction, and
final diff review. Require clean push, fresh root, fresh detector, and no-effect
preflight.

On success create `RESULT.md`, transition only STR-005 to `verified` with
`unit,golden,workflow,hardware-regression`, synchronize progress, archive the
new task and directly superseded STR-005 diagnostic/campaign continuation
records, final-verify, commit, and push.

On terminal failure publish the independently validated failed projection,
restore exactly, keep STR-005 `implemented`, record a truthful closure, and
create no hardware-regression evidence.

## Non-claims

External pools, production-pool interoperability, mixed-protocol fallback,
other boards, direct UART/pins, raw NVS/coredump access, new baselines, fault
injection, OTA, erase, arbitrary writes, unbounded mining, and release readiness
remain outside this plan.

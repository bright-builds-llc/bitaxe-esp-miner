# Parity work plan

- Run ID: `20260812T122256Z-STR-006`
- Parity row: `STR-006`
- Initial status: `implemented`
- Source commit: `8789f99abc885f41f89cf07981661a367be06233`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-str006-protocol-coordinator-promotion`

## Selection

The clean synchronized selector reports no open plan and returns `STR-006`
first, followed by `STR-007` and the remaining unfinished rows. No candidate is
skipped. The prior `STR-006` plan is validly closed without verification after
its only projection attempt exposed one over-broad source guard: the literal
`AsicWorkerCommand::Dispatch {` occurs in both the worker command consumer and
the effect-to-command mapper. Every other admitted semantic fragment remains
unique, all prerequisite evidence remains valid, and the row is immediately
actionable as a bounded software fix.

## Scope and non-scope

Replace only the false uniqueness assumption for the ASIC-worker module with a
closed assertion that admits exactly the two required dispatch spans: one span
must execute the admitted command through the production executor and one span
must map `ProductionSessionEffect::DispatchAsic` into the worker command. Keep
all other source, digest, lineage, cleanliness, independent-validator, atomic
publication, and redaction gates unchanged.

Add a regression using the production-shaped two-span ASIC worker source so the
real admitted module cannot be replaced by an under-shaped one-occurrence fake.
Also reject a missing, duplicate, reordered, or unbound span. Permit exactly one
new software-only projection attempt after a clean pushed implementation.

No protected campaign input, detector, package build, flash, reset, USB or
network session, credentials, mining, pool contact, fan/voltage/power/ASIC
actuation, recovery, direct UART, pins, or other hardware effect is permitted.
No raw protocol, device, network, credential, or local-path value may enter the
public projection.

## Implementation

- [ ] Replace the broad ASIC dispatch token with two uniquely identifying,
      ordered production spans while leaving all other source guards intact.
- [ ] Add production-shaped behavior regressions for the accepted two-span
      source and missing, duplicate, reordered, or unbound span failures.
- [ ] Produce the existing closed protocol-coordinator projection from the
      clean pushed implementation and independently validate it.

## Verification and promotion

Run focused automation tests and the real child-process seam, then the mandatory
ordered repository gate:

1. `cargo fmt --all`
2. `cargo clippy --all-targets --all-features -- -D warnings`
3. `cargo build --all-targets --all-features`
4. `cargo test --all-features`
5. `bun scripts/bright-builds-check.ts all`
6. `just test`
7. `just parity`
8. `just parity-progress`

Also require generated-contract verification, `just verify-redaction`,
`just verify-reference`, exact reference cleanliness, task uniqueness, this
immutable plan digest, source projection validators and digests, absence of
sensitive public values, mode `0644` publication only after mode `0600`
candidate validation, and `git diff --check`.

Promote only `STR-006` from `implemented` to `verified` with
`unit,workflow,hardware-smoke,hardware-regression` when the corrected closed
guard accepts exactly the two legitimate ASIC dispatch spans and the complete
coordinator projection passes independent validation from a clean pushed
commit. Any failure withholds evidence, leaves the row `implemented`, and ends
this plan without retry or hardware fallback.

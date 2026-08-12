# Parity work plan

- Run ID: `20260812T133713Z-STR-007`
- Parity row: `STR-007`
- Initial status: `implemented`
- Source commit: `b90d88c77dbc093d1ad7388a292c99856baf5f72`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-str007-mining-criteria-promotion`

## Selection and prior closures

The clean synchronized selector again selects `STR-007`. The first closed
ordinal proved that the projector Bazel target injects its command token. The
second added that regression and successfully created a closed projection, but
its separate `bazel run` validator command could not resolve a repository-
relative path from the runfiles working directory. The immutable stop rule
removed that unvalidated output. No candidate or projection remains, and no
hardware or protected evidence was used.

## Scope and non-scope

Add a repository-owned validator command boundary that canonicalizes the input
to an absolute filesystem path before `bazel run`, plus a regression for that
path conversion. Exercise the real validator boundary before publication,
rerun every gate, commit and push the guard, and then permit one atomic
software-only transaction: one flags-only projector invocation followed by
one independent validation through the guarded command.

Preserve the existing closed v1 evidence contract, exact public Phase 21 and
STR-006 input digests, source/cleanliness guards, candidate validation and
atomic rename, sensitive-value denylist, and row-specific promotion boundary.

No protected campaign artifact may be opened, copied, or summarized. No
detector, package build, flash, reset, USB/network session, credential input,
mining, pool contact, fan/voltage/power/ASIC actuation, recovery, direct UART,
pins, attempt-005, or other hardware effect is permitted. This plan does not
reopen the terminal attempt-004 continuity task or claim successful current
600-second continuity, accepted/rejected current soak behavior, arbitrary
pools, other boards, updates, recovery, profitability, unbounded mining, or
release readiness.

## Implementation

- [ ] Add a narrow repository-owned validation script/command that rejects a
      missing input and passes an absolute path to the existing Rust validator.
- [ ] Add a focused regression proving the path is absolute at the child
      process boundary and repository-relative input cannot be lost to Bazel's
      runfiles working directory.
- [ ] Run focused and complete gates, then commit and push the guard from a
      clean synchronized head.
- [ ] Run exactly one projector plus guarded independent-validator transaction,
      withholding output on any failure.

## Verification and promotion

Run the mandatory ordered repository gate:

1. `cargo fmt --all`
2. `cargo clippy --all-targets --all-features -- -D warnings`
3. `cargo build --all-targets --all-features`
4. `cargo test --all-features`
5. `bun scripts/bright-builds-check.ts all`
6. `just test`
7. `just parity`
8. `just parity-progress`

Also require focused wrapper, validator-boundary, contract, projector, and
real-child tests; generated-contract verification; independent projection
validation; exact plan and admitted-input digests; mode `0644`; absent
candidate; current source and reference cleanliness; redaction and reference
verification; task uniqueness; sensitive-value scanning; and diff checks.

Promote only `STR-007` from `implemented` to `verified` with
`workflow,hardware-smoke,soak` if the projection and independent validator pass
every gate while binding only the committed bounded Phase 21 proof, verified
STR-006 coordinator compatibility, and current fail-closed criteria. Any
failure removes or withholds the projection, leaves the row implemented,
closes this plan, and permits no retry or hardware fallback.

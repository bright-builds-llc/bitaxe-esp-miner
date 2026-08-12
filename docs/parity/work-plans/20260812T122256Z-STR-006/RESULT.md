# Parity work result

- Parity row: `STR-006`
- Final status: `verified`
- Implementation commit: `d6059a4330de070cca92b09346ac24a91ecd1300`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`

## Evidence and verification

The closed `bitaxe-protocol-coordinator-evidence-v1` projection is committed at
`docs/parity/evidence/str006-protocol-coordinator/protocol-coordinator-projection.json`
with SHA-256
`f008171f26b7a8ae6b08859e3cfef4f0c5bf88937c049dd66b6f868c9bbfd6f7`.
The immutable plan SHA-256 is
`d35415c87cc640f29749fcac4fa53132b7391e9e3e929b5ad2f2d0d1cb45f9da`.

From exact clean pushed commit
`d6059a4330de070cca92b09346ac24a91ecd1300`, the projector independently
validated and joined these accepted public proofs:

- initialization SHA-256
  `eee750561a7c1dcec1a5698b1e5827d3f1508d43655c3c4aa237097338dcf8d4`;
- work-send SHA-256
  `447af65ae9e6cd5cc2199ef639ff8e0fa7f63d4c9708570bd66781c5a162e80c`;
- result-parsing SHA-256
  `e99c054c4d660155d5c2b1ee38d3f17aed5ae7101e7e4a5fd1c6451d1b48b7c7`;
- Stratum socket SHA-256
  `dcb3eed396a268114b017d7ef4fbca9c427a390d7acf405fc52fbef6472122b8`.

They share accepted attempt source
`3e0966a140edbff1a14d2a48ca63d140649762c0` and the pinned reference above.
The projector verified accepted-source ancestry, unchanged coordinator modules,
clean relevant paths, unique lifecycle semantics, the exact ordered pair of
ASIC worker dispatch spans, and ordered terminal safe stop. It wrote a
mode-0600 candidate, passed the independent Rust validator, atomically renamed
it, and published the final projection at mode 0644. A second direct Rust
validation using the absolute evidence path passed; the explicit public
sensitive-value scan found no matches.

Verification passed:

- focused Rust contract and TypeScript projector tests;
- production-shaped missing, duplicate, reordered, and unbound dispatch-span
  regressions plus the real child-process/file seam;
- `cargo fmt --all`;
- `cargo clippy --all-targets --all-features -- -D warnings`;
- `cargo build --all-targets --all-features`;
- `cargo test --all-features`;
- `bun scripts/bright-builds-check.ts all`;
- `just test` with all 37 Bazel tests passing;
- `just parity` and `just parity-progress`;
- generated-contract, redaction, reference, reference-cleanliness,
  immutable-plan, task-uniqueness, source-digest, file-mode, and diff checks.

## Conclusion

The evidence proves the accepted conservative Ultra 205 lifecycle traversed
the unchanged production protocol coordinator through all six readiness gates,
hardware preparation before pool access, authorization before ASIC dispatch,
qualified result correlation before submit, a real accepted submit response,
ordered fail-closed safe stop, lease cleanup, and USB cleanup. Current source
also proves one bounded owner inbox, single-owner serialization, the 1,000-ms
readiness reread cadence, and owner-loop watchdog feeding. This is sufficient
to promote only `STR-006` to `verified` with
`unit,workflow,hardware-smoke,hardware-regression`.

## Non-claims and residual risks

This result does not prove fallback or reconnect hardware behavior,
long-running coordination or watchdog timing under sustained load, arbitrary
pools, rejected-submit behavior on hardware, automatic fan control, voltage or
thermal fault handling, profitability, upstream-default or unbounded mining,
TLS, Stratum v2, other boards, updates, recovery, or release readiness.

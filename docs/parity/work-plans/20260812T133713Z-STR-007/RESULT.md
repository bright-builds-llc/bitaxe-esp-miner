# Parity work result

- Parity row: `STR-007`
- Final status: `verified`
- Implementation commit: `381ddb5af93a84a48c4e410a32463e8b621e44bc`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`

## Evidence and verification

The closed `bitaxe-mining-criteria-evidence-v1` projection is committed at
`docs/parity/evidence/str007-mining-criteria/mining-criteria-projection.json`
with SHA-256
`c1ccb65e6a49d04049aabb2be1295949163526a197e20e3de51fc65d38c2a80f`.
The immutable plan SHA-256 is
`58424e52830a91acc8586d2a82b3089cb740f29d3b7e64767cc12101fa304922`.

From exact clean pushed commit
`381ddb5af93a84a48c4e410a32463e8b621e44bc`, the projector independently
validated and joined these committed public proofs:

- Phase 21 summary SHA-256
  `b411ed3d8a1ce427231ec2818ed74fb590e6b29e4539a0e131bfdc7bc7acec0c`;
- controlled no-share smoke SHA-256
  `faec052c13b55cc7a53a1206c25c2094d93945d4b17d69c17c8a976e860655ff`;
- approved 300-second bounded soak SHA-256
  `fc8904a9d9e2132789d70a9886c8aef05be96134e1ccd4d29bc793c9efa66003`;
- verified STR-006 coordinator SHA-256
  `f008171f26b7a8ae6b08859e3cfef4f0c5bf88937c049dd66b6f868c9bbfd6f7`.

The projector also admitted unique current source spans proving exact
600-second criteria, upstream-default profile admission, authorized active-
duration accounting, full-duration enforcement, accepted-share and network-
correlation requirements, terminal safe stop and cleanup, private evidence,
and redaction. It wrote a mode-0600 candidate, passed its internal independent
Rust validator, atomically renamed it, and published mode 0644. The separate
repository-owned validator canonicalized the projection to an absolute path;
the existing Rust validator opened and accepted it through Bazel. No candidate
remains, and the explicit public sensitive-value scan found no matches.

Verification passed:

- focused Rust contract, TypeScript projector, invocation-wrapper, validator-
  boundary, and real-child tests;
- `cargo fmt --all`;
- `cargo clippy --all-targets --all-features -- -D warnings`;
- `cargo build --all-targets --all-features`;
- `cargo test --all-features`;
- `bun scripts/bright-builds-check.ts all`;
- `just test` with all 37 Bazel targets passing;
- `just parity` and `just parity-progress`;
- generated contracts, redaction, reference and reference cleanliness, exact
  plan and input digests, file mode, candidate absence, task uniqueness,
  sensitive-value denial, source cleanliness, and diff checks.

## Conclusion

The sealed evidence proves the repository has explicit bounded mining smoke
and soak criteria grounded in the committed Phase 21 controlled no-share
hardware evidence, compatible with the verified coordinator, and strengthened
by current fail-closed 600-second upstream-default admission and completion
rules. This is sufficient to promote only `STR-007` to `verified` with
`workflow,hardware-smoke,soak`.

## Non-claims and residual risks

This result does not reopen or satisfy the terminal default-profile attempt-004
continuity task. It does not prove a current accepted or rejected share during
a 600-second soak, uninterrupted current HTTP/WebSocket or watchdog continuity,
automatic controls, arbitrary pools, profitability, unbounded mining, TLS,
Stratum v2, other boards, updates, recovery, or release readiness. No hardware
rerun or protected campaign access was used for this projection.

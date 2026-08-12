# STR-001 result

## Outcome

The closed `bitaxe-stratum-socket-evidence-v1` projection passed independent
Rust validation and proves the accepted conservative Ultra 205 session used
the production Stratum v1 TCP adapter now present in the repository.

The admitted hardware source is the exact attempt at commit
`3e0966a140edbff1a14d2a48ca63d140649762c0`. Its validated initialization
projection proves exact package admission, trusted runtime identity, completed
hardware preparation, live initialized work, a real accepted submit response,
fresh safety, confirmed safe stop, lease cleanup, and USB cleanup. The complete
TCP transport module is byte-for-byte unchanged from that source to committed
projector source `d0a91d3662046a1350e89f872c59e21a4bce73c2`.

Current source inspection additionally proves the compatible unique owner and
lifecycle spans still map typed connect, write, close, connected, bytes,
failure, and closed operations through transport epochs, and only classify an
accepted response for a matching pending submit in the authorized active
session. Current bounded adapter facts are an eight-command queue, 5,000-ms
connect timeout, 50-ms read timeout, 2,000-ms write timeout, 2,048-byte read
buffer, and TCP no-delay.

## Evidence

- Projection:
  `docs/parity/evidence/str001-socket/stratum-socket-projection.json`
- Projection SHA-256:
  `dcb3eed396a268114b017d7ef4fbca9c427a390d7acf405fc52fbef6472122b8`
- Source initialization projection SHA-256:
  `eee750561a7c1dcec1a5698b1e5827d3f1508d43655c3c4aa237097338dcf8d4`
- Immutable plan SHA-256:
  `86391ada9b048929534cc5e2cd4bb290fcaf089517109a44f3adcfb3310678ea`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`

The projector validated the prerequisite with the existing independent
validator, verified accepted-source ancestry, rejected relevant dirty paths,
required the unchanged complete transport module and compatible owner/lifecycle
spans, wrote a mode-0600 candidate, independently validated it, atomically
renamed it, and published the final file at mode 0644. Semantic redaction and
the explicit sensitive-value scan pass.

## Verification

- Focused Rust contract tests
- Focused TypeScript projector and real-child tests
- Canonical generated-contract verification
- Production transport loopback and production session tests through `just test`
- `cargo fmt --all`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo build --all-targets --all-features`
- `cargo test --all-features`
- `bun scripts/bright-builds-check.ts all`
- `just test`
- `just parity`
- `just parity-progress`
- `just verify-redaction`
- `just verify-reference`
- immutable-plan, prerequisite-digest, source-compatibility, task-uniqueness,
  reference-cleanliness, generated-contract, mode, and diff checks

## Promotion boundary

This evidence is sufficient to promote only `STR-001` to `verified` with
`unit,workflow,hardware-smoke,hardware-regression`. It does not prove fallback
or reconnect hardware behavior, upstream timeout/keepalive equivalence, DNS or
IP-family preference parity, arbitrary pool compatibility, TLS, Stratum v2,
unbounded socket stability, profitability, other boards, updates, recovery, or
release readiness.

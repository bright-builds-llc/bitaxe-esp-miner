# Parity work plan

- Run ID: `20260802T201136Z-STR-004`
- Parity row: `STR-004`
- Initial status: `implemented`
- Source commit: `fd656f10f95a3a1a81eef2f35d6141cff3255020`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-str-004-coinbase-decoder-verification`

## Selection

`bazel run //tools/parity:report -- next-item --format json` reported no open
plan. It ranked the following candidates before `STR-004`; each remains
ineligible for this invocation:

- `CFG-001`: the remaining frequency/voltage-default claim is explicitly
  safety-critical and lacks a row-specific authorized hardware contract.
- `CFG-005`: complete API PATCH and firmware NVS-adapter behavior remains
  absent, so the existing pure update model cannot close the broad runtime row.
- `NET-001`: retry reasons, fallback timing, IPv6, and long-running reconnect
  parity require network/hardware evidence without a row-specific task gate.
- `ASIC-002`: full BM1366 initialization and active safety prerequisites retain
  explicit hardware-regression gaps.
- `ASIC-003`: production work dispatch remains below verified without
  row-specific live hardware evidence.
- `ASIC-004`: valid live nonce/result parsing and share outcomes remain below
  verified without row-specific live hardware evidence.
- `ASIC-005`: accepted serial transport under mining load remains below
  verified without row-specific live hardware evidence.
- `ASIC-007`: live frequency transition is safety-critical and lacks an
  authorized hardware-regression contract.
- `STR-001`: the live socket adapter and hardware-backed lifecycle evidence
  remain incomplete.

`STR-004` is the first bounded software-verifiable candidate. The pinned
reference exposes deterministic coinbase transaction parsing for compact-size
integers, block height, ScriptSig text, outputs, totals, and BIP-54/BIP-110
signals. Address encoding remains separately owned by `STR-012`.

## Scope and non-scope

Implement a safe, typed decoder for the deterministic coinbase transaction
structure represented by a Stratum v1 notify: compact-size integers, BIP-34
block height, printable ScriptSig projection across the coinbase split,
sequence, bounded output values and scripts, total output value, lock time,
and the upstream BIP-54/BIP-110 signal decisions.

Preserve the existing hashing, Merkle, extranonce, and malformed-hex behavior.
Do not modify the pinned reference tree, copy GPL source expression, add
network or hardware effects, access credentials, encode payout addresses,
claim user-payout matching, start mining, dispatch ASIC work, parse ASIC
results, classify live shares, or claim timing or production parity. Base58,
Bech32/Bech32m, and network-specific address rendering remain `STR-012` work.

The plan follows repository task/archive, reference-integrity, and
evidence-privacy guidance plus the Bright Builds architecture, code-shape,
verification, testing, and Rust standards loaded for this run.

## Implementation

- [ ] Add a typed, bounds-checked coinbase decoder behind the existing
      `v1::coinbase` module without introducing address-codec dependencies.
- [ ] Add a pinned, provenance-bearing golden fixture and focused tests for all
      compact-size widths, transaction fields, signals, output bounds, and
      malformed/truncated inputs.
- [ ] Keep address rendering and user-payout matching explicitly unrepresented
      so the type cannot imply unverified `STR-012` behavior.
- [ ] Record exact commands, evidence, non-claims, and residual risks in
      `WORKLOG.md` and a terminal `RESULT.md` if promotion criteria pass.

## Verification and promotion

Focused acceptance commands:

- `cargo test -p bitaxe-stratum coinbase_decoder --all-features`
- `bazel test //crates/bitaxe-stratum:tests`
- `bazel run //scripts:verify_reference_clean`
- `jq empty crates/bitaxe-stratum/fixtures/v1/coinbase-decoder-cases.json`

The mandatory Rust checks, managed Bright Builds checks, full `just test`,
redaction verification, parity validation, and parity-progress validation must
also pass. Promotion to `verified` requires executable `unit,golden` evidence
for every in-scope deterministic decoder field and boundary, exact provenance
at the pinned reference commit, fail-closed truncation/overflow behavior, and
explicit preservation of `STR-012` plus all live/effectful non-claims. This
software-only plan enters no detector, credential, recovery, retry, destructive,
or hardware gate.

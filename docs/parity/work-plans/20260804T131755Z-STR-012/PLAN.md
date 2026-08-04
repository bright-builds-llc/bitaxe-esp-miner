# Parity work plan

- Run ID: `20260804T131755Z-STR-012`
- Parity row: `STR-012`
- Initial status: `not-started`
- Source commit: `08a76a8efcce7228dd3667d01b74b26152e78cfd`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-str012-payout-address-codecs`

## Selection

`bazel run //tools/parity:report -- next-item --format json` reported no open
plan after `V12-RUNTIME-HEALTH-205` closed. The candidates before `STR-012`
remain ineligible under their existing audited boundaries:

- `CFG-001`, the power/thermal rows, `SELF-001`, and the ASIC initialization,
  transport, work/result, and frequency rows retain safety-critical actuation,
  sensor, fault, or loaded-hardware gaps. Their prior attempts are terminal or
  non-promotable and cannot be repurposed.
- `CFG-005`, `NET-001`, `API-002`, `API-003`, `API-009`, and `LOG-001` remain
  broader than the closed hostname, snapshot, and runtime-health evidence;
  complete PATCH, reconnect/fallback/IPv6, full-field, command-effect, or log
  lifecycle behavior is still missing.
- `STR-001`, `STR-006`, `STR-007`, the production mining rows, and their safety
  rows require promotable live transport, ASIC correlation, share, safe-stop,
  soak, or watchdog evidence that the consumed terminal attempts withhold.
- `REL-001` through `REL-003` retain selected-partition, rollback, recovery,
  OTAWWW, destructive fault-injection, or release-readiness gaps.
- The earlier in-progress and not-started system, network, non-BM1366 ASIC,
  I/O, display, UI, statistics, and BAP rows require broader implementation or
  unavailable hardware/operator surfaces rather than a closed pure contract.

`STR-012` is the first independently actionable candidate. The pinned
reference provides MIT-licensed Base58Check and SegWit codec behavior and
golden vectors; the Rust Stratum crate already owns double SHA-256 and
structural coinbase script classification but explicitly reserves address
rendering for this row. No hardware, network, credential, or external service
is required.

## Scope and non-scope

Add a pure `v1::payout_address` module with typed Bitcoin network and address
kind values. Implement canonical Base58Check encode/decode with double-SHA-256
checksum and leading-zero rules; Bech32/Bech32m SegWit encode/decode with
human-readable-part, case, length, witness-version, program-length, checksum,
and padding validation; standard P2PKH, P2SH, P2WPKH, P2WSH, and P2TR script
address rendering; and payout-address-to-script validation for mainnet,
testnet, and regtest.

Use only the existing `sha2` dependency and standard library. Keep errors
typed and closed; never panic or accept noncanonical aliases. Preserve the
current coinbase decoder API and raw-script projection so this work does not
silently introduce configured user addresses into logs or evidence.

Do not read or persist an owner address, alter pool or NVS settings, connect to
a pool, start mining, dispatch ASIC work, touch firmware or hardware, claim
that a configured payout received funds, implement Stratum V2 key decoding, or
promote any other row. Address values in committed fixtures are public standard
test vectors, not local owner inputs.

## Implementation

- [ ] Add the typed codec module and minimal public API for Base58Check,
      SegWit, standard output-script rendering, and payout-script validation.
- [ ] Add upstream-derived and standards-derived golden fixtures covering
      mainnet, testnet, regtest, Bech32, Bech32m, and every supported standard
      script kind with explicit provenance.
- [ ] Add fail-closed tests for checksum, alphabet, canonical zero, mixed case,
      HRP, witness version, encoding variant, program length, padding, network,
      and script mismatch boundaries.
- [ ] Record focused and repository-wide verification in `WORKLOG.md`; create
      `RESULT.md` only if every codec and validation acceptance criterion passes.

## Verification and promotion

Focused verification will run the `bitaxe-stratum` Cargo and Bazel tests plus
golden fixture checks. Final verification will run, in order, `cargo fmt
--all`, strict all-target/all-feature Clippy, all-target/all-feature build,
all-feature tests, Bright Builds checks, `just test`, `just parity`,
`just parity-progress`, redaction, reference cleanliness, and diff checks.

Promote only `STR-012` to `verified` with `unit,golden` evidence when public
reference vectors round-trip exactly, invalid and cross-network inputs fail
closed, all five supported standard output-script kinds render and validate,
the fixture provenance is explicit, no new dependency or effectful path is
introduced, and all gates pass. Otherwise leave the row at `implemented` with
the exact failing boundary. No hardware attempt, recovery, or authorization
surface exists for this plan.

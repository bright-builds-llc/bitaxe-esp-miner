# SAFE-11 Production Blocker Reason Evidence

safe11_status: accepted
board: 205
source_commit: 0fee49423ec0c87becd3b363135ce051647fdeac
reference_commit: c1915b0a63bfabebdb95a515cedfee05146c1d50
redaction_status: passed
exact_non_claims: live fault injection, individual active control effects,
self-test, BAP/UART, other boards or ASICs, arbitrary profiles or pools,
unbounded mining, OTA/recovery, and release readiness

## Identity

| Input | Path or identity | SHA-256 |
| --- | --- | --- |
| Immutable plan | `docs/parity/work-plans/20260818T140738Z-SAFE-11/PLAN.md` | `da6d3fb92ee912c0025903b56202d45b152dd43ac82df292f8b2a896f36cfa47` |
| Detector-gated live safety hardware proof | `docs/parity/evidence/safe10-prerequisite-readiness/safe10-projection.json` | `4e9b91bd29629aec098b9967b9bb27b9c1358f64c11819a77f8c8da4c212a20e` |
| Production blocker model | `crates/bitaxe-stratum/src/v1/recovery_policy.rs` | `50c29e96d47fbe1898b6454f7fbfe61b9ff85ed5598851205b6526d306c55b82` |
| Production snapshot projection | `crates/bitaxe-stratum/src/v1/production_session/runtime.rs` | `c2baee573d56f4464b1afbe10d6b4fd699b14917a1ab723580a93ec058a765b7` |
| Mining runtime state | `crates/bitaxe-stratum/src/v1/state.rs` | `2862fb801ca5b407790ea8ca9f3ad9f4564903817538936e5bd2b007caa9b3d4` |
| API projection and regressions | `crates/bitaxe-api/src/mining.rs` | `5f852d0ae1fe9a48c1e44ec5ad06b070d6c5de22bc43a3da71cdc193322628ba` |
| Firmware readiness shell | `firmware/bitaxe/src/production_mining_session.rs` | `0e6bad9b4ef07c6eea5dcff25ad3b7045d2531ae3b4ec87c7f4ef5728da30ee4` |
| Corrected reason ledger | `docs/parity/evidence/phase-22-claim-ladder-and-safety-preconditions/blocker-reasons.md` | `0437c0393d853927d87d670c6cfcdfb682d667239faa9ca75ef276aa5c9824ba` |
| Upstream coordinator | `reference/esp-miner/main/tasks/protocol_coordinator.c` | `116294341e3f35d05131090fd540e651e114f3ca604e354d051702231bbc7260` |
| Upstream system state | `reference/esp-miner/main/system.c` | `bdd1de18ab21e7975c8cf548291a6b0dce050e031e32d218a62733edb5c5d079` |

The SAFE-10 projection is mode `0644` and passed its independent Rust validator
at the implementation source. The only implementation-commit changes are the
API regressions and corrected reason ledger. A direct diff over all nine
SAFE-10 production-inventory paths is empty, so the accepted live prerequisite
semantics remain byte-identical to their accepted projection.

## Closed Result

| Fact | Accepted result |
| --- | --- |
| Typed production labels | 17 |
| Unique lower-snake-case labels | 17 |
| Work-disabled blocker cases | 17 |
| Operator-controlled paused cases | 1 |
| Fail-closed `safe_blocked` cases | 16 |
| Exact API failure reasons | 16 |
| API failure reason for operator pause | empty |
| Ready-state stale-reason suppression | passed |
| Readiness blockers prevent secret network/ASIC effects | passed |
| SAFE-10 independent validation | passed |
| Pinned reference cleanliness | passed |

The API regressions enumerate every current `ProductionSessionBlocker`. They
prove the label vocabulary is unique and restricted to lowercase ASCII letters,
digits, and underscores. Applying every failure label through the same mining-
state operation used by production disables submission, preserves
`safe_blocked`, and exposes the exact label as `blockedReason`. The operator-
pause projection also disables submission but retains `paused` and exposes no
failure reason. Allowing work clears the stored reason before API projection.

The existing production-session lifecycle regression independently proves that
each readiness blocker prevents secret-bearing network and ASIC effects. The
accepted SAFE-10 projection and
`docs/parity/evidence/phase-28-hardware-evidence-and-checklist-promotion/summary.md`
provide the detector-gated live safety hardware proof required by the checklist;
this result does not reinterpret protected values or claim live fault stimulus.

## Reference Comparison

Pinned upstream `protocol_coordinator.c` marks all configured pools unavailable
and pauses mining after retry exhaustion. Upstream power management consumes
operator pause, hardware fault, or pool unavailability as stop conditions before
resuming only when the condition clears. Rust preserves that observable fail-
closed structure while using a finer stable operator reason taxonomy. Exact
wire equality with upstream log text is not claimed.

## Commands

The following passed at source commit
`0fee49423ec0c87becd3b363135ce051647fdeac`:

- `cargo test -p bitaxe-api mining::tests -- --test-threads=1`
- `cargo test -p bitaxe-stratum every_readiness_blocker_prevents_secret_network_and_asic_effects -- --test-threads=1`
- `bazel test //crates/bitaxe-api:tests //crates/bitaxe-stratum:tests --test_output=errors`
- `just validate-safe10-evidence docs/parity/evidence/safe10-prerequisite-readiness/safe10-projection.json`
- `just verify-reference`
- `cargo fmt --all`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo build --all-targets --all-features`
- `cargo test --all-features`
- `bun scripts/bright-builds-check.ts all`
- `just test`
- `just parity`
- `just parity-progress`

## Privacy and Non-Claims

This evidence contains only repository paths, commits, digests, closed labels,
counts, booleans, and command outcomes. Direct review found no credentials,
owner/pool/worker values, endpoints, ports, USB/network identity, telemetry,
raw logs/payloads, protected identifiers, or secret values.

This result does not prove live fault injection, each individual voltage/fan/
thermal/power actuation, self-test, BAP/UART, non-205 behavior, other ASICs,
arbitrary profiles or pools, unbounded mining, OTA/recovery, or release
readiness. Those remain separate parity rows and evidence gates.

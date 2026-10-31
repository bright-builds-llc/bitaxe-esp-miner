# Parity work result

- Parity row: `API-004`
- Final status: `verified`
- Implementation commits: `361e26ea` and `6269d90c`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`

## Evidence and verification

The redacted summary is `docs/parity/evidence/20261030-access-gate-peer-address.md`. Each run used `just flash-monitor` to install
production state-preserving, then `just access-gate-check` on that boot: Gamma 601
attempt 002 at `6269d90c` and Ultra 205 attempt 003 at `0c0698db`. The captures stay
private, with their addresses.

## Revision

The notes gain this passage:

Later evidence, outside the artifacts above, adds the access gate, `is_network_allowed` in the same upstream file. `firmware/bitaxe/src/http_api/access.rs` now reads the peer of the `PF_INET6` server socket as `sockaddr_in6`, and `crates/bitaxe-api/src/route_shell.rs` denies a peer that is unreadable, zero or not IPv4-mapped, which fixes a read that left the gate checking only the Origin. `just access-gate-check` on production Gamma 601 `6269d90c` and Ultra 205 `0c0698db` returned 200 with no Origin, 200 with a private Origin and 401 with a public Origin; see `docs/parity/evidence/20261030-access-gate-peer-address.md`. A public peer being denied is shown by unit tests only, and a real IPv6 peer, the WebSocket routes, and settings and OTA routes through this gate are not shown on hardware. Known differences from upstream: a real IPv6 peer is always denied where upstream classifies its last four bytes; an Origin with a port, `https://` or no scheme is accepted where upstream denies it; `/recovery` is not gated where upstream gates it. The earlier limits still apply.

## Non-claims

Nothing beyond the redacted summary is claimed. The status, the evidence tokens and
the owned target are unchanged. The bug itself was never seen on a device: firmware
from before the fix answers these requests the same way, and the gate's denied line
never leaves the device.

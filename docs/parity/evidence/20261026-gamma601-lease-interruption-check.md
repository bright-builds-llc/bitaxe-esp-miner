# Gamma 601: the lease interruption check

Phase F3.4 of the Gamma 601 bring-up (`task-gamma-601-bring-up`, ADR-0041
items 10, 11 and 13, ADR-0038 task 6) showed on the 601 that a Gate Work
Lease interrupts local mining and that local mining comes back afterwards,
as `20261018-ultra205-lease-interruption-check.md` showed on the Ultra 205.
Two repo-owned commands ran it together:
- `just local-mining-soak --soak gamma601-lease-001` for the local phases;
- `just ultra205-soak --interrupt-local-mining` for the lease.

Attempt 005 passed both halves. Attempts 001 to 004 failed, as recorded in
`TASKS.md`:
- 001 and 002 lost the Gate link about 20 s after the last browser call:
  the Gate tab was not on screen, so the page read hidden once the browser
  extension detached. The records first took it for a covered window;
  the check before attempt 003 found a background tab in its window. The
  contract now proves the tab stays visible while detached before each
  Gate session.
- 003's Start was refused, most likely (from the code; the refusal reason
  was not readable then) because the 601 never ran the 205's original
  acceptance campaign. ADR-0041 item 13 (`2d4158fa`) dropped that
  prerequisite on the 601, by owner decision, and attempt 004's Start was
  admitted.
- 004's lease ran to completion, but our own local soak read the device's
  whole retained log inside the Gate soak's judged window. The device
  serves that log on its single web server task, so the WebSocket stream
  had a 6.36 s gap against the 5 s bound, and HTTP polls were missed for
  8.2 s. The local soak now holds that read until the Gate soak's result
  (`9323ff2d`).

Parity is not promoted.

- **Commands** (the contract in `TASKS.md`):
  1. `just detect-ultra205`; `just package-gamma601-mining`; `just
     flash-monitor --board 601 --port <port> --manifest <the 601 package
     manifest> --allow-test-firmware gamma601-mining
     --capture-timeout-seconds 360`;
  2. `just otawww-endpoint preflight|serve|finish`;
  3. `just local-mining-soak preflight --soak gamma601-lease-001 ...
     --lease-root <root>`, `pause` and `run`, the pool credentials passed
     by path only;
  4. after `lease_ready`, `just detect-ultra205`, `just ultra205-soak
     preflight ... --interrupt-local-mining`, `serve`, the page's steps in
     the dedicated Gate tab, and `just hardware-operator owner-finish
     --owner ultra205-soak`;
  5. `just package --board 601`, `just flash --board 601 --port <port>`
     and `just monitor --board 601 --port <port>
     --capture-timeout-seconds 360`.
- **Sources:** the attempt at `9323ff2d`; the Gate at `b775145`; the
  reference at `c1915b0`.
- **Manifests:** step 1's, retained privately with the captures,
  `firmware_variant=gamma601-mining`, board 601, source `9323ff2d`. Step
  5 flashed `just package --board 601`'s production package from its
  build path; its manifest was not copied into the captures, and the
  production boot shows in the safe-boot records below.
- **Pool:** `pool_config: local-owner-supplied`, the same pool through the
  lease's grant.
- **Tab visibility:** before the run, the dedicated Gate tab read visible
  throughout 530 s with no browser call.

| Observation | Result |
| --- | --- |
| Local mining before the lease | 300,035 ms accumulated |
| Refused Connect | One, at stage `hello`; local mining stopped 25 s after `lease_ready` |
| Admitted Connect | After the ASIC's safe stop; the 15-s step's reload and Connect reached `ready` |
| Idle proof | 83.9 s, 42 HTTP and 145 WebSocket samples, one connection, no broken events, no mining |
| Lease | 600,000 active ms at 400 MHz / 1150 mV, ended by its budget (`lease_or_budget_expired`); safe stop complete at `fan_paused`; restoration and baseline confirmed |
| Lease windows | 20 of 20 credited on HTTP and WebSocket; gaps at most 2,187 ms and 772 ms; no WebSocket reconnect; clock residual 153 ms |
| Lease shares | 41 accepted, 0 rejected; 28 renewals |
| After the lease | The run intent on both transports (not paused, not mining, start-on-boot off); pool settings retained |
| Soak ledger | Ordinal 2, cumulative 1,200,000 ms (attempt 004's lease was ordinal 1); cleanup confirmed |
| Restored | Local mining back 20.8 s after the local soak read the Gate soak's result (bound 180 s, measured as for the 205) |
| Local mining after | 300,870 ms accumulated |
| Local shares | 42 accepted, 0 rejected over the local phases |
| Outages | None |
| Limits over the run | At most 16.09 W, chip 60.75 C, regulator 61.5 C; input 5,203–5,258 mV; fan at least 2,867 rpm; Wi-Fi at least -47 dBm |
| Stop | Paused, ASIC confirmed powered down |
| Persistence | Mining on boot off; one boot session through the run |

- **The break log:** the held log was read once, when local mining came
  back. Its journal kept one marker (`local_mining=run_began`, none
  dropped); the handover's lines were not among them. Why was not
  investigated.

## Production afterwards

The receive-only capture after step 5 holds 57 safe-boot records:
`built=601 stored=601 identity=match`, `reset_held=true`, `off=confirmed`,
20 mV, fan 30 %, one boot. The USB profile reads `worker_started` with a
confirmed baseline. Its 114 `flush_timeout` records are replays of one
early-boot flush failure, from the reader gap between `just flash` and
`just monitor`.

- **The boot counter:** this production boot read `boot_ordinal=1`, as
  attempt 004's did, while step 1's mining boot read 9. The cause is
  unexplained and was not investigated; one boot session held through the
  whole run.

## Effects

The installs; the handoff; start-on-boot saved off; the pool and the
400/1150 profile saved; local mining before and after; one refused Connect
and its ordered safe stop; one signed 600-s soak lease at the saved
400/1150; the soak ledger charged once in this attempt; the Gate soak's
settings write (400/1150, automatic fan). There was no live change, other
profile, second lease, factory reset, network discovery beyond the handoff
and the Gate soak's own station read, direct UART or pin access.

## Not shown here

- **The 601's original acceptance campaign** and qualification attempts:
  the 601 skips the first (ADR-0041 item 13) and still refuses the second.
- **A lease interrupted** by heartbeat loss, a fault, or a network or pool
  outage on the 601.
- **Upstream's 525 MHz default** under a lease, and live changes, which
  the 601 does not admit yet.
- **Parity:** nothing is promoted; production 601 dispatch stays refused
  until phase F4.

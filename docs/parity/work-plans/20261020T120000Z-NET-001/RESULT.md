# Parity work result

- Parity row: `NET-001`
- Final status: `verified`
- Implementation commit: `a24684f1b634103ad61dfb6d6ae9e53592837d00`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`

## Evidence and verification

The redacted summaries are `docs/parity/evidence/20261019-ultra205-handover-loop-and-wifi-power-save.md`, `docs/parity/evidence/20261020-ultra205-local-mining-soak-005.md`. Each run used `just local-mining-soak`
on an exact installed Ultra 205 package, sealed `passed`, and was
independently reviewed against its private journal.

## Revision

The notes gain this passage:

Later evidence, outside the projection above, adds Wi-Fi modem power save off as upstream sets it: `reference/esp-miner/components/connect/connect.c` calls `esp_wifi_set_ps(WIFI_PS_NONE)` in both of its branches, and `firmware/bitaxe/src/wifi_adapter.rs` now sets it after each driver start, before any association, confirmed on hardware by the retained line `wifi_power_save=none`. Before it, local mining soaks met six outages in about 9.1 hours, one logged as a `beacon_timeout` station disconnect. With it, exact-package Ultra 205 soak 004 attempt 002 (2 hours) and soak 005 (8 hours) met one `beacon_timeout` in about 10 hours; the station reconnected in about 10 s on the same boot and local mining recovered; see docs/parity/evidence/20261019-ultra205-handover-loop-and-wifi-power-save.md and docs/parity/evidence/20261020-ultra205-local-mining-soak-005.md. This narrows the long-running reconnect non-claim to markedly fewer disconnects, each recovered, over 10 hours; the cause of the remaining disconnect, router/RF failures, and the remaining non-claims above still apply.

## Non-claims

Nothing beyond the redacted summaries is claimed. The status is unchanged.

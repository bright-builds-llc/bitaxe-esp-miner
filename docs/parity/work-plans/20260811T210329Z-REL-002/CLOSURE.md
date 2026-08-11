# Parity work closure

- Parity row: `REL-002`
- Final status: `implemented`
- Outcome: `blocked`
- Verification claimed: `no`
- Plan SHA-256: `412918c67b586df9fb9204f045ac677aba813c62f177d1e4185a7b6a6c1f0d5e`
- Active task: `task-parity-rel002-rollback-interruption-attempt-001`

## Closure reason

The sole detector admitted one Ultra 205 and the sole conditional attempt
installed and observed the exact normal factory package. The bounded partial
OTA request left the boot session, boot ordinal, application identity, and
factory partition unchanged through all ten same-origin checks, but none of
the retained-log checks observed the required
`firmware_ota_update=protocol_error` marker. The typed transaction therefore
stopped as `interruption_not_observed`, published no evidence, and did not
start the probe or rollback sessions.

The host interruption helper currently calls `socket.end` and returns when its
outbound prefix has flushed. It neither destroys the receive half nor waits for
the socket `close` event. The synthetic child closes its own connection after
EOF, so that regression did not cover a production server that continues
waiting for the declared body. The admitted completion condition is therefore
insufficient to prove that the device observed a connection abort before the
retained-log checks. The exact normal factory build remained confirmed, so the
transaction correctly did not spend its recovery flash.

## Next safe action

Create a fresh immutable `REL-002` continuation only after changing the
interrupted-upload transport to force and observe a bounded local socket
teardown after the strict prefix. Add a real-child regression whose server
keeps its response half open and prove the helper cannot report completion
while the connection remains live. Run the full software gate, commit and push
the fix, then authorize a new detector and attempt ordinal with new protected
paths. Attempt-001 must not be retried or reused.

## Non-claims

This closure does not verify a device-observed OTA abort, rollback-probe boot,
pending validation, native ESP-IDF rollback, `REL-002`, OTAWWW, recovery-page
behavior, mining, ASIC behavior, hardware control, other boards, or release
readiness. No public hardware evidence was produced and `REL-002` remains
`implemented`.

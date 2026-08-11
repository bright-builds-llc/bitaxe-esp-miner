# Parity work result

- Parity row: `LOG-001`
- Final status: `verified`
- Implementation commit: `f1aca309239d38c1764992794cab2aa80832d037`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Board: Ultra 205
- Hardware attempts under this plan: one

## Evidence and verification

The sole detector admitted one Ultra 205 before the exact schema-v3 package
built from clean pushed implementation commit
`f1aca309239d38c1764992794cab2aa80832d037` was flashed. The capture admitted
one trusted same-boot origin, confirmed exact package identity and passive safe
state, downloaded `/api/system/logs`, connected once to raw `/api/ws`, captured
one text-protocol connection marker, and downloaded the retained log again.

The committed
[`bitaxe-log-buffer-evidence-v1`](../../evidence/log001-retained-stream/log-buffer-projection.json)
projection has SHA-256
`a72f2a89acdfeb71e9e172b553da5875d080877f16e5879faa9da9f2dbcbc62f`.
The repository-owned Rust validator independently accepts it, and the semantic
redaction scanner includes and accepts this schema. Its closed facts prove:

- exact clean source, pinned reference, package-manifest identity, and board
  205 admission in one boot session;
- exact `text/plain` and `attachment; filename="bitaxe-logs.txt"` response
  headers on both bounded retained-log downloads;
- a 190647-byte baseline is an exact prefix of the 192602-byte final body;
- the one 31-byte WebSocket frame used the text protocol type and matched the
  retained connection marker, whose count advanced exactly from zero to one;
- mining and hardware control remained disabled, cleanup completed, and every
  private wrapper/attempt file and directory used the required modes; and
- the public projection contains only closed provenance, digests, bounded
  counts, booleans, safety labels, cleanup, and redaction status.

The post-capture evidence review found that the generic semantic-redaction
scanner did not yet recognize the new schema. No prohibited field or value was
present. The schema was added to the scanner, the capture was changed to run
that scanner before future publication, and a rejecting operational-field
regression was added. The already-captured public projection then passed the
same scanner without another hardware attempt.

The exact effect-capable commands were the immutable plan's `just package`,
private `just detect-ultra205`, and conditional `just
capture-log-buffer-evidence` commands. Focused contract, header, correlation,
plain-text-frame, failure-category, no-clobber, real-child-process, validator,
and redaction tests passed, followed by the complete ordered repository gate
and the package, selector, immutable-plan, task-uniqueness, privacy, mode,
reference, and diff checks.

## Conclusion

The exact pushed Rust firmware on one detector-admitted Ultra 205 serves
upstream-compatible retained-log downloads and emits the raw WebSocket log
marker as a plain-text frame, while the same marker is appended exactly once
to the retained buffer in the same safe boot. This satisfies `LOG-001` with
typed unit, API-comparison, workflow, and hardware-smoke evidence.

## Non-claims and residual risks

Raw detector, USB, serial, HTTP, WebSocket, log, origin, network, credential,
and process material remains only in ignored private roots. The Wi-Fi
credential file was an opaque local input, and no pool credential was read.
This result does not claim retained-log persistence across reset,
maximum-capacity live wraparound, long-duration or multi-client streaming,
mining, ASIC work, hardware controls, settings mutation, network recovery,
OTA/recovery, other boards, or release readiness.

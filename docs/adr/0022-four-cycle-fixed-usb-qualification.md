# ADR-0022: Require four fixed-USB continuity cycles

Accepted 2026-09-05 by explicit owner request. Supersedes only ADR-0021's twenty-cycle qualification sample.

## Decision

Require four successful no-mining browser-connect/release/flash/reconnect cycles before bounded real-mining acceptance. Preserve already completed valid cycles on the exact tested firmware and browser runtime; do not relabel or fabricate additional cycles.

The smaller sample gives less repeated durability exposure. The owner accepts that tradeoff. Every retained cycle must still prove the disjoint update, exact runtime identity, fresh browser possession, maximum exchanges, unchanged Device Identity/settings/authorization marks, mining disabled and cleanup.

The firmware and browser retain their original qualified source identities. A separate qualification-source revision may implement this policy change only through the reviewed qualification/documentation allowlists. A write-once amendment binds the original context, all four cycle digests, unchanged campaign, exact artifact snapshot and qualification revisions. It cannot replace runtime bytes, reset a baseline, mint a new campaign or alter the 180/30/30-second windows or 240-second ceiling.

Preserve the same page and origin across supervisor restart, release the port, and explicitly reconnect to obtain fresh possession and authorization scope. Original contexts, consumed attempts, closed records and historical evidence remain untouched.

## Unchanged requirements

Keep fixed Serial/JTAG, direct Web Serial, signed manifest/firmware/session bindings, role-separated keys, durable replay marks, foreground heartbeats and the three-second revocation requirement. Retain the approved conservative mining profile, ordered shutdown, qualified cooling, final `mineonboot=false`, secret clearing and resource release. No unrelated parity or mining blocker is promoted.

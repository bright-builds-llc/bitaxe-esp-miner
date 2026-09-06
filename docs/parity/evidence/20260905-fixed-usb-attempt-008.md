# Fixed Serial/JTAG qualification — attempt 008

Board: Ultra 205. Host: macOS, desktop Chrome, direct Web Serial.

Runtime firmware source: `11436c7eef0fe719492e5c6f8ad8a45abe15a645`.
Application ELF SHA-256: `799947fb886e409c9f88f4a8e4777786d786b42da5a4097b09d482fe877db133`.
Browser Gate source: `61cf6c599807abbdb12d88f5b810df73e564c757`.

The owner reduced the continuity requirement from twenty cycles to four under
ADR-0022. Policy revisions `b935ebbe47516a0566808cb41e5947da05c18161`
(firmware) and `369b0b9f98a4a10641e8222d6d0c0111bb9e49e3` (Gate)
did not change those runtime artifacts. A sealed amendment binds the original
context, four cycle receipts and thirteen digest-verified protected artifact
copies. No predecessor evidence was rewritten.

## No-mining result

All four cycles passed. Each used fresh browser permission and possession,
released browser ownership before CLI access, wrote the five admitted disjoint
package segments with hash verification, returned to healthy exact application
execution, and released CLI resources before reconnecting. Every cycle verified
an actual 65536-byte request and response, stable Device Identity, matching
settings and durable authorization marks, and disabled mining.

The final same-page reconnect after the policy amendment again verified
possession, preservation and a fresh maximum-size exchange. A native tab
visibility dry-run observed both hidden and visible states without mining;
the temporary observer and tab were removed. Browser focus emulation was
disabled so the test used actual document visibility.

## First live-window failure

The protected supervisor issued normal window 0 and consumed its signed input
delivery once. The production browser adapter's Start ended with
`start_failed/liveness_lost`. Its bounded firmware diagnostics included the
secondary RX stage `session_revoked`; the original command rejection category
was not available in the browser's diagnostic grammar.

Nine public state samples were retained. None reports running or a qualification
generation. No renewal, ASIC-work correlation, accepted share, foreground-loss
mining test, or heartbeat-loss mining test was verified. These observations do
not establish the exact original command rejection category.

Fresh possession after the failure confirmed restoration to baseline, inactive
lease, mine-on-boot false, and unchanged settings, Device Identity and durable
authorization marks. The browser released its serial streams and ownership;
the exact supervisor process exited; neither serial node nor its listener had
a remaining holder. The fixed Serial/JTAG firmware remains installed.

## Disposition

The four no-mining cycles are successful evidence for the runtime identities
above. Live mining acceptance remains unresolved. The consumed signed window,
earliest failure and original campaign remain preserved. No interrupted device
reservation is refunded, and no replacement campaign is permitted.

A public synthetic cross-language regression subsequently reproduced rejection
of Gate-signed campaign grants by firmware. That software result supports a
targeted serialization correction, but does not retrospectively prove the
discarded device error or successful hardware mining. Corrected runtime
artifacts require fresh qualification under the existing task contract.

Protected originals remain local. This report intentionally includes only
closed outcomes, counts and source provenance; it contains no owner pool,
credential, session-binding or private preservation values. No parity status
is promoted by this report.

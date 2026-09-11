# Hello recovery attempt 003: consumed-request interruption

Four no-mining update/reconnect cycles and the precise consumed-request
interruption passed. Fresh browser recovery succeeded without a serial drain,
but it discarded no complete old control reply. The strict qualification remains
unverified, and this consumed attempt is closed as `stop_impossible_contract`.

## Exact runtime and observations

- Firmware: `772bd5c05e66f023303a148ee899647ce4c58d72`.
- ELF SHA-256: `ec5f77168806c5d76d28eedeea4ed442d792525bdb7a1bc5372ef08443e920da`.
- Gate: `54dd512cc53f8454a83f88e443e3fde3430362bb`.
- Initial installation and all four later flashes reported exact firmware
  execution, complete error-free startup and a safe stable baseline.
- Each cycle completed a fresh browser connection and actual 65536-byte request
  and response, with matching Device Identity, settings and authorization
  high-water observations. Cached `state.probe` values were not used as fresh
  exchange proof.
- The single planned interruption produced the validated consumed/pending/
  ownership-released receipt, after a fresh inactive-baseline status check.
- After a recorded minimum wait beyond the device heartbeat deadline and
  confirmed OS-level release, fresh Hello and possession succeeded without a
  drain. Recovery recorded one discarded record, zero discarded control replies
  and 2136 skipped bootstrap bytes.
- Both original and qualification-attempt accounting receipts remained equal
  and bound to ordered browser journal states. No mining or signing allowance
  was created or consumed.
- The final journal state was closed and released, with baseline confirmed,
  lease inactive and mine-on-boot false. The owned tab closed, the supervisor
  exited successfully and both serial nodes were holder-free.
- All 13 retained runtime artifacts and the supporting cycle/interruption
  evidence passed independent re-audit. The sealed child inventory contains
  104 original and derived evidence files.

The official judge returned `no_mining_stale_recovery_missing` and produced no
passing result. These observations establish operational recovery for this
trial, not coverage of the repaired complete-old-reply branch, post-work
restoration, live mining, or broader parity.

## Corrected host evidence findings

The enclosing parent remained mode 0700 throughout. Initial installation and
cycle-one output directories/files inherited 0755/0644 from the caller. They
were tightened to 0700/0600 with unchanged content digests before admission to
the audit. An explicit owner-only umask and its regression fixture governed
later cycles; their output permissions passed directly.

The additional offline auditor initially required two facts the producer does
not expose. Fixed Serial/JTAG intentionally reports the runtime reference commit
as `Unavailable`, and commit-redacted evidence replaces the manifest path with
`[redacted-path]`. Source-backed corrections retained package/reference provenance,
the frozen canonical manifest digest and fresh exact firmware/ELF admission,
while explicitly making no runtime-reference or receipt-path observation claim.
Original helper versions, failures and corrections remain private evidence.
The official judge and its missing-reply criterion were not changed.

## Offline discriminator and hard progress gate

A protected software experiment imported the pinned production Gate channel and
used valid fixture-generated credit and response frames in one fixture reader
chunk. The complete 791-byte status reply was already delivered within a
1030-byte chunk. Pausing only the test-side reply hashing boundary still produced
a positive consumed/pending/released interruption receipt. One test with six
assertions passed, including cleanup and no later writes. It used no hardware
or injected device traffic.

This is a counterexample to the inference that a pending response must still be
undelivered. It does not establish the cause of this physical trial. Source
inspection also shows that receive credit can precede dispatch and reply
transmission. Current diagnostics do not retain successful dispatch/TX and
pre-close reader-position facts across fresh Hello, so these possibilities
cannot be distinguished from the recorded aggregate counts.

The required complete-old-reply observation remains absent. No further hardware
ordinal is justified by an unchanged retry or by this software counterexample
alone. A successor requires a distinct diagnosed cause, a regression at the
relevant boundary, a verified correction and a published fresh contract. The
next investigation needs source-reviewed, bounded observability for dispatch,
TX completion/partial output and the browser reader's pre-close position.
Instrumentation alone must not be treated as permission to repeat the fault.

The task remains open at this evidence/observability blocker. No hardware fault,
ring overflow, specific discarded-record type or non-dispatch cause is claimed.

## Preserved bindings

| Artifact                      | SHA-256                                                            |
| ----------------------------- | ------------------------------------------------------------------ |
| Context binding               | `16d9333b8aecfb66dde9d8a4252d5ac605355aef18bd9380eaa752f67ba721a8` |
| Runtime snapshot              | `3d5fbea36c62d4574cd2b25a8cb3be72ceda3325bc7505a17b19023f3af71d08` |
| Interruption evidence file    | `853ca60480d65fa9458c00998097d667186169f3e5a73b661e16235730bb9b69` |
| Review file                   | `24422961cc9df564f7136ee2c96bf1ef8d6af1cb66b4af43695bcf38a207aabf` |
| Closed outcome file           | `6a3550caa35dd68d4904d3af44db63f961fe76b9442303295dba826b767b7ca1` |
| Sealed inventory              | `2f8dac8acb3f068f80b6b17edf020bff128bac39d840b1da24b9a06b80ad1ebd` |
| Offline discriminator fixture | `589a444cc632d3355d2a70f0c083f3d894adc4bca74ac35c5543136eddc29b8b` |
| Offline source binding        | `50a5cac4d8ebeb97b7a52877c189c5153f6f39d815382be092372df6e879cae1` |
| Offline discriminator result  | `2f7e619180b1256e3efa8d182cac2348db46ac51db82832fdaff27454a723baf` |

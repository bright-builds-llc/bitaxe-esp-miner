# Fresh Hello recovery: continuity and interruption observations

Four state-preserving update/reconnect cycles and both planned no-mining
recovery workflows passed. The stricter complete-old-control-reply hardware
criterion remains unverified: both recoveries discarded one old record and
zero control replies. The original failed judgment is preserved.

## Exact pair and verified observations

- Firmware: `854265b178c673669a050c29665a0177b8173171`.
- ELF SHA-256: `8acb33511f422a8f588f672ec2ddad1b9d41814f057edc51076ab1ab8b9eb089`.
- Gate: `627a537396d022f3087ee3837f811c44d28d5b5f`.
- Initial installation and all four subsequent flashes reported exact execution,
  complete startup, a safe stable baseline, and no assessment issue.
- All four cycle receipts validate. Each reconnected browser completed
  65536-byte requests and responses with the same preservation baseline,
  matching Device Identity, settings and authorization high-water observations,
  and mine-on-boot false.
- A trusted native visibility event hid the page while an ordinary read-only
  refresh remained pending. The request rejected and USB ownership released.
  Fresh browser admission succeeded without a serial drain.
- A separate connection was cancelled while still pending, after its fresh
  Hello acknowledgement and during capability admission. Ownership released;
  another fresh browser connection succeeded without a drain.
- Both original and qualification-attempt accounting records remained equal.
  The receipts bind their corresponding ordered browser journal states.
- Final state was closed, baseline confirmed, lease inactive and mining
  disabled. Both owned tabs closed, the observer was removed, USB nodes were
  holder-free, and the supervisor exited successfully. All 13 copied runtime
  artifacts revalidated.

## Evidence limit and disposition

Foreground recovery reported 2050 skipped bootstrap bytes, one discarded record,
and zero discarded control replies. Admission-cancellation recovery reported
2163 bytes, one record, and zero control replies. Byte totals include bootstrap
text and do not establish the discarded record's payload size or kind.

The strict judge returned `no_mining_stale_recovery_missing`; it produced no
passing result. Outcome is `stop_impossible_contract` at
`complete_stale_control_reply_not_observed`. No synthetic or injected records,
CLI drain, mining allowance, or mining operation was used. This does not claim
complete old control-reply hardware coverage, post-work restoration, live-work
heartbeat timing, or broader parity.

A future attempt needs a distinct, verified, non-injecting reproduction
boundary. Repeating the consumed faults unchanged cannot add that proof.

## Preserved digests

| Artifact          | SHA-256                                                            |
| ----------------- | ------------------------------------------------------------------ |
| Context           | `c78d4e9dd83b40417b4a8d52ad52f90010e3ff75c0cf99a64eaf5029f71d68b8` |
| Runtime snapshot  | `7825bd0dfbe9d6815a671d8ecf32c2f547a82c7afe3a3dd2e02572065a86006b` |
| Review            | `2818fec1d0b4850e18a4f13529d41a9a22333452bdf3b6d7e3adcafe0411b2c5` |
| Closed outcome    | `151b291200702834660c756bbad96da97e7649efc96cece78c153d6024ce33dc` |
| Accounting before | `7e9683337a4ffaf450ef34ce9bec7abfbdd45f21f267ad44eeddde2116f0a002` |
| Accounting after  | `69e49f8ac4962e39937a7941a04f03d9242a87cab32f8bc990d36f514fd2181b` |

An independent read-only audit confirmed the four cycle receipts, five flash
records, 46 journal records, unchanged accounting, interruption receipts, final
closed state and artifact snapshot. It also confirmed that the strict missing
control-reply judgment must remain unverified.

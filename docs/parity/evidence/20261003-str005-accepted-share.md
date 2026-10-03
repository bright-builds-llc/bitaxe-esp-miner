# One device-acknowledged accepted V2 share on the Ultra 205

Share001 ran one bounded `normal` Start on the audited `654338d0` image against
the local Stratum V2 fixture. The BM1366 found a share that met the pool target.
The firmware submitted it over the encrypted V2 channel, the fixture accepted
it, and the device recorded the acknowledgement. The device then stopped
normally and restored. The sealed result is `complete=true` with
`accepted_share_verified=true` and no blockers.

| Boundary            | Direct evidence                                                                                                            |
| ------------------- | -------------------------------------------------------------------------------------------------------------------------- |
| Image and Gate      | Source `654338d0101521490d90330c5a4a10e5ec32e5c2`, ELF `2641c24fc3f4fc3a80a4bcb8bfd389b2d70d3e048d9e18a5771b14309b588193`, Gate `86fc62d7`; all native audits passed |
| Lineage             | Step-5 reinstall seal `0751d602…`, then start004 `d37808f8…`, then restart002 `7f602f4f…` (boot 12 to 13, ledger unchanged) |
| Admission           | Fresh detector and physical identity, same-page baseline, measured ledger next 24 / last 23 / 2,640,000 ms                   |
| Start               | One attempt, generation 3; reply after 10.7 s; conservative profile 400 MHz / 1,100 mV, fan 100%                              |
| Renewals            | 1 of at most 2 pre-signed same-lease renewals confirmed                                                                    |
| Share               | Submitted 1, accepted 1, rejected 0, duplicate 0; Stop requested 23.4 s after the reply, as soon as the acceptance joined   |
| Independent proof   | Header, target, nonce, submission and native ACK joins recomputed from private fixture and device records: verified        |
| Normal stop         | Revocation `restoration_requested`; safe stop complete; retained record terminal, socket closed, worker quiescent, no fence |
| Authorization       | Recovery checkpoint matched for generation 3                                                                               |
| Accounting          | Ledger next 25 / last 24 / charged 2,820,000 ms (exactly +180,000), not pending; original budget complete                  |
| Fixture             | Exact peer, natural exit 0 after the accepted share and peer EOF; listener and socket closed                               |
| Release             | Host, serial, signer and fixture owners released; fresh-possession recovery complete; page closed                          |

Result: `str005-share-result-v1`, `complete=true`, `accepted_share_verified=true`,
`renewals_confirmed=1`, no blockers. The private root is
`scratch/str005-accepted-share/share001/attempt`, with result SHA-256
`9e17a8b28e0146f41c9d8d1079b42b86bb1b26c4236b70f157df85cb85d5a402` and seal
`abde26a6e92e3a8e0d1edfa18558c68bc24e880a027644cb51f98bfef39aedff`. Raw
device, signer, fixture and endpoint data stay private.

Non-claims:
- one share on a local fixture is not pool mining, sustained mining or
  heartbeat-loss evidence;
- the original Share001 panic cause is not resolved here; it remains with
  `task-str005-start-panic-diagnosis`;
- parity promotion belongs to `task-str005-piecewise-integration-review`, and
  parity stays 90/95 until that review.

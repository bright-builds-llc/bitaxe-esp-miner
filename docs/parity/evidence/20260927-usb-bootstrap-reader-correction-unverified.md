# USB bootstrap early-reader correction: unverified completion

Outcome: **ordinary capture qualified; overall attempt unverified**. Five of the
seven prospective correction checks passed. Missing post-install accounting and
its restoration receipt prevent correction acceptance. Attempt 003 remains sealed
and unverified; it receives no continuity, V2, mining or parity credit. Parity
remains **90/95** active rows verified.

## Tested pair and immutable evidence

One Ultra 205 received one state-preserving installation under the published
[early-reader correction contract](../../hardware/usb-bootstrap-reader-correction.md).
This campaign used the same qualified reset backend and preserved the native
2,000-ms record budget. Both admission passes remained mandatory while the single
early reader quarantined bytes.

- Firmware: `5bda5b91304f18d74bd64160abb807cfafbd0087`.
- ELF SHA256: `c16658a3472e5e508e85c6186d04882ee0eb968cb67bbc4c73c23ead718f1af0`.
- Gate: `e20c0fd52d2216596f904992ffa54fda33be9025`.
- Private evidence root: `scratch/usb-bootstrap-measure/attempt-003`.
- Context SHA256: `9f4375e1dd8602f47b3a6f5713395b701b09caf9a8fd758e74a9339b2bbd1cfe`.
- Result SHA256: `bb097f4b85e9ef92b433700ec2c83b15b701bf1de7ab7636180da39500ac4f5b`.
- Seal SHA256: `b53c0db5d163f17cedc33dc039274e2171339882a8d9a15ca157ae1c3cfec2b7`.
- Host timing SHA256: `1d43cf52c16e090e5e302709e4231f8a9bb01f8a0cee1e95b87c4b7e56ab4627`.
- Device observation SHA256: `f9b1c389a17efb636917c99fc31283085ef89890cd612911231cb91e577fb576`.

The ordinary flash/capture command exited 0 and qualified. The sealed result has
`measurement.complete:true`, but its overall status is `unverified` and
`correction.accepted:false`. Measurement completeness alone does not satisfy the
required restoration, accounting and cleanup joins.

## Observed correction behavior

Host intervals below use one host monotonic clock. Reset-call bounds include
launch, execution and reap, not a physical reset edge. Device intervals use only
device uptime; no host/device timestamp subtraction is performed.

| Observation                                     | Result                                            |
| ----------------------------------------------- | ------------------------------------------------- |
| Reset-command call start to reader opened       | 818.544 ms; within the prospective 1,500-ms guard |
| Reset-command return to reader opened           | 338.704 ms                                        |
| Reader ownership interval                       | 30,026.629 ms                                     |
| Descriptor count                                | One opened, zero reopens, actual join confirmed   |
| First bootstrap category / kind                 | `bootstrap_diagnostic` / `startup_progress`       |
| First bootstrap start / end                     | 91 / 92 device ms                                 |
| First bootstrap record / queued bytes           | 92 / 92; one positive queue call                  |
| Native drain result                             | One successful call; no timeout or other failure  |
| Retained actual failure count / integrity flags | Zero / zero through the last captured observation |

The capture binds these device observations to the exact tested pair and one
stable boot. Three observation markers were captured; the retained counters show
two completed measurement replays. No first-failure observation or legacy TX
failure marker was captured. The zero-failure statement ends at the last observed
counter snapshot; it is not proof about later transmissions.

The five passing checks were ordinary capture qualification, complete host timing,
reader headroom, native bootstrap completion and zero observed TX failures. These
results support the targeted early-reader correction on this run. They do not
identify the private IDF wait responsible for measurement 002 or replace fresh
four-cycle continuity and V2 qualification.

## Completion failure and source-review diagnosis

After the capture, the operator invoked Stop/Restore before exporting the final
accounting. The page reached `baseline_confirmed`. The retained earliest failure
is the browser's generic `bootstrap_client_failed`, with
`observationSha256:null`; it does not contain a narrower device failure diagnosis.

Parent source review identified an incompatible workflow guard: the measurement
client's accounting helper accepted only connected `ready`, while explicit
restoration produced `baseline_confirmed`. The server-side accounting path also
used a ready-only baseline check. This is the parent's source-review diagnosis,
not a fabricated browser exception or device event. The accounting export did not
produce `accounting-after.json` or the joined `restoration.json` receipt.

Final journal state 15 records explicit device restoration, confirmed baseline,
inactive leases and preserved identity/settings/authorization high-water, with
mine-on-boot false. It is closed, disconnected and confirms serial release.
Those observations do not replace the missing authenticated post-install ledger
export. No fresh unchanged-ledger conclusion is claimed.

## Cleanup and remaining obligations

The owned supervisor exited 0 within 5 ms of its recorded stop request. Native
window closure and resource-release facts were retained. Independent absence
checks found no owned process groups, supervisor listener, operator socket or
serial holders; preliminary cleanup facts report complete.

The formal cleanup receipt remains **incomplete**, because its required accounting
and restoration joins are absent. The overall judge therefore leaves
`preservationAndAccounting:false` and `cleanupComplete:false`. Physical resource
release and a passing formal qualification receipt are separate claims.

No mining or allowance consumption occurred. Do not replay, repair or upgrade this
sealed attempt. Correct the restored-state accounting workflow in software, test
its evidence joins, and publish a separate prospective successor contract before
any further hardware attempt. Neither this report nor the successful capture
supplies continuation authority or parity promotion.

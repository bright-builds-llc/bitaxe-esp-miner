# USB bootstrap correction accepted

Outcome: **measurement complete; bounded bootstrap correction accepted**.
Attempt 004 passed all seven prospective correction checks, including fresh
post-restoration accounting and complete cleanup. This is acceptance of one
no-mining bootstrap correction measurement, not broader USB/V2 qualification.
The artifacts retain `hardware_qualified:false`, `mining_authorized:false` and
`qualification_credit:"none"`. Parity remains **90/95** active rows verified.

## Prospective scope and tested pair

The [early-reader correction contract](../../hardware/usb-bootstrap-reader-correction.md)
and [restored-accounting successor amendment](../../hardware/usb-bootstrap-restored-accounting-successor.md)
were published before this attempt. One Ultra 205 received one state-preserving
installation 0 with unchanged reset backend, NVS-disjoint write segments and
native 2,000-ms record budget. No installation or reset retry was needed.

- Firmware: `77e2e4a431663ab2c2948d1e81183472a04e385c`.
- ELF SHA256: `d32d799586a72bcdf58871e0905a756a6499442c23abcce4e9de346e1fafaf51`.
- Gate: `e20c0fd52d2216596f904992ffa54fda33be9025`.
- Private evidence root: `scratch/usb-bootstrap-measure/attempt-004`.
- Context SHA256: `804f022dbdf4bb1f1f2298558486a5eb659c84fd851ae32a9e5a97cf1c542c78`.
- Independently reviewed result SHA256: `be8339ce97e8e91bf5194076181caa11e9e9407892bf6ab376f8a06fdfbd34c8`.
- Seal SHA256: `b9f85846b4c7875f4908cd44dc38067a11b7ac84a99c3c9b57391544ddc7a424`.
- Host timing SHA256: `65d18f6276fc436581138de2d98018f7a45ef51ee74e0fd1a4ffa49f7b637d0a`.
- Device observation SHA256: `493469f125981a5095074f7e79d711a36cb3a594de060b91f3fc74b09be2461d`.
- Restoration receipt SHA256: `a824c24762992476a7b75b611d9ceef2a7c5cc445b0288c17be24ea6b54df36f`.
- Cleanup receipt SHA256: `195829dd99f174b2c952bf071261a224e075e008738b50e87f2909ddd7c3c44e`.

The finalizer and separate read-only review reported `measurement_complete` and
correction acceptance true. The ordinary flash/capture command exited 0 and
qualified; every correction check is true.

## Measured transport behavior

Host intervals use one host monotonic clock. Reset-call bounds include supervised
launch, execution and reap; they are not physical reset edges. Device intervals
use device uptime only. No cross-clock subtraction establishes these results.

| Observation                                   | Result                                               |
| --------------------------------------------- | ---------------------------------------------------- |
| Reset-call start to reader opened             | 854.149 ms; below the prospective 1,500-ms guard     |
| Reset-command return to reader opened         | 378.956 ms                                           |
| Descriptor opening                            | 1.070 ms                                             |
| Reader opened to first nonempty read          | 0.191 ms                                             |
| Reader ownership interval                     | 30,007.258 ms                                        |
| Reader opened to quarantine release           | 2,952.190 ms; both admission passes completed first  |
| Reader count / reopens                        | One / zero; actual closure and join confirmed        |
| Host earliest failure                         | None                                                 |
| First bootstrap category / kind               | `bootstrap_diagnostic` / `startup_progress`          |
| First bootstrap start / end                   | 91 / 92 device ms                                    |
| Record bytes / queued bytes                   | 92 / 92; one positive queue call                     |
| Drain                                         | One successful call; zero timeouts or other failures |
| Retained actual TX failures / integrity flags | Zero / zero through the last captured snapshot       |

The observations bind to the exact executing pair and one stable boot. Three
observation markers were retained, with two completed measurement replays. No
first-failure observation, legacy TX failure marker, truncated tail or measurement
issue was observed. Zero failures is bounded to the captured observations; it does
not certify transmissions after the last counter snapshot.

## Restoration, accounting and release

The native workflow successfully followed candidate authentication -> Stop/Restore
-> `baseline_confirmed` -> fresh accounting export -> close and flush. The
accounting receipt retains the actual `baseline_confirmed` status and explicit
restoration confirmation; no status was rewritten to satisfy a guard.

Authenticated before/after qualification ledgers are equal: next ordinal **18**,
last completed **17**, total charged **1,560,000 ms**, pending **false**. The original
campaign ledger is also unchanged: **240,000 ms** charged, reserved/completed masks
**7/7**, pending **false**. No mining, grant loading, mining lease or allowance
consumption occurred.

Final journal state 15 is closed and disconnected with explicit restoration,
inactive leases and serial ownership released. Identity, settings and
authorization high-water preservation passed, and mine-on-boot remains false.
The native window was observed closed. The actual supervisor exited 0 within
5 ms of its recorded stop request. Independent checks confirmed no serial holders,
owned process groups, supervisor listener or operator socket. Both preliminary
resource facts and the formal restoration/accounting/cleanup joins are complete.

## Resolved defects and historical limits

The host sequencing defect delayed opening the receive descriptor until two
admission passes finished. [Measurement 002](20260927-usb-bootstrap-reader-gap-measurement.md)
measured 4,423.674 ms from reset-command return to reader opening and a full-budget
native drain failure. The correction starts one quarantined receiver earlier while
retaining both admission passes, their identity checks and the physical lease.
The passing transport observations in attempts 003 and 004 support this targeted
host correction. They do not identify whether the private IDF mutex or idle wait
produced the historical timeout.

A separate collector defect blocked completion of
[attempt 003](20260927-usb-bootstrap-reader-correction-unverified.md): explicit
restoration produced `baseline_confirmed`, but accounting required `ready`.
Version 4 uses a shared, phase-aware predicate: initial accounting still requires
`ready`; final accounting requires actual restoration and accepts its preserved
`baseline_confirmed` or `ready` state. Both fresh ledger reviews and all baseline,
lease and preservation checks remain required.

The independently tested sub-tick polling correction fixed premature deadline
termination. It did not explain or retroactively resolve the earlier full-budget
timeout. No driver fork or timeout relaxation was introduced.

Attempt 001 remains an interrupted preparation; measurement 002 retains its
unqualified ordinary capture; attempt 003 remains sealed and unverified. None is
rewritten or upgraded by this accepted successor.

## Verification and remaining work

Software verification included the ordered Rust format/Clippy/build/test sequence
with **2,366 passing tests and three ignored**, **161 Node tests**, and the latest
**19 affected canonical targets**. The broader **225-target canonical run** and
subsequent corrective verification precede this result. Gate's unchanged pin has
**788 passing tests**. These software checks supplement the exact-pair hardware
observations; they are not substituted for them.

This completes the bounded bootstrap correction and restored-accounting objective.
Fresh four-cycle continuity, V2 live qualification, and any later parity promotion
remain separate obligations with their own prospective contracts and evidence.

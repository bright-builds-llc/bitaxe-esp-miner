# STR-005 V2 reservation-clock amendment

Amendment ID: `str005-v2-serial-clock-v1`. Owner:
`task-str005-v2-admission-clock-amendment`. Applies to the prospective
`str005-v2-serial-v1` contract, published at `6a4a45f8`, whose unchanged SHA256
is `d23220a4ac5021470b18d7e97cb4b82ba28232d9d5a0f8340389852b39841c79`.

## Reason and unchanged authority

The ordinary runtime reserves the full allowance during authenticated Start, but
arms the work window at the first guarded ASIC dispatch attempt immediately
before UART execution. Dispatch completion and successful-dispatch counters are
later facts. A diagnostic cannot truthfully report that future absolute deadline
at Start admission.

Source evidence is `worker_qualification_budget::admit`,
`production_mining_session::revocation::GenerationGate::begin_dispatch`, and the
guarded call in `asic_adapter::production`. The existing shutdown reserve is
15550 ms: a normal reservation charges 180000 ms and permits at most 164450 ms
of work from that dispatch-arming epoch. Failed UART execution does not move the
epoch or refund the reservation.

This amendment changes the prospective diagnostic representation only. It does
not start a clock earlier, postpone arming until successful dispatch, change
lease duration/renewal, permit preparing renewals, reset accounting, or extend
heartbeat, work, cooling or cleanup authority. No device attempt has occurred
under the base contract.

## Exact interface correction

`DeviceRecord.authorityDeadlineDeviceUs` becomes `UInt | null`.

- Channel requires a non-null value equal to admission plus 120000000 us; its
  existing observation horizon and acceptance are unchanged.
- Share seeds the field as null at admitted funded Start. Null means the fixed
  reservation clock has not armed, including an honestly cancelled or failed
  pre-work terminal. It is not a clock failure and must not become zero or a
  fabricated admission-relative deadline.
- When the existing first guarded dispatch arms its window, capture that actual
  device-local millisecond epoch and set the field once to
  `(armingEpochMs + 180000) * 1000`. The actual work gate is 15550000 us
  earlier. Use checked arithmetic and the same arming observation; do not infer
  the epoch from a later successful-dispatch counter, host receipt, or an
  ambiguously wrapped 32-bit timestamp. Unavailable or discontinuous provenance
  is an explicit unverified clock/evidence outcome.
- Permit only the null-to-known transition. Once known, the value cannot change
  through renewal, status, cancellation, reconnect, shutdown or cooling. Any
  recorded dispatch/share fact or accepted Share result requires the known
  deadline and its native arming provenance. A later successful-dispatch fact
  must not precede the arming epoch; these facts need not share one timestamp.
- The renewed effective lease remains separate. The pre-fault guard still uses
  fresh actual effective-lease and work-gate remaining time, both at least 5000
  ms, and still requires heartbeat-timeout revocation/shutdown within 3000 ms of
  the last valid advancing heartbeat.

No other status, grant, evidence or historical-reader field changes. Runtime,
Gate and host parsers must agree on this scope-dependent nullable transition.

## Admission, verification and lifetime

Preflight and frozen evaluator inventories bind both the base contract and this
amendment by exact path and digest. The public contract provenance commits to
that pair. Later semantic changes still require an explicit amendment task.

Tests cover pre-dispatch null state and cancellation, first guarded dispatch,
UART failure after arming, exactly-once assignment, renewal/reconnect retention,
missing deadline with work evidence, checked arithmetic and clock ambiguity.
Existing reservation and independent-safety tests remain required.

The fixture's 300000-ms lifetime remains absolute. Its budget includes
ready-to-Start, initial preparation before dispatch, the subsequently armed
generation and actual EOF/cleanup. The existing 10000-ms ready-to-Start,
30000-ms Start-response and 60000-ms initial-lease limits still apply. This
explanation does not replace direct proof that actual EOF occurred before the
fixture deadline, or shorten the separate cooling/restoration wait.

No acceptance threshold, mining allowance or parity row changes. The base
contract and all historical evidence remain byte-for-byte unchanged.

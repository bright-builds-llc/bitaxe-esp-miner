# V2 successor: Share002 Start failure

Date: 2026-09-27. Owner: `task-str005-v2-serial-qualification`.
Outcome: **unverified / `stop_hardware_blocker`**. Parity remains **90/95**.

## Exact candidate and admission

The [published bootstrap successor contract](../../hardware/str005-v2-serial-bootstrap-successor-amendment.md)
admitted Channel006 and then Share002 on one unchanged clean pair:

- Firmware: `cf7a3f038dabdd5383083734336aeb64a22f837c`.
- ELF: `66a77d2cb699e2064469c7b124f482a57a7fd6f6b13ef737554f53f721e78590`.
- Gate: `e20c0fd52d2216596f904992ffa54fda33be9025`.
- Share context: `a453de1753acc78bfa3eaeebfd9f42339528ec4fb41704fd50d390a1cd1ff5c4`.
- Share result: `02c48ee223c0f7a892e1c9341381a0e32c5f194f2538be62140f02391e087d09`.
- Share seal: `14d2122208b2040f1482074c77328cd3a59c651e45bc17e7be7af2648d8f5950`.

The clean package, native resource audit and exact fixture/observer/validator
identities passed preflight. Fresh detection admitted the same Ultra205.
Channel006 completed five installations, four fresh maximum-exchange continuity
cycles, the network-only channel exchange, restoration, accounting and cleanup.
Private finalization and independent review both passed. Its accepted projection
remains private because Share restoration/cleanup proof is incomplete.

Share002 used a separate protected root, page, possession and private baseline.
All four additional state-preserving installations and maximum-size bidirectional
exchange cycles passed. Fresh pre-install and pre-issuance accounting matched
next18/last17/1,560,000ms/pendingfalse; the original campaign remained exhausted.
No historical cycle or baseline credit was imported.

## Failure and accounting limits

The retained first failure is supervisor-observed
`v2_client_start_operation_failed`. Native UI startup diagnostics observed boot12
followed by boot13 with reset reason `panic`; that observation is not a decoded
panic cause or an authenticated pre-crash trace. The fixture later timed out at
`setup_received`; the sealed result retains it as a secondary available cause.

The host issued ordinal18 and attempted delivery. Both issuance/delivery receipts
explicitly say `deviceReservationObserved:false`. No completed Start receipt,
accepted-share acknowledgement or heartbeat-suppression claim was collected.
These absences do **not** prove no reservation or hardware preparation occurred:
the production Start path persists the budget reservation before owner preparation
and before its reply. Post-failure durable accounting remains unknown. Neither
ordinal18 reuse nor an assumed next19 is authorized by this evidence.

The displayed `worker_owner_prepare` memory checkpoint belongs to startup, not a
measurement of the previous boot at Start. Likewise, a `corrupt` previous-boot
preparation receipt does not identify a failed preparation step. Neither observation
establishes out-of-memory as the panic cause. No accepted share, independent
heartbeat shutdown, bounded post-work cooling or mining parity is claimed.

## Restoration and cleanup

After the bounded recovery wait, a fresh native serial connection admitted the
same installed pair. Normal Stop restored the baseline. Final journal sequence53
records closed/disconnected/not-running state, restoration and baseline confirmed,
inactive leases, identity/settings match, mine-on-boot false and serial ownership
released. No new Start, flash, factory reset or rollback was attempted.

The native page disables restoration collection after its failure latch. Thus
`accounting-after.json`, the retained restoration/status join and device-resource
release proof were not collected. This is a collector gap, not permission to fill
missing evidence retrospectively or weaken the judge.

Actual host cleanup completed: the owned browser window closed, supervisor exited0,
fixture exited1, operator stopped, owned processes were absent, and supervisor/
fixture listeners and serial holders were absent. The cleanup receipt records
`deviceBaselineConfirmed:true` and `deviceResourcesReleased:false`. The parent
retains `v2_parent_cleanup_failed` at the record stage. Formal cleanup is therefore
incomplete despite the observed baseline and host release.

Finalization and independent read-only review reproduced the unverified hardware
blocker. Publication returned no scopes, blocked by `v2_evidence_incomplete`.
No accepted channel/share projection was copied into the public evidence directory.
All previous failures, seals, assignments and accounting remain unchanged.

## Required next proof

1. Freeze and verify a failure-only native recovery collector for fresh
   authenticated ledgers and retained status, with no Start/issuance/flash authority.
2. Obtain panic-location, native exception/backtrace or valid preparation-boundary
   evidence tied to the installed ELF. Startup heap values and the first retained
   allocation failure alone cannot establish causation.
3. Verify a targeted correction and boundary regression, then publish a new finite
   admission based on actual accounting. Preserve the consumed Share002 context;
   the current contract admits no Share003 or Channel007.

The qualification task remains active/blocked and unarchived. Evidence promotion
and BWG restoration retain their independent prerequisites.

## Verification

Independent read-only review reproduced both sealed dispositions. The outcome
update passed ordered Cargo format/Clippy/build/tests (2366 passed, three existing
ignores), Bright Builds, reference, redaction, native USB ownership/symbols and
read-only parity/progress checks. The implementation previously passed all68
affected canonical targets and Gate type checking/788 tests before effects.
Task IDs remain unique; the archive, parity checklist and progress history are
unchanged. Independent documentation review found no privacy or claim findings.

# ADR-0029: Qualify STR-005 through independent checkpoints

Prospective diagnostic admission is clarified by
[ADR-0031](0031-prospective-panic-diagnostics.md): fresh current-state safety,
accounting and release proof can admit new diagnostics while the historical
Share002 resource-record gap remains unresolved. Original evidence and non-claims
below are preserved.

Accepted 2026-09-27 by explicit owner approval. This is a prospective task and
acceptance strategy, not an executable hardware contract or a new evidence schema.
`TASKS.md` remains the sole active tracker; this ADR records stable policy and the
obligation transfer from superseded `task-str005-v2-serial-qualification`.

## Decision

Separate focused development probes from acceptance checks. Development probes may
use concise, privacy-filtered, human-reviewed diagnostic output. Acceptance needs
reproducible evidence appropriate to the claimed behavior; a diagnostic success
alone never establishes parity. Each successor owns one narrow repo command and
its execution contract, reusing connection, identity, accounting and cleanup
components rather than requiring a new general-purpose harness first.

Complete and archive each successor when its own boundary is verified. A later
failure preserves earlier valid observations. Missing cleanup or accounting blocks
the claims and further effects that depend on that proof; it does not erase an
independent protocol result or turn a failed attempt into a pass. Recovery and
failure-evidence collection must remain available after the happy path fails.

Review change impact before reusing evidence. Record the tested firmware/Gate and
relevant package, fixture and validator identities, what changed, what behavior a
checkpoint covers, and why it remains applicable or needs repetition. Uncertain
impact requires repeating the affected check. Evidence always retains its actual
tested identities; never relabel an old observation as a measurement on new bytes.

Reuse qualified continuity evidence where applicable rather than automatically
reinstall four times before each experiment. Changes to USB, startup, ownership,
packaging or preservation require explicit continuity reassessment. ADR-0022 and
frozen campaign contracts retain their original sample, identity and effect rules;
this prospective policy must be implemented by reviewed successor contracts before
any changed execution is admitted. It does not retroactively reuse failed cycles.

The final integration review checks compatibility and cumulative coverage. An
already completed final-candidate probe may supply the needed smoke evidence.
Only a concrete uncovered interaction requires an additional bounded probe; do not
require all checkpoints to pass in one large campaign. The existing promotion task
joins the results and remains evidence/documentation-only.

## Remaining obligation transfer

The four unchecked items in the original task remain unchecked in its archive.
Their obligations move as follows; earlier completed work and failed outcomes are
preserved rather than reclassified.

| Original outstanding obligation | Successor ownership |
| --- | --- |
| Fresh share baseline, continuity, accounting and normal reservation | Recovery resolves actual ledgers; startup/share tasks own fresh admission and accounting; integration reviews continuity applicability. Share002's four passed cycles remain observations on their tested pair, not proof of reservation consumption. |
| Actual ASIC result, encrypted submission and accepted device acknowledgement | `task-str005-v2-accepted-share-probe`, after independently verified startup and applicable channel evidence. |
| Heartbeat revocation, shutdown, ordered stop and cooling | `task-str005-heartbeat-shutdown-probe`; startup separately proves normal stop. Both require their own accounting, restoration and cleanup. |
| Identity/settings preservation, authorization checkpoint and charged ledger completion | Recovery collector plus each effectful probe; integration rejects missing joins and never refunds or resets accounting. |
| Seal/review both stages, actual cleanup, permitted publication and promotion handoff | Each successor retains its scoped result; `task-str005-piecewise-integration-review` reconciles it; existing `task-str005-evidence-promotion` owns any parity transition. |
| Same-pair share admission, live safety testing and truthful combined outcome (successor-plan checkbox) | Split across startup, accepted-share, shutdown and integration; recovery and panic correction are prerequisites. No single-run completion is inferred. |
| Software/native verification and fixture correctness | Each changed probe/correction owns production-seam regressions, applicable Gate/canonical/native checks and exact build identities. Share owns fixture target/header/nonce/submission/ack verification; integration audits compatibility and negative-test coverage. |
| Privacy, failure handling, bounded authority and host/device cleanup | Mandatory in every successor contract and result; recovery owns failure-only collection, integration audits gaps. No arbitrary logs, endpoints, credentials or grants enter public evidence. |

## Task graph and persistent records

Recovery is first actionable. Panic diagnosis may inspect code and existing evidence
in parallel, but new device evidence depends on recovery readiness. Startup depends
on recovery and a verified panic correction. Accepted-share and heartbeat-shutdown
work depend on startup and may proceed independently in software. Hardware remains
serialized under one physical-device owner. Integration depends on all four proof
areas and the panic correction; promotion depends on integration.

Each native task block contains objective, dependencies, checkable implementation
and verification steps, evidence pointers, tested identities, outcome, limitations,
invalidation conditions and completion review. Store observed results there with
links to scoped evidence; do not create another tracker, hot counters or GSD state.

## Preserved evidence and effect boundary

Channel006 passed private finalization/review on firmware `cf7a3f03` / Gate
`e20c0fd5`. Share002 remains sealed/unverified with a Start failure and panic-reset
observation; ordinal18 was issued and delivery attempted, but device consumption
is unknown. See the [failed successor report](../parity/evidence/20260927-str005-v2-share-start-unverified.md).
Fresh authenticated accounting is required before choosing any next ordinal.
Neither reuse18 nor assumed next19 is justified by the missing Start reply.

Keep both results, all previous failures, seals, assignments and publication
restrictions unchanged. Recovery adds separate successor evidence; it cannot append
missing inputs to a sealed failure. Channel006's accepted projection stays private
until the applicable recovery/cleanup and prospective publication gates are met.

Archiving the old task intentionally makes its existing effect guard reject it.
Historical readers remain available. New task records supply no executable effect
authority: each successor must publish its narrow objective, command, evidence and
privacy rules, effects, recovery, cleanup, retry bounds and stop conditions, then
verify/publish its implementation before hardware use. No generic force/retry path.

Retain signed authority, exclusive ownership, durable accounting, private runtime
inputs, the conservative profile, device-local heartbeat/safety deadlines, ordered
shutdown and cooling. External pools, factory reset, rollback and parity promotion
are not implicitly authorized. BWG restoration keeps its independent contract;
deferred cooperative cancellation remains outside current parity prerequisites.

## Consequences

Smaller probes localize diagnosis and allow independent completion. Evidence reuse
requires an explicit compatibility judgment rather than assuming identical behavior.
A final integration gap can still block promotion, but it does not reopen every
completed checkpoint. Parity remains 90/95 during this metadata restructuring.

# Detached parent ownership boundary for Channel004 admission

Contract ID: `str005-v2-install-ownership-v1`.

This prospective correction belongs to the still-active explicit amendment
task `task-str005-v2-install-review-amendment`. It supplements the published
[installation-review amendment](str005-v2-serial-install-review-amendment.md)
before its implementation is published or Channel004 is assigned. The earlier
document remains immutable.

The earlier phrase "recorded parent references" did not distinguish the
campaign's detached parent from the external execution service that launched
it. The captured parent process has its own process group. Its `ppid` identifies
an ancestor outside that owned group, not another campaign resource. Requiring
that external service to exit would not prove campaign cleanup and would block
the intended continuation for an unrelated reason.

For this exact sealed Channel003 classifier, use its recorded detached parent
as the outer ownership boundary. Require a checked parent identity with
`pid == pgid`, and require the recorded supervisor's `ppid` to equal that parent
PID. The parent itself, its process group, every recorded owned process and
their descendants must be absent. Collect parent references from the owned
supervisor, installation, detector and observed child records. Any remaining
unidentified parent reference must still be absent; an occupied PID fails
closed. Exclude only the detached root-parent-to-launcher reference. If another
owned record names that same PID as its parent, retain that reference. Do not
recursively include or terminate the root parent's external ancestors.

This is not a PID-name allowlist, a permission to ignore an owned actor, or an
inference of historical exit. Preserve the genuine parent-observed exit record
and original missing post-install baseline/accounting. Continue requiring both
serial nodes to have no holders and the known supervisor listener to be absent.
The unchanged receipt schema records counts derived from this bounded owner set;
it gains no fixture, pool-port or historical cleanup claim.

Add the exact digest of this document to the prospective context-v4 contract
binding as `installReviewOwnership`. Include this document and its validator in
the complete source/checker inventories. Existing context-v1/v2/v3, old receipt
membership, sealed artifacts and historical readers remain unchanged. No v4
context has yet been assigned.

Before admission, test that a live external launcher ancestor does not block
readiness once all owned actors are gone, while the detached parent itself,
its group, descendants, any remaining worker-parent reference, serial holder
or supervisor listener still fails. Reject a non-detached parent or mismatched
supervisor-parent join. Publish this reviewed correction before any admitted use of
the clarified boundary, then complete all original verification and publication
gates before effects.

Channel004/Share001 ordinals, new possession/baseline/accounting, four-cycle
requirements, mining reservation and all safety/privacy/acceptance bounds are
unchanged. This correction grants no hardware authority by itself.

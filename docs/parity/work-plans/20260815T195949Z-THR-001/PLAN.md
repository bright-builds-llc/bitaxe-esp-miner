# Parity work plan

- Run ID: `20260815T195949Z-THR-001`
- Parity row: `THR-001`
- Initial status: `implemented`
- Source commit: `524f074b8ad508179c01683c4a8cd613dbe971e1`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-thr001-emc2101-live-thermal`
- Continues plan: `docs/parity/work-plans/20260815T192115Z-THR-001/PLAN.md`

## Selection and exact symptom

The clean synchronized selector has no open plan and ranks THR-001 first; no
higher candidate was skipped. Consumed attempt-006 proved the bounded device
state machine and retained replay both completed, then failed as
`evidence_invalid` because the strict host parser admits direct tag
`bitaxe_firmware` but the authoritative replay loop emits through exact module
tag `bitaxe_firmware::boot_evidence`. The baseline existed only in replay;
fault and recovery were present both directly and in eleven replay triplets.

This is a software-only diagnosis and correction plan. It authorizes no
package, detector, USB, serial, network, HTTP, device, NVS, display, sensor,
mining, control, reset, OTA, erase, attempt-007, or parity promotion.

## Feedback loops and design constraints

First make the real-child late-attachment fixture production-shaped: its
incomplete direct prefix must use `bitaxe_firmware`, and its replayed complete
triplet must use `bitaxe_firmware::boot_evidence`. Prove the current parser
withholds evidence with the exact marker-sequence `evidence_invalid` category.

Then compare the smallest strict designs: an exact two-origin host allowlist,
preserving the root log target during firmware replay, or another existing
typed retained boundary. Prefer the choice that models truthful ownership,
changes the fewest layers, and can be proved without hardware. Arbitrary tags,
nested module suffixes, non-INFO levels, malformed timestamps, bare payloads,
extra payload text, missing states, and wrong order must remain inadmissible.
Direct or replay duplicates may precede a complete witness, but publication
still requires one exact contiguous baseline/fault/recovery triplet.

Do not fabricate byte-zero records, infer a missing baseline, weaken the
ordered quorum, add a public diagnostic setter, retag unrelated logs, delay
boot or capture by fixed sleep, expose protected values, or change ordinary
thermal freshness and fault semantics.

## Implementation and verification

- [ ] Add and run the exact direct-plus-replay-tag real-child red regression.
- [ ] Record ranked alternatives and implement the smallest closed origin
      contract with malformed-level/timestamp/tag/payload/order tests.
- [ ] Prove canonical late attachment publishes only on a complete replayed
      triplet while ordinary restoration remains non-replayed and redacted.
- [ ] Run focused, firmware, ordered Cargo, Bright Builds, full Bazel,
      parity/progress, redaction, reference, task/plan, and diff gates.
- [ ] Commit and push with THR-001 still `implemented`; close without hardware
      evidence. A distinct immutable plan is required for any attempt-007.

Required final gates are `cargo fmt --all`, strict Clippy, all-target build,
all-feature tests, `bun scripts/bright-builds-check.ts all`, `just build`, `just
test`, `just parity`, `just parity-progress`, `just verify-redaction`, `just
verify-reference`, the live selector, no temporary instrumentation, `git diff
--check`, and full diff review.

## Stop conditions

Stop if the production-shaped replay tag cannot reproduce the consumed
failure, if no unambiguous closed origin boundary exists, if a correction would
admit arbitrary module tags or private values, or if the fix would change
normal boot/sensor behavior. Preserve the earliest typed blocker; never
reinterpret attempt-006 or authorize attempt-007 from this plan.

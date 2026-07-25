# Progress-Gated Hardware Attempt Policy

This policy governs every current and future repository hardware attempt. An
active `TASKS.md` hardware task and a repo-owned command must narrow these
rules for the specific device and objective; neither may weaken them.

## Closed outcomes

Every fresh attempt selects exactly one terminal outcome before any
continuation decision:

| Outcome                             | Meaning                                                                                                                                                                                                        |
| ----------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `continue_after_verified_fix`       | A diagnosed repository defect received one targeted fix, the real failing boundary has a regression, every required software gate passes, and a fresh preflight binds the new exact HEAD and package identity. |
| `continue_after_manual_remediation` | An authorized non-invasive remediation has objective evidence that the failed boundary changed.                                                                                                                |
| `complete`                          | The active task's genuine hardware success and evidence criteria are all satisfied.                                                                                                                            |
| `stop_repeated_boundary`            | The same authoritative boundary signature recurred once after its targeted fix was verified at the real boundary.                                                                                             |
| `stop_hardware_blocker`             | An unresolved hardware or physical-environment boundary prevents truthful progress.                                                                                                                            |
| `stop_authority_boundary`           | The next action exceeds the user's, repository's, or active task's authority.                                                                                                                                  |
| `stop_impossible_contract`          | The required evidence contract cannot be satisfied without contradiction or weakened truth.                                                                                                                    |

There is no fixed numeric attempt cap. There is also no unchanged blind retry:
an attempt may continue only after verified repository progress or an authorized
manual remediation with objective proof that the failed boundary changed. A
repeated post-fix boundary signature stops immediately. Completion is reserved for
real task success; unresolved hardware, authority, and impossible-contract
outcomes must remain explicit instead of being relabeled or weakened.

An authoritative boundary signature is a closed, redacted tuple declared by
the active task and produced by its typed diagnostic. It includes the terminal
category plus the minimum shareable discriminator fields needed to distinguish
where that category arose. It never includes raw identifiers, secrets, paths,
unbounded values, or free-form errors. A repeated coarse category with a newly
discriminating signature may return to diagnosis, but it authorizes no attempt
until its distinct cause has a real-boundary regression and verified fix. A
renamed category or discriminator that describes unchanged conditions is not a
new signature and cannot evade the repeated-boundary stop.

## Fresh-attempt contract

Every continuation is a new attempt and must satisfy all of these invariants:

- Use a fresh ordinal and run the full task-gated repo-owned hardware
  command exactly once.
- Pass all required software gates and a preflight against exact current `HEAD`.
  Freeze and revalidate the exact package identity before detector, credential,
  device, or other effectful work.
- Create one mode-`0700` protected parent. Leave the supervisor-owned evidence
  child nonexistent through the immediate pre-launch assertion.
- Capture wrapper stdout and stderr in distinct mode-`0600` sibling files under
  that parent, never beneath or through the intended child.
- Give the supervisor exclusive ownership of child creation and seal the
  attempt root immutable at its terminal disposition. Never reuse, retry in
  place, rewrite, or splice a sealed root.
- Preserve the earliest typed failure through restoration, cleanup, sealing,
  and reporting. Later failures may be recorded separately but never replace
  the first boundary.
- Record exactly one closed outcome before evaluating whether another fresh
  ordinal is allowed.

## Progress decisions

A verified-fix continuation requires all of the following: diagnosis of the authoritative boundary signature,
one targeted fix, a regression that crosses the real production boundary, all required software verification gates, and a
new exact-current-HEAD preflight whose source and package identities agree.
Tests at a mocked or in-process substitute do not replace a required
operating-system, runfiles, transport, or device boundary.

A manual-remediation continuation is limited to an action already authorized by
the active task contract and repository guidance. It must be non-invasive and must yield
objective evidence that the failed boundary changed. Repeating an instruction,
waiting without a measured transition, or retrying the same inputs is not
progress.

## Task and command ownership

The active `TASKS.md` hardware task and the invoked repo-owned command must both
encode:

- detector admission and target identity;
- the exact allowed effects and prohibited effects;
- device and operator safety limits;
- bounded recovery, restoration, and cleanup;
- private capture, redaction, evidence admission, and non-promotion rules; and
- deterministic software regressions plus required hardware verification.

Agent-selected fault testing is allowed only when both the active task contract
and the repo-owned command encode repo- and vendor-safe limits, automatic abort,
recovery, and required evidence. Electrical overstress is prohibited.

## Unchanged authority and evidence boundaries

This policy does not expand hardware authority. Direct UART, pins, pads,
headers, GPIO, test points, probes, jumpers, soldering, or injected signals
remain subject to the fresh explicit authorization rule in `AGENTS.md`.
Archived Phase 28.1.1 and its descendants remain terminal unresolved history,
and Phase 30 remains a conservative no-promotion boundary. No attempt may
reopen either lineage.

All capture, retention, terminal-output, Git, admission, privacy, and redaction
behavior remains governed by
[`docs/parity/evidence-policy.md`](../parity/evidence-policy.md). Hardware work
must not expose credentials, private endpoints, device identifiers, protected
operational material, or other prohibited values, and software or lifecycle
completion alone is never parity evidence.

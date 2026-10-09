# Archived lessons

Archived by lesson audits; each block keeps its original text.

## lesson-gsd-frontmatter-body-separators | 2026-06-28 14:14

1. Date: 2026-06-28
2. What went wrong: A GSD summary used standalone `---` body separators after YAML frontmatter. The GSD frontmatter parser scans all `--- ... ---` blocks and selected the last body pair, so lifecycle validation ignored the real frontmatter and failed.
3. Preventive rule: In GSD artifacts and other frontmatter-parsed Markdown, use standalone `---` only for the opening and closing YAML frontmatter delimiters at the top of the file. Use headings or `***` for body breaks instead. Markdown table separator rows such as `| --- |` remain valid.
4. Trigger signal to catch it earlier: Lifecycle validation reports missing frontmatter fields even though the file visibly has them near the top, or a Markdown artifact has more than two standalone `---` lines.

- Archive date: 2026-10-06
- Archive reason: Obsolete: GSD is sunset for this repository; AGENTS.md forbids creating GSD plans, phases or other frontmatter-parsed GSD artifacts, and `.planning/` is archive-only.
- Replacement ID: none

## lesson-cdc-commit-receipt-requires-live-control-state | 2026-09-04 17:46

1. Date: 2026-09-04
2. What went wrong: The native-USB maintenance protocol used DTR falling as its commit edge and only afterward emitted the committed receipt over CDC. Real macOS/TinyUSB hardware accepted readiness but timed out on the receipt because clearing DTR made the acknowledgment channel unreliable.
3. Preventive rule: When a control transition must be acknowledged over CDC, emit and observe the acknowledgment while DTR still represents a live connection. Use an exact post-readiness class-control transition for commit, then clear DTR and close only after the receipt arrives.
4. Trigger signal to catch it earlier: Hardware consistently receives the ready receipt but times out on committed while the Worker remains mounted, or firmware emits CDC evidence only after the host deasserts DTR.

- Archive date: 2026-10-06
- Archive reason: Obsolete: the TinyUSB CDC maintenance protocol (1200-baud arm, DTR commit edge) no longer exists in the firmware or host tools; USB Serial/JTAG is the sole controller under ADR-0021 and ADR-0023.
- Replacement ID: none

## lesson-visible-cdc-is-not-flash-admission | 2026-08-29 20:10

1. Date: 2026-08-29
2. What went wrong: Enabling the ESP32-S3 TinyUSB application profile produced a visible CDC node, but the existing flash path treated every serial node as an admitted ROM downloader and sent bootloader synchronization traffic to the Worker runtime.
3. Preventive rule: Treat physical identity, USB profile, and enumeration identity separately. Any application profile that owns the internal USB PHY must retain a tested, repo-owned handoff to ROM; require successful ROM `board-info` before writes and never infer flash compatibility from CDC visibility.
4. Trigger signal: TinyUSB descriptors, USB sdkconfig, or startup ownership changes while flash/recovery tests still identify targets only by a `/dev/cu.*` node or VID/PID and do not prove a profile transition plus ROM admission.

- Archive date: 2026-10-06
- Archive reason: Obsolete: the TinyUSB application CDC profile it describes no longer exists; USB Serial/JTAG is the sole controller under ADR-0021 and ADR-0023. Its lasting rule (ROM `board-info` before any write, profile-classified detection) is enforced by AGENTS.md "Autonomous Ultra 205 Hardware Verification" and `just detect-ultra205`.
- Replacement ID: none

## lesson-espflash-no-reset-is-not-passive | 2026-07-11 14:55

1. Date: 2026-07-11
2. What went wrong: The retained-runtime capture treated `espflash monitor --no-reset` as a passive serial open. In espflash 4.0.1 that flag suppresses the monitor's final application reset, but the default connection still drives reset lines, synchronizes with the bootloader, and may load the flasher stub.
3. Preventive rule: A passive ESP32-S3 monitor must use all three controls together: `--before no-reset-no-sync --after no-reset --no-reset`, with `--chip esp32s3`. Treat bare `--no-reset` as a reset-capable and bootloader-affecting command.
4. Trigger signal to catch it earlier: Any retained-runtime or no-flash capture renders `espflash monitor` with `--no-reset` but omits either explicit `--before no-reset-no-sync` or `--after no-reset`.

- Archive date: 2026-10-09
- Archive reason: Fully superseded: AGENTS.md "Ultra 205 Serial Session Reuse" requires the complete passive monitor contract (`--chip esp32s3 --before no-reset-no-sync --after no-reset --no-reset --non-interactive`) and names bare `--no-reset` reset-capable.
- Replacement ID: none; the replacement is the cited always-loaded AGENTS.md rule

## lesson-power-and-usb-session-are-distinct | 2026-07-11 14:55

1. Date: 2026-07-11
2. What went wrong: USB replug, barrel-power retention, both-power cold start, and warm reset were sometimes discussed as interchangeable recovery actions even though they preserve different MCU and USB-peripheral state.
3. Preventive rule: Record barrel/DC state and USB state independently, plus the USB enumeration epoch. Label every action as a USB re-enumeration, warm reset, or true both-power cold start; never infer one from another.
4. Trigger signal to catch it earlier: A hardware checkpoint says only `replug`, `power-cycle`, or `reset` without naming both power paths and the expected USB-session transition.

- Archive date: 2026-10-09
- Archive reason: Fully superseded: AGENTS.md "Ultra 205 Serial Session Reuse" requires recording barrel/DC and USB power independently and distinguishing re-enumeration, warm reset and both-power cold start in every checkpoint and trace.
- Replacement ID: none; the replacement is the cited always-loaded AGENTS.md rule

## lesson-boot-proof-replay-must-outlive-service-sessions | 2026-07-12 04:55

1. Date: 2026-07-12
2. What went wrong: The prearmed native-USB watcher acquired the correct node, held passive monitor ownership for the full capture, and cleaned up completely, but firmware emitted no replay markers. Source inspection showed replay was driven only from the live Stratum socket pump, so Wi-Fi or pool-session progress could prevent transport evidence from ever being replayed.
3. Preventive rule: Evidence needed to prove boot independently of external services must be scheduled by a boot-lifetime owner. Keep transport proof, boot proof, listener proof, and network/session proof as separate boundaries with separate failure categories.
4. Trigger signal to catch it earlier: A boot-evidence replay method is called only from a network, socket, pool, HTTP, ASIC-session, or other optional service loop, or a clean serial attachment captures zero bytes without an ownership failure.

- Archive date: 2026-10-09
- Archive reason: Fully superseded: AGENTS.md "Ultra 205 Serial Session Reuse" requires cold-boot replay to be owned by a boot-lifetime task, not Wi-Fi, HTTP, Stratum, pool or ASIC-session progress, and keeps transport and service readiness separate.
- Replacement ID: none; the replacement is the cited always-loaded AGENTS.md rule

## lesson-heartbeat-cannot-prove-over-silent-transport | 2026-07-12 14:03

1. Date: 2026-07-12
2. What went wrong: An always-on boot-lifetime heartbeat passed strict reflash/reinit capture, but the retained both-power cold-start capture was still exactly empty after successful native-USB appearance, stable passive ownership, and a full bounded session. Moving evidence production earlier and making it service-independent did not restore byte delivery through a late-attached USB Serial/JTAG transport.
3. Preventive rule: Treat node appearance, serial ownership, firmware evidence production, and observed byte delivery as four separate boundaries. A heartbeat can measure boot age only after the transport proves it carries application bytes; it cannot substitute for that transport proof.
4. Trigger signal to catch it earlier: Reflash capture contains periodic application heartbeats, but an exact-node late-attach capture has zero bytes despite stable identity, expected ownership, and complete cleanup.

- Archive date: 2026-10-09
- Archive reason: Fully superseded: AGENTS.md "Ultra 205 Serial Session Reuse" keeps node appearance, identity, ownership, heartbeat production and observed bytes as independent facts and forbids a zero-byte cold retry without a newly planned transport diagnostic.
- Replacement ID: none; the replacement is the cited always-loaded AGENTS.md rule

## lesson-manual-removal-needs-owner-observation | 2026-07-12 11:16

1. Date: 2026-07-12
2. What went wrong: A lifecycle accepted the operator's power-removal token before a persistent exact-node owner was watching for disappearance, so the token could attest intent while the transport transition itself remained unobserved.
3. Preventive rule: Start the lifecycle owner and exact-node removal watcher before publishing the removal action. Accept a manual response only after that owner records node disappearance after action publication, then require the complete bounded absence interval.
4. Trigger signal to catch it earlier: A hardware continuation starts its watcher inside `deliver`, or a token can advance state while the selected node is still present or has no owner-recorded disappearance timestamp.

- Archive date: 2026-10-09
- Archive reason: Fully superseded: AGENTS.md "Ultra 205 Serial Session Reuse" requires the exact-node owner and removal watcher before the instruction, token acceptance only after observed disappearance, and the full bounded absence interval.
- Replacement ID: none; the replacement is the cited always-loaded AGENTS.md rule

## lesson-physical-usb-identity-excludes-enumeration-fields | 2026-07-12 17:27

1. Date: 2026-07-12
2. What went wrong: A cold-restore gate required both a new enumeration epoch and equality of a supposed physical-USB identity digest. On macOS that digest included `IOCalloutDevice`, `IODialinDevice`, `IOTTYDevice`, `IOTTYBaseName`, and the IORegistry entry ID, so the required re-enumeration could change the value and trigger `appearance_identity_changed` before capture.
3. Preventive rule: Model stable physical identity and enumeration identity separately. A physical-identity digest may use stable hardware attributes such as USB serial number, vendor/product IDs, and stable port location, but must exclude tty paths/names, device-node metadata, and IORegistry entry IDs that are expected to change across enumeration.
4. Trigger signal to catch it earlier: A lifecycle simultaneously requires `new_enumeration_epoch=true` and equality of a digest that contains callout/dial-in device names, tty base names, device-node inode data, or a registry-entry identifier.

- Archive date: 2026-10-09
- Archive reason: Fully superseded: AGENTS.md "Ultra 205 Serial Session Reuse" separates physical from enumeration identity and lists the excluded macOS fields, tty paths, inodes, dynamic instances and registry-entry IDs.
- Replacement ID: none; the replacement is the cited always-loaded AGENTS.md rule

## lesson-cold-boot-proof-needs-an-independent-observer | 2026-07-12 16:17

1. Date: 2026-07-12
2. What went wrong: Native USB was used as the authoritative cold-start evidence channel even though the same board power transition removes that transport and recreates it only after early application output may already have occurred. Watcher timing, passive ownership, replay, and heartbeat repairs could prove their own boundaries but could not make the late-enumerated channel preserve original bytes.
3. Preventive rule: When evidence must span destruction and recreation of a device-owned transport, use an independently powered receive-only observer that remains enumerated and open across the transition. Establish a quiet byte boundary while the target is unpowered, validate only post-boundary bytes, and keep target identity separate from observer identity and ownership.
4. Trigger signal to catch it earlier: A test requires original boot bytes while its authoritative reader node disappears with target power or cannot be opened until after the target has begun booting.

- Archive date: 2026-10-09
- Archive reason: Superseded: AGENTS.md "Ultra 205 Serial Session Reuse" now validates cold start through the repo-owned session-tagged replay proof, routes any first-byte alternate channel through the direct-UART authorization rule, and states the identity rule for a future authorized observer. The lesson's direct recommendation of an independent observer conflicts with the dormant external-UART policy.
- Replacement ID: none; the replacement is the cited always-loaded AGENTS.md rule

## lesson-direct-uart-and-pin-access-requires-authorization | 2026-07-12 18:42

1. Date: 2026-07-12
2. What went wrong: The next hardware plan treated a direct external-UART fixture as acceptable after native-USB evidence remained blocked, even though the user had not agreed to wire UART or manipulate board pads and pins.
3. Preventive rule: Default to the device's provided USB and barrel-power interfaces. Do not propose, request, instruct, or perform direct UART, probe, pin, pad, header, GPIO, jumper, solder, or injected-signal work unless the user explicitly requests that path, or a permanent blocker is documented after non-invasive paths are exhausted; in either case, obtain fresh explicit user authorization before physical instructions or hardware contact.
4. Trigger signal to catch it earlier: A plan or next action mentions RX/GND wiring, test pads, Tag-Connect pins, probes, soldering, jumpers, GPIO manipulation, or an external UART adapter without a recorded explicit authorization checkpoint.

- Archive date: 2026-10-09
- Archive reason: Fully superseded by an equally strong, always-loaded rule: AGENTS.md "Direct UART And Pin-Manipulation Authorization" restates and extends this guardrail (default to barrel and USB, explicit request or documented permanent blocker, fresh authorization before instructions or contact).
- Replacement ID: none; the replacement is the cited always-loaded AGENTS.md rule

## lesson-protected-evidence-root-ownership | 2026-07-19 10:31

1. Date: 2026-07-19
2. What went wrong: A wrapper could pre-create the exact evidence child through output redirection, weakening the supervisor's exclusive creation and rejection boundary before admission or effects.
3. Preventive rule: Create one private parent, prove the supervisor-owned child is absent immediately before launch, and capture wrapper output in separately created private sibling files. The supervisor must reject any existing child before admission, discovery, sensitive-input access, or effects.
4. Trigger signal to catch it earlier: A caller redirects stdout or stderr beneath the requested child, creates the child on the supervisor's behalf, or launches without a fresh absence assertion.

- Archive date: 2026-10-09
- Archive reason: Fully superseded: AGENTS.md "Protected Evidence Root Ownership" states the private parent, absent child, sibling mode-0600 logs and supervisor rejection before admission, discovery, credential access or effects.
- Replacement ID: none; the replacement is the cited always-loaded AGENTS.md rule

## lesson-redact-after-private-classification | 2026-07-20 10:02

1. Date: 2026-07-20
2. What went wrong: Commit redaction transformed the same protected monitor artifact that the Boot A classifier still needed, so required private runtime-origin structure became invalid before the HTTP diagnostic boundary was reached.
3. Preventive rule: Remove `NeverPersistRaw` values before the first write, preserve the resulting mode-`0600` secret-sanitized input for private classification, and produce a distinct commit-redacted shareable copy; never run a lossy redactor in place before all authorized private classifiers have consumed their required fields.
4. Trigger signal to catch it earlier: A downstream classifier requires a sensitive structured field from an artifact that an upstream step also sanitizes, redacts, truncates, or rewrites for sharing.

- Archive date: 2026-10-09
- Archive reason: Fully superseded: AGENTS.md "Repository-Wide Evidence Privacy" and `docs/parity/evidence-policy.md` keep `NeverPersistRaw` values off disk and require private classifiers to consume an immutable secret-sanitized artifact before a distinct commit-redacted projection is derived.
- Replacement ID: none; the replacement is the cited always-loaded AGENTS.md rule

## lesson-hardware-retries-require-new-information | 2026-07-20 23:43

1. Date: 2026-07-20
2. What went wrong: Repeating a hardware attempt without a verified fix or objectively changed boundary consumed a fresh ordinal but added no information and could reproduce the same failure indefinitely.
3. Preventive rule: Permit another hardware attempt only after one targeted fix is verified across the real failing boundary or an authorized non-invasive remediation objectively proves that boundary changed; stop when the same redacted authoritative boundary signature recurs after its targeted verified fix. A repeated coarse category may return to diagnosis only when closed discriminator fields prove a distinct signature.
4. Trigger signal to catch it earlier: A proposed continuation changes only the attempt number, evidence root, category label, timing, or hope of success while the code, inputs, physical state, and measured boundary signature remain unchanged.

- Archive date: 2026-10-09
- Archive reason: Fully superseded: AGENTS.md "Progress-Gated Hardware Attempts" and `docs/hardware/hardware-attempt-policy.md` forbid unchanged blind retries, require verified progress or proved boundary change, and select the repeated-boundary stop on recurrence after a targeted fix.
- Replacement ID: none; the replacement is the cited always-loaded AGENTS.md rule

## lesson-standing-task-authorization-avoids-confirmation-churn | 2026-08-03 17:37

1. Date: 2026-08-03 17:37 CDT
2. What went wrong: Repository workflows repeatedly required the user to authorize each fresh hardware-attempt ordinal even though the project already had standing authorization to execute its active tasks, creating artificial terminal blockers after every targeted fix.
3. Preventive rule: Treat active repository tasks as standing-authorized for autonomous execution, including selecting fresh attempt ordinals after verified progress, when their exact command, safety, privacy, evidence, recovery, retry, and stop contracts are complete. Do not ask for per-attempt confirmation. Keep materially different direct-UART, pin-manipulation, and ad hoc destructive or fault-injection actions behind their specific safety gates.
4. Trigger signal to catch it earlier: The next safe action is fully described by an active task and repo-owned command, but work is about to stop solely because the task text says a later ordinal needs fresh user authorization.

- Archive date: 2026-10-09
- Archive reason: Fully superseded: AGENTS.md "Autonomous Ultra 205 Hardware Verification" and "Progress-Gated Hardware Attempts" grant standing authorization, forbid per-attempt or per-ordinal confirmation, and keep direct-UART, pin and destructive actions separately gated.
- Replacement ID: none; the replacement is the cited always-loaded AGENTS.md rule

## lesson-hardware-fixes-do-not-require-plan-per-iteration | 2026-08-28 19:59

1. Date: 2026-08-28 19:59 UTC
2. What went wrong: Hardware progress was slowed by treating each diagnosed code fix and fresh progress-backed retry as requiring another formal immutable plan even after the active task already defined the complete effect, safety, privacy, recovery, evidence, retry, and stop contract.
3. Preventive rule: Within a complete active hardware task contract, diagnose, fix, regression-test, verify, commit/push, select a fresh ordinal, and retry autonomously. Create a new plan only when an explicitly invoked skill requires one, authority or effects materially expand, the safety/recovery contract changes, or no active task covers the effect.
4. Trigger signal: The next action is a targeted fix or fresh ordinal for the same admitted hardware boundary, and work is about to pause solely to create another plan rather than because scope, authority, safety, recovery, or evidence requirements changed.

- Archive date: 2026-10-09
- Archive reason: Fully superseded: AGENTS.md "Iterative Hardware Fix Authorization" states the same autonomous fix-and-retry rule and the same four conditions that require a new plan.
- Replacement ID: none; the replacement is the cited always-loaded AGENTS.md rule

## lesson-time-bounded-physical-checkpoints-must-be-prearmed-and-self-describing | 2026-08-13 14:49

1. Date: 2026-08-13
2. What went wrong: A 30-second physical IDENTIFY effect was triggered before the operator had confirmed they were watching, and the emitted checkpoint said only `rendered` without describing the expected frame. A later normal-screen report was then incorrectly treated as evidence that the frame never rendered even though the effect had already expired. The follow-up fix still guessed that the operator would return within one hour and propagated that estimate into the campaign, fixture, and parent-process lifetimes, which is incompatible with asynchronous work that may pause for hours or overnight.
3. Preventive rule: Never time-bound a safe wait for human availability. Pre-arm a self-describing checkpoint and either keep an explicitly operator-gated owner live or release resources behind a typed resume path. Before any finite physical observation effect, consume readiness locally, then enforce only the effect's exact evidence window; retain independent bounds for automated safety, protocol, recovery, cleanup, and resource phases. Classify late observations as expired authority boundaries rather than positive or negative device evidence.
4. Trigger signal to catch it earlier: Human readiness or response latency appears as a numeric timeout, fixture duration, parent-process budget, or task deadline; a physical effect starts before local readiness; a checkpoint omits the expected state or finite effect window; or a late report is used to classify what was displayed during an expired window.

- Archive date: 2026-10-09
- Archive reason: Fully superseded: AGENTS.md "Asynchronous Human Checkpoints" forbids deadlines on human waits, requires pre-armed self-describing checkpoints or typed resume, keeps automated bounds, and classifies late reports as expired authority.
- Replacement ID: none; the replacement is the cited always-loaded AGENTS.md rule

## lesson-development-ip-needs-share-redaction-not-interactive-secrecy | 2026-08-30 16:17

1. Date: 2026-08-30 16:17 CDT
2. What went wrong: A recovery design added masked dialogs and special interactive secrecy for a local RFC1918 address even though the user treats development IPs as ordinary local diagnostics.
3. Preventive rule: Allow local development UI, console, and protected task artifacts to show RFC1918 addresses normally. Apply strict redaction when evidence is committed or shared, and keep credentials, public endpoints, tokens, and owner identifiers under their stronger existing protections.
4. Trigger signal to catch it earlier: A workflow proposes hidden input, secret storage, or credential-grade handling solely because an operator must enter a private development IP address.

- Archive date: 2026-10-09
- Archive reason: Fully superseded: AGENTS.md "Autonomous Ultra 205 Hardware Verification" lets local development evidence and displays keep IP addresses and requires redaction only for committed or shared evidence, with credentials kept under stronger protection.
- Replacement ID: none; the replacement is the cited always-loaded AGENTS.md rule

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

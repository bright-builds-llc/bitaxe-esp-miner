# ADR-0030: Standing authorization for development core dumps

Accepted 2026-09-27 by explicit owner instruction. This decision applies to
current and future development, without per-dump or per-session confirmation.

## Decision

Agents may collect, persist, retain and inspect full core dumps while developing
and diagnosing this repository, including task stacks, registers and memory
captured by the platform's supported core-dump facility. Incidental credentials
or other normally `NeverPersistRaw` values inside a development dump do not block
private capture or analysis. This is an explicit exception for the dump and its
private derived analysis, not an authorization to export unrelated credentials.

Use mode-0600 files beneath mode-0700 repository-ignored development roots for
raw dumps, extracted ELF/memory files, debugger output and temporary files.
On-device core-dump storage is also authorized. Preserve original dumps and
record image/tool identities so later analysis remains reproducible. Agents may
inspect these artifacts locally; ordinary terminal summaries and reports use
redacted diagnostic facts rather than arbitrary memory contents.

Committing, publishing, uploading or sharing raw dumps or unredacted analysis
requires separate explicit authorization. Existing redaction verification for
shareable evidence remains unchanged. A protected raw dump is never itself an
admitted parity projection.

This decision supersedes the development raw-memory-dump prohibition in ADR-0026
and the conflicting private-persistence restrictions in the evidence policy.
It is the authoritative standing development rule, rather than a temporary
exception tied to STR-005 or a particular failed attempt.

## Independent effect and evidence controls

Capture permission does not imply permission to erase existing evidence, alter
credentials, weaken safety deadlines, start mining, issue grants, flash/reset a
device, attach an additional owner or use direct pins. Such effects retain their
repo task/command contracts. Read-flash acquisition is reset-capable and must use
the existing physical-device lease, ROM admission and cleanup protocol.

Historical failure records remain immutable. A new diagnostic capture cannot fill
missing fields in a sealed attempt or retroactively establish resource release.
The unresolved STR-005 historical-resource prerequisite and parity status remain
unchanged by this privacy authorization.

## Implementation direction

Prefer ESP-IDF's flash ELF core dump, its checksum verification and official
offline decoder over a custom native panic handler. Bind each capture to the
matching full ELF SHA-256, retain debug information and the link map, and reject
missing/truncated identity notes. Capture the supported internal DRAM and task
state; document external-memory exclusions and finite partition/task limits
instead of calling a bounded dump a complete physical-memory image.

Preserve the first dump until it has been collected; clearing the core-dump
partition is a separate explicit operation. Keep normal USB control ownership
and console routing intact. A diagnostic candidate must pass native build,
resource and package checks before any separately admitted installation.

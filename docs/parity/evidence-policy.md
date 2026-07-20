# Evidence Data and Artifact Policy

This policy governs repository evidence, local evidence capture, terminal output,
Git history, and promoted parity artifacts. It applies to contributors, agents,
repository commands, CI, and every phase-specific evidence wrapper.

## Data classes

Every evidence value belongs to exactly one class:

| Class | Examples | Allowed sinks |
| --- | --- | --- |
| `NeverPersistRaw` | Passwords, tokens, credential contents, pool URLs, pool ports, pool users, workers, owner addresses, pool passwords, and NVS secrets | Memory only for the minimum authorized operation. Never disk, terminal, Git, or promoted evidence. |
| `ProtectedOperational` | SSIDs, IP and MAC addresses, hostnames, device origins, USB identities and paths, PIDs, process paths, unredacted commands, settings, HTTP material, and detailed logs | Mode-`0600` files below a mode-`0700` repository-ignored root. Never terminal, Git, or admitted evidence. |
| `ShareableFact` | Typed categories, booleans, bounded counts and durations, status classes, board categories, and outcomes | Terminal, Git, and admitted evidence after validation. |
| `PublicProvenance` | Source and reference commits plus safely opaque package, artifact, and evidence-root digests | Terminal, Git, and admitted evidence. A digest of a low-entropy sensitive value is still sensitive and is not public provenance. |

Redaction is not permission to persist a `NeverPersistRaw` value first.
Processes must remove that class before the first write or terminal emission.
Private evidence may retain `ProtectedOperational` values only after
`NeverPersistRaw` sanitization.

## Artifact lifecycle

An evidence root moves through the following closed lifecycle:

1. `ActivePrivate`: an owned mode-`0700` ignored root containing only mode-`0600`
   files, still available to authorized private classifiers.
2. `SealedNonPromotion` or `SealedEligible`: immutable disposition after the
   attempt ends. A non-promotional root is never reused or spliced into another
   attempt.
3. `AdmittedProjection`: an optional, separately derived projection containing
   only `ShareableFact` and `PublicProvenance`.
4. `ExplicitlyPurged`: an explicit artifact-deletion action authorized by the
   owning workflow.

Process termination, descriptor closure, serial-holder cleanup, restoration, or
other resource cleanup does not delete evidence and does not imply
`ExplicitlyPurged`. Sealed private roots remain protected and retained until an
explicit policy-owned retention or purge decision applies.

## Capture and derivation order

Child stdout and stderr are captured through distinct pipe identities and passed
through independent, incremental, bounded `NeverPersistRaw` line state before
any disk write. Raw child bytes never reach inherited stdout or stderr and never
reach a file; partial lines from different streams are never joined.

When a private classifier needs operational structure, the workflow must:

1. write one immutable, secret-sanitized classifier input below the protected
   root;
2. close it and record its opaque digest;
3. run authorized private classification against that file;
4. derive a distinct commit-redacted projection without rewriting the private
   input; and
5. prove the private digest did not change.

Lossy shareable redaction must never run in place before private classification.
For dual flash evidence, `flash-monitor` performs only steps 1 and 2 and writes
only the private log and private record. After the authorized classifier passes,
the software-only `finalize-evidence` command verifies the classified digest,
creates the distinct admitted log and record, and rechecks the private digest.
Classifier failure therefore produces no admitted projection.

## Admission and sinks

Only `ShareableFact` and `PublicProvenance` may enter:

- inherited stdout or stderr;
- Git staged changes or commits;
- `docs/parity/evidence/`; or
- another admitted or promoted evidence tree.

Private paths, operational identifiers, detailed commands, raw settings, HTTP
material, and detailed logs stay out of admitted projections. Evidence
admission must fail closed before mutation or promotion when classification,
sanitization, derivation, permissions, immutability, or cleanup cannot be
proved.

## Repository redaction guard

`just verify-redaction` is the single repository adapter. It applies
`NeverPersistRaw` rules to every changed destination blob, applies
`ProtectedOperational` rules only to committed shareable, documentation, and
admission sinks, scans the complete tracked admitted evidence tree, and either:

- the staged snapshot, by default; or
- the changed snapshot at an explicit `--base` and `--head`, in CI.

The command never prints matched content. Findings contain only a stable rule
ID, category, path, and line number. CI and local use share the same script.

Reviewed exceptions live only in `scripts/redaction-exceptions.tsv`. Each entry
has a stable ID, exact category, exact repository-relative path, non-empty
reason, and optional ISO expiry date. Wildcards, inline suppressions, command
line bypasses, and environment-variable bypasses are forbidden. Exceptions
Exceptions apply only to unchanged tracked files during the complete admitted
baseline scan. Staged, changed, base/head, and new-branch destination blobs never
receive an exception. An all-zero push base means a new branch, so every blob at
the destination `HEAD` is scanned; malformed ordinary revisions fail closed.

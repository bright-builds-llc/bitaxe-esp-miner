# Verified-row revisions

Automatic transitions out of `verified` are forbidden. To refine a verified
row's notes, the owner records an authority here: a JSON file in schema
`parity-verified-revision-authority-v1`. It lists:
- the committed evidence documents, each digest-bound;
- for each row, the evidence tokens to add (`added_evidence`) and the SHA-256
  of the exact new notes (`after_notes_sha256`).

`added_evidence` may be empty for a passage-only revision of a row that
already holds every evidence token it needs. Its evidence cell must then stay
exactly unchanged; the revision changes only the notes.

A transition binds it with
`transition-item ... --to verified --verified-revision-authority <file>`.
The receipt records the file's digest. `just parity` re-checks the authority
and its evidence digests on every replay, so editing either fails closed.

The parity tool
(`tools/parity/src/checklist_revision/transitions/verified_revision.rs`)
accepts only the pinned, additive revision:
- the status stays `verified`;
- the owned target is unchanged;
- the evidence gains exactly the granted tokens, or stays unchanged when
  none are granted;
- the new notes keep the old notes verbatim, add a non-empty, separated,
  printable suffix, and match the granted digest. Identical notes are
  refused, so every revision changes something.

Because the notes digest is pinned, each row's grant applies once.

`authorized_by` is a record, not a check: anyone can write an authority file.
The control is that a revision can only add text that was pinned in a
reviewed commit. That text should state which non-claims it narrows, and
that the remaining ones still apply.

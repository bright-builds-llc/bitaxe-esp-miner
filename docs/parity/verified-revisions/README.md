# Verified-row revisions

Automatic transitions out of `verified` are forbidden. To refine a verified
row's notes, the owner records an authority here: a JSON file in schema
`parity-verified-revision-authority-v1`. It lists:
- the committed evidence documents, each digest-bound;
- for each row, the evidence tokens to add and the SHA-256 of the exact new
  notes.

A transition binds it with
`transition-item ... --to verified --verified-revision-authority <file>`.
The receipt records the file's digest. `just parity` re-checks the authority
and its evidence digests on every replay, so editing either fails closed.

The parity tool
(`tools/parity/src/checklist_revision/transitions/verified_revision.rs`)
accepts only the pinned, additive revision:
- the status stays `verified`;
- the owned target is unchanged;
- the evidence gains exactly the granted tokens;
- the new notes keep the old notes verbatim, add a separated printable
  suffix, and match the granted digest.

Because the notes digest is pinned, each row's grant applies once.

`authorized_by` is a record, not a check: anyone can write an authority file.
The control is that a revision can only add text that was pinned in a
reviewed commit. That text should state which non-claims it narrows, and
that the remaining ones still apply.

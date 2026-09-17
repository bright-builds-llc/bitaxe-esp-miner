# STR-005 Channel003 installation-review failure

Channel003 remains **unverified**. Its initial state-preserving installation
completed and the trusted boot capture reported healthy fixed-Serial startup.
The host rejected the installation evidence because the output directory and
files inherited permissions broader than the protected-evidence policy.

The tested package was firmware `0d2b6d061edc7aab8957c17764ce77ee82f313c1`,
Gate `e20c0fd52d2216596f904992ffa54fda33be9025`, ELF SHA-256
`8e766fbadca0678e4db534acb067e7621d7fed29e1a749ce9d806c275559d178`.
Fresh authenticated possession and initial accounting were collected on the
previously installed firmware before the write. No fresh authenticated
post-installation preservation, accounting or restoration baseline was
collected. The boot capture does not replace those missing observations.

## Sealed disposition

The canonical finalizer and independent read-only reviewer agree:

| Fact | Value |
| --- | --- |
| Context SHA-256 | `031552692b777c324b5633dfb5b1cbbc4625da4cdea1adedcd8c1c3ee0cbd1b1` |
| Result SHA-256 | `cf219da3a6b6830f1818f03eeb7e3d88b0b912d589a99e6a4a1f8b8b0a02ad54` |
| Inventory SHA-256 | `5941810323f06e9bdf86d79bc8c9f4aed6ce3a2f105716153e68b101e7f79bf3` |
| Outcome | `stop_evidence_incomplete` |
| First recorded failure | Supervisor `v2_operation_failed` |
| Separate parent diagnosis | Installation review rejected `private_path_policy` |

The parent recorded the original permissions and file digests before restricting
the directory to `0700` and the files to `0600`. That containment changed no
artifact bytes, created no successful installation review and did not repair the
failed qualification. The original supervisor failure remains unchanged.

The browser released serial ownership and its owned tab was closed. The parent
observed the supervisor exit with code zero. A separate cleanup-state failure
remains recorded; no complete passing cleanup receipt was manufactured.
Launch observations distinguish captured child output from a parent-derived
readiness-line normalization. All raw output is preserved privately.

## Accounting and remaining proof

There were no continuity cycles, fixture connection, V2 Channel Start, issued or
loaded grant, reservation, work, renewal, passive observer or heartbeat fault.
The initial authenticated ledger reported next ordinal 18, last completed 17,
1,560,000 ms charged and no pending reservation. This is a **pre-installation**
observation; a successor must independently confirm current accounting.

The single completed write does not establish four-cycle continuity, an accepted
channel, an accepted ASIC share or complete restoration. A fresh attempt requires
a verified correction, published guarded admission, fresh possession and both
accounting baselines before any new write. No failed context may be restarted.
STR-005 remains unverified and active parity remains **90/95**.

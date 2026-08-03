# Phase 33 Confirmed Settings Durability Evidence

This directory preserves the historical Phase 33 summary. Active settings-durability
classification is exposed as `just verify-settings-durability` and accepts only canonical
flags. Complete detector, flash, HTTP, serial, identity, process, and holder traces remain
under the gitignored `scratch/` root with directory mode `0700` and file mode `0600`.

The proof is intentionally narrow:

- one detector-approved Ultra 205;
- the just-built canonical package flashed after the sole detector preflight;
- one exact hostname PATCH confirmed by immediate system-info readback;
- one normal application restart observed through the complete passive ESP32-S3 monitor contract;
- the same physical USB identity, one fresh same-session origin, post-reboot digest equality, complete monitor/holder cleanup, and restoration of the original hostname without another reboot.

The tracked summary contains only commit and trace digests, categories, counts, durations, booleans, and a non-promotional conclusion. It must not contain raw hostnames, origins, addresses, USB identities, device paths, process identifiers, SSIDs, credentials, endpoints, workers, secrets, or commands containing sensitive input.

Run the typed automation tests first:

```bash
bazel test //tools/automation:automation_test
```

Classify protected trace segments with the semantic command:

```bash
just verify-settings-durability --trace scratch/settings/trace.log --mode baseline
just verify-settings-durability --trace scratch/settings/trace.log --mode delivery --start-byte 1
just verify-settings-durability --trace scratch/settings/trace.log --mode post-restart --start-byte 1 --expected-session session-1 --expected-ordinal 1
```

The protected trace remains a local input and is never copied into committed evidence.

A failed detector, package flash, origin, identity, restart, readback, cleanup, timeout,
redaction, or restoration gate leaves CFG-12 pending. Legacy Phase 33 evidence is not
accepted by active consumers after the typed automation cutover.

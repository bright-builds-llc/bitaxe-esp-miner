# Phase 33 Confirmed Settings Durability Evidence

This directory accepts only the redacted summary emitted by `just phase33-settings-durability`. Complete detector, flash, HTTP, serial, identity, process, and holder traces remain under the gitignored `scratch/` root with directory mode `0700` and file mode `0600`.

The proof is intentionally narrow:

- one detector-approved Ultra 205;
- the just-built canonical package flashed after the sole detector preflight;
- one exact hostname PATCH confirmed by immediate system-info readback;
- one normal application restart observed through the complete passive ESP32-S3 monitor contract;
- the same physical USB identity, one fresh same-session origin, post-reboot digest equality, complete monitor/holder cleanup, and restoration of the original hostname without another reboot.

The tracked summary contains only commit and trace digests, categories, counts, durations, booleans, and a non-promotional conclusion. It must not contain raw hostnames, origins, addresses, USB identities, device paths, process identifiers, SSIDs, credentials, endpoints, workers, secrets, or commands containing sensitive input.

Run the software simulation first:

```bash
bash scripts/phase33-confirmed-settings-durability-test.sh
bazel test //scripts:phase33_confirmed_settings_durability_test
```

After the ordered Rust, Bazel, build, package, and reference-clean gates pass, run the hardware proof with a wall-clock allowance greater than seven minutes:

```bash
just phase33-settings-durability --capture-seconds 360 --wifi-credentials wifi-credentials.json
```

The credential argument is a path-only local input. The wrapper never reads or prints its contents. Omit it when the device already has usable NVS Wi-Fi settings.

A failed detector, package flash, origin, identity, restart, readback, cleanup, timeout, redaction, or restoration gate leaves CFG-12 pending and blocks Phase 33 completion. This evidence does not change parity status and does not perform Phase 35 admission.

# ADR-0036: Measured identity, pool-persistence and credential-absence facts for BWG-007

Accepted 2026-10-09 under `task-bwg007-real-worker-restoration`, by the owner's
approval of the remaining BWG-007 items. It amends
[ADR-0035](0035-serial-bwg-restoration-campaign.md) and replaces one mechanism
of [ADR-0019](0019-supervise-bwg-restoration-through-a-protected-browser-campaign.md).

## Context

Attempt-008 passed all eight scenarios and published projection profile
`bwg-worker-restoration-result/0.2`. Three of BWG-007's obligations were still
not measured in that profile:

- **Same device and same-key reacquisition.** ADR-0019 had the harness read
  the adapter's challenge-scoped fingerprint record after possession, send it
  to a loopback owner and store it in the protected attempt. Publication then
  compared that fingerprint digest across scenarios. The serial campaign
  (ADR-0035) has no such loopback fingerprint path. Profile 0.2 wrote
  `sameDeviceAcrossScenarios: true` as a constant.
- **Pool configuration never persisted.** Signed grants carry the pool
  endpoint and credentials to the device. Nothing showed that the device left
  its stored pool configuration untouched.
- **Credential absence.** Serve refused any record that echoed a secret it
  held. Nothing scanned the sealed attempt as a whole, and profile 0.2 wrote
  `campaignEventCredentialsAbsent: true` as a constant.

## Decision

1. **Page-local identity tracker instead of the loopback fingerprint.** Every
   Worker status already carries `preservation.device_identity_sha256`, and the
   Gate requires it to equal the possession-verified key digest. In
   restoration mode the Gate page counts the distinct digests over its lifetime
   (`epoch`, expected 1) and all observations, and publishes only
   `deviceIdentity: null | {epoch, observations}`. A new identity after the
   first journals `device_identity_changed`. No raw digest leaves the page, so
   nothing reaches the host or the attempt root. One page lifetime covers the
   whole attempt, so epoch 1 throughout means one key across all eight
   scenarios. The host judge requires:
   - `deviceIdentityStable` in every scenario: from the first observation on,
     every recorded state and the final state show epoch 1, with
     non-decreasing observations;
   - `sameKeyReacquired` in `disconnect` and `reboot`: after the
     post-reconnect terminal status, a state shows epoch 1 with more
     observations than when the physical window began.

2. **Device-local pool-configuration boolean.** Once at boot, the firmware
   takes a SHA-256 over the canonical stored pool configuration (primary and
   fallback settings keys) and keeps it in RAM. Status preservation becomes
   `worker-preservation-v2`, which is v1 plus
   `pool_configuration_unchanged_since_boot`. Only the boolean leaves the
   device, never a digest of pool values. The Gate accepts v1 and v2 everywhere
   it parses preservation. On the restoration page it publishes
   `poolConfiguration: null | {observations, changed}`: only v2 observations
   count, and `changed` stays true after any `false`, which journals
   `pool_configuration_changed`. In every scenario, the host judge requires
   `poolConfigurationUnchanged`: no state from the Start on shows `changed`,
   and a state after the lease ended shows more observations than at the
   Start.

3. **Seal-time credential scan.** `finish` requires `--pool-credentials`, the
   same ignored mode-0600 file serve used, and refuses without it. Before
   sealing, it scans every file in the attempt root for two things:
   - the exact pool endpoint, host, user and password, raw and JSON-escaped,
     for values of at least 8 characters;
   - credential shapes: a compact JWS, a base64url run of 40 or more
     characters mixing upper case, lower case and digits (never a lower-case
     hex digest), and `challenge_` or `lease_` identifiers with a random
     suffix.

   The pool values stay in memory. Serve's verdict moves to
   `campaign-result.json`; finish seals it as `result.json` with
   `credential_scan: {files, hits}`. A hit makes a passed campaign `unverified`
   with `credential_in_evidence`. An earlier campaign failure keeps precedence.
   `just hardware-operator owner-finish --owner bwg-restoration` passes the
   flag through after checking the file's mode, without reading it.

4. **Projection 0.3.** Publish writes `bwg-worker-restoration-result/0.3`.
   Every boolean is derived:
   - `baselineConfirmed` and `cleanupConfirmed` from the judge's own checks;
   - `campaignEventCredentialsAbsent` from a scan with files and zero hits;
   - `sameDeviceAcrossScenarios` from `deviceIdentityStable` in all eight
     scenarios and epoch 1 in the last final state;
   - `poolConfigurationUnchangedPerBoot`, a new boolean, from
     `poolConfigurationUnchanged`.

   Every scenario's facts gain `deviceIdentityStable` and
   `poolConfigurationUnchanged`; `disconnect` and `reboot` also gain
   `sameKeyReacquired`. The 0.2 validator stays, so the published attempt-008
   files still validate. They keep their original meaning, including the two
   constant booleans.

## Consequences

- A new attempt (009 onward) passes only with a Gate that publishes both
  trackers and firmware that reports preservation v2. With an older Gate or
  firmware it fails these facts closed.
- Short pool values (under 8 characters, such as a password `x`) cannot be
  matched meaningfully. The shape rules and serve's per-record refusal still
  apply. The scan proves absence only in the attempt root, not in parent-level
  serve or finish logs.
- The shape rules are heuristics. A dry run over the 103 files of attempts
  001–008 found no false positives. A future false positive would make an
  attempt `unverified` rather than publish a leak.
- Finish cannot prove the supplied pool file is the one serve used, because
  no pool digest is persisted. The operator supplies the same ignored file.

## Non-claims

The identity tracker shows that one possession-verified key answered
throughout one page lifetime. It is not a physical identity: the presence
watcher's USB identity covers that. `changed: false` reflects only the device's
own comparison against its boot snapshot; it does not cover changes made before
that boot. The scan proves absence of the listed values and shapes only. This
ADR verifies nothing on hardware and promotes no parity row.

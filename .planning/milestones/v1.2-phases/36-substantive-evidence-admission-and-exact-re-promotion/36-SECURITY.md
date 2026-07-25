---
phase: 36
phase_name: Substantive Evidence Admission and Exact Re-Promotion
reviewed_commit: f1cb6101f2c384acaffe0b8523097433ff0f04cc
standard: OWASP ASVS L1
asvs_level: 1
block_on: high
status: passed
threats_total: 6
threats_closed: 6
threats_open: 0
critical_findings: 0
high_findings: 0
medium_findings: 0
low_findings: 0
reviewed_on: 2026-07-24
---

# Phase 36 Security Audit

## Result

Phase 36 passes the OWASP ASVS Level 1 review at commit
`f1cb6101f2c384acaffe0b8523097433ff0f04cc`. All six registered threats are
closed by implementation evidence and fresh offline verification. No accepted
or transferred risks apply, and the Phase 36 summaries contain no unregistered
threat flags.

| Severity | Findings |
| --- | ---: |
| Critical | 0 |
| High | 0 |
| Medium | 0 |
| Low | 0 |

## Threat Verification

| Threat ID | Category | Disposition | Status | Evidence |
| --- | --- | --- | --- | --- |
| T36-01 | Protected-value disclosure | mitigate | CLOSED | Protected inputs require an owned mode-`0700` root and owned mode-`0600` regular files, are opened relative to retained descriptors with `O_NOFOLLOW`, and emit closed error categories (`tools/parity/src/protected_input.rs:39-135`, `tools/parity/src/main.rs:831-861`). Public generations contain typed facts and safe provenance only; the repository data-class policy limits public sinks to `ShareableFact` and `PublicProvenance` (`docs/parity/evidence-policy.md:11-21`, `docs/parity/evidence-policy.md:66-77`). Canary/error-rendering tests and `just verify-redaction` passed. |
| T36-02 | Evidence rewriting, splicing, or mixed evidence | mitigate | CLOSED | The classifier checks the exact ordered role set, safe relative paths, artifact digests, a single evidence source commit, the authenticated Phase 35 root, and the committed Phase 35 generation (`tools/parity/src/phase36_evidence.rs:132-165`, `tools/parity/src/phase36_evidence.rs:241-344`). Every admitted descriptor is reverified before classification completes (`tools/parity/src/phase36_evidence.rs:73-89`, `tools/parity/src/phase36_evidence.rs:228-238`). The offline successor preserves unrelated rows and the Phase 35 hostname claim while producing explicit corrections for unsupported claims. Mutation, role-swap, root-drift, generation-drift, and byte-identity tests passed. |
| T36-03 | Supervisor self-attestation or generic-root authority | mitigate | CLOSED | Production authority is anchored to the committed Phase 35 root and generation but deliberately contains no caller role-digest authority; caller-authored companion trees therefore fail closed (`tools/parity/src/phase36_evidence.rs:194-225`, `tools/parity/src/phase36_evidence.rs:346-359`). Runtime identity is derived by replaying the event ledger through `SessionState::apply`, then comparing private and public terminal results and the exact package join (`tools/parity/src/phase36_evidence/runtime_identity.rs:93-139`, `tools/parity/src/phase36_evidence/runtime_identity.rs:141-241`). Independent effect admission requires a complete, ordered, closed ledger owned by `IndependentObserver`; the supervisor boolean has no authority (`tools/parity/src/phase36_evidence/effects.rs:117-185`). |
| T36-04 | Effect-ledger broker bypass | mitigate | CLOSED | Phase 36 adds no effect-capable broker. Its three commands return before workspace/environment detection and can reach only the evidence classifier, independent-effect classifier, and offline evaluator (`tools/parity/src/main.rs:758-822`, `tools/parity/src/main.rs:831-861`). The production effect module has no process-command surface, requires all eight ordered effect records, rejects prohibited categories, and rejects unledgered paths (`tools/parity/src/phase36_evidence/effects.rs:31-86`, `tools/parity/src/phase36_evidence/effects.rs:117-185`). The real-process structural guard passed. |
| T36-05 | Partial or divergent checklist/generation publication | mitigate | CLOSED | Publication recovers the derived checklist before reading current authority, renders and validates the complete generation, writes and syncs all owned files, validates the exact staged inventory and manifest fingerprints, and only then exchanges it (`tools/parity/src/operator_evidence/generation/phase36.rs:147-177`, `tools/parity/src/operator_evidence/generation/phase36.rs:375-459`). The transaction exchanges the authority generation first, rolls back failures, and repairs a stale derived checklist from the validated authoritative snapshot after a crash (`tools/parity/src/operator_evidence/generation/phase36/transaction.rs:16-117`). All injected publication boundaries and the real child-process crash-recovery test passed. |
| T36-06 | Unauthorized hardware fallback | mitigate | CLOSED | The Attempt 31 evaluator revalidates protected descriptors, evaluates the immutable evidence, and publishes only the software-derived generation; caller companions conservatively contribute no authority (`tools/parity/src/phase36_offline.rs:100-164`, `tools/parity/src/phase36_offline.rs:229-266`). Phase 36 dispatch is isolated before environment detection (`tools/parity/src/main.rs:758-822`). Structural tests reject detector, credential, flash, monitor, serial, network mutation, hardware-run, and archived Phase 28.1.1 tokens. This audit invoked no USB, serial, hardware, device discovery, credentials, or network operations. |

## Required Boundary Gates

- Protected evidence authority is fail-closed. The production classifier
  authenticates Phase 35 authority but does not permit supplied Phase 36
  companions to manufacture the missing role authority.
- Protected path handling is descriptor-relative. Every ancestor and leaf uses
  `openat` with `O_NOFOLLOW`; identity includes device, inode, owner, mode, and
  length; contents are read from the retained descriptor and rehashed before
  use (`tools/parity/src/protected_input.rs:98-188`).
- Evaluator identity is complete and path-sensitive. The inventory includes
  `tools/device-session/src/model.rs`, and each path and source body is
  independently length-delimited before hashing
  (`tools/parity/src/phase36_evidence.rs:786-885`). Bazel supplies the model
  through `//tools/device-session:phase36_runtime_identity_sources`
  (`tools/device-session/BUILD.bazel:3-7`,
  `tools/parity/BUILD.bazel:113-128`).
- Publication is crash-recoverable. The generation snapshot is authoritative;
  checklist repair occurs before authority-aware reads, and ordinary injected
  failures roll back both paths.
- Untrusted fixtures are synthetic, closed-schema inputs. Phase 36 tests cover
  unknown fields, missing roles, mutation, path replacement, symlink ancestors,
  self-attestation, incomplete ledgers, and protected-value canaries.
- Temporary test roots are isolated with `mktemp`, mode-`0700` directories,
  mode-`0600` files, and `umask 077`. Concurrent generation tests add process,
  time, and monotonic sequence identity to workspace names.
- Canonical firmware packaging declares and passes the ESP-IDF sdkconfig,
  bootloader, partition table, and OTA data image as explicit Bazel inputs
  (`firmware/bitaxe/BUILD.bazel:221`,
  `scripts/package-firmware.sh:301-354`). The package fixture gate passed.
- Phase 36 does not expand hardware, credential, direct-UART, pin-manipulation,
  network, or archived-lineage authority. Its truthful result is one preserved
  hostname claim and explicit insufficiency for package identity, operator
  snapshot substance, runtime health, and independent effect observation.

## Fresh Verification

All commands ran at the reviewed commit without hardware, USB, serial, device
discovery, credentials, or network access.

| Gate | Result |
| --- | --- |
| `git rev-parse HEAD` | PASS — exact reviewed commit |
| `CARGO_NET_OFFLINE=true cargo test -p bitaxe-parity phase36 --all-features` | PASS — 76 Phase 36-focused tests |
| `CARGO_NET_OFFLINE=true cargo test -p bitaxe-device-session --all-features` | PASS — 16 unit and 2 CLI tests |
| Scoped Bazel tests for Phase 36 evidence, promotion, generation, real-process guard, and firmware packaging | PASS — 5/5 targets |
| `just parity` | PASS — `validation_errors: none` |
| `just verify-reference` | PASS — pinned reference `c1915b0a63bfabebdb95a515cedfee05146c1d50` clean |
| `just verify-redaction` | PASS |

## Unregistered Flags

None. The Phase 36 summaries contain no `## Threat Flags` entries.

## Residual Non-Claims

This result verifies the declared Phase 36 mitigations and software-only
publication behavior. It does not promote unsupported hardware claims, grant
authority to caller-authored companions, authorize a new device attempt, or
verify any credential, hardware-control, mining, OTA/recovery, non-205,
direct-UART, pin, or archived Phase 28.1.1 behavior.

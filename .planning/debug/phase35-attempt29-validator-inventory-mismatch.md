---
status: resolved
trigger: "Investigate and document Phase 35 Attempt 29 validator failure. Symptoms: device-session reached ready and all hardware/restoration/cleanup steps passed, but the Phase 35 Rust validator rejected the generated evidence as InventoryMismatch. Local typed diagnosis has isolated a semantic digest mismatch on role boot_a_api: scripts/phase35-correlated-evidence-document.sh used `jq -er` to write embedded document strings, adding a newline, while the Rust validator hashes the exact string bytes. A proposed uncommitted fix changes these writes to `jq -jer` and strengthens the fixture validator to compare all six boot document artifacts against embedded document bytes; focused uncached supervisor test passes."
created: 2026-07-23T03:25:52Z
updated: 2026-07-23T03:30:13Z
---

## Current Focus

hypothesis: Confirmed. The shell writer appended jq's output newline to each boot document artifact, while the Rust validator hashed the exact embedded document string bytes.
test: Confirmed by source trace, one-variable byte/digest counterfactual, all-six-role fixture review, uncached supervisor regression, shell validation, and the mandatory Rust verification sequence.
expecting: Satisfied. Removing jq's framing newline makes every generated boot document artifact byte-identical to its embedded document and therefore restores semantic inventory digest equality.
next_action: No further debugging action. Preserve the resolved record and use the verified source changes as repository progress; do not repeat Attempt 29 unchanged.

## Symptoms

expected: After the Phase 35 device session reaches ready and restoration and cleanup complete, the Rust validator should accept the generated evidence inventory because each boot document artifact matches the embedded document bytes used to construct its digest.
actual: The device session reached ready and all hardware, restoration, and cleanup steps passed, but final Rust validation rejected the generated evidence inventory.
errors: `InventoryMismatch`, isolated by local typed diagnosis to semantic digest mismatch on inventory role `boot_a_api`.
reproduction: Generate Phase 35 correlated evidence with the Attempt 29 writer path, then run the Phase 35 Rust validator against that generated evidence inventory.
started: Observed in Phase 35 hardware Attempt 29; earlier history is not asserted by this session.

## Eliminated

## Evidence

- timestamp: 2026-07-23T03:25:52Z
  checked: Active lessons and repository guidance relevant to Phase 35 evidence.
  found: The repository requires exact protected-evidence ownership, privacy-preserving classification, earliest typed failure precedence, regression-backed progress before another hardware attempt, and no blind hardware retry.
  implication: This investigation must remain software-only and prove the serialization boundary before any future attempt can be considered.

- timestamp: 2026-07-23T03:26:17Z
  checked: Current uncommitted diff for the assigned producer and fixture paths.
  found: The proposed change alters only three embedded-document writes from `jq -er` to `jq -jer`, and adds a fixture helper invoked for the API, WebSocket, and retained-log documents in both boot epochs.
  implication: The proposed production change is narrowly scoped to output framing, while the regression targets all six artifacts created by the shared writer.

- timestamp: 2026-07-23T03:27:01Z
  checked: Complete producer and Rust inventory-validation paths.
  found: The shell builds each inventory digest from the artifact file bytes, so the newline-terminated file is internally consistent with its inventory entry. Rust then reads those exact bytes and separately computes the expected semantic digest from `input.boot_a.system_info_document.as_bytes()`; validation iterates in role order and `boot_a_api` is the first embedded-document role.
  implication: The observed role and error are the deterministic first manifestation of a representation mismatch shared by all six boot document artifacts, not an inventory-order, path, cleanup, or hardware failure.

- timestamp: 2026-07-23T03:27:40Z
  checked: Synthetic jq output-framing counterfactual using a 23-byte embedded document.
  found: `jq -er` emitted 24 bytes ending in hex `0a` and its SHA-256 did not match the exact string; `jq -jer` emitted 23 bytes ending in the document's final byte and its SHA-256 matched exactly.
  implication: The proposed `-j` flag removes precisely the causal byte and changes no document content, directly confirming the hypothesized mechanism.

- timestamp: 2026-07-23T03:29:16Z
  checked: Uncached focused supervisor regression, `bazel test //scripts:phase35_correlated_evidence_test --cache_test_results=no`.
  found: Bazel executed the target rather than serving a cached result; the full target passed in 46.6 seconds, including the success path whose validator runs after restoration and cleanup.
  implication: The proposed writer and fixture changes are compatible with the complete Phase 35 supervisor fixture suite and close the reproduced software boundary without a hardware retry.

- timestamp: 2026-07-23T03:30:13Z
  checked: Snapshot producer contract and explicit simplification review.
  found: The live snapshot producer constructs the API and WebSocket documents with no terminal newline and strips terminal newlines from the retained-log input through command substitution. The shared artifact writer is therefore the sole source of the extra byte. Adding `-j` to the three shared jq writes is the smallest causal production change; no Rust validator relaxation or inventory-schema change is needed.
  implication: The fix preserves the fail-closed validator contract and corrects the producer representation at its source.

- timestamp: 2026-07-23T03:30:13Z
  checked: Regression sufficiency and changed-path checks.
  found: The fixture iterates both `boot_a` and `boot_b` and compares the generated artifact digest with the exact embedded bytes for API, WebSocket, and retained-log fields, covering all six affected roles. `git diff --check`, `bash -n`, and ShellCheck passed for the touched shell paths. The orchestrator also reported that the mandatory Rust format, Clippy, all-target build, and all-feature test sequence passed.
  implication: The regression is sufficient for the shared defect class and the proposed changes pass the required repository verification surfaces.

## Resolution

root_cause: `write_epoch_artifacts` used `jq -er`, whose raw-output mode still terminates each result with a newline. `build_inventory` correctly hashed those newline-terminated files, but Rust `expected_role_digest` hashes the exact deserialized embedded strings with `as_bytes()`. The first affected ordered role, `boot_a_api`, therefore had an internally valid file/inventory digest that differed from its semantic document digest and deterministically produced `InventoryMismatch`.
fix: Change the three shared embedded-document writes in `scripts/phase35-correlated-evidence-document.sh` from `jq -er` to `jq -jer`, suppressing only jq's output separator. Strengthen `scripts/phase35-correlated-evidence-fixture.sh` so the success validator compares the generated artifact with the embedded bytes for API, WebSocket, and retained-log documents in both boot epochs.
verification: A synthetic 23-byte document proved the legacy writer emitted 24 bytes ending in `0a` with a mismatched digest, while the fixed writer emitted 23 byte-identical bytes with a matching digest. The uncached `//scripts:phase35_correlated_evidence_test` passed in 46.6 seconds. Changed-path whitespace, Bash syntax, and ShellCheck passed, and the orchestrator confirmed the mandatory Rust verification sequence passed.
files_changed:
  - scripts/phase35-correlated-evidence-document.sh
  - scripts/phase35-correlated-evidence-fixture.sh

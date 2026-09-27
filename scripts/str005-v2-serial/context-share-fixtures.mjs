import { readFile } from "node:fs/promises";
import { resolve } from "node:path";
import { sha256 } from "./values.mjs";
import { ledger, original } from "../str005-noise-serial/test-fixture.mjs";

/** Explicit synthetic readiness seam for current-context tests, never accepted by production inspection. */
export async function shareSuccessorFixture(base, parent, previous, operations) {
  const root = resolve(parent, "share-001"), receiptPath = `${root}.share-successor-readiness.json`;
  const context = { schema: "str005-v2-serial-context-v4", scope: "share", hostOrdinal: 1,
    firmware_commit: "f".repeat(40), app_elf_sha256: "f".repeat(64), manifest_sha256: "e".repeat(64),
    evaluator: [{ path: "synthetic-old-source", sha256: "e".repeat(64), length: 1 }], original_campaign_id: previous.context.original_campaign_id,
    predecessor: { root: resolve(parent, "channel-004"), resultSha256: "e".repeat(64), sealSha256: "f".repeat(64) } };
  const hash = sha256(JSON.stringify(context));
  await base.put(resolve(root, "context.json"), JSON.stringify({ context, sha256: hash }));
  await base.put(resolve(root, "final-result.json"), JSON.stringify({ synthetic: "unverified Share001" }));
  await base.put(resolve(root, "accounting-before-install.json"), JSON.stringify({ observedSequence: 4, ledger, original_budget: original }));
  const inputs = await Promise.all(["context.json", "final-result.json", "accounting-before-install.json"].map(async path => {
    const bytes = await readFile(resolve(root, path)); return { path, sha256: sha256(bytes), length: bytes.length };
  }));
  await base.put(resolve(root, "sealed-inventory.json"), JSON.stringify({ files: inputs }));
  const marker = { schema: "str005-v2-serial-assignment-v1", root, scope: "share", context_sha256: hash };
  await base.put(resolve(parent, "qualification-ordinal-18.json"), JSON.stringify(marker));
  await base.put(resolve(parent, "share-ordinal-1.json"), JSON.stringify(marker));
  const markerBytes = await readFile(resolve(parent, "qualification-ordinal-18.json"));
  const checker = await readFile(resolve(base.options.firmwareRoot, "scripts/str005-v2-serial/context.mjs"));
  const value = { schema: "str005-v2-share-successor-readiness-v1", failedRoot: root, failedContextSha256: hash,
    failedResultSha256: sha256(await readFile(resolve(root, "final-result.json"))), failedSealSha256: sha256(await readFile(resolve(root, "sealed-inventory.json"))),
    status: "unverified", classification: "ready_for_fresh_channel", hardwareQualified: false, historicalCleanupComplete: false, afterBaselineObserved: false,
    beforeSource: { firmware_commit: context.firmware_commit, app_elf_sha256: context.app_elf_sha256 }, predecessor: context.predecessor,
    initialAccounting: { path: "accounting-before-install.json", sha256: inputs[2].sha256, observedSequence: 4, ledger, original },
    ordinalMarker: { path: "qualification-ordinal-18.json", sha256: sha256(markerBytes), length: markerBytes.length }, inspectedInputs: inputs,
    checkerIdentity: { firmwareCommit: "a".repeat(40), sources: [{ path: "scripts/str005-v2-serial/context.mjs", sha256: sha256(checker), length: checker.length }] } };
  await base.put(receiptPath, JSON.stringify(value));
  const receiptSha256 = sha256(await readFile(receiptPath));
  operations.inspectShareSuccessor = async path => {
    if (path !== receiptPath || sha256(await readFile(path)) !== receiptSha256) throw Object.assign(Error("readiness changed"), { code: "v2_share_successor_pin_changed" });
    for (const input of inputs) if (sha256(await readFile(resolve(root, input.path))) !== input.sha256)
      throw Object.assign(Error("failed bytes changed"), { code: "v2_share_successor_pin_changed" });
    return { ...structuredClone(value), root, context, contextSha256: hash, resultSha256: value.failedResultSha256,
      sealSha256: value.failedSealSha256, receiptPath, receiptSha256 };
  };
  operations.checkCurrentShareSuccessorOwnership = async () => {};
  return receiptPath;
}

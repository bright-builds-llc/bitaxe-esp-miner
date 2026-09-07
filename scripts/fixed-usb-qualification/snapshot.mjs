import { readFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { BUNDLE, contextPage, digest, exactObject, fileDigest, hex, inspectPackage, protectedPath,
  readJson, requireCondition, within } from "./contract.mjs";

export async function verifyArtifactSnapshot(root, context) {
  const page = contextPage(context);
  const receiptPath = resolve(root, "artifact-snapshot.json");
  await protectedPath(receiptPath);
  const receipt = await readJson(receiptPath);
  exactObject(receipt, ["schema", "context_sha256", "files"]);
  requireCondition(receipt.schema === "fixed-usb-qualified-artifacts-v1" &&
    receipt.context_sha256 === digest(JSON.stringify(context)) && Array.isArray(receipt.files) &&
    receipt.files.length === 13, "snapshot_receipt");
  const snapshotRoot = resolve(root, "qualified-artifacts");
  await protectedPath(snapshotRoot, true);
  const observed = new Set();
  for (const entry of receipt.files) {
    exactObject(entry, ["path", "sha256", "length"]);
    requireCondition(typeof entry.path === "string" && hex(entry.sha256, 64) &&
      Number.isSafeInteger(entry.length) && entry.length > 0 && !observed.has(entry.path), "snapshot_file_shape");
    const path = within(snapshotRoot, resolve(snapshotRoot, entry.path));
    let parent = dirname(path);
    while (parent !== snapshotRoot) { await protectedPath(parent, true); parent = dirname(parent); }
    await protectedPath(path);
    const bytes = await readFile(path);
    requireCondition(bytes.length === entry.length && digest(bytes) === entry.sha256, "snapshot_file_integrity");
    observed.add(entry.path);
  }
  const manifestPath = resolve(snapshotRoot, "firmware/bitaxe-ultra205-package.json");
  const manifest = await readJson(manifestPath);
  const expected = ["firmware/bitaxe-ultra205-package.json", "firmware/docs/release/license-inventory.md",
    "firmware/docs/release/provenance-manifest.md", `gate/${BUNDLE}`, `gate/${page}`,
    ...manifest.artifacts.map((artifact) => `firmware/${artifact.path}`)];
  requireCondition(new Set(expected).size === 13 && expected.every((path) => observed.has(path)), "snapshot_file_set");
  const packaged = await inspectPackage(manifestPath, context.firmware_commit, () => resolve(snapshotRoot, "firmware"));
  for (const [key, value] of Object.entries(packaged)) {
    requireCondition(JSON.stringify(context[key]) === JSON.stringify(value), "snapshot_package_drift");
  }
  const gateRoot = resolve(snapshotRoot, "gate");
  requireCondition(await fileDigest(resolve(gateRoot, BUNDLE)) === context.gate_bundle_sha256 &&
    await fileDigest(resolve(gateRoot, page)) === context.gate_page_sha256, "snapshot_gate_drift");
  return { receipt_sha256: await fileDigest(receiptPath), gate_root: gateRoot };
}
